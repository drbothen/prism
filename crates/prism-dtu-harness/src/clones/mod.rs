//! Per-DTU clone routers for `prism-dtu-harness`.
//!
//! Each module provides a self-contained axum `Router` factory for a specific
//! DTU type, allowing the harness builder to dispatch on `DtuType` and serve
//! sensor-specific endpoints alongside the shared `CloneState` (failure injection,
//! admin configure, health checks).
//!
//! # Registration
//!
//! `clone_server::start_clone` dispatches on `DtuType` to select the
//! appropriate router factory. Adding a new sensor surface requires:
//! 1. Add a module here (e.g. `pub mod armis;`).
//! 2. Add a match arm in `clone_server::build_router_for` dispatching to
//!    the new module's router function.
//!
//! # Architecture
//!
//! Security Telemetry DTUs (Claroty, Armis, CrowdStrike, Cyberint) are deployed
//! one-per-org; each org gets its own clone instance with org-scoped IDs (D-059).
//!
//! MSSP Coordination DTUs (Slack, PagerDuty, Jira) run as single shared instances
//! serving all orgs. The harness mirrors this by starting ONE clone per `DtuType`,
//! and all orgs route through it. OrgId disambiguation is achieved by embedding the
//! org's UUID in captured payload/incident/issue records (via the `X-Prism-Org-Id`
//! request header).
//!
//! # Architecture Anchors
//!
//! - ADR-011 §2.2 — logical-mode per-org state segregation
//! - BC-3.2.004   — shared-mode org-id tagging
//! - BC-3.5.001   — harness logical isolation
//! - BC-3.5.002   — harness network isolation
//! - S-3.4.01     — Claroty harness migration
//! - S-3.4.02     — Armis harness migration
//! - S-3.4.03     — CrowdStrike harness migration
//! - S-3.4.04     — Cyberint harness migration (cookie auth + alert lifecycle routes)
//! - S-3.4.05     — Slack/PagerDuty/Jira harness migration

pub mod armis;
pub mod claroty;
pub mod crowdstrike;
pub mod cyberint;
pub mod jira;
pub mod pagerduty;
pub mod slack;
