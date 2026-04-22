//! `ArmisState` — in-memory state for the Armis Centrix DTU behavioral clone.
//!
//! Maintains:
//! - Immutable device fixture registry pre-loaded from `fixtures/devices.json`
//! - Immutable activity fixture pre-loaded from `fixtures/device-activity.json`
//! - Immutable alert fixture pre-loaded from `fixtures/alerts.json`
//! - Stateful tag store: `device_id → {tag_keys}` — mutated by tag write endpoints
//! - AQL capture log: ordered list of all AQL strings received since last reset
//!
//! No HTTP-layer types (`axum::Json`, `axum::extract::*`) appear here.
//! `ArmisState` is pure Rust — no Axum dependency for its public methods.

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

use prism_dtu_common::FailureMode;

use crate::types::{ActivityRecord, AlertRecord, DeviceRecord};

/// Shared mutable state for the Armis Centrix DTU clone.
///
/// `Arc<ArmisState>` is passed to every axum route handler via `axum::extract::State`.
pub struct ArmisState {
    // --- Immutable fixture registries (loaded once at construction) ---
    /// Device fixture registry, keyed by `device_id`.
    /// Loaded from `fixtures/devices.json`.
    pub device_registry: HashMap<String, DeviceRecord>,

    /// All device records in insertion order (for pagination).
    pub devices_ordered: Vec<DeviceRecord>,

    /// Activity fixture list (for all device_ids).
    /// Loaded from `fixtures/device-activity.json`.
    pub activity_fixture: Vec<ActivityRecord>,

    /// Alert fixture list.
    /// Loaded from `fixtures/alerts.json`.
    pub alert_fixture: Vec<AlertRecord>,

    // --- Mutable state (reset by `reset()`) ---
    /// Stateful tag store: `device_id → set of tag_keys`.
    ///
    /// Populated via `POST /api/v1/devices/{device_id}/tags/`.
    /// Drained by `DELETE /api/v1/devices/{device_id}/tags/{tag_key}`.
    /// Merged into device records at query time by route handlers.
    pub tag_store: Mutex<HashMap<String, HashSet<String>>>,

    /// AQL capture log: ordered list of AQL strings received since last reset.
    ///
    /// Every AQL string from device query requests is appended here verbatim
    /// (no parsing, no validation — per R-DTU-002 mitigation).
    pub aql_log: Mutex<Vec<String>>,

    /// Shared failure mode, read by `FailureLayerShared` on every request.
    ///
    /// Wrapped in `Arc` so `build_router()` can clone it into the tower layer
    /// while `apply_config()` can mutate it after the server starts.
    pub failure_mode: Arc<Mutex<FailureMode>>,
}

impl ArmisState {
    /// Construct a fresh `ArmisState` with pre-loaded fixture registries.
    pub fn new(
        devices: Vec<DeviceRecord>,
        activity: Vec<ActivityRecord>,
        alerts: Vec<AlertRecord>,
    ) -> Self {
        let device_registry: HashMap<String, DeviceRecord> = devices
            .iter()
            .map(|d| (d.device_id.clone(), d.clone()))
            .collect();

        Self {
            device_registry,
            devices_ordered: devices,
            activity_fixture: activity,
            alert_fixture: alerts,
            tag_store: Mutex::new(HashMap::new()),
            aql_log: Mutex::new(Vec::new()),
            failure_mode: Arc::new(Mutex::new(FailureMode::None)),
        }
    }

    /// Reset all mutable state to initial values (called by `BehavioralClone::reset`).
    ///
    /// - Clears the tag store (all device tags removed).
    /// - Clears the AQL log (all captured AQL strings removed).
    /// - Immutable fixture registries are NOT affected.
    pub fn reset(&self) {
        let mut tags = self.tag_store.lock().expect("tag_store poisoned");
        tags.clear();

        let mut aql = self.aql_log.lock().expect("aql_log poisoned");
        aql.clear();
    }

    /// Apply a JSON configuration patch (from `POST /dtu/configure`).
    ///
    /// Recognised keys:
    /// - `"failure_mode"` — one of `"none"`, `"rate_limit"`, `"malformed_response"`,
    ///   `"auth_reject"`, `"internal_error"`, `"network_timeout"`.
    ///   For `"rate_limit"` the following companion keys are also read:
    ///   - `"after_n_requests"` (u32, default 0)
    ///   - `"retry_after_secs"` (u32, default 30)
    ///
    /// Unknown keys are silently ignored per ADR-002 §5.
    pub fn apply_config(&self, config: &serde_json::Value) -> anyhow::Result<()> {
        if let Some(mode_str) = config.get("failure_mode").and_then(|v| v.as_str()) {
            let new_mode = match mode_str {
                "none" => FailureMode::None,
                "rate_limit" => {
                    let after_n = config
                        .get("after_n_requests")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0) as u32;
                    let retry_after = config
                        .get("retry_after_secs")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(30) as u32;
                    FailureMode::RateLimit {
                        after_n_requests: after_n,
                        retry_after_secs: retry_after,
                    }
                }
                "malformed_response" => FailureMode::MalformedResponse,
                "auth_reject" => FailureMode::AuthReject,
                "internal_error" => {
                    let at_n = config
                        .get("at_request_n")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(1) as u32;
                    FailureMode::InternalError { at_request_n: at_n }
                }
                "network_timeout" => {
                    let after_ms = config
                        .get("after_ms")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(5000);
                    FailureMode::NetworkTimeout { after_ms }
                }
                other => {
                    anyhow::bail!("unknown failure_mode: {other}");
                }
            };
            let mut guard = self
                .failure_mode
                .lock()
                .expect("ArmisState: failure_mode lock poisoned");
            *guard = new_mode;
        }
        Ok(())
    }

    /// Append an AQL string to the capture log.
    ///
    /// Called by device query route handlers. Stores verbatim — no parsing.
    pub fn capture_aql(&self, aql: &str) {
        let mut log = self.aql_log.lock().expect("aql_log poisoned");
        log.push(aql.to_owned());
    }

    /// Return all AQL strings received since last reset (for `GET /dtu/aql-log`).
    pub fn aql_log(&self) -> Vec<String> {
        let log = self.aql_log.lock().expect("aql_log poisoned");
        log.clone()
    }

    /// Add a tag to a device's tag set. Returns `true` if newly added (idempotent on re-add).
    pub fn add_tag(&self, device_id: &str, tag_key: &str) -> bool {
        let mut tags = self.tag_store.lock().expect("tag_store poisoned");
        let entry = tags.entry(device_id.to_owned()).or_default();
        entry.insert(tag_key.to_owned())
    }

    /// Remove a tag from a device's tag set. Returns `true` if tag was present and removed.
    pub fn remove_tag(&self, device_id: &str, tag_key: &str) -> bool {
        let mut tags = self.tag_store.lock().expect("tag_store poisoned");
        if let Some(entry) = tags.get_mut(device_id) {
            entry.remove(tag_key)
        } else {
            false
        }
    }

    /// Return the merged tag set for a device (fixture tags + tag_store tags).
    pub fn tags_for(&self, device_id: &str, fixture_tags: &[String]) -> Vec<String> {
        let tags = self.tag_store.lock().expect("tag_store poisoned");
        let mut merged: HashSet<String> = fixture_tags.iter().cloned().collect();
        if let Some(store_tags) = tags.get(device_id) {
            merged.extend(store_tags.iter().cloned());
        }
        let mut result: Vec<String> = merged.into_iter().collect();
        result.sort(); // deterministic output order
        result
    }
}
