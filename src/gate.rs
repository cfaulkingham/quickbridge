use std::sync::{Arc, Mutex};

use axum::http::{header, HeaderMap, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Redirect, Response};
use axum::Json;
use serde::Deserialize;
use uuid::Uuid;

use crate::util::secure_html;
use crate::util::{constant_time_eq, html_escape};

const PAGE: &str = include_str!("gate.html");
pub const COOKIE_NAME: &str = "qb";
const MAX_FAILURES: u32 = 12;

#[derive(Clone)]
pub struct Gate {
    inner: Option<Arc<Inner>>,
    tunneled: bool,
}

struct Inner {
    pin: String,
    cookie: String,
    failures: Mutex<u32>,
}

#[derive(Deserialize)]
pub struct UnlockForm {
    #[serde(default)]
    pub password: String,
    #[serde(default)]
    pub next: String,
}

impl Gate {
    pub fn off() -> Self {
        Self {
            inner: None,
            tunneled: false,
        }
    }

    pub fn pin(tunneled: bool) -> Self {
        Self {
            inner: Some(Arc::new(Inner {
                pin: generate_pin(),
                cookie: Uuid::new_v4().simple().to_string(),
                failures: Mutex::new(0),
            })),
            tunneled,
        }
    }

    pub fn enabled(&self) -> bool {
        self.inner.is_some()
    }

    pub fn pin_display(&self) -> Option<String> {
        self.inner.as_ref().map(|g| g.pin.clone())
    }

    pub fn is_open(&self, headers: &HeaderMap) -> bool {
        let Some(g) = &self.inner else {
            return true;
        };
        if cookie_matches(headers, &g.cookie) {
            return true;
        }
        if let Some(pass) = basic_password(headers) {
            return matches!(self.check_pin(&pass), PinCheck::Ok);
        }
        false
    }

    /// Strip only credentials belonging to this bridge, preserving a backend's
    /// Basic/Bearer authentication after the browser has obtained our cookie.
    pub fn is_pin_authorization(&self, headers: &HeaderMap) -> bool {
        match (&self.inner, basic_password(headers)) {
            (Some(g), Some(pass)) => constant_time_eq(pass.trim().as_bytes(), g.pin.as_bytes()),
            _ => false,
        }
    }

    pub fn unlock(&self, password: &str) -> Result<HeaderValue, String> {
        if self.inner.is_none() {
            return Err("no password on this session".into());
        }
        match self.check_pin(password) {
            PinCheck::Ok => Ok(self.cookie_header()),
            PinCheck::Wrong => Err("wrong password".into()),
            PinCheck::Locked => Err("too many attempts".into()),
            PinCheck::Busy => Err("session busy".into()),
            PinCheck::Off => Err("no password on this session".into()),
        }
    }

    fn check_pin(&self, offered: &str) -> PinCheck {
        let Some(g) = &self.inner else {
            return PinCheck::Off;
        };
        let mut fails = match g.failures.lock() {
            Ok(f) => f,
            Err(_) => return PinCheck::Busy,
        };
        if *fails >= MAX_FAILURES {
            return PinCheck::Locked;
        }
        if constant_time_eq(offered.trim().as_bytes(), g.pin.as_bytes()) {
            return PinCheck::Ok;
        }
        *fails = fails.saturating_add(1);
        PinCheck::Wrong
    }

    pub fn cookie_header(&self) -> HeaderValue {
        let g = self.inner.as_ref().expect("cookie header on gated session");
        let secure = if self.tunneled { "; Secure" } else { "" };
        let value = format!(
            "{COOKIE_NAME}={}; Path=/; HttpOnly; SameSite=Strict; Max-Age=7200{secure}",
            g.cookie
        );
        HeaderValue::from_str(&value).unwrap_or_else(|_| HeaderValue::from_static("qb="))
    }

