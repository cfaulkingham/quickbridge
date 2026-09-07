use super::*;
use axum::{
    body::{to_bytes, Body},
    extract::Request,
    http::{HeaderMap, StatusCode},
    routing::{get, post},
    Router,
};
use http_body_util::BodyExt;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

async fn spawn(app: Router, address: &str) -> (u16, tokio::task::JoinHandle<()>) {
    let listener = TcpListener::bind(address).await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let handle = tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    (port, handle)
}

#[tokio::test]
async fn independent_download_streams_start_at_zero() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("data.bin");
    let expected: Vec<u8> = (0..256 * 1024).map(|i| (i / (64 * 1024)) as u8).collect();
    std::fs::write(&path, &expected).unwrap();
    let share = share::open_file(path, 1024 * 1024, true).unwrap();
    let mut a = share.body();
    let mut b = share.body();
    let mut first = a
        .frame()
        .await
        .unwrap()
        .unwrap()
        .into_data()
        .unwrap()
        .to_vec();
    assert_eq!(share.read_prefix(10).unwrap(), expected[..10]);
    let mut second = b
        .frame()
        .await
        .unwrap()
        .unwrap()
        .into_data()
        .unwrap()
        .to_vec();
    first.extend_from_slice(&to_bytes(a, usize::MAX).await.unwrap());
    second.extend_from_slice(&to_bytes(b, usize::MAX).await.unwrap());
    assert_eq!(first, expected);
    assert_eq!(second, expected);
}

async fn start_upload(
    dir: &std::path::Path,
    max_files: u32,
    max_session_bytes: u64,
) -> (u16, upload::AppState, tokio::task::JoinHandle<()>) {
    let (shutdown, _) = watch::channel(false);
    let state = upload::AppState {
        token: "test".into(),
        dest: DestDir::open_existing(dir).unwrap(),
        max_bytes: 1024,
        max_session_bytes,
        max_files,
        last_activity: Arc::new(Mutex::new(Instant::now())),
        used: Arc::new(Mutex::new(SessionUsed { bytes: 0, files: 0 })),
        stop_after: false,
        shutdown,
        gate: gate::Gate::off(),
    };
    let (port, handle) = spawn(upload::router(state.clone()), "127.0.0.1:0").await;
    (port, state, handle)
}

async fn partial_upload(port: u16, filename: &str, content: &str) -> (TcpStream, String) {
    let first = format!("--boundary\r\nContent-Disposition: form-data; name=\"file\"; filename=\"{filename}\"\r\nContent-Type: application/octet-stream\r\n\r\n{content}");
    let tail = "\r\n--boundary--\r\n".to_string();
    let request = format!("POST /s/test/upload HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\nContent-Type: multipart/form-data; boundary=boundary\r\nContent-Length: {}\r\n\r\n{first}", first.len() + tail.len());
    let mut stream = TcpStream::connect(("127.0.0.1", port)).await.unwrap();
    stream.write_all(request.as_bytes()).await.unwrap();
    (stream, tail)
}

