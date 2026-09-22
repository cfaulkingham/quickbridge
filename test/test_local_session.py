#!/usr/bin/python3
"""Loopback checks for the no-toolchain local session."""

import http.client
import json
import os
import signal
import subprocess
import sys
import tempfile
import time
import unittest

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, os.path.join(ROOT, "scripts"))

import local_session as ls  # noqa: E402


class CatalogTests(unittest.TestCase):
    def test_phone_strings_and_route_parser(self):
        ls.CATALOG = ls.load_catalog(os.path.join(ROOT, "src", "i18n.rs"))
        ls.LANG = "en"
        self.assertEqual(ls.t("up_title"), "Send to desktop")
        self.assertEqual(ls.t("lan_bad_name"), "This computer's name cannot be used in a link")
        self.assertEqual(ls.fmt("lan_waiting", {"port": "9", "iface": "wlan0"}),
                         "Allow port 9 on wlan0 in the terminal window…")
        self.assertEqual(ls.sanitize_filename("../../etc/passwd"), "passwd")
        self.assertEqual(ls.sanitize_filename("notes.desktop"), "notes.bin")
        self.assertEqual(ls.iface_from_route("1.1.1.1 via 10.0.2.2 dev enp0s1 src 10.0.2.15 uid 1000"),
                         ("enp0s1", "10.0.2.15"))
        self.assertIsNone(ls.iface_from_route("1.1.1.1 via 203.0.113.1 dev eth0 src 203.0.113.8"))
        self.assertIsNone(ls.iface_from_route("1.1.1.1 via 10.0.0.1 dev lo src 10.0.0.1"))
        rows = ls.qr_matrix("http://desk.local:47821/s/" + "ab" * 16 + "/")
        self.assertGreater(len(rows), 8)
        self.assertTrue(all(len(row) == len(rows) and set(row) <= {"0", "1"} for row in rows))
        self.assertTrue(set(rows[0]) == {"0"})


def _stop(proc):
    if proc.poll() is None:
        proc.send_signal(signal.SIGTERM)
    try:
        proc.wait(timeout=5)
    except subprocess.TimeoutExpired:
        proc.kill()
        proc.wait(timeout=5)
    if proc.stdout:
        proc.stdout.close()
    if proc.stderr:
        proc.stderr.close()


def _ready(proc, timeout=8):
    deadline = time.monotonic() + timeout
    buf = ""
    while time.monotonic() < deadline:
        line = proc.stdout.readline()
        if not line:
            break
        buf += line
        try:
            event = json.loads(line)
        except json.JSONDecodeError:
            continue
        if event.get("event") == "ready":
            return event
        if event.get("event") == "error":
            raise AssertionError(event)
    raise AssertionError("no ready event:\n" + buf)


