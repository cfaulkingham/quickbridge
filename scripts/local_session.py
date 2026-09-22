#!/usr/bin/python3
"""Local Quick Bridge session for a machine with no Rust toolchain.

Serves the same phone pages as the Rust helper, on this computer's
network name, with the 6-digit code always on. The panel draws the QR
from the same JSON events. A visible terminal allows one firewall port
and removes it when this process exits.

--loopback is for tests: listen on 127.0.0.1 and do not touch the firewall.
"""

from __future__ import annotations

import ctypes
import errno
import fcntl
import hmac
import html
import ipaddress
import json
import mimetypes
import os
import pwd
import re
import secrets
import socket
import stat
import subprocess
import sys
import tempfile
import threading
import time
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from urllib.parse import parse_qs

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import qrcodegen
LANGS = ("en", "es", "fr", "de", "zh", "hi", "ja", "ko")
HTML_LANG = {
    "en": "en",
    "es": "es",
    "fr": "fr",
    "de": "de",
    "zh": "zh-Hans",
    "hi": "hi",
    "ja": "ja",
    "ko": "ko",
}
SENSITIVE = {".ssh", ".gnupg", ".pki", ".gpg", ".password-store"}
BAD_EXT = {
    "desktop",
    "lnk",
    "url",
    "executable",
    "appimage",
    "com",
    "pif",
    "scf",
    "search-ms",
}
MIN_FILE = 1024 * 1024
MAX_FILE = 2048 * 1024 * 1024
MIN_IDLE = 60
MAX_IDLE = 120 * 60
MAX_FILES = 32
MAX_FAILURES = 12
QUIET = 4
CSP_PAGE = (
    "default-src 'none'; script-src 'unsafe-inline'; style-src 'unsafe-inline'; "
    "img-src 'none'; connect-src 'self'; form-action 'self'; base-uri 'none'; "
    "frame-ancestors 'none'"
)
CSP_DOWNLOAD = CSP_PAGE.replace("img-src 'none'", "img-src 'self'")
PREVIEW_MAX = 16 * 1024
PART_OVERHEAD = 64 * 1024
HOST_RE = re.compile(
    r"^[a-z0-9](?:[a-z0-9-]{0,61}[a-z0-9])?\.local$"
)
IFACE_RE = re.compile(r"^[A-Za-z0-9._-]{1,15}$")
TOKEN_RE = re.compile(r"^[0-9a-f]{32}$")
ROW_FIELD = re.compile(r'\b(key|en|es|fr|de|zh|hi|ja|ko): "((?:\\.|[^"\\])*)"')

CATALOG = None
LANG = "en"
STOP = threading.Event()
EXPECTED_STOP = False
SERVER = None
HOLD_FD = None
STATE = None


class LocalError(Exception):
    pass


def emit(event):
    line = json.dumps(event, ensure_ascii=False, separators=(",", ":"))
    sys.stdout.write(line + "\n")
    sys.stdout.flush()


def cap_text(value, n):
    out = []
    for ch in str(value):
        code = ord(ch)
        if ch not in (" ", "-") and (code < 32 or 127 <= code < 160):
            continue
        out.append(ch)
        if len(out) >= n:
            break
    return "".join(out)


def status(state, message, progress=None):
    event = {
        "event": "status",
        "state": cap_text(state, 32),
        "message": cap_text(message, 160),
    }
    if progress is not None:
        event["progress"] = max(0.0, min(1.0, float(progress)))
    emit(event)


def error(message):
    emit({"event": "error", "message": cap_text(message, 160)})


def die(message):
    error(message)
    raise SystemExit(1)


def unescape(value):
    out = []
    i = 0
    while i < len(value):
        if value[i] == "\\" and i + 1 < len(value):
            nxt = value[i + 1]
            out.append({"n": "\n", "t": "\t", '"': '"', "\\": "\\"}.get(nxt, nxt))
            i += 2
            continue
        out.append(value[i])
        i += 1
    return "".join(out)


def load_catalog(path):
    with open(path, encoding="utf-8") as handle:
        text = handle.read()
    rows = {}
    for block in text.split("Row {"):
        fields = {name: unescape(raw) for name, raw in ROW_FIELD.findall(block)}
        key = fields.get("key")
        if not key or "en" not in fields:
            continue
        rows[key] = fields
    if "up_title" not in rows or "gate_title" not in rows:
        raise LocalError("phone catalog is incomplete")
    return rows


def language(raw):
    primary = (raw or "").strip().lower().split(".")[0].split("_")[0].split("-")[0]
    if primary in ("zh", "cmn"):
        return "zh"
    if primary in LANGS:
        return primary
    return "en"


def t(key):
    row = CATALOG.get(key) if CATALOG else None
    if not row:
        return key
    value = row.get(LANG) or ""
    return value or row.get("en") or key


def fmt(key, pairs):
    template = t(key)
    out = []
    i = 0
    while i < len(template):
        if template[i] == "{":
            end = template.find("}", i + 1)
            if end != -1 and template[i + 1 : end] in pairs:
                out.append(str(pairs[template[i + 1 : end]]))
                i = end + 1
                continue
        out.append(template[i])
        i += 1
    return "".join(out)


