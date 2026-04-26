//! `prism-dtu-slack` — L2-fidelity behavioral clone of the Slack Incoming Webhook API.
//!
//! Implements [`SlackClone`] which satisfies the [`prism_dtu_common::BehavioralClone`] trait.
//! The clone provides:
//! - `POST /services/{token}` — Slack Incoming Webhook endpoint (token ignored, any value accepted)
//! - `GET /dtu/received-payloads` — test API: returns all Block Kit payloads received since last reset
//! - `POST /dtu/configure` — runtime reconfiguration (failure injection, rate-limit threshold)
//! - `POST /dtu/reset` — reset all mutable state
//! - `GET /dtu/health` — liveness check for test setup
//!
//! Payload validation:
//! - Payload MUST contain `blocks` or `text` top-level key; otherwise HTTP 400 `"invalid_payload"`.
//! - Unknown top-level fields (outside `fixtures/block-kit-schema.json` allow-list) → HTTP 400 `"unknown_field"`.
//! - Valid payload: HTTP 200 `{"ok": true, "message_ts": "1234567890.123456"}`.
//!
//! Rate-limit behavior: when `FailureMode::RateLimit { after_n_requests, retry_after_secs }` is
//! configured, requests beyond the threshold return HTTP 429 with `Retry-After` header.
//!
//! Gated behind `#[cfg(any(test, feature = "dtu"))]` — never compiled into production.
#![cfg(any(test, feature = "dtu"))]

pub mod clone;
pub mod routes;
pub mod state;
pub mod types;

pub use clone::SlackClone;
pub use state::SlackState;
