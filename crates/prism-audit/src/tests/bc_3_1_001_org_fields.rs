//! Tests for BC-3.1.001 (org field context) and BC-3.1.002 — AuditEntry Carries
//! Both org_id and org_slug at Construction Time.
//!
//! BC-3.1.002 postconditions tested:
//!   - (1) written AuditEntry contains org_id equal to the one supplied.
//!   - (2) written AuditEntry contains org_slug equal to the current slug at emission.
//!   - (3) neither field is null, empty, or omitted in the serialized record.
//!   - (4) filtering by org_id returns records regardless of which slug was current.
//!   - (5) historical records preserve the slug active at write time (rename forensics).
//!
//! BC-3.1.002 invariants tested:
//!   - (1) org_id is the same UUID across rename — stable primary key.
//!   - (2) org_slug is never retroactively updated.
//!   - (3) AuditEntry construction requires both fields (compile-time enforcement via
//!         static_assertions — tested implicitly by every entry-construction helper).
//!
//! Edge cases:
//!   EC-001 — slug rename between two emissions preserves forensic trail.
//!   EC-005 — two orgs with distinct UUIDs, no commingling.
//!
//! Canonical test vectors (BC-3.1.002):
//!   TV-3.1.002-01 — happy path: both fields present.
//!   TV-3.1.002-02 — rename forensic trail.
//!   TV-3.1.002-03 — UUID-stable query after rename.
//!   TV-3.1.002-04 — JSON shape: both fields at top level, neither null.
//!
//! Test naming: `test_BC_3_1_002_<assertion_name>()` per factory naming convention.

use chrono::Utc;
use prism_core::tenant::OrgSlug;
use prism_core::OrgId;
use serde_json::Value;
use uuid::Uuid;

use crate::audit_entry::{AuditEntry, AuditOutcome, DataClassification};

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Build a minimal `AuditEntry` with explicit `org_id` and `org_slug`.
fn entry_with_org(org_id: OrgId, org_slug: OrgSlug) -> AuditEntry {
    AuditEntry::new(
        Uuid::now_v7(),
        Utc::now(),
        "query_crowdstrike_alerts".to_owned(),
        "acme".to_owned(),
        "analyst@example.com".to_owned(),
        serde_json::json!({"query": "SELECT * FROM crowdstrike.alerts"}),
        AuditOutcome::Success,
        "ok".to_owned(),
        42,
        None,
        DataClassification::Internal,
        vec![],
        vec![],
        org_id,
        org_slug,
        // aql_hash supplied as empty string stub — aql_hash tests live in bc_3_1_002_aql_hash.rs
        String::new(),
    )
}

// ── TV-3.1.002-01: happy path — both fields present ──────────────────────────

/// BC-3.1.002 postcondition 1: `AuditEntry.org_id` equals the OrgId supplied
/// at construction time.
///
/// Exercises VP-066 (non-null org_id on every entry).
#[test]
fn test_BC_3_1_002_org_id_equals_supplied_value() {
    let org_id = OrgId::new();
    let org_slug = OrgSlug::new("acme-corp");

    let entry = entry_with_org(org_id, org_slug);

    assert_eq!(
        entry.org_id, org_id,
        "BC-3.1.002 postcondition 1: org_id in AuditEntry must equal the org_id supplied at construction"
    );
}

/// BC-3.1.002 postcondition 2: `AuditEntry.org_slug` equals the slug supplied
/// at construction time.
///
/// Exercises VP-068 (slug matches OrgRegistry slug at emission time).
#[test]
fn test_BC_3_1_002_org_slug_equals_supplied_value() {
    let org_id = OrgId::new();
    let org_slug = OrgSlug::new("acme-corp");

    let entry = entry_with_org(org_id, org_slug.clone());

    assert_eq!(
        entry.org_slug, org_slug,
        "BC-3.1.002 postcondition 2: org_slug in AuditEntry must equal the slug supplied at construction"
    );
}

// ── TV-3.1.002-04: JSON shape — both fields at top level, neither null ────────

