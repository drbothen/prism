//! [`FailureLayer`] — Tower layer that injects configurable failure modes.

use crate::config::FailureMode;
use axum::body::Body;
use http::Response;
use std::future::Future;
use std::pin::Pin;
use std::sync::{
    atomic::{AtomicU32, Ordering},
    Arc,
};
use std::task::{Context, Poll};
use tower::Service;

/// Tower [`tower::Layer`] that wraps a service with configurable failure injection.
#[derive(Clone)]
pub struct FailureLayer {
    pub mode: FailureMode,
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

        Box::pin(async move {
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
                            .body(Body::empty())
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
                FailureMode::None => fut.await,
            }
        })
    }
}
