//! `prism-dtu-cyberint` — L2-fidelity behavioral clone of the Cyberint API.
//!
//! Implements [`CyberintClone`] which satisfies the [`prism_dtu_common::BehavioralClone`] trait.
//!
//! ## Auth model (ADR-031 §D1-b + §D3-a)
//!
//! Cyberint uses static `access_token` cookie injection — no login roundtrip route.
//! Callers inject the API key as `Cookie: access_token=<value>` on every request.
//! [`prism_spec_engine::StaticCookieAuthProvider`] provides the production auth implementation
//! per BC-2.01.017 §Postconditions P2. The DTU validates the `access_token` cookie on
//! protected routes and returns HTTP 401 if the cookie is absent or unrecognised.
//!
//! ## Routes
//!
//! - `GET /api/v1/alerts` — alert list with cursor pagination
//! - `GET /api/v1/alerts/{alert_id}` — alert detail
//! - `PATCH /api/v1/alerts/{alert_id}/status` — acknowledge alert (stateful)
//! - `POST /api/v1/alerts/{alert_id}/close` — close alert (irreversible in session)
//! - `GET /api/v1/threat-intel` — threat intelligence feed
//! - `POST /dtu/configure` — runtime reconfiguration
//! - `POST /dtu/reset` — reset all mutable state
//! - `GET /dtu/health` — liveness check
//!
//! This crate is gated behind `#[cfg(any(test, feature = "dtu", feature = "fixture-gen"))]`
//! and must NEVER link into a production binary.
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
pub use clone::CyberintClone;
#[cfg(feature = "fixture-gen")]
pub use generator::generate;
#[cfg(feature = "fixture-gen")]
pub use generator::generate_with_catalog;
#[cfg(any(test, feature = "dtu"))]
pub use state::CyberintState;
#[cfg(any(test, feature = "dtu"))]
pub use types::{AlertStatus, CyberintError};
