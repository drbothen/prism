//! `prism-dtu-pagerduty` — L3-fidelity behavioral clone of the PagerDuty Events API v2.
//!
//! Implements [`PagerDutyClone`] which satisfies the [`prism_dtu_common::BehavioralClone`] trait.
//! The clone provides:
//! - `POST /v2/enqueue` — trigger / acknowledge / resolve event lifecycle per PagerDuty Events API v2
//! - `GET /dtu/incidents` — test API: returns the current incident registry
//! - `POST /dtu/configure` — runtime reconfiguration (failure injection, auth_mode, etc.)
//! - `POST /dtu/reset` — reset all mutable state
//! - `GET /dtu/health` — liveness check for test setup
//!
//! Routing key validation: when `auth_mode == "reject"`, all requests return
//! HTTP 403 `{"status": "invalid key", "message": "Forbidden"}`.
//!
//! Severity is case-sensitive per PagerDuty spec — lowercase only
//! (`critical`, `error`, `warning`, `info`). `"CRITICAL"` returns HTTP 400.
//!
//! Dedup key semantics match PagerDuty: re-triggering an active incident is a no-op
//! (idempotent 202); triggering a resolved incident starts a fresh incident.
//!
//! Gated behind `#[cfg(any(test, feature = "dtu"))]` — never compiled into production.
#![cfg(any(test, feature = "dtu"))]

pub mod clone;
pub mod routes;
pub mod state;
pub mod types;

pub use clone::PagerDutyClone;
pub use state::PagerDutyState;