def fill(template, pairs):
    out = []
    rest = template
    while rest:
        found = None
        for token, value in pairs:
            if not token:
                continue
            at = rest.find(token)
            if at != -1 and (found is None or at < found[0]):
                found = (at, token, value)
        if found is None:
            out.append(rest)
            break
        at, token, value = found
        out.append(rest[:at])
        out.append(value)
        rest = rest[at + len(token) :]
    return "".join(out)


def html_lang():
    return HTML_LANG.get(LANG, "en")


def format_bytes(n):
    n = int(n)
    if n >= 1024 ** 3:
        return f"{n / 1024 ** 3:.1f} GB"
    if n >= 1024 ** 2:
        return f"{n / 1024 ** 2:.1f} MB"
    if n >= 1024:
        return f"{n / 1024:.1f} KB"
    return f"{n} B"


def html_escape(value):
    return html.escape(str(value), quote=True)


def ct_eq(left, right):
    a = left.encode() if isinstance(left, str) else left
    b = right.encode() if isinstance(right, str) else right
    if len(a) != len(b):
        hmac.compare_digest(b, b)
        return False
    return hmac.compare_digest(a, b)


def sanitize_filename(raw):
    name = str(raw or "").replace("\\", "/").rsplit("/", 1)[-1].strip().rstrip(". ")
    out = []
    for ch in name:
        if ord(ch) < 32 or (127 <= ord(ch) < 160) or ch in '/\\<>:"|?*&':
            continue
        if (ch == "." or ch == "-") and not out:
            continue
        out.append(ch)
        if len("".join(out)) >= 180:
            break
    cleaned = "".join(out).strip().rstrip(". ")
    if cleaned in ("", ".", ".."):
        return "upload.bin"
    ext = cleaned.rsplit(".", 1)[-1].lower() if "." in cleaned else ""
    if ext in BAD_EXT:
        stem = cleaned.rsplit(".", 1)[0] or "upload"
        return stem + ".bin"
    return cleaned


def euid():
    return os.geteuid()


def home_dir():
    return pwd.getpwuid(euid()).pw_dir


def _component_ok(name):
    if not name or name in (".", "..") or "/" in name or "\x00" in name:
        return False
    if len(name.encode()) > 255:
        return False
    if name in SENSITIVE:
        return False
    return True


def open_anchor(path):
    fd = os.open(path, os.O_RDONLY | os.O_DIRECTORY | os.O_CLOEXEC | os.O_NONBLOCK)
    st = os.fstat(fd)
    if not stat.S_ISDIR(st.st_mode) or st.st_uid != euid():
        os.close(fd)
        raise LocalError(t("dest_root_owner"))
    return fd


def walk_dest(anchor, parts):
    """Take ownership of anchor and return (leaf_fd, path)."""
    fd = anchor
    try:
        for index, name in enumerate(parts):
            if not _component_ok(name):
                raise LocalError(t("dest_forbidden"))
            leaf = index + 1 == len(parts)
            flags = os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW | os.O_CLOEXEC | os.O_NONBLOCK
            try:
                nxt = os.open(name, flags, dir_fd=fd)
            except FileNotFoundError:
                try:
                    os.mkdir(name, 0o700 if leaf else 0o755, dir_fd=fd)
                except FileExistsError:
                    pass
                try:
                    nxt = os.open(name, flags, dir_fd=fd)
                except OSError as exc:
                    if exc.errno in (errno.ELOOP, errno.ENOTDIR):
                        raise LocalError(t("dest_symlink")) from exc
                    raise
            except OSError as exc:
                if exc.errno in (errno.ELOOP, errno.ENOTDIR):
                    raise LocalError(t("dest_symlink")) from exc
                raise
            os.close(fd)
            fd = nxt
            st = os.fstat(fd)
            if not stat.S_ISDIR(st.st_mode):
                raise LocalError(t("dest_not_dir"))
            if st.st_uid != euid():
                raise LocalError(t("dest_owner"))
            if leaf:
                os.fchmod(fd, 0o700)
        path = os.readlink(f"/proc/self/fd/{fd}")
        for part in path.split("/"):
            if part in SENSITIVE:
                raise LocalError(t("dest_forbidden"))
        return fd, path
    except BaseException:
        os.close(fd)
        raise


def prepare_dest(user_path):
    home = home_dir()
    anchor = open_anchor(home)
    try:
        real_home = os.readlink(f"/proc/self/fd/{anchor}")
        if user_path:
            if not user_path.startswith("/") or "\x00" in user_path:
                raise LocalError(t("dest_absolute"))
            parts = [p for p in user_path.split("/") if p not in ("", ".")]
            if any(p == ".." for p in parts):
                raise LocalError(t("dest_bad_path"))
            prefix = None
            for base in (real_home, home):
                base_parts = [p for p in base.split("/") if p]
                if parts[: len(base_parts)] == base_parts and len(parts) > len(base_parts):
                    prefix = parts[len(base_parts) :]
                    break
            if prefix is None:
                raise LocalError(t("dest_home"))
            if len(prefix) > 16:
                raise LocalError(t("dest_deep"))
            rel_parts = prefix
        else:
            downloads = os.environ.get("XDG_DOWNLOAD_DIR") or os.path.join(home, "Downloads")
            if not downloads.startswith(home) and not downloads.startswith(real_home):
                downloads = os.path.join(home, "Downloads")
            base = real_home if downloads.startswith(real_home) else home
            rel = os.path.relpath(downloads, base)
            rel_parts = [p for p in rel.split("/") if p not in ("", ".")]
            if any(p == ".." for p in rel_parts):
                raise LocalError(t("dest_bad_path"))
            rel_parts.append("Quick Bridge")
    except BaseException:
        os.close(anchor)
        raise
    return walk_dest(anchor, rel_parts)


