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
    _client_id: &str,
    _tool_name: &str,
    _action_params: &Value,
) -> String {
    unimplemented!(
        "S-1.09: compute_action_hash — implement SHA-256 over canonically sorted JSON"
    )
}
