//! Route handler modules for the Jira Cloud REST API v3 DTU clone.
//!
//! Module layout:
//! - `issues` — create issue (POST) and get issue (GET) endpoints
//! - `comments` — add comment endpoint
//! - `transitions` — list and execute transitions endpoints
//! - `dtu` — DTU-internal test API (`/dtu/*`) per ADR-002 §6

pub mod comments;
pub mod dtu;
pub mod issues;
pub mod transitions;
