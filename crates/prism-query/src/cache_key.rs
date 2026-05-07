//! `cache_key` — SHA-256 cache key derivation for the sensor-fetch response cache.
//!
//! Implements BC-2.07.005. The full cache key is a 4-tuple
//! `(client_id, sensor_id, source_id, push_down_hash)` where:
//! - The first three components are stored as plain `String` values, enabling
//!   O(n) prefix-scan invalidation by `(client_id, sensor_id, source_id)`.
//! - `push_down_hash` is the SHA-256 hex of the **canonicalized** sensor-native
//!   push-down filter parameters (BC-2.07.005 §Hash Input).
//!
//! Canonicalization rules (BC-2.07.005 §Canonicalization):
//! - Parameters are sorted alphabetically by key name.
//! - Null/absent parameters are omitted from the canonical form.
//! - String values are compared case-sensitively.
//! - The canonical form is a JSON object with sorted keys, serialized to UTF-8
//!   bytes, then SHA-256 hashed.
//! - `push_down_hash` is a 64-character lowercase hex string.
//!
//! **Excluded from hash:** original PrismQL query string, `force_refresh` flag,
//! PrismQL post-filters, `limit` from the `query` tool (BC-2.07.005).
//!
//! # VP References
//! - VP-025 — Cache Key Derivation: Deterministic (kani proof in proofs/vp025_cache_key.rs)
//!
//! # BC References
//! - BC-2.07.005 — Cache Key Derivation from Push-Down Parameters
//!
//! Story: S-3.05

// S-3.05 stub phase — dead_code and unused vars/imports suppressed pending implementation.
#![allow(dead_code, unused_variables, unused_imports)]

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// Sensor identifier (e.g., `"crowdstrike"`, `"armis"`).
///
/// First-class key component enabling prefix-scan invalidation.
pub type SensorId = String;

/// Data source identifier within a sensor (e.g., `"crowdstrike_detections"`).
///
/// First-class key component enabling prefix-scan invalidation.
pub type SourceId = String;

// ---------------------------------------------------------------------------
// PushDownParams
// ---------------------------------------------------------------------------

/// Sensor-native push-down filter parameters — the output of BC-2.11.007
/// query planning, translated to sensor-specific API syntax.
///
/// Stored as a sorted `BTreeMap<String, serde_json::Value>` to allow
/// deterministic serialization. Null/absent values are omitted before hashing.
///
/// This is the input to `CacheKey::derive_push_down_hash`.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct PushDownParams(pub BTreeMap<String, serde_json::Value>);

impl PushDownParams {
    /// Create an empty parameter set.
    ///
    /// GREEN-BY-DESIGN: constructor returns `Self(BTreeMap::new())`.
    /// Zero branching, no I/O, no helpers, 1 line.
    pub fn new() -> Self {
        Self(BTreeMap::new())
    }

    /// Insert a push-down parameter.
    ///
    /// Body: non-trivial (calls BTreeMap::insert, conditional on nulls).
    pub fn insert(&mut self, key: impl Into<String>, value: serde_json::Value) {
        todo!()
    }

    /// Returns true if the parameter set is empty.
    ///
    /// GREEN-BY-DESIGN: pure delegation to `BTreeMap::is_empty()`.
    /// Zero branching, no I/O, no helpers, 1 line.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

// ---------------------------------------------------------------------------
// CacheKey
// ---------------------------------------------------------------------------

/// Full cache key for the sensor-fetch response cache.
///
/// Stores the 4-tuple `(client_id, sensor_id, source_id, push_down_hash)` per
/// BC-2.07.005. The first three components are searchable plain strings
/// enabling prefix-scan invalidation. The `push_down_hash` is a 64-character
/// lowercase SHA-256 hex string.
///
/// Implements `Eq` and `Hash` so it can be used as a key in `HashMap` and
/// `moka::sync::Cache`.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CacheKey {
    /// Tenant / client scope. First-class key component (not hashed).
    pub client_id: String,
    /// Sensor identifier (e.g. `"crowdstrike"`). First-class key component.
    pub sensor_id: SensorId,
    /// Data source identifier (e.g. `"crowdstrike_detections"`). First-class.
    pub source_id: SourceId,
    /// SHA-256 hex of the canonicalized sensor-native push-down filter params.
    /// 64 lowercase hex characters. (BC-2.07.005)
    pub push_down_hash: String,
}

impl CacheKey {
    /// Derive a `CacheKey` for a given `(client_id, sensor_id, source_id)`
    /// tuple and set of push-down parameters.
    ///
    /// The `push_down_hash` is computed as:
    /// `SHA-256(canonical_json_bytes(params))` where `canonical_json_bytes`
    /// serializes `params` as a JSON object with alphabetically sorted keys,
    /// omitting null/absent values.
    ///
    /// Canonicalization ensures that two parameter sets with identical key-value
    /// pairs but different insertion orders produce the same hash (BC-2.07.005
    /// §Canonicalization §Invariants).
    ///
    /// Body: non-trivial — SHA-256 hash computation + JSON serialization.
    pub fn derive(
        client_id: impl Into<String>,
        sensor_id: impl Into<String>,
        source_id: impl Into<String>,
        params: &PushDownParams,
    ) -> Self {
        todo!()
    }