    pub fn html(&self, error: Option<&str>, action: &str, next: &str) -> String {
        let err = error
            .map(|e| format!("<p class=\"bad\">{}</p>", html_escape(e)))
            .unwrap_or_default();
        PAGE.replace("{{ERROR}}", &err)
            .replace("{{UNLOCK_ACTION}}", action)
            .replace("{{NEXT}}", &html_escape(next))
    }
}

pub fn generate_pin() -> String {
    let bytes = Uuid::new_v4();
    let n = u32::from_le_bytes(bytes.as_bytes()[0..4].try_into().unwrap()) % 1_000_000;
    format!("{n:06}")
}

pub fn page_response(gate: &Gate, error: Option<&str>) -> Response {
    page_response_at(gate, error, "unlock", "./")
}

pub fn page_response_at(gate: &Gate, error: Option<&str>, action: &str, next: &str) -> Response {
    secure_html(gate.html(error, action, next))
}

pub fn unlock_response(gate: &Gate, password: &str, next: &str) -> (Response, bool) {
    unlock_response_at(gate, password, "unlock", next)
}

pub fn unlock_response_at(
    gate: &Gate,
    password: &str,
    action: &str,
    next: &str,
) -> (Response, bool) {
    match gate.unlock(password) {
        Ok(cookie) => {
            let mut response = Redirect::to(&safe_next_path(next)).into_response();
            response.headers_mut().insert(header::SET_COOKIE, cookie);
            (response, true)
        }
        Err(message) => {
            let mut response = page_response_at(gate, Some(&message), action, next);
            *response.status_mut() = StatusCode::UNAUTHORIZED;
            (response, false)
        }
    }
}

/// Only a same-origin relative path, so a hostile form cannot bounce the
/// browser off-site after unlock.
pub fn safe_next_path(raw: &str) -> String {
    let raw = raw.trim();
    if raw.is_empty() {
        return "/".into();
    }
    if raw == "./" {
        return "./".into();
    }
    if !raw.starts_with('/') || raw.starts_with("//") || raw.contains('\\') {
        return "/".into();
    }
    if raw
        .chars()
        .any(|c| c.is_control() || c == '<' || c == '>' || c == '"')
    {
        return "/".into();
    }
    raw.chars().take(200).collect()
}

pub fn json_password_required() -> Response {
    (
        StatusCode::UNAUTHORIZED,
        Json(serde_json::json!({"ok": false, "error": "password required"})),
    )
        .into_response()
}

enum PinCheck {
    Off,
    Ok,
    Wrong,
    Locked,
    Busy,
}

fn cookie_matches(headers: &HeaderMap, expected: &str) -> bool {
    let Some(raw) = headers.get(header::COOKIE).and_then(|v| v.to_str().ok()) else {
        return false;
    };
    for part in raw.split(';') {
        let part = part.trim();
        let Some(value) = part.strip_prefix(COOKIE_NAME) else {
            continue;
        };
        let Some(value) = value.strip_prefix('=') else {
            continue;
        };
        if constant_time_eq(value.as_bytes(), expected.as_bytes()) {
            return true;
        }
    }
    false
}

