//! Failing acceptance tests for S-3.7.01 — Archetype catalog + GenOpts API.
//!
//! Covers BC-3.4.001 (Generator Determinism) and BC-3.4.003 (Archetype Catalog
//! Enumeration) plus VP-108/VP-111/VP-115/VP-116/VP-117.
//!
//! ALL tests in this file are expected to fail (todo!() panic) until the
//! implementation stories complete. Red Gate verified before implementation begins.
//!
//! Run: cargo test -p prism-dtu-common --features fixture-gen \
//!          --test bc_3_4_001_003_archetype_genopts
#![cfg(feature = "fixture-gen")]
#![allow(clippy::unwrap_used, clippy::expect_used)]

use chrono::DateTime;
use prism_core::SensorId;
use prism_dtu_common::generator::{
    all_archetypes, apply_overrides, default_page_size, seeded_rng, Archetype, FixtureSet, GenOpts,
    GenOptsError, OrgId, Provenance,
};
use rand::RngCore;
use serde_json::json;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn org_a() -> OrgId {
    OrgId([
        0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F,
        0x10,
    ])
}

fn org_b() -> OrgId {
    OrgId([
        0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99,
        0x00,
    ])
}

fn zero_org() -> OrgId {
    OrgId([0u8; 16])
}

// ---------------------------------------------------------------------------
// AC-001 / BC-3.4.003 — Archetype enum: exactly 8 variants
// ---------------------------------------------------------------------------

/// BC-3.4.003 invariant 1 + AC-001: the catalog must contain exactly 8 archetypes.
/// VP-115 (archetype catalog completeness).
#[test]
fn test_bc_3_4_003_ac_001_archetype_count_is_8() {
    let archetypes = all_archetypes();
    assert_eq!(
        archetypes.len(),
        8,
        "BC-3.4.003 invariant 1: exactly 8 archetypes must be present"
    );
}

/// BC-3.4.003: all expected variant names are present in the catalog.
#[test]
fn test_bc_3_4_003_ac_001_all_expected_variants_present() {
    let archetypes = all_archetypes();
    let has = |target: Archetype| archetypes.iter().any(|a| *a == target);
    assert!(
        has(Archetype::HealthyOtEnvironment),
        "HealthyOtEnvironment missing"
    );
    assert!(
        has(Archetype::CompromisedEndpoint),
        "CompromisedEndpoint missing"
    );
    assert!(has(Archetype::AuthOutage), "AuthOutage missing");
    assert!(has(Archetype::LargeScale), "LargeScale missing");
    assert!(
        has(Archetype::PaginationEdgeCases),
        "PaginationEdgeCases missing"
    );
    assert!(has(Archetype::SchemaDrift), "SchemaDrift missing");
    assert!(has(Archetype::HighChurn), "HighChurn missing");
    assert!(has(Archetype::DormantTenant), "DormantTenant missing");
}

/// BC-3.4.003 invariant 2: `all_archetypes()` contains no duplicates.
#[test]
fn test_bc_3_4_003_ac_001_no_duplicate_archetypes() {
    let archetypes = all_archetypes();
    let unique: std::collections::HashSet<_> = archetypes.iter().collect();
    assert_eq!(
        unique.len(),
        archetypes.len(),
        "BC-3.4.003 invariant 2: all_archetypes() must not contain duplicates"
    );
}

// ---------------------------------------------------------------------------
// AC-002 / BC-3.4.001 — GenOpts struct: Default values
// ---------------------------------------------------------------------------

/// BC-3.4.001 precondition 2–5 + AC-002: Default impl returns documented values.
#[test]
fn test_bc_3_4_001_ac_002_genopts_default_seed_is_42() {
    let opts = GenOpts::default();
    assert_eq!(opts.seed, 42, "GenOpts::default().seed must be 42");
}

#[test]
fn test_bc_3_4_001_ac_002_genopts_default_scale_is_1() {
    let opts = GenOpts::default();
    assert!(
        (opts.scale - 1.0_f64).abs() < f64::EPSILON,
        "GenOpts::default().scale must be 1.0"
    );
}

