use std::ffi::{CString, OsStr, OsString};
use std::fs::{self, File};
use std::io;
use std::os::fd::{AsRawFd, FromRawFd};
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::MetadataExt;
use std::path::{Component, Path, PathBuf};
use std::sync::Arc;

use anyhow::{bail, Context, Result};

const MAX_DEST_DEPTH: usize = 16;
const SENSITIVE_NAMES: &[&str] = &[".ssh", ".gnupg", ".pki", ".gpg", ".password-store"];

/// Destination directory held by a verified descriptor.
/// The pathname is only for display; writes go through the dirfd.
#[derive(Clone)]
pub struct DestDir {
    pub path: PathBuf,
    file: Arc<File>,
}

impl DestDir {
    pub fn as_raw_fd(&self) -> i32 {
        self.file.as_raw_fd()
    }

    pub fn prepare(user: Option<PathBuf>) -> Result<Self> {
        let home = dirs::home_dir().context("no home directory")?;
        let home_fd = open_anchor(&home)?;
        let home_real = realpath_fd(home_fd.as_raw_fd()).unwrap_or_else(|_| home.clone());
        let (anchor, parts, display) = match user {
            None => default_walk(home_fd, &home, &home_real)?,
            Some(p) => user_walk(home_fd, &home, &home_real, p)?,
        };
        let file = walk_components(anchor, &parts, true)?;
        let path = realpath_fd(file.as_raw_fd()).unwrap_or(display);
        refuse_sensitive(&path)?;
        Ok(Self {
            path,
            file: Arc::new(file),
        })
    }

    #[cfg(test)]
    pub fn open_existing(path: &Path) -> Result<Self> {
        let file = open_dir_nofollow(path)?;
        Ok(Self {
            path: path.to_path_buf(),
            file: Arc::new(file),
        })
    }
}

pub fn euid() -> u32 {
    unsafe { libc::geteuid() as u32 }
}

fn default_walk(
    home_fd: File,
    home: &Path,
    home_real: &Path,
) -> Result<(File, Vec<OsString>, PathBuf)> {
    let downloads = dirs::download_dir().unwrap_or_else(|| home.join("Downloads"));
    if downloads.starts_with(home) || downloads.starts_with(home_real) {
        let mut parts = relative_components_either(home, home_real, &downloads)?;
        parts.push(OsString::from("Quick Bridge"));
        let display = home_real.join("Downloads").join("Quick Bridge");
        Ok((home_fd, parts, display))
    } else {
        drop(home_fd);
        let dl_fd = open_anchor(&downloads)?;
        let dl_real = realpath_fd(dl_fd.as_raw_fd()).unwrap_or_else(|_| downloads.clone());
        Ok((
            dl_fd,
            vec![OsString::from("Quick Bridge")],
            dl_real.join("Quick Bridge"),
        ))
    }
}

fn user_walk(
    home_fd: File,
    home: &Path,
    home_real: &Path,
    p: PathBuf,
) -> Result<(File, Vec<OsString>, PathBuf)> {
    if !p.is_absolute() {
        bail!("save folder must be an absolute path");
    }
    if p.as_os_str().as_bytes().contains(&0) {
        bail!("save folder contains NUL");
    }
    let name = p
        .file_name()
        .ok_or_else(|| anyhow::anyhow!("save folder has no name"))?;
    if name == "." || name == ".." {
        bail!("invalid save folder name");
    }
    if p.starts_with(home_real) || p.starts_with(home) {
        let parts = relative_components_either(home, home_real, &p)?;
        return Ok((home_fd, parts, p));
    }
    drop(home_fd);
    if let Some(downloads) = dirs::download_dir() {
        let dl_fd = open_anchor(&downloads)?;
        let dl_real = realpath_fd(dl_fd.as_raw_fd()).unwrap_or_else(|_| downloads.clone());
        if p.starts_with(&dl_real) || p.starts_with(&downloads) {
            let parts = relative_components_either(&downloads, &dl_real, &p)?;
            return Ok((dl_fd, parts, p));
        }
    }
    bail!("save folder must be under your home directory")
}

fn relative_components(base: &Path, dest: &Path) -> Result<Vec<OsString>> {
    let rel = dest
        .strip_prefix(base)
        .map_err(|_| anyhow::anyhow!("save folder must be under your home directory"))?;
    let mut parts = Vec::new();
    for c in rel.components() {
        match c {
            Component::Normal(s) => {
                if s.is_empty() || s == "." || s == ".." {
                    bail!("invalid save folder name");
                }
                parts.push(s.to_os_string());
            }
            Component::CurDir => {}
            _ => bail!("invalid save folder path"),
        }
    }
    if parts.is_empty() {
        bail!("path cannot be your home directory");
    }
    if parts.len() > MAX_DEST_DEPTH {
        bail!("save folder path is too deep");
    }
    Ok(parts)
}

