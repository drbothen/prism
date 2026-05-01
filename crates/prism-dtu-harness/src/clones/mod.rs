//! Per-surface DTU clone routers for the harness.
//!
//! Each sub-module exposes a router factory function that returns an
//! `axum::Router` wired to the clone's state. The state type is defined
//! within each module and does NOT depend on the corresponding production
//! DTU crate (prism-dtu-{surface}). This keeps the harness self-contained
//! and avoids circular dev-dependency chains.
//!
//! # Registration
//!
//! `clone_server::start_clone` dispatches on `DtuType` to select the
//! appropriate router factory. Adding a new sensor surface requires:
//! 1. Add a module here (e.g. `pub mod armis;`).
//! 2. Add a match arm in `clone_server::build_router_for` dispatching to
//!    the new module's router function.
//!
//! # Shared-mode design (BC-3.2.004)
//!
//! Unlike Security Telemetry DTUs (Claroty, Armis, CrowdStrike, Cyberint) which
//! are deployed one-per-org, MSSP Coordination DTUs (Slack, PagerDuty, Jira) run
//! as single shared instances serving all orgs. The harness mirrors this by
//! starting ONE clone per `DtuType`, and all orgs route through it.
//!
//! OrgId disambiguation is achieved by embedding the org's UUID in captured
//! payload/incident/issue records (via the `X-Prism-Org-Id` request header).
//!
//! # Architecture Anchors
//!
//! - ADR-011 §2.2 — logical-mode per-org state segregation
//! - BC-3.2.004   — shared-mode org-id tagging
//! - BC-3.5.001   — harness logical isolation
//! - S-3.4.01     — Claroty harness migration
//! - S-3.4.05     — Slack/PagerDuty/Jira harness migration

pub mod claroty;
pub mod jira;
pub mod pagerduty;
pub mod slack;
