//! Route handler modules for the Slack Incoming Webhook DTU clone.
//!
//! Module layout:
//! - `webhook` — Slack Incoming Webhook endpoint (`POST /services/{token}`)
//! - `dtu` — DTU-internal test API (`/dtu/*`) per ADR-002 §6

pub mod dtu;
pub mod webhook;
