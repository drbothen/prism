// S-1.09: Content Hash — SHA-256 of Canonically Serialized Action Params — STUB (Red Gate)
//
// All function bodies are `unimplemented!()`. The implementer must fill them
// in to make the test suite green.
//
// Story:  S-1.09 — prism-security: Confirmation Tokens (P1)
// BC:     BC-2.04.012 — Token Content Hash Verification Prevents Action Tampering
// VP:     VP-009 — Kani proof: modified action params produce different hash → rejection
//
// Architecture compliance rules:
//   - Hash algorithm: SHA-256 (BC-2.04.012).
//   - Input: `client_id`, tool name, and all action-specific parameters.
//   - Canonicalization: MUST sort JSON object keys before hashing to ensure
//     deterministic output regardless of insertion order (BC-2.04.012 invariant).
//   - The serialization format is canonical JSON (serde_json with sorted keys
//     via BTreeMap).

use std::collections::BTreeMap;

use sha2::{Digest, Sha256};
use serde_json::Value;

/// Compute the SHA-256 content hash of the given action parameters.
///
/// # Parameters
/// - `client_id`: the client for whom the action is being performed.
/// - `tool_name`: the name of the write tool being confirmed.
/// - `action_params`: the tool parameters as a `serde_json::Value`.
///
/// # Returns
/// A lowercase hex-encoded SHA-256 digest (64 characters).
///
/// # Canonical form
/// The function builds a JSON object `{"client_id": ..., "tool": ..., "params": ...}`,
/// serializes it with **sorted keys** (uses `BTreeMap` to guarantee order), and
/// computes SHA-256 over the resulting UTF-8 bytes.
///
/// This ensures that key ordering in `action_params` does not affect the hash —
/// `{a: 1, b: 2}` and `{b: 2, a: 1}` produce the same digest (BC-2.04.012 edge case
/// EC-04-025 and canonical test vector "key order variation").
pub fn compute_action_hash(
    client_id: &str,
    tool_name: &str,
    action_params: &Value,
) -> String {
    // Build a canonical envelope with sorted keys using BTreeMap.
    // Normalize the action_params recursively to sort all nested object keys.
    let normalized_params = normalize_value(action_params);

    let mut envelope: BTreeMap<&str, Value> = BTreeMap::new();
    envelope.insert("client_id", Value::String(client_id.to_string()));
    envelope.insert("params", normalized_params);
    envelope.insert("tool", Value::String(tool_name.to_string()));

    // Serialize to canonical JSON (BTreeMap guarantees sorted keys).
    let canonical = serde_json::to_string(&envelope)
        .expect("BTreeMap<&str, Value> serialization must not fail");

    // Compute SHA-256 and encode as lowercase hex.
    let digest = Sha256::digest(canonical.as_bytes());
    format!("{digest:x}")
}

/// Recursively normalize a JSON value so all object keys are sorted.
///
/// This ensures canonical representation regardless of insertion order.
fn normalize_value(value: &Value) -> Value {
    match value {
        Value::Object(map) => {
            let sorted: BTreeMap<_, _> = map
                .iter()
                .map(|(k, v)| (k.clone(), normalize_value(v)))
                .collect();
            Value::Object(sorted.into_iter().collect())
        }
        Value::Array(arr) => {
            Value::Array(arr.iter().map(normalize_value).collect())
        }
        other => other.clone(),
    }
}
