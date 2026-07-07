//! Red Gate tests for S-DEMO-PRISMQL-GRAMMAR-REMEDIATION-001 Area C.
//!
//! BC-2.11.022: `build_reference_content()` + CI 3-tier gate (ADR-045).
//!
//! Red Gate: `build_reference_content` is a `todo!()` stub — all tests panic on
//! the stub body. Tests will fail RED when the function is called.
//!
//! Red Gate tests: 3.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    unused_imports
)]

use prism_mcp::resources::{build_reference_content, ExampleKind, REFERENCE_EXAMPLES};
use prism_query::{ast::Ast, plan_sqlpipe_query, table_registry::TableRegistry, PrismQlParser};
use prism_spec_engine::{
    spec_parser::{AuthType, SensorSpec, TableSpec},
    InfusionField, InfusionRegistry, InfusionSpec, InfusionType,
};

/// AC-006 / BC-2.11.022 postcondition — content completeness.
///
/// `build_reference_content(None)` must return a string containing:
/// - Mode names: "Filter", "SQL", "Pipe", "SqlPipe"
/// - Operators: "CONTAINS", "=~", "IN CIDR", "HAS", "MISSING", "IS NULL", "IS NOT NULL"
/// - Temporal grammar: "NOW()" and "INTERVAL"
/// - Virtual fields: "_sensor", "_client"
/// - Error codes: "E-QUERY-001", "E-QUERY-038", "E-QUERY-040"
/// - Enrichment: "enrich"
/// - Aggregates: "percentile", "distinct_count" (AC-026 coverage per story)
///
/// Red Gate: `build_reference_content` panics on `todo!()`.
#[test]
fn test_bc_2_11_022_reference_content_completeness() {
    // Call the stub — panics on todo!() → RED.
    let content = build_reference_content(None);

    // Mode names (BC-2.11.022 postcondition — all four modes documented).
    for mode in &["Filter", "SQL", "Pipe", "SqlPipe"] {
        assert!(
            content.contains(mode),
            "BC-2.11.022 AC-006: reference content must contain mode name '{mode}'"
        );
    }

    // Operator names (BC-2.11.022 postcondition — operators table required).
    for op in &[
        "CONTAINS",
        "=~",
        "IN CIDR",
        "HAS",
        "MISSING",
        "IS NULL",
        "IS NOT NULL",
    ] {
        assert!(
            content.contains(op),
            "BC-2.11.022 AC-006: reference content must contain operator '{op}'"
        );
    }

    // Temporal grammar (BC-2.11.021 + BC-2.11.022 postcondition — temporal section required).
    for temporal in &["NOW()", "INTERVAL"] {
        assert!(
            content.contains(temporal),
            "BC-2.11.022 AC-006: reference content must contain temporal keyword '{temporal}'"
        );
    }

    // Virtual fields (BC-2.11.012 — virtual fields documented).
    for vf in &["_sensor", "_client"] {
        assert!(
            content.contains(vf),
            "BC-2.11.022 AC-006: reference content must contain virtual field '{vf}'"
        );
    }

    // Error codes (BC-2.11.022 postcondition — E-QUERY quick-reference).
    for code in &["E-QUERY-001", "E-QUERY-038", "E-QUERY-040"] {
        assert!(
            content.contains(code),
            "BC-2.11.022 AC-006: reference content must contain error code '{code}'"
        );
    }

    // Enrichment section (BC-2.11.022 postcondition — enrichment section required).
    assert!(
        content.contains("enrich"),
        "BC-2.11.022 AC-006: reference content must contain 'enrich' (enrichment section)"
    );

    // Aggregates (AC-026 — aggregates/stats section required).
    for agg in &["percentile", "distinct_count"] {
        assert!(
            content.contains(agg),
            "BC-2.11.022 AC-026: reference content must contain aggregate '{agg}'"
        );
    }
}

