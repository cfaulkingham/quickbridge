use std::collections::HashMap;
use std::fs::{self, File};
use std::io::Read;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use std::path::Path;
use std::time::Duration;

use anyhow::Result;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

use crate::dest;
use crate::event::{self, Event, PortInfo};

const PROBE_TIMEOUT: Duration = Duration::from_millis(400);
const MAX_PORTS: usize = 64;
const TITLE_MAX: usize = 80;
const PROC_NET_MAX: usize = 1024 * 1024;
const MAX_PROC_PIDS: usize = 4096;
const MAX_FDS_PER_PID: usize = 256;

#[derive(Clone, Debug)]
struct Listener {
    ip: IpAddr,
    port: u16,
    inode: u64,
}

pub async fn run() -> Result<()> {
    let listeners = list_listeners()?;
    let owners = socket_owners();
    let mut best: HashMap<u16, Listener> = HashMap::new();
    for lis in listeners {
        if !is_local_bind(lis.ip) {
            continue;
        }
        best.entry(lis.port)
            .and_modify(|cur| {
                if bind_rank(lis.ip) < bind_rank(cur.ip) {
                    *cur = lis.clone();
                }
            })
            .or_insert(lis);
    }

    let mut ports: Vec<u16> = best.keys().copied().collect();
    ports.sort_unstable();
    ports.truncate(MAX_PORTS);

    let mut tasks = Vec::new();
    for port in ports {
        let lis = best.get(&port).expect("port in map").clone();
        let name = owners.get(&lis.inode).cloned().unwrap_or_default();
        tasks.push(tokio::spawn(async move {
            let title = probe_http(lis.ip, lis.port).await?;
            Some(PortInfo {
                port: lis.port,
                bind: bind_label(lis.ip),
                name,
                title: cap_title(&title),
            })
        }));
    }

    let mut found = Vec::new();
    for task in tasks {
        if let Ok(Some(info)) = task.await {
            found.push(info);
            if found.len() >= MAX_PORTS {
                break;
            }
        }
    }
    found.sort_by_key(|p| p.port);
    event::emit(&Event::Ports { ports: found });
    Ok(())
}

fn is_local_bind(ip: IpAddr) -> bool {
    ip.is_loopback() || ip.is_unspecified()
}

fn bind_rank(ip: IpAddr) -> u8 {
    match ip {
        IpAddr::V4(v) if v.is_loopback() => 0,
        IpAddr::V6(v) if v.is_loopback() => 1,
        IpAddr::V4(v) if v.is_unspecified() => 2,
        IpAddr::V6(v) if v.is_unspecified() => 3,
        _ => 9,
    }
}

fn bind_label(ip: IpAddr) -> String {
    if ip.is_unspecified() {
        "all".into()
    } else if ip.is_loopback() {
        "localhost".into()
    } else {
        ip.to_string()
    }
}

fn cap_title(s: &str) -> String {
    s.chars()
        .filter(|c| *c == ' ' || !c.is_control())
        .take(TITLE_MAX)
        .collect::<String>()
        .trim()
        .to_string()
}

fn list_listeners() -> Result<Vec<Listener>> {
    let mut out = Vec::new();
    parse_proc_net("/proc/net/tcp", false, &mut out);
    parse_proc_net("/proc/net/tcp6", true, &mut out);
    Ok(out)
}

fn read_capped(path: &str, max: usize) -> Option<String> {
    let mut file = File::open(path).ok()?;
    let mut buf = Vec::new();
    let mut tmp = [0u8; 8192];
    loop {
        match file.read(&mut tmp) {
            Ok(0) => break,
            Ok(n) => {
                let room = max.saturating_sub(buf.len());
                if room == 0 {
                    break;
                }
                let take = n.min(room);
                buf.extend_from_slice(&tmp[..take]);
                if take < n {
                    break;
                }
            }
            Err(_) => break,
        }
    }
    Some(String::from_utf8_lossy(&buf).into_owned())
}

fn parse_proc_net(path: &str, ipv6: bool, out: &mut Vec<Listener>) {
    let Some(text) = read_capped(path, PROC_NET_MAX) else {
        return;
    };
    for line in text.lines().skip(1) {
        if let Some(lis) = parse_proc_net_line(line, ipv6) {
            out.push(lis);
        }
    }
}

fn parse_proc_net_line(line: &str, ipv6: bool) -> Option<Listener> {
    let cols: Vec<&str> = line.split_whitespace().collect();
    if cols.len() < 10 {
        return None;
    }
    if cols[3] != "0A" {
        return None;
    }
    let (ip, port) = parse_hex_ip_port(cols[1], ipv6)?;
    if port == 0 {
        return None;
    }
    let inode: u64 = cols[9].parse().ok()?;
    Some(Listener { ip, port, inode })
}

