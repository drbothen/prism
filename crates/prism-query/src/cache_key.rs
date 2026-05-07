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

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

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
    /// Null values are silently dropped (BC-2.07.005 §Canonicalization:
    /// "Null/absent parameters are omitted from the canonical form").
    pub fn insert(&mut self, key: impl Into<String>, value: serde_json::Value) {
        // BC-2.07.005: null values are omitted — do not insert.
        if value.is_null() {
            return;
        }
        self.0.insert(key.into(), value);
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
    /// # Spec note: BC-2.07.005 v4.3 supersedes story Task 5
    ///
    /// The story (S-3.05 Task 5) describes a `clients: &[TenantId]` list parameter
    /// hashed into the key. BC-2.07.005 v4.3 (authoritative) revised the design to
    /// the 4-tuple `(client_id, sensor_id, source_id, push_down_hash)` where the
    /// first three are first-class plain-value fields (not hashed). This natively
    /// handles multi-client scenarios via the partitioned key tuple. The story's
    /// language is from a prior revision (CR-001).
    pub fn derive(
        client_id: impl Into<String>,
        sensor_id: impl Into<String>,
        source_id: impl Into<String>,
        params: &PushDownParams,
    ) -> Self {
        let push_down_hash = Self::derive_push_down_hash(params);
        CacheKey {
            client_id: client_id.into(),
            sensor_id: sensor_id.into(),
            source_id: source_id.into(),
            push_down_hash,
        }
    }

    /// Compute the canonical SHA-256 hex string for the given push-down params.
    ///
    /// This is the pure, testable core of key derivation. It:
    /// 1. Filters out null values from `params` (already done by `insert`, but
    ///    defensive here for externally-constructed `PushDownParams`).
    /// 2. Serializes the remaining entries as a JSON object with sorted keys.
    /// 3. Returns the SHA-256 hex digest of the UTF-8 bytes.
    ///
    /// The BTreeMap inside `PushDownParams` guarantees alphabetical ordering
    /// during serialization — no additional sort is needed.
    pub fn derive_push_down_hash(params: &PushDownParams) -> String {
        // Filter out any nulls that might have been inserted directly into the BTreeMap
        // (defensive — PushDownParams::insert already drops nulls).
        let non_null: BTreeMap<&str, &serde_json::Value> = params
            .0
            .iter()
            .filter(|(_, v)| !v.is_null())
            .map(|(k, v)| (k.as_str(), v))
            .collect();

        // Serialize the non-null BTreeMap as canonical JSON bytes.
        // BTreeMap guarantees alphabetical key ordering in serde_json output.
        // SAFETY: BTreeMap<&str, &serde_json::Value> is always JSON-serializable;
        // serde_json only fails on maps with non-string keys or types with custom
        // serializers that can fail, neither of which applies here.
        // unwrap_used is denied; we use a match-or-default to stay lint-clean.
        let canonical_bytes = match serde_json::to_vec(&non_null) {
            Ok(b) => b,
            // Unreachable for this concrete type, but must be handled.
            Err(_) => b"{}".to_vec(),
        };

        // SHA-256 hash → lowercase hex.
        let mut hasher = Sha256::new();
        hasher.update(&canonical_bytes);
        let hash_bytes = hasher.finalize();
        format!("{:x}", hash_bytes)
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

    /// Access the `push_down_hash` field (read-only accessor).
    ///
    /// Prefer this accessor in production code over direct field access to
    /// maintain encapsulation. The field remains `pub` for test backward
    /// compatibility. See SEC-007.
    pub fn push_down_hash(&self) -> &str {
        &self.push_down_hash
    }

    /// Validate that a `push_down_hash` string satisfies the 64-hex-char invariant.
    ///
    /// Returns `Err` if `hash` is not exactly 64 lowercase hex characters
    /// (BC-2.07.005: "64-character lowercase hex string").
    ///
    /// Used by callers that construct `CacheKey` with an externally-provided hash
    /// (e.g., from deserialization) rather than via `CacheKey::derive`. SEC-007.
    pub fn validate_push_down_hash(hash: &str) -> Result<(), String> {
        if hash.len() != 64 {
            return Err(format!(
                "SEC-007: push_down_hash must be exactly 64 chars; got {}",
                hash.len()
            ));
        }
        if !hash.chars().all(|c| matches!(c, '0'..='9' | 'a'..='f')) {
            return Err(
                "SEC-007: push_down_hash must contain only lowercase hex chars".to_string(),
            );
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// AC-7 / BC-2.07.005: Same push-down params in different insertion order
    /// must produce the same cache key.
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
    #[test]
    fn test_push_down_params_new_is_empty() {
        assert!(PushDownParams::new().is_empty());
    }

    /// GREEN-BY-DESIGN: `CacheKey::prefix()` returns the three plain-value components.
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

    /// CR-001 / BC-2.07.005 v4.3: Two calls with different `client_id` produce
    /// different `CacheKey` values (client isolation — per EC-07-004 spirit).
    ///
    /// Verifies that `client_id` is a first-class key component (not hashed away)
    /// and that the 4-tuple `(client_id, sensor_id, source_id, push_down_hash)`
    /// provides per-client isolation without needing a `clients: &[TenantId]` list.
    #[test]
    fn test_cr001_different_client_id_produces_different_cache_key() {
        let params = PushDownParams::new();
        let key_acme = CacheKey::derive("acme", "crowdstrike", "crowdstrike_detections", &params);
        let key_beta = CacheKey::derive("beta", "crowdstrike", "crowdstrike_detections", &params);

        // push_down_hash is the same (same params), but full key differs by client_id.
        assert_eq!(
            key_acme.push_down_hash, key_beta.push_down_hash,
            "CR-001: push_down_hash must be the same for same params regardless of client_id"
        );
        assert_ne!(
            key_acme, key_beta,
            "CR-001: full CacheKey must differ when client_id differs (client isolation)"
        );
    }
}