/// AC-007 / BC-2.11.022 CI 3-tier gate (ADR-045 §B).
///
/// The shared `REFERENCE_EXAMPLES` constant must contain:
/// (1) At least one `ExampleKind::Positive` entry.
/// (2) At least one `ExampleKind::NegativeE040` entry (non-vacuous FORBID-BOTH gate).
/// (3) At least one `ExampleKind::NegativeOther` entry.
///
/// Additionally, every `ExampleKind::Positive` PQL snippet must round-trip through
/// `PrismQlParser::parse` without error (positive round-trip gate).
///
/// Every `ExampleKind::NegativeE040` PQL snippet must parse as `Ast::SqlPipe` AND
/// produce a `RedundantRowLimit` error from `plan_sqlpipe_query` (negative E-QUERY-040
/// FORBID-BOTH plan-time gate — AC-007 literal semantics).
///
/// Red Gate: The current `REFERENCE_EXAMPLES` constant uses the old `Basic/Advanced/Error`
/// variant names — these are now renamed to `Positive/NegativeE040/NegativeOther` per
/// BC-2.11.022 / ADR-045 D3. Compilation fails RED until the rename is complete.
#[test]
fn test_bc_2_11_022_ci_3tier_gate() {
    // Tier shape assertions (BC-2.11.022 ADR-045 §B).
    let has_positive = REFERENCE_EXAMPLES
        .iter()
        .any(|(k, _, _)| matches!(k, ExampleKind::Positive));
    let has_negative_e040 = REFERENCE_EXAMPLES
        .iter()
        .any(|(k, _, _)| matches!(k, ExampleKind::NegativeE040));
    let has_negative_other = REFERENCE_EXAMPLES
        .iter()
        .any(|(k, _, _)| matches!(k, ExampleKind::NegativeOther));

    assert!(
        has_positive,
        "BC-2.11.022 AC-007: REFERENCE_EXAMPLES must contain at least one ExampleKind::Positive entry"
    );
    assert!(
        has_negative_e040,
        "BC-2.11.022 AC-007: REFERENCE_EXAMPLES must contain at least one ExampleKind::NegativeE040 entry \
         (non-vacuous FORBID-BOTH gate — tautological gate is a paper-fix per TD-VSDD-059)"
    );
    assert!(
        has_negative_other,
        "BC-2.11.022 AC-007: REFERENCE_EXAMPLES must contain at least one ExampleKind::NegativeOther entry"
    );

    // Positive round-trip gate (ADR-045 §B): Positive PQL snippets must parse.
    for (kind, title, snippet) in REFERENCE_EXAMPLES.iter() {
        if !matches!(kind, ExampleKind::Positive) {
            continue;
        }
        // Skip comment-prefixed entries (should not appear in Positive tier, but guard defensively).
        if snippet.trim_start().starts_with("--") {
            continue;
        }
        let result = PrismQlParser::parse(snippet);
        assert!(
            result.is_ok(),
            "BC-2.11.022 AC-007: Positive example '{title}' must parse via PrismQlParser::parse; \
             got errors: {:?}",
            result
        );
    }

    // Negative E-QUERY-040 gate (AC-007 literal semantics): every NegativeE040 example
    // must parse as Ast::SqlPipe AND be rejected by plan_sqlpipe_query with
    // PrismError::RedundantRowLimit { .. } (FORBID-BOTH plan-time rejection).
    for (kind, title, snippet) in REFERENCE_EXAMPLES.iter() {
        if !matches!(kind, ExampleKind::NegativeE040) {
            continue;
        }
        // Skip comment-prefixed entries.
        if snippet.trim_start().starts_with("--") {
            continue;
        }
        // Step 1: snippet must parse successfully (E-QUERY-040 fires at plan time, not parse time).
        let parse_result = PrismQlParser::parse(snippet);
        let ast = parse_result.unwrap_or_else(|errs| {
            panic!(
                "BC-2.11.022 AC-007: NegativeE040 example '{title}' must parse successfully \
                 (E-QUERY-040 fires at plan time, not parse time); parse errors: {errs:?}"
            )
        });

        // Step 2: snippet must be a SqlPipe AST (dual-limited queries are SqlPipe ASTs).
        let spq = match ast {
            prism_query::ast::Ast::SqlPipe(spq) => spq,
            other => panic!(
                "BC-2.11.022 AC-007: NegativeE040 example '{title}' must parse as \
                 Ast::SqlPipe (got {other:?}) — dual-limited queries are SqlPipe ASTs"
            ),
        };

        // Step 3: plan must reject with RedundantRowLimit (E-QUERY-040 FORBID-BOTH gate).
        let plan_result = plan_sqlpipe_query(&spq);
        match &plan_result {
            Err(prism_core::error::PrismError::RedundantRowLimit { .. }) => {
                // Expected: this is the FORBID-BOTH pattern.
            }
            Ok(()) => panic!(
                "BC-2.11.022 AC-007: NegativeE040 example '{title}' must fail planning with \
                 PrismError::RedundantRowLimit; got Ok(()) — the example is NOT the FORBID-BOTH \
                 pattern, making the CI gate vacuous (tautological-gate = paper-fix per TD-VSDD-059)"
            ),
            Err(other) => panic!(
                "BC-2.11.022 AC-007: NegativeE040 example '{title}' must fail with \
                 PrismError::RedundantRowLimit; got different error: {other:?}"
            ),
        }
    }

    // Verify all three ExampleKind variants are constructable (compile-time check).
    let _p = ExampleKind::Positive;
    let _n = ExampleKind::NegativeE040;
    let _o = ExampleKind::NegativeOther;
}

