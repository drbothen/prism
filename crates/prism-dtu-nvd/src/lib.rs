//! `prism-dtu-nvd` — L2-fidelity behavioral clone of the NVD/NIST CVE API 2.0.
//!
//! Implements [`NvdClone`] which satisfies the [`prism_dtu_common::BehavioralClone`] trait.
//! The clone provides:
//! - `GET /rest/json/cves/2.0` — single CVE lookup and bulk paginated fetch
//! - `GET /dtu/request-count/{cve_id}` — test API for cache-hit assertion
//! - `POST /dtu/configure` — runtime reconfiguration (auth_mode, failure injection)
//! - Dual rate-limit buckets (5/30s unauthenticated, 50/30s authenticated)
//! - 10 fixture CVEs spanning the full CVSS severity spectrum
//!
//! Gated behind `#[cfg(any(test, feature = "dtu"))]` — never compiled into production.
#![cfg(any(test, feature = "dtu"))]
#![allow(unused_variables, dead_code)]

pub mod clone;
pub mod routes;
pub mod state;
pub mod types;

pub use clone::NvdClone;
pub use state::NvdState;
pub use types::{CveRecord, CveResponse, NvdError};
