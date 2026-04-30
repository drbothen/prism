//! Per-surface DTU clone routers for the harness.
//!
//! Each sub-module exposes a `router(state)` function that returns an
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
//!    the new module's `router()` function.
//!
//! # Architecture Anchors
//!
//! - ADR-011 §2.2 — logical-mode per-org state segregation
//! - S-3.4.01     — Claroty harness migration

pub mod claroty;
