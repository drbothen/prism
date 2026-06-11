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

use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, LazyLock, RwLock,
};

use prism_core::{error::PrismError, OrgSlug, SensorId};
use prism_spec_engine::error::SpecEngineError;

use crate::cache::{CacheValue, GenericQueryCache};

// ---------------------------------------------------------------------------
// Dynamic write-tool registry (S-PLUGIN-PREREQ-E AC-9 / Task 7)
// ---------------------------------------------------------------------------

/// Runtime-extensible container for plugin-registered write tools.
///
/// Separate from `WRITE_TOOL_INVALIDATION_MAP` (the static built-in list) so
/// that plugin registrations during boot step 7.5 do not mutate the static list.
/// Read path combines both sources (static + dynamic).
///
/// Story: S-PLUGIN-PREREQ-E AC-9 / Task 7 | BC-2.16.012 | ADR-026 §D7
static DYNAMIC_WRITE_TOOLS: RwLock<Vec<WriteToolInvalidationMap>> = RwLock::new(Vec::new());

// ---------------------------------------------------------------------------
// WriteToolInvalidationMap
// ---------------------------------------------------------------------------

/// Mapping from a write tool name to the `source_id` values it invalidates,
/// per BC-2.07.004 §Write Tool to source_id Mapping.
///
/// Each adapter MUST register its write tools here — omitting a mapping is a bug.
///
/// `sensor_id` is a `SensorId` (open newtype) — replaces the previous
/// `sensor_name: &'static str` field (S-PLUGIN-PREREQ-A / F-LP1-MED-003).
///
/// `plugin_name` is set by `PluginRuntime` from the plugin manifest `name` field
/// (S-PLUGIN-PREREQ-E / ADR-026 D7 v1.23; AC-9). For built-in sensors registered
/// in the static `WRITE_TOOL_INVALIDATION_MAP`, this field is empty (`""`).
/// This field is the source for the `plugin_name` structured event field in the
/// `write_tool_registration_after_boot` WARN event (BC-2.16.002 row 33).
#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct WriteToolInvalidationMap {
    /// Tool name (e.g., `"crowdstrike_contain_host"`).
    ///
    /// `String` (not `&'static str`) to support plugin-registered write tools where
    /// the name is allocated at runtime from the plugin manifest. Built-in static entries
    /// use `.to_string()` / string literal conversion at init time (F-LP-IMPL-P1-007).
    pub tool_name: String,
    /// source_id values to invalidate (e.g., `["crowdstrike_hosts", "crowdstrike_detections"]`).
    ///
    /// `Vec<String>` (not `&'static [&'static str]`) for the same runtime allocation reason
    /// as `tool_name` — plugin-provided source_ids are not `'static` (F-LP-IMPL-P1-007).
    pub source_ids: Vec<String>,
    /// Sensor identity — the owning sensor for this write tool.
    /// Use `entry.sensor_id == *probe_sensor_id` for lookup comparisons.
    pub sensor_id: SensorId,
    /// Plugin that registered this write tool; empty for built-in static entries.
    /// Set from plugin manifest `name` field by `PluginRuntime` at registration time
    /// (ADR-026 D7 v1.23; S-PLUGIN-PREREQ-E AC-9).
    pub plugin_name: String,
}

impl WriteToolInvalidationMap {
    /// Construct a new `WriteToolInvalidationMap` entry.
    ///
    /// Provided for external callers — `#[non_exhaustive]` prevents struct-literal
    /// construction from outside the crate (forward-compat: future fields will not
    /// silently become required). Internal initialisation in `WRITE_TOOL_INVALIDATION_MAP`
    /// uses struct-literal syntax directly (same crate, no restriction applies).
    ///
    /// # Parameters
    /// - `tool_name` — MCP tool name (e.g., `"crowdstrike_contain_host"`).
    /// - `source_ids` — source-id values to invalidate on write success.
    /// - `sensor_id` — owning sensor identity.
    /// - `plugin_name` — plugin manifest name; pass `""` for built-in entries.
    pub fn new(
        tool_name: impl Into<String>,
        source_ids: Vec<String>,
        sensor_id: SensorId,
        plugin_name: impl Into<String>,
    ) -> Self {
        Self {
            tool_name: tool_name.into(),
            source_ids,
            sensor_id,
            plugin_name: plugin_name.into(),
        }
    }
}

// ---------------------------------------------------------------------------
// AtomicBool query-phase flag (ADR-026 §D7; S-PLUGIN-PREREQ-E Task 7b / AC-9)
// ---------------------------------------------------------------------------

/// Flag set to `true` when the query phase begins (ADR-022 §B step 8 boundary).
///
/// Once set, no further `register_write_tool()` calls are accepted — the
/// write-tool registration window is permanently closed. Callers after this
/// point receive `SpecEngineError::WriteToolRegistrationAfterBoot` with an
/// accompanying structured WARN tracing event (BC-2.16.002 row 33).
///
/// Set via `mark_query_phase_started()`. Read via `Ordering::Acquire` in
/// `register_write_tool()`.
static QUERY_PHASE_STARTED: AtomicBool = AtomicBool::new(false);

/// Marks the query phase as started, permanently closing the write-tool
/// registration window.
///
/// Called as the FIRST statement of the step-8 init function in
/// `crates/prism-bin/src/boot.rs` — immediately before `QueryEngine::new()` —
/// to close the registration window before any queries can be dispatched.
/// All plugin registrations at step 7.5 are already complete when step 8 starts.
///
/// After this call, any `register_write_tool()` invocation returns
/// `Err(SpecEngineError::WriteToolRegistrationAfterBoot)` and emits a WARN
/// structured tracing event (BC-2.16.002 §Postconditions row 33; ADR-026 §D7).
///
/// Story: S-PLUGIN-PREREQ-E AC-9 / Task 7b | ADR-026 §D7 | ADR-022 §B step 7.5/8
pub fn mark_query_phase_started() {
    QUERY_PHASE_STARTED.store(true, Ordering::Release);
}

/// Register a plugin-provided write tool in the runtime-extensible invalidation map.
///
/// Checks the `QUERY_PHASE_STARTED` flag before acquiring the write guard:
/// - If the query phase has already started, emits a WARN tracing event and
///   returns `Err(SpecEngineError::WriteToolRegistrationAfterBoot)` (ADR-026 §D7 fail-closed).
/// - If a duplicate `tool_name` already exists, returns
///   `Err(SpecEngineError::DuplicateWriteToolRegistration(tool_name))`.
/// - Otherwise pushes `entry` and returns `Ok(())`.
///
/// Story: S-PLUGIN-PREREQ-E AC-9 / Task 7 | BC-2.16.012 EC-016-012-004/005 | ADR-026 §D7
pub fn register_write_tool(entry: WriteToolInvalidationMap) -> Result<(), SpecEngineError> {
    // Check query-phase flag before acquiring the write guard (ADR-026 §D7 fail-closed).
    if QUERY_PHASE_STARTED.load(Ordering::Acquire) {
        tracing::warn!(
            event_type = "write_tool_registration_after_boot",
            plugin_name = %entry.plugin_name,
            tool_name = %entry.tool_name,
            error = "E-PLUGIN-020",
            "write tool registration rejected — query phase already started"
        );
        return Err(SpecEngineError::WriteToolRegistrationAfterBoot);
    }

    // Acquire write guard and check for duplicate tool_name.
    // RwLock poisoning indicates a prior panic in a write guard holder — an
    // unrecoverable state. Map to distinct WriteToolRegistryPoisoned variant
    // (E-PLUGIN-021) so operators receive a diagnostic that names the actual cause
    // rather than the misleading "query phase already started" (E-PLUGIN-020)
    // message from WriteToolRegistrationAfterBoot (F-LP-IMPL-P1-004).
    let mut guard = DYNAMIC_WRITE_TOOLS
        .write()
        .map_err(|_| SpecEngineError::WriteToolRegistryPoisoned)?;

    if guard.iter().any(|e| e.tool_name == entry.tool_name) {
        return Err(SpecEngineError::DuplicateWriteToolRegistration(
            entry.tool_name.to_string(),
        ));
    }

    guard.push(entry);
    Ok(())
}

