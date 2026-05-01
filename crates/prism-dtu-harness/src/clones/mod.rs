//! Per-DTU clone routers for `prism-dtu-harness`.
//!
//! Each module provides a self-contained axum `Router` factory for a specific
//! DTU type, allowing the harness builder to dispatch on `DtuType` and serve
//! sensor-specific endpoints alongside the shared `CloneState` (failure injection,
//! admin configure, health checks).
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
//! # BC anchors
//!
//! - BC-3.2.004 — shared-mode org-id tagging
//! - BC-3.5.001 — harness logical isolation
//! - S-3.4.04 — Cyberint harness migration (cookie auth + alert lifecycle routes)

pub mod cyberint;
pub mod jira;
pub mod pagerduty;
pub mod slack;
