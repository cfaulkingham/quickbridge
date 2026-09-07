use std::io::{self, Write};

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct PortInfo {
    pub port: u16,
    pub bind: String,
    pub name: String,
    pub title: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "event", rename_all = "kebab-case")]
pub enum Event {
    Status {
        state: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        message: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        progress: Option<f64>,
    },
    Ready {
        url: String,
        dest: String,
        qr: Vec<String>,
        location: String,
        mode: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        name: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        port: Option<u16>,
        #[serde(skip_serializing_if = "Option::is_none")]
        password: Option<String>,
    },
    Upload {
        name: String,
        path: String,
        size: u64,
    },
    Download {
        name: String,
        size: u64,
    },
    Ports {
        ports: Vec<PortInfo>,
    },
    Error {
        message: String,
    },
}

fn cap_text(s: impl Into<String>, n: usize) -> String {
    s.into()
        .chars()
        .filter(|c| *c == ' ' || *c == '-' || !c.is_control())
        .take(n)
        .collect()
}

pub fn emit(event: &Event) {
    let mut out = io::stdout().lock();
    if serde_json::to_writer(&mut out, event).is_ok() {
        let _ = out.write_all(b"\n");
    }
    let _ = out.flush();
}

pub fn status(state: &str, message: Option<String>) {
    emit(&Event::Status {
        state: cap_text(state, 32),
        message: message.map(|m| cap_text(m, 160)),
        progress: None,
    });
}

pub fn progress(state: &str, message: impl Into<String>, progress: f64) {
    emit(&Event::Status {
        state: cap_text(state, 32),
        message: Some(cap_text(message, 160)),
        progress: Some(progress.clamp(0.0, 1.0)),
    });
}

pub fn error(message: impl Into<String>) {
    emit(&Event::Error {
        message: cap_text(message, 160),
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn progress_event_serializes_fraction() {
        let json = serde_json::to_string(&Event::Status {
            state: "connecting".into(),
            message: Some("Opening Cloudflare tunnel…".into()),
            progress: Some(0.4),
        })
        .unwrap();
        assert!(json.contains("\"event\":\"status\""));
        assert!(json.contains("\"progress\":0.4"));
        assert!(json.contains("connecting"));
    }

    #[test]
    fn ready_includes_mode() {
        let json = serde_json::to_string(&Event::Ready {
            url: "https://abc.trycloudflare.com/s/0123456789abcdef0123456789abcdef/".into(),
            dest: "/tmp".into(),
            qr: vec!["01".into()],
            location: "edge".into(),
            mode: "upload".into(),
            name: None,
            port: None,
            password: None,
        })
        .unwrap();
        assert!(json.contains("\"mode\":\"upload\""));
    }
}