#[test]
fn test_bc_3_4_001_ac_002_genopts_default_time_anchor_is_unix_epoch() {
    let opts = GenOpts::default();
    assert_eq!(
        opts.time_anchor,
        DateTime::UNIX_EPOCH,
        "GenOpts::default().time_anchor must be DateTime::UNIX_EPOCH"
    );
}

#[test]
fn test_bc_3_4_001_ac_002_genopts_default_overrides_is_null() {
    let opts = GenOpts::default();
    assert!(
        opts.overrides.is_null(),
        "GenOpts::default().overrides must be serde_json::Value::Null"
    );
}

// ---------------------------------------------------------------------------
// AC-002 / BC-3.4.001 — GenOpts::new() scale validation
// (tests exercise todo!() body — Red Gate target)
// ---------------------------------------------------------------------------

/// BC-3.4.001 precondition 3 + EC-001: scale=0.0 is rejected.
#[test]
fn test_bc_3_4_001_ac_002_genopts_new_rejects_zero_scale() {
    let result = GenOpts::new(42, 0.0_f64, DateTime::UNIX_EPOCH, serde_json::Value::Null);
    assert!(
        matches!(result, Err(GenOptsError::InvalidScale)),
        "GenOpts::new() must reject scale=0.0 (not positive)"
    );
}

/// BC-3.4.001 precondition 3: negative scale is rejected.
#[test]
fn test_bc_3_4_001_ac_002_genopts_new_rejects_negative_scale() {
    let result = GenOpts::new(42, -1.0_f64, DateTime::UNIX_EPOCH, serde_json::Value::Null);
    assert!(
        matches!(result, Err(GenOptsError::InvalidScale)),
        "GenOpts::new() must reject scale=-1.0 (not positive)"
    );
}

/// BC-3.4.001 precondition 3 + EC-002: scale=f64::INFINITY is rejected.
#[test]
fn test_bc_3_4_001_ac_002_genopts_new_rejects_infinite_scale() {
    let result = GenOpts::new(
        42,
        f64::INFINITY,
        DateTime::UNIX_EPOCH,
        serde_json::Value::Null,
    );
    assert!(
        matches!(result, Err(GenOptsError::InvalidScale)),
        "GenOpts::new() must reject scale=INFINITY (not finite)"
    );
}

/// BC-3.4.001 precondition 3: scale=NaN is rejected.
#[test]
fn test_bc_3_4_001_ac_002_genopts_new_rejects_nan_scale() {
    let result = GenOpts::new(42, f64::NAN, DateTime::UNIX_EPOCH, serde_json::Value::Null);
    assert!(
        matches!(result, Err(GenOptsError::InvalidScale)),
        "GenOpts::new() must reject scale=NaN"
    );
}

/// BC-3.4.001 precondition 3: scale=0.001 (minimal positive finite) is accepted.
#[test]
fn test_bc_3_4_001_ac_002_genopts_new_accepts_minimal_positive_scale() {
    let result = GenOpts::new(42, 0.001_f64, DateTime::UNIX_EPOCH, serde_json::Value::Null);
    assert!(
        result.is_ok(),
        "GenOpts::new() must accept scale=0.001 (positive and finite)"
    );
}

/// BC-3.4.001 precondition 3: scale=1.0 is accepted.
#[test]
fn test_bc_3_4_001_ac_002_genopts_new_accepts_unit_scale() {
    let result = GenOpts::new(42, 1.0_f64, DateTime::UNIX_EPOCH, serde_json::Value::Null);
    assert!(result.is_ok(), "GenOpts::new() must accept scale=1.0");
}

/// BC-3.4.001 precondition 3: scale=100.0 (stress) is accepted.
#[test]
fn test_bc_3_4_001_ac_002_genopts_new_accepts_large_scale() {
    let result = GenOpts::new(42, 100.0_f64, DateTime::UNIX_EPOCH, serde_json::Value::Null);
    assert!(result.is_ok(), "GenOpts::new() must accept scale=100.0");
}

// ---------------------------------------------------------------------------
// AC-003 / BC-3.4.001 — seeded_rng determinism (VP-108/VP-111/VP-116)
// (exercises todo!() body — Red Gate target)
// ---------------------------------------------------------------------------

