//! Harness-internal behavioral clone implementations for sensor DTU types.
//!
//! Each submodule provides a self-contained axum router for a specific DTU type.
//! The builder dispatches on `DtuType` to select the correct router.
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
//! # BC anchors
//!
//! - BC-3.2.004 — shared-mode org-id tagging
//! - BC-3.5.001 — harness logical isolation
//! - BC-3.5.002 — harness network isolation

pub mod crowdstrike;
pub mod jira;
pub mod pagerduty;
pub mod slack;
