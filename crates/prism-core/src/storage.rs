//! StorageDomain enum — maps to RocksDB column families.
//! ColumnOptions — per-column-family configuration.

use serde::{Deserialize, Serialize};

/// Enumerates all 16 RocksDB column families used by Prism.
///
/// `StorageDomain::column_family_name()` returns the snake_case string used
/// to open/create the column family. `StorageDomain::all()` returns a static
/// slice of all 16 variants for use during storage initialization.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StorageDomain {
    Default,
    Schedules,
    DiffResults,
    DetectionRules,
    DetectionState,
    Alerts,
    Cases,
    AuditBuffer,
    DirtyBits,
    Watchdog,
    Aliases,
    Decorators,
    InfusionCache,
    ActionState,
    PluginState,
    EventBuffer,
}

/// All 16 `StorageDomain` variants in a static array.
///
/// Used by `StorageDomain::all()` to avoid heap allocation in the
/// storage initialization hot path.
const ALL_DOMAINS: [StorageDomain; 16] = [
    StorageDomain::Default,
    StorageDomain::Schedules,
    StorageDomain::DiffResults,
    StorageDomain::DetectionRules,
    StorageDomain::DetectionState,
    StorageDomain::Alerts,
    StorageDomain::Cases,
    StorageDomain::AuditBuffer,
    StorageDomain::DirtyBits,
    StorageDomain::Watchdog,
    StorageDomain::Aliases,
    StorageDomain::Decorators,
    StorageDomain::InfusionCache,
    StorageDomain::ActionState,
    StorageDomain::PluginState,
    StorageDomain::EventBuffer,
];

impl StorageDomain {
    /// Returns the snake_case column family name for this domain.
    pub fn column_family_name(&self) -> &'static str {
        match self {
            StorageDomain::Default => "default",
            StorageDomain::Schedules => "schedules",
            StorageDomain::DiffResults => "diff_results",
            StorageDomain::DetectionRules => "detection_rules",
            StorageDomain::DetectionState => "detection_state",
            StorageDomain::Alerts => "alerts",
            StorageDomain::Cases => "cases",
            StorageDomain::AuditBuffer => "audit_buffer",
            StorageDomain::DirtyBits => "dirty_bits",
            StorageDomain::Watchdog => "watchdog",
            StorageDomain::Aliases => "aliases",
            StorageDomain::Decorators => "decorators",
            StorageDomain::InfusionCache => "infusion_cache",
            StorageDomain::ActionState => "action_state",
            StorageDomain::PluginState => "plugin_state",
            StorageDomain::EventBuffer => "event_buffer",
        }
    }

    /// Returns a static slice of all 16 `StorageDomain` variants.
    ///
    /// Used during RocksDB initialization to open/create all column families.
    pub fn all() -> &'static [StorageDomain] {
        &ALL_DOMAINS
    }
}

impl std::fmt::Display for StorageDomain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.column_family_name())
    }
}

/// Per-column-family configuration passed to the storage backend on init.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ColumnOptions {
    /// Optional TTL in seconds. `None` means no expiry.
    pub ttl_seconds: Option<u64>,
    /// Whether to enable block compression for this column family.
    pub compression: bool,
    /// LRU block cache allocation in megabytes.
    pub block_cache_mb: u32,
}

impl Default for ColumnOptions {
    fn default() -> Self {
        Self {
            ttl_seconds: None,
            compression: true,
            block_cache_mb: 8,
        }
    }
}
