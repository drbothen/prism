//! `prism-dtu-crowdstrike` — L4-adversarial behavioral clone of the CrowdStrike Falcon API.
//!
//! Provides a full L4 (adversarial) behavioral clone of the CrowdStrike Falcon API with
//! stateful write support (device containment), session-scoped ID registry for two-step
//! pagination, and configurable failure injection via [`prism_dtu_common::FailureLayer`].
//!
//! This crate wires VP-033 (audit buffer write-before-delivery) and VP-036 (SessionContext
//! drop) integration tests. It must NEVER link into a production binary.
#![cfg(any(test, feature = "dtu"))]
// Production code must not use .unwrap() — use `?` or `.expect("reason")` instead.
// Test binaries suppress this via per-file `#![allow(clippy::unwrap_used, clippy::expect_used)]`.
#![deny(clippy::unwrap_used)]

pub mod clone;
pub mod routes;
pub mod state;

pub use clone::CrowdstrikeClone;
pub use state::{ContainmentStatus, CrowdstrikeState, SessionData};
