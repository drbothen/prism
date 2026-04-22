//! ConfigSnapshot — shell struct to break dependency cycles.
//!
//! S-1.11 (prism-spec-engine) populates this with real fields. This stub
//! exists solely so downstream crates can hold a `ConfigSnapshot` reference
//! without depending on prism-spec-engine.

use serde::{Deserialize, Serialize};

/// Opaque snapshot of the platform configuration at a given version.
///
/// `version` is a monotonically increasing counter. `raw` holds the full
/// configuration as a JSON value; structured accessors are added by S-1.11.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ConfigSnapshot {
    /// Monotonically increasing version counter.
    pub version: u64,
    /// Raw JSON representation of the configuration at this snapshot.
    pub raw: serde_json::Value,
}

impl Default for ConfigSnapshot {
    fn default() -> Self {
        Self {
            version: 0,
            raw: serde_json::Value::Object(serde_json::Map::new()),
        }
    }
}