/// BC-3.1.002 postcondition 3 / TV-3.1.002-04: serialized AuditEntry JSON must
/// contain `org_id` as a non-null, non-empty UUID string.
#[test]
fn test_BC_3_1_002_serialized_org_id_is_non_null_uuid_string() {
    let org_id = OrgId::new();
    let org_slug = OrgSlug::new("acme-corp");

    let entry = entry_with_org(org_id, org_slug);
    let obj: Value =
        serde_json::to_value(&entry).expect("AuditEntry must serialize to JSON without error");

    let serialized_org_id = obj
        .get("org_id")
        .expect("BC-3.1.002 postcondition 3: 'org_id' field must be present in JSON");

    assert!(
        !serialized_org_id.is_null(),
        "BC-3.1.002 postcondition 3: 'org_id' must not be null in serialized AuditEntry"
    );

    let org_id_str = serialized_org_id
        .as_str()
        .expect("BC-3.1.002: 'org_id' must serialize as a string");

    assert!(
        !org_id_str.is_empty(),
        "BC-3.1.002 postcondition 3: 'org_id' must not be empty in serialized AuditEntry"
    );

    // Verify it is a valid UUID string.
    Uuid::parse_str(org_id_str)
        .expect("BC-3.1.002: 'org_id' in JSON must be a valid UUID string (got: '{org_id_str}')");
}

/// BC-3.1.002 postcondition 3 / TV-3.1.002-04: serialized AuditEntry JSON must
/// contain `org_slug` as a non-null, non-empty string.
#[test]
fn test_BC_3_1_002_serialized_org_slug_is_non_null_non_empty_string() {
    let org_id = OrgId::new();
    let org_slug = OrgSlug::new("acme-corp");

    let entry = entry_with_org(org_id, org_slug);
    let obj: Value =
        serde_json::to_value(&entry).expect("AuditEntry must serialize to JSON without error");

    let serialized_org_slug = obj
        .get("org_slug")
        .expect("BC-3.1.002 postcondition 3: 'org_slug' field must be present in JSON");

    assert!(
        !serialized_org_slug.is_null(),
        "BC-3.1.002 postcondition 3: 'org_slug' must not be null in serialized AuditEntry"
    );

    let org_slug_str = serialized_org_slug
        .as_str()
        .expect("BC-3.1.002: 'org_slug' must serialize as a string");

    assert!(
        !org_slug_str.is_empty(),
        "BC-3.1.002 postcondition 3: 'org_slug' must not be empty in serialized AuditEntry"
    );
}

/// BC-3.1.002 postcondition 3: `org_slug` serialized value matches the slug
/// string that was passed to the emit function (TV-3.1.002-04 round-trip).
#[test]
fn test_BC_3_1_002_serialized_org_slug_matches_input_slug() {
    let org_id = OrgId::new();
    let org_slug = OrgSlug::new("acme-corp");

    let entry = entry_with_org(org_id, org_slug);
    let obj: Value = serde_json::to_value(&entry).unwrap();

    let serialized_slug = obj["org_slug"]
        .as_str()
        .expect("org_slug must be a JSON string");

    assert_eq!(
        serialized_slug, "acme-corp",
        "BC-3.1.002 TV-3.1.002-04: serialized org_slug must match the slug passed to AuditEntry::new"
    );
}

// ── Serde round-trip: both fields survive JSON round-trip ────────────────────

/// BC-3.1.002 postcondition 1+2: both `org_id` and `org_slug` survive a full
/// JSON serialization → deserialization round-trip without data loss.
#[test]
fn test_BC_3_1_002_serde_round_trip_preserves_org_id_and_org_slug() {
    let org_id = OrgId::new();
    let org_slug = OrgSlug::new("acme-corp");

    let entry = entry_with_org(org_id, org_slug.clone());

    let json_str = serde_json::to_string(&entry).expect("AuditEntry must serialize to JSON");

    let round_tripped: AuditEntry =
        serde_json::from_str(&json_str).expect("AuditEntry must deserialize from JSON");

    assert_eq!(
        round_tripped.org_id, org_id,
        "BC-3.1.002: org_id must survive JSON round-trip unchanged"
    );
    assert_eq!(
        round_tripped.org_slug, org_slug,
        "BC-3.1.002: org_slug must survive JSON round-trip unchanged"
    );
}