def child_exists(dirfd, name):
    try:
        st = os.lstat(name, dir_fd=dirfd)
    except FileNotFoundError:
        return False
    return True


def unique_name(dirfd, name):
    if not child_exists(dirfd, name):
        return name
    stem, dot, ext = name.rpartition(".")
    if not dot:
        stem, ext = name, None
    for i in range(1, 10000):
        candidate = f"{stem}-{i}.{ext}" if ext else f"{stem}-{i}"
        if not child_exists(dirfd, candidate):
            return candidate
    return f"{stem}-{secrets.token_hex(8)}.bin"


def rename_noreplace(dirfd, old, new):
    libc = ctypes.CDLL(None, use_errno=True)
    libc.renameat2.argtypes = [
        ctypes.c_int,
        ctypes.c_char_p,
        ctypes.c_int,
        ctypes.c_char_p,
        ctypes.c_uint,
    ]
    libc.renameat2.restype = ctypes.c_int
    rc = libc.renameat2(
        dirfd,
        old.encode(),
        dirfd,
        new.encode(),
        1,  # RENAME_NOREPLACE
    )
    if rc == 0:
        return True
    err = ctypes.get_errno()
    if err == errno.EEXIST:
        return False
    raise OSError(err, "renameat2")


def open_share(path, max_bytes, ephemeral):
    if not path or not path.startswith("/") or "\x00" in path:
        raise LocalError(t("path_absolute"))
    name = sanitize_filename(os.path.basename(path))
    fd = os.open(path, os.O_RDONLY | os.O_NOFOLLOW | os.O_NONBLOCK | os.O_CLOEXEC)
    try:
        st = os.fstat(fd)
        if not stat.S_ISREG(st.st_mode):
            raise LocalError(t("not_regular"))
        if st.st_uid != euid():
            raise LocalError(t("not_owner"))
        if st.st_nlink != 1:
            raise LocalError(t("hard_link"))
        if st.st_size == 0:
            raise LocalError(t("file_empty"))
        if st.st_size > max_bytes:
            raise LocalError(fmt("file_larger", {"size": format_bytes(max_bytes)}))
        flags = fcntl.fcntl(fd, fcntl.F_GETFL)
        fcntl.fcntl(fd, fcntl.F_SETFL, flags & ~os.O_NONBLOCK)
        real = os.readlink(f"/proc/self/fd/{fd}")
        for part in real.split("/"):
            if part in SENSITIVE:
                raise LocalError(t("dest_forbidden"))
        if ephemeral:
            try:
                os.unlink(path)
            except OSError:
                pass
        mime = mime_for(name, fd, st.st_size)
        return {"fd": fd, "name": name, "size": st.st_size, "mime": mime, "path": real}
    except BaseException:
        os.close(fd)
        raise


def mime_for(name, fd, size):
    sample = b""
    if size:
        sample = os.pread(fd, min(64, size), 0)
    if sample.startswith(b"\x89PNG\r\n\x1a\n"):
        return "image/png"
    if sample.startswith(b"\xff\xd8\xff"):
        return "image/jpeg"
    if sample.startswith(b"GIF87a") or sample.startswith(b"GIF89a"):
        return "image/gif"
    if sample.startswith(b"RIFF") and sample[8:12] == b"WEBP":
        return "image/webp"
    guessed, _ = mimetypes.guess_type(name)
    if guessed == "image/svg+xml":
        return "application/octet-stream"
    return guessed or "application/octet-stream"


def content_disposition(name, inline):
    kind = "inline" if inline else "attachment"
    ascii_name = []
    for ch in name:
        if ch.isascii() and (ch.isalnum() or ch in ".-_"):
            ascii_name.append(ch)
        else:
            ascii_name.append("_")
        if len(ascii_name) >= 180:
            break
    cleaned = "".join(ascii_name) or "download.bin"
    return f'{kind}; filename="{cleaned}"'


def qr_matrix(payload):
    code = qrcodegen.QrCode.encode_text(payload, qrcodegen.QrCode.Ecc.MEDIUM)
    width = code.get_size()
    size = width + QUIET * 2
    if size > 200:
        raise LocalError(t("qr_too_large"))
    rows = []
    for y in range(size):
        chars = []
        for x in range(size):
            dark = (
                QUIET <= x < QUIET + width
                and QUIET <= y < QUIET + width
                and code.get_module(x - QUIET, y - QUIET)
            )
            chars.append("1" if dark else "0")
        rows.append("".join(chars))
    return rows


def network_name():
    host = socket.gethostname().split(".")[0].lower()
    if not re.fullmatch(r"[a-z0-9](?:[a-z0-9-]{0,61}[a-z0-9])?", host):
        raise LocalError(t("lan_bad_name"))
    return host


def private_v4(text):
    try:
        addr = ipaddress.ip_address(text)
    except ValueError:
        return False
    if addr.version != 4:
        return False
    parts = addr.packed
    if parts[0] == 10:
        return True
    if parts[0] == 172 and 16 <= parts[1] <= 31:
        return True
    if parts[0] == 192 and parts[1] == 168:
        return True
    return False