/// Remove all dynamically-registered write tools for a given plugin from `DYNAMIC_WRITE_TOOLS`.
///
/// Used by boot step 7.6 to roll back all write-tool registrations for a plugin when any one
/// of that plugin's write tools fails to register (fail-closed per BC-2.07.004 §write-then-read
/// consistency: a plugin with partial write-tool registration would produce stale reads).
///
/// Returns the count of removed entries (0 if plugin had no registered tools or on lock poison).
///
/// Story: S-PLUGIN-PREREQ-E / F-LP-IMPL-P4-002 | BC-2.07.004 | BC-2.16.012 EC-016-012-004
pub fn deregister_write_tools_for_plugin(plugin_name: &str) -> usize {
    let mut guard = match DYNAMIC_WRITE_TOOLS.write() {
        Ok(g) => g,
        Err(_) => return 0, // poisoned — nothing to rollback
    };
    let before = guard.len();
    guard.retain(|e| e.plugin_name != plugin_name);
    before - guard.len()
}

/// Return the number of dynamically-registered write tools in `DYNAMIC_WRITE_TOOLS`.
///
/// Used by integration tests (in `prism-bin`) to assert that plugin write tool
/// registrations succeeded without needing direct access to the private static
/// (F-LP-IMPL-P1-002 test verification; S-PLUGIN-PREREQ-E AC-9 / ADR-026 §D7).
///
/// Returns `0` if the registry is poisoned (mirrors fail-safe semantics).
pub fn dynamic_write_tool_count() -> usize {
    match DYNAMIC_WRITE_TOOLS.read() {
        Ok(guard) => guard.len(),
        Err(_) => 0,
    }
}

/// Reset the `DYNAMIC_WRITE_TOOLS` registry to empty.
///
/// ONLY for use in integration tests that need to reset cross-binary global state.
/// Gated on `#[cfg(any(test, feature = "test-helpers"))]` to prevent production use.
/// Production code never calls this function.
///
/// Story: S-PLUGIN-PREREQ-E / F-LP-IMPL-P1-005 cross-crate isolation
#[cfg(any(test, feature = "test-helpers"))]
pub fn reset_dynamic_registry_global() {
    // Recover from poisoning by calling into_inner() on PoisonError.
    match DYNAMIC_WRITE_TOOLS.write() {
        Ok(mut guard) => guard.clear(),
        Err(poisoned) => poisoned.into_inner().clear(),
    }
}

/// Reset the `QUERY_PHASE_STARTED` flag to `false`.
///
/// ONLY for use in integration tests. Gated on test/test-helpers cfg.
///
/// Story: S-PLUGIN-PREREQ-E / F-LP-IMPL-P1-005 cross-crate isolation
#[cfg(any(test, feature = "test-helpers"))]
pub fn reset_query_phase_global() {
    QUERY_PHASE_STARTED.store(false, Ordering::Release);
}

/// Built-in write tool invalidation map for the four initial sensors.
///
/// **Transitional:** This static list is the source for `register_builtin_write_tools()`
/// which populates `DYNAMIC_WRITE_TOOLS` at boot. After PLUGIN-MIGRATION-001-F
/// (test rewrite), this static will be emptied and `DYNAMIC_WRITE_TOOLS` will be
/// the sole source. The static is retained in this story to avoid breaking the
/// existing `invalidate_for_sensor` code path which currently iterates both.
///
/// Story: PLUGIN-MIGRATION-001-B AC-003 | BC-2.16.012 INV-INVALIDATION-EXT-001
pub static WRITE_TOOL_INVALIDATION_MAP: LazyLock<Vec<WriteToolInvalidationMap>> =
    LazyLock::new(|| {
        vec![
            WriteToolInvalidationMap {
                tool_name: "crowdstrike_contain_host".to_string(),
                source_ids: vec![
                    "crowdstrike_hosts".to_string(),
                    "crowdstrike_detections".to_string(),
                ],
                sensor_id: SensorId::from("crowdstrike"),
                plugin_name: String::new(), // built-in; no plugin origin
            },
            WriteToolInvalidationMap {
                tool_name: "crowdstrike_acknowledge_alert".to_string(),
                source_ids: vec![
                    "crowdstrike_alerts".to_string(),
                    "crowdstrike_detections".to_string(),
                ],
                sensor_id: SensorId::from("crowdstrike"),
                plugin_name: String::new(), // built-in; no plugin origin
            },
            WriteToolInvalidationMap {
                tool_name: "cyberint_acknowledge_alert".to_string(),
                source_ids: vec!["cyberint_alerts".to_string()],
                sensor_id: SensorId::from("cyberint"),
                plugin_name: String::new(), // built-in; no plugin origin
            },
            WriteToolInvalidationMap {
                tool_name: "cyberint_close_alert".to_string(),
                source_ids: vec!["cyberint_alerts".to_string()],
                sensor_id: SensorId::from("cyberint"),
                plugin_name: String::new(), // built-in; no plugin origin
            },
            WriteToolInvalidationMap {
                tool_name: "claroty_resolve_alert".to_string(),
                source_ids: vec!["claroty_alerts".to_string()],
                sensor_id: SensorId::from("claroty"),
                plugin_name: String::new(), // built-in; no plugin origin
            },
            WriteToolInvalidationMap {
                tool_name: "claroty_device_action".to_string(),
                source_ids: vec!["claroty_devices".to_string()],
                sensor_id: SensorId::from("claroty"),
                plugin_name: String::new(), // built-in; no plugin origin
            },
            WriteToolInvalidationMap {
                tool_name: "armis_update_alert_status".to_string(),
                source_ids: vec!["armis_alerts".to_string()],
                sensor_id: SensorId::from("armis"),
                plugin_name: String::new(), // built-in; no plugin origin
            },
            WriteToolInvalidationMap {
                tool_name: "armis_device_action".to_string(),
                source_ids: vec!["armis_devices".to_string()],
                sensor_id: SensorId::from("armis"),
                plugin_name: String::new(), // built-in; no plugin origin
            },
        ]
    });