async fn wait_parts(dir: &std::path::Path, n: usize) {
    tokio::time::timeout(Duration::from_secs(2), async {
        loop {
            let count = std::fs::read_dir(dir)
                .unwrap()
                .flatten()
                .filter(|f| f.file_name().to_string_lossy().ends_with(".part"))
                .count();
            if count == n {
                return;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .unwrap();
}

async fn finish_upload((mut stream, tail): (TcpStream, String)) -> String {
    if let Err(err) = stream.write_all(tail.as_bytes()).await {
        assert!(matches!(
            err.kind(),
            std::io::ErrorKind::BrokenPipe | std::io::ErrorKind::ConnectionReset
        ));
    }
    let mut buf = Vec::new();
    let result = tokio::time::timeout(Duration::from_secs(2), stream.read_to_end(&mut buf))
        .await
        .unwrap();
    if let Err(err) = result {
        // Rejection can close a socket with unread request bytes. macOS may
        // deliver a reset instead of the response; valid uploads still require 200.
        assert_eq!(err.kind(), std::io::ErrorKind::ConnectionReset);
        if buf.is_empty() {
            return "connection reset".into();
        }
    }
    String::from_utf8(buf).unwrap()
}

#[tokio::test]
async fn concurrent_same_name_uploads_preserve_both_files() {
    let dir = tempfile::tempdir().unwrap();
    let (port, state, handle) = start_upload(dir.path(), 32, 8192).await;
    let a = partial_upload(port, "photo.jpg", "AAAA").await;
    wait_parts(dir.path(), 1).await;
    let b = partial_upload(port, "photo.jpg", "BBBB").await;
    wait_parts(dir.path(), 2).await;
    let first = finish_upload(a).await;
    let second = finish_upload(b).await;
    handle.abort();
    assert!(first.contains("200 OK") && second.contains("200 OK"));
    assert_eq!(state.used.lock().unwrap().files, 2);
    assert_eq!(
        std::fs::read(dir.path().join("photo.jpg")).unwrap(),
        b"AAAA"
    );
    assert_eq!(
        std::fs::read(dir.path().join("photo-1.jpg")).unwrap(),
        b"BBBB"
    );
    assert_eq!(
        std::fs::read_dir(dir.path()).unwrap().count(),
        2,
        "two successful uploads must leave two files"
    );
}

#[tokio::test]
async fn concurrent_uploads_enforce_session_quota() {
    let dir = tempfile::tempdir().unwrap();
    let (port, state, handle) = start_upload(dir.path(), 1, 4).await;
    let a = partial_upload(port, "a.txt", "AAAA").await;
    wait_parts(dir.path(), 1).await;
    let b = partial_upload(port, "b.txt", "BBBB").await;
    let rejected = finish_upload(b).await;
    assert!(
        rejected.contains("400 Bad Request") || rejected == "connection reset",
        "{rejected}"
    );
    assert!(finish_upload(a).await.contains("200 OK"));
    handle.abort();
    let used = state.used.lock().unwrap();
    assert!(
        used.files <= 1 && used.bytes <= 4,
        "quota exceeded: {} files, {} bytes",
        used.files,
        used.bytes
    );
}

async fn start_proxy_at(
    target: SocketAddr,
) -> (
    u16,
    String,
    Arc<tokio::sync::Semaphore>,
    tokio::task::JoinHandle<()>,
) {
    let gate = gate::Gate::pin(false);
    let cookie = gate
        .cookie_header()
        .to_str()
        .unwrap()
        .split(';')
        .next()
        .unwrap()
        .to_string();
    let limit = Arc::new(tokio::sync::Semaphore::new(proxy::MAX_PROXY_CONCURRENT));
    let state = proxy::AppState {
        target,
        client: proxy::client(),
        last_activity: Arc::new(Mutex::new(Instant::now())),
        tunneled: false,
        gate,
        limit: limit.clone(),
    };
    let (port, handle) = spawn(proxy::router(state), "127.0.0.1:0").await;
    (port, cookie, limit, handle)
}

async fn start_proxy(
    backend: u16,
) -> (
    u16,
    String,
    Arc<tokio::sync::Semaphore>,
    tokio::task::JoinHandle<()>,
) {
    start_proxy_at(SocketAddr::from(([127, 0, 0, 1], backend))).await
}

#[tokio::test]
async fn proxy_rejects_body_over_32_mib() {
    let app = Router::new().route(
        "/",
        post(|req: Request| async move {
            to_bytes(req.into_body(), usize::MAX)
                .await
                .unwrap()
                .len()
                .to_string()
        }),
    );
    let (backend, b) = spawn(app, "127.0.0.1:0").await;
    let (port, cookie, _, p) = start_proxy(backend).await;
    // A declared oversize request should be rejected before uploading its body.
    let mut stream = TcpStream::connect(("127.0.0.1", port)).await.unwrap();
    stream.write_all(format!("POST / HTTP/1.1\r\nHost: localhost\r\nCookie: {cookie}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n", proxy::MAX_PROXY_BYTES + 1).as_bytes()).await.unwrap();
    let mut response = String::new();
    tokio::time::timeout(Duration::from_secs(2), stream.read_to_string(&mut response))
        .await
        .unwrap()
        .unwrap();
    b.abort();
    p.abort();
    assert!(response.starts_with("HTTP/1.1 413"), "{response}");
}

#[tokio::test]
async fn proxy_preserves_backend_bearer_auth_after_pin_cookie() {
    let app = Router::new().route(
        "/",
        get(|headers: HeaderMap| async move {
            headers
                .get("authorization")
                .and_then(|v| v.to_str().ok())
                .unwrap_or("")
                .to_string()
        }),
    );
    let (backend, b) = spawn(app, "127.0.0.1:0").await;
    let (port, cookie, _, p) = start_proxy(backend).await;
    let body = reqwest::Client::new()
        .get(format!("http://127.0.0.1:{port}/"))
        .header("cookie", cookie)
        .bearer_auth("backend-token")
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    b.abort();
    p.abort();
    assert_eq!(body, "Bearer backend-token");
}

#[tokio::test]
async fn proxy_holds_permit_during_response_body() {
    let app = Router::new().route(
        "/",
        get(|| async {
            let (read, write) = tokio::io::duplex(1024);
            tokio::spawn(async move {
                std::future::pending::<()>().await;
                drop(write);
            });
            Body::from_stream(tokio_util::io::ReaderStream::new(read))
        }),
    );
    let (backend, b) = spawn(app, "127.0.0.1:0").await;
    let (port, cookie, limit, p) = start_proxy(backend).await;
    let response = reqwest::Client::new()
        .get(format!("http://127.0.0.1:{port}/"))
        .header("cookie", cookie)
        .send()
        .await
        .unwrap();
    let permits = limit.available_permits();
    drop(response);
    b.abort();
    p.abort();
    assert_eq!(
        permits,
        proxy::MAX_PROXY_CONCURRENT - 1,
        "unfinished response must retain its permit"
    );
}

#[tokio::test]
async fn accepted_ipv6_backend_is_reachable_through_proxy() {
    let listener = TcpListener::bind("[::1]:0").await.unwrap();
    let backend = listener.local_addr().unwrap().port();
    let b = tokio::spawn(async move {
        loop {
            let (mut stream, _) = listener.accept().await.unwrap();
            tokio::spawn(async move {
                let mut req = vec![0u8; 4096];
                let _ = stream.read(&mut req).await;
                let _ = stream
                    .write_all(
                        b"HTTP/1.1 200 OK\r\nContent-Length: 5\r\nConnection: close\r\n\r\nhello",
                    )
                    .await;
            });
        }
    });
    let target = ports::confirm_local_http(backend).await.unwrap();
    assert!(target.is_ipv6());
    let (port, cookie, _, p) = start_proxy_at(target).await;
    let response = reqwest::Client::new()
        .get(format!("http://127.0.0.1:{port}/"))
        .header("cookie", cookie)
        .send()
        .await
        .unwrap();
    b.abort();
    p.abort();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn head_request_does_not_finish_download_session() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("data.txt");
    std::fs::write(&path, b"hello").unwrap();
    let (shutdown, mut rx) = watch::channel(false);
    let state = download::AppState {
        token: "test".into(),
        share: share::open_file(path, 100, false).unwrap(),
        last_activity: Arc::new(Mutex::new(Instant::now())),
        stop_after: true,
        shutdown,
        gate: gate::Gate::off(),
    };
    let (port, p) = spawn(download::router(state), "127.0.0.1:0").await;
    reqwest::Client::new()
        .head(format!("http://127.0.0.1:{port}/s/test/file"))
        .send()
        .await
        .unwrap();
    let stopped = tokio::time::timeout(Duration::from_millis(500), rx.wait_for(|v| *v))
        .await
        .is_ok();
    p.abort();
    assert!(
        !stopped,
        "HEAD transferred no file, but scheduled session shutdown"
    );
}

#[tokio::test]
async fn shutdown_is_bounded_with_stalled_upload() {
    let arrived = Arc::new(tokio::sync::Notify::new());
    let notify = arrived.clone();
    let app = Router::new().route(
        "/",
        post(move |req: Request| {
            let notify = notify.clone();
            async move {
                notify.notify_one();
                let _ = to_bytes(req.into_body(), 100).await;
                "ok"
            }
        }),
    );
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let (tx, rx) = watch::channel(false);
    let mut server = tokio::spawn(serve_until_ready(
        listener,
        app,
        Arc::new(Mutex::new(Instant::now())),
        Instant::now(),
        Duration::from_secs(60),
        Duration::from_secs(120),
        rx,
    ));
    let mut client = TcpStream::connect(("127.0.0.1", port)).await.unwrap();
    client
        .write_all(b"POST / HTTP/1.1\r\nHost: localhost\r\nContent-Length: 5\r\n\r\na")
        .await
        .unwrap();
    arrived.notified().await;
    tx.send(true).unwrap();
    let stopped = tokio::time::timeout(Duration::from_secs(3), &mut server)
        .await
        .is_ok();
    server.abort();
    assert!(stopped, "shutdown waits indefinitely on the stalled body");
    let mut remaining = Vec::new();
    assert!(
        tokio::time::timeout(
            Duration::from_millis(500),
            client.read_to_end(&mut remaining)
        )
        .await
        .is_ok(),
        "server returned but an active socket stayed open"
    );
}

#[tokio::test]
async fn session_byte_quota_is_reserved_independently_of_file_count() {
    let dir = tempfile::tempdir().unwrap();
    let (port, state, handle) = start_upload(dir.path(), 32, 4).await;
    let a = partial_upload(port, "a.txt", "AAAA").await;
    wait_parts(dir.path(), 1).await;
    let b = partial_upload(port, "b.txt", "BBBB").await;
    let rejected = finish_upload(b).await;
    assert!(
        rejected.contains("session size limit reached") || rejected == "connection reset",
        "{rejected}"
    );
    assert!(finish_upload(a).await.contains("200 OK"));
    handle.abort();
    let used = state.used.lock().unwrap();
    assert_eq!((used.files, used.bytes), (1, 4));
}

#[tokio::test]
async fn failed_empty_and_canceled_uploads_release_reservations_and_temp_files() {
    let dir = tempfile::tempdir().unwrap();
    let (port, state, handle) = start_upload(dir.path(), 1, 1024).await;
    for content in [String::new(), "x".repeat(1025)] {
        let response = finish_upload(partial_upload(port, "bad.txt", &content).await).await;
        assert!(
            response.contains("400 Bad Request") || response == "connection reset",
            "{response}"
        );
        assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 0);
        let used = state.used.lock().unwrap();
        assert_eq!((used.files, used.bytes), (0, 0));
    }
    let canceled = partial_upload(port, "canceled.txt", "AAAA").await;
    wait_parts(dir.path(), 1).await;
    drop(canceled);
    tokio::time::timeout(Duration::from_secs(2), async {
        loop {
            if state.used.lock().unwrap().files == 0
                && std::fs::read_dir(dir.path()).unwrap().count() == 0
            {
                break;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .unwrap();
    assert!(
        finish_upload(partial_upload(port, "good.txt", "okay").await)
            .await
            .contains("200 OK")
    );
    handle.abort();
    let used = state.used.lock().unwrap();
    assert_eq!((used.files, used.bytes), (1, 4));
}

#[tokio::test]
async fn proxy_rejects_chunked_body_over_limit() {
    use std::sync::atomic::{AtomicUsize, Ordering};
    let received = Arc::new(AtomicUsize::new(0));
    let seen = received.clone();
    let app = Router::new().route(
        "/",
        post(move |request: Request| {
            let seen = seen.clone();
            async move {
                let mut body = request.into_body();
                while let Some(frame) = body.frame().await {
                    let Ok(frame) = frame else {
                        return StatusCode::BAD_REQUEST;
                    };
                    if let Some(data) = frame.data_ref() {
                        seen.fetch_add(data.len(), Ordering::SeqCst);
                    }
                }
                StatusCode::OK
            }
        }),
    );
    let (backend, b) = spawn(app, "127.0.0.1:0").await;
    let (port, cookie, _, p) = start_proxy(backend).await;
    let reader = std::io::Cursor::new(vec![b'a'; proxy::MAX_PROXY_BYTES + 1]);
    let body = Body::from_stream(tokio_util::io::ReaderStream::new(reader));
    let request = hyper::Request::builder()
        .method("POST")
        .uri(format!("http://127.0.0.1:{port}/"))
        .header("cookie", cookie)
        .body(body)
        .unwrap();
    let response = tokio::time::timeout(Duration::from_secs(5), proxy::client().request(request))
        .await
        .unwrap();
    if let Ok(response) = response {
        assert_ne!(response.status(), StatusCode::OK);
    }
    assert!(received.load(Ordering::SeqCst) <= proxy::MAX_PROXY_BYTES);
    b.abort();
    p.abort();
}

#[tokio::test]
async fn proxy_caps_responses_and_releases_failed_transfer_permits() {
    let app = Router::new().route(
        "/",
        get(|| async { Body::from(vec![0; proxy::MAX_PROXY_BYTES + 1]) }),
    );
    let (backend, b) = spawn(app, "127.0.0.1:0").await;
    let (port, cookie, limit, p) = start_proxy(backend).await;
    let response = reqwest::Client::new()
        .get(format!("http://127.0.0.1:{port}/"))
        .header("cookie", cookie)
        .send()
        .await;
    if let Ok(response) = response {
        assert!(response.bytes().await.is_err());
    }
    tokio::time::timeout(Duration::from_secs(1), async {
        while limit.available_permits() != proxy::MAX_PROXY_CONCURRENT {
            tokio::task::yield_now().await;
        }
    })
    .await
    .unwrap();
    b.abort();
    p.abort();
}

#[tokio::test]
async fn only_complete_download_bodies_signal_success() {
    use std::sync::atomic::{AtomicUsize, Ordering};
    let completions = Arc::new(AtomicUsize::new(0));
    let tracked = |body, size| {
        let completions = completions.clone();
        Body::new(transfer::TransferBody::download(body, size, move || {
            completions.fetch_add(1, Ordering::SeqCst);
        }))
    };
    drop(tracked(Body::from("hello"), 5));
    assert!(to_bytes(tracked(Body::from("short"), 10), 100)
        .await
        .is_err());
    assert_eq!(completions.load(Ordering::SeqCst), 0);
    let bytes = to_bytes(tracked(Body::from("hello"), 5), 100)
        .await
        .unwrap();
    assert_eq!(&bytes[..], b"hello");
    assert_eq!(completions.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn successful_get_stops_download_but_truncated_file_allows_retry() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("data.bin");
    std::fs::write(&path, b"hello").unwrap();
    let share = share::open_file(path.clone(), 100, false).unwrap();
    let (shutdown, mut rx) = watch::channel(false);
    let state = download::AppState {
        token: "test".into(),
        share,
        last_activity: Arc::new(Mutex::new(Instant::now())),
        stop_after: true,
        shutdown,
        gate: gate::Gate::off(),
    };
    let (port, p) = spawn(download::router(state), "127.0.0.1:0").await;
    let client = reqwest::Client::new();
    let url = format!("http://127.0.0.1:{port}/s/test/file");
    std::fs::write(&path, b"x").unwrap();
    if let Ok(response) = client.get(&url).send().await {
        assert!(response.bytes().await.is_err());
    }
    assert!(
        tokio::time::timeout(Duration::from_millis(350), rx.wait_for(|v| *v))
            .await
            .is_err()
    );
    std::fs::write(&path, b"hello").unwrap();
    assert_eq!(
        &client
            .get(&url)
            .send()
            .await
            .unwrap()
            .bytes()
            .await
            .unwrap()[..],
        b"hello"
    );
    assert!(
        tokio::time::timeout(Duration::from_secs(1), rx.wait_for(|v| *v))
            .await
            .is_ok()
    );
    p.abort();
}

#[tokio::test]
async fn proxy_body_deadline_and_cancellation_release_permit() {
    let limit = Arc::new(tokio::sync::Semaphore::new(1));
    let (reader, writer) = tokio::io::duplex(10);
    let body = Body::from_stream(tokio_util::io::ReaderStream::new(reader));
    let body = transfer::TransferBody::proxy(
        body,
        tokio::time::Instant::now() + Duration::from_millis(20),
        limit.clone().acquire_owned().await.unwrap(),
    );
    assert!(
        tokio::time::timeout(Duration::from_secs(1), to_bytes(Body::new(body), 100))
            .await
            .unwrap()
            .is_err()
    );
    assert_eq!(limit.available_permits(), 1);
    drop(writer);
    let canceled = transfer::TransferBody::proxy(
        Body::from("hello"),
        tokio::time::Instant::now() + Duration::from_secs(30),
        limit.clone().acquire_owned().await.unwrap(),
    );
    drop(canceled);
    assert_eq!(limit.available_permits(), 1);
}

#[tokio::test]
async fn socket_deadline_closes_connection_when_client_stops_reading() {
    let app = Router::new().route(
        "/",
        get(|request: Request| async move {
            request
                .extensions()
                .get::<transfer::ConnectionDeadline>()
                .unwrap()
                .arm(tokio::time::Instant::now() + Duration::from_millis(50));
            Body::from(vec![b'x'; 16 * 1024 * 1024])
        }),
    );
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let (tx, rx) = watch::channel(false);
    let server = tokio::spawn(serve_until_ready(
        listener,
        app,
        Arc::new(Mutex::new(Instant::now())),
        Instant::now(),
        Duration::from_secs(60),
        Duration::from_secs(120),
        rx,
    ));
    let mut client = TcpStream::connect(("127.0.0.1", port)).await.unwrap();
    client
        .write_all(b"GET / HTTP/1.1\r\nHost: localhost\r\n\r\n")
        .await
        .unwrap();
    tokio::time::sleep(Duration::from_millis(150)).await;
    let mut bytes = Vec::new();
    let _ = tokio::time::timeout(Duration::from_secs(2), client.read_to_end(&mut bytes))
        .await
        .unwrap();
    assert!(
        bytes.len() < 16 * 1024 * 1024,
        "the deadline must interrupt the unfinished response"
    );
    tx.send(true).unwrap();
    server.await.unwrap().unwrap();
}

#[tokio::test]
async fn idle_and_wall_timeouts_abort_uploads_and_clean_up() {
    for idle_expires_first in [true, false] {
        let dir = tempfile::tempdir().unwrap();
        let (_, state, unused_server) = start_upload(dir.path(), 1, 1024).await;
        unused_server.abort();
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let rx = state.shutdown.subscribe();
        let short = Duration::from_millis(150);
        let long = Duration::from_secs(60);
        let (idle, wall) = if idle_expires_first {
            (short, long)
        } else {
            (long, short)
        };
        let server = tokio::spawn(serve_until_ready(
            listener,
            upload::router(state.clone()),
            state.last_activity.clone(),
            Instant::now(),
            idle,
            wall,
            rx,
        ));
        let (mut client, _) = partial_upload(port, "stalled.txt", "AAAA").await;
        wait_parts(dir.path(), 1).await;
        tokio::time::timeout(Duration::from_secs(3), server)
            .await
            .unwrap()
            .unwrap()
            .unwrap();
        assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 0);
        let mut buf = Vec::new();
        assert!(
            tokio::time::timeout(Duration::from_millis(500), client.read_to_end(&mut buf))
                .await
                .is_ok()
        );
        let used = state.used.lock().unwrap();
        assert_eq!((used.files, used.bytes), (0, 0));
    }
}
