use std::sync::{Arc, Mutex};
use std::time::Instant;

use axum::body::Bytes;
use axum::extract::{DefaultBodyLimit, Multipart, Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Redirect, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Serialize;
use tokio::io::AsyncWriteExt;
use tokio::sync::watch;

use crate::dest::{self, DestDir};
use crate::event::{self, Event};
use crate::sanitize::sanitize_filename;
use crate::gate::{self, Gate, UnlockForm};
use crate::util::{constant_time_eq, format_bytes, secure_html};

const PAGE: &str = include_str!("upload.html");

#[derive(Clone)]
pub struct SessionUsed {
    pub bytes: u64,
    pub files: u32,
}

#[derive(Clone)]
pub struct AppState {
    pub token: String,
    pub dest: DestDir,
    pub max_bytes: u64,
    pub max_session_bytes: u64,
    pub max_files: u32,
    pub last_activity: Arc<Mutex<Instant>>,
    pub used: Arc<Mutex<SessionUsed>>,
    pub stop_after: bool,
    pub shutdown: watch::Sender<bool>,
    pub gate: Gate,
}

#[derive(Serialize)]
struct UploadResponse {
    ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    size: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

impl AppState {
    fn touch(&self) {
        if let Ok(mut guard) = self.last_activity.lock() {
            *guard = Instant::now();
        }
    }

    fn allowed(&self, token: &str) -> bool {
        constant_time_eq(token.as_bytes(), self.token.as_bytes())
    }
}

pub fn router(state: AppState) -> Router {
    let limit = state.max_bytes.saturating_add(1024 * 1024) as usize;
    Router::new()
        .route("/", get(root))
        .route("/s/{token}", get(page_redirect))
        .route("/s/{token}/", get(page))
        .route("/s/{token}/unlock", post(unlock))
        .route("/s/{token}/upload", post(upload))
        .layer(DefaultBodyLimit::max(limit))
        .with_state(state)
}

async fn root() -> StatusCode {
    StatusCode::NOT_FOUND
}

async fn page_redirect(
    Path(token): Path<String>,
    State(state): State<AppState>,
) -> Response {
    if !state.allowed(&token) {
        return StatusCode::NOT_FOUND.into_response();
    }
    Redirect::permanent(&format!("/s/{token}/")).into_response()
}

async fn page(
    Path(token): Path<String>,
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
) -> Response {
    if !state.allowed(&token) {
        return StatusCode::NOT_FOUND.into_response();
    }
    if !state.gate.is_open(&headers) {
        return gate::page_response(&state.gate, None);
    }
    let html = PAGE
        .replace("{{MAX_BYTES}}", &state.max_bytes.to_string())
        .replace("{{MAX_LABEL}}", &format_bytes(state.max_bytes));
    secure_html(html)
}

async fn unlock(
    Path(token): Path<String>,
    State(state): State<AppState>,
    axum::extract::Form(form): axum::extract::Form<UnlockForm>,
) -> Response {
    if !state.allowed(&token) {
        return StatusCode::NOT_FOUND.into_response();
    }
    let (response, ok) = gate::unlock_response(&state.gate, &form.password, "./");
    if ok {
        state.touch();
    }
    response
}

async fn upload(
    Path(token): Path<String>,
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    multipart: Multipart,
) -> Response {
    if !state.allowed(&token) {
        return StatusCode::NOT_FOUND.into_response();
    }
    if !state.gate.is_open(&headers) {
        return gate::json_password_required();
    }
    match save_files(state, multipart).await {
        Ok(Some((name, size))) => (
            StatusCode::OK,
            Json(UploadResponse {
                ok: true,
                name: Some(name),
                size: Some(size),
                error: None,
            }),
        )
            .into_response(),
        Ok(None) => json_error(StatusCode::BAD_REQUEST, "no file in request"),
        Err(message) => json_error(StatusCode::BAD_REQUEST, message),
    }
}

fn json_error(status: StatusCode, message: impl Into<String>) -> Response {
    let message = message.into();
    (
        status,
        Json(UploadResponse {
            ok: false,
            name: None,
            size: None,
            error: Some(message),
        }),
    )
        .into_response()
}

async fn save_files(
    state: AppState,
    mut multipart: Multipart,
) -> Result<Option<(String, u64)>, String> {
    {
        let used = state.used.lock().map_err(|_| "session busy".to_string())?;
        if used.files >= state.max_files {
            return Err("session file limit reached".into());
        }
        if used.bytes >= state.max_session_bytes {
            return Err("session size limit reached".into());
        }
    }

    let mut saved = None;
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| format!("invalid upload: {e}"))?
    {
        let original = match field.file_name() {
            Some(name) if !name.is_empty() => {
                let mut s = name.to_string();
                if s.len() > 4096 {
                    s.truncate(4096);
                }
                s
            }
            _ => continue,
        };
        let name = sanitize_filename(&original);
        let remaining = {
            let used = state.used.lock().map_err(|_| "session busy".to_string())?;
            if used.files >= state.max_files {
                return Err("session file limit reached".into());
            }
            state
                .max_session_bytes
                .saturating_sub(used.bytes)
                .min(state.max_bytes)
        };
        if remaining == 0 {
            return Err("session size limit reached".into());
        }
        let (saved_name, size) = write_field(field, &state.dest, &name, remaining).await?;
        if size == 0 {
            let _ = dest::unlinkat(state.dest.as_raw_fd(), &saved_name);
            continue;
        }
        {
            let mut used = state.used.lock().map_err(|_| "session busy".to_string())?;
            used.files = used.files.saturating_add(1);
            used.bytes = used.bytes.saturating_add(size);
        }
        state.touch();
        let path = state.dest.path.join(&saved_name);
        event::emit(&Event::Upload {
            name: saved_name.clone(),
            path: path.to_string_lossy().into_owned(),
            size,
        });
        saved = Some((saved_name, size));
    }
    if state.stop_after && saved.is_some() {
        let tx = state.shutdown.clone();
        tokio::spawn(async move {
            tokio::time::sleep(std::time::Duration::from_millis(400)).await;
            event::status("stop-after", Some("Stopped after upload".to_string()));
            let _ = tx.send(true);
        });
    }
    Ok(saved)
}

async fn write_field(
    mut field: axum::extract::multipart::Field<'_>,
    dest: &DestDir,
    name: &str,
    max_bytes: u64,
) -> Result<(String, u64), String> {
    let dirfd = dest.as_raw_fd();
    let final_name = dest::unique_name(dirfd, name).map_err(|e| e.to_string())?;
    let tmp_name = format!(".quickbridge-{}.part", uuid::Uuid::new_v4().simple());
    let std_file = dest::openat_excl(dirfd, &tmp_name).map_err(|e| e.to_string())?;
    let mut file = tokio::fs::File::from_std(std_file);
    let mut written: u64 = 0;
    let result: Result<u64, String> = async {
        loop {
            let chunk: Option<Bytes> = field
                .chunk()
                .await
                .map_err(|e| format!("upload interrupted: {e}"))?;
            let Some(chunk) = chunk else { break };
            written = written.saturating_add(chunk.len() as u64);
            if written > max_bytes {
                return Err(format!("file is larger than {}", format_bytes(max_bytes)));
            }
            file.write_all(&chunk)
                .await
                .map_err(|e| format!("write failed: {e}"))?;
        }
        file.flush()
            .await
            .map_err(|e| format!("write failed: {e}"))?;
        file.sync_all()
            .await
            .map_err(|e| format!("write failed: {e}"))?;
        Ok(written)
    }
    .await;
    drop(file);
    match result {
        Ok(n) => {
            dest::renameat(dirfd, &tmp_name, &final_name).map_err(|e| e.to_string())?;
            let _ = dest::fsync_dir(dirfd);
            Ok((final_name, n))
        }
        Err(err) => {
            let _ = dest::unlinkat(dirfd, &tmp_name);
            Err(err)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::time::Instant;

    use tokio::net::TcpListener;

    fn test_state(dir: &std::path::Path) -> AppState {
        let (shutdown, _) = watch::channel(false);
        AppState {
            token: "testhold0123456789abcdef012345".into(),
            dest: DestDir::open_existing(dir).unwrap(),
            max_bytes: 1024 * 1024,
            max_session_bytes: 8 * 1024 * 1024,
            max_files: 32,
            last_activity: Arc::new(Mutex::new(Instant::now())),
            used: Arc::new(Mutex::new(SessionUsed { bytes: 0, files: 0 })),
            stop_after: false,
            shutdown,
            gate: Gate::off(),
        }
    }

    async fn spawn_app(dir: &std::path::Path) -> (u16, AppState) {
        let state = test_state(dir);
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let app = router(state.clone());
        tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });
        let client = reqwest::Client::new();
        for _ in 0..50 {
            if client
                .get(format!("http://127.0.0.1:{port}/"))
                .send()
                .await
                .is_ok()
            {
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(20)).await;
        }
        (port, state)
    }

    #[tokio::test]
    async fn unknown_token_is_404() {
        let dir = tempfile::tempdir().unwrap();
        let (port, _) = spawn_app(dir.path()).await;
        let client = reqwest::Client::new();
        let res = client
            .get(format!("http://127.0.0.1:{port}/s/nope/"))
            .send()
            .await
            .unwrap();
        assert_eq!(res.status(), reqwest::StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn upload_saves_sanitized_file() {
        let dir = tempfile::tempdir().unwrap();
        let (port, state) = spawn_app(dir.path()).await;
        let form = reqwest::multipart::Form::new().part(
            "file",
            reqwest::multipart::Part::bytes(b"hello from the phone".to_vec())
                .file_name("../../notes.txt")
                .mime_str("text/plain")
                .unwrap(),
        );
        let client = reqwest::Client::new();
        let res = client
            .post(format!(
                "http://127.0.0.1:{port}/s/{}/upload",
                state.token
            ))
            .multipart(form)
            .send()
            .await
            .unwrap();
        assert_eq!(res.status(), reqwest::StatusCode::OK);
        let body: serde_json::Value = res.json().await.unwrap();
        assert_eq!(body["ok"], true);
        assert_eq!(body["name"], "notes.txt");

        let saved = dir.path().join("notes.txt");
        let bytes = std::fs::read(saved).unwrap();
        assert_eq!(bytes, b"hello from the phone");
        assert_eq!(state.used.lock().unwrap().files, 1);
    }

    #[tokio::test]
    async fn upload_does_not_write_through_symlink() {
        let dir = tempfile::tempdir().unwrap();
        let victim = dir.path().join("victim");
        std::fs::write(&victim, b"must survive").unwrap();
        std::os::unix::fs::symlink(&victim, dir.path().join("notes.txt")).unwrap();
        let (port, state) = spawn_app(dir.path()).await;
        let form = reqwest::multipart::Form::new().part(
            "file",
            reqwest::multipart::Part::bytes(b"from phone".to_vec())
                .file_name("notes.txt")
                .mime_str("text/plain")
                .unwrap(),
        );
        let client = reqwest::Client::new();
        let res = client
            .post(format!(
                "http://127.0.0.1:{port}/s/{}/upload",
                state.token
            ))
            .multipart(form)
            .send()
            .await
            .unwrap();
        assert_eq!(res.status(), reqwest::StatusCode::OK);
        assert_eq!(std::fs::read(&victim).unwrap(), b"must survive");
        let body: serde_json::Value = res.json().await.unwrap();
        let name = body["name"].as_str().unwrap();
        assert_ne!(name, "notes.txt");
        assert_eq!(
            std::fs::read(dir.path().join(name)).unwrap(),
            b"from phone"
        );
    }

    #[tokio::test]
    async fn password_blocks_until_unlock() {
        let dir = tempfile::tempdir().unwrap();
        let mut state = test_state(dir.path());
        state.gate = Gate::pin(false);
        let pin = state.gate.pin_display().unwrap();
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let app = router(state.clone());
        tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });
        let client = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .unwrap();
        let page = client
            .get(format!("http://127.0.0.1:{port}/s/{}/", state.token))
            .send()
            .await
            .unwrap();
        let html = page.text().await.unwrap();
        assert!(html.contains("desktop code"));

        let form = reqwest::multipart::Form::new().part(
            "file",
            reqwest::multipart::Part::bytes(b"secret".to_vec())
                .file_name("notes.txt")
                .mime_str("text/plain")
                .unwrap(),
        );
        let denied = client
            .post(format!(
                "http://127.0.0.1:{port}/s/{}/upload",
                state.token
            ))
            .multipart(form)
            .send()
            .await
            .unwrap();
        assert_eq!(denied.status(), reqwest::StatusCode::UNAUTHORIZED);

        let unlocked = client
            .post(format!(
                "http://127.0.0.1:{port}/s/{}/unlock",
                state.token
            ))
            .header("content-type", "application/x-www-form-urlencoded")
            .body(format!("password={pin}"))
            .send()
            .await
            .unwrap();
        assert_eq!(unlocked.status(), reqwest::StatusCode::SEE_OTHER);
        let cookie = unlocked
            .headers()
            .get(reqwest::header::SET_COOKIE)
            .unwrap()
            .to_str()
            .unwrap()
            .split(';')
            .next()
            .unwrap()
            .to_string();
        let form = reqwest::multipart::Form::new().part(
            "file",
            reqwest::multipart::Part::bytes(b"secret".to_vec())
                .file_name("notes.txt")
                .mime_str("text/plain")
                .unwrap(),
        );
        let ok = client
            .post(format!(
                "http://127.0.0.1:{port}/s/{}/upload",
                state.token
            ))
            .header(reqwest::header::COOKIE, cookie)
            .multipart(form)
            .send()
            .await
            .unwrap();
        assert_eq!(ok.status(), reqwest::StatusCode::OK);
    }
}
