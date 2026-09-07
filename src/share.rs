use std::fs::File;
use std::io::{self, Read};
use std::os::fd::{AsRawFd, FromRawFd};
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;
use std::time::Duration;

use anyhow::{bail, Context, Result};

use crate::dest;
use crate::sanitize::sanitize_filename;

const CLIPBOARD_CAP: u64 = 32 * 1024 * 1024;

#[derive(Clone)]
pub struct ShareFile {
    pub path: PathBuf,
    pub name: String,
    pub size: u64,
    pub mime: String,
    inner: ShareInner,
}

#[derive(Clone)]
enum ShareInner {
    Fd(Arc<File>),
    Memory(Arc<Vec<u8>>),
}

impl ShareFile {
    pub fn is_image(&self) -> bool {
        self.mime.starts_with("image/")
    }

    pub fn is_text(&self) -> bool {
        self.mime.starts_with("text/")
    }

    pub fn preview_text(&self, max: usize) -> Option<String> {
        if !self.is_text() || self.size == 0 || self.size > max as u64 {
            return None;
        }
        let bytes = self.read_prefix(max + 1).ok()?;
        if !looks_like_text(&bytes) {
            return None;
        }
        Some(String::from_utf8_lossy(&bytes).into_owned())
    }

    pub fn read_prefix(&self, n: usize) -> Result<Vec<u8>> {
        match &self.inner {
            ShareInner::Memory(bytes) => Ok(bytes.iter().copied().take(n).collect()),
            ShareInner::Fd(file) => {
                let mut clone = file.try_clone().context("clone share fd")?;
                use std::io::Seek;
                clone.seek(io::SeekFrom::Start(0))?;
                let mut buf = vec![0u8; n.min(self.size as usize)];
                let got = clone.read(&mut buf)?;
                buf.truncate(got);
                Ok(buf)
            }
        }
    }

    pub fn body_bytes(&self) -> Result<Arc<Vec<u8>>> {
        match &self.inner {
            ShareInner::Memory(bytes) => Ok(bytes.clone()),
            ShareInner::Fd(file) => {
                let mut clone = file.try_clone().context("clone share fd")?;
                use std::io::Seek;
                clone.seek(io::SeekFrom::Start(0))?;
                let mut buf = Vec::with_capacity(self.size as usize);
                clone.read_to_end(&mut buf)?;
                Ok(Arc::new(buf))
            }
        }
    }

    pub fn tokio_file(&self) -> Result<Option<tokio::fs::File>> {
        match &self.inner {
            ShareInner::Fd(file) => {
                let clone = file.try_clone().context("clone share fd")?;
                Ok(Some(tokio::fs::File::from_std(clone)))
            }
            ShareInner::Memory(_) => Ok(None),
        }
    }
}

pub fn open_file(path: PathBuf, max_bytes: u64, ephemeral: bool) -> Result<ShareFile> {
    if !path.is_absolute() {
        bail!("file path must be absolute");
    }
    if path.as_os_str().as_bytes().contains(&0) {
        bail!("file path contains NUL");
    }
    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("download.bin");
    let name = sanitize_filename(name);

    let file = open_nofollow(&path)?;
    let meta = file.metadata().context("stat shared file")?;
    if !meta.is_file() {
        bail!("that path is not a regular file");
    }
    if meta.uid() != dest::euid() {
        bail!("file is not owned by you");
    }
    if meta.len() == 0 {
        bail!("file is empty");
    }
    if meta.len() > max_bytes {
        bail!(
            "file is larger than {}",
            crate::util::format_bytes(max_bytes)
        );
    }

    let real = real_path(file.as_raw_fd()).unwrap_or_else(|_| path.clone());
    dest::refuse_sensitive(&real)?;
    dest::refuse_sensitive(&path)?;

    if ephemeral {
        let _ = std::fs::remove_file(&path);
    }

    let mime = mime_for(&name, None);
    Ok(ShareFile {
        path: real,
        name,
        size: meta.len(),
        mime,
        inner: ShareInner::Fd(Arc::new(file)),
    })
}

