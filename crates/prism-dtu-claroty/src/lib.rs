//! `prism-dtu-claroty` — L4 (adversarial) behavioral clone of the Claroty xDome API.
//!
//! Implements 8 application endpoints (6 read via POST-body filtering, 2 write):
//! the original 7 from `dtu-assessment.md §3.2` (5 read + 2 write) plus
//! `audit_log::list_audit_logs` added by S-DEMO-CLAROTY-AUDIT-DTU-001 (Wave-5
//! fidelity addition).  Also implements `group_by` parameter behavior, stateful
//! device tag store, and full failure injection via `FailureLayer`.
//!
//! Gated behind `#[cfg(any(test, feature = "dtu"))]` — never compiled into
//! production binaries.
//!
//! The `generator` module is additionally gated behind `#[cfg(feature = "fixture-gen")]`
//! (S-3.7.02 / AC-007 / D-056).
#![cfg(any(test, feature = "dtu", feature = "fixture-gen"))]

#[cfg(any(test, feature = "dtu"))]
pub mod clone;
#[cfg(feature = "fixture-gen")]
pub mod generator;
#[cfg(any(test, feature = "dtu"))]
pub mod routes;
#[cfg(any(test, feature = "dtu"))]
pub mod state;
#[cfg(any(test, feature = "dtu"))]
pub mod types;

#[cfg(any(test, feature = "dtu"))]
pub use clone::ClarotyClone;
#[cfg(feature = "fixture-gen")]
pub use generator::generate;
#[cfg(any(test, feature = "dtu"))]
pub use state::ClarotyState;
