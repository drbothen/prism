//! Stub configuration types: [`StubConfig`] and [`FailureMode`].

/// Top-level configuration for a DTU behavioral clone stub.
#[derive(Debug, Clone)]
pub struct StubConfig {
    /// Seed for the deterministic RNG (ChaCha20). Never use `thread_rng()`.
    pub seed: u64,
    /// Artificial latency injected by [`crate::LatencyLayer`], in milliseconds.
    pub latency_ms: u64,
    /// Failure injection mode applied by [`crate::FailureLayer`].
    pub failure_mode: FailureMode,
}

impl Default for StubConfig {
    fn default() -> Self {
        Self {
            seed: 42,
            latency_ms: 0,
            failure_mode: FailureMode::None,
        }
    }
}

/// Failure injection modes for [`crate::FailureLayer`].
#[derive(Debug, Clone)]
pub enum FailureMode {
    /// No failure injection; all requests pass through normally.
    None,
    /// Return HTTP 429 after `after_n_requests` successful requests.
    RateLimit {
        after_n_requests: u32,
        retry_after_secs: u32,
    },
    /// Return HTTP 500 on request number `at_request_n` (1-indexed).
    InternalError { at_request_n: u32 },
    /// Delay the response until the client times out.
    NetworkTimeout { after_ms: u64 },
    /// Reject all requests with HTTP 401.
    AuthReject,
}
