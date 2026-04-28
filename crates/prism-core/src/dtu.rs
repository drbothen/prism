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

/// Compile-time registry of DTU default modes per ADR-007 §2.3.
///
/// Contains all 10 known DTU types: 4 Security Telemetry (Client, production),
/// 5 MSSP Coordination (Shared, production), and 1 test-infrastructure entry.
pub static DTU_DEFAULT_MODE: &[DtuRegistryEntry] = &[
    // Security Telemetry — Client mode, production (AC-6 / VP-093)
    DtuRegistryEntry {
        type_name: "claroty",
        default_mode: DtuMode::Client,
        test_only: false,
    },
    DtuRegistryEntry {
        type_name: "armis",
        default_mode: DtuMode::Client,
        test_only: false,
    },
    DtuRegistryEntry {
        type_name: "crowdstrike",
        default_mode: DtuMode::Client,
        test_only: false,
    },
    DtuRegistryEntry {
        type_name: "cyberint",
        default_mode: DtuMode::Client,
        test_only: false,
    },
    // D-051: test infrastructure only — excluded from production-allowed set by validator (E-CFG-013)
    DtuRegistryEntry {
        type_name: "demo-server",
        default_mode: DtuMode::Client,
        test_only: true,
    },
    // MSSP Coordination — Shared mode, production (AC-5)
    DtuRegistryEntry {
        type_name: "slack",
        default_mode: DtuMode::Shared,
        test_only: false,
    },
    DtuRegistryEntry {
        type_name: "pagerduty",
        default_mode: DtuMode::Shared,
        test_only: false,
    },
    DtuRegistryEntry {
        type_name: "jira",
        default_mode: DtuMode::Shared,
        test_only: false,
    },
    DtuRegistryEntry {
        type_name: "nvd",
        default_mode: DtuMode::Shared,
        test_only: false,
    },
    DtuRegistryEntry {
        type_name: "threatintel",
        default_mode: DtuMode::Shared,
        test_only: false,
    },
];