/// AC-008 / BC-2.11.022 invariant — `None` registry placeholder.
///
/// `build_reference_content(None)` must:
/// 1. Complete synchronously without panicking.
/// 2. Return a string containing the placeholder text:
///    "Call `list_infusions` to see available enrichment functions for your deployment."
///
/// Red Gate: `build_reference_content` panics on `todo!()`.
#[test]
fn test_bc_2_11_022_none_registry_placeholder() {
    // Call the stub — panics on todo!() → RED.
    let content = build_reference_content(None);

    // Must not be empty.
    assert!(
        !content.is_empty(),
        "BC-2.11.022 AC-008: build_reference_content(None) must return non-empty string"
    );

    // Must contain the infusion placeholder text (BC-2.11.022 invariant).
    let placeholder =
        "Call `list_infusions` to see available enrichment functions for your deployment.";
    assert!(
        content.contains(placeholder),
        "BC-2.11.022 AC-008: build_reference_content(None) must contain infusion placeholder text; \
         got content (first 200 chars): {:?}",
        &content[..content.len().min(200)]
    );
}

// ─── LOW-002: exhaustive match coverage assertion ────────────────────────────

/// LOW-002 / BC-2.11.022 — `build_reference_content` renders every entry from
/// `REFERENCE_EXAMPLES` regardless of kind.
///
/// This is the load-bearing test proving the exhaustive `match kind { ... }` fix.
/// With the OLD three-pass `matches!()` approach, a new ExampleKind variant would
/// be silently dropped. With the new exhaustive match, the compiler would reject
/// an unhandled variant — but this test ALSO catches silent drops at runtime.
///
/// For each (kind, title, snippet) in REFERENCE_EXAMPLES, the rendered content
/// must contain the snippet. The snippet is unique enough to identify each entry.
#[test]
fn test_bc_2_11_022_low002_all_examples_rendered() {
    let content = build_reference_content(None);

    for (kind, title, snippet) in REFERENCE_EXAMPLES.iter() {
        assert!(
            content.contains(snippet),
            "LOW-002 BC-2.11.022: build_reference_content must render snippet from \
             '{title}' (kind={kind:?}); snippet not found in rendered content. \
             This would fire if a new ExampleKind variant was added but not handled \
             in the exhaustive match in build_reference_content."
        );
        // Also verify the title appears (section formatting).
        assert!(
            content.contains(title),
            "LOW-002 BC-2.11.022: build_reference_content must render title '{title}' \
             (kind={kind:?}); title not found in rendered content."
        );
    }
}

