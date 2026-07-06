//! Red Gate test for S-DEMO-FIDELITY-REMEDIATION-001 AC-N1 — BC-2.11.022 v1.1.
//!
//! Finding N1 (EC-11-022-006): `build_reference_content` deduplicates by `infusion_id`
//! instead of `descriptor.name` (the per-field UDF name). For a registry with
//! infusion `threat_intel` (fields: `threat_score`, `threat_is_known_malicious`,
//! `threat_sources`) and infusion `nvd` (fields: `cvss_base_score`, `cvss_severity`,
//! `cvss_vector`), the current code emits `threat_intel` and `nvd` as callable names
//! (one entry per infusion_id), NOT the six per-field UDF names.
//!
//! # Test → AC mapping
//!
//! | Test | AC | BC |
//! |------|----|----|
//! | test_bc_2_11_022_n1_per_field_udf_names | AC-N1 | BC-2.11.022 v1.1 EC-11-022-006 |
//!
//! # Red Gate failure mode
//!
//! The test asserts:
//! 1. The enrichment section contains all 6 per-field UDF names.
//! 2. The strings `threat_intel(` and `nvd(` do NOT appear (callable form guard).
//!
//! Current code inserts `infusion_id.clone()` into `infusion_names`, producing
//! `enrich threat_intel(col)` and `enrich nvd(col)` instead of the 6 per-field entries.
//! Assertion (1) fails because none of `threat_score(`, `cvss_base_score(` etc. appear.

use prism_mcp::resources::build_reference_content;
use prism_spec_engine::{
    infusion::{InfusionField, InfusionSpec, InfusionType},
    InfusionRegistry,
};

/// Build a two-infusion `InfusionRegistry` fixture:
/// - `threat_intel` infusion with fields: `threat_score`, `threat_is_known_malicious`, `threat_sources`
/// - `nvd` infusion with fields: `cvss_base_score`, `cvss_severity`, `cvss_vector`
///
/// NullSource is used (no file-backed source required for descriptor export).
fn make_two_infusion_registry() -> InfusionRegistry {
    let registry = InfusionRegistry::new();

    // threat_intel infusion — 3 per-field UDF names
    // InfusionField::new(name, input_field, input_type, output_type)
    let threat_intel_spec = InfusionSpec::new(
        "threat_intel",
        "ThreatIntel enrichment",
        InfusionType::LocalLookup,
        vec![
            InfusionField::new("threat_score", "iocs_value_first", "string", "float64"),
            InfusionField::new(
                "threat_is_known_malicious",
                "iocs_value_first",
                "string",
                "bool",
            ),
            InfusionField::new("threat_sources", "iocs_value_first", "string", "string"),
        ],
        "/dev/null",
    );
    registry
        .load_spec(threat_intel_spec)
        .expect("threat_intel spec must load");

    // nvd infusion — 3 per-field UDF names
    let nvd_spec = InfusionSpec::new(
        "nvd",
        "NVD CVSS enrichment",
        InfusionType::LocalLookup,
        vec![
            InfusionField::new("cvss_base_score", "cve_id", "string", "float64"),
            InfusionField::new("cvss_severity", "cve_id", "string", "string"),
            InfusionField::new("cvss_vector", "cve_id", "string", "string"),
        ],
        "/dev/null",
    );
    registry.load_spec(nvd_spec).expect("nvd spec must load");

    registry
}

/// BC-2.11.022 v1.1 EC-11-022-006 — AC-N1 Red Gate test.
///
/// `build_reference_content` with a live `InfusionRegistry` must deduplicate by
/// `descriptor.name` (per-field UDF name), NOT by `descriptor.infusion_id`.
///
/// For a registry with `threat_intel` (fields: `threat_score`, `threat_is_known_malicious`,
/// `threat_sources`) and `nvd` (fields: `cvss_base_score`, `cvss_severity`, `cvss_vector`),
/// the enrichment section MUST list exactly 6 per-field callable entries.
///
/// Load-bearing: reverting to infusion_id deduplication (producing `enrich threat_intel(col)`
/// and `enrich nvd(col)` — 2 entries instead of 6) causes all per-field name assertions to fail.
#[test]
fn test_bc_2_11_022_n1_per_field_udf_names() {
    let registry = make_two_infusion_registry();
    let content = build_reference_content(Some(&registry));

    // ── Positive assertions: all 6 per-field UDF names must appear in callable form ──

    let per_field_names = [
        "threat_score",
        "threat_is_known_malicious",
        "threat_sources",
        "cvss_base_score",
        "cvss_severity",
        "cvss_vector",
    ];

    for name in &per_field_names {
        // The callable form emitted by build_reference_content is `enrich {name}(col)`.
        let callable = format!("enrich {name}(");
        assert!(
            content.contains(&callable),
            "BC-2.11.022 AC-N1: enrichment section must contain per-field UDF callable \
             'enrich {name}(col)' — current code deduplicates by infusion_id and emits \
             'enrich threat_intel(col)' / 'enrich nvd(col)' instead. \
             Content enrichment section (first 2000 chars): {}",
            &content[..content.len().min(2000)]
        );
    }

    // ── Negative assertions: infusion_id forms must NOT appear as callable entries ──
    // The infusion_ids `threat_intel` and `nvd` are not callable UDF names.
    // They MUST NOT appear in the form `enrich threat_intel(` or `enrich nvd(`.

    assert!(
        !content.contains("enrich threat_intel("),
        "BC-2.11.022 AC-N1 regression guard: enrichment section must NOT contain \
         'enrich threat_intel(' — threat_intel is an infusion_id (not callable UDF name). \
         Found in content."
    );

    assert!(
        !content.contains("enrich nvd("),
        "BC-2.11.022 AC-N1 regression guard: enrichment section must NOT contain \
         'enrich nvd(' — nvd is an infusion_id (not callable UDF name). \
         Found in content."
    );
}
