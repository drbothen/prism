//! DTU (Data Tenancy Unit) mode registry — S-3.0.02.
//!
//! Defines the default tenancy mode for each registered Prism type and exposes
//! the compile-time registry slice [`DTU_DEFAULT_MODE`].

use serde::Deserialize;

/// Tenancy mode governing how a type's data is partitioned.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DtuMode {
    /// Data is shared across all clients in a tenant.
    Shared,
    /// Data is isolated per client within a tenant.
    Client,
}

/// A single entry in the DTU default-mode registry.
#[derive(Debug)]
pub struct DtuRegistryEntry {
    /// Fully-qualified Rust type name (e.g. `"prism_core::alert::AlertSeverity"`).
    pub type_name: &'static str,
    /// The default [`DtuMode`] assigned to this type.
    pub default_mode: DtuMode,
    /// When `true`, this entry exists solely for test infrastructure and must not
    /// appear in production registry validation.
    pub test_only: bool,
}

/// Compile-time registry of DTU default modes.
///
/// Populated in step (c). Empty here so that AC-4 (`.len() == 10`) fails as
/// required by Red Gate discipline (BC-5.38.001).
pub static DTU_DEFAULT_MODE: &[DtuRegistryEntry] = &[];
