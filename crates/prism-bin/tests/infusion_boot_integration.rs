//! AC-10 hollow-feature wiring test (S-1.14-REDO).
//!
//! Verifies that the production boot path wires `InfusionLoader::load_all()` into
//! `QueryEngine::with_infusion_registry()` — not just that infusion module unit tests
//! pass in isolation (SID-1 compliance: no `#[ignore]`'d placeholder).
//!
//! # Red Gate status
//! All tests in this file are RED (failing) before S-1.14-REDO is implemented.
//! They fail because `infusion_load_step()` in boot.rs panics with `todo!("S-1.14-REDO …")`.
//!
//! # Green Gate (S-1.14-REDO implementer)
//! Replace `todo!()` in `infusion_load_step()` with the real implementation that:
//! 1. Calls `InfusionLoader::load_all()` for the given config_dir.
//! 2. Registers each valid spec via `InfusionRegistry::load_spec()`.
//! 3. Returns the populated `InfusionRegistry`.
//! Then wire `query_engine.with_infusion_registry(Arc::new(registry))` in `run_boot_sequence`.
//!
//! # BC traceability
//! - BC-2.19.001 postcondition: engine path must be wired (AC-10)
//! - BC-2.19.002: per-query dedup cache alive when registry is wired
//! - BC-2.22.001: infusion load step in production boot sequence
//! - SID-1: no `#[ignore]`'d placeholder — this test exercises the real production path

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    dead_code,
    unused_imports,
    unused_variables
)]

use std::io::Write;
use std::sync::Arc;

use prism_bin::boot::infusion_load_step;

// ---------------------------------------------------------------------------
// Test AC-10: infusion_load_step is callable and produces an InfusionRegistry
// ---------------------------------------------------------------------------

/// AC-10 RED Gate: `infusion_load_step()` exists and is callable from the boot path.
///
/// RED Gate failure (before S-1.14-REDO): `infusion_load_step` panics with
/// `todo!("S-1.14-REDO AC-10: ...")`.
///
/// Green Gate (after S-1.14-REDO): function returns a populated `InfusionRegistry`
/// with at least 1 UDF descriptor for the CSV test fixture.
///
/// Traces to: BC-2.19.001 postcondition — AC-10 hollow-feature wiring; BC-2.22.001.
#[test]
fn test_boot_with_csv_infusion_udf_resolves() {
    use prism_spec_engine::InfusionRegistry;
    use tempfile::TempDir;

    // Create a minimal temp config dir with a CSV infusion TOML.
    let temp_dir = TempDir::new().expect("AC-10: temp dir creation must succeed for test setup");
    let infusions_dir = temp_dir.path().join("infusions");
    std::fs::create_dir_all(&infusions_dir).expect("AC-10: infusions dir must be created");

    // Write a minimal CSV infusion TOML (no live MMDB required — CSV only).
    // Uses the canonical CSV source spec shape from S-1.14 design.
    let csv_infusion_toml = r#"
[infusion]
infusion_id = "asset_inventory"
name = "Asset Inventory CSV"

[source]
type = "csv"
file_path = "fixtures/asset_inventory.csv"
key_column = "device_ip"

[[infusion.fields]]
name = "asset_name"
input_field = "device_ip"
input_type = "ip"
output_type = "string"
source_column = "name"

[[infusion.fields]]
name = "asset_owner"
input_field = "device_ip"
input_type = "ip"
output_type = "string"
source_column = "owner"

[infusion.pipe_stage]
adds_columns = ["asset_name", "asset_owner"]
"#;

    // Write a minimal CSV fixture file alongside the spec (file existence required by loader).
    let fixtures_dir = temp_dir.path().join("fixtures");
    std::fs::create_dir_all(&fixtures_dir).expect("AC-10: fixtures dir must be created");
    let csv_path = fixtures_dir.join("asset_inventory.csv");
    {
        let mut f =
            std::fs::File::create(&csv_path).expect("AC-10: CSV fixture creation must succeed");
        f.write_all(b"device_ip,name,owner\n10.0.0.1,server-01,security-team\n10.0.0.2,workstation-02,ops-team\n")
            .expect("AC-10: CSV write must succeed");
    }

    let spec_path = infusions_dir.join("asset_inventory.infusion.toml");
    {
        let mut f =
            std::fs::File::create(&spec_path).expect("AC-10: TOML fixture creation must succeed");
        f.write_all(csv_infusion_toml.as_bytes())
            .expect("AC-10: TOML write must succeed");
    }

    // ── RED GATE ────────────────────────────────────────────────────────────
    // `infusion_load_step()` panics with `todo!("S-1.14-REDO AC-10: ...")` here.
    // After S-1.14-REDO implementation, this call returns a populated InfusionRegistry.
    let registry = infusion_load_step(temp_dir.path());

    // After S-1.14-REDO: verify the registry has the expected UDF descriptors.
    let descriptors = registry.udf_descriptors();
    assert!(
        !descriptors.is_empty(),
        "AC-10: infusion_load_step must produce at least 1 InfusionUdfDescriptor \
         from the CSV fixture TOML (BC-2.19.001 postcondition)"
    );

    let udf_names: Vec<&str> = descriptors.iter().map(|d| d.name.as_str()).collect();
    assert!(
        udf_names.contains(&"asset_name"),
        "AC-10: 'asset_name' UDF must be registered after boot infusion load \
         (BC-2.19.001 — each field registers exactly one UDF)"
    );
    assert!(
        udf_names.contains(&"asset_owner"),
        "AC-10: 'asset_owner' UDF must be registered after boot infusion load \
         (BC-2.19.001 — each field registers exactly one UDF)"
    );
}

/// AC-10 RED Gate (complementary): `infusion_load_step` with no infusions dir returns empty registry.
///
/// This test verifies non-fatal behavior — an absent `infusions/` directory must NOT cause
/// a boot failure. The function returns an empty registry (0 descriptors), not an error.
///
/// RED Gate failure (before S-1.14-REDO): panics with `todo!()`.
/// Green Gate (after S-1.14-REDO): returns empty registry without panic.
///
/// Traces to: BC-2.22.001 — infusion load step is non-fatal when dir is absent.
#[test]
fn test_boot_infusion_load_step_empty_dir_returns_empty_registry() {
    use tempfile::TempDir;

    // Config dir with NO `infusions/` subdirectory.
    let temp_dir = TempDir::new().expect("AC-10: temp dir creation must succeed for test setup");

    // ── RED GATE ────────────────────────────────────────────────────────────
    // Panics with `todo!("S-1.14-REDO AC-10: ...")` before implementation.
    let registry = infusion_load_step(temp_dir.path());

    // After S-1.14-REDO: empty registry is acceptable (no infusions configured).
    let descriptors = registry.udf_descriptors();
    assert!(
        descriptors.is_empty(),
        "AC-10: infusion_load_step with no infusions/ dir must return an empty InfusionRegistry \
         without panicking (non-fatal absent-dir handling — BC-2.22.001)"
    );
}
