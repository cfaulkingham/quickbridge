use std::future::Future;
use std::io;
use std::pin::Pin;
use std::task::{Context, Poll};

use axum::body::{Body, Bytes};
use hyper::body::{Body as HttpBody, Frame, SizeHint};
use tokio::sync::{watch, OwnedSemaphorePermit};
use tokio::time::{Instant, Sleep};

/// The server also enforces the deadline on the socket, since downstream
/// backpressure can stop Hyper from polling the response body entirely.
#[derive(Clone)]
pub struct ConnectionDeadline(pub watch::Sender<Option<Instant>>);

impl ConnectionDeadline {
    pub fn arm(&self, deadline: Instant) {
        self.0.send_replace(Some(deadline));
    }
}

pub async fn wait_deadline(mut receiver: watch::Receiver<Option<Instant>>) {
    loop {
        let deadline = *receiver.borrow_and_update();
        match deadline {
            Some(deadline) => tokio::select! {
                biased;
                _ = tokio::time::sleep_until(deadline) => return,
                changed = receiver.changed() => {
                    if changed.is_err() { return; }
                }
            },
            None => {
                if receiver.changed().await.is_err() {
                    return;
                }
            }
        }
    }
}

/// Keep transfer resources alive until the body finishes, fails, or is dropped.
pub struct TransferBody {
    inner: Body,
    remaining: Option<u64>,
    deadline: Option<Pin<Box<Sleep>>>,
    permit: Option<OwnedSemaphorePermit>,
    on_complete: Option<Box<dyn FnOnce() + Send>>,
    ended: bool,
}

impl TransferBody {
    pub fn download(inner: Body, size: u64, on_complete: impl FnOnce() + Send + 'static) -> Self {
        Self {
            inner,
            remaining: Some(size),
            deadline: None,
            permit: None,
            on_complete: Some(Box::new(on_complete)),
            ended: false,
        }
    }

    pub fn proxy(inner: Body, deadline: Instant, permit: OwnedSemaphorePermit) -> Self {
        Self {
            inner,
            remaining: None,
            deadline: Some(Box::pin(tokio::time::sleep_until(deadline))),
            permit: Some(permit),
            on_complete: None,
            ended: false,
        }
    }

    fn finish(&mut self, success: bool) {
        self.ended = true;
        self.permit.take();
        if let Some(complete) = self.on_complete.take() {
            if success {
                complete();
            }
        }
    }

    fn error(
        &mut self,
        kind: io::ErrorKind,
        message: &str,
    ) -> Poll<Option<Result<Frame<Bytes>, axum::Error>>> {
        self.finish(false);
        self.inner = Body::empty();
        Poll::Ready(Some(Err(axum::Error::new(io::Error::new(kind, message)))))
    }
}

impl HttpBody for TransferBody {
    type Data = Bytes;
    type Error = axum::Error;

    fn poll_frame(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Frame<Bytes>, axum::Error>>> {
        if self.ended {
            return Poll::Ready(None);
        }
        if let Some(deadline) = &mut self.deadline {
            if deadline.as_mut().poll(cx).is_ready() {
                return self.error(io::ErrorKind::TimedOut, "proxy transfer timed out");
            }
        }
        match std::task::ready!(Pin::new(&mut self.inner).poll_frame(cx)) {
            Some(Ok(frame)) => {
                if let (Some(remaining), Some(data)) = (&mut self.remaining, frame.data_ref()) {
                    let Some(left) = remaining.checked_sub(data.len() as u64) else {
                        return self.error(
                            io::ErrorKind::InvalidData,
                            "download exceeded its declared size",
                        );
                    };
                    *remaining = left;
                }
                // Hyper need not poll EOF after a known Content-Length is satisfied.
                if self.remaining == Some(0)
                    || (self.remaining.is_none() && self.inner.is_end_stream())
                {
                    self.finish(true);
                }
                Poll::Ready(Some(Ok(frame)))
            }
            Some(Err(err)) => {
                self.finish(false);
                self.inner = Body::empty();
                Poll::Ready(Some(Err(err)))
            }
            None => {
                if self.remaining.is_some_and(|n| n != 0) {
                    return self.error(
                        io::ErrorKind::UnexpectedEof,
                        "download ended before its declared size",
                    );
                }
                self.finish(true);
                Poll::Ready(None)
            }
        }
    }

    fn is_end_stream(&self) -> bool {
        self.ended
    }
    fn size_hint(&self) -> SizeHint {
        match self.remaining {
            Some(n) => SizeHint::with_exact(n),
            None => self.inner.size_hint(),
        }
    }
}
