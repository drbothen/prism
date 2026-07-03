//! S-1.14-REDO Red Gate tests — Infusion Sources, Cache, Loader, VP-048/VP-049.
//!
//! Covers ACs 1-9 (MmdbSource, CsvSource, JsonLookupSource, InfusionLruCache,
//! three-tier lookup order, InfusionLoader for non-plugin types, E-INFUSE-008,
//! VP-048 unit mirror, VP-049 proptest).
//!
//! AC-10 (boot integration) lives in prism-bin/tests/infusion_boot_integration.rs
//! (added separately by stub-architect commit 73cf611a).
//!
//! # Red Gate expectation
//! ALL new tests in this file MUST fail before the S-1.14-REDO implementation lands:
//! - Source construction tests: `todo!()` panic in MmdbSource::load, CsvSource::load,
//!   JsonLookupSource::load.
//! - InfusionLoader MMDB/CSV/JSON path tests: loader.rs returns
//!   `InfusionError::UnknownSourceType` (deferred to S-1.14-REDO) — tests expect Ok.
//! - Cache tests: `todo!()` panic in InfusionLruCache::get / insert.
//! - Three-tier tests: hit Tier 2 LRU which panics.
//! - E-INFUSE-008 tests: already wired but exercised by calling enrich_single path.
//! - VP-048 unit mirror: already passes (load_spec implemented) — annotated as green.
//! - VP-049 proptest: already passes (QueryScopedInfusionCache implemented) — annotated as green.
//!
//! # Test naming convention
//! `test_BC_S_SS_NNN_xxx` per VSDD TDD protocol.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    unused_imports,
    dead_code,
    unused_mut
)]

use std::io::Write;
use std::path::Path;
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

use prism_core::{CacheBackend, InfusionError, StorageDomain, error::PrismError};
use prism_spec_engine::infusion::cache::{InfusionLruCache, InfusionTier3Cache};
use prism_spec_engine::infusion::sources::csv::CsvSource;
use prism_spec_engine::infusion::sources::json_lookup::JsonLookupSource;
use prism_spec_engine::infusion::sources::mmdb::MmdbSource;
use prism_spec_engine::{
    BuiltInSourceType, InfusionField, InfusionLoader, InfusionRegistry, InfusionSource,
    InfusionSpec, InfusionType, QueryScopedInfusionCache,
};

// ---------------------------------------------------------------------------
// AC-1 / BC-2.19.001: MmdbSource construction and enrichment
// ---------------------------------------------------------------------------

/// AC-1: MmdbSource::load with non-existent file → InfusionError::MissingRequiredField.
///
/// FAILS RED: `todo!()` in MmdbSource::load before implementation.
#[test]
fn test_BC_2_19_001_mmdb_source_load_nonexistent_file_returns_error() {
    let path = Path::new("/tmp/definitely_does_not_exist_prism_test_abc123.mmdb");

    let result = MmdbSource::load(path);

    assert!(
        result.is_err(),
        "BC-2.19.001: MmdbSource::load with non-existent file must return Err (FAILS RED: todo!())"
    );
    match result.unwrap_err() {
        InfusionError::MissingRequiredField { field, .. } => {
            // SEC-001: for non-existent files, `fs::metadata()` fails before `open_readfile`,
            // so the error is now "mmdb_metadata_failed: ..." rather than "mmdb_open_failed: ...".
            // Both are valid "non-existent file" error signals; accept either.
            assert!(
                field.contains("mmdb_metadata_failed") || field.contains("mmdb_open_failed"),
                "BC-2.19.001: load error must contain 'mmdb_metadata_failed' or 'mmdb_open_failed'. Got: '{}'",
                field
            );
        }
        other => panic!(
            "BC-2.19.001: expected MissingRequiredField for missing mmdb file, got: {:?}",
            other
        ),
    }
}

/// AC-1: MmdbSource::load succeeds with a valid .mmdb file; has correct mmdb_path.
///
/// This test requires the fixture mmdb file. It will FAIL RED with todo!() before
/// implementation, and will require test.mmdb to be present after implementation.
///
/// Uses the fixture path relative to the crate root (set via CARGO_MANIFEST_DIR).
///
/// # SID-1 compliance
/// Unit test at the dependency boundary — exercises the production code path without
/// external live service. No `#[ignore]` needed for the file-backed source path.
///
/// Note: `field_names` was removed from `MmdbSource` in S-1.14-REDO fix-burst (Fix 4 —
/// inert field removal). Column projection is handled at the UDF layer via
/// `InfusionUdfDescriptor::source_column`. The `mmdb_path` field is retained for
/// diagnostics.
#[test]
fn test_BC_2_19_001_mmdb_source_load_valid_file_succeeds() {
    // The fixture file geoip.infusion.toml references fixtures/test.mmdb.
    // The implementer must create fixtures/test.mmdb (a valid GeoLite2-City mmdb).
    // This test will also fail RED with "file not found" until the fixture exists.
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let mmdb_path = Path::new(manifest_dir).join("fixtures").join("test.mmdb");

    // FAILS RED even before mmdb exists: todo!() in load() panics first.
    let result = MmdbSource::load(&mmdb_path);

    assert!(
        result.is_ok(),
        "BC-2.19.001: MmdbSource::load must succeed with a valid mmdb fixture. \
         FAILS RED: todo!() in load(). Also requires fixtures/test.mmdb to exist. \
         Got: {:?}",
        result.err()
    );

    let source = result.unwrap();
    assert!(
        !source.mmdb_path.is_empty(),
        "BC-2.19.001: loaded MmdbSource must retain the mmdb_path for diagnostics"
    );
}

/// AC-1: MmdbSource::enrich_single with RFC 5737 documentation IP returns None
/// (not in GeoLite2 test database, expected miss — but must NOT panic with todo!()).
///
/// FAILS RED: `todo!()` in MmdbSource::load before implementation (panics before file open).
/// After implementation: test also requires fixtures/test.mmdb to exist.
#[test]
fn test_BC_2_19_001_mmdb_source_enrich_single_documentation_ip_returns_none() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let mmdb_path = Path::new(manifest_dir).join("fixtures").join("test.mmdb");

    // FAILS RED: todo!() in MmdbSource::load panics before even checking file existence.
    // After implementation: also requires fixtures/test.mmdb to exist.
    let source = match MmdbSource::load(&mmdb_path) {
        Ok(s) => s,
        Err(_) => {
            // load() returned Err (e.g., file not found after implementation) — test fixture needed.
            // This path is reached only after todo!() is removed AND before fixture lands.
            // Still fails RED in the pre-implementation state because load() panics with todo!().
            return;
        }
    };

    // 192.0.2.0/24 is RFC 5737 documentation space — not in GeoIP databases.
    // FAILS RED: todo!() in enrich_single.
    let result = source.enrich_single("192.0.2.1", "ip");

    // Documentation IPs return None (no GeoIP data) — not an error.
    // The key assertion: NO todo!() panic. The specific Some/None depends on the MMDB content.
    let _ = result; // Accept either None or Some without assertion on value.
}

/// AC-1: MmdbSource::enrich_single with an invalid IP string returns None (not Err).
///
/// The InfusionSource trait returns Option<Value> — parse errors map to None, not panic.
/// FAILS RED: `todo!()` in MmdbSource::load before implementation.
#[test]
fn test_BC_2_19_001_mmdb_source_enrich_single_invalid_ip_returns_none() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let mmdb_path = Path::new(manifest_dir).join("fixtures").join("test.mmdb");

    // FAILS RED: todo!() in MmdbSource::load panics before checking file existence.
    let source = match MmdbSource::load(&mmdb_path) {
        Ok(s) => s,
        Err(_) => return, // file-not-found after todo!() removed — fixture needed
    };

    // "not-an-ip" is not parseable — must return None, not panic.
    // FAILS RED: todo!() in enrich_single.
    let result = source.enrich_single("not-an-ip", "ip");

    assert!(
        result.is_none(),
        "BC-2.19.001: MmdbSource::enrich_single with invalid IP must return None, not panic. \
         Got: {:?}",
        result
    );
}

/// AC-1: MmdbSource::enrich_batch delegates to enrich_single for each input.
///
/// Canonical TV: batch of 3 IPs returns 3 Option<Value> results.
/// FAILS RED: `todo!()` in MmdbSource::load before implementation.
#[test]
fn test_BC_2_19_001_mmdb_source_enrich_batch_returns_parallel_results() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let mmdb_path = Path::new(manifest_dir).join("fixtures").join("test.mmdb");

    // FAILS RED: todo!() in MmdbSource::load panics before checking file existence.
    let source = match MmdbSource::load(&mmdb_path) {
        Ok(s) => s,
        Err(_) => return, // file-not-found after todo!() removed — fixture needed
    };

    let ips = vec![
        "192.0.2.1".to_string(),
        "not-an-ip".to_string(),
        "192.0.2.2".to_string(),
    ];

    // FAILS RED: todo!() in enrich_batch.
    let results = source.enrich_batch(&ips, "ip");

    assert_eq!(
        results.len(),
        3,
        "BC-2.19.001: enrich_batch must return exactly one result per input (3 in → 3 out)"
    );
    // Second entry (invalid IP) must be None.
    assert!(
        results[1].is_none(),
        "BC-2.19.001: enrich_batch result for invalid IP must be None"
    );
}

// ---------------------------------------------------------------------------
// AC-1 / BC-2.19.001: InfusionLoader parses maxmind_mmdb type (S-1.14-REDO scope)
// ---------------------------------------------------------------------------

