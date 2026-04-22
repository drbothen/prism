//! [`FailureLayer`] — Tower layer that injects configurable failure modes.

use crate::config::FailureMode;
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
        let _count = self.request_count.fetch_add(1, Ordering::SeqCst);
        todo!("implement failure injection per AC-3")
    }
}