// ─── CRIT-003: registry-parity assertion ──────────────────────────────────────

/// CRIT-003 / BC-2.11.022 v1.1 — `build_reference_content(Some(&registry))` renders
/// EXACTLY the per-field UDF callable names from the live registry (EC-11-022-006).
///
/// A registry is constructed with two known infusion specs: `"geoip"` (field: `geoip_country`)
/// and `"threatintel"` (field: `threatintel_score`). After calling
/// `build_reference_content(Some(&registry))`:
/// - Content must list `enrich geoip_country(col)` and `enrich threatintel_score(col)`
///   (per-field descriptor.name, NOT the infusion_id).
/// - Content must NOT contain the placeholder text (registry is present and non-empty).
/// - Content must contain "Available enrichment functions:" (the populated header).
/// - Content must NOT list any enrichment not in the registry (no phantom names).
///
/// This is a LOAD-BEARING production test: `build_reference_content` calls
/// `registry.udf_descriptors()` and deduplicates by `descriptor.name` (v1.1 contract) —
/// testing that per-field callable names match what is rendered.
///
/// Red Gate (CRIT-003): build_reference_content used `include_str!` (static file)
/// that ignored the registry entirely; the registry-controlled enrichment section
/// always showed placeholder text regardless of the live registry.
#[test]
fn test_bc_2_11_022_registry_parity() {
    // Build a known registry with two infusion specs.
    let registry = InfusionRegistry::new();

    let geoip_spec = InfusionSpec::new(
        "geoip",
        "GeoIP Lookup",
        InfusionType::LocalLookup,
        vec![InfusionField::new(
            "geoip_country",
            "src_ip",
            "ip",
            "string",
        )],
        "tests/geoip.toml",
    );
    let threatintel_spec = InfusionSpec::new(
        "threatintel",
        "ThreatIntel Lookup",
        InfusionType::LocalLookup,
        vec![InfusionField::new(
            "threatintel_score",
            "src_ip",
            "ip",
            "string",
        )],
        "tests/threatintel.toml",
    );

    registry
        .load_spec(geoip_spec)
        .expect("CRIT-003: geoip spec must load successfully");
    registry
        .load_spec(threatintel_spec)
        .expect("CRIT-003: threatintel spec must load successfully");

    // Call the production function.
    let content = build_reference_content(Some(&registry));

    // Both registered per-field UDF callable names MUST appear as formatted lines.
    // BC-2.11.022 v1.1 / EC-11-022-006: the reference lists descriptor.name (per-field
    // callable), NOT the infusion_id. For geoip field "geoip_country" the callable is
    // "enrich geoip_country(col)"; for threatintel field "threatintel_score" it is
    // "enrich threatintel_score(col)".
    let geoip_line = "- `enrich geoip_country(col)`";
    let threatintel_line = "- `enrich threatintel_score(col)`";

    assert!(
        content.contains(geoip_line),
        "CRIT-003 BC-2.11.022 v1.1: content must contain '{geoip_line}' for registered \
         per-field UDF 'geoip_country' (descriptor.name); \
         content snippet: {:?}",
        &content[..content.len().min(600)]
    );
    assert!(
        content.contains(threatintel_line),
        "CRIT-003 BC-2.11.022 v1.1: content must contain '{threatintel_line}' for registered \
         per-field UDF 'threatintel_score' (descriptor.name); \
         content snippet: {:?}",
        &content[..content.len().min(600)]
    );

    // Must contain the populated header (not empty registry message).
    assert!(
        content.contains("Available enrichment functions:"),
        "CRIT-003 BC-2.11.022: content must contain 'Available enrichment functions:' header \
         when registry has 2 infusions; got content snippet: {:?}",
        &content[..content.len().min(600)]
    );

    // Must NOT contain the placeholder text (registry is wired and non-empty).
    let placeholder =
        "Call `list_infusions` to see available enrichment functions for your deployment.";
    assert!(
        !content.contains(placeholder),
        "CRIT-003 BC-2.11.022: content must NOT contain placeholder text when a non-empty \
         registry is wired; found placeholder in content"
    );

    // Must NOT list phantom enrichment names not in the registry.
    let phantom_names = ["unknown_enrichment", "test_phantom", "not_registered"];
    for phantom in &phantom_names {
        let phantom_line = format!("- `enrich {phantom}(col)`");
        assert!(
            !content.contains(&phantom_line),
            "CRIT-003 BC-2.11.022: content must NOT list phantom enrichment '{phantom}' \
             that is not registered in the registry"
        );
    }
}

