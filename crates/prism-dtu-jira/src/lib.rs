//! `prism-dtu-jira` — L3-fidelity behavioral clone of the Jira Cloud REST API v3.
//!
//! Implements [`JiraClone`] which satisfies the [`prism_dtu_common::BehavioralClone`] trait.
//! The clone provides:
//! - `POST /rest/api/3/issue` — create issue (field validation, status machine init)
//! - `GET /rest/api/3/issue/{key}` — get issue by key
//! - `POST /rest/api/3/issue/{key}/comment` — add comment (increments comment_count)
//! - `GET /rest/api/3/issue/{key}/transitions` — list available transitions for current status
//! - `POST /rest/api/3/issue/{key}/transitions` — execute transition (status machine enforced)
//! - `GET /dtu/issues` — test API: returns all current issues for assertions
//! - `POST /dtu/configure` — runtime reconfiguration (failure injection)
//! - `POST /dtu/reset` — reset all mutable state (issue registry + counter)
//! - `GET /dtu/health` — liveness check for test setup
//!
//! Basic auth: HTTP 401 on missing/invalid `Authorization: Basic {base64}` header.
//! Status machine (L3 behavioral fidelity): invalid transitions return 400.
//!
//! Gated behind `#[cfg(any(test, feature = "dtu"))]` — never compiled into production.
#![cfg(any(test, feature = "dtu"))]

pub mod clone;
pub mod routes;
pub mod state;
pub mod types;

pub use clone::JiraClone;
pub use state::JiraState;
