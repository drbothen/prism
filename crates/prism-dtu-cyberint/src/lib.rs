//! `prism-dtu-cyberint` — L2-fidelity behavioral clone of the Cyberint API.
//!
//! Implements [`CyberintClone`] which satisfies the [`prism_dtu_common::BehavioralClone`] trait.
//! The clone provides:
//! - `POST /login` — cookie-based auth (CookieRoundtrip pattern)
//! - `GET/POST /api/v1/alerts` — alert list with cursor pagination
//! - `GET /api/v1/alerts/{alert_id}` — alert detail
//! - `PATCH /api/v1/alerts/{alert_id}/status` — acknowledge alert (stateful)
//! - `POST /api/v1/alerts/{alert_id}/close` — close alert (irreversible in session)
//! - `GET /api/v1/threat-intel` — threat intelligence feed
//! - `POST /dtu/configure` — runtime reconfiguration
//! - `POST /dtu/reset` — reset all mutable state
//! - `GET /dtu/health` — liveness check
//!
//! Gated behind `#[cfg(any(test, feature = "dtu"))]` — never compiled into production.
#![cfg(any(test, feature = "dtu"))]

pub mod clone;
pub mod routes;
pub mod state;
pub mod types;

pub use clone::CyberintClone;
pub use state::CyberintState;
pub use types::{AlertStatus, CyberintError};
