//! Route handler modules for the Armis Centrix DTU clone.
//!
//! Module layout:
//! - `devices` — device inventory, activity log, and risk score endpoints
//! - `alerts` — alert / policy violation list endpoint
//! - `search` — AQL unified search endpoint (`GET /api/v1/search`, ADR-031 §D8-a)
//! - `tags` — stateful device tag write endpoints
//! - `dtu` — DTU-internal test API (`/dtu/*`) per ADR-002 §6

pub mod alerts;
pub mod devices;
pub mod dtu;
pub mod search;
pub mod tags;
