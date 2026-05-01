//! Harness-internal behavioral clone implementations for MSSP Coordination DTU types.
//!
//! Each submodule provides:
//! - DTU-specific state struct (payload store, incident registry, issue registry)
//! - Combined context struct pairing DTU-specific state with generic `CloneState`
//! - Route handlers implementing the vendor API surface
//! - A `build_X_router` function that returns the axum `Router`
//!
//! The routers are dispatched by `clone_server::start_clone` based on `DtuType`.
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

pub mod jira;
pub mod pagerduty;
pub mod slack;