def iface_from_route(text):
    parts = text.split()
    if "dev" not in parts or "src" not in parts:
        return None
    iface = parts[parts.index("dev") + 1]
    src = parts[parts.index("src") + 1]
    if not IFACE_RE.fullmatch(iface) or iface == "lo" or iface.startswith(("-", ".")):
        return None
    if not private_v4(src):
        return None
    return iface, src


def default_iface():
    try:
        out = subprocess.run(
            ["/usr/bin/ip", "-4", "route", "get", "1.1.1.1"],
            check=False,
            capture_output=True,
            timeout=3,
            env={"PATH": "/usr/bin", "LANG": "C"},
        )
    except (OSError, subprocess.TimeoutExpired) as exc:
        raise LocalError(t("lan_no_iface")) from exc
    text = out.stdout.decode("utf-8", "replace")
    found = iface_from_route(text)
    if found is None or not os.path.isdir(os.path.join("/sys/class/net", found[0])):
        raise LocalError(t("lan_no_iface"))
    return found[0]


def runtime_dir():
    runtime = os.environ.get("XDG_RUNTIME_DIR") or ""
    if not runtime.startswith("/") or "\x00" in runtime:
        raise LocalError(t("lan_bind_failed"))
    st = os.lstat(runtime)
    if not stat.S_ISDIR(st.st_mode) or st.st_uid != euid():
        raise LocalError(t("lan_bind_failed"))
    return runtime


def open_port_window(iface, port):
    """Start the visible firewall terminal. Return (hold_fd, life_fd)."""
    global HOLD_FD
    runtime = runtime_dir()
    folder = tempfile.mkdtemp(prefix="quickbridge-lan.", dir=runtime)
    os.chmod(folder, 0o700)
    hold_path = os.path.join(folder, "hold")
    life_path = os.path.join(folder, "life")
    os.mkfifo(hold_path, 0o600)
    os.mkfifo(life_path, 0o600)
    hold = os.open(hold_path, os.O_RDWR | os.O_CLOEXEC)
    life = os.open(life_path, os.O_RDONLY | os.O_NONBLOCK | os.O_CLOEXEC)
    HOLD_FD = hold
    script = os.path.join(ROOT, "scripts", "lan-port")
    env = {
        "PATH": "/usr/bin",
        "HOME": os.environ.get("HOME", home_dir()),
        "USER": os.environ.get("USER", ""),
        "LANG": os.environ.get("LANG", "C.UTF-8"),
        "QUICKBRIDGE_LANG": LANG,
        "XDG_RUNTIME_DIR": runtime,
        "WAYLAND_DISPLAY": os.environ.get("WAYLAND_DISPLAY", ""),
        "DBUS_SESSION_BUS_ADDRESS": os.environ.get("DBUS_SESSION_BUS_ADDRESS", ""),
        "XDG_CURRENT_DESKTOP": os.environ.get("XDG_CURRENT_DESKTOP", ""),
        "XDG_SESSION_TYPE": os.environ.get("XDG_SESSION_TYPE", ""),
        "DISPLAY": os.environ.get("DISPLAY", ""),
    }
    subprocess.Popen(
        [
            "/usr/bin/omarchy-launch-floating-terminal-with-presentation",
            script,
            iface,
            str(port),
            folder,
        ],
        stdin=subprocess.DEVNULL,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
        env=env,
        cwd=home_dir(),
        start_new_session=True,
    )
    marker = os.path.join(folder, "open")
    deadline = time.monotonic() + 180
    announced = False
    while time.monotonic() < deadline:
        if STOP.is_set():
            raise LocalError(t("lan_firewall_timeout"))
        if os.path.isfile(marker) and not os.path.islink(marker):
            flags = fcntl.fcntl(life, fcntl.F_GETFL)
            fcntl.fcntl(life, fcntl.F_SETFL, flags & ~os.O_NONBLOCK)
            return hold, life
        if not announced:
            status("connecting", fmt("lan_waiting", {"port": str(port), "iface": iface}), 0.35)
            announced = True
        time.sleep(0.25)
    raise LocalError(t("lan_firewall_timeout"))


def watch_window(life_fd):
    try:
        while True:
            chunk = os.read(life_fd, 64)
            if chunk == b"":
                break
    except OSError:
        pass
    finally:
        if not EXPECTED_STOP:
            error(t("lan_window_closed"))
        begin_stop()


def begin_stop():
    global EXPECTED_STOP
    EXPECTED_STOP = True
    STOP.set()
    server = SERVER
    if server is not None:
        threading.Thread(target=server.shutdown, daemon=True).start()


def release_hold():
    global HOLD_FD
    fd = HOLD_FD
    HOLD_FD = None
    if fd is not None:
        os.close(fd)


