use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use axum::body::Body;
use axum::extract::{DefaultBodyLimit, Form, Request, State};
use axum::http::{header, HeaderValue, Method, StatusCode, Uri};
use axum::response::{IntoResponse, Response};
use axum::routing::{any, post};
use axum::Router;
use http_body_util::Limited;
use hyper_util::client::legacy::connect::HttpConnector;
use hyper_util::client::legacy::Client;
use hyper_util::rt::TokioExecutor;
use tokio::sync::Semaphore;

use crate::gate::{self, Gate, UnlockForm};

const UNLOCK_PATH: &str = "/__quickbridge/unlock";
/// Phone → helper request and helper → backend response ceiling.
pub const MAX_PROXY_BYTES: usize = 32 * 1024 * 1024;
pub const PROXY_REQUEST_TIMEOUT: Duration = Duration::from_secs(30);
pub const MAX_PROXY_CONCURRENT: usize = 8;

pub type HttpClient = Client<HttpConnector, Body>;

#[derive(Clone)]
pub struct AppState {
    pub port: u16,
    pub client: HttpClient,
    pub last_activity: Arc<Mutex<Instant>>,
    pub tunneled: bool,
    pub gate: Gate,
    pub limit: Arc<Semaphore>,
}

impl AppState {
    fn touch(&self) {
        if let Ok(mut guard) = self.last_activity.lock() {
            *guard = Instant::now();
        }
    }
}

pub fn client() -> HttpClient {
    let mut connector = HttpConnector::new();
    connector.set_nodelay(true);
    connector.set_connect_timeout(Some(Duration::from_secs(5)));
    Client::builder(TokioExecutor::new()).build(connector)
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route(UNLOCK_PATH, post(unlock))
        .fallback(any(forward))
        .layer(DefaultBodyLimit::max(MAX_PROXY_BYTES))
        .with_state(state)
}

async fn unlock(State(state): State<AppState>, Form(form): Form<UnlockForm>) -> Response {
    if !state.gate.enabled() {
        return axum::response::Redirect::to("/").into_response();
    }
    let (response, ok) =
        gate::unlock_response_at(&state.gate, &form.password, UNLOCK_PATH, &form.next);
    if ok {
        state.touch();
    }
    response
}

/// Rewrite a backend Location that points at the local origin into a
/// relative URL so the phone stays on the public hostname.
pub fn rewrite_location(value: &str, port: u16) -> Option<String> {
    let value = value.trim();
    let prefixes = [
        format!("http://127.0.0.1:{port}"),
        format!("http://localhost:{port}"),
        format!("http://[::1]:{port}"),
        format!("https://127.0.0.1:{port}"),
        format!("https://localhost:{port}"),
        format!("https://[::1]:{port}"),
    ];
    for prefix in prefixes {
        if value == prefix {
            return Some("/".into());
        }
        let slash = format!("{prefix}/");
        let query = format!("{prefix}?");
        if let Some(rest) = value.strip_prefix(&slash) {
            return Some(format!("/{rest}"));
        }
        if let Some(rest) = value.strip_prefix(&query) {
            return Some(format!("/?{rest}"));
        }
    }
    None
}

fn is_hop_by_hop(name: &header::HeaderName) -> bool {
    matches!(
        name.as_str(),
        "connection"
            | "keep-alive"
            | "proxy-authenticate"
            | "proxy-authorization"
            | "te"
            | "trailers"
            | "transfer-encoding"
            | "upgrade"
    )
}

