//! `prism-dtu-armis` — L2-fidelity behavioral clone of the Armis Centrix API.
//!
//! Implements [`ArmisClone`] which satisfies the [`prism_dtu_common::BehavioralClone`] trait.
//! The clone provides:
//! - `GET /api/v1/devices` / `POST /api/v1/devices` — AQL-forwarded device inventory
//! - `GET /api/v1/devices/{device_id}/activity` — device activity log
//! - `GET /api/v1/devices/{device_id}/risk` — device risk score
//! - `GET /api/v1/alerts` — alert / policy violation list
//! - `POST /api/v1/devices/{device_id}/tags/` — add device tag (stateful)
//! - `DELETE /api/v1/devices/{device_id}/tags/{tag_key}` — remove device tag (stateful)
//! - `GET /dtu/aql-log` — test API: returns all AQL strings received since last reset
//! - `POST /dtu/configure` — runtime reconfiguration
//! - `POST /dtu/reset` — reset all mutable state
//! - `GET /dtu/health` — liveness check for test setup
//!
//! BearerStatic auth: HTTP 403 (not 401) on missing/invalid Authorization header,
//! per Armis Centrix API spec (intentionally differs from Bearer-standard behavior).
//!
//! Gated behind `#[cfg(any(test, feature = "dtu"))]` — never compiled into production.
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
pub use clone::ArmisClone;
#[cfg(feature = "fixture-gen")]
pub use generator::generate;
#[cfg(any(test, feature = "dtu"))]
pub use state::ArmisState;