class Session:
    def __init__(self, mode, token, pin, dest, share, max_bytes, idle, stop_after, origin, host_value):
        self.mode = mode
        self.token = token
        self.pin = pin
        self.cookie = secrets.token_hex(16)
        self.failures = 0
        self.lock = threading.Lock()
        self.dest = dest
        self.share = share
        self.max_bytes = max_bytes
        self.max_session = min(max_bytes * 8, 8 * 1024 * 1024 * 1024)
        self.max_files = 1 if stop_after else MAX_FILES
        self.used_bytes = 0
        self.used_files = 0
        self.idle = idle
        self.stop_after = stop_after
        self.origin = origin
        self.host_value = host_value
        self.started = time.monotonic()
        self.last = time.monotonic()
        self.slots = threading.BoundedSemaphore(8)

    def touch(self):
        self.last = time.monotonic()

    def allowed(self, token):
        return TOKEN_RE.fullmatch(token or "") and ct_eq(token, self.token)

    def cookie_ok(self, header):
        offered = ""
        for part in (header or "").split(";"):
            name, _, value = part.strip().partition("=")
            if name == "qb":
                offered = value
                break
        return ct_eq(offered, self.cookie)

    def check_pin(self, offered):
        with self.lock:
            if self.failures >= MAX_FAILURES:
                return "locked"
            if ct_eq((offered or "").strip(), self.pin):
                return "ok"
            self.failures += 1
            return "wrong"

    def reserve(self, nbytes):
        with self.lock:
            if self.used_files >= self.max_files:
                raise LocalError(t("session_files"))
            if self.used_bytes >= self.max_session:
                raise LocalError(t("session_size"))
            room = min(self.max_bytes, self.max_session - self.used_bytes)
            if nbytes > room:
                raise LocalError(fmt("file_larger", {"size": format_bytes(room)}))
            self.used_files += 1
            self.used_bytes += nbytes
            return nbytes

    def commit(self, reserved, actual):
        with self.lock:
            self.used_bytes -= reserved - actual

    def rollback(self, reserved):
        with self.lock:
            self.used_bytes -= reserved
            self.used_files -= 1


def page_bytes(kind, session, error_text=None):
    if kind == "gate":
        with open(os.path.join(ROOT, "src", "gate.html"), encoding="utf-8") as handle:
            template = handle.read()
        err = ""
        if error_text:
            err = f'<p class="bad">{html_escape(error_text)}</p>'
        body = fill(
            template,
            [
                ("{{HTML_LANG}}", html_lang()),
                ("{{TITLE}}", html_escape(t("gate_title"))),
                ("{{H1}}", html_escape(t("gate_h1"))),
                ("{{LEDE}}", html_escape(t("gate_lede"))),
                ("{{UNLOCK}}", html_escape(t("gate_unlock"))),
                ("{{LIMIT}}", html_escape(t("gate_limit"))),
                ("{{ERROR}}", err),
                ("{{UNLOCK_ACTION}}", "unlock"),
                ("{{NEXT}}", "./"),
            ],
        )
        return body, CSP_PAGE
    if kind == "upload":
        with open(os.path.join(ROOT, "src", "upload.html"), encoding="utf-8") as handle:
            template = handle.read()
        strings = json.dumps(
            {
                "tooLarge": t("up_too_large"),
                "sending": t("up_sending"),
                "network": t("up_network"),
                "failed": t("up_failed"),
            },
            ensure_ascii=False,
        ).replace("<", "\\u003c")
        body = fill(
            template,
            [
                ("{{HTML_LANG}}", html_lang()),
                ("{{TITLE}}", html_escape(t("up_title"))),
                ("{{H1}}", html_escape(t("up_h1"))),
                ("{{LEDE}}", html_escape(t("up_lede"))),
                ("{{CHOOSE}}", html_escape(t("up_choose"))),
                ("{{CHOOSE_HELP}}", html_escape(t("up_choose_help"))),
                ("{{PICK}}", html_escape(t("up_pick"))),
                ("{{CAMERA}}", html_escape(t("up_camera"))),
                ("{{LIMIT}}", html_escape(fmt("up_limit", {"max": format_bytes(session.max_bytes)}))),
                ("{{STRINGS}}", strings),
                ("{{MAX_BYTES}}", str(session.max_bytes)),
            ],
        )
        return body, CSP_PAGE
    with open(os.path.join(ROOT, "src", "download.html"), encoding="utf-8") as handle:
        template = handle.read()
    share = session.share
    preview = ""
    if share["mime"].startswith("image/") and share["mime"] != "image/svg+xml":
        preview = '<img class="preview" src="preview" alt="">'
    elif share["mime"].startswith("text/") and 0 < share["size"] <= PREVIEW_MAX:
        raw = os.pread(share["fd"], share["size"], 0)
        if b"\x00" not in raw:
            preview = f'<pre class="preview">{html_escape(raw.decode("utf-8", "replace"))}</pre>'
    body = fill(
        template,
        [
            ("{{HTML_LANG}}", html_lang()),
            ("{{TITLE}}", html_escape(t("dl_title"))),
            ("{{NAME}}", html_escape(share["name"])),
            ("{{LEDE}}", html_escape(fmt("dl_lede", {"size": format_bytes(share["size"])}))),
            ("{{DOWNLOAD}}", html_escape(t("dl_button"))),
            ("{{LIMIT}}", html_escape(t("dl_limit"))),
            ("{{AUTO}}", "1" if session.stop_after else "0"),
            ("{{PREVIEW}}", preview),
        ],
    )
    return body, CSP_DOWNLOAD


def split_target(path):
    if "?" in path or "\\" in path or "\x00" in path:
        return None
    parts = path.split("/")
    if len(parts) < 3 or parts[1] != "s":
        return None
    token = parts[2]
    action = parts[3] if len(parts) > 3 else ""
    if len(parts) > 4 and parts[4] != "":
        return None
    if not TOKEN_RE.fullmatch(token):
        return None
    return token, action


def disposition_filename(header):
    match = re.search(r'filename="([^"]{1,4096})"', header)
    if match:
        return match.group(1)
    match = re.search(r"filename=([^;\r\n]{1,4096})", header)
    if match:
        return match.group(1).strip().strip('"')
    return ""