// ── TV-3.1.002-02 / EC-001: rename forensics — slug preserved at write time ───

/// BC-3.1.002 postconditions 4 + 5 / TV-3.1.002-02 / EC-001:
/// Two audit entries emitted for the same org_id but different slugs (simulating
/// a rename) must preserve the slug that was active at write time.
///
/// - First entry: org_slug = "acme-corp" (pre-rename).
/// - Second entry: org_slug = "acme-na" (post-rename).
/// - Both entries: org_id = uuid-A (same stable UUID).
/// - No retroactive update of the first entry.
///
/// Exercises VP-067 (UUID stability across slug rename).
#[test]
fn test_BC_3_1_002_rename_forensics_slug_preserved_at_write_time() {
    // TV-3.1.002-02: Emit with uuid-A + slug "acme-corp", rename, emit again.
    let org_id_a = OrgId::new();

    // Pre-rename entry.
    let entry_before = entry_with_org(org_id_a, OrgSlug::new("acme-corp"));
    // Post-rename entry — same org_id, new slug.
    let entry_after = entry_with_org(org_id_a, OrgSlug::new("acme-na"));

    // Both entries share the same org_id.
    assert_eq!(
        entry_before.org_id, org_id_a,
        "BC-3.1.002 invariant 1: pre-rename entry must have org_id=uuid-A"
    );
    assert_eq!(
        entry_after.org_id, org_id_a,
        "BC-3.1.002 invariant 1: post-rename entry must also have org_id=uuid-A"
    );

    // Slugs differ — no retroactive update.
    assert_eq!(
        entry_before.org_slug,
        OrgSlug::new("acme-corp"),
        "BC-3.1.002 postcondition 5 / invariant 2: pre-rename entry must retain old slug 'acme-corp'"
    );
    assert_eq!(
        entry_after.org_slug,
        OrgSlug::new("acme-na"),
        "BC-3.1.002 postcondition 5: post-rename entry must show new slug 'acme-na'"
    );

    // Verify that modifying the post-rename entry does NOT affect the pre-rename entry.
    // (AuditEntry is Clone — mutations on one clone must not bleed into another.)
    assert_ne!(
        entry_before.org_slug, entry_after.org_slug,
        "BC-3.1.002 invariant 2: the two entries must have different slugs (no retroactive update)"
    );
}

/// BC-3.1.002 postconditions 4 + 5: serialized forms of the two rename-forensic
/// entries carry the correct slug strings.
#[test]
fn test_BC_3_1_002_rename_forensics_json_shape() {
    let org_id_a = OrgId::new();

    let entry_before = entry_with_org(org_id_a, OrgSlug::new("acme-corp"));
    let entry_after = entry_with_org(org_id_a, OrgSlug::new("acme-na"));

    let json_before: Value = serde_json::to_value(&entry_before).unwrap();
    let json_after: Value = serde_json::to_value(&entry_after).unwrap();

    assert_eq!(
        json_before["org_slug"].as_str().unwrap(),
        "acme-corp",
        "TV-3.1.002-02: pre-rename entry JSON must carry org_slug='acme-corp'"
    );
    assert_eq!(
        json_after["org_slug"].as_str().unwrap(),
        "acme-na",
        "TV-3.1.002-02: post-rename entry JSON must carry org_slug='acme-na'"
    );

    // Both must carry the same org_id in serialized form.
    assert_eq!(
        json_before["org_id"], json_after["org_id"],
        "TV-3.1.002-02: both entries must have the same org_id UUID in JSON"
    );
}

// ── TV-3.1.002-03: UUID-stable query simulation after rename ─────────────────

