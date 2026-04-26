//! [`FailureLayer`] — Tower layer that injects configurable failure modes.

use crate::config::FailureMode;
use axum::body::Body;
use http::Response;
use std::future::Future;
use std::pin::Pin;
use std::sync::{
    atomic::{AtomicU32, Ordering},
    Arc, Mutex,
};
use std::task::{Context, Poll};
use tower::Service;

/// Tower [`tower::Layer`] that wraps a service with configurable failure injection.
///
/// The `mode` field accepts either a static [`FailureMode`] or a shared
/// `Arc<Mutex<FailureMode>>` for dynamic reconfiguration after the server starts.
/// Use [`FailureLayer::shared`] to create a layer backed by a shared mode.
#[derive(Clone)]
pub struct FailureLayer {
    pub mode: FailureMode,
}

impl FailureLayer {
    /// Create a `FailureLayer` backed by a shared `Arc<Mutex<FailureMode>>`.
    ///
    /// This variant reads the current mode on every request, allowing the failure
    /// mode to be updated at runtime (e.g., via `POST /dtu/configure`).
    pub fn shared(mode: Arc<Mutex<FailureMode>>) -> FailureLayerShared {
        FailureLayerShared { mode }
    }
}

impl<S> tower::Layer<S> for FailureLayer {
    type Service = FailureMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        FailureMiddleware {
            inner,
            mode: self.mode.clone(),
            request_count: Arc::new(AtomicU32::new(0)),
        }
    }
}

/// Dynamically-reconfigurable variant of [`FailureLayer`].
///
/// Reads `Arc<Mutex<FailureMode>>` on every request so that `POST /dtu/configure`
/// can update the failure mode without restarting the server.
#[derive(Clone)]
pub struct FailureLayerShared {
    pub mode: Arc<Mutex<FailureMode>>,
}

impl<S> tower::Layer<S> for FailureLayerShared {
    type Service = FailureMiddlewareShared<S>;

    fn layer(&self, inner: S) -> Self::Service {
        FailureMiddlewareShared {
            inner,
            mode: Arc::clone(&self.mode),
            request_count: Arc::new(AtomicU32::new(0)),
        }
    }
}

/// Middleware produced by [`FailureLayerShared`].
#[derive(Clone)]
pub struct FailureMiddlewareShared<S> {
    inner: S,
    mode: Arc<Mutex<FailureMode>>,
    request_count: Arc<AtomicU32>,
}

impl<S, Req> Service<Req> for FailureMiddlewareShared<S>
where
    S: Service<Req, Response = Response<Body>> + Send + 'static,
    S::Future: Send + 'static,
    S::Error: Send + 'static,
    Req: Send + 'static,
{
    type Response = Response<Body>;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Req) -> Self::Future {
        let count = self.request_count.fetch_add(1, Ordering::SeqCst) + 1;
        // SAFETY: mutex poisoning means a panic already occurred; propagating is correct.
        #[allow(clippy::expect_used)]
        let mode = self
            .mode
            .lock()
            .expect("FailureMiddlewareShared: mode lock poisoned")
            .clone();
        let fut = self.inner.call(req);

        Box::pin(apply_failure_mode(mode, count, fut))
    }
}

/// Middleware produced by [`FailureLayer`].
#[derive(Clone)]
pub struct FailureMiddleware<S> {
    inner: S,
    mode: FailureMode,
    request_count: Arc<AtomicU32>,
}

impl<S, Req> Service<Req> for FailureMiddleware<S>
where
    S: Service<Req, Response = Response<Body>> + Send + 'static,
    S::Future: Send + 'static,
    S::Error: Send + 'static,
    Req: Send + 'static,
{
    type Response = Response<Body>;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Req) -> Self::Future {
        let count = self.request_count.fetch_add(1, Ordering::SeqCst) + 1;
        let mode = self.mode.clone();
        let fut = self.inner.call(req);

        Box::pin(apply_failure_mode(mode, count, fut))
    }
}

/// Shared failure-mode dispatch logic used by both [`FailureMiddleware`] and
/// [`FailureMiddlewareShared`].
///
/// # Allow: expect_used
/// All `.expect()` calls here construct `Response::builder()` with static status codes
/// (401, 429, 500, 422, 200). These builders only fail if the status code is invalid,
/// which cannot happen with compile-time constants.
#[allow(clippy::expect_used)]
async fn apply_failure_mode<F, E>(
    mode: FailureMode,
    count: u32,
    fut: F,
) -> Result<Response<Body>, E>
where
    F: Future<Output = Result<Response<Body>, E>>,
{
    match mode {
        FailureMode::AuthReject => Ok(Response::builder()
            .status(401)
            .body(Body::empty())
            .expect("build 401 response")),
        FailureMode::RateLimit {
            after_n_requests,
            retry_after_secs,
        } => {
            if count > after_n_requests {
                Ok(Response::builder()
                    .status(429)
                    .header("Retry-After", retry_after_secs.to_string())
                    .body(Body::from("\"ratelimited\""))
                    .expect("build 429 response"))
            } else {
                fut.await
            }
        }
        FailureMode::InternalError { at_request_n } => {
            if count == at_request_n {
                Ok(Response::builder()
                    .status(500)
                    .body(Body::empty())
                    .expect("build 500 response"))
            } else {
                fut.await
            }
        }
        FailureMode::NetworkTimeout { after_ms } => {
            tokio::time::sleep(std::time::Duration::from_millis(after_ms + 1)).await;
            fut.await
        }
        FailureMode::Unprocessable { at_request_n } => {
            if count == at_request_n {
                Ok(Response::builder()
                    .status(422)
                    .body(Body::empty())
                    .expect("build 422 response"))
            } else {
                fut.await
            }
        }
        FailureMode::MalformedResponse => {
            // Return a response with raw bytes that cannot be parsed as JSON.
            // This exercises Prism's parse-error handling path (EC-006).
            Ok(Response::builder()
                .status(200)
                .header("Content-Type", "application/json")
                .body(Body::from(b"\xff\xfe{not valid json!@#$%^&*(" as &[u8]))
                .expect("build malformed response"))
        }
        FailureMode::None => fut.await,
    }
}