async fn forward(State(state): State<AppState>, req: Request) -> Response {
    if !state.gate.is_open(req.headers()) {
        let next = req
            .uri()
            .path_and_query()
            .map(|pq| pq.as_str())
            .unwrap_or("/");
        return gate::page_response_at(&state.gate, None, UNLOCK_PATH, next);
    }
    if req.method() == Method::CONNECT || req.method() == Method::TRACE {
        return StatusCode::METHOD_NOT_ALLOWED.into_response();
    }
    if req.headers().contains_key(header::UPGRADE) {
        return (
            StatusCode::NOT_IMPLEMENTED,
            "WebSocket proxy is not supported",
        )
            .into_response();
    }

    let path_and_query = req
        .uri()
        .path_and_query()
        .map(|pq| pq.as_str().to_string())
        .unwrap_or_else(|| "/".into());
    let target = format!("http://127.0.0.1:{}{}", state.port, path_and_query);
    let uri: Uri = match target.parse() {
        Ok(u) => u,
        Err(_) => return StatusCode::BAD_REQUEST.into_response(),
    };

    let forwarded_host = req
        .headers()
        .get(header::HOST)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();

    let (parts, body) = req.into_parts();
    let mut builder = hyper::Request::builder()
        .method(parts.method)
        .uri(uri)
        .version(parts.version);

    for (name, value) in parts.headers.iter() {
        if is_hop_by_hop(name) || name == header::HOST {
            continue;
        }
        if matches!(
            name.as_str(),
            "x-forwarded-for" | "x-real-ip" | "forwarded"
        ) {
            continue;
        }
        if state.gate.enabled() && name == header::AUTHORIZATION {
            continue;
        }
        if name == header::COOKIE {
            if let Some(filtered) = strip_qb_cookie(value) {
                builder = builder.header(name, filtered);
            }
            continue;
        }
        builder = builder.header(name, value);
    }
    let host = format!("127.0.0.1:{}", state.port);
    builder = builder.header(header::HOST, &host);
    if !forwarded_host.is_empty() {
        builder = builder.header("x-forwarded-host", forwarded_host);
    }
    builder = builder.header(
        "x-forwarded-proto",
        if state.tunneled { "https" } else { "http" },
    );

    let request = match builder.body(body) {
        Ok(r) => r,
        Err(_) => return StatusCode::BAD_REQUEST.into_response(),
    };

    let Ok(_permit) = state.limit.clone().try_acquire_owned() else {
        return StatusCode::SERVICE_UNAVAILABLE.into_response();
    };

    state.touch();
    match tokio::time::timeout(PROXY_REQUEST_TIMEOUT, state.client.request(request)).await {
        Ok(Ok(resp)) => {
            let (mut parts, incoming) = resp.into_parts();
            if let Some(loc) = parts.headers.get(header::LOCATION).cloned() {
                if let Ok(s) = loc.to_str() {
                    if let Some(rewritten) = rewrite_location(s, state.port) {
                        if let Ok(v) = HeaderValue::from_str(&rewritten) {
                            parts.headers.insert(header::LOCATION, v);
                        }
                    }
                }
            }
            for hop in [
                header::CONNECTION,
                header::TRANSFER_ENCODING,
                header::HeaderName::from_static("keep-alive"),
            ] {
                parts.headers.remove(hop);
            }
            Response::from_parts(parts, Body::new(Limited::new(incoming, MAX_PROXY_BYTES)))
        }
        Ok(Err(err)) => {
            tracing::warn!("proxy to 127.0.0.1:{} failed: {err}", state.port);
            StatusCode::BAD_GATEWAY.into_response()
        }
        Err(_) => StatusCode::GATEWAY_TIMEOUT.into_response(),
    }
}

