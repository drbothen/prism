//! prism-core — foundational types for the Prism platform.
//!
//! This crate is the dependency root: it has zero internal Prism dependencies.
//! Every other crate in the workspace depends on `prism-core`.
//!
//! # Public API surface
//!
//! - [`tenant::TenantId`] — validated tenant identifier (`Arc<str>` inner)
//! - [`error::PrismError`] — canonical error taxonomy (90+ variants)
//! - [`storage::StorageDomain`] — 16 RocksDB column families
//! - [`storage::ColumnOptions`] — per-column-family configuration
//! - [`types::ClientId`], [`types::AnalystId`], [`types::SeverityId`]
//! - [`types::Timestamp`], [`types::SensorType`], [`types::ColumnType`]
//! - [`cache::CacheBackend`] — subset of StorageBackend (get/set/delete)
//! - [`config::ConfigSnapshot`] — opaque config snapshot shell
//! - [`telemetry::TracingConfig`], [`telemetry::init_tracing`]

pub mod cache;
pub mod config;
pub mod error;
pub mod proofs;
pub mod storage;
pub mod telemetry;
pub mod tenant;
pub mod types;

// Flat re-exports for ergonomic use by downstream crates.
pub use cache::CacheBackend;
pub use config::ConfigSnapshot;
pub use error::PrismError;
pub use storage::{ColumnOptions, StorageDomain};
pub use telemetry::{init_tracing, TracingConfig};
pub use tenant::TenantId;
pub use types::{AnalystId, ClientId, ColumnType, SensorType, SeverityId, Timestamp};