fn relative_components_either(a: &Path, b: &Path, dest: &Path) -> Result<Vec<OsString>> {
    if dest.starts_with(b) {
        relative_components(b, dest)
    } else {
        relative_components(a, dest)
    }
}

/// Home or XDG Downloads: follow this one configured path, then never
/// follow again. Every later component is `O_NOFOLLOW`.
fn open_anchor(path: &Path) -> Result<File> {
    let c = c_path(path)?;
    let fd = unsafe {
        libc::open(
            c.as_ptr(),
            libc::O_RDONLY | libc::O_DIRECTORY | libc::O_CLOEXEC | libc::O_NONBLOCK,
        )
    };
    if fd < 0 {
        return Err(io::Error::last_os_error()).context("could not open save folder root");
    }
    let file = unsafe { File::from_raw_fd(fd) };
    let meta = file.metadata().context("stat save folder root")?;
    if !meta.is_dir() {
        bail!("save folder root is not a directory");
    }
    if meta.uid() != euid() {
        bail!("save folder root is not owned by you");
    }
    Ok(file)
}

fn walk_components(mut fd: File, parts: &[OsString], chmod_leaf: bool) -> Result<File> {
    if parts.is_empty() {
        bail!("save folder has no name");
    }
    for (i, part) in parts.iter().enumerate() {
        if forbidden_component(part) {
            bail!("that path is not allowed");
        }
        let is_leaf = i + 1 == parts.len();
        let name = c_component(part)?;
        let dirfd = fd.as_raw_fd();
        let next = match openat_dir(dirfd, &name) {
            Ok(f) => f,
            Err(err) if err.raw_os_error() == Some(libc::ENOENT) => {
                let mode = if is_leaf { 0o700 } else { 0o755 };
                mkdirat(dirfd, &name, mode)?;
                match openat_dir(dirfd, &name) {
                    Ok(f) => f,
                    Err(_) if is_symlink_at(dirfd, &name) => {
                        bail!("save folder path cannot contain a symlink");
                    }
                    Err(err) => {
                        return Err(err).context("could not open save folder");
                    }
                }
            }
            Err(err) => {
                if is_symlink_at(dirfd, &name) {
                    bail!("save folder path cannot contain a symlink");
                }
                return Err(err).context("could not open save folder");
            }
        };
        let meta = next.metadata().context("stat save folder")?;
        if !meta.is_dir() {
            bail!("save folder is not a directory");
        }
        if meta.uid() != euid() {
            bail!("save folder is not owned by you");
        }
        if is_leaf && chmod_leaf {
            let rc = unsafe { libc::fchmod(next.as_raw_fd(), 0o700) };
            if rc != 0 {
                return Err(io::Error::last_os_error())
                    .context("could not set save folder permissions");
            }
        }
        let real = realpath_fd(next.as_raw_fd()).unwrap_or_default();
        if !real.as_os_str().is_empty() {
            refuse_sensitive(&real)?;
        }
        fd = next;
    }
    Ok(fd)
}

fn openat_dir(dirfd: i32, name: &CString) -> io::Result<File> {
    let fd = unsafe {
        libc::openat(
            dirfd,
            name.as_ptr(),
            libc::O_RDONLY
                | libc::O_DIRECTORY
                | libc::O_NOFOLLOW
                | libc::O_CLOEXEC
                | libc::O_NONBLOCK,
        )
    };
    if fd < 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(unsafe { File::from_raw_fd(fd) })
}

fn is_symlink_at(dirfd: i32, name: &CString) -> bool {
    let mut st: libc::stat = unsafe { std::mem::zeroed() };
    let rc = unsafe { libc::fstatat(dirfd, name.as_ptr(), &mut st, libc::AT_SYMLINK_NOFOLLOW) };
    rc == 0 && (st.st_mode & libc::S_IFMT) == libc::S_IFLNK
}

fn mkdirat(dirfd: i32, name: &CString, mode: libc::mode_t) -> Result<()> {
    let rc = unsafe { libc::mkdirat(dirfd, name.as_ptr(), mode) };
    if rc == 0 {
        return Ok(());
    }
    let err = io::Error::last_os_error();
    if err.raw_os_error() == Some(libc::EEXIST) {
        return Ok(());
    }
    Err(err).context("could not create save folder")
}