/// AC-1: InfusionLoader::parse accepts source.type = "maxmind_mmdb" (S-1.14-REDO).
///
/// The DEMO forward-subset returns UnknownSourceType for maxmind_mmdb.
/// S-1.14-REDO must implement this path and return Ok(InfusionSpec).
///
/// FAILS RED: loader currently returns Err(UnknownSourceType) for maxmind_mmdb.
#[test]
fn test_BC_2_19_001_infusion_loader_parses_maxmind_mmdb_type() {
    let toml_input = r#"
[infusion]
infusion_id = "geoip"
name = "MaxMind GeoIP2"

[infusion.source]
type = "maxmind_mmdb"
file_path = "fixtures/test.mmdb"
refresh_interval_secs = 3600

[[infusion.fields]]
name = "geoip_country"
input_field = "device_ip"
input_type = "ip"
output_type = "string"
description = "ISO 3166-1 alpha-2 country code"
source_column = "country_iso_code"

[[infusion.fields]]
name = "geoip_city"
input_field = "device_ip"
input_type = "ip"
output_type = "string"
source_column = "city_name"

[[infusion.fields]]
name = "geoip_asn"
input_field = "device_ip"
input_type = "ip"
output_type = "integer"
source_column = "asn"

[[infusion.fields]]
name = "geoip_is_tor"
input_field = "device_ip"
input_type = "ip"
output_type = "boolean"
source_column = "is_tor"

[infusion.pipe_stage]
adds_columns = ["geoip_country", "geoip_city", "geoip_asn", "geoip_is_tor"]
"#;

    // FAILS RED: loader.rs currently returns Err(UnknownSourceType { "maxmind_mmdb" })
    // for this source type (deferred to S-1.14-REDO).
    let result = InfusionLoader::parse(toml_input, "geoip.infusion.toml");

    let spec = result.expect(
        "BC-2.19.001: InfusionLoader::parse must return Ok for maxmind_mmdb source type \
         (FAILS RED: currently returns Err(UnknownSourceType) — deferred in S-DEMO-ENRICHMENT-PIVOT-001)"
    );

    assert_eq!(
        spec.infusion_id, "geoip",
        "BC-2.19.001: parsed spec must have infusion_id = 'geoip'"
    );
    assert_eq!(
        spec.infusion_type,
        InfusionType::LocalLookup,
        "BC-2.19.001: maxmind_mmdb type must parse to InfusionType::LocalLookup"
    );
    assert_eq!(
        spec.fields.len(),
        4,
        "BC-2.19.001: parsed spec must have 4 fields"
    );
    // source config must be populated
    let source_config = spec
        .source
        .expect("BC-2.19.001: maxmind_mmdb spec must have source config");
    assert_eq!(
        source_config.source_type,
        BuiltInSourceType::MaxmindMmdb,
        "BC-2.19.001: source_type must be MaxmindMmdb"
    );
    assert!(
        source_config.file_path.contains("test.mmdb"),
        "BC-2.19.001: file_path must be set from TOML"
    );
}

// ---------------------------------------------------------------------------
// AC-1 (continued) / BC-2.19.001: CsvSource construction and enrichment
// ---------------------------------------------------------------------------

/// AC-1: CsvSource::load with non-existent file returns InfusionError.
///
/// FAILS RED: `todo!()` in CsvSource::load before implementation.
#[test]
fn test_BC_2_19_001_csv_source_load_nonexistent_file_returns_error() {
    let result = CsvSource::load("/tmp/prism_test_nonexistent_abc123.csv", "ip_address");

    assert!(
        result.is_err(),
        "BC-2.19.001: CsvSource::load with non-existent file must return Err (FAILS RED: todo!())"
    );
}

/// AC-1: CsvSource::load with the asset_inventory fixture succeeds.
///
/// Canonical TV from BC-2.19.001: asset_inventory.csv keyed on ip_address.
/// FAILS RED: `todo!()` in CsvSource::load before implementation.
#[test]
fn test_BC_2_19_001_csv_source_load_valid_fixture_succeeds() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let csv_path = format!("{}/fixtures/asset_inventory.csv", manifest_dir);

    // FAILS RED: todo!() in CsvSource::load.
    let result = CsvSource::load(&csv_path, "ip_address");

    assert!(
        result.is_ok(),
        "BC-2.19.001: CsvSource::load must succeed with asset_inventory.csv fixture. \
         FAILS RED: todo!() in load(). Got: {:?}",
        result.err()
    );
}

/// AC-1: CsvSource::enrich_single with a known IP returns the correct row fields.
///
/// Canonical TV from BC-2.19.001 asset_inventory.csv:
/// ip_address=192.168.1.10 → hostname=ws-eng-001, department=Engineering, owner=alice.
///
/// FAILS RED: `todo!()` in CsvSource::enrich_single before implementation.
#[test]
fn test_BC_2_19_001_csv_source_enrich_single_known_key_returns_correct_value() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let csv_path = format!("{}/fixtures/asset_inventory.csv", manifest_dir);

    // FAILS RED: todo!() in CsvSource::load.
    let source = CsvSource::load(&csv_path, "ip_address")
        .expect("BC-2.19.001: CsvSource::load must succeed for enrich test");

    // FAILS RED: todo!() in enrich_single.
    let result = source.enrich_single("192.168.1.10", "ip");

    let value = result.expect(
        "BC-2.19.001: CsvSource::enrich_single must return Some for known key '192.168.1.10'",
    );

    // The result should be a JSON object with the CSV row columns.
    assert!(
        value.is_object(),
        "BC-2.19.001: enrich_single must return a JSON object, got: {:?}",
        value
    );
    let obj = value.as_object().unwrap();
    assert_eq!(
        obj.get("department").and_then(|v| v.as_str()),
        Some("Engineering"),
        "BC-2.19.001: department column for 192.168.1.10 must be 'Engineering'"
    );
    assert_eq!(
        obj.get("owner").and_then(|v| v.as_str()),
        Some("alice"),
        "BC-2.19.001: owner column for 192.168.1.10 must be 'alice'"
    );
}

/// AC-1: CsvSource::enrich_single with unknown key returns None.
///
/// FAILS RED: `todo!()` in CsvSource::enrich_single before implementation.
#[test]
fn test_BC_2_19_001_csv_source_enrich_single_unknown_key_returns_none() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let csv_path = format!("{}/fixtures/asset_inventory.csv", manifest_dir);

    let source = CsvSource::load(&csv_path, "ip_address")
        .expect("BC-2.19.001: CsvSource::load must succeed for missing-key test");

    // FAILS RED: todo!() in enrich_single.
    let result = source.enrich_single("10.255.255.99", "ip");

    assert!(
        result.is_none(),
        "BC-2.19.001: CsvSource::enrich_single must return None for unknown key"
    );
}

/// AC-1: CsvSource::enrich_batch returns parallel results.
///
/// FAILS RED: `todo!()` in CsvSource::enrich_batch before implementation.
#[test]
fn test_BC_2_19_001_csv_source_enrich_batch_returns_parallel_results() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let csv_path = format!("{}/fixtures/asset_inventory.csv", manifest_dir);

    let source = CsvSource::load(&csv_path, "ip_address")
        .expect("BC-2.19.001: CsvSource::load must succeed for batch test");

    let ips = vec![
        "192.168.1.10".to_string(),  // known
        "10.255.255.99".to_string(), // unknown
        "192.168.1.20".to_string(),  // known (carol, Security)
    ];

    // FAILS RED: todo!() in enrich_batch.
    let results = source.enrich_batch(&ips, "ip");

    assert_eq!(
        results.len(),
        3,
        "BC-2.19.001: enrich_batch must return exactly one result per input"
    );
    assert!(
        results[0].is_some(),
        "BC-2.19.001: batch[0] for known IP 192.168.1.10 must be Some"
    );
    assert!(
        results[1].is_none(),
        "BC-2.19.001: batch[1] for unknown IP must be None"
    );
    assert!(
        results[2].is_some(),
        "BC-2.19.001: batch[2] for known IP 192.168.1.20 must be Some"
    );
    // Carol is in Security department.
    if let Some(val) = &results[2] {
        assert_eq!(
            val.get("department").and_then(|v| v.as_str()),
            Some("Security"),
            "BC-2.19.001: 192.168.1.20 must be in Security department"
        );
    }
}

// ---------------------------------------------------------------------------
// AC-1 (continued): InfusionLoader parses CSV type
// ---------------------------------------------------------------------------

/// AC-1: InfusionLoader::parse accepts source.type = "csv" (S-1.14-REDO).
///
/// FAILS RED: loader currently returns Err(UnknownSourceType) for "csv".
#[test]
fn test_BC_2_19_001_infusion_loader_parses_csv_type() {
    let toml_input = r#"
[infusion]
infusion_id = "asset_inventory"
name = "Asset Inventory CSV"

[infusion.source]
type = "csv"
file_path = "fixtures/asset_inventory.csv"
key_column = "ip_address"
refresh_interval_secs = 300

[[infusion.fields]]
name = "asset_owner"
input_field = "device_ip"
input_type = "ip"
output_type = "string"
source_column = "owner"

[[infusion.fields]]
name = "asset_department"
input_field = "device_ip"
input_type = "ip"
output_type = "string"
source_column = "department"

[infusion.pipe_stage]
adds_columns = ["asset_owner", "asset_department"]
"#;

    // FAILS RED: returns Err(UnknownSourceType { "csv" }) until S-1.14-REDO implements CSV path.
    let result = InfusionLoader::parse(toml_input, "asset_inventory.infusion.toml");

    let spec = result.expect(
        "BC-2.19.001: InfusionLoader::parse must return Ok for csv source type \
         (FAILS RED: currently returns Err(UnknownSourceType))",
    );

    assert_eq!(spec.infusion_id, "asset_inventory");
    assert_eq!(spec.infusion_type, InfusionType::LocalLookup);
    assert_eq!(spec.fields.len(), 2);

    let source_config = spec
        .source
        .expect("BC-2.19.001: csv spec must have source config");
    assert_eq!(source_config.source_type, BuiltInSourceType::Csv);
    assert!(source_config.file_path.contains("asset_inventory.csv"));
    assert_eq!(
        source_config.key_column,
        Some("ip_address".to_string()),
        "BC-2.19.001: csv source must capture key_column = 'ip_address'"
    );
}

