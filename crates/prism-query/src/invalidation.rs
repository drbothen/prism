//! `invalidation` — Synchronous cache invalidation on write operations.
//!
//! Implements BC-2.07.004. When a write operation succeeds against a sensor API,
//! all cache entries matching the `(client_id, sensor_id, source_id)` prefix are
//! **synchronously** invalidated before the write response is returned. This
//! prevents stale reads after writes (BC-2.07.004 §Write-then-read consistency).
//!
//! # Invalidation model
//! - Invalidation is a prefix scan on `(client_id, sensor_id, source_id)`.
//! - Each write tool has a static mapping to the `source_id` values it invalidates.
//! - If no entries exist for the prefix, the operation is a no-op.
//! - `configure_credential_source` and `delete_credential` do NOT invalidate cache
//!   entries — they operate on the credential store, not sensor data (BC-2.07.004
//!   §Write Tool to source_id Mapping).
//!
//! # BC References
//! - BC-2.07.004 — Cache Invalidation on Write Operations
//!
//! Story: S-3.05

use std::sync::Arc;

use prism_core::error::PrismError;
use prism_core::types::SensorType;
use prism_core::OrgSlug;

use crate::cache::QueryCache;

// ---------------------------------------------------------------------------
// WriteToolInvalidationMap
// ---------------------------------------------------------------------------

/// Static mapping from a write tool name to the `source_id` values it
/// invalidates, per BC-2.07.004 §Write Tool to source_id Mapping.
///
/// Each adapter MUST register its write tools here — omitting a mapping is a bug.
#[derive(Debug, Clone)]
pub struct WriteToolInvalidationMap {
    /// Tool name (e.g., `"crowdstrike_contain_host"`).
    pub tool_name: &'static str,
    /// source_id values to invalidate (e.g., `&["crowdstrike_hosts", "crowdstrike_detections"]`).
    pub source_ids: &'static [&'static str],
    /// The sensor that owns this write tool.
    pub sensor_type: SensorType,
}

/// Full static mapping of all write tools to their invalidation targets.
///
/// BC-2.07.004 §Write Tool to source_id Mapping (authoritative table).
/// `configure_credential_source` and `delete_credential` are excluded — they
/// do not invalidate sensor cache entries.
pub static WRITE_TOOL_INVALIDATION_MAP: &[WriteToolInvalidationMap] = &[
    WriteToolInvalidationMap {
        tool_name: "crowdstrike_contain_host",
        source_ids: &["crowdstrike_hosts", "crowdstrike_detections"],
        sensor_type: SensorType::CrowdStrike,
    },
    WriteToolInvalidationMap {
        tool_name: "crowdstrike_acknowledge_alert",
        source_ids: &["crowdstrike_alerts", "crowdstrike_detections"],
        sensor_type: SensorType::CrowdStrike,
    },
    WriteToolInvalidationMap {
        tool_name: "cyberint_acknowledge_alert",
        source_ids: &["cyberint_alerts"],
        sensor_type: SensorType::Cyberint,
    },
    WriteToolInvalidationMap {
        tool_name: "cyberint_close_alert",
        source_ids: &["cyberint_alerts"],
        sensor_type: SensorType::Cyberint,
    },
    WriteToolInvalidationMap {
        tool_name: "claroty_resolve_alert",
        source_ids: &["claroty_alerts"],
        sensor_type: SensorType::Claroty,
    },
    WriteToolInvalidationMap {
        tool_name: "claroty_device_action",
        source_ids: &["claroty_devices"],
        sensor_type: SensorType::Claroty,
    },
    WriteToolInvalidationMap {
        tool_name: "armis_update_alert_status",
        source_ids: &["armis_alerts"],
        sensor_type: SensorType::Armis,
    },
    WriteToolInvalidationMap {
        tool_name: "armis_device_action",
        source_ids: &["armis_devices"],
        sensor_type: SensorType::Armis,
    },
];

// ---------------------------------------------------------------------------
// CacheInvalidator
// ---------------------------------------------------------------------------

/// Orchestrates synchronous cache invalidation for write operations.
///
/// Holds a shared reference to the `QueryCache` and performs prefix-scan
/// eviction before the write response is returned to the caller (BC-2.07.004
/// §Postconditions — "synchronous before write response").
pub struct CacheInvalidator {
    cache: Arc<QueryCache>,
}

impl CacheInvalidator {
    /// Construct a `CacheInvalidator` wrapping the given shared cache.
    ///
    /// GREEN-BY-DESIGN: stores the `Arc` reference, no branching, no I/O, 1 line.
    pub fn new(cache: Arc<QueryCache>) -> Self {
        CacheInvalidator { cache }
    }

    /// Invalidate all cache entries for all `source_id` values associated with
    /// `sensor_type` for `client_id`.
    ///
    /// Called from write tool handlers that modify sensor data (BC-2.07.004).
    ///
    /// Performs one prefix-scan per affected `source_id`. If the `sensor_type`
    /// has no entries in `WRITE_TOOL_INVALIDATION_MAP`, this is a no-op.
    ///
    /// Errors only if the cache data structure is in an unrecoverable state
    /// (E-CACHE-001 per BC-2.07.004 §Error Cases).
    pub fn invalidate_for_sensor(
        &self,
        client_id: &OrgSlug,
        sensor_type: SensorType,
    ) -> Result<(), PrismError> {
        let sensor_name = sensor_type.to_string();
        let client_str = client_id.as_str();

        // Collect all unique source_ids for this sensor type from the map.
        let mut sources_to_invalidate: Vec<&'static str> = Vec::new();
        for entry in WRITE_TOOL_INVALIDATION_MAP {
            if entry.sensor_type == sensor_type {
                for &source_id in entry.source_ids {
                    if !sources_to_invalidate.contains(&source_id) {
                        sources_to_invalidate.push(source_id);
                    }
                }
            }
        }

        // Prefix-scan invalidation for each source_id.
        for source_id in sources_to_invalidate {
            self.cache
                .invalidate_by_prefix(client_str, &sensor_name, source_id)?;
        }

        Ok(())
    }

