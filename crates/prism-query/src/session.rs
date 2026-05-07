//! `session` — ephemeral `SessionContext` lifecycle management.
//!
//! `SessionScope` is a RAII wrapper that guarantees the DataFusion
//! `SessionContext` is dropped when the enclosing `execute()` call returns,
//! including on panic. This satisfies BC-2.11.005 AC-7.
//!
//! ## RAII via Drop
//!
//! `SessionScope` wraps an `Option<SessionContext>`. The `Drop` impl
//! calls `drop(self.inner.take())`, releasing the inner DataFusion
//! context. This pattern is panic-safe because Rust's drop semantics
//! guarantee `Drop::drop` runs even on panic unwind, unless the
//! runtime aborts.
//!
//! # Architecture Compliance (BC-2.11.005)
//! - Non-scheduled queries: `SessionContext` MUST NOT outlive `execute()`.
//! - `SessionContext` MUST NOT be stored as a field on `QueryEngine`.
//! - `execute_scheduled` is the sole exception — it returns `Arc<SessionContext>`
//!   for the detection engine (S-4.03) to manage.
//!
//! # BC References
//! - BC-2.11.005 — Ephemeral Materialization (AC-7: context dropped on return)
//!
//! Story: S-3.02

// S-3.02 stub functions: dead_code suppressed pending implementation (stub-phase convention).
#![allow(dead_code)]

use std::sync::Arc;

use datafusion::execution::context::SessionContext;

// ---------------------------------------------------------------------------
// SessionScope
// ---------------------------------------------------------------------------

/// RAII wrapper for a DataFusion `SessionContext`.
///
/// Wraps the `SessionContext` in an `Option` so it is explicitly dropped when
/// `SessionScope` is dropped — even on panic unwind. The `Drop` impl calls
/// `self.inner.take()` to consume the context, making the pattern
/// self-documenting and preventing accidental double-drop.
///
/// For non-scheduled queries, `SessionScope` is created at the top of
/// `execute()` and dropped at the end (including error paths). The query plan
/// is executed while the scope is live; results are collected before drop.
///
/// # BC-2.11.005 (AC-7)
/// "Given the SessionContext is created for a non-scheduled query, When
/// `execute()` returns (including on error or panic), Then the SessionContext
/// is dropped and its memory is released."
pub struct SessionScope {
    /// The wrapped session context, held in an Option so we can move out on
    /// `into_arc`. The drop impl fires unconditionally on the inner context.
    inner: Option<SessionContext>,
}

impl SessionScope {
    /// Create a new `SessionScope` wrapping the given `SessionContext`.
    ///
    /// The context is dropped when `SessionScope` is dropped.
    pub fn new(ctx: SessionContext) -> Self {
        Self { inner: Some(ctx) }
    }

    /// Obtain a reference to the inner `SessionContext` for query execution.
    ///
    /// # Panics
    /// Panics if `into_arc` has already been called (inner is None).
    pub fn context(&self) -> &SessionContext {
        match self.inner.as_ref() {
            Some(ctx) => ctx,
            None => panic!(
                "E-INT-001: SessionScope::context called after into_arc — context already moved out"
            ),
        }
    }

    /// Consume the scope and return the inner `SessionContext` as an `Arc`
    /// for use by `execute_scheduled` callers.
    ///
    /// MUST only be called from `execute_scheduled`. Regular `execute()` paths
    /// MUST NOT call this method — they let `SessionScope` drop normally.
    ///
    /// # BC-2.11.005
    /// `execute_scheduled` returns `Arc<SessionContext>` so the detection
    /// engine can run additional queries against already-materialized data.
    pub fn into_arc(mut self) -> Arc<SessionContext> {
        let ctx = match self.inner.take() {
            Some(c) => c,
            None => {
                panic!("E-INT-001: SessionScope::into_arc called after context already moved out")
            }
        };
        Arc::new(ctx)
    }
}

impl Drop for SessionScope {
    fn drop(&mut self) {
        // Explicitly drop the inner context by taking it out of the Option.
        // This is a no-op if `into_arc` already took the context (inner is
        // None). The explicit `take()` is documentation of intent — the
        // compiler would drop `inner` anyway, but making the release visible
        // here clarifies the RAII contract in code review and makes it clear
        // that double-drop is impossible.
        drop(self.inner.take());
    }
}
