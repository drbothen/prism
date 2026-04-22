//! [`WebhookReceiver`] — Generic HTTP POST capture server.

use axum::body::Body;
use axum::extract::{Request, State};
use axum::http::StatusCode;
use axum::routing::any;
use axum::Router;
use bytes::Bytes;
use http::HeaderMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

/// A captured HTTP POST request recorded by [`WebhookReceiver`].
#[derive(Debug, Clone)]
pub struct CapturedRequest {
    pub path: String,
    pub body: Bytes,
    pub headers: HeaderMap,
}

/// Captures inbound HTTP POST requests for assertion in integration tests.
pub struct WebhookReceiver {
    bound_addr: SocketAddr,
    captured: Arc<Mutex<Vec<CapturedRequest>>>,
}

impl WebhookReceiver {
    /// Start a webhook receiver on an OS-assigned port.
    pub async fn start() -> anyhow::Result<Self> {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
        let bound_addr = listener.local_addr()?;
        let captured: Arc<Mutex<Vec<CapturedRequest>>> = Arc::new(Mutex::new(Vec::new()));
        let state = captured.clone();

        let app = Router::new()
            .fallback(any(capture_handler))
            .with_state(state);

        tokio::spawn(async move {
            axum::serve(listener, app).await.ok();
        });

        Ok(Self {
            bound_addr,
            captured,
        })
    }

    /// Return a snapshot of all captured requests since the last [`reset`](Self::reset).
    pub fn received_payloads(&self) -> Vec<CapturedRequest> {
        self.captured
            .lock()
            .expect("captured lock poisoned")
            .clone()
    }

    /// Return the address the server is bound to.
    pub fn bound_addr(&self) -> SocketAddr {
        self.bound_addr
    }

    /// Clear all captured requests and reset internal state.
    pub fn reset(&self) {
        self.captured
            .lock()
            .expect("captured lock poisoned")
            .clear();
    }
}

/// Maximum body size accepted by [`WebhookReceiver`].
///
/// Requests exceeding this limit cause axum to return HTTP 413 Payload Too Large
/// automatically, preventing unbounded memory growth from oversized payloads.
const MAX_WEBHOOK_BODY_SIZE: usize = 1 * 1024 * 1024; // 1 MiB

async fn capture_handler(
    State(captured): State<Arc<Mutex<Vec<CapturedRequest>>>>,
    req: Request<Body>,
) -> StatusCode {
    let path = req.uri().path().to_owned();
    let headers = req.headers().clone();
    let body_bytes = axum::body::to_bytes(req.into_body(), MAX_WEBHOOK_BODY_SIZE)
        .await
        .unwrap_or_default();

    captured
        .lock()
        .expect("captured lock poisoned")
        .push(CapturedRequest {
            path,
            body: body_bytes,
            headers,
        });

    StatusCode::OK
}