// ─── AC-023: IS NOT NULL JSON-list semantics note ─────────────────────────────

/// AC-023 / GRAMMAR-006 — `build_reference_content(None)` must include the
/// verbatim JSON-list null-semantics note in the Operators section.
///
/// The note text is: "`IS NOT NULL` on a JSON-list field returns `true` if the
/// field is present and non-null (empty list `[]` is NOT null; `null` value is null)."
///
/// This is a load-bearing test: the note describes actual DataFusion / Arrow
/// runtime behavior (JSON columns are stored as Utf8; `Value::Null → Arrow null`,
/// empty array `[]` → serialized as non-null string `"[]"`), so the note must be
/// present and accurate. Verified against `spec_driven_adapter.rs`
/// `build_column_array` (Null branch → None; Array branch → `Some("[]")`)
/// and `column_type_to_arrow` (ColumnType::Json → DataType::Utf8).
#[test]
fn test_bc_2_11_022_ac023_json_list_is_not_null_note() {
    let content = build_reference_content(None);

    // AC-023 verbatim note substring — the note must appear in the reference content.
    let note =
        "`IS NOT NULL` on a JSON-list field returns `true` if the field is present and non-null \
                (empty list `[]` is NOT null; `null` value is null).";
    assert!(
        content.contains(note),
        "AC-023 GRAMMAR-006: build_reference_content must contain IS NOT NULL JSON-list \
         semantics note; note not found in content (first 800 chars): {:?}",
        &content[..content.len().min(800)]
    );
}

// ─── CRIT-001: plan-availability gate for Positive examples ──────────────────

