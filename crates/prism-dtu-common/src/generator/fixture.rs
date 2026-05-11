//! `FixtureSet`, `Provenance`, and `OrgId` types (BC-3.4.001 postcondition 1).

use serde_json::Value;

use prism_core::SensorId;

use super::archetype::Archetype;

/// Tenant organisation identifier — wraps UUID v7 bytes.
///
/// Used as org-namespace input to the XOR-seed formula (BC-3.4.001 invariant 2).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct OrgId(pub [u8; 16]);

impl OrgId {
    /// Return the raw UUID bytes.
    pub fn as_bytes(&self) -> &[u8; 16] {
        &self.0
    }
}

/// Metadata describing the provenance of a `FixtureSet` (BC-3.4.001 postcondition 1).
#[derive(Clone, Debug)]
pub struct Provenance {
    /// Organisation for which this fixture was generated.
    pub org_id: OrgId,
    /// Sensor id targeted by the fixture.
    pub sensor_type: SensorId,
    /// Archetype that shaped the generated data.
    pub archetype: Archetype,
    /// Seed used to initialise the RNG.
    pub seed: u64,
    /// `false` for `SchemaDrift` archetype; `true` for all others (BC-3.4.002).
    pub schema_valid: bool,
}

/// The output of a single generator call (BC-3.4.001 postcondition 1).
#[derive(Clone, Debug)]
pub struct FixtureSet {
    /// Generated records as raw JSON values.
    pub records: Vec<Value>,
    /// Pagination cursors representing page boundaries in the simulated API.
    pub cursors: Vec<String>,
    /// Provenance metadata (excluded from byte-identity comparison per BC-3.4.001 invariant 3).
    pub provenance: Provenance,
}

/// Apply a JSON Merge Patch (RFC 7396) to a base value (BC-3.4.001 postcondition 7 / AC-004).
///
/// Rules:
/// - If `patch` is not an object, it replaces `base` entirely.
/// - If `patch` is an object and `base` is an object, keys are merged recursively:
///   - A `null` patch value removes the corresponding key from `base`.
///   - A non-null patch value replaces or adds the key in `base`.
/// - Result is deterministic: same base + same patch always produces the same output.
pub fn apply_overrides(base: Value, patch: &Value) -> Value {
    match patch {
        // Non-object patch replaces the entire base (RFC 7396 §2).
        Value::Object(patch_map) => {
            let mut result = match base {
                Value::Object(map) => map,
                // If base is not an object, start from an empty object then merge.
                _ => serde_json::Map::new(),
            };
            for (key, patch_val) in patch_map {
                if patch_val.is_null() {
                    // null patch value removes the key (RFC 7396 §2).
                    result.remove(key);
                } else {
                    // Recursively merge nested objects; scalar/array values replace.
                    let existing = result.remove(key).unwrap_or(Value::Null);
                    result.insert(key.clone(), apply_overrides(existing, patch_val));
                }
            }
            Value::Object(result)
        }
        // Non-object patch replaces base entirely.
        _ => patch.clone(),
    }
}
