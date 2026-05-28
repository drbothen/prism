//! Tool handler modules for PrismServer (BC-2.10.002).
//!
//! All tools registered via `#[tool_router]` macro live under this module.
//! Every tool handler must call `injection_scanner.scan_all()` at entry before
//! any domain logic (BC-2.09.001 invariant, ADR-022 §F NON-NEGOTIABLE).
//!
//! Tool categories per BC-2.13.* catalog:
//! - `query`          — PrismQL query execution + explain + alias CRUD
//! - `write`          — confirm_action (irreversible write confirmation via WriteExecutor)
//! - `sensor_health`  — sensor connectivity + diagnostics
//! - `config`         — config reload + sensor spec management + capability listing
//! - `operations`     — schedule / detection / case management (NotImplemented until prism-operations merges)

pub mod config;
pub mod operations;
pub mod query;
pub mod sensor_health;
pub mod write;