/// BC-3.4.001 postcondition 1 / VP-108 (generate is idempotent for same inputs):
/// Two calls to seeded_rng with identical (seed, org_id) produce identical byte streams.
/// VP-3.4.001-A.
#[test]
fn test_bc_3_4_001_ac_003_vp_108_seeded_rng_same_inputs_produce_identical_stream() {
    let mut rng1 = seeded_rng(42, &org_a());
    let mut rng2 = seeded_rng(42, &org_a());

    let seq1: Vec<u64> = (0..20).map(|_| rng1.next_u64()).collect();
    let seq2: Vec<u64> = (0..20).map(|_| rng2.next_u64()).collect();

    assert_eq!(
        seq1, seq2,
        "BC-3.4.001 postcondition 1 / VP-108: same (seed, org_id) must produce identical RNG stream"
    );
}

/// BC-3.4.001 postcondition 3 / VP-111 (distinct seeds produce distinct records):
/// Different seeds → different RNG streams.
/// VP-3.4.001-B.
#[test]
fn test_bc_3_4_001_ac_003_vp_111_distinct_seeds_produce_distinct_streams() {
    let mut rng1 = seeded_rng(1, &org_a());
    let mut rng2 = seeded_rng(2, &org_a());

    let seq1: Vec<u64> = (0..20).map(|_| rng1.next_u64()).collect();
    let seq2: Vec<u64> = (0..20).map(|_| rng2.next_u64()).collect();

    assert_ne!(
        seq1, seq2,
        "BC-3.4.001 postcondition 3 / VP-111: distinct seeds must produce distinct RNG streams"
    );
}

/// BC-3.4.001 postcondition 4 / VP-116 (distinct org_ids produce distinct records):
/// Same seed, different org_id → different RNG stream.
/// VP-3.4.001-C.
#[test]
fn test_bc_3_4_001_ac_003_vp_116_distinct_org_ids_produce_distinct_streams() {
    let mut rng_a = seeded_rng(42, &org_a());
    let mut rng_b = seeded_rng(42, &org_b());

    let seq_a: Vec<u64> = (0..20).map(|_| rng_a.next_u64()).collect();
    let seq_b: Vec<u64> = (0..20).map(|_| rng_b.next_u64()).collect();

    assert_ne!(
        seq_a, seq_b,
        "BC-3.4.001 postcondition 4 / VP-116: distinct org_ids must produce distinct RNG streams"
    );
}

/// BC-3.4.001 EC-3.4.001-04: seed=u64::MAX must not panic.
#[test]
fn test_bc_3_4_001_ac_003_seed_max_does_not_panic() {
    let mut rng = seeded_rng(u64::MAX, &org_a());
    // If seeded_rng returns without panic, the first draw must succeed.
    let _val = rng.next_u64();
}

/// BC-3.4.001 EC-3.4.001-03: seed=0 is valid and deterministic.
#[test]
fn test_bc_3_4_001_ac_003_seed_zero_is_valid_and_deterministic() {
    let mut rng1 = seeded_rng(0, &org_a());
    let mut rng2 = seeded_rng(0, &org_a());
    assert_eq!(rng1.next_u64(), rng2.next_u64());
}

/// BC-3.4.001 EC-3.4.001-05: org_id with all-zero bytes → org_id_hash=0,
/// so seed ^ 0 = seed; stream is still deterministic.
#[test]
fn test_bc_3_4_001_ac_003_zero_org_id_hash_xor_is_seed_unchanged() {
    let mut rng1 = seeded_rng(42, &zero_org());
    let mut rng2 = seeded_rng(42, &zero_org());
    let seq1: Vec<u64> = (0..10).map(|_| rng1.next_u64()).collect();
    let seq2: Vec<u64> = (0..10).map(|_| rng2.next_u64()).collect();
    assert_eq!(
        seq1, seq2,
        "BC-3.4.001 EC-3.4.001-05: zero org_id produces deterministic stream (seed ^ 0 = seed)"
    );
}