/// CRIT-001 / BC-2.11.022 CI gate — every `Positive` entry in `REFERENCE_EXAMPLES`
/// must NOT trigger E-QUERY-037 (`TableNotAvailable`) at plan time against a registry
/// that contains the `sensor_table` generic placeholder as a registered table.
///
/// This closes the false-green in the existing parse-only positive gate:
/// `PrismQlParser::parse` succeeds on dot-notation FROM targets (e.g.
/// `SELECT * FROM crowdstrike.detections …`) because parsing is purely syntactic.
/// The plan-time check fires only when the registry is wired. Without this test,
/// a Positive example using a dot-notation FROM target (which returns E-QUERY-037
/// at plan time) would pass the parse-only gate, teaching users an erroring query.
///
/// Test protocol:
/// 1. Build a `TableRegistry` containing `sensor_table` (generic placeholder used in
///    REFERENCE_EXAMPLES; sensor_id="sensor", table_name="table" → registered name
///    = "sensor_table" per {sensor_id}_{table_name} convention). This also satisfies
///    BC-2.10.014 AC-008 (no hardcoded vendor names in reference content) — the generic
///    placeholder `sensor_table` is not vendor-specific.
/// 2. For each `(Positive, title, snippet)` in REFERENCE_EXAMPLES (skip comment-prefixed):
///    a. Call `TableRegistry::check_availability_gate(snippet, None, None)`.
///    b. Assert it returns `Ok(())` — NOT `Err(PrismError::TableNotAvailable)`.
///
/// Load-bearing: this gate will RED if a future Positive example reintroduces a
/// dot-notation FROM target (e.g. `crowdstrike.detections`) that would return
/// E-QUERY-037 at runtime on a properly-wired deployment.
///
/// Relation to CRIT-003 residual: that gate verifies NegativeE040 entries fire
/// `RedundantRowLimit`; this gate verifies Positive entries do NOT fire
/// `TableNotAvailable`. Together they close both sides of the plan-validity invariant.
///
/// Pass 3 fix (S-DEMO-FIDELITY-REMEDIATION-001 CRIT-001 sibling-sweep recurrence
/// prevention): the parse-only gate is insufficient because dot-notation parses fine;
/// plan-time table availability checking is the only gate that catches E-QUERY-037.
#[test]
fn test_bc_2_11_022_crit001_positive_examples_runtime_valid() {
    use prism_core::error::PrismError;

    // Build a TableRegistry with `sensor_table` registered.
    // The REFERENCE_EXAMPLES use "sensor_table" as a generic placeholder (BC-2.10.014
    // AC-008: no hardcoded vendor names). Sensor ID = "sensor", table_name = "table" →
    // registered key = "sensor_table" (TableRegistry::register_sensor format:
    // "{sensor_id}_{table_name}").
    let registry = TableRegistry::new();
    let placeholder_spec = SensorSpec::new(
        "sensor",
        "Generic sensor (test fixture for CRIT-001 plan-time gate)",
        AuthType::ApiKey,
        "https://example.com",
        vec![TableSpec::new_point_in_time(
            "table",
            "security_finding",
            vec![],
            vec![],
        )],
        None,
        "1.0.0",
        Vec::new(),
    );
    registry
        .register_sensor(&placeholder_spec)
        .expect("CRIT-001 gate: register sensor_table placeholder must not fail");

    // Sanity check: sensor_table must be registered.
    let gate_result = registry.check_availability_gate("FROM sensor_table | limit 1", None, None);
    assert!(
        gate_result.is_ok(),
        "CRIT-001 gate setup: sanity check failed — sensor_table must be registered; \
         got: {gate_result:?}"
    );

    // For each Positive example, assert plan-time availability passes (no E-QUERY-037).
    for (kind, title, snippet) in REFERENCE_EXAMPLES.iter() {
        if !matches!(kind, ExampleKind::Positive) {
            continue;
        }
        // Skip comment-prefixed entries (guard defensively; no Positive entries should be comments).
        if snippet.trim_start().starts_with("--") {
            continue;
        }

        let result = registry.check_availability_gate(snippet, None, None);
        match &result {
            Ok(()) => {
                // Expected: Positive example is runtime-valid against a registered registry.
            }
            Err(PrismError::TableNotAvailable(details)) => {
                panic!(
                    "CRIT-001 BC-2.11.022 AC-007: Positive example '{title}' returns E-QUERY-037 \
                     (TableNotAvailable) at plan time against a registry containing \
                     crowdstrike_detections. This means the example uses a dot-notation FROM \
                     target (e.g. crowdstrike.detections) that is illegal in SQL/pipe mode. \
                     Fix: change the FROM target to the sensor-prefixed table name \
                     (crowdstrike_detections). Details: {details}"
                );
            }
            Err(other) => {
                // Other plan-time errors (E-QUERY-038 column not found, etc.) are NOT failures
                // for this gate — the registry has no column spec, so column gates are skipped.
                // Only E-QUERY-037 (table not found) indicates a broken Positive example.
                let _ = other; // Non-E-QUERY-037 errors are acceptable for this gate.
            }
        }
    }
}

// ─── CRIT-003 residual: plan-rejection gate ───────────────────────────────────