fn parse_hex_ip_port(addr: &str, ipv6: bool) -> Option<(IpAddr, u16)> {
    let (ip_hex, port_hex) = addr.rsplit_once(':')?;
    let port = u16::from_str_radix(port_hex, 16).ok()?;
    let raw = decode_hex(ip_hex)?;
    if ipv6 {
        if raw.len() != 16 {
            return None;
        }
        let mut bytes = [0u8; 16];
        for (i, chunk) in raw.chunks_exact(4).enumerate() {
            let word = u32::from_be_bytes(chunk.try_into().ok()?);
            bytes[i * 4..i * 4 + 4].copy_from_slice(&word.to_le_bytes());
        }
        Some((IpAddr::V6(Ipv6Addr::from(bytes)), port))
    } else {
        if raw.len() != 4 {
            return None;
        }
        let arr: [u8; 4] = raw.try_into().ok()?;
        let word = u32::from_be_bytes(arr);
        let b = word.to_le_bytes();
        Some((IpAddr::V4(Ipv4Addr::new(b[0], b[1], b[2], b[3])), port))
    }
}

fn decode_hex(s: &str) -> Option<Vec<u8>> {
    if s.len() % 2 != 0 {
        return None;
    }
    let mut out = Vec::with_capacity(s.len() / 2);
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        let hi = from_hex(bytes[i])?;
        let lo = from_hex(bytes[i + 1])?;
        out.push((hi << 4) | lo);
        i += 2;
    }
    Some(out)
}

fn from_hex(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

fn socket_owners() -> HashMap<u64, String> {
    let uid = dest::euid();
    let mut map = HashMap::new();
    let Ok(proc) = fs::read_dir("/proc") else {
        return map;
    };
    let mut pids = 0usize;
    for entry in proc.flatten() {
        pids += 1;
        if pids > MAX_PROC_PIDS {
            break;
        }
        let name = entry.file_name();
        let pid = match name.to_str().and_then(|s| s.parse::<u32>().ok()) {
            Some(p) => p,
            None => continue,
        };
        let dir = entry.path();
        if process_uid(&dir) != Some(uid) {
            continue;
        }
        let comm = read_capped(dir.join("comm").to_str().unwrap_or(""), 64)
            .unwrap_or_default()
            .trim()
            .to_string();
        let comm = sanitize_comm(&comm);
        let Ok(fd_dir) = fs::read_dir(dir.join("fd")) else {
            continue;
        };
        let mut nfd = 0usize;
        for fd in fd_dir.flatten() {
            nfd += 1;
            if nfd > MAX_FDS_PER_PID {
                break;
            }
            let Ok(link) = fs::read_link(fd.path()) else {
                continue;
            };
            if let Some(inode) = parse_socket_inode(&link) {
                map.entry(inode).or_insert_with(|| comm.clone());
            }
        }
        let _ = pid;
    }
    map
}

fn process_uid(dir: &Path) -> Option<u32> {
    let status = read_capped(dir.join("status").to_str()?, 8192)?;
    for line in status.lines() {
        let Some(rest) = line.strip_prefix("Uid:") else {
            continue;
        };
        let first = rest.split_whitespace().next()?;
        return first.parse().ok();
    }
    None
}

fn parse_socket_inode(path: &Path) -> Option<u64> {
    let s = path.to_str()?;
    let rest = s.strip_prefix("socket:[")?.strip_suffix(']')?;
    rest.parse().ok()
}

fn sanitize_comm(s: &str) -> String {
    s.chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_' || *c == '.')
        .take(32)
        .collect()
}

async fn probe_http(ip: IpAddr, port: u16) -> Option<String> {
    let addr = SocketAddr::new(loopback_for(ip), port);
    let host = match addr.ip() {
        IpAddr::V6(_) => format!("[::1]:{port}"),
        IpAddr::V4(_) => format!("127.0.0.1:{port}"),
    };
    tokio::time::timeout(PROBE_TIMEOUT, probe_once(addr, &host))
        .await
        .unwrap_or_default()
}

pub async fn confirm_local_http(port: u16) -> Result<SocketAddr> {
    if port == 0 {
        anyhow::bail!("invalid port");
    }
    if probe_http(IpAddr::V4(Ipv4Addr::LOCALHOST), port)
        .await
        .is_some()
    {
        return Ok(SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), port));
    }
    if probe_http(IpAddr::V6(Ipv6Addr::LOCALHOST), port)
        .await
        .is_some()
    {
        return Ok(SocketAddr::new(IpAddr::V6(Ipv6Addr::LOCALHOST), port));
    }
    anyhow::bail!("nothing HTTP is listening on localhost:{port}")
}