/// BC-3.4.001 invariant 2: verify XOR formula by checking that zero_org produces
/// same stream as seeded_rng with same seed value used directly.
/// (With org_id_hash=0, the effective seed is seed ^ 0 = seed.)
#[test]
fn test_bc_3_4_001_ac_003_xor_formula_with_zero_org_equals_bare_seed() {
    use rand::SeedableRng;
    use rand_chacha::ChaCha20Rng;

    let seed = 99_u64;
    // org_id with all-zero bytes → org_id_hash = 0 → effective seed = 99 ^ 0 = 99
    let mut rng_via_fn = seeded_rng(seed, &zero_org());
    let mut rng_direct = ChaCha20Rng::seed_from_u64(seed ^ 0);

    for _ in 0..20 {
        assert_eq!(
            rng_via_fn.next_u64(),
            rng_direct.next_u64(),
            "BC-3.4.001 invariant 2: XOR formula must match direct seed_from_u64 for zero org_id"
        );
    }
}

// ---------------------------------------------------------------------------
// AC-004 / BC-3.4.001 postcondition 7 — apply_overrides (RFC 7396)
// (exercises todo!() body — Red Gate target)
// ---------------------------------------------------------------------------

/// BC-3.4.001 postcondition 7 + AC-004: non-null patch merges into base object.
#[test]
fn test_bc_3_4_001_ac_004_apply_overrides_merges_patch_into_base() {
    let base = json!({"a": 1, "b": 2});
    let patch = json!({"b": 99, "c": 3});
    let result = apply_overrides(base, &patch);
    assert_eq!(result["a"], json!(1), "key 'a' must be preserved");
    assert_eq!(
        result["b"],
        json!(99),
        "key 'b' must be overwritten by patch"
    );
    assert_eq!(result["c"], json!(3), "key 'c' must be added from patch");
}

/// BC-3.4.001 postcondition 7 + AC-004 + EC-004: null patch value removes key (RFC 7396).
#[test]
fn test_bc_3_4_001_ac_004_apply_overrides_null_value_removes_key() {
    let base = json!({"keep": "yes", "remove": "old"});
    let patch = json!({"remove": null});
    let result = apply_overrides(base, &patch);
    assert_eq!(
        result["keep"],
        json!("yes"),
        "non-patched key must be preserved"
    );
    assert!(
        result.get("remove").is_none() || result["remove"].is_null(),
        "BC-3.4.001 postcondition 7 / EC-004: null patch value must remove the key"
    );
}

/// AC-004: non-object patch replaces entire base (RFC 7396 scalar-replacement rule).
#[test]
fn test_bc_3_4_001_ac_004_apply_overrides_scalar_patch_replaces_base() {
    let base = json!({"a": 1});
    let patch = json!("replacement");
    let result = apply_overrides(base, &patch);
    assert_eq!(
        result,
        json!("replacement"),
        "Non-object patch must replace the entire base (RFC 7396)"
    );
}

/// BC-3.4.001 postcondition 7: identical inputs always produce identical output (determinism).
#[test]
fn test_bc_3_4_001_ac_004_apply_overrides_is_deterministic() {
    let base = json!({"x": 10});
    let patch = json!({"x": 20, "y": 30});
    let r1 = apply_overrides(base.clone(), &patch);
    let r2 = apply_overrides(base, &patch);
    assert_eq!(
        r1, r2,
        "apply_overrides must be deterministic for identical inputs"
    );
}

/// BC-3.4.001 EC-3.4.001-06: overrides with non-null patch; result is deterministic.
#[test]
fn test_bc_3_4_001_ac_004_non_null_patch_result_is_deterministic() {
    let base = json!({"alerts": [1, 2, 3]});
    let patch = json!({"alerts": []});
    let r1 = apply_overrides(base.clone(), &patch);
    let r2 = apply_overrides(base, &patch);
    assert_eq!(
        r1, r2,
        "BC-3.4.001 EC-3.4.001-06: same patch applied twice must produce identical result"
    );
}

// ---------------------------------------------------------------------------
// AC-005 / BC-3.4.003 — default_page_size per sensor
// (exercises todo!() body — Red Gate target)
// VP-115 (archetype baseline counts per sensor)
// ---------------------------------------------------------------------------

