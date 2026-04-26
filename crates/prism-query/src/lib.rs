//! `prism-query` — Prism query orchestration crate.
//!
//! Created in S-2.08 with the pure-data `_source_type` virtual field injection
//! function. S-3.02 extends this crate with the DataFusion `TableProvider`
//! integration that consumes the injection function.
//!
//! # Architecture Compliance (S-2.08)
//! This crate MUST NOT depend on DataFusion, Arrow, `arrow-schema`, or `arrow2`.
//! Those dependencies are added by S-3.02. See `Cargo.toml` for the enforcement
//! comment.
//!
//! # Modules
//! - [`types`]           — `SensorQueryDescriptor` struct (table routing context, S-2.08)
//! - [`materialization`] — `inject_source_type()` pure-data virtual field injection (S-2.08)

pub mod materialization;
pub mod types;