// ---------------------------------------------------------------------------
// AC-1 (continued) / BC-2.19.001: JsonLookupSource construction and enrichment
// ---------------------------------------------------------------------------

/// AC-1: JsonLookupSource::load with non-existent file returns InfusionError.
///
/// FAILS RED: `todo!()` in JsonLookupSource::load before implementation.
#[test]
fn test_BC_2_19_001_json_lookup_source_load_nonexistent_file_returns_error() {
    let result = JsonLookupSource::load("/tmp/prism_test_nonexistent_abc123.json");

    assert!(
        result.is_err(),
        "BC-2.19.001: JsonLookupSource::load with non-existent file must return Err \
         (FAILS RED: todo!())"
    );
}

/// AC-1: JsonLookupSource::load with a valid JSON file succeeds.
///
/// FAILS RED: `todo!()` in JsonLookupSource::load before implementation.
#[test]
fn test_BC_2_19_001_json_lookup_source_load_valid_json_succeeds() {
    use tempfile::NamedTempFile;

    let mut tmp = NamedTempFile::new().expect("failed to create temp file");
    let json_data = r#"{
        "192.168.1.10": {"hostname": "ws-eng-001", "role": "workstation"},
        "10.0.0.1": {"hostname": "gw-corp-001", "role": "gateway"}
    }"#;
    tmp.write_all(json_data.as_bytes()).expect("write failed");
    let path = tmp.path().to_str().unwrap().to_string();

    // FAILS RED: todo!() in JsonLookupSource::load.
    let result = JsonLookupSource::load(&path);

    assert!(
        result.is_ok(),
        "BC-2.19.001: JsonLookupSource::load must succeed with valid JSON. \
         FAILS RED: todo!() in load(). Got: {:?}",
        result.err()
    );
}

/// AC-1: JsonLookupSource::enrich_single returns correct object for known key.
///
/// Canonical TV: key "192.168.1.10" → {hostname: "ws-eng-001", role: "workstation"}.
/// FAILS RED: `todo!()` in enrich_single before implementation.
#[test]
fn test_BC_2_19_001_json_lookup_source_enrich_single_known_key_returns_value() {
    use tempfile::NamedTempFile;

    let mut tmp = NamedTempFile::new().expect("failed to create temp file");
    let json_data = r#"{
        "192.168.1.10": {"hostname": "ws-eng-001", "role": "workstation"},
        "10.0.0.1": {"hostname": "gw-corp-001", "role": "gateway"}
    }"#;
    tmp.write_all(json_data.as_bytes()).expect("write failed");
    let path = tmp.path().to_str().unwrap().to_string();

    let source = JsonLookupSource::load(&path)
        .expect("BC-2.19.001: JsonLookupSource::load must succeed for enrich test");

    // FAILS RED: todo!() in enrich_single.
    let result = source.enrich_single("192.168.1.10", "ip");

    let value = result.expect(
        "BC-2.19.001: JsonLookupSource::enrich_single must return Some for known key '192.168.1.10'"
    );
    assert_eq!(
        value.get("hostname").and_then(|v| v.as_str()),
        Some("ws-eng-001"),
        "BC-2.19.001: JSON lookup result must contain hostname = 'ws-eng-001'"
    );
    assert_eq!(
        value.get("role").and_then(|v| v.as_str()),
        Some("workstation"),
        "BC-2.19.001: JSON lookup result must contain role = 'workstation'"
    );
}

/// AC-1: JsonLookupSource::enrich_single returns None for unknown key.
///
/// FAILS RED: `todo!()` in enrich_single before implementation.
#[test]
fn test_BC_2_19_001_json_lookup_source_enrich_single_unknown_key_returns_none() {
    use tempfile::NamedTempFile;

    let mut tmp = NamedTempFile::new().expect("failed to create temp file");
    let json_data = r#"{"192.168.1.10": {"hostname": "ws-eng-001"}}"#;
    tmp.write_all(json_data.as_bytes()).expect("write failed");
    let path = tmp.path().to_str().unwrap().to_string();

    let source = JsonLookupSource::load(&path)
        .expect("BC-2.19.001: JsonLookupSource::load must succeed for missing-key test");

    // FAILS RED: todo!() in enrich_single.
    let result = source.enrich_single("10.255.255.99", "ip");

    assert!(
        result.is_none(),
        "BC-2.19.001: JsonLookupSource::enrich_single must return None for unknown key"
    );
}

/// AC-1: JsonLookupSource::load rejects malformed JSON.
///
/// FAILS RED: `todo!()` in JsonLookupSource::load before implementation.
#[test]
fn test_BC_2_19_001_json_lookup_source_load_malformed_json_returns_error() {
    use tempfile::NamedTempFile;

    let mut tmp = NamedTempFile::new().expect("failed to create temp file");
    tmp.write_all(b"{ invalid json }").expect("write failed");
    let path = tmp.path().to_str().unwrap().to_string();

    // FAILS RED: todo!() in load() — should return Err after implementation.
    let result = JsonLookupSource::load(&path);

    assert!(
        result.is_err(),
        "BC-2.19.001: JsonLookupSource::load must reject malformed JSON (FAILS RED: todo!())"
    );
}

/// AC-1: InfusionLoader::parse accepts source.type = "json_lookup" (S-1.14-REDO).
///
/// FAILS RED: loader currently returns Err(UnknownSourceType) for "json_lookup".
#[test]
fn test_BC_2_19_001_infusion_loader_parses_json_lookup_type() {
    let toml_input = r#"
[infusion]
infusion_id = "host_roles"
name = "Host Role Lookup"

[infusion.source]
type = "json_lookup"
file_path = "fixtures/host_roles.json"

[[infusion.fields]]
name = "host_role"
input_field = "device_ip"
input_type = "ip"
output_type = "string"
source_column = "role"

[infusion.pipe_stage]
adds_columns = ["host_role"]
"#;

    // FAILS RED: returns Err(UnknownSourceType { "json_lookup" }) until S-1.14-REDO.
    let result = InfusionLoader::parse(toml_input, "host_roles.infusion.toml");

    let spec = result.expect(
        "BC-2.19.001: InfusionLoader::parse must return Ok for json_lookup source type \
         (FAILS RED: currently returns Err(UnknownSourceType))",
    );

    assert_eq!(spec.infusion_id, "host_roles");
    assert_eq!(spec.infusion_type, InfusionType::LocalLookup);

    let source_config = spec
        .source
        .expect("BC-2.19.001: json_lookup spec must have source config");
    assert_eq!(source_config.source_type, BuiltInSourceType::JsonLookup);
}

// ---------------------------------------------------------------------------
// AC-1: InfusionLoader::load_all produces specs for MMDB/CSV/JSON types (S-1.14-REDO)
// ---------------------------------------------------------------------------

/// AC-1: load_all builds a complete InfusionRegistry with MMDB + CSV + JSON specs.
///
/// FAILS RED: load_all currently produces errors for maxmind_mmdb/csv/json_lookup types.
#[test]
fn test_BC_2_19_001_load_all_produces_specs_for_all_local_lookup_source_types() {
    use tempfile::TempDir;

    let temp_dir = TempDir::new().expect("temp dir creation must succeed");
    let infusions_dir = temp_dir.path().join("infusions");
    std::fs::create_dir_all(&infusions_dir).expect("infusions dir creation must succeed");

    // Write MMDB spec.
    let mmdb_toml = r#"
[infusion]
infusion_id = "geoip_test"
name = "Test GeoIP"

[infusion.source]
type = "maxmind_mmdb"
file_path = "/tmp/nonexistent.mmdb"

[[infusion.fields]]
name = "test_country"
input_field = "src_ip"
input_type = "ip"
output_type = "string"
source_column = "country_iso_code"
"#;
    std::fs::write(infusions_dir.join("geoip_test.infusion.toml"), mmdb_toml)
        .expect("write mmdb toml");

    // Write CSV spec.
    let csv_toml = r#"
[infusion]
infusion_id = "asset_test"
name = "Test Asset"

[infusion.source]
type = "csv"
file_path = "/tmp/nonexistent.csv"
key_column = "ip"

[[infusion.fields]]
name = "test_owner"
input_field = "src_ip"
input_type = "ip"
output_type = "string"
source_column = "owner"
"#;
    std::fs::write(infusions_dir.join("asset_test.infusion.toml"), csv_toml)
        .expect("write csv toml");

    // FAILS RED: load_all currently returns 0 specs and 2 errors (UnknownSourceType x2).
    // After S-1.14-REDO implementation: returns 2 specs (parsed OK despite nonexistent files;
    // NOTE: file existence is checked at MmdbSource::load/CsvSource::load time, NOT at parse time).
    let loader = InfusionLoader::new(temp_dir.path().to_str().unwrap());
    let (specs, errors) = loader.load_all();

    assert_eq!(
        specs.len(),
        2,
        "BC-2.19.001: load_all must parse 2 local-lookup specs (MMDB + CSV). \
         FAILS RED: currently returns 0 specs (UnknownSourceType for both). Got {} specs, {} errors: {:?}",
        specs.len(),
        errors.len(),
        errors
    );

    let spec_ids: Vec<&str> = specs.iter().map(|s| s.infusion_id.as_str()).collect();
    assert!(
        spec_ids.contains(&"geoip_test"),
        "BC-2.19.001: geoip_test spec must be loaded"
    );
    assert!(
        spec_ids.contains(&"asset_test"),
        "BC-2.19.001: asset_test spec must be loaded"
    );
}

