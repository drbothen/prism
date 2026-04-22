//! Plugin sandbox enforcement — memory limits, epoch interruption, trap isolation.
//!
//! # Architecture Compliance
//! - Memory limit: `wasmtime::StoreLimits` with `memory_size: 64 * 1024 * 1024` (default).
//! - CPU limit: `Store::epoch_deadline` set per call; background tokio task ticks engine
//!   epoch every 1ms (started once in `PluginRuntime::new`, NOT per call).
//! - Trap isolation: all `instance.call_*` invocations wrapped to catch `wasmtime::Trap`
//!   and return `Err(PluginError::Trapped)` or `Err(PluginError::Timeout)` or
//!   `Err(PluginError::MemoryExceeded)` — the host process MUST NOT unwind.
//!
//! # Red Gate stubs (S-1.15)
//! All functions are `unimplemented!()`.

use prism_core::PluginError;
use super::loader::HostState;

/// Default memory limit per plugin instance: 64 MiB.
pub const DEFAULT_MEMORY_LIMIT_MB: u64 = 64;

/// Default CPU time limit per plugin call: 5 seconds.
pub const DEFAULT_TIMEOUT_SECONDS: u64 = 5;

/// Epoch ticks per second (epoch ticker fires every 1ms → 1000 ticks/s).
pub const EPOCH_TICKS_PER_SECOND: u64 = 1000;

/// Create a fresh `wasmtime::Store<HostState>` configured with:
/// - `StoreLimits` enforcing `memory_limit_mb * 1024 * 1024` linear memory cap.
/// - `epoch_deadline` set to `timeout_seconds * EPOCH_TICKS_PER_SECOND`.
///
/// Each plugin call creates one `Store` via this function. Stores are NOT reused
/// across calls (BC-2.17.001: per-call Store creation prevents state leakage).
///
/// # Errors
/// Returns `Err(PluginError::CompilationFailed)` (misuse of type, just a placeholder)
/// if store creation itself fails (should be infallible in practice).
pub fn create_store(
    engine: &wasmtime::Engine,
    host_state: HostState,
    memory_limit_mb: u64,
    timeout_seconds: u64,
) -> wasmtime::Store<HostState> {
    unimplemented!("S-1.15 Red Gate: create_store not yet implemented")
}

/// A lower-level helper used by VP-041 (proptest) to create a store with a
/// specific memory limit for boundary testing.
///
/// Takes `limit_mb` in `1..=512` (per VP-041 range).
pub fn create_store_with_limit(
    engine: &wasmtime::Engine,
    limit_mb: u64,
) -> wasmtime::Store<HostState> {
    unimplemented!("S-1.15 Red Gate: create_store_with_limit not yet implemented")
}

/// Attempt to allocate `bytes` of WASM linear memory in a fresh store configured
/// with `create_store_with_limit(limit_mb)`.
///
/// Used by VP-041 proptest to exercise the exact boundary:
/// - `bytes == limit_mb * 1024 * 1024` → must succeed (`Ok(())`)
/// - `bytes == limit_mb * 1024 * 1024 + 1` → must trap (`Err(PluginError::MemoryExceeded)`)
pub fn try_allocate_wasm_memory(
    engine: &wasmtime::Engine,
    limit_mb: u64,
    bytes: usize,
) -> Result<(), PluginError> {
    unimplemented!("S-1.15 Red Gate: try_allocate_wasm_memory not yet implemented")
}

/// Start the epoch ticker background task.
///
/// Spawns a long-lived `tokio::task` that calls `engine.increment_epoch()` every 1ms.
/// MUST be called exactly once during `PluginRuntime::new()`. Never spawned per call.
///
/// If the ticker task terminates unexpectedly, logs `ERROR "Plugin epoch ticker terminated"`.
pub fn start_epoch_ticker(engine: wasmtime::Engine) -> tokio::task::JoinHandle<()> {
    unimplemented!("S-1.15 Red Gate: start_epoch_ticker not yet implemented")
}

/// Wrap a `wasmtime` call result (`anyhow::Error` or `wasmtime::Trap`) into a
/// structured `PluginError`.
///
/// Classification:
/// 1. If the error is a `wasmtime::Trap` with an interrupt cause → `PluginError::Timeout`
/// 2. If the error is a memory limit trap (ResourceLimiter rejection) → `PluginError::MemoryExceeded`
/// 3. Any other trap → `PluginError::Trapped`
///
/// The host process MUST NOT unwind past this boundary (BC-2.17.001).
pub fn classify_wasm_error(
    plugin_id: &str,
    err: anyhow::Error,
    memory_limit_mb: u64,
    elapsed_ms: u64,
    timeout_ms: u64,
) -> PluginError {
    unimplemented!("S-1.15 Red Gate: classify_wasm_error not yet implemented")
}
