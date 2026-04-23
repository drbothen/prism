//! prism-core — foundational types for the Prism platform.
//!
//! This crate is the dependency root: it has zero internal Prism dependencies.
//! Every other crate in the workspace depends on `prism-core`.
//!
//! # Public API surface (S-1.01 + S-1.02 + S-1.10 + S-1.11)
//!
//! - [`tenant::TenantId`] — validated tenant identifier (`Arc<str>` inner)
//! - [`error::PrismError`] — canonical error taxonomy (90+ variants)
//! - [`storage::StorageDomain`] — RocksDB column families
//! - [`storage::ColumnOptions`] — per-column-family configuration
//! - [`column::ColumnOptions`] — spec-engine column options (S-1.11)
//! - [`column::ColumnType`] — spec-engine column type enum (S-1.11)
//! - [`types::ClientId`], [`types::AnalystId`], [`types::SeverityId`]
//! - [`types::Timestamp`], [`types::SensorType`], [`types::ColumnType`]
//! - [`cache::CacheBackend`] — subset of StorageBackend (get/set/execute)
//! - [`config::ConfigSnapshot`] — opaque config snapshot shell
//! - [`telemetry::TracingConfig`], [`telemetry::init_tracing`]
//! - [`capability::CapabilityPath`], [`capability::CapabilityEffect`],
//!   [`capability::CapabilityExplanation`], [`capability::ClientCapabilities`]
//! - [`trust::TrustLevel`] — trust classification for MCP responses (S-1.10)
//! - [`safety::SafetyFlag`], [`safety::PatternCategory`] — injection detection records (S-1.10)
//! - [`alert::AlertSeverity`] — OCSF-aligned alert severity (S-1.02)
//! - [`case::CaseStatus`], [`case::advance_case_state`] — state machine (S-1.02)
//! - [`credentials::CredentialName`] — validated credential name (S-1.02)
//! - [`cursor::CursorRegistry`] — 200-cursor cap enforcement (S-1.02)
//! - [`ids::ScheduleId`], [`ids::CaseId`], etc. — UUID v7 ID newtypes (S-1.02)

// cfg(kani) is set by the Kani verification toolchain, not by Cargo features.
#![allow(unexpected_cfgs)]

// ── S-1.01 modules ────────────────────────────────────────────────────────────
pub mod cache;
pub mod capability;
pub mod column;
pub mod config;
pub mod error;
pub mod risk;
pub mod safety;
pub mod storage;
pub mod telemetry;
pub mod tenant;
pub mod trust;
pub mod types;

// ── S-1.02 additions ─────────────────────────────────────────────────────────
pub mod alert;
pub mod case;
pub mod credentials;
pub mod cursor;
pub mod ids;

// ── Kani proofs (cfg-gated; compile everywhere, run only under cargo kani) ───
pub mod proofs;

// ── Unit tests ───────────────────────────────────────────────────────────────
#[cfg(test)]
pub mod tests;

// ── Re-exports ────────────────────────────────────────────────────────────────
// S-1.01
pub use cache::CacheBackend;
pub use capability::{CapabilityEffect, CapabilityExplanation, CapabilityPath, ClientCapabilities};
pub use column::{ColumnOptions, ColumnType};
pub use config::ConfigSnapshot;
pub use error::{PrismError, SpecError, SpecErrorCode};
pub use risk::RiskTier;
pub use safety::{PatternCategory, SafetyFlag};
pub use storage::StorageDomain;
pub use telemetry::{init_tracing, TracingConfig};
pub use tenant::TenantId;
pub use trust::TrustLevel;
pub use types::{AnalystId, ClientId, SensorType, SeverityId, Timestamp};

// S-1.02
pub use alert::AlertSeverity;
pub use case::{
    advance_case_state, CaseStatus, CaseTransitionError, DispositionCode, TimelineEntryType,
    VALID_TRANSITIONS,
};
pub use credentials::CredentialName;
pub use cursor::{CursorId, CursorRegistry, CURSOR_CAP};
pub use ids::{AlertId, CaseId, RuleId, ScheduleId};