def save_upload(session, rfile, length, boundary):
    if length > session.max_bytes + PART_OVERHEAD:
        raise LocalError(fmt("file_larger", {"size": format_bytes(session.max_bytes)}))
    # Read exactly Content-Length. One extra byte would block until the
    # client closes a keep-alive connection.
    body = bytearray()
    while len(body) < length:
        chunk = rfile.read(length - len(body))
        if not chunk:
            break
        body += chunk
    if len(body) != length:
        raise LocalError(t("no_file"))
    body = bytes(body)
    marker = b"--" + boundary.encode("ascii")
    chunks = body.split(marker)
    saved = None
    for chunk in chunks[1:]:
        if chunk.startswith(b"--"):
            break
        if chunk.startswith(b"\r\n"):
            chunk = chunk[2:]
        if chunk.endswith(b"\r\n"):
            chunk = chunk[:-2]
        header, sep, data = chunk.partition(b"\r\n\r\n")
        if not sep:
            continue
        text = header.decode("utf-8", "replace")
        if "filename=" not in text.lower():
            continue
        original = disposition_filename(text)
        if not original:
            continue
        name = sanitize_filename(original)
        if len(data) == 0:
            continue
        if len(data) > session.max_bytes:
            raise LocalError(fmt("file_larger", {"size": format_bytes(session.max_bytes)}))
        reserved = session.reserve(len(data))
        tmp = ".qb-" + secrets.token_hex(8) + ".part"
        dirfd = session.dest[0]
        fd = os.open(
            tmp,
            os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_NOFOLLOW | os.O_CLOEXEC,
            0o600,
            dir_fd=dirfd,
        )
        try:
            view = memoryview(data)
            while view:
                wrote = os.write(fd, view)
                view = view[wrote:]
            os.fsync(fd)
        except BaseException:
            os.close(fd)
            os.unlink(tmp, dir_fd=dirfd)
            session.rollback(reserved)
            raise
        os.close(fd)
        final = None
        try:
            for _ in range(8):
                candidate = unique_name(dirfd, name)
                if rename_noreplace(dirfd, tmp, candidate):
                    final = candidate
                    break
            if final is None:
                raise LocalError(t("reserve_name"))
            os.fsync(dirfd)
        except BaseException:
            try:
                os.unlink(tmp, dir_fd=dirfd)
            except OSError:
                pass
            session.rollback(reserved)
            raise
        session.commit(reserved, len(data))
        saved = (final, len(data))
    if saved is None:
        raise LocalError(t("no_file"))
    return saved