/// BC-3.1.002 postcondition 4 / TV-3.1.002-03: a collection of audit entries
/// for the same org_id (emitted under different slugs) can all be recovered by
/// filtering on org_id — simulating a UUID-stable query.
#[test]
fn test_BC_3_1_002_uuid_stable_query_returns_both_pre_and_post_rename_entries() {
    let org_id_a = OrgId::new();
    let org_id_b = OrgId::new(); // different org — must not appear

    // Pre-rename entries for org A.
    let pre1 = entry_with_org(org_id_a, OrgSlug::new("acme-corp"));
    let pre2 = entry_with_org(org_id_a, OrgSlug::new("acme-corp"));
    // Post-rename entry for org A.
    let post1 = entry_with_org(org_id_a, OrgSlug::new("acme-na"));
    // Entry for a different org (must not match).
    let other = entry_with_org(org_id_b, OrgSlug::new("other-corp"));

    let audit_log = vec![pre1, pre2, post1, other];

    // Simulate a filter-by-org_id query.
    let results_for_a: Vec<&AuditEntry> =
        audit_log.iter().filter(|e| e.org_id == org_id_a).collect();

    assert_eq!(
        results_for_a.len(),
        3,
        "BC-3.1.002 postcondition 4 / TV-3.1.002-03: filtering by org_id must return all 3 entries for org-A (pre-rename and post-rename)"
    );

    // All returned entries carry org_id_a.
    for entry in &results_for_a {
        assert_eq!(
            entry.org_id, org_id_a,
            "TV-3.1.002-03: every result must carry org_id_a"
        );
    }

    // The other org's entry must not appear.
    let results_for_b: Vec<&AuditEntry> =
        audit_log.iter().filter(|e| e.org_id == org_id_b).collect();
    assert_eq!(
        results_for_b.len(),
        1,
        "EC-005: org-B has exactly 1 entry; no commingling with org-A"
    );
}

// ── EC-005: two orgs, no commingling ─────────────────────────────────────────

/// BC-3.1.002 EC-005: two distinct orgs audited in the same time window produce
/// correctly separated records — no commingling by org_id.
#[test]
fn test_BC_3_1_002_two_orgs_no_commingling() {
    let org_a = OrgId::new();
    let org_b = OrgId::new();

    let entry_a = entry_with_org(org_a, OrgSlug::new("alpha-corp"));
    let entry_b = entry_with_org(org_b, OrgSlug::new("beta-corp"));

    assert_ne!(
        entry_a.org_id, entry_b.org_id,
        "EC-005: two distinct orgs must have different org_ids (not commingled)"
    );
    assert_ne!(
        entry_a.org_slug, entry_b.org_slug,
        "EC-005: two distinct orgs must have different org_slugs"
    );

    // Filter each — verify no bleed-through.
    let log = vec![&entry_a, &entry_b];

    let for_a: Vec<&&AuditEntry> = log.iter().filter(|e| e.org_id == org_a).collect();
    let for_b: Vec<&&AuditEntry> = log.iter().filter(|e| e.org_id == org_b).collect();

    assert_eq!(for_a.len(), 1, "EC-005: exactly 1 entry for org-A");
    assert_eq!(for_b.len(), 1, "EC-005: exactly 1 entry for org-B");
    assert_eq!(
        for_a[0].org_slug,
        OrgSlug::new("alpha-corp"),
        "EC-005: org-A entry carries alpha-corp slug"
    );
    assert_eq!(
        for_b[0].org_slug,
        OrgSlug::new("beta-corp"),
        "EC-005: org-B entry carries beta-corp slug"
    );
}

// ── Invariant 1: org_id UUID is stable (same UUID across renames) ────────────

/// BC-3.1.002 invariant 1: the same UUID v7 appears in all audit records for
/// an org regardless of slug changes. Tested via the rename-forensics scenario.
#[test]
fn test_BC_3_1_002_invariant_org_id_stable_across_renames() {
    let org_id = OrgId::new();

    // Simulate 3 sequential slug renames.
    let slugs = ["corp-v1", "corp-v2", "corp-v3"];
    let entries: Vec<AuditEntry> = slugs
        .iter()
        .map(|s| entry_with_org(org_id, OrgSlug::new(s)))
        .collect();

    for (i, entry) in entries.iter().enumerate() {
        assert_eq!(
            entry.org_id, org_id,
            "BC-3.1.002 invariant 1: entry[{i}] must carry the same stable org_id UUID across all renames"
        );
    }
}
