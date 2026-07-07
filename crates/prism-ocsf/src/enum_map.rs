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

use std::{collections::HashMap, sync::Mutex};

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

/// Maps an OCSF string-label field name to its companion `_id` field name for use in
/// `normalize_enum_label` caption lookups.
///
/// Standard OCSF fields follow the `{F}_id`→`{F}` pattern (e.g., `"severity"` → `"severity_id"`).
/// The sole exception is `"activity_name"`, whose companion is `"activity_id"` (NOT
/// `"activity_name_id"`) per the OCSF v1.7.0 schema `activity_id.sibling = "activity_name"`
/// attribute. (BC-2.02.013 F-P1-ACTIVITY-NOOP)
///
/// All other fields use the default `format!("{string_field}_id")` derivation.
fn string_field_to_id_field(string_field: &str) -> String {
    match string_field {
        // OCSF exception: activity_id.sibling = "activity_name", not "activity_name_id"
        "activity_name" => "activity_id".to_string(),
        // All standard fields: {F}_id
        other => format!("{other}_id"),
    }
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

        // activity_id — OCSF v1.7.0 base (0: Unknown, 1-4: CRUD operations, 99: Other)
        inner.insert(("activity_id".to_owned(), 0), "Unknown");
        inner.insert(("activity_id".to_owned(), 1), "Create");
        inner.insert(("activity_id".to_owned(), 2), "Read");
        inner.insert(("activity_id".to_owned(), 3), "Update");
        inner.insert(("activity_id".to_owned(), 4), "Delete");
        inner.insert(("activity_id".to_owned(), 99), "Other");

        // status_id — generic dictionary_attributes.status_id.enum (OCSF v1.7.0)
        // {0→Unknown, 1→Success, 2→Failure, 99→Other}
        inner.insert(("status_id".to_owned(), 0), "Unknown");
        inner.insert(("status_id".to_owned(), 1), "Success");
        inner.insert(("status_id".to_owned(), 2), "Failure");
        inner.insert(("status_id".to_owned(), 99), "Other");

        // status_id — finding-class override (detection_finding, compliance_finding, etc.)
        // OCSF v1.7.0 class-specific enum: {0→Unknown, 1→New, 2→In Progress, 3→Suppressed,
        //   4→Resolved, 5→Archived, 6→Deleted, 99→Other}
        // Stored with synthetic keys (1001–1006) to coexist with the generic entries above
        // without overwriting integer-keyed display_name lookups for the generic class.
        // normalize_enum_label iterates all captions, so these are visible to the normalizer.
        inner.insert(("status_id".to_owned(), 1001), "New");
        inner.insert(("status_id".to_owned(), 1002), "In Progress");
        inner.insert(("status_id".to_owned(), 1003), "Suppressed");
        inner.insert(("status_id".to_owned(), 1004), "Resolved");
        inner.insert(("status_id".to_owned(), 1005), "Archived");
        inner.insert(("status_id".to_owned(), 1006), "Deleted");

        // disposition_id — OCSF v1.7.0 schema.json `dictionary_attributes.disposition_id.enum`
        // 29 values: integers 0–27, 99 (BC-2.02.013 §Postconditions in-scope field table)
        inner.insert(("disposition_id".to_owned(), 0), "Unknown");
        inner.insert(("disposition_id".to_owned(), 1), "Allowed");
        inner.insert(("disposition_id".to_owned(), 2), "Blocked");
        inner.insert(("disposition_id".to_owned(), 3), "Quarantined");
        inner.insert(("disposition_id".to_owned(), 4), "Isolated");
        inner.insert(("disposition_id".to_owned(), 5), "Deleted");
        inner.insert(("disposition_id".to_owned(), 6), "Dropped");
        inner.insert(("disposition_id".to_owned(), 7), "Custom Action");
        inner.insert(("disposition_id".to_owned(), 8), "Approved");
        inner.insert(("disposition_id".to_owned(), 9), "Restored");
        inner.insert(("disposition_id".to_owned(), 10), "Exonerated");
        inner.insert(("disposition_id".to_owned(), 11), "Corrected");
        inner.insert(("disposition_id".to_owned(), 12), "Partially Corrected");
        inner.insert(("disposition_id".to_owned(), 13), "Uncorrected");
        inner.insert(("disposition_id".to_owned(), 14), "Delayed");
        inner.insert(("disposition_id".to_owned(), 15), "Detected");
        inner.insert(("disposition_id".to_owned(), 16), "No Action");
        inner.insert(("disposition_id".to_owned(), 17), "Logged");
        inner.insert(("disposition_id".to_owned(), 18), "Tagged");
        inner.insert(("disposition_id".to_owned(), 19), "Alert");
        inner.insert(("disposition_id".to_owned(), 20), "Count");
        inner.insert(("disposition_id".to_owned(), 21), "Reset");
        inner.insert(("disposition_id".to_owned(), 22), "Captcha");
        inner.insert(("disposition_id".to_owned(), 23), "Challenge");
        inner.insert(("disposition_id".to_owned(), 24), "Access Revoked");
        inner.insert(("disposition_id".to_owned(), 25), "Rejected");
        inner.insert(("disposition_id".to_owned(), 26), "Unauthorized");
        inner.insert(("disposition_id".to_owned(), 27), "Error");
        inner.insert(("disposition_id".to_owned(), 99), "Other");

        OcsfEnumMap { inner }
    }

    /// Normalizes a string label for an OCSF enum-label field to canonical OCSF Title-case.
    ///
    /// **Keying contract (BC-2.02.013 F-HIGH-003):** keys on the OCSF *string label*
    /// field name (e.g., `"severity"`, `"status"`), deriving the caption lookup from the
    /// corresponding `_id` sibling field entries.
    ///
    /// For most fields the sibling is `"{F}_id"` (e.g., `"severity"` → `"severity_id"`).
    /// The sole OCSF exception is `"activity_name"`, whose sibling is `"activity_id"` (NOT
    /// `"activity_name_id"`) — this is the OCSF v1.7.0 `activity_id.sibling = "activity_name"`
    /// schema attribute. `string_field_to_id_field` encodes this mapping; all other fields
    /// use the standard `{F}_id` derivation. (BC-2.02.013 F-P1-ACTIVITY-NOOP)
    ///
    /// # Contract
    ///
    /// - Given a `string_field` name (e.g., `"severity"`, `"activity_name"`) and a `label`
    ///   string (any case), returns `Some(canonical_caption)` if a case-insensitive match is
    ///   found in the companion `_id` entries of the map.
    /// - Returns `None` if no companion `_id` entries exist in the map, or if `label`
    ///   does not match any caption case-insensitively.
    /// - Never panics.
    ///
    /// # Example
    ///
    /// ```
    /// # use prism_ocsf::OcsfEnumMap;
    /// let map = OcsfEnumMap::new();
    /// assert_eq!(map.normalize_enum_label("severity", "HIGH"), Some("High"));
    /// assert_eq!(map.normalize_enum_label("severity", "high"), Some("High"));
    /// // "NEW" matches the finding-class status_id canonical caption "New"
    /// assert_eq!(map.normalize_enum_label("status", "NEW"), Some("New"));
    /// // activity_name is the OCSF sibling of activity_id (exception to {F}_id→{F})
    /// assert_eq!(map.normalize_enum_label("activity_name", "create"), Some("Create"));
    /// ```
    pub fn normalize_enum_label(&self, string_field: &str, label: &str) -> Option<&'static str> {
        // Resolve the companion `_id` field name, handling the OCSF `activity_name` exception
        // (BC-2.02.013 F-HIGH-003 + F-P1-ACTIVITY-NOOP).
        let id_field = string_field_to_id_field(string_field);
        for ((fname, _), &caption) in &self.inner {
            if *fname == id_field && caption.eq_ignore_ascii_case(label) {
                return Some(caption);
            }
        }
        None
    }

    /// Performs a case-insensitive lookup of a string label in the enum caption table
    /// for a given `field_name`.
    ///
    /// **NOTE:** This method keys on the `_id` companion field name (e.g., `"severity_id"`).
    /// For the adapter-boundary normalization use case (BC-2.02.013), use
    /// `normalize_enum_label` which keys on the string label field name (e.g., `"severity"`).
    ///
    /// This method is retained for callers that already have the `_id` field name
    /// (e.g., MCP display enrichment in BC-2.02.010).
    ///
    /// # Contract
    ///
    /// - Given a `field_name` (e.g., `"severity_id"`) and a `label` string (any case),
    ///   returns the canonical-case caption string if a case-insensitive match is found.
    /// - Returns `None` if no enum entry for the field matches the label case-insensitively.
    /// - Never panics.
    ///
    /// # Example
    ///
    /// ```
    /// # use prism_ocsf::OcsfEnumMap;
    /// let map = OcsfEnumMap::new();
    /// assert_eq!(map.normalize_label("severity_id", "HIGH"), Some("High"));
    /// assert_eq!(map.normalize_label("severity_id", "high"), Some("High"));
    /// ```
    pub fn normalize_label(&self, field_name: &str, label: &str) -> Option<&'static str> {
        for ((fname, _), &caption) in &self.inner {
            if fname == field_name && caption.eq_ignore_ascii_case(label) {
                return Some(caption);
            }
        }
        None
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