fn loopback_for(ip: IpAddr) -> IpAddr {
    match ip {
        IpAddr::V6(v) if v.is_loopback() || v.is_unspecified() => IpAddr::V6(Ipv6Addr::LOCALHOST),
        _ => IpAddr::V4(Ipv4Addr::LOCALHOST),
    }
}

async fn probe_once(addr: SocketAddr, host: &str) -> Option<String> {
    let mut stream = TcpStream::connect(addr).await.ok()?;
    let req = format!(
        "GET / HTTP/1.1\r\nHost: {host}\r\nUser-Agent: quickbridge/{}\r\nConnection: close\r\n\r\n",
        env!("CARGO_PKG_VERSION")
    );
    stream.write_all(req.as_bytes()).await.ok()?;
    let _ = stream.shutdown().await;
    let mut buf = Vec::with_capacity(2048);
    let mut tmp = [0u8; 1024];
    loop {
        if buf.len() >= 8192 {
            break;
        }
        match stream.read(&mut tmp).await {
            Ok(0) => break,
            Ok(n) => {
                let take = n.min(8192 - buf.len());
                buf.extend_from_slice(&tmp[..take]);
            }
            Err(_) => break,
        }
    }
    parse_http_response(&buf)
}

pub fn parse_http_response(buf: &[u8]) -> Option<String> {
    if buf.len() >= 3 && buf[0] == 0x16 && buf[1] == 0x03 {
        return None;
    }
    let text = std::str::from_utf8(buf).ok()?;
    let first = text.split(['\r', '\n']).next().unwrap_or("");
    if !first.starts_with("HTTP/") {
        return None;
    }
    Some(extract_title(text).unwrap_or_default())
}

pub fn extract_title(html: &str) -> Option<String> {
    let lower = html.to_ascii_lowercase();
    let start = lower.find("<title>")? + 7;
    let end_rel = lower[start..].find("</title>")?;
    let raw = html.get(start..start + end_rel)?.trim();
    if raw.is_empty() {
        return None;
    }
    let stripped = raw
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&nbsp;", " ")
        .replace("&#39;", "'")
        .replace("&quot;", "\"");
    Some(cap_title(&stripped))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn process_owner_is_read_after_other_status_lines() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join("status"),
            "Name:\tnode\nState:\tS (sleeping)\nUid:\t1000\t1000\t1000\t1000\n",
        )
        .unwrap();
        assert_eq!(process_uid(dir.path()), Some(1000));
    }

    #[test]
    fn parses_ipv4_loopback_listen() {
        let line = "   0: 0100007F:1F90 00000000:0000 0A 00000000:00000000 00:00000000 00000000     1000        0 12345 1 0000000000000000 100 0 0 10 0";
        let lis = parse_proc_net_line(line, false).unwrap();
        assert_eq!(lis.port, 8080);
        assert_eq!(lis.ip, IpAddr::V4(Ipv4Addr::LOCALHOST));
        assert_eq!(lis.inode, 12345);
    }

    #[test]
    fn parses_ipv4_unspecified() {
        let line = "   1: 00000000:0050 00000000:0000 0A 00000000:00000000 00:00000000 00000000        0        0 99 1 0000000000000000 100 0 0 10 0";
        let lis = parse_proc_net_line(line, false).unwrap();
        assert_eq!(lis.port, 80);
        assert!(lis.ip.is_unspecified());
    }

    #[test]
    fn ignores_non_listen() {
        let line = "   0: 0100007F:1F90 0100007F:0050 01 00000000:00000000 00:00000000 00000000     1000        0 1 1 0000000000000000 100 0 0 10 0";
        assert!(parse_proc_net_line(line, false).is_none());
    }

    #[test]
    fn parses_ipv6_loopback() {
        let line = "   0: 00000000000000000000000001000000:0BB8 00000000000000000000000000000000:0000 0A 00000000:00000000 00:00000000 00000000     1000        0 7 1 0000000000000000 100 0 0 10 0";
        let lis = parse_proc_net_line(line, true).unwrap();
        assert_eq!(lis.port, 3000);
        assert_eq!(lis.ip, IpAddr::V6(Ipv6Addr::LOCALHOST));
    }

    #[test]
    fn http_probe_rejects_tls() {
        assert!(parse_http_response(&[0x16, 0x03, 0x01, 0x00]).is_none());
    }

    #[test]
    fn http_probe_reads_title() {
        let body = b"HTTP/1.1 200 OK\r\nContent-Type: text/html\r\n\r\n<html><title>Vite + React</title></html>";
        assert_eq!(parse_http_response(body).as_deref(), Some("Vite + React"));
    }

    #[test]
    fn http_without_title_is_still_http() {
        let body = b"HTTP/1.0 204 No Content\r\n\r\n";
        assert_eq!(parse_http_response(body).as_deref(), Some(""));
    }
}
