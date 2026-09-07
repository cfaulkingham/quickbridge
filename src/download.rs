use std::sync::{Arc, Mutex};
use std::time::Instant;

use axum::body::Body;
use axum::extract::{Form, Path, State};
use axum::http::{header, HeaderMap, HeaderValue, Method, StatusCode};
use axum::response::{IntoResponse, Redirect, Response};
use axum::routing::{get, post};
use axum::Router;
use tokio::sync::watch;

use crate::event::{self, Event};
use crate::gate::{self, Gate, UnlockForm};
use crate::share::ShareFile;
use crate::transfer::TransferBody;
use crate::util::{
    constant_time_eq, content_disposition, format_bytes, html_escape, secure_html_with_csp,
};

const PAGE: &str = include_str!("download.html");
const TEXT_PREVIEW_MAX: usize = 16_384;
const CSP: &str = "default-src 'none'; script-src 'unsafe-inline'; style-src 'unsafe-inline'; img-src 'self'; connect-src 'self'; form-action 'self'; base-uri 'none'; frame-ancestors 'none'";

#[derive(Clone)]
pub struct AppState {
    pub token: String,
    pub share: ShareFile,
    pub last_activity: Arc<Mutex<Instant>>,
    pub stop_after: bool,
    pub shutdown: watch::Sender<bool>,
    pub gate: Gate,
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
    Router::new()
        .route("/", get(root))
        .route("/s/{token}", get(page_redirect))
        .route("/s/{token}/", get(page))
        .route("/s/{token}/unlock", post(unlock))
        .route("/s/{token}/file", get(file))
        .route("/s/{token}/preview", get(preview))
        .with_state(state)
}

async fn root() -> StatusCode {
    StatusCode::NOT_FOUND
}

async fn page_redirect(Path(token): Path<String>, State(state): State<AppState>) -> Response {
    if !state.allowed(&token) {
        return StatusCode::NOT_FOUND.into_response();
    }
    Redirect::permanent(&format!("/s/{token}/")).into_response()
}

