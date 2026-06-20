//! Red Gate tests for S-DEMO-ENRICHMENT-PIVOT-003 — Cyberint IOC stamping.
//!
//! Covers:
//!   Test 1: `test_BC_2_06_019_cyberint_alert_struct_has_real_ioc_fields`
//!   Test 2: `test_BC_2_06_019_cyberint_ioc_struct_dual_alias_deserializes_both_key_forms`
//!   Test 3: `test_BC_2_06_019_cyberint_fixture_generator_stamps_scenario_iocs`
//!   Test 4: `test_BC_2_06_019_cyberint_alerts_real_schema_ioc_filter_no_synthetic`
//!   Test 7: `test_BC_2_06_019_cyberint_alert_toml_spec_has_ioc_columns`
//!   Test 10: `test_BC_2_06_019_ioc_hashes_false_withholds_cyberint_alert_with_matching_hash`
//!
//! Story: S-DEMO-ENRICHMENT-PIVOT-003
//! Traces to:
//!   BC-2.06.019 v1.10 PC-4 — Cyberint alerts carry real IOC fields; StageMask filter on
//!     real-schema accessors (ioc.value / iocs[].value / alert_data.ip / alert_data.domain)
//!   BC-2.06.019 v1.10 §Interim State — _ioc_value synthetic filter MUST NOT coexist with
//!     real-schema filter (removed atomically)
//!   BC-2.06.020 PC-1 — Scenario generator stamps IOC catalog values on alert records
//!
//! FAIL modes (Red Gate):
//!   Tests 1, 2: These tests verify struct shape/serde already present in the stub.
//!     They are STRUCTURAL tests; if the stub's type definitions are ever rolled back,
//!     these FAIL (compile error or assertion failure).
//!   Test 3: `generate_with_scenario_iocs()` contains `todo!()` → panics.
//!   Test 4: routes/alerts.rs scenario path contains `todo!()` → server panics on request.
//!   Test 7: cyberint.sensor.toml does NOT yet contain ioc/iocs[]/alert_data columns →
//!     assertion fails.
//!   Test 10: routes/alerts.rs scenario path contains `todo!()` → server panics on request.
//!
//! Run:
//!   cargo test -p prism-dtu-cyberint --features dtu,fixture-gen \
//!       --test bc_2_06_019_ioc_stamping -- --nocapture

#![cfg(all(feature = "dtu", feature = "fixture-gen"))]
#![allow(clippy::unwrap_used, clippy::expect_used, non_snake_case)]

use std::sync::Arc;

use prism_dtu_common::{
    build_default_incident_timeline, build_scenario_entity_catalog, Archetype, BehavioralClone,
    GenOpts, OrgId,
};
use prism_dtu_cyberint::{
    generator::generate_with_scenario_iocs,
    types::{Alert, Ioc},
    CyberintClone,
};
use serde_json::json;

// ---------------------------------------------------------------------------
// Test fixtures
// ---------------------------------------------------------------------------

/// Org ID with well-known first 4 bytes → org_slug = "deadbeef".
fn deadbeef_org() -> OrgId {
    OrgId([
        0xde, 0xad, 0xbe, 0xef, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00,
    ])
}

fn access_token_cookie(token: &str) -> String {
    format!("access_token={token}")
}

fn default_opts(seed: u64) -> GenOpts {
    GenOpts {
        seed,
        ..Default::default()
    }
}

// ---------------------------------------------------------------------------
// Test 1 — Cyberint Alert struct has real IOC fields
// ---------------------------------------------------------------------------

