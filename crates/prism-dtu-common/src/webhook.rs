//! [`WebhookReceiver`] — Generic HTTP POST capture server.

use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

/// A captured HTTP POST request recorded by [`WebhookReceiver`].
#[derive(Debug, Clone)]
pub struct CapturedRequest {
    pub path: String,
    pub body: bytes::Bytes,
    pub headers: http::HeaderMap,
}

/// Captures inbound HTTP POST requests for assertion in integration tests.
pub struct WebhookReceiver {
    bound_addr: SocketAddr,
    captured: Arc<Mutex<Vec<CapturedRequest>>>,
}

impl WebhookReceiver {
    /// Start a webhook receiver on an OS-assigned port.
    pub async fn start() -> anyhow::Result<Self> {
        todo!("implement WebhookReceiver::start per AC-8")
    }

    /// Return a snapshot of all captured requests since the last [`reset`](Self::reset).
    pub fn received_payloads(&self) -> Vec<CapturedRequest> {
        todo!("implement WebhookReceiver::received_payloads")
    }

    /// Return the address the server is bound to.
    pub fn bound_addr(&self) -> SocketAddr {
        self.bound_addr
    }

    /// Clear all captured requests and reset internal state.
    pub fn reset(&self) {
        todo!("implement WebhookReceiver::reset")
    }
}
