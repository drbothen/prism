//! `memory` — DataFusion `GreedyMemoryPool` configuration and error mapping.
//!
//! Each query execution creates a fresh `SessionContext` with a dedicated
//! `GreedyMemoryPool`. The pool is scoped to a single query — it MUST NOT
//! be shared across queries (that would serialize all queries through a single
//! 200MB budget). (BC-2.11.006 architecture compliance rule)
//!
//! # Memory Limit: 200 MB per query
//! `200 * 1024 * 1024 = 209_715_200` bytes (BC-2.11.006)
//!
//! # 3-Tier Fallback
//! 1. Query runs within 200MB budget — normal path.
//! 2. Pool trips → DataFusion returns `ResourcesExhausted` → mapped to
//!    `PrismError::QueryMemoryBudgetExceeded` (E-QUERY-004). (BC-2.11.006)
//! 3. OOM before pool trips (should not happen) — caught at task boundary.
//!
//! # BC References
//! - BC-2.11.006 — Query Security Limits: 200MB per-query pool
//!
//! Story: S-3.02

// S-3.02 stub functions: dead_code suppressed for stub phase (BC-5.38.001).
#![allow(dead_code)]

use datafusion::execution::context::SessionContext;
use prism_core::PrismError;

/// Default per-query memory pool size: 200 MB. (BC-2.11.006)
pub const QUERY_MEMORY_POOL_BYTES: usize = 200 * 1024 * 1024;

/// Default materialization record cap: 10,000 records. (BC-2.11.006)
pub const MAX_MATERIALIZED_RECORDS: usize = 10_000;

/// Default query execution timeout: 30 seconds. (BC-2.11.006)
pub const QUERY_TIMEOUT_SECS: u64 = 30;

// ---------------------------------------------------------------------------
// build_session_context
// ---------------------------------------------------------------------------

/// Build a DataFusion `SessionContext` with a per-query `GreedyMemoryPool`.
///
/// The `RuntimeEnv` is configured with a `GreedyMemoryPool` of `pool_bytes`
/// capacity. Each call produces a fresh, independent `SessionContext` — the
/// pool is never shared across queries. (BC-2.11.006)
///
/// # Pool Semantics
/// `GreedyMemoryPool` grants allocations until the limit is reached. When the
/// limit is exceeded, DataFusion returns a `ResourcesExhausted` error. The
/// caller maps this via `map_datafusion_memory_error`. (BC-2.11.006)
pub fn build_session_context(_pool_bytes: usize) -> Result<SessionContext, PrismError> {
    todo!("S-3.02 — build_session_context")
}

// ---------------------------------------------------------------------------
// map_datafusion_memory_error
// ---------------------------------------------------------------------------

/// Map a DataFusion `ResourcesExhausted` error to `PrismError::QueryMemoryBudgetExceeded`.
///
/// If `err` is not a `ResourcesExhausted` error, wraps it in
/// `PrismError::QueryExecutionFailed`. (BC-2.11.006)
///
/// # BC-2.11.006
/// Pool trips return E-QUERY-004 (memory budget exceeded).
pub fn map_datafusion_memory_error(_err: datafusion::error::DataFusionError) -> PrismError {
    todo!("S-3.02 — map_datafusion_memory_error")
}