/// Test 1 — BC-2.06.019 v1.10 PC-4: Cyberint `Alert` type carries real IOC fields.
///
/// Asserts:
/// - `Alert` has an `ioc: Option<Ioc>` field
/// - `Alert` has an `iocs: Vec<Ioc>` field
/// - `Alert` has an `alert_data: Option<AlertData>` field
/// - `Ioc` has an `ioc_type: String` field (primary wire name "type", alias "ioc_type")
/// - `Ioc` has a `value: String` field (alias "ioc_value")
///
/// FAIL mode: if the stub type definitions are removed/rolled back, this fails to compile.
/// STRUCTURAL test: already GREEN in stub state (type defs present). Included so the
/// behavioral contract is encoded in the test suite and regression protection is established.
///
/// BC-2.06.019 v1.10 PC-4 (Per-Sensor IOC-Surface Matrix — Cyberint: YES).
/// Red Gate test plan #1 (S-DEMO-ENRICHMENT-PIVOT-003).
#[test]
fn test_BC_2_06_019_cyberint_alert_struct_has_real_ioc_fields() {
    // Construct Ioc instances via deserialization (Ioc is #[non_exhaustive] — no struct literal).
    let ioc_primary: Ioc = serde_json::from_value(json!({
        "type": "domain",
        "value": "malicious.example.com"
    }))
    .expect("Ioc must deserialize from primary 'type'/'value' keys");
    let ioc_from_alias: Ioc = serde_json::from_value(json!({
        "ioc_type": "ip",
        "ioc_value": "1.2.3.4"
    }))
    .expect("Ioc must deserialize from ioc_type/ioc_value keys");

    // Alert deserialization with both ioc (singular) and iocs (plural) fields.
    let raw_alert = json!({
        "alert_id": "test-alert-001",
        "title": "Test Alert",
        "severity": "high",
        "status": "open",
        "created_at": "2026-01-01T00:00:00Z",
        "source": "cyberint",
        "source_category": "external",
        "category": "Phishing",
        "type": "phishing",
        "affected_assets": [],
        "confidence": 90,
        "description": "Test alert for struct shape verification",
        "ioc": { "type": "domain", "value": "evil.example.com" },
        "iocs": [
            { "type": "hash_sha256", "value": "abc123" },
            { "ioc_type": "ip", "ioc_value": "5.6.7.8" }
        ],
        "alert_data": {
            "ip": "10.0.0.1",
            "domain": "internal.example.com",
            "url": null
        }
    });

    let alert: Alert =
        serde_json::from_value(raw_alert).expect("Alert must deserialize from full JSON shape");

    // Assert ioc (singular) field present and correct.
    let ioc = alert.ioc.as_ref().expect("alert.ioc must be Some");
    assert_eq!(
        ioc.value, "evil.example.com",
        "alert.ioc.value must be 'evil.example.com'"
    );

    // Assert iocs (plural) field present and non-empty.
    assert_eq!(
        alert.iocs.len(),
        2,
        "alert.iocs must have 2 entries; got {}",
        alert.iocs.len()
    );
    assert_eq!(
        alert.iocs[0].ioc_type, "hash_sha256",
        "alert.iocs[0].ioc_type must be 'hash_sha256'"
    );
    assert_eq!(
        alert.iocs[1].value, "5.6.7.8",
        "alert.iocs[1].value (via ioc_value alias) must be '5.6.7.8'"
    );

    // Assert alert_data field present and correct.
    let alert_data = alert
        .alert_data
        .as_ref()
        .expect("alert.alert_data must be Some");
    assert_eq!(
        alert_data.ip.as_deref(),
        Some("10.0.0.1"),
        "alert.alert_data.ip must be '10.0.0.1'"
    );
    assert_eq!(
        alert_data.domain.as_deref(),
        Some("internal.example.com"),
        "alert.alert_data.domain must be 'internal.example.com'"
    );

    // Cross-check: ioc_primary and ioc_from_alias are usable Ioc values.
    assert_eq!(ioc_primary.ioc_type, "domain");
    assert_eq!(ioc_from_alias.value, "1.2.3.4");
}

// ---------------------------------------------------------------------------
// Test 2 — Cyberint Ioc struct dual-alias deserializes both key forms
// ---------------------------------------------------------------------------

/// Test 2 — BC-2.06.019 v1.10 §Cyberint INCONCLUSIVE inner-key: `Ioc` struct must
/// deserialize BOTH `{"type","value"}` (primary) AND `{"ioc_type","ioc_value"}` (alias)
/// JSON forms to the same `Ioc` value.
///
/// The dual-alias serde annotations:
///   `#[serde(rename = "type", alias = "ioc_type")]` on `ioc_type` field
///   `#[serde(alias = "ioc_value")]` on `value` field
///
/// This test uses exact canonical test vectors from BC-2.06.019 v1.10 §Cyberint IOC section.
///
/// FAIL mode: if serde annotations are removed/changed, deserialization fails or returns
///   wrong field values.
/// STRUCTURAL test: already GREEN in stub state. Included for regression protection.
///
/// BC-2.06.019 v1.10 §Cyberint INCONCLUSIVE inner-key.
/// Red Gate test plan #2 (S-DEMO-ENRICHMENT-PIVOT-003).
#[test]
fn test_BC_2_06_019_cyberint_ioc_struct_dual_alias_deserializes_both_key_forms() {
    // Primary wire form: {"type": ..., "value": ...}
    let primary_form = json!({
        "type": "hash_sha256",
        "value": "deadbeefdeadbeefdeadbeefdeadbeef"
    });

    // Alias wire form: {"ioc_type": ..., "ioc_value": ...}
    let alias_form = json!({
        "ioc_type": "hash_sha256",
        "ioc_value": "deadbeefdeadbeefdeadbeefdeadbeef"
    });

    // Mixed form: {"type": ..., "ioc_value": ...}
    let mixed_form = json!({
        "type": "domain",
        "ioc_value": "evil.example.com"
    });

    let ioc_primary: Ioc = serde_json::from_value(primary_form)
        .expect("Ioc must deserialize from primary {'type','value'} form");
    let ioc_alias: Ioc = serde_json::from_value(alias_form)
        .expect("Ioc must deserialize from alias {'ioc_type','ioc_value'} form");
    let ioc_mixed: Ioc = serde_json::from_value(mixed_form)
        .expect("Ioc must deserialize from mixed {'type','ioc_value'} form");

    // Both forms must produce identical field values.
    assert_eq!(
        ioc_primary.ioc_type, ioc_alias.ioc_type,
        "ioc_type must be identical for primary and alias forms; \
         primary={:?} alias={:?}",
        ioc_primary.ioc_type, ioc_alias.ioc_type
    );
    assert_eq!(
        ioc_primary.value, ioc_alias.value,
        "value must be identical for primary and alias forms; \
         primary={:?} alias={:?}",
        ioc_primary.value, ioc_alias.value
    );

    // Cross-check exact values (canonical test vector from BC-2.06.019 v1.10).
    assert_eq!(ioc_primary.ioc_type, "hash_sha256");
    assert_eq!(ioc_primary.value, "deadbeefdeadbeefdeadbeefdeadbeef");

    // Mixed form.
    assert_eq!(ioc_mixed.ioc_type, "domain");
    assert_eq!(ioc_mixed.value, "evil.example.com");

    // Serialize primary form → JSON should use primary wire keys ("type", "value").
    let serialized = serde_json::to_value(&ioc_primary).expect("Ioc must serialize to JSON");
    assert!(
        serialized.get("type").is_some(),
        "Serialized Ioc must have 'type' key (rename annotation); got: {serialized}"
    );
    assert!(
        serialized.get("ioc_type").is_none(),
        "Serialized Ioc must NOT have 'ioc_type' key (only alias, not rename target); got: {serialized}"
    );
}

