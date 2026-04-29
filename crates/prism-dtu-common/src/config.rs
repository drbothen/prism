//! Stub configuration types: [`StubConfig`], [`FailureMode`], and [`DtuMode`].
//!
//! # DtuMode reconciliation (chore(S-3.2.05))
//!
//! `DtuMode` is re-exported from `prism_core::dtu` rather than defined here.
//! Rationale: S-3.0.02 introduced the authoritative `DtuMode` enum in `prism-core`
//! with `#[serde(rename_all = "lowercase")]` + `Deserialize` already wired up.
//! Maintaining a duplicate definition in this crate (without serde) would create
//! two incompatible `DtuMode` types in the workspace, break the prism-core mode
//! registry lookups, and violate BC-3.2.005 invariant 1 ("no interior mutability,
//! value type"). The re-export unifies all DTU crates on a single, serde-capable
//! `DtuMode` type.
//!
//! Decision: option (a) — `prism_dtu_common::DtuMode` re-exports `prism_core::DtuMode`.
//! The `dtu` feature now depends on `prism-core` so this re-export is always available
//! when the crate is used by consumers.

/// Re-export the authoritative `DtuMode` enum from `prism-core`.
///
/// `DtuMode` is defined in `prism_core::dtu` (S-3.0.02) with:
/// - `#[derive(Debug, Clone, Copy, PartialEq, Eq)]` — no interior mutability.
/// - `#[serde(rename_all = "lowercase")]` + `Deserialize` — rejects unknown
///   variants (e.g. `"Hybrid"`) at deserialization time (BC-3.2.005 postcondition 3).
///
/// All DTU clone crates (`prism-dtu-slack`, `prism-dtu-nvd`, …) import `DtuMode`
/// through this re-export so there is exactly one definition in the workspace.
pub use prism_core::DtuMode;

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
    /// Defaults to `DtuMode::Client`. The Slack DTU registers as `DtuMode::Shared`
    /// in the `prism-core` mode registry (ADR-007 §2.3).
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
