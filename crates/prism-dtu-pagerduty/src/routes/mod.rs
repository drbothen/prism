//! Route handler modules for the PagerDuty Events API v2 DTU clone.
//!
//! Module layout:
//! - `enqueue` — `POST /v2/enqueue` handler (trigger / acknowledge / resolve lifecycle)
//! - `dtu` — DTU-internal test API (`/dtu/*`) per ADR-002 §6

pub mod dtu;
pub mod enqueue;