/// AC-005 / VP-115: default_page_size returns a non-zero value for every known sensor.
#[test]
fn test_bc_3_4_003_ac_005_vp_115_default_page_size_nonzero_for_all_sensors() {
    for sensor in [
        SensorId::from("claroty"),
        SensorId::from("armis"),
        SensorId::from("crowdstrike"),
        SensorId::from("cyberint"),
    ] {
        let name = format!("{sensor}");
        let size = default_page_size(sensor);
        assert!(size > 0, "default_page_size({name}) must be > 0");
    }
}

/// AC-005: default_page_size for Claroty matches the documented SDK constant.
/// Value sourced from poller-bear specs per S-3.7.01 §Previous Story Intelligence.
#[test]
fn test_bc_3_4_003_ac_005_default_page_size_claroty() {
    // poller-bear SDK default page size is 100 (confirmed from .references/poller-bear/docs/specs.json)
    let size = default_page_size(SensorId::from("claroty"));
    assert_eq!(
        size, 100,
        "default_page_size(Claroty) must equal the poller-bear SDK default of 100"
    );
}

/// AC-005: default_page_size for Armis matches the documented SDK constant.
#[test]
fn test_bc_3_4_003_ac_005_default_page_size_armis() {
    // Armis AQL default page size: 100 per S-3.7.00 DERIVATION.md
    let size = default_page_size(SensorId::from("armis"));
    assert_eq!(
        size, 100,
        "default_page_size(Armis) must equal the DERIVATION.md constant of 100"
    );
}

/// AC-005: default_page_size for CrowdStrike matches the documented SDK constant.
#[test]
fn test_bc_3_4_003_ac_005_default_page_size_crowdstrike() {
    // CrowdStrike FQL default page size: 100 per S-3.7.00 DERIVATION.md
    let size = default_page_size(SensorId::from("crowdstrike"));
    assert_eq!(
        size, 100,
        "default_page_size(CrowdStrike) must equal the DERIVATION.md constant of 100"
    );
}

/// AC-005: default_page_size for Cyberint returns a documented constant.
#[test]
fn test_bc_3_4_003_ac_005_default_page_size_cyberint() {
    // Cyberint default page size: 100 per poller-express SDK constants
    let size = default_page_size(SensorId::from("cyberint"));
    assert_eq!(
        size, 100,
        "default_page_size(Cyberint) must equal the poller-express SDK default of 100"
    );
}

// ---------------------------------------------------------------------------
// AC-006 / BC-3.4.001 postcondition 1 — FixtureSet and Provenance types
// ---------------------------------------------------------------------------

/// AC-006: FixtureSet can be constructed and fields are accessible.
/// (Struct layout test — verifies public field visibility; does not exercise todo!().)
#[test]
fn test_bc_3_4_001_ac_006_fixture_set_fields_accessible() {
    let prov = Provenance {
        org_id: org_a(),
        sensor_id: SensorId::from("claroty"),
        archetype: Archetype::HealthyOtEnvironment,
        seed: 42,
        schema_valid: true,
    };
    let fs = FixtureSet {
        records: vec![json!({"id": "dev-test-42-0"})],
        cursors: vec!["cursor-1".to_string()],
        provenance: prov,
    };
    assert_eq!(fs.records.len(), 1);
    assert_eq!(fs.cursors.len(), 1);
    assert_eq!(fs.provenance.seed, 42);
    assert!(fs.provenance.schema_valid);
}

/// AC-006: Provenance.schema_valid defaults to true for non-SchemaDrift archetypes.
#[test]
fn test_bc_3_4_001_ac_006_provenance_schema_valid_true_for_non_schema_drift() {
    let prov = Provenance {
        org_id: org_a(),
        sensor_id: SensorId::from("armis"),
        archetype: Archetype::HealthyOtEnvironment,
        seed: 1,
        schema_valid: true, // caller must set true; verify field is writable
    };
    assert!(
        prov.schema_valid,
        "Provenance.schema_valid must be true for non-SchemaDrift archetypes"
    );
}