    /// Invalidate all cache entries for a specific write tool operation.
    ///
    /// `tool_name` is looked up in `WRITE_TOOL_INVALIDATION_MAP`; each matching
    /// `source_id` is evicted for `client_id`. If `tool_name` is not in the map,
    /// a `PrismError::Internal` is returned (missing mapping = bug).
    pub fn invalidate_for_write_tool(
        &self,
        client_id: &OrgSlug,
        tool_name: &str,
    ) -> Result<(), PrismError> {
        let mapping = WRITE_TOOL_INVALIDATION_MAP
            .iter()
            .find(|e| e.tool_name == tool_name);

        let entry = mapping.ok_or_else(|| PrismError::Internal {
            detail: format!(
                "E-INT-001: write tool '{}' has no invalidation mapping — this is a bug; \
                 add it to WRITE_TOOL_INVALIDATION_MAP",
                tool_name
            ),
        })?;

        let client_str = client_id.as_str();
        let sensor_name = entry.sensor_type.to_string();

        for &source_id in entry.source_ids {
            self.cache
                .invalidate_by_prefix(client_str, &sensor_name, source_id)?;
        }

        Ok(())
    }

    /// Invalidate all cache entries for the given `client_id` across all sensors
    /// and sources.
    ///
    /// Called from client management write operations (BC-2.07.004).
    pub fn invalidate_for_client(&self, client_id: &OrgSlug) -> Result<(), PrismError> {
        self.cache.invalidate_by_client(client_id.as_str())?;
        Ok(())
    }
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use super::*;

    /// BC-2.07.004: `WRITE_TOOL_INVALIDATION_MAP` contains all 8 write tools.
    ///
    /// GREEN-BY-DESIGN: constant length check; zero branching, no I/O, 1 line.
    #[test]
    fn test_invalidation_map_has_all_write_tools() {
        assert_eq!(
            WRITE_TOOL_INVALIDATION_MAP.len(),
            8,
            "map must contain all 8 write tools defined in BC-2.07.004"
        );
    }

    /// BC-2.07.004: `configure_credential_source` must NOT be in the map.
    ///
    /// GREEN-BY-DESIGN: pure slice scan, no branching beyond iter, no I/O, 1 line.
    #[test]
    fn test_configure_credential_source_not_in_invalidation_map() {
        let found = WRITE_TOOL_INVALIDATION_MAP
            .iter()
            .any(|e| e.tool_name == "configure_credential_source");
        assert!(
            !found,
            "BC-2.07.004: configure_credential_source must not invalidate sensor cache"
        );
    }

    /// BC-2.07.004: `crowdstrike_contain_host` invalidates crowdstrike_hosts and
    /// crowdstrike_detections.
    ///
    /// GREEN-BY-DESIGN: pure slice lookup + comparison; no I/O, no helpers.
    #[test]
    fn test_crowdstrike_contain_host_invalidates_correct_sources() {
        let entry = WRITE_TOOL_INVALIDATION_MAP
            .iter()
            .find(|e| e.tool_name == "crowdstrike_contain_host")
            .expect("crowdstrike_contain_host must be in the invalidation map");

        assert!(
            entry.source_ids.contains(&"crowdstrike_hosts"),
            "must invalidate crowdstrike_hosts"
        );
        assert!(
            entry.source_ids.contains(&"crowdstrike_detections"),
            "must invalidate crowdstrike_detections"
        );
    }

    /// AC-6 / BC-2.07.004: Invalidating a CrowdStrike sensor evicts matching
    /// cache entries before the write response is returned.
    #[test]
    fn test_ac6_sensor_invalidation_evicts_cache_entries() {
        use prism_core::tenant::OrgSlug;

        let cache = Arc::new(QueryCache::with_defaults());
        let key = crate::cache_key::CacheKey {
            client_id: "acme".to_string(),
            sensor_id: "crowdstrike".to_string(),
            source_id: "crowdstrike_detections".to_string(),
            push_down_hash: "d".repeat(64),
        };
        let rows = vec![serde_json::json!({"id": "det-1"})];
        cache.put(key.clone(), rows).expect("put must succeed");

        let invalidator = CacheInvalidator::new(Arc::clone(&cache));
        let client_id = OrgSlug::new("acme");
        invalidator
            .invalidate_for_sensor(&client_id, SensorType::CrowdStrike)
            .expect("invalidation must not fail");

        assert!(
            cache.get(&key).expect("get must not fail").is_none(),
            "AC-6: invalidation must evict cache entries for the affected sensor"
        );
    }

    /// EC-07-010 / BC-2.07.004: Invalidation with no matching entries is a no-op.
    #[test]
    fn test_ec07010_invalidation_no_matching_entries_is_noop() {
        use prism_core::tenant::OrgSlug;

        let cache = Arc::new(QueryCache::with_defaults());
        let invalidator = CacheInvalidator::new(Arc::clone(&cache));
        let client_id = OrgSlug::new("no-data");

        // Must succeed without error even if no entries exist.
        let result = invalidator.invalidate_for_sensor(&client_id, SensorType::Armis);
        assert!(
            result.is_ok(),
            "EC-07-010: invalidation with no matching entries must be a no-op"
        );
    }
}
