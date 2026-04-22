//! [`LatencyLayer`] — Tower layer that injects artificial latency into responses.

use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use tower::Service;

/// Tower [`tower::Layer`] that wraps a service with configurable artificial latency.
#[derive(Clone)]
pub struct LatencyLayer {
    pub latency_ms: u64,
}

impl<S> tower::Layer<S> for LatencyLayer {
    type Service = LatencyMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        LatencyMiddleware {
            inner,
            latency_ms: self.latency_ms,
        }
    }
}

/// Middleware produced by [`LatencyLayer`].
#[derive(Clone)]
pub struct LatencyMiddleware<S> {
    inner: S,
    latency_ms: u64,
}

impl<S, Req> Service<Req> for LatencyMiddleware<S>
where
    S: Service<Req> + Send + 'static,
    S::Future: Send + 'static,
    Req: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Req) -> Self::Future {
        todo!("implement latency injection per AC-4")
    }
}