// ---------------------------------------------------------------------------
// Test 3 — Cyberint fixture generator stamps scenario IOCs
// ---------------------------------------------------------------------------

/// Test 3 — BC-2.06.020 PC-1 + BC-2.06.019 v1.10 PC-4: `generate_with_scenario_iocs()`
/// stamps scenario IOC catalog values onto `CompromisedEndpoint` alert records.
///
/// After stamping:
/// - At least one alert record in the FixtureSet has `iocs[0].value` matching a
///   value from `catalog_ioc_hashes` (or `catalog_ioc_ips` or `catalog_ioc_domains`).
///
/// FAIL mode: `generate_with_scenario_iocs()` contains `todo!()` → panics with
///   "not yet implemented: AC-002 S-DEMO-ENRICHMENT-PIVOT-003: stamp scenario IOCs onto
///    CompromisedEndpoint alert records…"
///
/// Red Gate test plan #3 (S-DEMO-ENRICHMENT-PIVOT-003).
#[test]
fn test_BC_2_06_019_cyberint_fixture_generator_stamps_scenario_iocs() {
    let org = deadbeef_org();
    let seed: u64 = 42;
    let opts = default_opts(seed);
    let catalog = build_scenario_entity_catalog(seed, &org);

    assert!(
        !catalog.ioc_hashes.is_empty(),
        "Scenario catalog must have non-empty ioc_hashes for this test to be meaningful"
    );
    assert!(
        !catalog.ioc_ips.is_empty(),
        "Scenario catalog must have non-empty ioc_ips for this test to be meaningful"
    );
    assert!(
        !catalog.ioc_domains.is_empty(),
        "Scenario catalog must have non-empty ioc_domains for this test to be meaningful"
    );

    // This call hits the todo!() and MUST PANIC (Red Gate).
    let fixture_set = generate_with_scenario_iocs(
        &org,
        Archetype::CompromisedEndpoint,
        &opts,
        &catalog.ioc_ips,
        &catalog.ioc_domains,
        &catalog.ioc_hashes,
        &catalog.device_cves,
    );

    // If generate_with_scenario_iocs returns (implementation lands), verify IOC stamping.
    // At least one alert record must carry an ioc value from the catalog.
    let alert_records: Vec<&serde_json::Value> = fixture_set
        .records
        .iter()
        .filter(|r| r.get("_surface").and_then(|v| v.as_str()) == Some("alert"))
        .collect();

    assert!(
        !alert_records.is_empty(),
        "CompromisedEndpoint generator must produce at least one alert record"
    );

    // At least one alert record must carry iocs[0].value from the catalog.
    let all_ioc_values: Vec<String> = catalog
        .ioc_hashes
        .iter()
        .chain(catalog.ioc_ips.iter())
        .chain(catalog.ioc_domains.iter())
        .cloned()
        .collect();

    let has_stamped_ioc = alert_records.iter().any(|rec| {
        // Check iocs[] array (primary path for CompromisedEndpoint stamping).
        if let Some(iocs) = rec.get("iocs").and_then(|v| v.as_array()) {
            return iocs.iter().any(|ioc_entry| {
                let v = ioc_entry
                    .get("value")
                    .or_else(|| ioc_entry.get("ioc_value"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                all_ioc_values.iter().any(|catalog_v| catalog_v == v)
            });
        }
        // Fallback: check ioc (singular) field.
        if let Some(ioc) = rec.get("ioc") {
            let v = ioc
                .get("value")
                .or_else(|| ioc.get("ioc_value"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            return all_ioc_values.iter().any(|catalog_v| catalog_v == v);
        }
        false
    });

    assert!(
        has_stamped_ioc,
        "BC-2.06.019 v1.10 PC-4 / AC-002: at least one alert record must carry an IOC value \
         from the scenario catalog (ioc_hashes / ioc_ips / ioc_domains). \
         Catalog ioc_hashes={:?}, ioc_ips={:?}, ioc_domains={:?}. \
         Alert records found: {}",
        catalog.ioc_hashes,
        catalog.ioc_ips,
        catalog.ioc_domains,
        alert_records.len()
    );
}

// ---------------------------------------------------------------------------
// Test 4 — Cyberint alerts route real-schema IOC filter (no synthetic _ioc_value)
// ---------------------------------------------------------------------------

/// Test 4 — BC-2.06.019 v1.10 PC-4 §Interim State: the Cyberint alerts route
/// real-schema IOC filter replaces the removed `_ioc_value` synthetic filter.
///
/// Asserts:
/// - After the route is implemented, an alert record with `iocs[0].value` matching
///   a catalog hash is EXCLUDED from the response when `ioc_hashes=false` (stage 0).
/// - An alert record without any IOC fields passes through regardless of mask state.
///
/// FAIL mode: routes/alerts.rs scenario path contains `todo!()` → the server panics
///   when the HTTP request arrives and the test fails due to connection error or panic
///   propagation.
///
/// SID-1 note: this test needs a running CyberintClone (DTU-EXT dependency via HTTP).
/// It is NOT #[ignore]'d because the CyberintClone runs in-process via prism-dtu-harness.
/// The `todo!()` in routes/alerts.rs IS the blocking dependency — once AC-003 lands,
/// the server stops panicking and this test passes.
///
/// BC-2.06.019 v1.10 PC-4 §Interim State.
/// Red Gate test plan #4 (S-DEMO-ENRICHMENT-PIVOT-003).
#[tokio::test]
async fn test_BC_2_06_019_cyberint_alerts_real_schema_ioc_filter_no_synthetic() {
    let org = deadbeef_org();
    let seed: u64 = 42;
    let demo_token = "test-demo-token-real-schema-filter".to_owned();

    let catalog = build_scenario_entity_catalog(seed, &org);
    assert!(
        !catalog.ioc_hashes.is_empty(),
        "catalog.ioc_hashes must be non-empty for this test to be meaningful"
    );
    let catalog_hash = catalog.ioc_hashes[0].clone();

    // Stage 0 server: elapsed ≈ 10s < 60s → ioc_hashes=false.
    let now = chrono::Utc::now().timestamp();
    let start_stage0: i64 = now - 10;
    let time_anchor_stage0 = chrono::DateTime::from_timestamp(start_stage0, 0)
        .expect("valid timestamp")
        .with_timezone(&chrono::Utc);
    let timeline_stage0 = Arc::new(build_default_incident_timeline(
        catalog.clone(),
        start_stage0,
        &[],
    ));

    let mut clone_stage0 = CyberintClone::new_with_scenario(
        seed,
        Archetype::CompromisedEndpoint,
        org.clone(),
        Arc::clone(&timeline_stage0),
        time_anchor_stage0,
        &catalog,
    )
    .expect("new_with_scenario must succeed");

    clone_stage0.state.register_access_token(demo_token.clone());

    // Inject an alert record with real-schema iocs[].value referencing the catalog hash.
    // This uses the REAL schema format (iocs array with Ioc structs), NOT the old _ioc_value.
    {
        let state_mut = Arc::get_mut(&mut clone_stage0.state)
            .expect("Arc refcount must be 1 before server start");
        state_mut.generated_records.push(json!({
            "alert_id": "real-schema-ioc-alert-003",
            "id": "real-schema-ioc-alert-003",
            "ref_id": "REF-real-schema-003",
            "environment": "production",
            "confidence": 95u64,
            "status": "open",
            "severity": "high",
            "severity_id": 4u64,
            "created_at": "2026-01-01T00:00:00Z",
            "created_by": "system",
            "category": "Malware",
            "type": "malware",
            "source_category": "external",
            "source": "cyberint",
            "affected_assets": ["endpoint.example.com"],
            "title": "Real-Schema IOC Alert (BC-2.06.019 AC-003 test)",
            "modification_date": "2026-01-01T00:01:00Z",
            "description": "Alert with real iocs[] field for AC-003 filter test.",
            "recommendation": "Investigate.",
            "update_date": "2026-01-01T00:01:00Z",
            "_surface": "alert",
            // Real-schema: iocs[] array, NOT the _ioc_value synthetic field.
            "iocs": [
                {
                    "type": "hash_sha256",
                    "value": catalog_hash.clone()
                }
            ]
        }));

        // Inject a non-IOC alert to verify it always passes through.
        state_mut.generated_records.push(json!({
            "alert_id": "non-ioc-alert-003",
            "id": "non-ioc-alert-003",
            "ref_id": "REF-non-ioc-003",
            "environment": "production",
            "confidence": 70u64,
            "status": "open",
            "severity": "medium",
            "severity_id": 3u64,
            "created_at": "2026-01-01T00:00:00Z",
            "created_by": "system",
            "category": "Phishing",
            "type": "phishing",
            "source_category": "external",
            "source": "cyberint",
            "affected_assets": [],
            "title": "Non-IOC Alert (always visible)",
            "modification_date": "2026-01-01T00:01:00Z",
            "description": "Alert without IOC fields — must always appear.",
            "recommendation": "Review.",
            "update_date": "2026-01-01T00:01:00Z",
            "_surface": "alert"
            // No ioc / iocs / alert_data fields — this record must always pass through.
        }));
    }

    // FAIL: this will panic due to todo!() in routes/alerts.rs scenario path.
    clone_stage0
        .start()
        .await
        .expect("stage-0 server start must succeed");
    let base_url = clone_stage0.base_url();
    let client = prism_dtu_common::build_test_client();

    let resp = client
        .get(format!("{base_url}/api/v1/alerts"))
        .header("Cookie", access_token_cookie(&demo_token))
        .send()
        .await
        .expect("GET /api/v1/alerts must reach the server");

    assert_eq!(resp.status().as_u16(), 200, "must return HTTP 200");

    let body: serde_json::Value = resp.json().await.expect("response must be JSON");
    let data = body["data"].as_array().cloned().unwrap_or_default();

    let ids: Vec<String> = data
        .iter()
        .filter_map(|rec| {
            rec.get("alert_id")
                .and_then(|v| v.as_str())
                .map(|s| s.to_owned())
        })
        .collect();

    // At stage 0 (ioc_hashes=false): the IOC-referencing alert MUST be ABSENT.
    assert!(
        !ids.contains(&"real-schema-ioc-alert-003".to_owned()),
        "BC-2.06.019 v1.10 PC-4 / AC-003: at stage 0 (ioc_hashes=false), alert \
         'real-schema-ioc-alert-003' with iocs[0].value='{}' must be ABSENT; \
         found in response ids: {:?}. \
         The real-schema filter (not _ioc_value) must apply.",
        catalog_hash,
        ids
    );

    // The non-IOC alert MUST be PRESENT (no IOC fields → always passes through).
    assert!(
        ids.contains(&"non-ioc-alert-003".to_owned()),
        "BC-2.06.019 v1.10 PC-4: non-IOC alert 'non-ioc-alert-003' must always be \
         PRESENT regardless of StageMask state; not found in: {:?}",
        ids
    );

    // Also verify no _ioc_value key appears anywhere in the source code's route
    // handler (compile-time enforcement: the static filter must be gone).
    // NOTE: This assertion is enforced at the Rust compiler level (types.rs no longer
    // has _ioc_value field), so this comment serves as the test documentation anchor.
}

// ---------------------------------------------------------------------------
// Test 7 — Cyberint TOML sensor spec has IOC columns
// ---------------------------------------------------------------------------

/// Test 7 — BC-2.06.019 v1.10 PC-4 + SAP-2: the Cyberint sensor TOML spec must
/// declare IOC columns matching the real-schema fields introduced by this story.
///
/// Expected columns (per AC-006, primary wire names):
///   `ioc.type`, `ioc.value`
///   `iocs[].type`, `iocs[].value`
///   `alert_data.ip`, `alert_data.domain`, `alert_data.url`
///
/// NOTE: column names use the primary wire key "type" (renamed by serde), NOT "ioc_type"
/// (alias). The TOML spec tracks primary wire keys; aliases are transparent at runtime.
///
/// FAIL mode: neither `sensors/cyberint.sensor.toml` nor
///   `crates/prism-sensors/specs/cyberint.sensor.toml` currently contains these columns →
///   at least one assertion fails.
///
/// SAP-2 parity rule: column in TOML with no DTU struct equivalent = P1 CRITICAL.
///   The reverse (DTU field with no TOML column) = MEDIUM (missing coverage).
///
/// BC-2.06.019 v1.10 PC-4 + SAP-2.
/// Red Gate test plan #7 (S-DEMO-ENRICHMENT-PIVOT-003).
#[test]
fn test_BC_2_06_019_cyberint_alert_toml_spec_has_ioc_columns() {
    // The canonical sensor TOML path (project root relative to workspace).
    // Both locations must be checked; they must be in sync.
    let workspace_root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crates/ parent")
        .parent()
        .expect("workspace root")
        .to_path_buf();

    let toml_paths = [
        workspace_root.join("sensors/cyberint.sensor.toml"),
        workspace_root.join("crates/prism-sensors/specs/cyberint.sensor.toml"),
    ];

    let required_columns = [
        "ioc.type",
        "ioc.value",
        "iocs[].type",
        "iocs[].value",
        "alert_data.ip",
        "alert_data.domain",
        "alert_data.url",
    ];

    for toml_path in &toml_paths {
        let content = std::fs::read_to_string(toml_path).unwrap_or_else(|e| {
            panic!(
                "Failed to read cyberint sensor TOML at {:?}: {e}. \
                 The file must exist after this story ships.",
                toml_path
            )
        });

        for col in &required_columns {
            assert!(
                content.contains(col),
                "BC-2.06.019 v1.10 PC-4 / AC-006: cyberint.sensor.toml at {:?} must \
                 declare column '{}' (IOC real-schema field). \
                 This column is absent — TOML spec update is required as part of \
                 S-DEMO-ENRICHMENT-PIVOT-003.",
                toml_path,
                col
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Test 10 — ioc_hashes=false withholds Cyberint alert with matching hash
// ---------------------------------------------------------------------------

/// Test 10 — BC-2.06.019 v1.10 PC-4 ioc_hashes StageMask field:
/// At stage 0 (ioc_hashes=false), a Cyberint alert carrying `iocs[0].value` equal to
/// a catalog IOC hash must be WITHHELD from the `/api/v1/alerts` response.
///
/// This is a direct StageMask filter correctness test using the real-schema IOC accessor
/// path (`iocs[].value`), confirming the filter works for the hash IOC type.
///
/// Test vectors (canonical from BC-2.06.019 v1.10 §Cyberint route coverage):
/// - Stage 0 (elapsed ≈ 10s < 60s): ioc_hashes=false → hash-IOC alert ABSENT
/// - Stage 3 (elapsed ≈ 400s ≥ 360s): ioc_hashes=true → hash-IOC alert PRESENT
///
/// FAIL mode: routes/alerts.rs scenario path contains `todo!()` → server panics,
///   test fails with panic propagation or connection error.
///
/// BC-2.06.019 v1.10 PC-4, ioc_hashes StageMask field.
/// Red Gate test plan #10 (S-DEMO-ENRICHMENT-PIVOT-003).
#[tokio::test]
async fn test_BC_2_06_019_ioc_hashes_false_withholds_cyberint_alert_with_matching_hash() {
    let org = deadbeef_org();
    let seed: u64 = 99;
    let demo_token = "test-demo-token-ioc-hashes-gate".to_owned();

    let catalog = build_scenario_entity_catalog(seed, &org);
    assert!(
        !catalog.ioc_hashes.is_empty(),
        "Scenario catalog must have non-empty ioc_hashes for this test to be meaningful"
    );
    let catalog_hash = catalog.ioc_hashes[0].clone();

    // -------------------------------------------------------------------------
    // Stage 0 server: elapsed ≈ 10s < 60s → ioc_hashes=false
    // -------------------------------------------------------------------------
    let now = chrono::Utc::now().timestamp();
    let start_stage0: i64 = now - 10;
    let time_anchor_stage0 = chrono::DateTime::from_timestamp(start_stage0, 0)
        .expect("valid timestamp")
        .with_timezone(&chrono::Utc);
    let timeline_stage0 = Arc::new(build_default_incident_timeline(
        catalog.clone(),
        start_stage0,
        &[],
    ));

    let mut clone_stage0 = CyberintClone::new_with_scenario(
        seed,
        Archetype::CompromisedEndpoint,
        org.clone(),
        Arc::clone(&timeline_stage0),
        time_anchor_stage0,
        &catalog,
    )
    .expect("stage-0 CyberintClone must construct");

    clone_stage0.state.register_access_token(demo_token.clone());

    // Inject alert carrying catalog hash in iocs[].value (real schema).
    {
        let state_mut = Arc::get_mut(&mut clone_stage0.state)
            .expect("Arc refcount must be 1 before server start");
        state_mut.generated_records.push(json!({
            "alert_id": "hash-ioc-alert-test10",
            "id": "hash-ioc-alert-test10",
            "ref_id": "REF-hash-ioc-test10",
            "environment": "production",
            "confidence": 99u64,
            "status": "open",
            "severity": "critical",
            "severity_id": 5u64,
            "created_at": "2026-01-01T00:00:00Z",
            "created_by": "system",
            "category": "Malware",
            "type": "malware",
            "source_category": "external",
            "source": "cyberint",
            "affected_assets": ["endpoint.example.com"],
            "title": "Hash IOC Alert (BC-2.06.019 test10)",
            "modification_date": "2026-01-01T00:01:00Z",
            "description": "Alert with matching catalog hash in iocs[].value.",
            "recommendation": "Block hash.",
            "update_date": "2026-01-01T00:01:00Z",
            "_surface": "alert",
            "iocs": [
                {
                    // Primary wire key "type" (serde rename); value matches catalog hash.
                    "type": "hash_sha256",
                    "value": catalog_hash.clone()
                }
            ]
        }));
    }

    // FAIL: this will panic in routes/alerts.rs at the todo!() when a request arrives.
    clone_stage0
        .start()
        .await
        .expect("stage-0 server must start");
    let base_url_stage0 = clone_stage0.base_url();
    let client = prism_dtu_common::build_test_client();

    // Stage 0 request: ioc_hashes=false → hash-IOC alert must be ABSENT.
    let resp0 = client
        .get(format!("{base_url_stage0}/api/v1/alerts"))
        .header("Cookie", access_token_cookie(&demo_token))
        .send()
        .await
        .expect("GET /api/v1/alerts (stage 0) must reach the server");

    assert_eq!(resp0.status().as_u16(), 200, "Stage 0 must return 200");

    let body0: serde_json::Value = resp0.json().await.expect("stage-0 response must be JSON");
    let data0 = body0["data"].as_array().cloned().unwrap_or_default();
    let ids0: Vec<String> = data0
        .iter()
        .filter_map(|rec| {
            rec.get("alert_id")
                .and_then(|v| v.as_str())
                .map(|s| s.to_owned())
        })
        .collect();

    assert!(
        !ids0.contains(&"hash-ioc-alert-test10".to_owned()),
        "BC-2.06.019 v1.10 PC-4 / test #10: at stage 0 (ioc_hashes=false), alert \
         'hash-ioc-alert-test10' with iocs[0].value='{}' (catalog hash) must be ABSENT; \
         found in response ids: {:?}.",
        catalog_hash,
        ids0
    );

    // -------------------------------------------------------------------------
    // Stage 3 server: elapsed ≈ 400s ≥ 360s → ioc_hashes=true
    // -------------------------------------------------------------------------
    let start_stage3: i64 = now - 400;
    let time_anchor_stage3 = chrono::DateTime::from_timestamp(start_stage3, 0)
        .expect("valid timestamp")
        .with_timezone(&chrono::Utc);
    let timeline_stage3 = Arc::new(build_default_incident_timeline(
        catalog.clone(),
        start_stage3,
        &[],
    ));

    let mut clone_stage3 = CyberintClone::new_with_scenario(
        seed,
        Archetype::CompromisedEndpoint,
        org.clone(),
        Arc::clone(&timeline_stage3),
        time_anchor_stage3,
        &catalog,
    )
    .expect("stage-3 CyberintClone must construct");

    clone_stage3.state.register_access_token(demo_token.clone());

    {
        let state_mut = Arc::get_mut(&mut clone_stage3.state)
            .expect("Arc refcount must be 1 before server start");
        state_mut.generated_records.push(json!({
            "alert_id": "hash-ioc-alert-test10",
            "id": "hash-ioc-alert-test10",
            "ref_id": "REF-hash-ioc-test10",
            "environment": "production",
            "confidence": 99u64,
            "status": "open",
            "severity": "critical",
            "severity_id": 5u64,
            "created_at": "2026-01-01T00:00:00Z",
            "created_by": "system",
            "category": "Malware",
            "type": "malware",
            "source_category": "external",
            "source": "cyberint",
            "affected_assets": ["endpoint.example.com"],
            "title": "Hash IOC Alert (BC-2.06.019 test10 stage3)",
            "modification_date": "2026-01-01T00:01:00Z",
            "description": "Alert with matching catalog hash — must be visible at stage 3.",
            "recommendation": "Block hash.",
            "update_date": "2026-01-01T00:01:00Z",
            "_surface": "alert",
            "iocs": [{ "type": "hash_sha256", "value": catalog_hash.clone() }]
        }));
    }

    clone_stage3
        .start()
        .await
        .expect("stage-3 server must start");
    let base_url_stage3 = clone_stage3.base_url();

    let resp3 = client
        .get(format!("{base_url_stage3}/api/v1/alerts"))
        .header("Cookie", access_token_cookie(&demo_token))
        .send()
        .await
        .expect("GET /api/v1/alerts (stage 3) must reach the server");

    assert_eq!(resp3.status().as_u16(), 200, "Stage 3 must return 200");

    let body3: serde_json::Value = resp3.json().await.expect("stage-3 response must be JSON");
    let data3 = body3["data"].as_array().cloned().unwrap_or_default();
    let ids3: Vec<String> = data3
        .iter()
        .filter_map(|rec| {
            rec.get("alert_id")
                .and_then(|v| v.as_str())
                .map(|s| s.to_owned())
        })
        .collect();

    assert!(
        ids3.contains(&"hash-ioc-alert-test10".to_owned()),
        "BC-2.06.019 v1.10 PC-4 / test #10: at stage 3 (ioc_hashes=true), alert \
         'hash-ioc-alert-test10' with iocs[0].value='{}' (catalog hash) must be PRESENT; \
         not found in: {:?}.",
        catalog_hash,
        ids3
    );
}

// ---------------------------------------------------------------------------
// Test 11 — F-PIVOT003-R2-005: fail-closed projection integrity (BC-2.06.019 v1.10 PC-4 §6)
// ---------------------------------------------------------------------------

/// Test 11 — BC-2.06.019 v1.10 PC-4 step 6: fail-closed projection integrity.
///
/// A record that CANNOT be deserialized as `Alert` (e.g., missing required fields)
/// MUST be WITHHELD from the response — not passed through.
///
/// Rationale: the StageMask IOC filter cannot be correctly applied to an
/// undeserializable record, so surfacing it would violate the IOC masking guarantee
/// (F-PIVOT003-R2-005, BC-2.06.019 v1.10 PC-4 §6).
///
/// LOAD-BEARING (production path): goes through `CyberintClone::new_with_scenario`
/// (the same constructor harness.rs uses), starts an HTTP server, and asserts the
/// injected malformed record is absent from the response.
///
/// BC-2.06.019 v1.10 PC-4 step 6. F-PIVOT003-R2-005.
#[tokio::test]
async fn test_BC_2_06_019_fail_closed_malformed_alert_is_withheld() {
    let org = deadbeef_org();
    let seed: u64 = 77;
    let demo_token = "test-demo-token-fail-closed".to_owned();

    let catalog = build_scenario_entity_catalog(seed, &org);
    let now = chrono::Utc::now().timestamp();

    // Stage 3 server (elapsed ≈ 400s): all IOC mask fields true.
    // A malformed record must still be withheld even when all mask bits are true.
    let start_stage3: i64 = now - 400;
    let time_anchor = chrono::DateTime::from_timestamp(start_stage3, 0)
        .expect("valid timestamp")
        .with_timezone(&chrono::Utc);
    let timeline = Arc::new(build_default_incident_timeline(
        catalog.clone(),
        start_stage3,
        &[],
    ));

    let mut clone_stage3 = CyberintClone::new_with_scenario(
        seed,
        Archetype::CompromisedEndpoint,
        org.clone(),
        Arc::clone(&timeline),
        time_anchor,
        &catalog,
    )
    .expect("new_with_scenario must succeed for fail-closed test");

    clone_stage3.state.register_access_token(demo_token.clone());

    {
        let state_mut = Arc::get_mut(&mut clone_stage3.state)
            .expect("Arc refcount must be 1 before server start");

        // Inject a record that is entirely wrong — missing ALL required Alert fields.
        // serde_json::from_value::<Alert>(...) will Err → must be withheld (fail-closed).
        state_mut.generated_records.push(json!({
            "not_an_alert_field": "this record cannot deserialize as Alert",
            "random_noise": 42,
            "_surface": "alert"
        }));

        // Inject a valid alert to confirm the server is functioning (non-vacuous).
        state_mut.generated_records.push(json!({
            "alert_id": "valid-alert-fail-closed-test11",
            "id": "valid-alert-fail-closed-test11",
            "ref_id": "REF-valid-fc-test11",
            "environment": "production",
            "confidence": 80u64,
            "status": "open",
            "severity": "medium",
            "severity_id": 3u64,
            "created_at": "2026-01-01T00:00:00Z",
            "created_by": "system",
            "category": "Phishing",
            "type": "phishing",
            "source_category": "external",
            "source": "cyberint",
            "affected_assets": [],
            "title": "Valid Alert (fail-closed test11)",
            "modification_date": "2026-01-01T00:01:00Z",
            "description": "Valid alert — must appear in response.",
            "recommendation": "Review.",
            "update_date": "2026-01-01T00:01:00Z",
            "_surface": "alert"
        }));
    }

    clone_stage3
        .start()
        .await
        .expect("stage-3 server must start for fail-closed test");
    let base_url = clone_stage3.base_url();
    let client = prism_dtu_common::build_test_client();

    let resp = client
        .get(format!("{base_url}/api/v1/alerts"))
        .header("Cookie", access_token_cookie(&demo_token))
        .send()
        .await
        .expect("GET /api/v1/alerts (fail-closed) must reach the server");

    assert_eq!(resp.status().as_u16(), 200, "must return HTTP 200");

    let body: serde_json::Value = resp.json().await.expect("response must be JSON");
    let data = body["data"].as_array().cloned().unwrap_or_default();

    // The malformed record must NOT appear — fail-closed withholds it.
    // Since the malformed record has no alert_id, check by count and absence of noise field.
    let noise_records: Vec<&serde_json::Value> = data
        .iter()
        .filter(|rec| rec.get("not_an_alert_field").is_some())
        .collect();

    assert!(
        noise_records.is_empty(),
        "BC-2.06.019 v1.10 PC-4 §6 / F-PIVOT003-R2-005: fail-closed — malformed record \
         with 'not_an_alert_field' key MUST be withheld from the response; \
         found {} such record(s) in response. The fail-open path was changed to fail-closed.",
        noise_records.len()
    );

    // The valid alert must appear (confirms server is running and response is not empty).
    let valid_ids: Vec<String> = data
        .iter()
        .filter_map(|rec| {
            rec.get("alert_id")
                .and_then(|v| v.as_str())
                .map(|s| s.to_owned())
        })
        .collect();

    assert!(
        valid_ids.contains(&"valid-alert-fail-closed-test11".to_owned()),
        "BC-2.06.019 v1.10 PC-4 §6: valid alert 'valid-alert-fail-closed-test11' must be PRESENT; \
         not found in: {:?}",
        valid_ids
    );
}