async fn page(
    Path(token): Path<String>,
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Response {
    if !state.allowed(&token) {
        return StatusCode::NOT_FOUND.into_response();
    }
    if !state.gate.is_open(&headers) {
        return gate::page_response(&state.gate, None);
    }
    state.touch();
    let preview = if state.share.is_image() {
        "<img class=\"preview\" src=\"preview\" alt=\"\">".to_string()
    } else if let Some(text) = state.share.preview_text(TEXT_PREVIEW_MAX) {
        format!("<pre class=\"preview\">{}</pre>", html_escape(&text))
    } else {
        String::new()
    };
    let html = PAGE
        .replace("{{NAME}}", &html_escape(&state.share.name))
        .replace("{{SIZE_LABEL}}", &format_bytes(state.share.size))
        .replace("{{AUTO}}", if state.stop_after { "1" } else { "0" })
        .replace("{{PREVIEW}}", &preview);
    secure_html_with_csp(html, CSP)
}

async fn unlock(
    Path(token): Path<String>,
    State(state): State<AppState>,
    Form(form): Form<UnlockForm>,
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

async fn file(
    Path(token): Path<String>,
    State(state): State<AppState>,
    method: Method,
    headers: HeaderMap,
) -> Response {
    if !state.allowed(&token) {
        return StatusCode::NOT_FOUND.into_response();
    }
    if !state.gate.is_open(&headers) {
        return Redirect::temporary("./").into_response();
    }
    state.touch();
    file_response(&state, false, method == Method::HEAD)
}

async fn preview(
    Path(token): Path<String>,
    State(state): State<AppState>,
    method: Method,
    headers: HeaderMap,
) -> Response {
    if !state.allowed(&token) {
        return StatusCode::NOT_FOUND.into_response();
    }
    if !state.gate.is_open(&headers) {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    if !state.share.is_image() {
        return StatusCode::NOT_FOUND.into_response();
    }
    state.touch();
    file_response(&state, true, method == Method::HEAD)
}

fn file_response(state: &AppState, inline: bool, head: bool) -> Response {
    let body = if head {
        Body::empty()
    } else if inline {
        state.share.body()
    } else {
        let completed = state.clone();
        Body::new(TransferBody::download(
            state.share.body(),
            state.share.size,
            move || {
                completed.touch();
                event::emit(&Event::Download {
                    name: completed.share.name.clone(),
                    size: completed.share.size,
                });
                if completed.stop_after {
                    tokio::spawn(async move {
                        // Give Hyper time to flush the final frame before shutting down.
                        tokio::time::sleep(std::time::Duration::from_millis(250)).await;
                        event::status("stop-after", Some("Stopped after download".to_string()));
                        let _ = completed.shutdown.send(true);
                    });
                }
            },
        ))
    };

    let mut response = Response::new(body);
    let headers = response.headers_mut();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_str(&state.share.mime)
            .unwrap_or_else(|_| HeaderValue::from_static("application/octet-stream")),
    );
    headers.insert(
        header::CONTENT_DISPOSITION,
        HeaderValue::from_str(&content_disposition(&state.share.name, inline))
            .unwrap_or_else(|_| HeaderValue::from_static("attachment")),
    );
    headers.insert(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    );
    headers.insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    headers.insert(
        header::HeaderName::from_static("x-content-length"),
        HeaderValue::from_str(&state.share.size.to_string())
            .unwrap_or_else(|_| HeaderValue::from_static("0")),
    );
    if let Ok(len) = HeaderValue::from_str(&state.share.size.to_string()) {
        headers.insert(header::CONTENT_LENGTH, len);
    }
    response
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::time::Instant;

    use tokio::net::TcpListener;

    async fn spawn_app(share: ShareFile, stop_after: bool) -> (u16, AppState) {
        let (shutdown, _) = watch::channel(false);
        let state = AppState {
            token: "testhold0123456789abcdef012345".into(),
            share,
            last_activity: Arc::new(Mutex::new(Instant::now())),
            stop_after,
            shutdown,
            gate: Gate::off(),
        };
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let app = router(state.clone());
        tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });
        (port, state)
    }

    #[tokio::test]
    async fn unknown_token_is_404() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("notes.txt");
        std::fs::write(&path, b"hello").unwrap();
        let share = crate::share::open_file(path, 1024 * 1024, false).unwrap();
        let (port, _) = spawn_app(share, false).await;
        let client = reqwest::Client::new();
        let res = client
            .get(format!("http://127.0.0.1:{port}/s/nope/"))
            .send()
            .await
            .unwrap();
        assert_eq!(res.status(), reqwest::StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn file_is_served_as_attachment() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("notes.txt");
        std::fs::write(&path, b"hello from desktop").unwrap();
        let share = crate::share::open_file(path, 1024 * 1024, false).unwrap();
        let (port, state) = spawn_app(share, false).await;
        let client = reqwest::Client::new();
        let res = client
            .get(format!("http://127.0.0.1:{port}/s/{}/file", state.token))
            .send()
            .await
            .unwrap();
        assert_eq!(res.status(), reqwest::StatusCode::OK);
        let disp = res
            .headers()
            .get(reqwest::header::CONTENT_DISPOSITION)
            .unwrap()
            .to_str()
            .unwrap();
        assert!(disp.contains("attachment"));
        let body = res.text().await.unwrap();
        assert_eq!(body, "hello from desktop");
    }

    #[tokio::test]
    async fn page_escapes_filename() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("ok.txt");
        std::fs::write(&path, b"hi").unwrap();
        let share = crate::share::open_file(path, 1024 * 1024, false).unwrap();
        let (port, state) = spawn_app(share, false).await;
        let client = reqwest::Client::new();
        let res = client
            .get(format!("http://127.0.0.1:{port}/s/{}/", state.token))
            .send()
            .await
            .unwrap();
        let html = res.text().await.unwrap();
        assert!(html.contains("ok.txt"));
        assert!(html.contains("href=\"file\""));
        assert!(html.contains("<h1>ok.txt</h1>"));
    }
}