fn forbidden_component(name: &OsStr) -> bool {
    SENSITIVE_NAMES
        .iter()
        .any(|s| name.as_bytes() == s.as_bytes())
}

pub fn refuse_sensitive(path: &Path) -> Result<()> {
    let Some(home) = dirs::home_dir() else {
        return Ok(());
    };
    let home = fs::canonicalize(&home).unwrap_or(home);
    if path == home {
        bail!("path cannot be your home directory");
    }
    for name in SENSITIVE_NAMES {
        let sensitive = home.join(name);
        if path == sensitive || path.starts_with(&sensitive) {
            bail!("that path is not allowed");
        }
    }
    Ok(())
}

fn realpath_fd(fd: i32) -> Result<PathBuf> {
    std::fs::read_link(format!("/proc/self/fd/{fd}")).context("resolve directory path")
}

fn c_path(path: &Path) -> Result<CString> {
    CString::new(path.as_os_str().as_bytes()).context("path contains NUL")
}

fn c_name(name: &str) -> Result<CString> {
    if name.is_empty() || name == "." || name == ".." || name.contains('/') || name.contains('\0') {
        bail!("invalid file name");
    }
    CString::new(name).context("file name contains NUL")
}

fn c_component(name: &OsStr) -> Result<CString> {
    let bytes = name.as_bytes();
    if bytes.is_empty()
        || bytes.contains(&0)
        || bytes.contains(&b'/')
        || name == "."
        || name == ".."
        || bytes.len() > 255
    {
        bail!("invalid save folder name");
    }
    CString::new(bytes).context("path component contains NUL")
}

#[cfg(test)]
fn open_dir_nofollow(path: &Path) -> Result<File> {
    let c = c_path(path)?;
    let fd = unsafe {
        libc::open(
            c.as_ptr(),
            libc::O_RDONLY | libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC,
        )
    };
    if fd < 0 {
        return Err(io::Error::last_os_error()).context("could not open save folder");
    }
    Ok(unsafe { File::from_raw_fd(fd) })
}

pub fn child_exists(dirfd: i32, name: &str) -> Result<bool> {
    let c = c_name(name)?;
    let rc = unsafe { libc::faccessat(dirfd, c.as_ptr(), libc::F_OK, libc::AT_SYMLINK_NOFOLLOW) };
    if rc == 0 {
        return Ok(true);
    }
    let err = io::Error::last_os_error();
    if err.raw_os_error() == Some(libc::ENOENT) {
        return Ok(false);
    }
    Err(err).context("stat dest entry")
}

pub fn unique_name(dirfd: i32, name: &str) -> Result<String> {
    if !child_exists(dirfd, name)? {
        return Ok(name.to_string());
    }
    let stem = Path::new(name)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("upload");
    let ext = Path::new(name).extension().and_then(|s| s.to_str());
    for i in 1..10_000 {
        let candidate = match ext {
            Some(e) => format!("{stem}-{i}.{e}"),
            None => format!("{stem}-{i}"),
        };
        if !child_exists(dirfd, &candidate)? {
            return Ok(candidate);
        }
    }
    Ok(format!("{stem}-{}.bin", uuid::Uuid::new_v4().simple()))
}

pub fn openat_excl(dirfd: i32, name: &str) -> Result<File> {
    let c = c_name(name)?;
    let fd = unsafe {
        libc::openat(
            dirfd,
            c.as_ptr(),
            libc::O_WRONLY | libc::O_CREAT | libc::O_EXCL | libc::O_NOFOLLOW | libc::O_CLOEXEC,
            0o600,
        )
    };
    if fd < 0 {
        return Err(io::Error::last_os_error()).context("could not create temp file");
    }
    let rc = unsafe { libc::fchmod(fd, 0o600) };
    if rc != 0 {
        let err = io::Error::last_os_error();
        unsafe { libc::close(fd) };
        return Err(err).context("could not set temp file permissions");
    }
    Ok(unsafe { File::from_raw_fd(fd) })
}