class LoopbackTests(unittest.TestCase):
    def test_upload_pin_and_symlink(self):
        home = ls.home_dir()
        dest = tempfile.mkdtemp(prefix=".qb-local-test-", dir=home)
        victim = os.path.join(dest, "victim")
        with open(victim, "w", encoding="utf-8") as fh:
            fh.write("must survive")
        os.symlink(victim, os.path.join(dest, "photo.jpg"))
        proc = subprocess.Popen(
            [
                "/usr/bin/python3", "-I", "-S",
                os.path.join(ROOT, "scripts", "local_session.py"),
                "serve", "--loopback", "--mode", "upload",
                "--dest", dest, "--max-bytes", "1048576", "--idle-secs", "60",
                "--password",
            ],
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
            env={
                "PATH": "/usr/bin",
                "HOME": home,
                "QUICKBRIDGE_LANG": "en",
                "LANG": "C.UTF-8",
                "PYTHONUNBUFFERED": "1",
            },
        )
        try:
            ready = _ready(proc)
            self.assertEqual(ready["location"], "local")
            self.assertEqual(len(ready["password"]), 6)
            self.assertTrue(ready["url"].startswith("http://127.0.0.1:"))
            url = ready["url"]
            host = url.split("/")[2]
            token = url.rstrip("/").rsplit("/", 1)[-1]
            conn = http.client.HTTPConnection(host, timeout=5)
            conn.request("GET", f"/s/{'0' * 32}/", headers={"Host": host})
            missing = conn.getresponse()
            missing.read()
            self.assertEqual(missing.status, 404)
            conn.request("GET", f"/s/{token}/", headers={"Host": host})
            page = conn.getresponse()
            body = page.read()
            self.assertEqual(page.status, 200)
            self.assertIn(b"Enter the desktop code", body)
            conn.request(
                "POST", f"/s/{token}/unlock",
                body="password=000000&next=./",
                headers={"Host": host, "Content-Type": "application/x-www-form-urlencoded"},
            )
            bad = conn.getresponse()
            bad.read()
            self.assertEqual(bad.status, 401)
            conn.request(
                "POST", f"/s/{token}/unlock",
                body=f"password={ready['password']}&next=./",
                headers={"Host": host, "Content-Type": "application/x-www-form-urlencoded"},
            )
            unlocked = conn.getresponse()
            unlocked.read()
            self.assertEqual(unlocked.status, 303)
            cookie = unlocked.getheader("Set-Cookie")
            self.assertIn("qb=", cookie)
            boundary = "qbtestboundary"
            payload = (
                f"--{boundary}\r\n"
                'Content-Disposition: form-data; name="file"; filename="photo.jpg"\r\n'
                "\r\n"
                "hello-from-phone\r\n"
                f"--{boundary}--\r\n"
            ).encode()
            conn.request(
                "POST", f"/s/{token}/upload",
                body=payload,
                headers={
                    "Host": host,
                    "Content-Type": f"multipart/form-data; boundary={boundary}",
                    "Cookie": cookie.split(";", 1)[0],
                },
            )
            uploaded = conn.getresponse()
            data = json.loads(uploaded.read().decode())
            self.assertEqual(uploaded.status, 200, data)
            self.assertTrue(data["ok"])
            saved = os.path.join(dest, data["name"])
            with open(saved, encoding="utf-8") as fh:
                self.assertEqual(fh.read(), "hello-from-phone")
            with open(victim, encoding="utf-8") as fh:
                self.assertEqual(fh.read(), "must survive")
            self.assertNotEqual(os.path.realpath(saved), os.path.realpath(victim))
        finally:
            _stop(proc)

    def test_download_requires_pin(self):
        home = ls.home_dir()
        folder = tempfile.mkdtemp(prefix=".qb-local-dl-", dir=home)
        path = os.path.join(folder, "notes.txt")
        with open(path, "w", encoding="utf-8") as fh:
            fh.write("desktop notes")
        proc = subprocess.Popen(
            [
                "/usr/bin/python3", "-I", "-S",
                os.path.join(ROOT, "scripts", "local_session.py"),
                "serve", "--loopback", "--mode", "download",
                "--file", path, "--max-bytes", "1048576", "--idle-secs", "60",
            ],
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
            env={
                "PATH": "/usr/bin",
                "HOME": home,
                "QUICKBRIDGE_LANG": "en",
                "LANG": "C.UTF-8",
                "PYTHONUNBUFFERED": "1",
            },
        )
        try:
            ready = _ready(proc)
            host = ready["url"].split("/")[2]
            token = ready["url"].rstrip("/").rsplit("/", 1)[-1]
            conn = http.client.HTTPConnection(host, timeout=5)
            conn.request("GET", f"/s/{token}/file", headers={"Host": host})
            blocked = conn.getresponse()
            blocked.read()
            self.assertIn(blocked.status, (302, 303))
            conn.request(
                "POST", f"/s/{token}/unlock",
                body=f"password={ready['password']}&next=./",
                headers={"Host": host, "Content-Type": "application/x-www-form-urlencoded"},
            )
            unlocked = conn.getresponse()
            unlocked.read()
            cookie = unlocked.getheader("Set-Cookie").split(";", 1)[0]
            conn.request("GET", f"/s/{token}/file", headers={"Host": host, "Cookie": cookie})
            got = conn.getresponse()
            self.assertEqual(got.status, 200)
            self.assertEqual(got.read(), b"desktop notes")
        finally:
            _stop(proc)


if __name__ == "__main__":
    unittest.main()