pub fn capture_clipboard(max_bytes: u64) -> Result<ShareFile> {
    let cap = max_bytes.min(CLIPBOARD_CAP);
    let types = wl_paste_output(&["--list-types"])?;
    let type_list: Vec<&str> = types.lines().collect();
    let (args, name, _mime): (&[&str], &str, &str) =
        if type_list.iter().any(|t| t.starts_with("image/png")) {
            (&["--type", "image/png"], "clipboard.png", "image/png")
        } else if type_list.iter().any(|t| t.starts_with("image/jpeg")) {
            (&["--type", "image/jpeg"], "clipboard.jpg", "image/jpeg")
        } else if type_list.iter().any(|t| t.starts_with("image/webp")) {
            (&["--type", "image/webp"], "clipboard.webp", "image/webp")
        } else {
            (&["--type", "text/plain"], "clipboard.txt", "text/plain")
        };
    let bytes = wl_paste_bytes(args, cap)?;
    if bytes.is_empty() {
        bail!("clipboard is empty");
    }
    let mime = mime_for(name, Some(&bytes));
    let name = if mime.starts_with("image/") {
        match mime.as_str() {
            "image/jpeg" => "clipboard.jpg",
            "image/webp" => "clipboard.webp",
            "image/gif" => "clipboard.gif",
            _ => "clipboard.png",
        }
    } else if looks_like_text(&bytes) {
        "clipboard.txt"
    } else {
        "clipboard.bin"
    };
    Ok(ShareFile {
        path: PathBuf::from(name),
        name: name.to_string(),
        size: bytes.len() as u64,
        mime: mime.to_string(),
        inner: ShareInner::Memory(Arc::new(bytes)),
    })
}

fn wl_paste_output(args: &[&str]) -> Result<String> {
    let bytes = wl_paste_bytes(args, 64 * 1024)?;
    Ok(String::from_utf8_lossy(&bytes).into_owned())
}

fn wl_paste_bytes(args: &[&str], cap: u64) -> Result<Vec<u8>> {
    let mut child = Command::new("/usr/bin/wl-paste")
        .args(args)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .spawn()
        .context("could not read the clipboard (wl-paste)")?;
    let mut stdout = child.stdout.take().context("clipboard stdout")?;
    let mut buf = Vec::new();
    let mut chunk = [0u8; 8192];
    let start = std::time::Instant::now();
    loop {
        if start.elapsed() > Duration::from_secs(2) {
            let _ = child.kill();
            bail!("clipboard read timed out");
        }
        match stdout.read(&mut chunk) {
            Ok(0) => break,
            Ok(n) => {
                if buf.len() as u64 + n as u64 > cap {
                    let _ = child.kill();
                    bail!("clipboard is larger than {}", crate::util::format_bytes(cap));
                }
                buf.extend_from_slice(&chunk[..n]);
            }
            Err(err) if err.kind() == io::ErrorKind::Interrupted => continue,
            Err(err) => {
                let _ = child.kill();
                return Err(err).context("clipboard read failed");
            }
        }
    }
    let _ = child.wait();
    Ok(buf)
}

fn open_nofollow(path: &Path) -> Result<File> {
    let c = std::ffi::CString::new(path.as_os_str().as_bytes()).context("path contains NUL")?;
    let fd = unsafe {
        libc::open(
            c.as_ptr(),
            libc::O_RDONLY | libc::O_NOFOLLOW | libc::O_NONBLOCK | libc::O_CLOEXEC,
        )
    };
    if fd < 0 {
        return Err(io::Error::last_os_error()).context("could not open file");
    }
    let file = unsafe { File::from_raw_fd(fd) };
    let meta = file.metadata().context("stat shared file")?;
    if !meta.is_file() {
        bail!("that path is not a regular file");
    }
    if meta.nlink() != 1 {
        bail!("refusing a hard-linked file");
    }
    if meta.uid() != dest::euid() {
        bail!("file is not owned by you");
    }
    set_blocking(file.as_raw_fd())?;
    Ok(file)
}

fn set_blocking(fd: i32) -> Result<()> {
    let flags = unsafe { libc::fcntl(fd, libc::F_GETFL, 0) };
    if flags < 0 {
        return Err(io::Error::last_os_error()).context("fcntl getfl");
    }
    let rc = unsafe { libc::fcntl(fd, libc::F_SETFL, flags & !libc::O_NONBLOCK) };
    if rc != 0 {
        return Err(io::Error::last_os_error()).context("fcntl setfl");
    }
    Ok(())
}

