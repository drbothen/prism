//! Regression test: init_tracing must not panic when called more than once.
//!
//! Addresses review finding F1: double-init safety in test harnesses.

use prism_core::{init_tracing, TracingConfig};

/// Calling init_tracing twice must not panic.
#[test]
fn test_init_tracing_double_init_does_not_panic() {
    let cfg = TracingConfig::default();
    init_tracing(&cfg);
    // Second call — must silently no-op, not panic.
    init_tracing(&cfg);
}