    /// Compute the canonical SHA-256 hex string for the given push-down params.
    ///
    /// This is the pure, testable core of key derivation. It:
    /// 1. Filters out null values from `params`.
    /// 2. Serializes the remaining entries as a JSON object with sorted keys.
    /// 3. Returns the SHA-256 hex digest of the UTF-8 bytes.
    ///
    /// Body: non-trivial — involves SHA-256 hash computation (I/O-adjacent,
    /// uses `sha2` crate).
    pub fn derive_push_down_hash(params: &PushDownParams) -> String {
        todo!()
    }

    /// Return the prefix 3-tuple for prefix-scan invalidation.
    ///
    /// Used by `CacheInvalidator` to efficiently evict all `push_down_hash`
    /// variants for a `(client_id, sensor_id, source_id)` prefix.
    ///
    /// GREEN-BY-DESIGN: returns a tuple of cloned string references.
    /// Zero branching, no I/O, no helpers, 1 line.
    pub fn prefix(&self) -> (&str, &str, &str) {
        (&self.client_id, &self.sensor_id, &self.source_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// AC-7 / BC-2.07.005: Same push-down params in different insertion order
    /// must produce the same cache key.
    ///
    /// RED by design — `derive_push_down_hash` is `todo!()`.
    #[test]
    fn test_ac7_same_params_different_order_produces_same_key() {
        let mut params_a = PushDownParams::new();
        params_a.insert("z_filter", serde_json::json!("value1"));
        params_a.insert("a_filter", serde_json::json!("value2"));

        let mut params_b = PushDownParams::new();
        params_b.insert("a_filter", serde_json::json!("value2"));
        params_b.insert("z_filter", serde_json::json!("value1"));

        let key_a = CacheKey::derive("acme", "crowdstrike", "crowdstrike_detections", &params_a);
        let key_b = CacheKey::derive("acme", "crowdstrike", "crowdstrike_detections", &params_b);

        assert_eq!(
            key_a.push_down_hash, key_b.push_down_hash,
            "AC-7: sorted canonicalization must make order-independent keys identical"
        );
    }

    /// EC-07-042: Absent optional param vs explicit null must produce same hash.
    ///
    /// RED by design — `derive_push_down_hash` is `todo!()`.
    #[test]
    fn test_ec07042_absent_vs_null_params_same_hash() {
        let mut params_with_null = PushDownParams::new();
        params_with_null.insert("optional_filter", serde_json::Value::Null);

        let params_without = PushDownParams::new();

        let hash_with = CacheKey::derive_push_down_hash(&params_with_null);
        let hash_without = CacheKey::derive_push_down_hash(&params_without);

        assert_eq!(
            hash_with, hash_without,
            "EC-07-042: absent and explicit-null params must produce identical hash"
        );
    }

    /// BC-2.07.005 §Invariants: identical inputs always produce the same hash.
    ///
    /// RED by design — `derive_push_down_hash` is `todo!()`.
    #[test]
    fn test_identical_params_produce_identical_hash() {
        let mut params = PushDownParams::new();
        params.insert("severity", serde_json::json!("High"));
        params.insert("status", serde_json::json!("open"));

        let hash1 = CacheKey::derive_push_down_hash(&params);
        let hash2 = CacheKey::derive_push_down_hash(&params);

        assert_eq!(
            hash1, hash2,
            "determinism: same input must always yield same hash"
        );
    }

    /// BC-2.07.005: Different push-down params must produce different hashes.
    ///
    /// RED by design — `derive_push_down_hash` is `todo!()`.
    #[test]
    fn test_different_params_produce_different_hashes() {
        let mut params_a = PushDownParams::new();
        params_a.insert("status", serde_json::json!("open"));

        let mut params_b = PushDownParams::new();
        params_b.insert("status", serde_json::json!("closed"));

        let hash_a = CacheKey::derive_push_down_hash(&params_a);
        let hash_b = CacheKey::derive_push_down_hash(&params_b);

        assert_ne!(
            hash_a, hash_b,
            "different inputs must yield different hashes"
        );
    }

    /// GREEN-BY-DESIGN: `PushDownParams::new()` is empty.
    /// Zero branching, no I/O, no helpers, 1 line.
    #[test]
    fn test_push_down_params_new_is_empty() {
        assert!(PushDownParams::new().is_empty());
    }

    /// GREEN-BY-DESIGN: `CacheKey::prefix()` returns the three plain-value components.
    /// Pure tuple construction, zero branching, no I/O, 1 line.
    #[test]
    fn test_cache_key_prefix_returns_three_tuple() {
        let key = CacheKey {
            client_id: "acme".to_string(),
            sensor_id: "crowdstrike".to_string(),
            source_id: "crowdstrike_detections".to_string(),
            push_down_hash: "a".repeat(64),
        };
        assert_eq!(
            key.prefix(),
            ("acme", "crowdstrike", "crowdstrike_detections")
        );
    }
}
