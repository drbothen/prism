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
    /// Bind address override for standalone `start()` calls.
    ///
    /// `None` means "OS-assigned on 127.0.0.1:0" (the default).
    ///
    /// When the demo harness calls `clone.start_on(addr, shutdown)` it always
    /// passes an explicit `addr` — this field is consulted only by the `start()`
    /// default-impl shim. In practice, all harness-driven starts go through
    /// `start_on`. (ADR-002 Amendment §M4)
    pub bind: Option<std::net::SocketAddr>,
}

impl Default for StubConfig {
    fn default() -> Self {
        Self {
            seed: 42,
            latency_ms: 0,
            failure_mode: FailureMode::None,
            bind: None, // None = 127.0.0.1:0 (OS-assigned)
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
    /// Return HTTP 422 on request number `at_request_n` (1-indexed).
    /// Maps to `E-SENSOR-004` in Prism's error taxonomy (invalid filter syntax).
    Unprocessable { at_request_n: u32 },
    /// Return a response with a non-JSON body (exercises parse-error path in Prism).
    ///
    /// The response body is raw bytes that will fail JSON deserialization.
    MalformedResponse,
}
