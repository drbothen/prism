//! `prism-sensors` — Sensor adapter framework for the Prism platform.
//!
//! Defines the `SensorAdapter` async trait and supporting infrastructure that
//! makes the query engine's fan-out layer sensor-agnostic. Concrete sensor
//! implementations (CrowdStrike, Cyberint, Claroty, Armis) live in S-2.07.
//!
//! # Modules
//! - [`adapter`]  — `SensorAdapter` trait, `SensorSpec`, `QueryParams`, `SensorError`
//! - [`auth`]     — `SensorAuth` sealed trait + per-sensor auth subtypes
//! - [`registry`] — `AdapterRegistry` mapping `SensorType` → `Arc<dyn SensorAdapter>`
//! - [`fanout`]   — Cross-client fan-out orchestrator (`fan_out()`)
//! - [`retry`]    — `retry_with_backoff()` with full-jitter exponential backoff
//! - [`http`]     — Global HTTP connection semaphore (200 permits)
//!
//! # Architecture Compliance (S-2.06)
//! - `SensorAdapter` is object-safe — no generic methods (BC-2.01.013)
//! - `SensorAuth` is sealed — external crates cannot add auth types (BC-2.01.013, DI-012)
//! - Fan-out concurrency: 10 per query (BC-2.01.002) AND global HTTP cap: 200 (AC-5)
//! - Non-transient 4xx errors (400, 401, 403, 404) are NEVER retried (BC-2.01.014)
//!
//! Subsystem: SS-01 (Sensor Adapters) | Layer: 1 (Infrastructure)

// ── Module declarations ────────────────────────────────────────────────────
pub mod adapter;
pub mod auth;
pub mod fanout;
pub mod http;
pub mod registry;
pub mod retry;

// ── Test modules (cfg-gated) ───────────────────────────────────────────────
#[cfg(test)]
pub mod tests;

// ── Re-exports ─────────────────────────────────────────────────────────────
pub use adapter::{is_transient_status, QueryParams, SensorAdapter, SensorError, SensorSpec};
pub use auth::{ArmisAuth, ClarotyAuth, CrowdStrikeAuth, CyberintAuth, SensorAuth};
pub use fanout::{
    error_to_retry_metadata, fan_out, CredentialResolver, FanOutError, FanOutResult, FanOutTarget,
    RetryMetadata, MAX_FANOUT_CONCURRENCY,
};
pub use http::{
    acquire_http_permit, available_http_permits, init_http_semaphore, HTTP_SEMAPHORE_PERMITS,
    HTTP_SEMAPHORE_TIMEOUT,
};
pub use registry::AdapterRegistry;
pub use retry::{retry_with_backoff, RetryConfig, DEFAULT_TRANSIENT_CODES};