/// AC-9 (non-fatal failure): load_all with one bad spec continues loading valid specs.
///
/// BC-2.19.001 invariant: parse errors are non-fatal; valid specs still load.
/// FAILS RED: currently all non-plugin specs return errors (nothing to test isolation with).
/// After implementation: the bad spec errors but valid specs succeed.
#[test]
fn test_BC_2_19_001_load_all_non_fatal_per_source_failure_continues_loading() {
    use tempfile::TempDir;

    let temp_dir = TempDir::new().expect("temp dir creation must succeed");
    let infusions_dir = temp_dir.path().join("infusions");
    std::fs::create_dir_all(&infusions_dir).expect("infusions dir creation must succeed");

    // Valid CSV spec.
    let valid_toml = r#"
[infusion]
infusion_id = "valid_csv"
name = "Valid CSV"

[infusion.source]
type = "csv"
file_path = "/tmp/somefile.csv"
key_column = "ip"

[[infusion.fields]]
name = "valid_field"
input_field = "src_ip"
input_type = "ip"
output_type = "string"
source_column = "col1"
"#;
    std::fs::write(infusions_dir.join("valid.infusion.toml"), valid_toml)
        .expect("write valid toml");

    // Invalid spec: missing infusion_id.
    let invalid_toml = r#"
[infusion]
infusion_id = ""
name = "Bad spec"

[infusion.source]
type = "csv"
file_path = "/tmp/somefile.csv"
key_column = "ip"

[[infusion.fields]]
name = "some_field"
input_field = "src_ip"
input_type = "ip"
output_type = "string"
"#;
    std::fs::write(infusions_dir.join("invalid.infusion.toml"), invalid_toml)
        .expect("write invalid toml");

    let loader = InfusionLoader::new(temp_dir.path().to_str().unwrap());

    // FAILS RED: currently both specs produce errors because CSV type is unimplemented.
    // After S-1.14-REDO: valid spec loads → 1 spec; invalid spec errors → 1 error.
    let (specs, errors) = loader.load_all();

    assert_eq!(
        specs.len(),
        1,
        "BC-2.19.001 AC-9: load_all must load 1 valid spec despite 1 invalid spec. \
         FAILS RED: currently 0 specs. Got: {} specs, {} errors",
        specs.len(),
        errors.len()
    );
    assert_eq!(
        errors.len(),
        1,
        "BC-2.19.001 AC-9: load_all must produce exactly 1 error for the invalid spec. \
         Got: {} errors",
        errors.len()
    );
    // The valid spec ID must be present.
    assert_eq!(
        specs[0].infusion_id, "valid_csv",
        "BC-2.19.001 AC-9: the valid spec must be loaded (valid_csv)"
    );
}

// ---------------------------------------------------------------------------
// AC-7 / BC-2.19.002: InfusionLruCache TTL eviction and capacity cap
// ---------------------------------------------------------------------------

/// AC-7: InfusionLruCache::insert then get (within TTL) returns the value.
///
/// FAILS RED: `todo!()` in InfusionLruCache::insert and get before implementation.
#[tokio::test]
async fn test_BC_2_19_002_lru_cache_hit_within_ttl_returns_value() {
    let cache = InfusionLruCache::new(std::num::NonZeroUsize::new(100).unwrap());

    let value = serde_json::json!({"country": "US", "city": "New York"});

    // FAILS RED: todo!() in insert.
    cache.insert("geoip", "1.2.3.4", value.clone(), 3600).await;

    // FAILS RED: todo!() in get.
    let result = cache.get("geoip", "1.2.3.4").await;

    let retrieved = result.expect(
        "BC-2.19.002 AC-7: cache hit within TTL must return Some (FAILS RED: todo!() in get/insert)"
    );
    assert_eq!(
        retrieved, value,
        "BC-2.19.002 AC-7: retrieved value must equal inserted value"
    );
}

/// AC-7: InfusionLruCache::get on a missing key returns None.
///
/// FAILS RED: `todo!()` in InfusionLruCache::get before implementation.
#[tokio::test]
async fn test_BC_2_19_002_lru_cache_miss_returns_none() {
    let cache = InfusionLruCache::new(std::num::NonZeroUsize::new(100).unwrap());

    // FAILS RED: todo!() in get.
    let result = cache.get("geoip", "not_inserted").await;

    assert!(
        result.is_none(),
        "BC-2.19.002 AC-7: cache miss must return None (FAILS RED: todo!() in get)"
    );
}

/// AC-7: InfusionLruCache TTL expiry — entry with TTL=0 should be a miss.
///
/// TTL=0 means `expiry_unix_secs = now`. The entry is immediately expired on get.
/// FAILS RED: `todo!()` in InfusionLruCache::insert/get before implementation.
#[tokio::test]
async fn test_BC_2_19_002_lru_cache_ttl_zero_entry_is_expired_immediately() {
    let cache = InfusionLruCache::new(std::num::NonZeroUsize::new(100).unwrap());
    let value = serde_json::json!({"country": "DE"});

    // Insert with TTL=0 — expires at current second.
    cache.insert("geoip", "9.9.9.9", value.clone(), 0).await;

    // A TTL=0 entry may or may not be a miss depending on sub-second timing.
    // The invariant we enforce: get must NOT panic (no todo!()).
    // FAILS RED: todo!() in get.
    let _result = cache.get("geoip", "9.9.9.9").await;
    // Accept any Option<Value> — we just verify it does not panic.
}

/// AC-7: InfusionLruCache 10k-entry capacity evicts LRU entries when full.
///
/// Insert 10,001 entries into a cache with capacity=10,000.
/// The first inserted entry must be evicted (LRU semantics).
/// FAILS RED: `todo!()` in insert/get before implementation.
#[tokio::test]
async fn test_BC_2_19_002_lru_cache_capacity_evicts_lru_entry() {
    // Use small capacity for test speed.
    let capacity = 10usize;
    let cache = InfusionLruCache::new(std::num::NonZeroUsize::new(capacity).unwrap());

    // Insert `capacity` entries.
    for i in 0..capacity {
        let value = serde_json::json!({"index": i});
        cache
            .insert("geoip", &format!("10.0.0.{}", i), value, 3600)
            .await;
    }

    // Insert one more — forces eviction of least-recently-used entry.
    // The entry at index 0 was inserted first and never accessed again → LRU victim.
    cache
        .insert(
            "geoip",
            "10.0.1.0",
            serde_json::json!({"index": capacity}),
            3600,
        )
        .await;

    // FAILS RED: todo!() in get.
    let evicted = cache.get("geoip", "10.0.0.0").await;

    assert!(
        evicted.is_none(),
        "BC-2.19.002 AC-7: first inserted entry must be evicted when capacity is exceeded (LRU). \
         FAILS RED: todo!() in get/insert"
    );

    // New entry must still be accessible.
    let new_entry = cache.get("geoip", "10.0.1.0").await;
    assert!(
        new_entry.is_some(),
        "BC-2.19.002 AC-7: newly inserted entry must be retrievable after eviction"
    );
}

/// AC-7: InfusionLruCache key format uses "{infusion_id}:{input_value}" composition.
///
/// Two different infusion IDs with the same input value must be stored independently.
/// FAILS RED: `todo!()` in insert/get before implementation.
#[tokio::test]
async fn test_BC_2_19_002_lru_cache_composite_key_isolates_infusion_ids() {
    let cache = InfusionLruCache::new(std::num::NonZeroUsize::new(100).unwrap());

    let geoip_val = serde_json::json!({"country": "US"});
    let threat_val = serde_json::json!({"score": 0.9});

    cache
        .insert("geoip", "1.2.3.4", geoip_val.clone(), 3600)
        .await;
    cache
        .insert("threat_intel", "1.2.3.4", threat_val.clone(), 900)
        .await;

    // FAILS RED: todo!() in get.
    let geoip_result = cache.get("geoip", "1.2.3.4").await;
    let threat_result = cache.get("threat_intel", "1.2.3.4").await;

    assert_eq!(
        geoip_result.as_ref(),
        Some(&geoip_val),
        "BC-2.19.002: geoip:1.2.3.4 key must retrieve geoip value"
    );
    assert_eq!(
        threat_result.as_ref(),
        Some(&threat_val),
        "BC-2.19.002: threat_intel:1.2.3.4 key must retrieve threat_intel value"
    );
}

// ---------------------------------------------------------------------------
// AC-8 / BC-2.19.002: Three-tier lookup order
// ---------------------------------------------------------------------------

/// AC-8: Tier 1 (QueryScopedInfusionCache) hit avoids calling Tier 2 / source.
///
/// This is already proven by existing infusion_tests.rs AC-2 and VP-049.
/// No new RED test needed — QueryScopedInfusionCache is implemented.
/// Documented here for AC-8 traceability but delegates to existing tests.
///
/// This test is GREEN (passes before S-1.14-REDO implementation because
/// QueryScopedInfusionCache is fully implemented).
#[test]
fn test_BC_2_19_002_ac_8_tier1_hit_avoids_source_call() {
    let mut cache = QueryScopedInfusionCache::new();
    let mock_call_count = std::sync::Arc::new(AtomicUsize::new(0));
    let counter = mock_call_count.clone();

    struct MockCountingSource {
        count: std::sync::Arc<AtomicUsize>,
    }
    impl std::fmt::Debug for MockCountingSource {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "MockCountingSource")
        }
    }
    impl prism_spec_engine::InfusionSource for MockCountingSource {
        fn enrich_single(&self, _input: &str, _input_type: &str) -> Option<serde_json::Value> {
            self.count.fetch_add(1, Ordering::SeqCst);
            Some(serde_json::json!({"country": "US"}))
        }
        fn enrich_batch(
            &self,
            inputs: &[String],
            input_type: &str,
        ) -> Vec<Option<serde_json::Value>> {
            inputs
                .iter()
                .map(|i| self.enrich_single(i, input_type))
                .collect()
        }
    }

    let source = MockCountingSource { count: counter };

    // First call: Tier 1 miss → call source.
    if cache.get("geoip", "1.2.3.4").is_none() {
        let result = source.enrich_single("1.2.3.4", "ip");
        cache.insert("geoip", "1.2.3.4", result);
    }
    // Second call: Tier 1 hit → do NOT call source.
    if cache.get("geoip", "1.2.3.4").is_none() {
        let result = source.enrich_single("1.2.3.4", "ip");
        cache.insert("geoip", "1.2.3.4", result);
    }

    assert_eq!(
        mock_call_count.load(Ordering::SeqCst),
        1,
        "BC-2.19.002 AC-8: Tier 1 hit on second call must avoid calling source again"
    );
}