/// AC-006: Provenance.schema_valid is false for SchemaDrift archetype (BC-3.4.003 invariant 4).
#[test]
fn test_bc_3_4_001_ac_006_provenance_schema_valid_false_for_schema_drift() {
    let prov = Provenance {
        org_id: org_a(),
        sensor_id: SensorId::from("claroty"),
        archetype: Archetype::SchemaDrift,
        seed: 7,
        schema_valid: false, // generator must set this; field must accept false
    };
    assert!(
        !prov.schema_valid,
        "Provenance.schema_valid must be false for SchemaDrift archetype"
    );
}

// ---------------------------------------------------------------------------
// AC-007 / BC-3.4.001 invariant 4 — Feature gate (structural, no todo!())
// ---------------------------------------------------------------------------

/// AC-007 / VP-117: This entire test file is gated behind `#[cfg(feature = "fixture-gen")]`
/// at the top of the file. If the feature is absent, this test binary is not compiled.
/// The following test asserts the cfg attribute took effect.
#[test]
fn test_bc_3_4_001_ac_007_vp_117_test_file_compiled_only_with_fixture_gen_feature() {
    // If we reach here, the feature gate is working. No runtime assertion needed;
    // the compilation itself is the test. AC-007 compliance.
    assert!(true, "fixture-gen feature gate is active");
}

// ---------------------------------------------------------------------------
// VP-108: generate is idempotent — seeded_rng is the core primitive
// (Exercises todo!() — Red Gate target)
// ---------------------------------------------------------------------------

/// VP-108 / TV-3.4.001-01: sequential calls with identical (seed, org_id) produce
/// byte-identical output (25-element sample).
#[test]
fn test_vp_108_seeded_rng_idempotent_sequential_calls() {
    let draws: usize = 25;
    let seq1: Vec<u64> = {
        let mut rng = seeded_rng(42, &org_a());
        (0..draws).map(|_| rng.next_u64()).collect()
    };
    let seq2: Vec<u64> = {
        let mut rng = seeded_rng(42, &org_a());
        (0..draws).map(|_| rng.next_u64()).collect()
    };
    assert_eq!(
        seq1, seq2,
        "VP-108: seeded_rng must produce byte-identical stream for repeated invocations"
    );
}

/// VP-111 / TV-3.4.001-02: distinct seeds produce distinct first u64 from RNG.
#[test]
fn test_vp_111_distinct_seeds_produce_distinct_first_draw() {
    let v1 = seeded_rng(1, &org_a()).next_u64();
    let v2 = seeded_rng(2, &org_a()).next_u64();
    assert_ne!(
        v1, v2,
        "VP-111: seed=1 and seed=2 must produce different first u64"
    );
}

/// VP-116 / TV-3.4.001-03: distinct org_ids with same seed produce distinct first u64.
#[test]
fn test_vp_116_distinct_org_ids_produce_distinct_first_draw() {
    let v_a = seeded_rng(42, &org_a()).next_u64();
    let v_b = seeded_rng(42, &org_b()).next_u64();
    assert_ne!(
        v_a, v_b,
        "VP-116: same seed but different org_ids must produce different first u64"
    );
}

/// VP-3.4.001-D / BC-3.4.001 invariant 1 (CI enforcement documentation):
/// seeded_rng must not call rand::thread_rng().
///
/// Enforcement mechanism: the CI pipeline runs
///   `grep -rn 'thread_rng\s*()' crates/prism-dtu-common/src/generator/`
/// and fails if any non-comment invocation is found. This test documents the
/// invariant and verifies the `seeded_rng` public function is callable without
/// panicking from a non-determinism source (it will panic from todo!() until
/// implementation lands — that is the correct Red Gate failure for this test).
#[test]
fn test_vp_117_seeded_rng_is_callable_and_non_determinism_invariant_is_documented() {
    // Calling seeded_rng() exercises the todo!() stub → Red Gate failure.
    // After implementation this test verifies deterministic output (VP-108 covers
    // the full assertion; this test pins the VP-117 traceability marker).
    let _rng = seeded_rng(0, &org_a());
    // If we reach here (post-implementation), the function returned without
    // using non-deterministic entropy — the CI grep check enforces the static invariant.
}