fn basic_password(headers: &HeaderMap) -> Option<String> {
    let raw = headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())?;
    let b64 = raw
        .strip_prefix("Basic ")
        .or_else(|| raw.strip_prefix("basic "))?;
    let bytes =
        base64::Engine::decode(&base64::engine::general_purpose::STANDARD, b64.trim()).ok()?;
    let pair = String::from_utf8(bytes).ok()?;
    let pass = pair.split_once(':').map(|(_, p)| p).unwrap_or(&pair);
    Some(pass.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pin_is_six_digits() {
        for _ in 0..20 {
            let pin = generate_pin();
            assert_eq!(pin.len(), 6);
            assert!(pin.bytes().all(|b| b.is_ascii_digit()));
        }
    }

    #[test]
    fn off_gate_is_open() {
        let gate = Gate::off();
        assert!(gate.is_open(&HeaderMap::new()));
        assert!(gate.pin_display().is_none());
    }

    #[test]
    fn pin_gate_requires_cookie_or_basic() {
        let gate = Gate::pin(false);
        let pin = gate.pin_display().unwrap();
        assert!(!gate.is_open(&HeaderMap::new()));
        assert!(gate.unlock("nope").is_err());
        let cookie = gate.unlock(&pin).unwrap();
        let set = cookie.to_str().unwrap();
        assert!(set.contains(&format!("{COOKIE_NAME}=")));
        assert!(!set.contains("Secure"));

        let mut headers = HeaderMap::new();
        let value = set.split(';').next().unwrap();
        headers.insert(header::COOKIE, HeaderValue::from_str(value).unwrap());
        assert!(gate.is_open(&headers));
    }

    fn basic_header(pin: &str) -> HeaderValue {
        let token = base64::Engine::encode(
            &base64::engine::general_purpose::STANDARD,
            format!("bridge:{pin}"),
        );
        HeaderValue::from_str(&format!("Basic {token}")).unwrap()
    }

    #[test]
    fn basic_auth_accepts_pin() {
        let gate = Gate::pin(false);
        let pin = gate.pin_display().unwrap();
        let mut headers = HeaderMap::new();
        headers.insert(header::AUTHORIZATION, basic_header(&pin));
        assert!(gate.is_open(&headers));
    }

    #[test]
    fn distinguishes_pin_credentials_from_backend_authorization() {
        let gate = Gate::pin(false);
        let pin = gate.pin_display().unwrap();
        let mut headers = HeaderMap::new();
        headers.insert(header::AUTHORIZATION, basic_header(&pin));
        assert!(gate.is_pin_authorization(&headers));
        let cookie = gate.cookie_header();
        headers.insert(
            header::COOKIE,
            HeaderValue::from_str(cookie.to_str().unwrap().split(';').next().unwrap()).unwrap(),
        );
        assert!(gate.is_pin_authorization(&headers));
        headers.insert(header::AUTHORIZATION, basic_header("backend-password"));
        assert!(gate.is_open(&headers));
        assert!(!gate.is_pin_authorization(&headers));
        headers.insert(
            header::AUTHORIZATION,
            HeaderValue::from_static("Bearer backend-token"),
        );
        assert!(gate.is_open(&headers));
        assert!(!gate.is_pin_authorization(&headers));
    }

    #[test]
    fn basic_auth_failures_lock_out() {
        let gate = Gate::pin(false);
        let pin = gate.pin_display().unwrap();
        for _ in 0..MAX_FAILURES {
            let mut headers = HeaderMap::new();
            headers.insert(header::AUTHORIZATION, basic_header("000000"));
            assert!(!gate.is_open(&headers));
        }
        let mut headers = HeaderMap::new();
        headers.insert(header::AUTHORIZATION, basic_header(&pin));
        assert!(!gate.is_open(&headers));
        assert_eq!(gate.unlock(&pin).unwrap_err(), "too many attempts");
    }

    #[test]
    fn next_path_stays_relative() {
        assert_eq!(safe_next_path(""), "/");
        assert_eq!(safe_next_path("./"), "./");
        assert_eq!(safe_next_path("/app?x=1"), "/app?x=1");
        assert_eq!(safe_next_path("//evil.example"), "/");
        assert_eq!(safe_next_path("https://evil.example/"), "/");
        assert_eq!(safe_next_path("/x\"onclick"), "/");
    }

    #[test]
    fn too_many_failures_lock_out() {
        let gate = Gate::pin(false);
        for _ in 0..MAX_FAILURES {
            assert_eq!(gate.unlock("abcdef").unwrap_err(), "wrong password");
        }
        assert_eq!(gate.unlock("abcdef").unwrap_err(), "too many attempts");
        let pin = gate.pin_display().unwrap();
        assert_eq!(gate.unlock(&pin).unwrap_err(), "too many attempts");
    }
}
