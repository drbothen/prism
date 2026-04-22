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

impl StorageDomain {
    /// Returns the snake_case column family name for this domain.
    pub fn column_family_name(&self) -> &'static str {
        todo!("S-1.01: implement StorageDomain::column_family_name")
    }

    /// Returns a static slice of all 16 `StorageDomain` variants.
    ///
    /// Used during RocksDB initialization to open/create all column families.
    pub fn all() -> &'static [StorageDomain] {
        todo!("S-1.01: implement StorageDomain::all")
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
