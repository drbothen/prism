//! `FixtureSet`, `Provenance`, and `OrgId` types (BC-3.4.001 postcondition 1).

use serde_json::Value;

use prism_core::types::SensorType;

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
    /// Sensor type targeted by the fixture.
    pub sensor_type: SensorType,
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
/// A `null` patch value removes the corresponding key from the base.
/// A non-object patch replaces the entire base.
pub fn apply_overrides(_base: Value, _patch: &Value) -> Value {
    todo!()
}
