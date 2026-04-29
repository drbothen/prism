//! Stub configuration types: [`StubConfig`], [`FailureMode`], and [`DtuMode`].

/// Deployment-time DTU operating mode (BC-3.2.005).
///
/// Set once at startup from the TOML config field `mode = "shared"` or `mode = "client"`.
/// This enum is immutable after startup — no setter methods are provided post-construction.
/// Serde deserialization rejects any value other than `"shared"` or `"client"` with a
/// human-readable error (BC-3.2.005 postcondition 3, AC-006).
///
/// # Constraints
/// - `#[derive(Debug, Clone, Copy, PartialEq, Eq)]` — no interior mutability.
/// - Security Telemetry DTU types (claroty, armis, crowdstrike, cyberint) must reject
///   `DtuMode::Shared` at startup (EC-005 / BC-3.2.005).
/// - `DtuMode` MUST NOT appear in OCSF-normalized event records (BC-3.2.004 postcondition 5).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DtuMode {
    /// Shared infrastructure mode: one DTU instance serves all orgs.
    ///
    /// `OrgId` is embedded in each outgoing payload body for attribution (ADR-007 §2.6 Step 3).
    /// The state store is NOT re-keyed by OrgId (ADR-008 §1.2).
    Shared,
    /// Client-dedicated mode: one DTU instance per client org.
    ///
    /// Used by Security Telemetry DTU types (claroty, armis, crowdstrike, cyberint).
    /// Mixing `mode = "shared"` with a Security Telemetry type is a startup error.
    Client,
}

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
    /// Deployment-time operating mode for this DTU clone (BC-3.2.005).
    ///
    /// Defaults to `DtuMode::Client`. The Slack DTU overrides this to `DtuMode::Shared`
    /// via `DTU_DEFAULT_MODE` in `prism-dtu-slack/src/clone.rs`.
    pub mode: DtuMode,
}

impl Default for StubConfig {
    fn default() -> Self {
        Self {
            seed: 42,
            latency_ms: 0,
            failure_mode: FailureMode::None,
            bind: None, // None = 127.0.0.1:0 (OS-assigned)
            mode: DtuMode::Client,
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