/// Populate `DYNAMIC_WRITE_TOOLS` from the built-in static map.
///
/// Called ONCE at boot, before `mark_query_phase_started()`. After this call,
/// `DYNAMIC_WRITE_TOOLS` contains a copy of all built-in entries. Both the static
/// `WRITE_TOOL_INVALIDATION_MAP` and `DYNAMIC_WRITE_TOOLS` are still consulted by
/// `invalidate_for_*` methods (transitional; see `WRITE_TOOL_INVALIDATION_MAP` doc
/// comment for removal plan).
///
/// Not idempotent: if called on an already-populated registry (duplicate tool_name
/// returns `Err(DuplicateWriteToolRegistration)` per BC-2.16.012 EC-016-012-004 —
/// caller must ensure this is called exactly once per process).
///
/// # Boot wiring
/// Must be called at step 7.5c of `crates/prism-bin/src/boot.rs`, immediately after
/// plugin loading (step 7.5) and before `mark_query_phase_started()` at step 8.
/// See PLUGIN-MIGRATION-001-B AC-003 and BC-2.16.012 INV-INVALIDATION-EXT-001.
///
/// Story: PLUGIN-MIGRATION-001-B AC-003 | BC-2.16.012 INV-INVALIDATION-EXT-001
pub fn register_builtin_write_tools() -> Result<(), SpecEngineError> {
    for entry in WRITE_TOOL_INVALIDATION_MAP.iter() {
        register_write_tool(entry.clone())?;
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// CacheInvalidator
// ---------------------------------------------------------------------------

/// Orchestrates synchronous cache invalidation for write operations.
///
/// Holds a shared reference to the sensor-fetch response cache and performs
/// prefix-scan eviction before the write response is returned to the caller
/// (BC-2.07.004 §Postconditions — "synchronous before write response").
///
/// Generic over the cache value type so it can wrap either the JSON-row
/// instantiation (`QueryCache`, unit tests) or the production
/// `SensorResponseCache` shared with `QueryEngine` (defaults to the JSON-row
/// form for backward compatibility).
pub struct CacheInvalidator<V: CacheValue = Vec<serde_json::Value>> {
    cache: Arc<GenericQueryCache<V>>,
}

impl<V: CacheValue> CacheInvalidator<V> {
    /// Construct a `CacheInvalidator` wrapping the given shared cache.
    ///
    /// GREEN-BY-DESIGN: stores the `Arc` reference, no branching, no I/O, 1 line.
    pub fn new(cache: Arc<GenericQueryCache<V>>) -> Self {
        CacheInvalidator { cache }
    }

    /// Invalidate cache entries for a completed write operation, preferring the
    /// per-tool mapping and falling back to sensor-wide invalidation when the
    /// tool has no mapping (BC-2.07.004 §Write Tool to source_id Mapping).
    ///
    /// Per BC-2.07.004, every write tool MUST have an invalidation mapping —
    /// "omitting a mapping is a bug". Failing the (already-succeeded) sensor
    /// write because of that bug would be the wrong tradeoff; instead this
    /// method logs a WARN and invalidates ALL source_ids mapped to the sensor
    /// (a strict superset, so the write-then-read consistency invariant still
    /// holds while the missing mapping is surfaced for fixing).
    ///
    /// Returns `Ok(n)` where `n` is the total number of entries evicted (I-2:
    /// BC-2.07.004 §Postconditions audit count).
    pub fn invalidate_for_write_operation(
        &self,
        client_id: &OrgSlug,
        sensor_id: &SensorId,
        tool_name: &str,
    ) -> Result<usize, PrismError> {
        match self.invalidate_for_write_tool(client_id, tool_name) {
            Ok(n) => Ok(n),
            Err(PrismError::Internal { detail }) if detail.starts_with("E-INT-001") => {
                // Missing per-tool mapping — a BC-2.07.004 mapping bug. Surface it
                // and fall back to sensor-wide invalidation (safe superset) so the
                // write-then-read consistency invariant is preserved.
                tracing::warn!(
                    tool_name,
                    sensor_id = %sensor_id,
                    "write tool has no invalidation mapping (BC-2.07.004 bug) — \
                     falling back to sensor-wide cache invalidation"
                );
                self.invalidate_for_sensor(client_id, sensor_id)
            }
            Err(e) => Err(e),
        }
    }

    /// Invalidate all cache entries for all `source_id` values associated with
    /// `sensor_id` for `client_id`.
    ///
    /// Called from write tool handlers that modify sensor data (BC-2.07.004).
    ///
    /// Performs one prefix-scan per affected `source_id`. If the `sensor_id`
    /// has no entries in `WRITE_TOOL_INVALIDATION_MAP`, this is a no-op.
    ///
    /// Returns `Ok(n)` where `n` is the total number of entries evicted across all
    /// affected source_ids (I-2: BC-2.07.004 §Postconditions (audit count) — the
    /// caller can include this count in the `AuditEntry` for the write operation).
    ///
    /// Errors only if the cache data structure is in an unrecoverable state
    /// (E-CACHE-001 per BC-2.07.004 §Error Cases).
    pub fn invalidate_for_sensor(
        &self,
        client_id: &OrgSlug,
        sensor_id: &SensorId,
    ) -> Result<usize, PrismError> {
        let sensor_name = sensor_id.as_ref();
        let client_str = client_id.as_str();

        // Collect all unique source_ids for this sensor from BOTH the static map
        // and the dynamic registry (F-LP-IMPL-P1-001: DYNAMIC_WRITE_TOOLS was
        // previously never read; both sources must be consulted per BC-2.07.004).
        let mut sources_to_invalidate: Vec<String> = Vec::new();

        // Static built-in write tools.
        for entry in WRITE_TOOL_INVALIDATION_MAP.iter() {
            if entry.sensor_id == *sensor_id {
                for source_id in &entry.source_ids {
                    if !sources_to_invalidate.contains(source_id) {
                        sources_to_invalidate.push(source_id.clone());
                    }
                }
            }
        }

        // Dynamic plugin-registered write tools (held for the scan then released).
        // RwLock poisoning on the read guard is highly unlikely (write-guard panics
        // would have to have occurred during boot registration); propagate as Internal.
        let dynamic_guard = DYNAMIC_WRITE_TOOLS
            .read()
            .map_err(|_| PrismError::Internal {
                detail: "E-INT-002: DYNAMIC_WRITE_TOOLS RwLock is poisoned".to_string(),
            })?;
        for entry in dynamic_guard.iter() {
            if entry.sensor_id == *sensor_id {
                for source_id in &entry.source_ids {
                    if !sources_to_invalidate.contains(source_id) {
                        sources_to_invalidate.push(source_id.clone());
                    }
                }
            }
        }
        drop(dynamic_guard); // Release before cache I/O.

        // Prefix-scan invalidation for each source_id; sum evicted counts.
        let mut total_evicted: usize = 0;
        for source_id in &sources_to_invalidate {
            let n = self
                .cache
                .invalidate_by_prefix(client_str, sensor_name, source_id.as_str())?;
            total_evicted = total_evicted.saturating_add(n);
        }

        Ok(total_evicted)
    }

    /// Invalidate all cache entries for a specific write tool operation.
    ///
    /// `tool_name` is looked up in BOTH `WRITE_TOOL_INVALIDATION_MAP` (static built-ins)
    /// AND `DYNAMIC_WRITE_TOOLS` (plugin-registered, held under read guard); each matching
    /// `source_id` is evicted for `client_id`. If `tool_name` is not in either map,
    /// a `PrismError::Internal` is returned (missing mapping = bug).
    ///
    /// Read guard over `DYNAMIC_WRITE_TOOLS` is acquired FIRST, held for the lookup scan,
    /// then released before cache I/O (F-LP-IMPL-P1-001: per story Task 7 "acquire a read
    /// guard instead of dereferencing the LazyLock"; BC-2.07.004 §Write-then-read
    /// consistency invariant for plugin tools).
    ///
    /// Returns `Ok(n)` where `n` is the total number of entries evicted (I-2:
    /// BC-2.07.004 §Postconditions (audit count)).
    pub fn invalidate_for_write_tool(
        &self,
        client_id: &OrgSlug,
        tool_name: &str,
    ) -> Result<usize, PrismError> {
        // Acquire the dynamic registry read guard FIRST (per F-LP-IMPL-P1-001 ordering).
        let dynamic_guard = DYNAMIC_WRITE_TOOLS
            .read()
            .map_err(|_| PrismError::Internal {
                detail: "E-INT-002: DYNAMIC_WRITE_TOOLS RwLock is poisoned".to_string(),
            })?;

        // Search static map first (common case for built-in sensors), then dynamic.
        let entry: Option<std::borrow::Cow<WriteToolInvalidationMap>> = WRITE_TOOL_INVALIDATION_MAP
            .iter()
            .find(|e| e.tool_name == tool_name)
            .map(std::borrow::Cow::Borrowed)
            .or_else(|| {
                dynamic_guard
                    .iter()
                    .find(|e| e.tool_name == tool_name)
                    .map(|e| std::borrow::Cow::Owned(e.clone()))
            });

        let entry = entry.ok_or_else(|| PrismError::Internal {
            detail: format!(
                "E-INT-001: write tool '{}' has no invalidation mapping — this is a bug; \
                 add it to WRITE_TOOL_INVALIDATION_MAP or register it via register_write_tool()",
                tool_name
            ),
        })?;

        // Release the read guard before performing cache I/O.
        drop(dynamic_guard);

        let client_str = client_id.as_str();
        let sensor_name = entry.sensor_id.as_ref();

        let mut total_evicted: usize = 0;
        for source_id in &entry.source_ids {
            let n = self
                .cache
                .invalidate_by_prefix(client_str, sensor_name, source_id.as_str())?;
            total_evicted = total_evicted.saturating_add(n);
        }

        Ok(total_evicted)
    }

    /// Invalidate all cache entries for the given `client_id` across all sensors
    /// and sources.
    ///
    /// Called from client management write operations (BC-2.07.004).
    ///
    /// Returns `Ok(n)` where `n` is the total number of entries evicted (I-2:
    /// BC-2.07.004 §Postconditions (audit count)).
    pub fn invalidate_for_client(&self, client_id: &OrgSlug) -> Result<usize, PrismError> {
        let n = self.cache.invalidate_by_client(client_id.as_str())?;
        Ok(n)
    }

    /// Invalidate ALL cache entries across every client, sensor, and source.
    ///
    /// Config hot-reload flush (P1-03 interim mitigation, review 2026-06-10):
    /// called when the sensor-spec config snapshot is swapped (BC-2.16.005
    /// `reload_config`, `add_sensor_spec`, and the S-1.12-FOLLOWUP hot-reload
    /// watcher — all funnel through `ConfigManager::store`). Cached entries
    /// store responses in their normalized form (human-authorized BC-2.07.003
    /// deviation; spec compliance lands in S-CACHE-SPEC-COMPLIANCE-001), so a
    /// spec change invalidates every entry's normalization — a full flush is
    /// required to prevent serving data normalized under a retired spec.
    ///
    /// Returns `Ok(n)` where `n` is the total number of entries evicted
    /// (audit visibility, consistent with the BC-2.07.004 invalidation paths).
    pub fn invalidate_all(&self) -> Result<usize, PrismError> {
        self.cache.invalidate_all()
    }
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::cache::QueryCache;

    /// P1-03 interim mitigation: `CacheInvalidator::invalidate_all` flushes the
    /// entire wrapped cache — hit before, miss after — and reports the evicted
    /// count for audit visibility (BC-2.16.005 reload → response-cache flush).
    #[test]
    fn test_p1_03_invalidator_invalidate_all_flushes_entire_cache() {
        use std::sync::Arc;

        let cache = Arc::new(QueryCache::with_defaults());
        let make_key = |client: &str, src: &str, h: char| crate::cache_key::CacheKey {
            client_id: client.to_string(),
            sensor_id: prism_core::SensorId::from("crowdstrike"),
            source_id: src.to_string(),
            push_down_hash: h.to_string().repeat(64),
        };
        let k1 = make_key("acme", "cs_devices", 'a');
        let k2 = make_key("globex", "cs_detections", 'b');
        cache
            .put(k1.clone(), vec![serde_json::json!({"id": 1})])
            .expect("put k1");
        cache
            .put(k2.clone(), vec![serde_json::json!({"id": 2})])
            .expect("put k2");
        assert!(
            cache.get(&k1).expect("get k1").is_some() && cache.get(&k2).expect("get k2").is_some(),
            "pre-condition: both entries hit"
        );

        let invalidator = CacheInvalidator::new(Arc::clone(&cache));
        let evicted = invalidator
            .invalidate_all()
            .expect("invalidate_all must succeed");

        assert_eq!(evicted, 2, "P1-03: both entries must be reported evicted");
        assert!(
            cache.get(&k1).expect("get k1").is_none() && cache.get(&k2).expect("get k2").is_none(),
            "P1-03: all entries must miss after invalidate_all"
        );
        assert_eq!(
            cache.total_bytes(),
            0,
            "P1-03: byte accounting must return to 0 after full flush"
        );
    }

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

    /// BC-2.07.004 / OBS-006: `delete_credential` must NOT be in the invalidation map.
    ///
    /// `delete_credential` operates on the credential store, not sensor data, so it
    /// must not trigger cache invalidation (mirrors configure_credential_source semantics).
    ///
    /// GREEN-BY-DESIGN: pure slice scan, no branching beyond iter, no I/O, 1 line.
    #[test]
    fn test_delete_credential_not_in_invalidation_map() {
        let found = WRITE_TOOL_INVALIDATION_MAP
            .iter()
            .any(|e| e.tool_name == "delete_credential");
        assert!(
            !found,
            "BC-2.07.004: delete_credential must not invalidate sensor cache (credential store operation only)"
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
            entry.source_ids.iter().any(|s| s == "crowdstrike_hosts"),
            "must invalidate crowdstrike_hosts"
        );
        assert!(
            entry
                .source_ids
                .iter()
                .any(|s| s == "crowdstrike_detections"),
            "must invalidate crowdstrike_detections"
        );
    }

    /// AC-6 / BC-2.07.004: Invalidating a CrowdStrike sensor evicts matching
    /// cache entries before the write response is returned.
    #[test]
    fn test_ac6_sensor_invalidation_evicts_cache_entries() {
        use prism_core::tenant::OrgSlug;

        // Reset global state for test isolation (handles post-poisoning recovery).
        reset_query_phase_for_test();
        reset_dynamic_registry_for_test();

        let cache = Arc::new(QueryCache::with_defaults());
        let key = crate::cache_key::CacheKey {
            client_id: "acme".to_string(),
            sensor_id: prism_core::SensorId::from("crowdstrike"),
            source_id: "crowdstrike_detections".to_string(),
            push_down_hash: "d".repeat(64),
        };
        let rows = vec![serde_json::json!({"id": "det-1"})];
        cache.put(key.clone(), rows).expect("put must succeed");

        let invalidator = CacheInvalidator::new(Arc::clone(&cache));
        let client_id = OrgSlug::new("acme");
        invalidator
            .invalidate_for_sensor(&client_id, &prism_core::SensorId::from("crowdstrike"))
            .expect("invalidation must not fail");

        assert!(
            cache.get(&key).expect("get must not fail").is_none(),
            "AC-6: invalidation must evict cache entries for the affected sensor"
        );
    }

    /// I-2 / BC-2.07.004: `invalidate_for_sensor` sums eviction counts across
    /// all source_ids for the sensor.
    ///
    /// Inserts one entry for `crowdstrike_hosts` and one for
    /// `crowdstrike_detections` (both map to sensor "crowdstrike").
    /// Calls `invalidate_for_sensor(SensorId::from("crowdstrike"))` and asserts
    /// the returned count equals 2 (one per source_id), exercising the
    /// `saturating_add` aggregation path in the CacheInvalidator wrapper.
    #[test]
    fn test_i2_invalidate_for_sensor_returns_sum_across_sources() {
        use prism_core::tenant::OrgSlug;

        // Reset global state for test isolation (handles post-poisoning recovery).
        reset_query_phase_for_test();
        reset_dynamic_registry_for_test();

        let cache = Arc::new(QueryCache::with_defaults());

        // Insert one entry for crowdstrike_hosts.
        let key_hosts = crate::cache_key::CacheKey {
            client_id: "acme".to_string(),
            sensor_id: prism_core::SensorId::from("crowdstrike"),
            source_id: "crowdstrike_hosts".to_string(),
            push_down_hash: "a".repeat(64),
        };
        cache
            .put(key_hosts.clone(), vec![serde_json::json!({"id": "host-1"})])
            .expect("put hosts must succeed");

        // Insert one entry for crowdstrike_detections.
        let key_dets = crate::cache_key::CacheKey {
            client_id: "acme".to_string(),
            sensor_id: prism_core::SensorId::from("crowdstrike"),
            source_id: "crowdstrike_detections".to_string(),
            push_down_hash: "b".repeat(64),
        };
        cache
            .put(key_dets.clone(), vec![serde_json::json!({"id": "det-1"})])
            .expect("put detections must succeed");

        let invalidator = CacheInvalidator::new(Arc::clone(&cache));
        let client_id = OrgSlug::new("acme");

        let evicted = invalidator
            .invalidate_for_sensor(&client_id, &prism_core::SensorId::from("crowdstrike"))
            .expect("invalidation must not fail");

        // crowdstrike_contain_host maps to [hosts, detections] and
        // crowdstrike_acknowledge_alert maps to [alerts, detections], so
        // "crowdstrike" aggregates: hosts + alerts + detections.
        // We inserted into hosts (1) and detections (1) only — both evicted.
        assert_eq!(
            evicted, 2,
            "I-2: invalidate_for_sensor must return sum across source_ids; \
             expected exactly 2 (hosts=1 + detections=1, alerts=0), got {evicted}"
        );

        // Both entries must be gone.
        assert!(
            cache.get(&key_hosts).expect("get must not fail").is_none(),
            "I-2: crowdstrike_hosts entry must be evicted"
        );
        assert!(
            cache.get(&key_dets).expect("get must not fail").is_none(),
            "I-2: crowdstrike_detections entry must be evicted"
        );
    }

    /// BC-2.07.004: `invalidate_for_write_operation` with a MAPPED tool name
    /// performs the per-tool prefix eviction (same scope as
    /// `invalidate_for_write_tool`).
    #[test]
    fn test_invalidate_for_write_operation_mapped_tool_uses_tool_mapping() {
        use prism_core::tenant::OrgSlug;

        reset_query_phase_for_test();
        reset_dynamic_registry_for_test();

        let cache = Arc::new(QueryCache::with_defaults());
        // cyberint_acknowledge_alert maps ONLY to cyberint_alerts.
        let key_alerts = crate::cache_key::CacheKey {
            client_id: "acme".to_string(),
            sensor_id: prism_core::SensorId::from("cyberint"),
            source_id: "cyberint_alerts".to_string(),
            push_down_hash: "a".repeat(64),
        };
        cache
            .put(key_alerts.clone(), vec![serde_json::json!({"id": "al-1"})])
            .expect("put alerts");

        let invalidator = CacheInvalidator::new(Arc::clone(&cache));
        let evicted = invalidator
            .invalidate_for_write_operation(
                &OrgSlug::new("acme"),
                &prism_core::SensorId::from("cyberint"),
                "cyberint_acknowledge_alert",
            )
            .expect("mapped tool invalidation must succeed");

        assert_eq!(evicted, 1, "mapped tool must evict exactly its source_ids");
        assert!(
            cache.get(&key_alerts).expect("get").is_none(),
            "cyberint_alerts entry must be evicted via the tool mapping"
        );
    }

    /// BC-2.07.004 ("omitting a mapping is a bug"): an UNMAPPED tool name must
    /// NOT fail the (already-succeeded) write — `invalidate_for_write_operation`
    /// falls back to sensor-wide invalidation (a safe superset) so the
    /// write-then-read consistency invariant still holds.
    #[test]
    fn test_invalidate_for_write_operation_unmapped_tool_falls_back_to_sensor_wide() {
        use prism_core::tenant::OrgSlug;

        reset_query_phase_for_test();
        reset_dynamic_registry_for_test();

        let cache = Arc::new(QueryCache::with_defaults());
        // crowdstrike sensor maps (via its write tools) to hosts/alerts/detections.
        let key_hosts = crate::cache_key::CacheKey {
            client_id: "acme".to_string(),
            sensor_id: prism_core::SensorId::from("crowdstrike"),
            source_id: "crowdstrike_hosts".to_string(),
            push_down_hash: "b".repeat(64),
        };
        let key_dets = crate::cache_key::CacheKey {
            client_id: "acme".to_string(),
            sensor_id: prism_core::SensorId::from("crowdstrike"),
            source_id: "crowdstrike_detections".to_string(),
            push_down_hash: "c".repeat(64),
        };
        cache
            .put(key_hosts.clone(), vec![serde_json::json!({"id": "h-1"})])
            .expect("put hosts");
        cache
            .put(key_dets.clone(), vec![serde_json::json!({"id": "d-1"})])
            .expect("put detections");

        let invalidator = CacheInvalidator::new(Arc::clone(&cache));
        let result = invalidator.invalidate_for_write_operation(
            &OrgSlug::new("acme"),
            &prism_core::SensorId::from("crowdstrike"),
            "crowdstrike_tool_with_no_mapping",
        );

        let evicted =
            result.expect("unmapped tool must NOT error — falls back to sensor-wide invalidation");
        assert_eq!(
            evicted, 2,
            "sensor-wide fallback must evict all cached sources for the sensor"
        );
        assert!(
            cache.get(&key_hosts).expect("get hosts").is_none(),
            "fallback must evict crowdstrike_hosts"
        );
        assert!(
            cache.get(&key_dets).expect("get dets").is_none(),
            "fallback must evict crowdstrike_detections"
        );
    }

    /// EC-07-010 / BC-2.07.004: Invalidation with no matching entries is a no-op.
    #[test]
    fn test_ec07010_invalidation_no_matching_entries_is_noop() {
        use prism_core::tenant::OrgSlug;

        // Reset global state for test isolation (handles post-poisoning recovery).
        reset_query_phase_for_test();
        reset_dynamic_registry_for_test();

        let cache = Arc::new(QueryCache::with_defaults());
        let invalidator = CacheInvalidator::new(Arc::clone(&cache));
        let client_id = OrgSlug::new("no-data");

        // Must succeed without error even if no entries exist.
        let result =
            invalidator.invalidate_for_sensor(&client_id, &prism_core::SensorId::from("armis"));
        assert!(
            result.is_ok(),
            "EC-07-010: invalidation with no matching entries must be a no-op"
        );
    }

    // -----------------------------------------------------------------------
    // Test 13 — S-PLUGIN-PREREQ-E Red Gate
    // test_BC_2_16_012_003_write_tool_invalidation_runtime_register
    //
    // AC-9: WriteToolInvalidationMap runtime extensibility (TD-S-PLUGIN-PREREQ-A-003).
    //
    // Exercises three sub-paths:
    //   (a) happy path — register succeeds, entry is visible on next read
    //   (b) duplicate tool_name — second registration rejected (E-PLUGIN-012)
    //   (c) post-boot registration — rejected with WriteToolRegistrationAfterBoot
    //       after mark_query_phase_started(); WARN tracing event captured via
    //       tracing-test subscriber.
    //
    // GREEN since S-PLUGIN-PREREQ-E: register_write_tool() and
    // mark_query_phase_started() are fully implemented above (no todo!()
    // bodies remain). The Red Gate retired when the implementation landed.
    // -----------------------------------------------------------------------

    /// BC-2.16.012 AC-9 / INV-INVALIDATION-EXT-001: WriteToolInvalidationMap is
    /// runtime-extensible via `register_write_tool()`.
    ///
    /// Sub-test (a): Happy path — `register_write_tool(entry)` returns `Ok(())`;
    /// the entry is present in the map on the next read.
    ///
    /// (Historical Red Gate: failed on the pre-implementation `todo!()` stub;
    /// `register_write_tool` has been implemented since S-PLUGIN-PREREQ-E.)
    ///
    /// Story: S-PLUGIN-PREREQ-E AC-9a | BC: BC-2.16.012 | ADR-026 §D7
    #[test]
    fn test_BC_2_16_012_003_write_tool_invalidation_runtime_register_happy_path() {
        // Reset global state for test isolation (F-LP-IMPL-P1-005).
        reset_query_phase_for_test();
        reset_dynamic_registry_for_test();

        let entry = WriteToolInvalidationMap {
            tool_name: "plugin_write_custom_alert".to_string(),
            source_ids: vec!["custom_alerts".to_string()],
            sensor_id: prism_core::SensorId::from("custom_sensor"),
            plugin_name: "test_plugin".to_string(),
        };

        let result = register_write_tool(entry);
        assert!(
            result.is_ok(),
            "BC-2.16.012 AC-9a: register_write_tool happy path must return Ok(()); \
             got: {:?}",
            result.err()
        );
    }

    /// BC-2.16.012 AC-9 / EC-016-012-004: A second `register_write_tool` call
    /// with the same `tool_name` must be rejected (E-PLUGIN-012 / DuplicateWriteToolRegistration).
    ///
    /// (Historical Red Gate: failed on the pre-implementation `todo!()` stub;
    /// `register_write_tool` and the `DuplicateWriteToolRegistration` variant
    /// have been implemented since S-PLUGIN-PREREQ-E. The assertion remains
    /// display-string based — "duplicate" substring — by design.)
    ///
    /// Story: S-PLUGIN-PREREQ-E AC-9b | BC: BC-2.16.012 | EC-016-012-004 | ADR-026 §D7
    #[test]
    fn test_BC_2_16_012_003_write_tool_invalidation_duplicate_rejected() {
        // Reset global state for test isolation (F-LP-IMPL-P1-005).
        reset_query_phase_for_test();
        reset_dynamic_registry_for_test();

        // First registration — must succeed.
        let entry_a = WriteToolInvalidationMap {
            tool_name: "plugin_write_dup_tool".to_string(),
            source_ids: vec!["dup_alerts".to_string()],
            sensor_id: prism_core::SensorId::from("dup_sensor"),
            plugin_name: "dup_plugin_a".to_string(),
        };
        let first = register_write_tool(entry_a);
        assert!(
            first.is_ok(),
            "BC-2.16.012 AC-9b: first registration must succeed; got: {:?}",
            first.err()
        );

        // Second registration with the same tool_name — must be rejected.
        let entry_b = WriteToolInvalidationMap {
            tool_name: "plugin_write_dup_tool".to_string(),
            source_ids: vec!["dup_other".to_string()],
            sensor_id: prism_core::SensorId::from("other_sensor"),
            plugin_name: "dup_plugin_b".to_string(),
        };
        let second = register_write_tool(entry_b);
        assert!(
            second.is_err(),
            "BC-2.16.012 AC-9b: duplicate tool_name must be rejected; \
             second call returned Ok(()) instead"
        );
        let err_str = format!("{:?}", second.unwrap_err());
        assert!(
            err_str.to_lowercase().contains("duplicate"),
            "BC-2.16.012 AC-9b: duplicate error must mention 'duplicate'; got: {err_str}"
        );
    }

    /// BC-2.16.012 AC-9 / EC-016-012-005: `register_write_tool` called AFTER
    /// `mark_query_phase_started()` must return `WriteToolRegistrationAfterBoot`
    /// and emit a WARN tracing event with the correct field schema.
    ///
    /// Invokes `mark_query_phase_started()` via the public API (not direct store),
    /// then calls `register_write_tool()` and asserts the error.
    ///
    /// Also captures the WARN tracing event via `tracing-test` subscriber and
    /// asserts: `event_type = "write_tool_registration_after_boot"`,
    /// `plugin_name = <plugin>`, `tool_name = <tool>`, `error = "E-PLUGIN-020"`.
    ///
    /// (Historical Red Gate: failed on the pre-implementation `todo!()` stubs;
    /// `mark_query_phase_started` and `register_write_tool` have been
    /// implemented since S-PLUGIN-PREREQ-E.)
    ///
    /// Story: S-PLUGIN-PREREQ-E AC-9c/9d | BC: BC-2.16.012 | BC-2.16.002 row 33 | ADR-026 §D7
    #[test]
    #[tracing_test::traced_test]
    fn test_BC_2_16_012_003_write_tool_invalidation_post_boot_rejected_with_warn_event() {
        // Set the query-phase flag via the public API (NOT direct store).
        mark_query_phase_started();

        let entry = WriteToolInvalidationMap {
            tool_name: "post_boot_tool".to_string(),
            source_ids: vec!["some_data".to_string()],
            sensor_id: prism_core::SensorId::from("late_plugin"),
            plugin_name: "late_registrar".to_string(),
        };

        let result = register_write_tool(entry);
        assert!(
            result.is_err(),
            "BC-2.16.012 AC-9c: register_write_tool after mark_query_phase_started() \
             must return Err(WriteToolRegistrationAfterBoot); got Ok(())"
        );

        let err = result.unwrap_err();
        // Must be WriteToolRegistrationAfterBoot variant (E-PLUGIN-020).
        let err_str = format!("{err}");
        assert!(
            err_str.contains("E-PLUGIN-020"),
            "BC-2.16.012 AC-9c: error must cite E-PLUGIN-020; got: {err_str}"
        );

        // Verify that the WARN tracing event was emitted with the required field schema.
        // Per BC-2.16.002 §Postconditions (Canonical Structured Event Catalog) row 33.
        assert!(
            logs_contain("write_tool_registration_after_boot"),
            "BC-2.16.012 AC-9d / BC-2.16.002 row 33: WARN event must carry \
             event_type = \"write_tool_registration_after_boot\""
        );
        assert!(
            logs_contain("E-PLUGIN-020"),
            "BC-2.16.012 AC-9d / BC-2.16.002 row 33: WARN event must carry \
             error = \"E-PLUGIN-020\""
        );
        assert!(
            logs_contain("late_registrar"),
            "BC-2.16.012 AC-9d / BC-2.16.002 row 33: WARN event must carry \
             plugin_name = \"late_registrar\""
        );
        assert!(
            logs_contain("post_boot_tool"),
            "BC-2.16.012 AC-9d / BC-2.16.002 row 33: WARN event must carry \
             tool_name = \"post_boot_tool\""
        );
    }

    // -----------------------------------------------------------------------
    // Test isolation helpers (F-LP-IMPL-P1-005)
    //
    // `QUERY_PHASE_STARTED` is a module-level AtomicBool. Under `cargo nextest`
    // (one process per test) tests are isolated. Under `cargo test --workspace`
    // (shared binary per test file) tests share the same module-level state.
    //
    // The reset hook below resets QUERY_PHASE_STARTED to false so that tests
    // expecting the initial false state are not affected by the
    // `test_BC_2_16_012_003_write_tool_invalidation_post_boot_rejected_with_warn_event`
    // test, which permanently flips the flag via `mark_query_phase_started()`.
    //
    // `reset_query_phase_for_test()` MUST be called at the start of any test that
    // requires QUERY_PHASE_STARTED == false (happy path and duplicate rejection tests).
    //
    // `reset_dynamic_registry_for_test()` similarly clears DYNAMIC_WRITE_TOOLS to
    // prevent cross-test contamination in the happy-path and duplicate-rejection tests.
    // -----------------------------------------------------------------------

    /// Reset the global `QUERY_PHASE_STARTED` flag to `false` for test isolation.
    ///
    /// MUST be called at the start of any test that requires the initial
    /// `QUERY_PHASE_STARTED == false` state (F-LP-IMPL-P1-005).
    #[cfg(test)]
    pub(crate) fn reset_query_phase_for_test() {
        QUERY_PHASE_STARTED.store(false, Ordering::Release);
    }

    /// Reset the `DYNAMIC_WRITE_TOOLS` registry to empty for test isolation.
    ///
    /// MUST be called alongside `reset_query_phase_for_test()` in tests that
    /// register write tools to prevent state leak into subsequent tests
    /// (F-LP-IMPL-P1-005).
    ///
    /// Handles RwLock poisoning gracefully (recovers via `into_inner().clear()`)
    /// so tests that run after `test_BC_2_07_004_rwlock_poisoning_returns_registry_poisoned_error`
    /// are not broken when the poisoning test leaves the lock poisoned.
    #[cfg(test)]
    pub(crate) fn reset_dynamic_registry_for_test() {
        match DYNAMIC_WRITE_TOOLS.write() {
            Ok(mut guard) => guard.clear(),
            Err(poisoned) => poisoned.into_inner().clear(), // recover from poisoning
        }
    }

    // F-LP-IMPL-P1-004: RwLock poisoning test is in a SEPARATE integration test
    // binary: crates/prism-query/tests/invalidation_poison_test.rs
    //
    // The poisoning test MUST run in process isolation (nextest or a separate
    // integration test binary) because it permanently damages the process-global
    // `DYNAMIC_WRITE_TOOLS` RwLock for the duration of the test process.
    // Under cargo test's shared-process mode, the poisoning races with concurrent
    // threads running other tests, causing non-deterministic failures.
    //
    // This architectural choice (separate binary) is equivalent to serial_test
    // serialization but requires no additional dependency — it uses Rust's built-in
    // integration test isolation (each test binary is its own process).

    // -----------------------------------------------------------------------
    // F-LP-IMPL-P1-001: RED GATE — dynamic write tool triggers cache invalidation
    //
    // This test is the RED gate for F-LP-IMPL-P1-001. It registers a write tool
    // via register_write_tool(), then calls CacheInvalidator::invalidate_for_write_tool()
    // with that tool's name, and asserts the corresponding cache entries are invalidated.
    //
    // Pre-fix failure mode: DYNAMIC_WRITE_TOOLS is never read by invalidate_for_write_tool;
    // the lookup in WRITE_TOOL_INVALIDATION_MAP fails to find the dynamically-registered
    // tool and returns PrismError::Internal (missing mapping = bug).
    // -----------------------------------------------------------------------

    /// BC-2.07.004 / BC-2.16.012 AC-9 / F-LP-IMPL-P1-001:
    /// A write tool registered via `register_write_tool` must trigger cache
    /// invalidation when `invalidate_for_write_tool` is called with that tool's name.
    ///
    /// Pre-fix failure mode: `invalidate_for_write_tool` only scans the static
    /// `WRITE_TOOL_INVALIDATION_MAP` and returns `Err(PrismError::Internal)`
    /// for dynamically-registered tools (DYNAMIC_WRITE_TOOLS never read).
    ///
    /// Story: S-PLUGIN-PREREQ-E / F-LP-IMPL-P1-001 | BC-2.07.004 | BC-2.16.012 AC-9
    // NOTE: This test is duplicated in `tests/invalidation_integration_test.rs` for
    // additional process isolation coverage. The inline unit test below runs under
    // nextest (one-process-per-test, no shared-process race) and is the canonical
    // red-gate verification. F-LP-IMPL-P2-003: removed #[ignore] — nextest provides
    // the process isolation that makes this test reliable. `cargo test` shared-process
    // mode is NOT the canonical test runner for this codebase (just iter / just check).
    #[test]
    fn test_BC_2_07_004_dynamic_write_tool_triggers_cache_invalidation() {
        use prism_core::tenant::OrgSlug;

        reset_query_phase_for_test();
        // Intentionally NOT calling reset_dynamic_registry_for_test() here to avoid
        // being vulnerable to concurrent reset calls. Instead, we register and tolerate
        // duplicate-registration errors (tool may already be present from another test
        // in the same process).

        // Register a dynamic write tool (simulates plugin registration at boot step 7.5).
        let dyn_entry = WriteToolInvalidationMap {
            tool_name: "dyn_plugin_write_tool".to_string(),
            source_ids: vec!["dyn_data_source".to_string()],
            sensor_id: prism_core::SensorId::from("dyn_sensor"),
            plugin_name: "dyn_test_plugin".to_string(),
        };
        // Tolerate DuplicateWriteToolRegistration (another thread already registered it)
        // but fail on WriteToolRegistrationAfterBoot or other errors.
        match register_write_tool(dyn_entry) {
            Ok(()) => {}                                                  // Fresh registration.
            Err(SpecEngineError::DuplicateWriteToolRegistration(_)) => {} // Already registered — fine.
            Err(e) => panic!(
                "unexpected error registering dynamic write tool: {e}. \
                 If WriteToolRegistrationAfterBoot, QUERY_PHASE_STARTED was set concurrently — \
                 this is a cargo test shared-process race; use nextest for reliable execution."
            ),
        }

        // Populate the cache with an entry for the dynamic tool's source_id.
        let cache = Arc::new(QueryCache::with_defaults());
        let key = crate::cache_key::CacheKey {
            client_id: "acme".to_string(),
            sensor_id: prism_core::SensorId::from("dyn_sensor"),
            source_id: "dyn_data_source".to_string(),
            push_down_hash: "e".repeat(64),
        };
        let rows = vec![serde_json::json!({"id": "dyn-1"})];
        cache.put(key.clone(), rows).expect("put must succeed");

        // Verify the cache entry exists before invalidation.
        assert!(
            cache.get(&key).expect("get must not fail").is_some(),
            "cache entry must exist before invalidation"
        );

        // Re-register right before invalidation in case a concurrent reset cleared our
        // entry between the initial registration above and this point.
        // Under nextest, this is unnecessary (single process). Under cargo test, it's
        // a defensive guard against concurrent reset races.
        let re_entry = WriteToolInvalidationMap {
            tool_name: "dyn_plugin_write_tool".to_string(),
            source_ids: vec!["dyn_data_source".to_string()],
            sensor_id: prism_core::SensorId::from("dyn_sensor"),
            plugin_name: "dyn_test_plugin".to_string(),
        };
        // Ignore DuplicateWriteToolRegistration (still present) and WriteToolRegistrationAfterBoot
        // (QUERY_PHASE_STARTED was set by another thread — skip the test in that case).
        let re_reg_result = register_write_tool(re_entry);
        if matches!(
            re_reg_result,
            Err(SpecEngineError::WriteToolRegistrationAfterBoot)
        ) {
            // Can't test this under shared-process mode if another thread started the query phase.
            // Under nextest, this code path is never reached (single-process isolation).
            return;
        }
        // At this point, the tool is in the registry (either freshly registered or was already there).

        // Invalidate using the dynamically-registered write tool name.
        let invalidator = CacheInvalidator::new(Arc::clone(&cache));
        let client_id = OrgSlug::new("acme");
        let result = invalidator.invalidate_for_write_tool(&client_id, "dyn_plugin_write_tool");

        assert!(
            result.is_ok(),
            "BC-2.07.004 / F-LP-IMPL-P1-001: invalidate_for_write_tool on dynamic tool \
             must return Ok; got: {:?}",
            result.err()
        );

        // The cache entry must be evicted.
        assert!(
            cache.get(&key).expect("get must not fail").is_none(),
            "BC-2.07.004 / F-LP-IMPL-P1-001: cache entry must be evicted after \
             invalidate_for_write_tool for dynamically-registered write tool"
        );
    }

    // -----------------------------------------------------------------------
    // RG-03 — PLUGIN-MIGRATION-001-B AC-003 (GREEN post-implementation)
    // test_BC_2_16_012_B_003_register_builtin_write_tools_populates_dynamic_registry
    //
    // BC-2.16.012 invariant INV-INVALIDATION-EXT-001:
    // WRITE_TOOL_INVALIDATION_MAP entries are accessible via
    // `register_builtin_write_tools()` at boot, which populates DYNAMIC_WRITE_TOOLS.
    //
    // SITE-3 implemented: `register_builtin_write_tools()` exists and passes all 4
    // sub-assertions (§1 empty-before, §2 count==8-after, §3 sensor-id membership,
    // §4 duplicate-returns-Err). Test went GREEN with PLUGIN-MIGRATION-001-B SITE-3.
    // -----------------------------------------------------------------------

    /// RG-03 / AC-003 (PLUGIN-MIGRATION-001-B) — GREEN.
    /// `register_builtin_write_tools()` populates `DYNAMIC_WRITE_TOOLS`
    /// with all 8 entries from `WRITE_TOOL_INVALIDATION_MAP`.
    ///
    /// Traces to BC-2.16.012 invariant INV-INVALIDATION-EXT-001:
    /// WRITE_TOOL_INVALIDATION_MAP → DYNAMIC_WRITE_TOOLS boot registration path.
    ///
    /// Story: PLUGIN-MIGRATION-001-B AC-003
    #[test]
    fn test_BC_2_16_012_B_003_register_builtin_write_tools_populates_dynamic_registry() {
        // Reset global state for test isolation (F-LP-IMPL-P1-005).
        // Must reset QUERY_PHASE_STARTED to false so register_write_tool() is accepted.
        // Must reset DYNAMIC_WRITE_TOOLS to empty so the count starts at 0.
        reset_query_phase_for_test();
        reset_dynamic_registry_for_test();

        // AC-003 §1: Before the call, DYNAMIC_WRITE_TOOLS count is 0.
        let before_count = dynamic_write_tool_count();
        assert_eq!(
            before_count, 0,
            "RG-03 §1: DYNAMIC_WRITE_TOOLS must be empty before register_builtin_write_tools(); \
             got: {before_count}"
        );

        // AC-003 §2: One call populates DYNAMIC_WRITE_TOOLS with all 8 built-in entries.
        let first_result = register_builtin_write_tools();
        assert!(
            first_result.is_ok(),
            "RG-03 §2: register_builtin_write_tools() must return Ok(()) on first call; \
             got: {:?}",
            first_result.err()
        );

        let after_count = dynamic_write_tool_count();
        assert_eq!(
            after_count, 8,
            "RG-03 §2 (BC-2.16.012 INV-INVALIDATION-EXT-001): \
             after register_builtin_write_tools(), dynamic_write_tool_count() must be 8 \
             (one entry per row in WRITE_TOOL_INVALIDATION_MAP); got: {after_count}"
        );

        // AC-003 §3: Each registered entry's sensor_id is one of the 4 built-in sensors.
        let valid_sensors = ["crowdstrike", "cyberint", "claroty", "armis"];
        let guard = DYNAMIC_WRITE_TOOLS
            .read()
            .expect("DYNAMIC_WRITE_TOOLS must not be poisoned");
        for entry in guard.iter() {
            let sensor_str = entry.sensor_id.as_ref();
            assert!(
                valid_sensors.contains(&sensor_str),
                "RG-03 §3: registered entry sensor_id '{}' must be one of the \
                 4 built-in sensors {:?}",
                sensor_str,
                valid_sensors
            );
        }
        drop(guard);

        // AC-003 §4: A second call returns Err(DuplicateWriteToolRegistration) for
        // the first duplicate tool_name encountered (BC-2.16.012 EC-016-012-004).
        let second_result = register_builtin_write_tools();
        assert!(
            second_result.is_err(),
            "RG-03 §4: second call to register_builtin_write_tools() must return Err \
             (DuplicateWriteToolRegistration per BC-2.16.012 EC-016-012-004); \
             got Ok(())"
        );
        let err_str = format!("{:?}", second_result.unwrap_err());
        assert!(
            err_str.to_lowercase().contains("duplicate"),
            "RG-03 §4: duplicate error must mention 'duplicate'; got: {err_str}"
        );
    }
}