class Handler(BaseHTTPRequestHandler):
    # HTTP/1.0 closes after each response. Keep-alive on this handler deadlocks
    # a client that pipelines the next request while a write is still buffered.
    protocol_version = "HTTP/1.0"
    server_version = "QuickBridge"
    sys_version = ""

    def log_message(self, fmt, *args):
        return

    def _session(self):
        return self.server.session

    def _host_ok(self):
        host = (self.headers.get("Host") or "").strip().lower()
        return ct_eq(host, self._session().host_value)

    def _origin_ok(self):
        origin = (self.headers.get("Origin") or "").strip()
        if origin == "":
            site = (self.headers.get("Sec-Fetch-Site") or "").strip().lower()
            return site not in ("cross-site", "cross-origin")
        return ct_eq(origin, self._session().origin)

    def _reject(self, code):
        body = b""
        self.send_response(code)
        self.send_header("Content-Length", "0")
        self.send_header("Cache-Control", "no-store")
        self.send_header("X-Content-Type-Options", "nosniff")
        self.end_headers()
        return body

    def _html(self, status_code, body, csp, cookie=None):
        data = body.encode("utf-8")
        self.send_response(status_code)
        self.send_header("Content-Type", "text/html; charset=utf-8")
        self.send_header("Content-Length", str(len(data)))
        self.send_header("Cache-Control", "no-store")
        self.send_header("X-Content-Type-Options", "nosniff")
        self.send_header("Referrer-Policy", "no-referrer")
        self.send_header("X-Frame-Options", "DENY")
        self.send_header("Content-Security-Policy", csp)
        if cookie:
            self.send_header("Set-Cookie", cookie)
        self.end_headers()
        if self.command != "HEAD":
            self.wfile.write(data)

    def _json(self, status_code, payload):
        data = json.dumps(payload, ensure_ascii=False).encode("utf-8")
        self.send_response(status_code)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(data)))
        self.send_header("Cache-Control", "no-store")
        self.send_header("X-Content-Type-Options", "nosniff")
        self.end_headers()
        if self.command != "HEAD":
            self.wfile.write(data)

    def _redirect(self, location, cookie=None):
        self.send_response(303)
        self.send_header("Location", location)
        self.send_header("Content-Length", "0")
        self.send_header("Cache-Control", "no-store")
        if cookie:
            self.send_header("Set-Cookie", cookie)
        self.end_headers()

    def _cookie_header(self):
        session = self._session()
        return (
            f"qb={session.cookie}; Path=/; HttpOnly; SameSite=Strict; Max-Age=7200"
        )

    def _gate(self, message):
        body, csp = page_bytes("gate", self._session(), message)
        self._html(401 if message else 200, body, csp)

    def _open(self):
        return self._session().cookie_ok(self.headers.get("Cookie"))

    def do_HEAD(self):
        self.do_GET()

    def do_GET(self):
        session = self._session()
        if not session.slots.acquire(blocking=False):
            self._reject(503)
            return
        try:
            self._get(session)
        finally:
            session.slots.release()

    def _get(self, session):
        if not self._host_ok():
            self._reject(404)
            return
        parsed = split_target(self.path)
        if parsed is None:
            self._reject(404)
            return
        token, action = parsed
        if not session.allowed(token):
            self._reject(404)
            return
        if action == "" and not self.path.endswith("/"):
            self._redirect(f"/s/{token}/")
            return
        if action in ("", "/"):
            if not self._open():
                self._gate(None)
                return
            session.touch()
            kind = "upload" if session.mode == "upload" else "download"
            body, csp = page_bytes(kind, session)
            self._html(200, body, csp)
            return
        if action == "file" and session.mode == "download":
            if not self._open():
                self._redirect("./")
                return
            session.touch()
            self._send_file(session, inline=False)
            return
        if action == "preview" and session.mode == "download":
            if not self._open():
                self._reject(401)
                return
            if not session.share["mime"].startswith("image/"):
                self._reject(404)
                return
            session.touch()
            self._send_file(session, inline=True)
            return
        self._reject(404)

    def _send_file(self, session, inline):
        share = session.share
        self.send_response(200)
        self.send_header("Content-Type", share["mime"])
        self.send_header("Content-Length", str(share["size"]))
        self.send_header("Content-Disposition", content_disposition(share["name"], inline))
        self.send_header("X-Content-Type-Options", "nosniff")
        self.send_header("Cache-Control", "no-store")
        self.send_header("X-Content-Length", str(share["size"]))
        self.end_headers()
        if self.command == "HEAD":
            return
        remaining = share["size"]
        offset = 0
        while remaining:
            chunk = os.pread(share["fd"], min(65536, remaining), offset)
            if not chunk:
                break
            self.wfile.write(chunk)
            offset += len(chunk)
            remaining -= len(chunk)
            session.touch()
        if session.stop_after and not inline:
            status("stop-after", t("stop_download"))
            begin_stop()

    def do_POST(self):
        session = self._session()
        if not session.slots.acquire(blocking=False):
            self._reject(503)
            return
        try:
            self._post(session)
        finally:
            session.slots.release()

    def _post(self, session):
        if not self._host_ok() or not self._origin_ok():
            self._reject(404)
            return
        parsed = split_target(self.path)
        if parsed is None:
            self._reject(404)
            return
        token, action = parsed
        if not session.allowed(token):
            self._reject(404)
            return
        if action == "unlock":
            self._unlock(session)
            return
        if action == "upload" and session.mode == "upload":
            self._upload(session)
            return
        self._reject(404)

    def _unlock(self, session):
        try:
            length = int(self.headers.get("Content-Length") or "0")
        except ValueError:
            length = 0
        if length <= 0 or length > 512:
            self._gate(t("wrong_password"))
            return
        ctype = (self.headers.get("Content-Type") or "").split(";", 1)[0].strip().lower()
        if ctype != "application/x-www-form-urlencoded":
            self._gate(t("wrong_password"))
            return
        raw = self.rfile.read(length)
        form = parse_qs(raw.decode("utf-8", "replace"), keep_blank_values=True, max_num_fields=4)
        password = (form.get("password") or [""])[0][:32]
        result = session.check_pin(password)
        if result == "ok":
            session.touch()
            self._redirect("./", self._cookie_header())
            return
        message = t("too_many") if result == "locked" else t("wrong_password")
        self._gate(message)

    def _upload(self, session):
        if not self._open():
            self._json(401, {"ok": False, "error": "password required"})
            return
        try:
            length = int(self.headers.get("Content-Length") or "0")
        except ValueError:
            length = 0
        ctype = self.headers.get("Content-Type") or ""
        if not ctype.lower().startswith("multipart/form-data"):
            self._json(400, {"ok": False, "error": t("no_file")})
            return
        match = re.search(r'boundary="?([A-Za-z0-9\'()+_,\-./:=?]{1,70})"?', ctype)
        if not match:
            self._json(400, {"ok": False, "error": t("no_file")})
            return
        try:
            name, size = save_upload(session, self.rfile, length, match.group(1))
        except LocalError as exc:
            self._json(400, {"ok": False, "error": str(exc)})
            return
        except Exception:
            self._json(400, {"ok": False, "error": t("save_file")})
            return
        session.touch()
        path = os.path.join(session.dest[1], name)
        emit({"event": "upload", "name": name, "path": path, "size": size})
        self._json(200, {"ok": True, "name": name, "size": size})
        if session.stop_after:
            status("stop-after", t("stop_upload"))
            begin_stop()


class Server(ThreadingHTTPServer):
    daemon_threads = True
    allow_reuse_address = False

    def __init__(self, address, session):
        self.session = session
        super().__init__(address, Handler)


class V6Server(Server):
    address_family = socket.AF_INET6

    def server_bind(self):
        self.socket.setsockopt(socket.IPPROTO_IPV6, socket.IPV6_V6ONLY, 0)
        super().server_bind()


def bind_server(loopback, session_factory):
    if loopback:
        server = Server(("127.0.0.1", 0), session_factory)
        return server
    try:
        return V6Server(("::", 0), session_factory)
    except OSError:
        return Server(("0.0.0.0", 0), session_factory)


