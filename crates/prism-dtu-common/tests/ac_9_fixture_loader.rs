// AC-9: load_fixture returns parsed JSON for an existing fixture file.
//
// Expected failure mode: todo!("implement fixture loader per AC-6") panics at runtime.
//
// Test setup: a fixture file is written to a temp directory that acts as the
// "crate_dir". The fixture path is {crate_dir}/fixtures/{name}.json.

use prism_dtu_common::load_fixture;
use std::fs;
use tempfile::TempDir;

fn write_fixture(dir: &TempDir, name: &str, content: &str) -> String {
    let fixtures_dir = dir.path().join("fixtures");
    fs::create_dir_all(&fixtures_dir).expect("create fixtures dir");
    let path = fixtures_dir.join(format!("{name}.json"));
    fs::write(&path, content).expect("write fixture file");
    dir.path().to_str().expect("temp dir path is valid UTF-8").to_owned()
}

#[test]
fn ac_9_load_fixture_returns_parsed_json_for_existing_file() {
    let dir = TempDir::new().expect("create temp dir");
    let payload = r#"{"devices": [], "total": 0}"#;
    let crate_dir = write_fixture(&dir, "devices-page1", payload);

    let value = load_fixture(&crate_dir, "devices-page1");

    assert!(
        value.get("devices").is_some(),
        "AC-9: loaded fixture must contain 'devices' key"
    );
    assert_eq!(value["total"], 0, "AC-9: 'total' field must equal 0");
}

// EC-003 test for missing-file panic is NOT included here because load_fixture
// is currently todo!() — it panics regardless of whether the file exists.
// A #[should_panic] test against a todo!() stub would be tautological (it would
// pass vacuously). This test will be added when the implementation is in place
// and can distinguish "missing file" panics from "not implemented" panics.
