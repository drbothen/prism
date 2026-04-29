//! AC-7: OrgSlug serializes to JSON and deserializes back to an equal value.

use prism_core::OrgSlug;

/// AC-7: serde round-trip for a valid OrgSlug.
#[test]
fn test_ac7_tenant_id_serde_round_trip() {
    let original = OrgSlug::new("acme").expect("'acme' is a valid OrgSlug");
    let json = serde_json::to_string(&original).expect("serialization must succeed");
    let restored: OrgSlug = serde_json::from_str(&json).expect("deserialization must succeed");
    assert_eq!(original, restored, "round-trip must produce equal OrgSlug");
}

/// AC-7 supplement: JSON form must be a bare string, not a nested object.
#[test]
fn test_ac7_tenant_id_serializes_as_bare_string() {
    let tenant = OrgSlug::new("acme").expect("valid");
    let json = serde_json::to_string(&tenant).expect("must serialize");
    assert_eq!(
        json, "\"acme\"",
        "OrgSlug must serialize as a JSON string, not an object"
    );
}

/// AC-7 supplement: deserializing an invalid string returns Err.
#[test]
fn test_ac7_tenant_id_deserialize_invalid_string_returns_err() {
    let result: Result<OrgSlug, _> = serde_json::from_str("\"../etc\"");
    assert!(
        result.is_err(),
        "deserializing an invalid org slug string must return Err"
    );
}
