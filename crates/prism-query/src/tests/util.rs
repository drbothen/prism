//! Test utilities for `prism-query`.
//!
//! Story: S-3.01 — SIGBUS triage Path B
//!
//! # Deep-stack helper
//!
//! The default test thread stack on macOS aarch64 is 2 MB. Tests that exercise
//! deep chumsky parser recursion (nesting depth ≥ ~50) can exhaust that ceiling
//! when `[profile.dev] debug = "line-tables-only"` is active, because that
//! codegen option produces marginally larger stack frames.
//!
//! Earlier "all green" runs were lucky — the parser was already at the edge
//! of the 2 MB ceiling. Adding `debug = "line-tables-only"` in commit 931f3c6f
//! pushed depth-65 tests over the edge, causing SIGBUS on macOS aarch64.
//!
//! Use [`run_with_deep_stack`] for **every** test that builds a query with
//! nesting depth ≥ 50.

/// Run a test closure on a thread with an 8 MB stack.
///
/// Default test thread stack is 2 MB on macOS aarch64. Tests that exercise
/// deep parser recursion (depth ≥ ~50) can SIGBUS at that ceiling — see
/// triage of `test_BC_2_11_006_sql_and_chain_depth_65_rejected` SIGBUS
/// when `[profile.dev] debug = "line-tables-only"` is active.
///
/// Use this helper for ALL tests that build queries with depth ≥ 50.
///
/// # Example
///
/// ```ignore
/// #[test]
/// fn test_deep_recursion() {
///     run_with_deep_stack(|| {
///         // test body unchanged
///     });
/// }
/// ```
pub(crate) fn run_with_deep_stack<F>(f: F)
where
    F: FnOnce() + Send + 'static,
{
    std::thread::Builder::new()
        .stack_size(8 * 1024 * 1024)
        .spawn(f)
        .expect("spawn deep-stack test thread")
        .join()
        .expect("deep-stack test thread panicked")
}