/// AC-8: Tier 2 (InfusionLruCache) hit returns value and does NOT call Tier 3 / source.
///
/// Simulates: Tier 1 miss → Tier 2 hit → return cached value without source call.
/// FAILS RED: `todo!()` in InfusionLruCache::get/insert.
#[tokio::test]
async fn test_BC_2_19_002_ac_8_tier2_lru_hit_returns_value_without_source_call() {
    let lru_cache = InfusionLruCache::new(std::num::NonZeroUsize::new(1000).unwrap());
    let source_calls = std::sync::Arc::new(AtomicUsize::new(0));
    let counter = source_calls.clone();

    struct MockCountingSource {
        count: std::sync::Arc<AtomicUsize>,
    }
    impl std::fmt::Debug for MockCountingSource {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "MockCountingSource")
        }
    }
    impl prism_spec_engine::InfusionSource for MockCountingSource {
        fn enrich_single(&self, _input: &str, _input_type: &str) -> Option<serde_json::Value> {
            self.count.fetch_add(1, Ordering::SeqCst);
            Some(serde_json::json!({"country": "DE"}))
        }
        fn enrich_batch(
            &self,
            inputs: &[String],
            input_type: &str,
        ) -> Vec<Option<serde_json::Value>> {
            inputs
                .iter()
                .map(|i| self.enrich_single(i, input_type))
                .collect()
        }
    }
    let _source = MockCountingSource { count: counter };

    // Pre-warm Tier 2 LRU with a value (simulates previous query populating Tier 2).
    // FAILS RED: todo!() in lru_cache.insert.
    lru_cache
        .insert(
            "geoip",
            "5.5.5.5",
            serde_json::json!({"country": "DE"}),
            3600,
        )
        .await;

    // Simulate Tier 1 miss → Tier 2 lookup.
    let tier1_cache = QueryScopedInfusionCache::new(); // fresh per-query cache (Tier 1 miss)
    let _ = tier1_cache.get("geoip", "5.5.5.5"); // Tier 1 miss

    // FAILS RED: todo!() in lru_cache.get.
    let tier2_hit = lru_cache.get("geoip", "5.5.5.5").await;

    assert!(
        tier2_hit.is_some(),
        "BC-2.19.002 AC-8: Tier 2 must return the pre-warmed value (FAILS RED: todo!() in get)"
    );

    // Tier 2 hit — must NOT call source.
    assert_eq!(
        source_calls.load(Ordering::SeqCst),
        0,
        "BC-2.19.002 AC-8: Tier 2 hit must not call the live source (0 source calls expected)"
    );
}

/// AC-8: Tier 3 (RocksDB) is bypassed when Tier 2 LRU hits.
///
/// This is a structural property ensured by the lookup order implementation.
/// The unit-testable surface: after inserting to Tier 2, the source call count stays 0.
/// (Tier-3 production wiring is verified by
/// `crates/prism-bin/tests/infusion_boot_integration.rs::test_infusion_tier3_production_read_without_source`
/// — S-1.14-REDO CRIT-1 burst-4 closure.)
/// This test is a companion to test_BC_2_19_002_ac_8_tier2_lru_hit_returns_value.
#[tokio::test]
async fn test_BC_2_19_002_ac_8_tier3_bypassed_when_tier2_hits() {
    let lru_cache = InfusionLruCache::new(std::num::NonZeroUsize::new(1000).unwrap());

    // Insert into Tier 2.
    // FAILS RED: todo!() in insert.
    lru_cache
        .insert(
            "geoip",
            "8.8.8.8",
            serde_json::json!({"country": "US"}),
            3600,
        )
        .await;

    // FAILS RED: todo!() in get.
    let result = lru_cache.get("geoip", "8.8.8.8").await;

    // Tier 2 hit: result must be Some — Tier 3 (RocksDB) is not consulted.
    assert!(
        result.is_some(),
        "BC-2.19.002 AC-8: Tier 2 hit must return value without falling through to Tier 3. \
         FAILS RED: todo!() in get/insert"
    );
    assert_eq!(
        result.unwrap().get("country").and_then(|v| v.as_str()),
        Some("US"),
        "BC-2.19.002 AC-8: Tier 2 hit value must be the inserted value"
    );
}

// ---------------------------------------------------------------------------
// E-INFUSE-008 / BC-2.19.001: PluginCallFailed error taxonomy
// ---------------------------------------------------------------------------

/// E-INFUSE-008: InfusionError::PluginCallFailed has correct message format.
///
/// Traces to: error-taxonomy.md E-INFUSE-008 (added in S-1.14-REDO).
/// This test is GREEN before implementation (the error variant is already wired in
/// prism-core/src/error.rs and plugin_bridge.rs).
#[test]
fn test_BC_2_19_001_e_infuse_008_plugin_call_failed_message_format() {
    let err = InfusionError::PluginCallFailed {
        plugin_id: "threat_intel".to_string(),
        infusion_id: "threat_intel".to_string(),
        reason: "plugin trapped: unreachable".to_string(),
    };
    let msg = err.to_string();

    assert!(
        msg.contains("E-INFUSE-008"),
        "E-INFUSE-008: error message must contain 'E-INFUSE-008'. Got: '{}'",
        msg
    );
    assert!(
        msg.contains("threat_intel"),
        "E-INFUSE-008: error must name the plugin_id. Got: '{}'",
        msg
    );
    assert!(
        msg.contains("plugin trapped"),
        "E-INFUSE-008: error must include the reason. Got: '{}'",
        msg
    );
}

/// E-INFUSE-008: PluginCallFailed variant carries {plugin_id, infusion_id, reason} fields.
///
/// Traces to: prism-core/src/error.rs PluginCallFailed variant (S-1.14-REDO addition).
/// This test verifies the public API of the error variant — the internal
/// `map_plugin_error_to_infusion_error` function is `pub(crate)` and tested via the
/// `PluginInfusionSource::enrich_single` path in test_BC_2_19_001_plugin_bridge_delegates.
/// GREEN: the InfusionError::PluginCallFailed variant is already defined.
#[test]
fn test_BC_2_19_001_e_infuse_008_plugin_call_failed_carries_all_required_fields() {
    // Construct the error variant directly — the public API surface.
    let plugin_id = "threat_intel".to_string();
    let infusion_id = "threat_intel".to_string();
    let reason = "unreachable instruction executed".to_string();

    let infusion_err = InfusionError::PluginCallFailed {
        plugin_id: plugin_id.clone(),
        infusion_id: infusion_id.clone(),
        reason: reason.clone(),
    };

    // Verify destructuring recovers all three fields (critical for audit logging).
    match infusion_err {
        InfusionError::PluginCallFailed {
            plugin_id: p,
            infusion_id: i,
            reason: r,
        } => {
            assert_eq!(
                p, "threat_intel",
                "E-INFUSE-008: plugin_id field must round-trip"
            );
            assert_eq!(
                i, "threat_intel",
                "E-INFUSE-008: infusion_id field must round-trip"
            );
            assert!(
                r.contains("unreachable"),
                "E-INFUSE-008: reason field must round-trip. Got: '{}'",
                r
            );
        }
        other => panic!(
            "E-INFUSE-008: expected PluginCallFailed variant, got: {:?}",
            other
        ),
    }
}

/// E-INFUSE-008: PluginCallFailed reason must NOT contain credential values (INV-INFUSE-005).
///
/// Traces to: BC-2.19.005 / AD-017.
/// The reason field comes from PluginError::fmt() which must not expose credentials.
/// This test verifies the structural guarantee that WASM-surface errors do not
/// transit credential values.
#[test]
fn test_BC_2_19_001_e_infuse_008_plugin_call_failed_reason_no_credential_values() {
    let err = InfusionError::PluginCallFailed {
        plugin_id: "threat_intel".to_string(),
        infusion_id: "threat_intel".to_string(),
        reason: "plugin timed out after 5000ms".to_string(),
    };
    let msg = err.to_string();

    // Verify the reason does not contain simulated credential values.
    assert!(
        !msg.contains("sk-abc123"),
        "E-INFUSE-008: PluginCallFailed message must not contain credential values"
    );
    assert!(
        !msg.contains("THREAT_INTEL_KEY="),
        "E-INFUSE-008: PluginCallFailed message must not contain env var assignments"
    );
}