def server_port(server):
    return server.server_address[1]


def parse_args(argv):
    if not argv or argv[0] != "serve":
        die("local session expects serve")
    out = {
        "mode": "",
        "dest": "",
        "file": "",
        "ephemeral": False,
        "max_bytes": 100 * MIN_FILE,
        "idle": 15 * 60,
        "stop_after": False,
        "loopback": False,
    }
    items = list(argv[1:])
    i = 0
    while i < len(items):
        arg = items[i]
        def take():
            nonlocal i
            i += 1
            if i >= len(items):
                die(t("need_port") if False else "missing value")
            return items[i]

        if arg == "--mode" or arg.startswith("--mode="):
            out["mode"] = arg.split("=", 1)[1] if "=" in arg else take()
        elif arg == "--dest" or arg.startswith("--dest="):
            out["dest"] = arg.split("=", 1)[1] if "=" in arg else take()
        elif arg == "--file" or arg.startswith("--file="):
            out["file"] = arg.split("=", 1)[1] if "=" in arg else take()
        elif arg == "--max-bytes" or arg.startswith("--max-bytes="):
            raw = arg.split("=", 1)[1] if "=" in arg else take()
            if not re.fullmatch(r"[0-9]{1,12}", raw):
                die(t("invalid_port"))
            out["max_bytes"] = int(raw)
        elif arg == "--idle-secs" or arg.startswith("--idle-secs="):
            raw = arg.split("=", 1)[1] if "=" in arg else take()
            if not re.fullmatch(r"[0-9]{1,8}", raw):
                die(t("invalid_port"))
            out["idle"] = int(raw)
        elif arg == "--ephemeral":
            out["ephemeral"] = True
        elif arg == "--stop-after":
            out["stop_after"] = True
        elif arg == "--password":
            pass
        elif arg == "--clipboard":
            die(t("need_file_or_clip"))
        elif arg == "--loopback":
            out["loopback"] = True
        else:
            die(t("invalid_port"))
        i += 1
    if out["mode"] not in ("upload", "download"):
        die(t("need_port"))
    out["max_bytes"] = min(MAX_FILE, max(MIN_FILE, out["max_bytes"]))
    out["idle"] = min(MAX_IDLE, max(MIN_IDLE, out["idle"]))
    return out


def supervise(session):
    while not STOP.wait(0.5):
        now = time.monotonic()
        if now - session.last >= session.idle:
            status("idle-timeout", t("idle_timeout"))
            begin_stop()
            return
        if now - session.started >= session.idle * 2:
            status("session-timeout", t("session_timeout"))
            begin_stop()
            return


def main(argv=None):
    global CATALOG, LANG, SERVER, STATE, EXPECTED_STOP
    argv = list(sys.argv[1:] if argv is None else argv)
    try:
        CATALOG = load_catalog(os.path.join(ROOT, "src", "i18n.rs"))
        LANG = language(os.environ.get("QUICKBRIDGE_LANG") or os.environ.get("LANG") or "")
        args = parse_args(argv)
        status("connecting", t("starting"), 0.08)
        dest = None
        share = None
        if args["mode"] == "upload":
            dest = prepare_dest(args["dest"] or None)
        else:
            if not args["file"]:
                die(t("need_file_or_clip"))
            status("connecting", t("reading_clipboard") if args["ephemeral"] else t("starting"), 0.16)
            share = open_share(args["file"], args["max_bytes"], args["ephemeral"])
        token = secrets.token_hex(16)
        pin = f"{secrets.randbelow(1_000_000):06d}"
        placeholder = Session(
            args["mode"], token, pin, dest, share, args["max_bytes"], args["idle"],
            args["stop_after"], "http://invalid", "invalid",
        )
        server = bind_server(args["loopback"], placeholder)
        SERVER = server
        port = server_port(server)
        if args["loopback"]:
            host = "127.0.0.1"
        else:
            iface = default_iface()
            host = network_name() + ".local"
            status("connecting", t("listening_local"), 0.2)
            _hold, life = open_port_window(iface, port)
            threading.Thread(target=watch_window, args=(life,), daemon=True).start()
        origin = f"http://{host}:{port}"
        host_value = f"{host}:{port}"
        session = Session(
            args["mode"], token, pin, dest, share, args["max_bytes"], args["idle"],
            args["stop_after"], origin, host_value,
        )
        server.session = session
        STATE = session
        SERVER = server
        url = f"{origin}/s/{token}/"
        status("connecting", t("preparing_qr"), 0.9)
        matrix = qr_matrix(url)
        ready = {
            "event": "ready",
            "url": url,
            "dest": dest[1] if dest else share["name"],
            "qr": matrix,
            "location": "local",
            "mode": args["mode"],
            "password": pin,
        }
        if share is not None:
            ready["name"] = share["name"]
        emit(ready)
        threading.Thread(target=supervise, args=(session,), daemon=True).start()
        signal_stop()
        server.serve_forever(poll_interval=0.3)
    except LocalError as exc:
        die(str(exc))
    except BrokenPipeError:
        raise SystemExit(0)
    finally:
        release_hold()
        if SERVER is not None:
            SERVER.server_close()


def signal_stop():
    import signal

    def _stop(_signum, _frame):
        begin_stop()

    signal.signal(signal.SIGTERM, _stop)
    signal.signal(signal.SIGINT, _stop)


if __name__ == "__main__":
    main()
