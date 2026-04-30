//! AC-008 Timeout-Knob Compile-Gate Test — S-3.3.04 Red Gate
//!
//! This file is a COMPILE-ERROR Red Gate. It will NOT compile until the implementer
//! adds `with_network_bind_timeout(Duration)` to `HarnessBuilder`.
//!
//! # Purpose
//!
//! ADR-011 §2.5 specifies a 5s build timeout for `IsolationMode::Network`. The builder
//! must expose a knob to configure this timeout for testing short-timeout scenarios.
//!
//! # Implementer instruction
//!
//! Add to `HarnessBuilder` (in `crates/prism-dtu-harness/src/builder.rs`):
//!
//! ```rust
//! /// Override the Network-mode build timeout (default: 5 seconds per BC-3.5.002 postcondition 5).
//! ///
//! /// Used in tests to exercise `StartupTimeout` with short timeouts.
//! ///
//! /// (BC-3.5.002 postcondition 5; ADR-011 §2.5; AC-008)
//! pub fn with_network_bind_timeout(mut self, timeout: std::time::Duration) -> Self {
//!     self.network_bind_timeout = Some(timeout);
//!     self
//! }
//! ```
//!
//! Also add the field to `HarnessBuilder`:
//! ```rust
//! /// Configurable Network-mode build timeout. None = use default 5s.
//! pub(crate) network_bind_timeout: Option<std::time::Duration>,
//! ```
//!
//! And wire it in `build_network()`:
//! ```rust
//! let timeout = builder.network_bind_timeout.unwrap_or(Duration::from_secs(5));
//! let start_results = tokio::time::timeout(timeout, start_all(startup_futures)).await;
//! ```
//!
//! # Red Gate
//!
//! This file will not compile until `with_network_bind_timeout` exists.
//! The compile error is intentional — it documents the missing builder API.
//! When the implementer adds the method, this test will transition to
//! FAILING at runtime (via `build_network()` `todo!()`) until the full
//! implementation is complete.
//!
//! # Traceability
//!
//! - BC-3.5.002 postcondition 5 — 5s total build timeout
//! - ADR-011 §2.5 — timeout specification
//! - AC-008 — timeout knob required

use prism_dtu_harness::{DtuType, HarnessError, IsolationMode};
use std::time::Duration;

/// AC-008 (timeout-knob compile gate): `with_network_bind_timeout(Duration)` method.
///
/// Calls `with_network_bind_timeout(Duration::from_millis(100))` which DOES NOT YET
/// EXIST on `HarnessBuilder`. This causes a compile error until the implementer adds it.
///
/// Once the method exists, this test will fail at runtime (via `build_network()` `todo!()`)
/// until the full implementation is complete.
///
/// RED: compile error — `with_network_bind_timeout` not found in `HarnessBuilder`.
///
/// (BC-3.5.002 postcondition 5; ADR-011 §2.5; AC-008)
#[tokio::test]
async fn test_BC_3_5_002_ac008_timeout_knob_compile_gate() {
    let result = prism_dtu_harness::Harness::builder()
        .isolation(IsolationMode::Network)
        .with_network_bind_timeout(Duration::from_millis(100))
        // ^^^ compile error until implementer adds this method to HarnessBuilder
        .with_customer_overrides("acme-corp", |spec| {
            spec.dtu_types = vec![DtuType::Claroty];
            // 500ms startup delay — exceeds the 100ms configured timeout
            spec.startup_delay_ms = Some(500);
        })
        .build()
        .await;

    assert!(
        matches!(result, Err(HarnessError::StartupTimeout)),
        "Network-mode build with 100ms timeout and 500ms startup delay must return \
         StartupTimeout (BC-3.5.002 postcondition 5; ADR-011 §2.5; AC-008)"
    );
}
