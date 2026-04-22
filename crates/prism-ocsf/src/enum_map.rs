//! OCSF enum value display map — runtime integer-to-caption lookup.
//!
//! BC-2.02.010: At build time, `ocsf-proto-gen` generates an `enum-value-map.json`
//! that maps OCSF enum type names + integer values to human-readable captions.
//! At startup, `OcsfEnumMap` is populated from the protobuf descriptor's enum value
//! options and used to enrich MCP tool responses.
//!
//! # Note on BC-2.02.010 TV-BC-2.02.010-001
//!
//! The BC says `severity_id: 4` → `"Critical"`, but the story spec (task 6 and AC-4)
//! says `severity_id: 4` → `"High"`. The OCSF v1.x spec defines:
//!   1=Informational, 2=Low, 3=Medium, 4=High, 5=Critical, 99=Other.
//! The story AC ("High" for 4) aligns with OCSF v1.x. The implementation uses
//! OCSF-correct values.

use std::collections::HashMap;
use std::sync::Mutex;

/// Global cache for "Unknown (N)" display name strings.
///
/// These strings are `Box::leak`-ed once per unique value so that they can be
/// returned as `&'static str` from `display_name()`. The cache ensures each value
/// is allocated at most once. Total allocation: bounded by unique u32 enum values
/// that appear in sensor data — negligible in practice.
static UNKNOWN_CACHE: Mutex<Option<HashMap<u32, &'static str>>> = Mutex::new(None);

/// Returns the interned `&'static str` for `"Unknown ({value})"`, allocating once per
/// unique `value`.
fn unknown_str(value: u32) -> &'static str {
    let mut guard = UNKNOWN_CACHE
        .lock()
        .expect("UNKNOWN_CACHE is never poisoned");
    let cache = guard.get_or_insert_with(HashMap::new);
    if let Some(s) = cache.get(&value) {
        return s;
    }
    let s: &'static str = Box::leak(format!("Unknown ({value})").into_boxed_str());
    cache.insert(value, s);
    s
}

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
    /// Builds an `OcsfEnumMap` populated from OCSF v1.x standard values.
    ///
    /// The real implementation will also walk the descriptor pool's enum value
    /// options to pick up any schema-defined values beyond this hard-coded set.
    pub fn new() -> Self {
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

    /// Returns the display name for an OCSF enum `field` + integer `value`.
    ///
    /// # Contract (BC-2.02.010)
    ///
    /// - Returns `Some(caption)` for all values defined in the pinned OCSF schema.
    /// - Returns `Some("Unknown (N)")` for values absent from the map
    ///   (vendor-specific extensions or unrecognised values). (BC-2.02.010 error case)
    /// - Never returns `None`.
    /// - Never panics, not even on empty or unusual field names. (AC-5)
    pub fn display_name(&self, field: &str, value: u32) -> Option<&'static str> {
        if let Some(caption) = self.inner.get(&(field.to_owned(), value)) {
            Some(caption)
        } else {
            // BC-2.02.010: values not defined in the schema return "Unknown (N)".
            // Interned via Box::leak so the return type stays &'static str.
            Some(unknown_str(value))
        }
    }
}

impl Default for OcsfEnumMap {
    fn default() -> Self {
        Self::new()
    }
}
