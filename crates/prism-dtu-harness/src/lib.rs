//! `prism-dtu-harness` — Multi-tenant DTU test harness.
//!
//! Provides `IsolationMode::Logical` in-process org-keyed clone orchestration,
//! per-`(OrgId, DtuType)` failure injection, and crash detection via `JoinHandle`
//! monitoring.
//!
//! # Gate
//!
//! All public items are gated behind `#[cfg(any(test, feature = "dtu"))]`.
//! This crate MUST NEVER link into a production binary.
//!
//! # Architecture Anchors
//!
//! - ADR-011 §2.2  — Logical mode: in-process org-keyed routing
//! - ADR-011 §2.5  — Port allocation and cleanup (simultaneous bind; D-058)
//! - ADR-011 §2.6  — Crash detection: `JoinHandle` monitoring
//! - ADR-011 §2.7  — Failure injection: `inject_failure`, `clear_failure`, `FailureMode`
//! - D-058          — 200ms parallel startup budget locked decision
//!
//! # BCs
//!
//! - BC-3.5.001 — Harness Logical Isolation Invariants
//! - BC-3.6.001 — Per-Org Failure Injection
//! - BC-3.6.002 — Harness Crash Detection
#![cfg(any(test, feature = "dtu"))]

pub mod builder;
pub mod clone_server;
pub mod crash_monitor;
pub mod error;
pub mod harness;
pub mod types;

pub use builder::HarnessBuilder;
pub use error::HarnessError;
pub use harness::Harness;
pub use types::{CustomerSpec, DtuType, IsolationMode, OrgKey};