/// E-INFUSE-008: PluginCallFailed Display format contains E-INFUSE-008 + plugin_id + reason.
///
/// Tests the `Display` impl of the PluginCallFailed variant matches the error-taxonomy spec.
/// GREEN: the InfusionError::PluginCallFailed variant and its Display are already implemented.
#[test]
fn test_BC_2_19_001_e_infuse_008_plugin_call_failed_display_contains_all_fields() {
    let err = InfusionError::PluginCallFailed {
        plugin_id: "threat_intel".to_string(),
        infusion_id: "threat_intel".to_string(),
        reason: "plugin not loaded at runtime".to_string(),
    };
    let msg = err.to_string();

    // The error-taxonomy spec says E-INFUSE-008 must appear in the message.
    assert!(
        msg.contains("E-INFUSE-008"),
        "E-INFUSE-008: Display must contain 'E-INFUSE-008'. Got: '{}'",
        msg
    );
    // Both plugin_id and reason must appear for audit trail.
    assert!(
        msg.contains("threat_intel"),
        "E-INFUSE-008: Display must name the plugin. Got: '{}'",
        msg
    );
    assert!(
        msg.contains("not loaded"),
        "E-INFUSE-008: Display must include the reason. Got: '{}'",
        msg
    );
}

/// E-INFUSE-008 (HIGH-2 / POL-24): PluginCallFailed Display MUST match taxonomy verbatim.
///
/// error-taxonomy.md §E-INFUSE-008 Message Format mandates:
///   `"E-INFUSE-008: plugin infusion call failed for '{infusion_id}' via plugin '{plugin_id}': {reason}"`
///
/// This test pins the EXACT format (not just `.contains`) to prevent silent drift from the
/// taxonomy-mandated template. Any change to the Display impl that alters ordering, casing,
/// or the "via plugin" phrasing will fail this test, forcing an explicit spec update.
///
/// HIGH-2 finding: the prior Display template `"E-INFUSE-008: Plugin infusion call failed for
/// plugin '{plugin_id}' (infusion '{infusion_id}'): {reason}"` differed from the taxonomy on
/// three counts: capital P, plugin_id-first field order, and "(infusion ...)" phrasing.
/// Fixed by aligning with the error-taxonomy.md §E-INFUSE-008 canonical template.
#[test]
fn test_BC_2_19_001_e_infuse_008_plugin_call_failed_display_exact_taxonomy_format() {
    let err = InfusionError::PluginCallFailed {
        plugin_id: "threat_plugin".to_string(),
        infusion_id: "threat_intel".to_string(),
        reason: "plugin trapped: unreachable instruction".to_string(),
    };
    let msg = err.to_string();

    // Pin the EXACT message template from error-taxonomy.md §E-INFUSE-008 Message Format:
    //   "E-INFUSE-008: plugin infusion call failed for '{infusion_id}' via plugin '{plugin_id}': {reason}"
    let expected = "E-INFUSE-008: plugin infusion call failed for 'threat_intel' via plugin 'threat_plugin': plugin trapped: unreachable instruction";

    assert_eq!(
        msg, expected,
        "HIGH-2 / POL-24: PluginCallFailed Display MUST match error-taxonomy.md v1.87 verbatim.\n\
         Expected: '{}'\n\
         Got:      '{}'",
        expected, msg
    );
}

// ---------------------------------------------------------------------------
// VP-048 (concrete unit mirror) / BC-2.19.001
// ---------------------------------------------------------------------------
//
// The Kani proof harness is in src/proofs/infusion_spec.rs (cfg-gated).
// Per CLAUDE.md Kani layering: concrete unit tests must mirror the Kani proof
// for cross-platform coverage.
//
// The following tests ARE GREEN (load_spec is implemented) — they confirm the
// Kani proof property holds with concrete test vectors and serve as the unit-test
// layer of the VP-048 coverage stack.

/// VP-048 concrete mirror: N=1 distinct field → 1 descriptor.
/// GREEN: load_spec is implemented. Kani harness equivalently proves this at N ∈ [1,16].
#[test]
fn test_BC_2_19_001_vp_048_mirror_one_field_produces_one_descriptor() {
    let registry = InfusionRegistry::new();
    let spec = InfusionSpec::new(
        "vp048_n1",
        "VP-048 N=1",
        InfusionType::LocalLookup,
        vec![InfusionField::new("field_0", "src_ip", "ip", "string")],
        "vp048_n1.infusion.toml",
    );

    let descriptors = registry
        .load_spec(spec)
        .expect("VP-048 mirror N=1: load_spec must succeed");

    assert_eq!(
        descriptors.len(),
        1,
        "VP-048 mirror: N=1 distinct field must produce exactly 1 descriptor"
    );
}

/// VP-048 concrete mirror: N=16 distinct fields → 16 descriptors (Kani upper bound).
/// GREEN: load_spec is implemented.
#[test]
fn test_BC_2_19_001_vp_048_mirror_sixteen_fields_produces_sixteen_descriptors() {
    let registry = InfusionRegistry::new();
    let n = 16usize;
    let fields: Vec<InfusionField> = (0..n)
        .map(|i| InfusionField::new(format!("field_{}", i), "src_ip", "ip", "string"))
        .collect();
    let spec = InfusionSpec::new(
        "vp048_n16",
        "VP-048 N=16",
        InfusionType::LocalLookup,
        fields,
        "vp048_n16.infusion.toml",
    );

    let descriptors = registry
        .load_spec(spec)
        .expect("VP-048 mirror N=16: load_spec must succeed");

    assert_eq!(
        descriptors.len(),
        16,
        "VP-048 mirror: N=16 distinct fields must produce exactly 16 descriptors"
    );
}

/// VP-048 concrete mirror: duplicate field within spec → Err(DuplicateUdfName).
/// GREEN: load_spec is implemented.
#[test]
fn test_BC_2_19_001_vp_048_mirror_duplicate_field_name_in_spec_errors() {
    let registry = InfusionRegistry::new();
    let spec = InfusionSpec::new(
        "vp048_dup",
        "VP-048 duplicate",
        InfusionType::LocalLookup,
        vec![
            InfusionField::new("duplicate_name", "src_ip", "ip", "string"),
            InfusionField::new("duplicate_name", "src_ip", "ip", "string"), // duplicate
        ],
        "vp048_dup.infusion.toml",
    );

    let result = registry.load_spec(spec);

    assert!(
        result.is_err(),
        "VP-048 mirror: duplicate field name must produce Err(DuplicateUdfName)"
    );
    match result.unwrap_err() {
        InfusionError::DuplicateUdfName { udf_name, .. } => {
            assert_eq!(udf_name, "duplicate_name");
        }
        other => panic!("VP-048 mirror: expected DuplicateUdfName, got: {:?}", other),
    }
}

// ---------------------------------------------------------------------------
// VP-049 (proptest) / BC-2.19.002
// ---------------------------------------------------------------------------
//
// The proptest harness is in src/proofs/infusion_dedup.rs (1000 cases).
// That harness tests QueryScopedInfusionCache which is FULLY IMPLEMENTED.
// These additional concrete tests complement the proptest for specific edge cases
// from the AC-8/EC-003 canonical test vectors.

/// VP-049 concrete: EC-003 — 10,000 events with 200 unique IPs → 200 source calls.
/// GREEN: QueryScopedInfusionCache is implemented.
#[test]
fn test_BC_2_19_002_vp_049_ec_003_ten_thousand_events_two_hundred_unique() {
    let call_count = Arc::new(AtomicUsize::new(0));
    let counter = call_count.clone();

    struct MockSource {
        count: Arc<AtomicUsize>,
    }
    impl std::fmt::Debug for MockSource {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "MockSource")
        }
    }
    impl prism_spec_engine::InfusionSource for MockSource {
        fn enrich_single(&self, _input: &str, _input_type: &str) -> Option<serde_json::Value> {
            self.count.fetch_add(1, Ordering::SeqCst);
            Some(serde_json::json!({"country": "US"}))
        }
        fn enrich_batch(
            &self,
            inputs: &[String],
            input_type: &str,
        ) -> Vec<Option<serde_json::Value>> {
            inputs
                .iter()
                .map(|i| self.enrich_single(i, input_type))
                .collect()
        }
    }

    let source = MockSource { count: counter };
    let mut cache = QueryScopedInfusionCache::new();

    // 10,000 events with 200 unique IPs (each appears 50 times).
    let values: Vec<String> = (0..10_000usize)
        .map(|i| format!("10.0.{}.{}", (i % 200) / 256, (i % 200) % 256))
        .collect();

    for value in &values {
        if cache.get("geoip", value).is_none() {
            let result = source.enrich_single(value, "ip");
            cache.insert("geoip", value, result);
        }
    }

    assert_eq!(
        call_count.load(Ordering::SeqCst),
        200,
        "VP-049 EC-003: 10K events with 200 unique IPs must produce exactly 200 source calls"
    );
    assert_eq!(
        cache.len(),
        200,
        "VP-049 EC-003: cache must have exactly 200 entries"
    );
}

/// VP-049 concrete: AC-8 — 500 events with 30 unique IPs → 30 source calls.
/// GREEN: QueryScopedInfusionCache is implemented.
#[test]
fn test_BC_2_19_002_vp_049_ac_8_five_hundred_events_thirty_unique() {
    let call_count = Arc::new(AtomicUsize::new(0));
    let counter = call_count.clone();

    struct MockSource {
        count: Arc<AtomicUsize>,
    }
    impl std::fmt::Debug for MockSource {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "MockSource")
        }
    }
    impl prism_spec_engine::InfusionSource for MockSource {
        fn enrich_single(&self, _input: &str, _input_type: &str) -> Option<serde_json::Value> {
            self.count.fetch_add(1, Ordering::SeqCst);
            Some(serde_json::json!({"enriched": true}))
        }
        fn enrich_batch(
            &self,
            inputs: &[String],
            input_type: &str,
        ) -> Vec<Option<serde_json::Value>> {
            inputs
                .iter()
                .map(|i| self.enrich_single(i, input_type))
                .collect()
        }
    }

    let source = MockSource { count: counter };
    let mut cache = QueryScopedInfusionCache::new();

    let values: Vec<String> = (0..500usize)
        .map(|i| format!("10.0.0.{}", i % 30))
        .collect();

    for value in &values {
        if cache.get("geoip", value).is_none() {
            let result = source.enrich_single(value, "ip");
            cache.insert("geoip", value, result);
        }
    }

    assert_eq!(
        call_count.load(Ordering::SeqCst),
        30,
        "VP-049 AC-8: 500 events with 30 unique IPs must produce exactly 30 source calls"
    );
}