fn strip_qb_cookie(value: &HeaderValue) -> Option<HeaderValue> {
    let raw = value.to_str().ok()?;
    let kept: Vec<&str> = raw
        .split(';')
        .map(str::trim)
        .filter(|part| {
            let name = part.split('=').next().unwrap_or("").trim();
            !name.eq_ignore_ascii_case(gate::COOKIE_NAME)
        })
        .collect();
    if kept.is_empty() {
        return None;
    }
    HeaderValue::from_str(&kept.join("; ")).ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::routing::get;
    use tokio::net::TcpListener;

    #[test]
    fn rewrites_local_absolute_location() {
        assert_eq!(rewrite_location("http://127.0.0.1:3000", 3000).as_deref(), Some("/"));
        assert_eq!(
            rewrite_location("http://127.0.0.1:3000/app?x=1", 3000).as_deref(),
            Some("/app?x=1")
        );
        assert_eq!(
            rewrite_location("http://localhost:3000/x", 3000).as_deref(),
            Some("/x")
        );
        assert_eq!(rewrite_location("/already", 3000), None);
        assert_eq!(rewrite_location("https://evil.example/x", 3000), None);
    }

    async fn spawn_backend() -> u16 {
        let app = Router::new()
            .route("/hello", get(|| async { "hello from backend" }))
            .route("/xff", get(|headers: axum::http::HeaderMap| async move {
                headers
                    .get("x-forwarded-for")
                    .and_then(|v| v.to_str().ok())
                    .unwrap_or("")
                    .to_string()
            }))
            .route("/go", get(|| async {
                ([(header::LOCATION, "http://127.0.0.1:9/next")], StatusCode::FOUND)
            }));
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });
        port
    }

    async fn spawn_proxy(backend: u16) -> u16 {
        let state = AppState {
            port: backend,
            client: client(),
            last_activity: Arc::new(Mutex::new(Instant::now())),
            tunneled: false,
            gate: Gate::off(),
            limit: Arc::new(Semaphore::new(MAX_PROXY_CONCURRENT)),
        };
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let app = router(state);
        tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });
        port
    }

    #[tokio::test]
    async fn forwards_get() {
        let backend = spawn_backend().await;
        let port = spawn_proxy(backend).await;
        let client = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .unwrap();
        for _ in 0..50 {
            if client
                .get(format!("http://127.0.0.1:{port}/hello"))
                .send()
                .await
                .is_ok()
            {
                break;
            }
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
        let res = client
            .get(format!("http://127.0.0.1:{port}/hello"))
            .send()
            .await
            .unwrap();
        assert_eq!(res.status(), reqwest::StatusCode::OK);
        assert_eq!(res.text().await.unwrap(), "hello from backend");
        let xff = client
            .get(format!("http://127.0.0.1:{port}/xff"))
            .header("x-forwarded-for", "127.0.0.1")
            .send()
            .await
            .unwrap();
        assert_eq!(xff.text().await.unwrap(), "");
    }

    #[tokio::test]
    async fn unknown_backend_is_bad_gateway() {
        let port = spawn_proxy(1).await;
        let client = reqwest::Client::new();
        let res = client
            .get(format!("http://127.0.0.1:{port}/"))
            .send()
            .await
            .unwrap();
        assert_eq!(res.status(), reqwest::StatusCode::BAD_GATEWAY);
    }

    #[tokio::test]
    async fn password_shows_pin_form_not_basic_auth() {
        let backend = spawn_backend().await;
        let gate = Gate::pin(false);
        let pin = gate.pin_display().unwrap();
        let state = AppState {
            port: backend,
            client: client(),
            last_activity: Arc::new(Mutex::new(Instant::now())),
            tunneled: false,
            gate,
            limit: Arc::new(Semaphore::new(MAX_PROXY_CONCURRENT)),
        };
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let app = router(state);
        tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });
        let client = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .unwrap();
        let denied = client
            .get(format!("http://127.0.0.1:{port}/hello"))
            .send()
            .await
            .unwrap();
        assert_eq!(denied.status(), reqwest::StatusCode::OK);
        assert!(denied
            .headers()
            .get(reqwest::header::WWW_AUTHENTICATE)
            .is_none());
        let html = denied.text().await.unwrap();
        assert!(html.contains("desktop code"));
        assert!(html.contains(UNLOCK_PATH));

        let unlocked = client
            .post(format!("http://127.0.0.1:{port}{UNLOCK_PATH}"))
            .header("content-type", "application/x-www-form-urlencoded")
            .body(format!("password={pin}&next=/hello"))
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
        let ok = client
            .get(format!("http://127.0.0.1:{port}/hello"))
            .header(reqwest::header::COOKIE, cookie)
            .send()
            .await
            .unwrap();
        assert_eq!(ok.status(), reqwest::StatusCode::OK);
        assert_eq!(ok.text().await.unwrap(), "hello from backend");
    }
}