/// CRIT-003 residual / BC-2.11.022 AC-007 — every `NegativeE040` entry in
/// `REFERENCE_EXAMPLES` must be proven to be the FORBID-BOTH pattern by asserting
/// that `plan_sqlpipe_query` returns `Err(PrismError::RedundantRowLimit { .. })`.
///
/// The earlier closure of CRIT-003 proved the static content contains the E-QUERY-040
/// error code. This test closes the residual gap: the EXAMPLE ITSELF must be the
/// forbidden pattern, not merely an adjacent text string. Without this assertion a
/// catalogued NegativeE040 snippet could be well-formed SqlPipe that is NOT actually
/// dual-limited, making the CI gate vacuous (tautological gate = paper-fix per
/// TD-VSDD-059).
///
/// Test protocol:
/// 1. For each (NegativeE040, title, snippet) in REFERENCE_EXAMPLES:
///    a. Parse via `PrismQlParser::parse` → must produce `Ok(Ast::SqlPipe(spq))`.
///    b. Call `plan_sqlpipe_query(&spq)` → must produce `Err(PrismError::RedundantRowLimit { .. })`.
///
/// Load-bearing: drives the production `plan_sqlpipe_query` function in `prism-query`
/// against real example data, proving the example is the forbidden pattern, not just
/// that the example string contains "LIMIT" twice.
#[test]
fn test_bc_2_11_022_crit003_residual_negativee040_plan_rejected() {
    use prism_core::error::PrismError;

    let negative_e040_entries: Vec<(&str, &str)> = REFERENCE_EXAMPLES
        .iter()
        .filter_map(|(k, title, snippet)| {
            if matches!(k, ExampleKind::NegativeE040) {
                Some((*title, *snippet))
            } else {
                None
            }
        })
        .collect();

    assert!(
        !negative_e040_entries.is_empty(),
        "CRIT-003 residual: REFERENCE_EXAMPLES must contain at least one NegativeE040 entry \
         (non-vacuous FORBID-BOTH gate)"
    );

    for (title, snippet) in &negative_e040_entries {
        // Step 1: snippet must parse as a valid SqlPipe AST.
        let parse_result = PrismQlParser::parse(snippet);
        let ast = parse_result.unwrap_or_else(|errs| {
            panic!(
                "CRIT-003 residual: NegativeE040 example '{title}' must parse successfully \
                 (E-QUERY-040 fires at plan time, not parse time); parse errors: {errs:?}"
            )
        });

        let spq = match ast {
            Ast::SqlPipe(spq) => spq,
            other => panic!(
                "CRIT-003 residual: NegativeE040 example '{title}' must parse as Ast::SqlPipe \
                 (got {other:?}) — dual-limited queries are SqlPipe ASTs"
            ),
        };

        // Step 2: plan must reject with RedundantRowLimit (E-QUERY-040).
        let plan_result = plan_sqlpipe_query(&spq);
        match &plan_result {
            Err(PrismError::RedundantRowLimit { .. }) => {
                // Expected: this is the FORBID-BOTH pattern.
            }
            Ok(()) => panic!(
                "CRIT-003 residual: NegativeE040 example '{title}' must fail planning with \
                 PrismError::RedundantRowLimit; got Ok(()) — the example is NOT actually the \
                 FORBID-BOTH pattern, making the CI gate vacuous (tautological-gate = \
                 paper-fix per TD-VSDD-059)"
            ),
            Err(other) => panic!(
                "CRIT-003 residual: NegativeE040 example '{title}' must fail with \
                 PrismError::RedundantRowLimit; got different error: {other:?}"
            ),
        }
    }
}

// ─── Some(empty-registry) placeholder path ───────────────────────────────────