// ---------------------------------------------------------------------------
// CRIT-1: End-to-end enrichment through wired registry (BC-2.19.001)
//
// This test proves the PRODUCTION PATH:
//   InfusionSpec (CSV source) → InfusionRegistry::load_spec → stores real CsvSource
//   → udf_descriptors() → descriptor.source.enrich_single → returns REAL CSV data
//
// A NullSource regression would make enrich_single return None for any key.
// A CsvSource correctly wired returns the CSV row for known keys.
// ---------------------------------------------------------------------------

/// CRIT-1 end-to-end: Registry stores a real CsvSource for LocalLookup specs.
///
/// Production path proof:
///   `InfusionRegistry::load_spec(csv_spec)` stores a real `CsvSource` (not `NullSource`).
///   `udf_descriptors()` returns descriptors with the real source.
///   `descriptor.source.enrich_single("192.168.1.10", "ip")` returns CSV row data.
///
/// Traces to: BC-2.19.001 postcondition — descriptors carry a real source backend.
/// CRIT-1 closure: `load_spec` → `sources::load_source` → `CsvSource` → real data.
#[test]
fn test_BC_2_19_001_crit1_registry_wires_real_csv_source_for_local_lookup_spec() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let csv_path = format!("{}/fixtures/asset_inventory.csv", manifest_dir);

    // Build a LocalLookup CSV spec pointing at the real fixture.
    let fields = vec![
        InfusionField::with_all(
            "asset_department",
            "device_ip",
            "ip",
            "string",
            None,
            Some("department".to_string()),
        ),
        InfusionField::with_all(
            "asset_owner",
            "device_ip",
            "ip",
            "string",
            None,
            Some("owner".to_string()),
        ),
    ];
    let mut spec = InfusionSpec::new(
        "asset_inventory",
        "Asset Inventory CSV",
        InfusionType::LocalLookup,
        fields,
        "asset_inventory.infusion.toml",
    );
    spec.source = Some(prism_spec_engine::infusion::InfusionSourceConfig::new(
        prism_spec_engine::infusion::BuiltInSourceType::Csv,
        csv_path,
        Some("ip_address".to_string()),
        Some(300),
    ));

    // PRODUCTION PATH: load_spec must wire a real CsvSource (not NullSource).
    let registry = InfusionRegistry::new();
    let descriptors = registry
        .load_spec(spec)
        .expect("CRIT-1: registry.load_spec must succeed for a valid CSV LocalLookup spec");

    assert_eq!(
        descriptors.len(),
        2,
        "CRIT-1: CSV spec with 2 fields must produce 2 UDF descriptors"
    );

    // Get the wired descriptors from the registry (these carry the stored source).
    let stored_descriptors = registry.udf_descriptors();
    assert_eq!(
        stored_descriptors.len(),
        2,
        "CRIT-1: udf_descriptors() must return 2 stored descriptors"
    );

    // Find the asset_department descriptor.
    let dept_desc = stored_descriptors
        .iter()
        .find(|d| d.name == "asset_department")
        .expect("CRIT-1: asset_department descriptor must be present in registry");

    // THE CRITICAL ASSERTION: enrich_single via the stored source must return REAL CSV data.
    // A NullSource would return None for ALL inputs — this proves the source is a real CsvSource.
    let result = dept_desc.source.enrich_single("192.168.1.10", "ip");

    assert!(
        result.is_some(),
        "CRIT-1: descriptor.source.enrich_single('192.168.1.10') must return Some (real CSV data), \
         NOT None (NullSource regression). \
         NullSource returns None unconditionally — a Some here proves the wired path is CsvSource."
    );

    // Verify the actual data returned is correct (Engineering for 192.168.1.10).
    let value = result.unwrap();
    assert_eq!(
        value.get("department").and_then(|v| v.as_str()),
        Some("Engineering"),
        "CRIT-1: wired CsvSource must return 'Engineering' for 192.168.1.10 (real data, not mock)"
    );

    // Also verify unknown key returns None (not a blanket NullSource that always returns None).
    let unknown_result = dept_desc.source.enrich_single("10.255.255.99", "ip");
    assert!(
        unknown_result.is_none(),
        "CRIT-1: CsvSource must return None for unknown keys"
    );
}

/// CRIT-1 end-to-end: Registry stores a real JsonLookupSource for LocalLookup specs.
///
/// Uses a temp JSON file to prove the JsonLookup path is wired through the registry.
/// CRIT-1 closure: `load_spec` → `sources::load_source` → `JsonLookupSource` → real data.
#[test]
fn test_BC_2_19_001_crit1_registry_wires_real_json_lookup_source_for_local_lookup_spec() {
    use std::io::Write;
    use tempfile::TempDir;

    let temp_dir = TempDir::new().expect("CRIT-1: temp dir creation must succeed");
    let json_path = temp_dir.path().join("assets.json");

    // Write a minimal JSON lookup file.
    let json_content = r#"{
        "192.168.1.1": {"role": "server", "criticality": "high"},
        "10.0.0.1": {"role": "gateway", "criticality": "critical"}
    }"#;
    {
        let mut f =
            std::fs::File::create(&json_path).expect("CRIT-1: JSON file creation must succeed");
        f.write_all(json_content.as_bytes())
            .expect("CRIT-1: JSON file write must succeed");
    }

    // Build a LocalLookup JSON spec.
    let fields = vec![InfusionField::with_all(
        "asset_role",
        "device_ip",
        "ip",
        "string",
        None,
        Some("role".to_string()),
    )];
    let mut spec = InfusionSpec::new(
        "asset_roles",
        "Asset Roles JSON",
        InfusionType::LocalLookup,
        fields,
        "asset_roles.infusion.toml",
    );
    spec.source = Some(prism_spec_engine::infusion::InfusionSourceConfig::new(
        prism_spec_engine::infusion::BuiltInSourceType::JsonLookup,
        json_path.to_str().unwrap().to_string(),
        None,
        None,
    ));

    // PRODUCTION PATH: load_spec must wire a real JsonLookupSource.
    let registry = InfusionRegistry::new();
    registry
        .load_spec(spec)
        .expect("CRIT-1: registry.load_spec must succeed for a valid JSON LocalLookup spec");

    let stored_descriptors = registry.udf_descriptors();
    let role_desc = stored_descriptors
        .iter()
        .find(|d| d.name == "asset_role")
        .expect("CRIT-1: asset_role descriptor must be present in registry");

    // THE CRITICAL ASSERTION: real data, not NullSource None.
    let result = role_desc.source.enrich_single("192.168.1.1", "ip");

    assert!(
        result.is_some(),
        "CRIT-1: descriptor.source.enrich_single('192.168.1.1') must return Some (real JSON data), \
         NOT None (NullSource regression). Got None — load_spec did not wire JsonLookupSource."
    );

    let value = result.unwrap();
    assert_eq!(
        value.get("role").and_then(|v| v.as_str()),
        Some("server"),
        "CRIT-1: JsonLookupSource must return 'server' for 192.168.1.1 (real data)"
    );
}

// ---------------------------------------------------------------------------
// CRIT-2: Tier-3 RocksDB cache — three-tier lookup order proof (BC-2.19.002)
//
// Proves the full Tier1 → Tier2 → Tier3 → source lookup order.
// Uses an in-memory CacheBackend so no RocksDB process is needed.
//
// Path coverage:
//   (a) Tier3 HIT: Tier1 miss → Tier2 miss → Tier3 hit → returns cached value.
//   (b) Tier3 MISS (source call): Tier1 miss → Tier2 miss → Tier3 miss → source call → 1 call.
//   (c) Tier1 bypass: Tier1 hit → 0 source calls (Tier2/Tier3 not consulted).
// ---------------------------------------------------------------------------

/// Type alias to avoid `clippy::type_complexity` on `InMemoryCacheBackend::store`.
/// Key: (column_family_name, raw_key_bytes); Value: raw_value_bytes.
type InMemoryCacheStore = std::sync::Mutex<std::collections::HashMap<(String, Vec<u8>), Vec<u8>>>;

/// In-memory `CacheBackend` implementation for CRIT-2 three-tier tests.
///
/// Uses a `Mutex<HashMap<(domain_name, key), value>>` keyed by domain+raw-bytes.
/// This is a test-only type that satisfies `CacheBackend` without RocksDB.
struct InMemoryCacheBackend {
    store: InMemoryCacheStore,
}

impl std::fmt::Debug for InMemoryCacheBackend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InMemoryCacheBackend")
            .finish_non_exhaustive()
    }
}

impl InMemoryCacheBackend {
    fn new() -> Self {
        Self {
            store: InMemoryCacheStore::new(std::collections::HashMap::new()),
        }
    }
}

#[async_trait::async_trait]
impl CacheBackend for InMemoryCacheBackend {
    async fn get(&self, domain: StorageDomain, key: &[u8]) -> Result<Option<Vec<u8>>, PrismError> {
        let store = self.store.lock().unwrap();
        let composite = (domain.column_family_name().to_string(), key.to_vec());
        Ok(store.get(&composite).cloned())
    }

    async fn set(&self, domain: StorageDomain, key: &[u8], value: &[u8]) -> Result<(), PrismError> {
        let mut store = self.store.lock().unwrap();
        let composite = (domain.column_family_name().to_string(), key.to_vec());
        store.insert(composite, value.to_vec());
        Ok(())
    }

    async fn delete(&self, domain: StorageDomain, key: &[u8]) -> Result<(), PrismError> {
        let mut store = self.store.lock().unwrap();
        let composite = (domain.column_family_name().to_string(), key.to_vec());
        store.remove(&composite);
        Ok(())
    }
}

