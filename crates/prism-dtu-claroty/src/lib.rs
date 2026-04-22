//! `prism-dtu-claroty` — L4 (adversarial) behavioral clone of the Claroty xDome API.
//!
//! Implements all 7 in-scope endpoints (5 read via POST-body filtering, 2 write),
//! `group_by` parameter behavior, stateful device tag store, and full failure
//! injection via `FailureLayer`. See `dtu-assessment.md §3.2` for scope matrix.
//!
//! Gated behind `#[cfg(any(test, feature = "dtu"))]` — never compiled into
//! production binaries.
#![cfg(any(test, feature = "dtu"))]

pub mod clone;
pub mod routes;
pub mod state;
pub mod types;

pub use clone::ClarotyClone;
pub use state::ClarotyState;
