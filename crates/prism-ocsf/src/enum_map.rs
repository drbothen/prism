//! OCSF enum value display map — runtime integer-to-caption lookup.
//!
//! BC-2.02.010: At build time, `ocsf-proto-gen` generates an `enum-value-map.json`
//! that maps OCSF enum type names + integer values to human-readable captions.
//! At startup, `OcsfEnumMap` is populated from the protobuf descriptor's enum value
//! options and used to enrich MCP tool responses.
//!
//! # Stub Status
//!
//! The real implementation populates `OcsfEnumMap` from the descriptor pool enum
//! value options at startup. This stub hard-codes the canonical `severity_id` and
//! `activity_id` values from BC-2.02.010 / the story spec so that tests can be
//! written against known-good data.
//!
//! The stub `display_name()` returns `None` for any key absent from the hard-coded
//! map. The real implementation must query the descriptor pool and fall back to
//! `"Unknown ({value})"` for absent values (BC-2.02.010 error case). Tests for
//! `OcsfEnumMap` that assert the fallback string format will fail until the real
//! implementation lands — this is intentional (Red Gate).
//!
//! # Note on BC-2.02.010 TV-BC-2.02.010-001
//!
//! The BC says `severity_id: 4` → `"Critical"`, but the story spec (task 6 and AC-4)
//! says `severity_id: 4` → `"High"`. These are inconsistent. Per Red Gate rules the
//! tests use the story AC values ("High" for 4, "Critical" for 5) because the ACs are
//! the direct acceptance criteria for this story. The BC is flagged as a concern below.
//!
//! CONCERN: BC-2.02.010 TV-BC-2.02.010-001 says severity_id:4 → "Critical", but
//! S-1.04 AC-4 says severity_id:4 → "High". The OCSF v1.x spec defines:
//!   1=Informational, 2=Low, 3=Medium, 4=High, 5=Critical, 99=Other.
//! The story AC ("High" for 4) aligns with OCSF v1.x. The BC test vector appears
//! to be incorrect. Tests are written to match the story AC (OCSF-correct value).
//! This discrepancy MUST be resolved before implementation.

use std::collections::HashMap;

/// OCSF enum display name map.
///
/// Maps `(field_name, integer_value)` pairs to human-readable display captions.
/// Populated at startup from the compiled OCSF protobuf descriptor enum value options.
///
/// See BC-2.02.010 for the full contract.
pub struct OcsfEnumMap {
    inner: HashMap<(String, u32), &'static str>,
}

impl OcsfEnumMap {
    /// Builds an `OcsfEnumMap` populated from the real OCSF descriptor pool.
    ///
    /// # Stub
    ///
    /// The real implementation queries `OcsfDescriptors::get()` and iterates all
    /// enum value descriptors to populate the map. This stub hard-codes the values
    /// from the story spec / OCSF v1.x standard so tests can be authored.
    pub fn new() -> Self {
        // STUB: In the real implementation this map is built by walking the descriptor
        // pool's enum value options. Until ocsf-proto-gen lands, we seed the map with
        // the values referenced by the story's acceptance criteria and BC test vectors.
        let mut inner: HashMap<(String, u32), &'static str> = HashMap::new();

        // severity_id — OCSF v1.x standard values (AC-4, AC-5, BC-2.02.010)
        inner.insert(("severity_id".to_owned(), 0), "Unknown");
        inner.insert(("severity_id".to_owned(), 1), "Informational");
        inner.insert(("severity_id".to_owned(), 2), "Low");
        inner.insert(("severity_id".to_owned(), 3), "Medium");
        inner.insert(("severity_id".to_owned(), 4), "High");
        inner.insert(("severity_id".to_owned(), 5), "Critical");
        inner.insert(("severity_id".to_owned(), 99), "Other");

        // activity_id — story spec task 6 examples
        inner.insert(("activity_id".to_owned(), 1), "Create");
        inner.insert(("activity_id".to_owned(), 2), "Read");
        inner.insert(("activity_id".to_owned(), 3), "Update");
        inner.insert(("activity_id".to_owned(), 4), "Delete");

        OcsfEnumMap { inner }
    }

    /// Returns the display name for an OCSF enum `field` + integer `value`, or `None`
    /// if the value is not in the map.
    ///
    /// # Contract (BC-2.02.010)
    ///
    /// - Returns `Some(caption)` for all values defined in the pinned OCSF schema.
    /// - Returns `None` for values absent from the map (vendor-specific extensions).
    ///   This is NOT an error — callers handle `None` gracefully. (AC-5)
    /// - The real implementation should return `Some("Unknown ({value})")` for absent
    ///   values per BC-2.02.010 error case. This stub returns `None` instead.
    ///   Tests asserting the "Unknown ({value})" format will fail (Red Gate) until
    ///   the real implementation replaces this stub.
    pub fn display_name(&self, field: &str, value: u32) -> Option<&'static str> {
        self.inner.get(&(field.to_owned(), value)).copied()
    }
}

impl Default for OcsfEnumMap {
    fn default() -> Self {
        Self::new()
    }
}