/// BC-2.11.022 — `build_reference_content(Some(&empty_registry))` renders the
/// wired-but-empty placeholder (not the `None`/unwired placeholder).
///
/// This test covers the `Some(registry)` branch where the registry has ZERO
/// loaded infusion specs (zero `load_spec` calls). The code path at
/// `resources.rs` emits:
///   "No enrichment functions are currently registered for your deployment."
///
/// This is DISTINCT from the `None` path (`test_bc_2_11_022_none_registry_placeholder`)
/// which emits the `list_infusions` placeholder.
///
/// Invariants asserted:
/// 1. Content contains the exact Some(empty) placeholder string.
/// 2. Content does NOT contain the None-path placeholder (`list_infusions`).
/// 3. Content does NOT contain any `enrich <name>(col)` line (zero infusions registered).
#[test]
fn test_bc_2_11_022_some_empty_registry_placeholder() {
    // Construct a wired-but-EMPTY registry — no load_spec calls.
    let empty_registry = InfusionRegistry::new();

    let content = build_reference_content(Some(&empty_registry));

    // Must not be empty.
    assert!(
        !content.is_empty(),
        "BC-2.11.022: build_reference_content(Some(&empty_registry)) must return non-empty string"
    );

    // Must contain the Some(empty) placeholder (resources.rs ~line 1519).
    let some_empty_placeholder =
        "No enrichment functions are currently registered for your deployment.";
    assert!(
        content.contains(some_empty_placeholder),
        "BC-2.11.022: build_reference_content(Some(&empty_registry)) must contain the \
         wired-but-empty placeholder; got content (first 300 chars): {:?}",
        &content[..content.len().min(300)]
    );

    // Must NOT contain the None-path placeholder (list_infusions text).
    let none_placeholder =
        "Call `list_infusions` to see available enrichment functions for your deployment.";
    assert!(
        !content.contains(none_placeholder),
        "BC-2.11.022: build_reference_content(Some(&empty_registry)) must NOT contain the \
         None-path list_infusions placeholder — these are distinct code paths"
    );

    // Must NOT contain any `enrich <name>(col)` line (no infusions registered).
    // Scan line-by-line: any line containing "enrich " followed by a word and "(col)" is a bug.
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("- `enrich ") && trimmed.contains("(col)`") {
            panic!(
                "BC-2.11.022: build_reference_content(Some(&empty_registry)) must NOT render \
                 any `enrich <name>(col)` line when no infusions are registered; \
                 found line: {:?}",
                trimmed
            );
        }
    }
}

// ─── S-PRISMQL-CASE-INSENSITIVE-001: IEQ/IIN/INE discoverability gate ─────────

/// AC-023 / BC-2.11.024 — `build_reference_content(None)` must include
/// IEQ, INE, and IIN in the operators table (ADR-047 §D.4 discoverability).
///
/// The note in RG-023/024 delegated this assertion to the implementer because
/// prism-mcp tests cannot import prism-query directly without a circular dep,
/// so the grammar completeness proxy (RG-023) lives in prism-query; but the
/// MCP resource content assertion must live here.
///
/// Assertions:
/// 1. `build_reference_content(None)` contains "IEQ", "INE", "IIN".
/// 2. `build_reference_content(None)` contains the OCSF Title-case note.
/// 3. `REFERENCE_EXAMPLES` has at least one Positive entry for each of IEQ, INE, IIN.
#[test]
fn test_bc_2_11_024_ieq_iin_ine_in_reference_content() {
    let content = build_reference_content(None);

    // Operator table must include IEQ, INE, IIN (ADR-047 §D.4 discoverability).
    for op in &["IEQ", "INE", "IIN"] {
        assert!(
            content.contains(op),
            "BC-2.11.024 AC-023: build_reference_content must include operator '{op}' \
             in the operators table; not found in content"
        );
    }

    // OCSF Title-case note must be present (RG-024 / ADR-047 §D.4).
    assert!(
        content.contains("OCSF Title-case"),
        "BC-2.11.024 AC-023: build_reference_content must include OCSF Title-case \
         note explaining IEQ/IIN/INE rationale; not found in content"
    );

    // REFERENCE_EXAMPLES must have at least one Positive entry containing each operator.
    for op in &["IEQ", "INE", "IIN"] {
        let has_example = REFERENCE_EXAMPLES
            .iter()
            .any(|(k, _, snippet)| matches!(k, ExampleKind::Positive) && snippet.contains(op));
        assert!(
            has_example,
            "BC-2.11.024 AC-023: REFERENCE_EXAMPLES must contain at least one Positive \
             entry demonstrating operator '{op}'; no such entry found"
        );
    }
}