fn real_path(fd: i32) -> Result<PathBuf> {
    let link = format!("/proc/self/fd/{fd}");
    std::fs::read_link(&link).context("resolve file path")
}

pub fn mime_for(name: &str, bytes: Option<&[u8]>) -> String {
    if let Some(b) = bytes {
        if b.starts_with(&[0x89, b'P', b'N', b'G']) {
            return "image/png".into();
        }
        if b.len() >= 3 && b[0] == 0xFF && b[1] == 0xD8 && b[2] == 0xFF {
            return "image/jpeg".into();
        }
        if b.len() >= 12 && b.starts_with(b"RIFF") && &b[8..12] == b"WEBP" {
            return "image/webp".into();
        }
        if b.starts_with(b"GIF87a") || b.starts_with(b"GIF89a") {
            return "image/gif".into();
        }
        if looks_like_text(b) {
            return "text/plain; charset=utf-8".into();
        }
    }
    let ext = Path::new(name)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    match ext.as_str() {
        "png" => "image/png".into(),
        "jpg" | "jpeg" => "image/jpeg".into(),
        "gif" => "image/gif".into(),
        "webp" => "image/webp".into(),
        "pdf" => "application/pdf".into(),
        "txt" | "md" | "csv" => "text/plain; charset=utf-8".into(),
        "json" => "application/json".into(),
        "svg" | "html" | "htm" | "xml" | "xhtml" => "application/octet-stream".into(),
        _ => "application/octet-stream".into(),
    }
}

fn looks_like_text(bytes: &[u8]) -> bool {
    if bytes.contains(&0) {
        return false;
    }
    let controls = bytes
        .iter()
        .filter(|b| **b < 32 && !matches!(**b, b'\n' | b'\r' | b'\t'))
        .count();
    controls < bytes.len() / 20 + 4
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::symlink;

    #[test]
    fn open_refuses_symlink() {
        let dir = tempfile::tempdir().unwrap();
        let victim = dir.path().join("victim");
        std::fs::write(&victim, b"secret").unwrap();
        let link = dir.path().join("notes.txt");
        symlink(&victim, &link).unwrap();
        assert!(open_file(link, 1024 * 1024, false).is_err());
        assert_eq!(std::fs::read(&victim).unwrap(), b"secret");
    }

    #[test]
    fn open_reads_regular_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("notes.txt");
        std::fs::write(&path, b"hello phone").unwrap();
        let share = open_file(path, 1024 * 1024, false).unwrap();
        assert_eq!(share.name, "notes.txt");
        assert_eq!(share.size, 11);
        assert_eq!(share.body_bytes().unwrap().as_ref(), b"hello phone");
    }

    #[test]
    fn mime_detects_png_magic() {
        let mut bytes = vec![0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];
        bytes.extend_from_slice(&[0; 8]);
        assert_eq!(mime_for("x.bin", Some(&bytes)), "image/png");
    }

    #[test]
    fn html_is_octet_stream() {
        assert_eq!(mime_for("page.html", None), "application/octet-stream");
    }

    #[test]
    fn open_refuses_fifo() {
        let dir = tempfile::tempdir().unwrap();
        let fifo = dir.path().join("pipe");
        let c = std::ffi::CString::new(fifo.as_os_str().as_bytes()).unwrap();
        let rc = unsafe { libc::mkfifo(c.as_ptr(), 0o600) };
        assert_eq!(rc, 0);
        let err = open_file(fifo, 1024 * 1024, false);
        assert!(err.is_err(), "fifo should not hang or open as a file");
    }

    #[test]
    fn open_refuses_hard_link() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("notes.txt");
        std::fs::write(&path, b"hello").unwrap();
        let link = dir.path().join("alias.txt");
        std::fs::hard_link(&path, &link).unwrap();
        assert!(open_file(link, 1024 * 1024, false).is_err());
        assert_eq!(std::fs::read(&path).unwrap(), b"hello");
    }
}