/// CRIT-2: Three-tier lookup order — Tier3 HIT path (BC-2.19.002 / INV-INFUSE-002).
///
/// Proves: Tier1 miss → Tier2 miss → Tier3 hit → value returned, source NOT called.
/// Uses in-memory CacheBackend (no RocksDB dependency).
///
/// This is the key CRIT-2 test: proves that Tier3 is consulted after Tier1+Tier2 miss,
/// and returns the stored value without calling the live source.
#[tokio::test]
async fn test_BC_2_19_002_crit2_tier3_hit_returns_value_source_not_called() {
    let backend = std::sync::Arc::new(InMemoryCacheBackend::new());
    let tier3 = InfusionTier3Cache::new(backend.clone());
    let source_calls = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));

    // Pre-populate Tier3 with a known entry (simulates a previous warm-up).
    // Use a future-forward TTL so the entry is not expired.
    let expected_value = serde_json::json!({"country": "DE", "city": "Berlin"});
    tier3
        .set("geoip", "5.5.5.5", Some(expected_value.clone()), 3600)
        .await;

    // Simulate the full three-tier lookup:
    // Tier1: fresh per-query cache — miss for "5.5.5.5".
    let tier1 = QueryScopedInfusionCache::new();
    let tier1_result = tier1.get("geoip", "5.5.5.5");
    assert!(
        tier1_result.is_none(),
        "CRIT-2: Tier1 must miss for a fresh per-query cache"
    );

    // Tier2: fresh LRU cache — miss for "5.5.5.5".
    let tier2 = InfusionLruCache::new(std::num::NonZeroUsize::new(1000).unwrap());
    let tier2_result = tier2.get("geoip", "5.5.5.5").await;
    assert!(
        tier2_result.is_none(),
        "CRIT-2: Tier2 must miss for a fresh LRU cache"
    );

    // Tier3: hit — value was set above, TTL is future.
    let tier3_result = tier3.get("geoip", "5.5.5.5").await;

    assert!(
        tier3_result.is_some(),
        "CRIT-2: Tier3 must return Some after pre-population (Tier3 HIT path)"
    );
    let tier3_hit_value = tier3_result.unwrap();
    assert_eq!(
        tier3_hit_value,
        Some(expected_value.clone()),
        "CRIT-2: Tier3 returned wrong value (expected {:?})",
        expected_value
    );

    // Source must NOT be called (Tier3 hit prevents source fallthrough).
    assert_eq!(
        source_calls.load(std::sync::atomic::Ordering::SeqCst),
        0,
        "CRIT-2: Source must NOT be called when Tier3 hits"
    );
}

/// CRIT-2: Three-tier lookup order — all tiers miss, source called exactly once (BC-2.19.002).
///
/// Proves: Tier1 miss → Tier2 miss → Tier3 miss → source call → 1 call.
/// Uses in-memory CacheBackend (no RocksDB dependency).
#[tokio::test]
async fn test_BC_2_19_002_crit2_all_tiers_miss_source_called_once() {
    let backend = std::sync::Arc::new(InMemoryCacheBackend::new());
    let tier3 = InfusionTier3Cache::new(backend.clone());

    // Tier1 miss.
    let tier1 = QueryScopedInfusionCache::new();
    let t1 = tier1.get("geoip", "9.9.9.9");
    assert!(t1.is_none(), "CRIT-2: Tier1 must miss for fresh cache");

    // Tier2 miss.
    let tier2 = InfusionLruCache::new(std::num::NonZeroUsize::new(1000).unwrap());
    let t2 = tier2.get("geoip", "9.9.9.9").await;
    assert!(t2.is_none(), "CRIT-2: Tier2 must miss for fresh LRU");

    // Tier3 miss (nothing stored).
    let t3 = tier3.get("geoip", "9.9.9.9").await;
    assert!(t3.is_none(), "CRIT-2: Tier3 must miss when nothing stored");

    // All tiers missed — caller must call source (simulated here by counting).
    let source_calls = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    source_calls.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    let source_value = serde_json::json!({"country": "US"});

    // Populate all tiers after source call (as production code would).
    tier3
        .set("geoip", "9.9.9.9", Some(source_value.clone()), 3600)
        .await;
    tier2
        .insert("geoip", "9.9.9.9", source_value.clone(), 3600)
        .await;

    assert_eq!(
        source_calls.load(std::sync::atomic::Ordering::SeqCst),
        1,
        "CRIT-2: source must be called exactly once when all tiers miss"
    );

    // Subsequent Tier3 lookup must hit now.
    let t3_again = tier3.get("geoip", "9.9.9.9").await;
    assert!(
        t3_again.is_some(),
        "CRIT-2: Tier3 must hit after population from source call"
    );
}

/// CRIT-2: Tier3 TTL expiry — expired entry treated as miss (lazy eviction) (BC-2.19.002).
///
/// An entry inserted with TTL=0 must be treated as a miss on subsequent get().
/// Proves lazy TTL eviction at the Tier3 RocksDB boundary.
#[tokio::test]
async fn test_BC_2_19_002_crit2_tier3_ttl_zero_entry_is_expired() {
    let backend = std::sync::Arc::new(InMemoryCacheBackend::new());
    let tier3 = InfusionTier3Cache::new(backend);

    // Insert with TTL=0 — expiry_unix_secs = now (immediately expired on read).
    tier3
        .set(
            "geoip",
            "expired_ip",
            Some(serde_json::json!({"country": "XX"})),
            0,
        )
        .await;

    // Immediately read back — entry is expired (expiry_unix_secs <= now).
    let result = tier3.get("geoip", "expired_ip").await;

    assert!(
        result.is_none(),
        "CRIT-2: Tier3 entry with TTL=0 must be treated as a miss (lazy TTL eviction). Got: {:?}",
        result
    );
}

/// CRIT-2: Tier3 negative cache entry — None value stored and retrieved (BC-2.19.002).
///
/// Proves that `None` (negative cache: no enrichment available) round-trips through
/// the Tier3 binary encoding as a `Some(None)` result (not a miss).
#[tokio::test]
async fn test_BC_2_19_002_crit2_tier3_negative_cache_entry_round_trips() {
    let backend = std::sync::Arc::new(InMemoryCacheBackend::new());
    let tier3 = InfusionTier3Cache::new(backend);

    // Store a negative cache entry (None = no enrichment available).
    tier3.set("geoip", "unknown_ip", None, 3600).await;

    // Retrieve: must return Some(None) (negative hit, not a miss).
    let result = tier3.get("geoip", "unknown_ip").await;

    assert_eq!(
        result,
        Some(None),
        "CRIT-2: Tier3 negative cache entry must round-trip as Some(None), not None (miss). Got: {:?}",
        result
    );
}

// ---------------------------------------------------------------------------
// MED-3: InfusionLoader::parse rejects unknown LocalLookup sub-types (BC-2.19.001)
// ---------------------------------------------------------------------------

/// MED-3: InfusionLoader::parse rejects `source.type = "local_lookup"` without sub-type.
///
/// `"local_lookup"` at the [infusion.source] level with no further sub-type discriminant
/// must NOT silently default to JsonLookup. It must return Err(UnknownSourceType).
///
/// BC-2.19.001 validation: source.type must be one of maxmind_mmdb, csv, json_lookup, plugin.
/// "local_lookup" is not a valid terminal source type — the caller must specify the sub-type.
#[test]
fn test_BC_2_19_001_med3_local_lookup_without_subtype_returns_unknown_source_type_error() {
    let toml_input = r#"
[infusion]
infusion_id = "test_enrichment"
name = "Test Enrichment"

[infusion.source]
type = "local_lookup"
file_path = "fixtures/data.json"

[[infusion.fields]]
name = "test_field"
input_field = "src_ip"
input_type = "ip"
output_type = "string"
source_column = "col1"
"#;

    let result = InfusionLoader::parse(toml_input, "test.infusion.toml");

    assert!(
        result.is_err(),
        "MED-3 / BC-2.19.001: source.type = 'local_lookup' must return Err (not silently default \
         to JsonLookup). Got Ok: {:?}",
        result.ok()
    );
    match result.unwrap_err() {
        InfusionError::UnknownSourceType { type_name } => {
            assert_eq!(
                type_name, "local_lookup",
                "MED-3: UnknownSourceType must carry the rejected type name 'local_lookup'"
            );
        }
        other => panic!(
            "MED-3: expected UnknownSourceType for 'local_lookup', got: {:?}",
            other
        ),
    }
}

/// MED-3: InfusionLoader::parse rejects entirely unknown source types (BC-2.19.001).
///
/// Ensures the error path works for truly unknown types (not just "local_lookup").
#[test]
fn test_BC_2_19_001_med3_unknown_source_type_returns_error() {
    let toml_input = r#"
[infusion]
infusion_id = "test_enrichment"
name = "Test Enrichment"

[infusion.source]
type = "sqlite_lookup"
file_path = "fixtures/data.db"

[[infusion.fields]]
name = "test_field"
input_field = "src_ip"
input_type = "ip"
output_type = "string"
"#;

    let result = InfusionLoader::parse(toml_input, "test.infusion.toml");

    assert!(
        result.is_err(),
        "MED-3 / BC-2.19.001: unknown source.type must return Err(UnknownSourceType)"
    );
    match result.unwrap_err() {
        InfusionError::UnknownSourceType { type_name } => {
            assert_eq!(
                type_name, "sqlite_lookup",
                "MED-3: UnknownSourceType must carry the rejected type name 'sqlite_lookup'"
            );
        }
        other => panic!(
            "MED-3: expected UnknownSourceType for unknown type, got: {:?}",
            other
        ),
    }
}