/// Atomically publish a completed upload without replacing an existing entry.
/// Returns false when another writer claimed the name first.
pub fn publish_noreplace(dirfd: i32, from: &str, to: &str) -> Result<bool> {
    let src = c_name(from)?;
    let dst = c_name(to)?;
    #[cfg(target_os = "linux")]
    let rc = unsafe {
        libc::renameat2(
            dirfd,
            src.as_ptr(),
            dirfd,
            dst.as_ptr(),
            libc::RENAME_NOREPLACE,
        )
    };
    // The caller's temporary-file guard removes the extra link on other Unix hosts.
    #[cfg(not(target_os = "linux"))]
    let rc = unsafe { libc::linkat(dirfd, src.as_ptr(), dirfd, dst.as_ptr(), 0) };
    if rc != 0 {
        let err = io::Error::last_os_error();
        if err.raw_os_error() == Some(libc::EEXIST) {
            return Ok(false);
        }
        return Err(err).context("could not save file");
    }
    Ok(true)
}

pub fn unlinkat(dirfd: i32, name: &str) -> Result<()> {
    let c = c_name(name)?;
    let rc = unsafe { libc::unlinkat(dirfd, c.as_ptr(), 0) };
    if rc != 0 {
        return Err(io::Error::last_os_error()).context("unlink");
    }
    Ok(())
}

pub fn fsync_dir(dirfd: i32) -> Result<()> {
    let rc = unsafe { libc::fsync(dirfd) };
    if rc != 0 {
        return Err(io::Error::last_os_error()).context("fsync save folder");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::symlink;

    #[test]
    fn open_refuses_symlink_leaf() {
        let dir = tempfile::tempdir().unwrap();
        let dest = dir.path().join("leaf");
        let victim = dir.path().join("victim");
        fs::write(&victim, b"must survive").unwrap();
        symlink(&victim, &dest).unwrap();
        assert!(open_dir_nofollow(&dest).is_err());
        assert_eq!(fs::read(&victim).unwrap(), b"must survive");
    }

    #[test]
    fn unique_name_skips_existing() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("photo.jpg"), b"a").unwrap();
        fs::write(dir.path().join("photo-1.jpg"), b"b").unwrap();
        let dest = DestDir::open_existing(dir.path()).unwrap();
        let next = unique_name(dest.as_raw_fd(), "photo.jpg").unwrap();
        assert_eq!(next, "photo-2.jpg");
    }

    #[test]
    fn walk_refuses_symlink_component() {
        let dir = tempfile::tempdir().unwrap();
        let victim = dir.path().join("victim");
        fs::create_dir(&victim).unwrap();
        symlink(&victim, dir.path().join("Downloads")).unwrap();
        let anchor = open_anchor(dir.path()).unwrap();
        let err = walk_components(
            anchor,
            &[OsString::from("Downloads"), OsString::from("Quick Bridge")],
            true,
        )
        .unwrap_err();
        assert!(
            err.to_string().contains("symlink"),
            "unexpected error: {err:#}"
        );
    }

    #[test]
    fn walk_creates_owned_leaf() {
        let dir = tempfile::tempdir().unwrap();
        let anchor = open_anchor(dir.path()).unwrap();
        let leaf = walk_components(anchor, &[OsString::from("Quick Bridge")], true).unwrap();
        let meta = leaf.metadata().unwrap();
        assert!(meta.is_dir());
        assert_eq!(meta.uid(), euid());
        assert_eq!(meta.mode() & 0o777, 0o700);
        assert!(dir.path().join("Quick Bridge").is_dir());
    }

    #[test]
    fn prepare_refuses_tmp() {
        assert!(DestDir::prepare(Some(std::path::PathBuf::from("/tmp/quickbridge-dest"))).is_err());
    }

    #[test]
    fn publish_never_replaces_symlinks_or_files() {
        let dir = tempfile::tempdir().unwrap();
        let victim = dir.path().join("victim");
        fs::write(&victim, b"must survive").unwrap();
        symlink(&victim, dir.path().join("notes.txt")).unwrap();
        let dest = DestDir::open_existing(dir.path()).unwrap();
        let fd = dest.as_raw_fd();
        let mut tmp = openat_excl(fd, ".quickbridge-test.part").unwrap();
        use std::io::Write;
        tmp.write_all(b"upload").unwrap();
        tmp.sync_all().unwrap();
        drop(tmp);
        assert!(!publish_noreplace(fd, ".quickbridge-test.part", "notes.txt").unwrap());
        assert_eq!(fs::read(&victim).unwrap(), b"must survive");
        assert!(dir.path().join("notes.txt").is_symlink());
        assert!(!publish_noreplace(fd, ".quickbridge-test.part", "victim").unwrap());
        assert!(publish_noreplace(fd, ".quickbridge-test.part", "notes-1.txt").unwrap());
        assert_eq!(fs::read(dir.path().join("notes-1.txt")).unwrap(), b"upload");
    }
}
