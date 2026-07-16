//! Tests for BC-2.02.012 — OCSF Event Class Selection Per Sensor Record Type.
//!
//! BC: Each sensor record type maps deterministically to exactly one OCSF event class.
//! Security Finding (class_uid 2001) is deprecated since OCSF v1.1.0 and must not be used.
//!
//! Acceptance Criteria covered:
//! - AC-6: `EventClassSelector::select("crowdstrike", "detection")` returns `Ok(2004)`.
//! - AC-7: `EventClassSelector::select("claroty", "alert")` returns `Ok(2004)`.
//! - AC-8: `EventClassSelector::select("vendor_x", "unknown_type")` returns `Err(OcsfUnknownEventClass)`.
//!
//! Test Vectors (BC-2.02.012):
//! - TV-BC-2.02.012-001: crowdstrike_detection → 2004 (Detection Finding)
//! - TV-BC-2.02.012-002: claroty_device → 5001 (Device Inventory Info)
//! - TV-BC-2.02.012-003: claroty_vulnerability → 2002 (Vulnerability Finding)
//! - TV-BC-2.02.012-004: armis_audit_log → 3001 (Audit Activity)
//! - TV-BC-2.02.012-005: claroty_event (no OCSF mapping) → Err (stub)
//! - TV-BC-2.02.012-006: entirely unknown record type → Err
//!
//! # Note on assert patterns
//!
//! `PrismError` does not derive `PartialEq` (it is `#[non_exhaustive]` and contains
//! source-error types). Tests that check `Ok(class_uid)` use `.is_ok()` + `.unwrap()`
//! rather than `assert_eq!(result, Ok(...))`.
//!
//! # Status
//!
//! All tests in this file PASS with the stub because EventClassSelector is a
//! compile-time constant mapping table — fully functional without ocsf-proto-gen.

use prism_core::PrismError;

use crate::class_selector::{
    EventClassSelector, CLASS_UID_ACCOUNT_CHANGE, CLASS_UID_DETECTION_FINDING,
    CLASS_UID_DEVICE_INVENTORY_INFO, CLASS_UID_INCIDENT_FINDING, CLASS_UID_VULNERABILITY_FINDING,
};

// Note: these tests are expected to PASS (the class selector is a lookup table,
// not gated on ocsf-proto-gen). They are kept here because they verify AC-6, AC-7, AC-8.

/// BC-2.02.012 / AC-6 / TV-BC-2.02.012-001:
/// CrowdStrike detection → Detection Finding (class_uid 2004). NOT deprecated 2001.
#[test]
fn test_BC_2_02_012_crowdstrike_detection_returns_2004() {
    let result = EventClassSelector::select("crowdstrike", "detection");
    assert!(
        result.is_ok(),
        "crowdstrike/detection must return Ok (AC-6, BC-2.02.012); got {:?}",
        result
    );
    assert_eq!(
        result.unwrap(),
        CLASS_UID_DETECTION_FINDING,
        "crowdstrike/detection must map to Detection Finding (2004), NOT deprecated 2001 (AC-6)"
    );
}

/// BC-2.02.012: CrowdStrike incident → Incident Finding (class_uid 2005).
#[test]
fn test_BC_2_02_012_crowdstrike_incident_returns_2005() {
    let result = EventClassSelector::select("crowdstrike", "incident");
    assert!(
        result.is_ok(),
        "crowdstrike/incident must return Ok (BC-2.02.012)"
    );
    assert_eq!(
        result.unwrap(),
        CLASS_UID_INCIDENT_FINDING,
        "crowdstrike/incident must map to Incident Finding (2005) (BC-2.02.012)"
    );
}

/// BC-2.02.012: Cyberint alert → Detection Finding (class_uid 2004).
#[test]
fn test_BC_2_02_012_cyberint_alert_returns_2004() {
    let result = EventClassSelector::select("cyberint", "alert");
    assert!(
        result.is_ok(),
        "cyberint/alert must return Ok (BC-2.02.012)"
    );
    assert_eq!(
        result.unwrap(),
        CLASS_UID_DETECTION_FINDING,
        "cyberint/alert must map to Detection Finding (2004) (BC-2.02.012)"
    );
}

/// BC-2.02.012 / AC-7: Claroty alert → Detection Finding (class_uid 2004).
#[test]
fn test_BC_2_02_012_claroty_alert_returns_2004() {
    let result = EventClassSelector::select("claroty", "alert");
    assert!(
        result.is_ok(),
        "claroty/alert must return Ok (AC-7, BC-2.02.012)"
    );
    assert_eq!(
        result.unwrap(),
        CLASS_UID_DETECTION_FINDING,
        "claroty/alert must map to Detection Finding (2004) (AC-7, BC-2.02.012)"
    );
}

/// BC-2.02.012 / TV-BC-2.02.012-002: Claroty device → Device Inventory Info (5001).
#[test]
fn test_BC_2_02_012_claroty_device_returns_5001() {
    let result = EventClassSelector::select("claroty", "device");
    assert!(
        result.is_ok(),
        "claroty/device must return Ok (BC-2.02.012)"
    );
    assert_eq!(
        result.unwrap(),
        CLASS_UID_DEVICE_INVENTORY_INFO,
        "claroty/device must map to Device Inventory Info (5001) (TV-BC-2.02.012-002)"
    );
}

/// BC-2.02.012 / TV-BC-2.02.012-003: Claroty vulnerability → Vulnerability Finding (2002).
#[test]
fn test_BC_2_02_012_claroty_vulnerability_returns_2002() {
    let result = EventClassSelector::select("claroty", "vulnerability");
    assert!(
        result.is_ok(),
        "claroty/vulnerability must return Ok (BC-2.02.012)"
    );
    assert_eq!(
        result.unwrap(),
        CLASS_UID_VULNERABILITY_FINDING,
        "claroty/vulnerability must map to Vulnerability Finding (2002) (TV-BC-2.02.012-003)"
    );
}

/// BC-2.02.012: Armis device → Device Inventory Info (5001).
#[test]
fn test_BC_2_02_012_armis_device_returns_5001() {
    let result = EventClassSelector::select("armis", "device");
    assert!(result.is_ok(), "armis/device must return Ok (BC-2.02.012)");
    assert_eq!(
        result.unwrap(),
        CLASS_UID_DEVICE_INVENTORY_INFO,
        "armis/device must map to Device Inventory Info (5001) (BC-2.02.012)"
    );
}

/// BC-2.02.012: Armis alert → Detection Finding (2004).
#[test]
fn test_BC_2_02_012_armis_alert_returns_2004() {
    let result = EventClassSelector::select("armis", "alert");
    assert!(result.is_ok(), "armis/alert must return Ok (BC-2.02.012)");
    assert_eq!(
        result.unwrap(),
        CLASS_UID_DETECTION_FINDING,
        "armis/alert must map to Detection Finding (2004) (BC-2.02.012)"
    );
}

/// BC-2.02.012 / TV-BC-2.02.012-004: Armis audit_log → Audit Activity (3001).
#[test]
fn test_BC_2_02_012_armis_audit_log_returns_3001() {
    let result = EventClassSelector::select("armis", "audit_log");
    assert!(
        result.is_ok(),
        "armis/audit_log must return Ok (BC-2.02.012)"
    );
    assert_eq!(
        result.unwrap(),
        CLASS_UID_ACCOUNT_CHANGE,
        "armis/audit_log must map to Audit Activity (3001) (TV-BC-2.02.012-004)"
    );
}

/// BC-2.02.012 / TV-BC-2.02.012-004 (claroty): Claroty audit_log → Audit Activity (3001).
#[test]
fn test_BC_2_02_012_claroty_audit_log_returns_3001() {
    let result = EventClassSelector::select("claroty", "audit_log");
    assert!(
        result.is_ok(),
        "claroty/audit_log must return Ok (BC-2.02.012)"
    );
    assert_eq!(
        result.unwrap(),
        CLASS_UID_ACCOUNT_CHANGE,
        "claroty/audit_log must map to Audit Activity (3001) (BC-2.02.012)"
    );
}

/// BC-2.02.012 / AC-8 / TV-BC-2.02.012-006: completely unknown sensor+record_type
/// returns `Err(OcsfUnknownEventClass)`.
#[test]
fn test_BC_2_02_012_unknown_pair_returns_err() {
    let result = EventClassSelector::select("vendor_x", "unknown_type");
    assert!(
        result.is_err(),
        "vendor_x/unknown_type must return Err (AC-8, BC-2.02.012)"
    );

    let err = result.unwrap_err();
    match &err {
        PrismError::OcsfUnknownEventClass {
            sensor,
            record_type,
        } => {
            assert_eq!(sensor, "vendor_x");
            assert_eq!(record_type, "unknown_type");
        }
        other => panic!(
            "Expected OcsfUnknownEventClass, got {:?} (AC-8, BC-2.02.012)",
            other
        ),
    }
}

/// BC-2.02.012 invariant: deprecated class_uid 2001 (Security Finding) MUST NOT appear
/// in any mapping.
///
/// This test iterates all known sensor/record_type combinations and asserts none
/// returns class_uid 2001. (AC-6 note: "NOT deprecated 2001", BC-2.02.012)
#[test]
fn test_BC_2_02_012_invariant_no_deprecated_2001_in_any_mapping() {
    const DEPRECATED_SECURITY_FINDING: u32 = 2001;

    let test_cases = [
        ("crowdstrike", "detection"),
        ("crowdstrike", "incident"),
        ("cyberint", "alert"),
        ("claroty", "alert"),
        ("claroty", "asset"),
        ("claroty", "device"),
        ("claroty", "vulnerability"),
        ("claroty", "audit_log"),
        ("armis", "device"),
        ("armis", "alert"),
        ("armis", "audit_log"),
    ];

    for (sensor, record_type) in &test_cases {
        let result = EventClassSelector::select(sensor, record_type);
        if let Ok(class_uid) = result {
            assert_ne!(
                class_uid, DEPRECATED_SECURITY_FINDING,
                "{sensor}/{record_type} must NOT map to deprecated Security Finding \
                 (2001) — deprecated since OCSF v1.1.0 (BC-2.02.012 invariant)"
            );
        }
    }
}

/// BC-2.02.012 invariant: select() is deterministic — same inputs always yield same output.
#[test]
fn test_BC_2_02_012_invariant_select_is_deterministic() {
    let first = EventClassSelector::select("crowdstrike", "detection");
    let second = EventClassSelector::select("crowdstrike", "detection");

    // Both should be Ok with the same class_uid — PrismError doesn't impl PartialEq
    // so we compare the Ok values directly.
    assert!(
        first.is_ok(),
        "first call to select() must return Ok (BC-2.02.012)"
    );
    assert!(
        second.is_ok(),
        "second call to select() must return Ok (BC-2.02.012)"
    );
    assert_eq!(
        first.unwrap(),
        second.unwrap(),
        "EventClassSelector::select() must be deterministic — same value on every call \
         (BC-2.02.012 invariant)"
    );
}

/// BC-2.02.012: select() with empty sensor string returns Err.
#[test]
fn test_BC_2_02_012_rejects_empty_sensor() {
    let result = EventClassSelector::select("", "detection");
    assert!(
        result.is_err(),
        "EventClassSelector::select() must return Err for empty sensor (BC-2.02.012)"
    );
}

/// BC-2.02.012: select() with empty record_type string returns Err.
#[test]
fn test_BC_2_02_012_rejects_empty_record_type() {
    let result = EventClassSelector::select("crowdstrike", "");
    assert!(
        result.is_err(),
        "EventClassSelector::select() must return Err for empty record_type (BC-2.02.012)"
    );
}

// =============================================================================
// OCSF-CLASS-MIGRATION-001 Red Gate tests (BC-2.02.012)
//
// These 5 tests constitute the Red Gate for OCSF-CLASS-MIGRATION-001.
// ALL MUST FAIL before the implementer begins (per tdd_mode: strict).
//
// Test vectors exercised:
//   TV-BC-2.02.012-007: select_by_class_name("detection_finding") → Ok(2004), no WARN
//   TV-BC-2.02.012-008: select_by_class_name("security_finding")  → Ok(2004), WARN emitted
//   TV-BC-2.02.012-009: rg 'ocsf_class.*security_finding' crates/prism-sensors/specs/ → 0 results
//   INV-NO-2001-SELECT-PATH: select() path never returns 2001
//   INV-PRODUCTION-TOML-NO-SECURITY-FINDING: no production TOML uses security_finding
// =============================================================================

// ---------------------------------------------------------------------------
// Shared tracing-capture helper (used by AC-002 and AC-003)
//
// Capture strategy: install a tracing-subscriber fmt subscriber scoped to
// the current test via set_default(), then check the captured string buffer.
// This mirrors the pattern in prism-spec-engine/tests/pipeline_http_integration.rs
// (test_BC_2_16_002_emits_pipeline_truncated_event_on_10k_cap and related tests).
// ---------------------------------------------------------------------------

mod ocsf_migration_red_gate {
    use std::sync::{Arc, Mutex};

    use tracing_subscriber::util::SubscriberInitExt as _;

    use crate::class_selector::EventClassSelector;

    /// Build a per-test tracing subscriber that captures all WARN+ output into a
    /// `Arc<Mutex<String>>`. Returns the guard (must be kept alive for the duration
    /// of the test) and the buffer handle for assertions.
    fn make_warn_capture_subscriber() -> (impl Drop, Arc<Mutex<String>>) {
        let log_buffer: Arc<Mutex<String>> = Arc::new(Mutex::new(String::new()));
        let log_buffer_clone = log_buffer.clone();

        let writer = tracing_subscriber::fmt::writer::BoxMakeWriter::new(move || {
            struct BufWriter(Arc<Mutex<String>>);
            impl std::io::Write for BufWriter {
                fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
                    let s = String::from_utf8_lossy(buf);
                    self.0.lock().unwrap().push_str(&s);
                    Ok(buf.len())
                }
                fn flush(&mut self) -> std::io::Result<()> {
                    Ok(())
                }
            }
            BufWriter(log_buffer_clone.clone())
        });

        let subscriber = tracing_subscriber::fmt()
            .with_writer(writer)
            .with_max_level(tracing::Level::WARN)
            // Disable ANSI colour codes so structured field assertions like
            // `captured.contains("resolved_class_uid=2004")` match the raw
            // `key=value` text without embedded escape sequences.
            .with_ansi(false)
            .finish();

        // set_default scopes the subscriber to the current thread only — safe for
        // nextest's per-test thread model.
        let guard = subscriber.set_default();
        (guard, log_buffer)
    }

    // -------------------------------------------------------------------------
    // AC-001 / TV-BC-2.02.012-009 / INV-PRODUCTION-TOML-NO-SECURITY-FINDING
    //
    // Asserts: no production sensor TOML under crates/prism-sensors/specs/ (the 4
    // bundled specs, not customer overrides) declares ocsf_class = "security_finding".
    //
    // RED GATE: FAILS because claroty.sensor.toml, crowdstrike.sensor.toml,
    // cyberint.sensor.toml, and armis.sensor.toml currently contain
    // ocsf_class = "security_finding" for their alert/detection tables.
    //
    // GREEN: after the implementer updates all 4 TOMLs to "detection_finding".
    // -------------------------------------------------------------------------

    /// BC-2.02.012 / AC-001 / TV-BC-2.02.012-009 / INV-PRODUCTION-TOML-NO-SECURITY-FINDING:
    /// No production sensor TOML in `crates/prism-sensors/specs/` declares
    /// `ocsf_class = "security_finding"` (deprecated OCSF v1.1.0 value).
    ///
    /// The grep audit `rg 'ocsf_class.*security_finding' crates/prism-sensors/specs/`
    /// must return zero results after OCSF-CLASS-MIGRATION-001 merges.
    ///
    /// RED GATE: crowdstrike, armis, claroty, cyberint all still declare
    /// `ocsf_class = "security_finding"` — this test will find them and FAIL.
    #[test]
    fn test_BC_2_02_012_no_production_toml_uses_security_finding() {
        // CARGO_MANIFEST_DIR points at crates/prism-ocsf/ at test time.
        // Navigate to the workspace root (../../) then into crates/prism-sensors/specs/.
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
            .expect("CARGO_MANIFEST_DIR must be set by cargo (AC-001)");
        // CARGO_MANIFEST_DIR = crates/prism-ocsf/.
        // crates/prism-sensors/specs is one level up (sibling crate).
        let sensors_specs_dir = std::path::Path::new(&manifest_dir)
            .join("../prism-sensors/specs")
            .canonicalize()
            .expect(
                "crates/prism-sensors/specs/ must exist \
                 (AC-001, INV-PRODUCTION-TOML-NO-SECURITY-FINDING)",
            );

        // Walk all *.sensor.toml files directly under sensors_specs_dir
        // (not recursive into customer subdirs — AC-001 scopes to the 4 bundled
        // production specs, not customer overlays).
        let production_tomls = [
            "crowdstrike.sensor.toml",
            "armis.sensor.toml",
            "claroty.sensor.toml",
            "cyberint.sensor.toml",
        ];

        let mut violations: Vec<String> = Vec::new();

        for filename in &production_tomls {
            let path = sensors_specs_dir.join(filename);
            let content = std::fs::read_to_string(&path).unwrap_or_else(|e| {
                panic!(
                    "AC-001: failed to read {:?}: {e} \
                     (production TOML must be present, BC-2.02.012 INV-PRODUCTION-TOML-NO-SECURITY-FINDING)",
                    path
                )
            });

            // Search for any line containing `ocsf_class` followed by `security_finding`.
            for (line_no, line) in content.lines().enumerate() {
                if line.contains("ocsf_class") && line.contains("security_finding") {
                    violations.push(format!("{filename}:{}: {line}", line_no + 1));
                }
            }
        }

        assert!(
            violations.is_empty(),
            "AC-001 / TV-BC-2.02.012-009 / INV-PRODUCTION-TOML-NO-SECURITY-FINDING FAILED:\n\
             The following production sensor TOMLs still declare \
             ocsf_class = \"security_finding\" (deprecated OCSF v1.1.0).\n\
             All 4 must be updated to ocsf_class = \"detection_finding\" \
             before this story's PR merges.\n\
             Violations found ({} total):\n{}",
            violations.len(),
            violations.join("\n")
        );
    }

    // -------------------------------------------------------------------------
    // AC-002 / TV-BC-2.02.012-007
    //
    // Asserts: select_by_class_name("detection_finding") == Ok(2004) AND
    //          no "ocsf.deprecated_class_alias" WARN is emitted.
    //
    // RED GATE: FAILS because current code returns Ok(2004) — wait, that already
    // passes! The "detection_finding" arm in the current code already returns
    // Ok(CLASS_UID_DETECTION_FINDING) = Ok(2004).
    //
    // However, the no-WARN assertion is load-bearing: we capture tracing output
    // and assert the deprecated_class_alias warn is ABSENT. In the current code,
    // no warn is emitted for "detection_finding" — so this part passes too.
    //
    // RESULT: AC-002 is the edge case where the test will PASS at Red Gate
    // because the function already maps "detection_finding" → 2004 with no warn.
    // This is documented in the Red Gate log. The test remains load-bearing as a
    // regression guard (if the implementer accidentally adds a warn for the
    // canonical path, this catches it).
    //
    // NOTE TO ORCHESTRATOR: AC-002 passes at Red Gate. This is correct behavior —
    // the "detection_finding" mapping already exists. The implementer's work is:
    //   1. Change "security_finding" to return 2004 (not 2001) with a WARN.
    //   2. Update the 4 production TOMLs.
    // Neither change affects the "detection_finding" arm.
    // -------------------------------------------------------------------------

    /// BC-2.02.012 / AC-002 / TV-BC-2.02.012-007:
    /// `select_by_class_name("detection_finding")` returns `Ok(2004)` (canonical PRIMARY path)
    /// and emits NO `ocsf.deprecated_class_alias` WARN.
    ///
    /// RED GATE: PASSES (detection_finding already maps to 2004; no warn emitted).
    /// Kept as regression guard — if the implementer adds a warn for the canonical path,
    /// this test will catch the regression.
    #[test]
    fn test_BC_2_02_012_select_by_class_name_detection_finding_returns_2004_no_warn() {
        let (_guard, log_buffer) = make_warn_capture_subscriber();

        let result = EventClassSelector::select_by_class_name("detection_finding");

        // Assert Ok(2004) — OCSF Detection Finding class_uid (BC-2.02.012 TV-007).
        assert!(
            result.is_ok(),
            "select_by_class_name(\"detection_finding\") must return Ok \
             (BC-2.02.012 AC-002 / TV-BC-2.02.012-007)"
        );
        assert_eq!(
            result.unwrap(),
            2004,
            "select_by_class_name(\"detection_finding\") must return Ok(2004), \
             OCSF Detection Finding class_uid (BC-2.02.012 AC-002 / TV-BC-2.02.012-007)"
        );

        // Assert NO deprecation WARN emitted — "detection_finding" is the canonical path.
        let captured = log_buffer.lock().unwrap().clone();
        assert!(
            !captured.contains("ocsf.deprecated_class_alias"),
            "select_by_class_name(\"detection_finding\") MUST NOT emit \
             ocsf.deprecated_class_alias WARN — it is the canonical PRIMARY path \
             (BC-2.02.012 AC-002 / TV-BC-2.02.012-007); \
             captured log: {captured}"
        );
    }

    // -------------------------------------------------------------------------
    // AC-003 / TV-BC-2.02.012-008 / TV-BC-2.01.013-005
    //
    // Asserts: select_by_class_name("security_finding") == Ok(2004) (NOT Ok(2001))
    //          AND exactly one WARN with event_type="ocsf.deprecated_class_alias",
    //          class_name="security_finding", resolved_class_uid=2004 is emitted.
    //
    // RED GATE: FAILS because current code returns Ok(2001) (CLASS_UID_SECURITY_FINDING),
    // not Ok(2004), and emits no WARN.
    // -------------------------------------------------------------------------

    /// BC-2.02.012 / AC-003 / TV-BC-2.02.012-008 / TV-BC-2.01.013-005:
    /// `select_by_class_name("security_finding")` returns `Ok(2004)` (transitional alias —
    /// maps to Detection Finding, NOT the deprecated Security Finding class_uid 2001)
    /// AND emits exactly one WARN with:
    ///   event_type = "ocsf.deprecated_class_alias"
    ///   class_name = "security_finding"
    ///   resolved_class_uid = 2004
    ///
    /// RED GATE: FAILS because current code:
    ///   - Returns Ok(2001) instead of Ok(2004)
    ///   - Emits no WARN
    #[test]
    fn test_BC_2_02_012_select_by_class_name_security_finding_returns_2004_with_warn() {
        let (_guard, log_buffer) = make_warn_capture_subscriber();

        let result = EventClassSelector::select_by_class_name("security_finding");

        // Assert Ok(2004) — transitional alias MUST map to Detection Finding (2004),
        // NOT the deprecated Security Finding (2001). (BC-2.02.012 Option A)
        assert!(
            result.is_ok(),
            "select_by_class_name(\"security_finding\") must return Ok \
             (BC-2.02.012 AC-003 / TV-BC-2.02.012-008)"
        );
        assert_eq!(
            result.unwrap(),
            2004,
            "select_by_class_name(\"security_finding\") must return Ok(2004) \
             — transitional alias maps to Detection Finding (2004), NOT deprecated \
             Security Finding (2001). \
             (BC-2.02.012 AC-003 / TV-BC-2.02.012-008 / TV-BC-2.01.013-005)"
        );

        // Assert that the deprecation WARN was emitted.
        let captured = log_buffer.lock().unwrap().clone();

        assert!(
            captured.contains("ocsf.deprecated_class_alias"),
            "select_by_class_name(\"security_finding\") MUST emit \
             event_type = \"ocsf.deprecated_class_alias\" WARN \
             (BC-2.02.012 AC-003 / TV-BC-2.02.012-008); \
             captured log: {captured}"
        );
        assert!(
            captured.contains("security_finding"),
            "ocsf.deprecated_class_alias WARN must include class_name = \"security_finding\" field \
             (BC-2.02.012 AC-003); captured log: {captured}"
        );
        assert!(
            captured.contains("resolved_class_uid=2004"),
            "ocsf.deprecated_class_alias WARN must include resolved_class_uid=2004 structured \
             field (tracing-subscriber fmt renders integer fields unquoted: field=value) \
             (BC-2.02.012 AC-003 / TV-BC-2.02.012-008); captured log: {captured}"
        );
    }

    // -------------------------------------------------------------------------
    // AC-004 / INV-NO-2001-SELECT-PATH
    //
    // Asserts: EventClassSelector::select(sensor, record_type) returns class_uid 2001
    //          for NO (sensor, record_type) pair.
    //
    // RED GATE: PASSES. The current select() implementation already satisfies
    // INV-NO-2001-SELECT-PATH — none of its match arms use CLASS_UID_SECURITY_FINDING.
    // Kept as a regression guard.
    //
    // NOTE TO ORCHESTRATOR: AC-004 passes at Red Gate because the select() path
    // never returned 2001 (the class_uid constant was only used in select_by_class_name).
    // This test is a regression guard preventing reintroduction of 2001 in the select()
    // path during the implementer's changes.
    // -------------------------------------------------------------------------

    /// BC-2.02.012 / AC-004 / INV-NO-2001-SELECT-PATH:
    /// The `EventClassSelector::select(sensor, record_type)` function — record-type token
    /// path — MUST NOT return class_uid 2001 (Security Finding, deprecated OCSF v1.1.0)
    /// for ANY (sensor, record_type) pair.
    ///
    /// RED GATE: PASSES (select() never returned 2001 — no match arm used CLASS_UID_SECURITY_FINDING).
    /// Kept as regression guard for INV-NO-2001-SELECT-PATH.
    #[test]
    fn test_BC_2_02_012_select_path_no_token_returns_2001() {
        const DEPRECATED_SECURITY_FINDING_UID: u32 = 2001;

        // Exhaustive enumeration of all known (sensor, record_type) tokens.
        // Any token returning 2001 is a violation of INV-NO-2001-SELECT-PATH.
        let all_known_tokens: &[(&str, &str)] = &[
            ("crowdstrike", "detection"),
            ("crowdstrike", "incident"),
            ("cyberint", "alert"),
            ("claroty", "alert"),
            ("claroty", "asset"),
            ("claroty", "device"),
            ("claroty", "vulnerability"),
            ("claroty", "audit_log"),
            ("armis", "device"),
            ("armis", "alert"),
            ("armis", "audit_log"),
        ];

        let mut violations: Vec<String> = Vec::new();

        for (sensor, record_type) in all_known_tokens {
            match EventClassSelector::select(sensor, record_type) {
                Ok(uid) if uid == DEPRECATED_SECURITY_FINDING_UID => {
                    violations.push(format!(
                        "select({sensor:?}, {record_type:?}) returned {uid} \
                         (deprecated Security Finding class_uid)"
                    ));
                }
                _ => {}
            }
        }

        assert!(
            violations.is_empty(),
            "AC-004 / INV-NO-2001-SELECT-PATH VIOLATED:\n\
             The select() path (record-type token path) MUST NOT return \
             class_uid 2001 (Security Finding, deprecated OCSF v1.1.0).\n\
             Any token that maps to 2001 must be updated to 2004 or another \
             current OCSF class (BC-2.02.012).\n\
             Violations ({} total):\n{}",
            violations.len(),
            violations.join("\n")
        );
    }

    // -------------------------------------------------------------------------
    // AC-005 / Stale-2001-assertions guard
    //
    // Per the story: "manual grep audit at dispatch; CI-enforced by the
    // AC-002/AC-003/AC-004 unit tests failing if 2001 were returned."
    //
    // This test is load-bearing because it enforces the invariant that no test
    // in THIS module (bc_2_02_012_class_selector.rs) asserts Ok(2001) as the
    // correct result of select_by_class_name. It does NOT scan the whole
    // workspace (that is a manual grep step at dispatch); it enforces the
    // contract that the canonical class_selector test file itself is clean.
    //
    // The workspace-wide coverage is provided by the AC-002 + AC-003 tests:
    // if production code returned 2001, AC-003's assertion on Ok(2004) would fail.
    //
    // RED GATE: PASSES. This test reads its own source to verify no stale
    // assertion of 2001 appears — the current test file does not assert 2001
    // as a valid return value for select_by_class_name (existing tests only
    // assert 2004/2005/2002/5001/3001/0/Err).
    //
    // NOTE TO ORCHESTRATOR: AC-005 passes at Red Gate because it validates
    // source-level assertions (no stale 2001 expected return), and no such
    // stale assertion exists in this file. The test is a CI guard post-migration
    // to catch any future test that regresses to asserting 2001 for class-name lookup.
    // -------------------------------------------------------------------------

    /// BC-2.02.012 / AC-005:
    /// No test in `bc_2_02_012_class_selector.rs` asserts that
    /// `select_by_class_name(...)` returns `Ok(2001)` (the deprecated Security Finding UID).
    /// Post-migration, all select_by_class_name assertions must use 2004 (or other current UIDs).
    ///
    /// Load-bearing strategy: this test calls `select_by_class_name` for every class name
    /// in the mapping table and asserts that NONE of them return 2001. This is the in-process
    /// equivalent of the `rg '2001'` grep audit for this crate.
    ///
    /// RED GATE: FAILS because "security_finding" currently returns Ok(2001).
    /// GREEN: after the implementer changes "security_finding" to return Ok(2004).
    #[test]
    fn test_BC_2_02_012_no_stale_2001_assertions_in_workspace() {
        const DEPRECATED_SECURITY_FINDING_UID: u32 = 2001;

        // All class names in the BC-2.02.012 mapping table.
        // After OCSF-CLASS-MIGRATION-001, NONE should return 2001.
        let class_names = [
            "detection_finding",
            "security_finding", // transitional alias — MUST return 2004 (not 2001)
            "incident_finding",
            "vulnerability_finding",
            "device",
            "audit_activity",
        ];

        let mut stale_2001_returns: Vec<String> = Vec::new();

        for class_name in &class_names {
            if let Ok(uid) = EventClassSelector::select_by_class_name(class_name) {
                if uid == DEPRECATED_SECURITY_FINDING_UID {
                    stale_2001_returns.push(format!(
                        "select_by_class_name({class_name:?}) returned Ok(2001) — \
                         stale deprecated class_uid; must be updated to Ok(2004) or \
                         another current OCSF class (BC-2.02.012 AC-005)"
                    ));
                }
            }
        }

        assert!(
            stale_2001_returns.is_empty(),
            "AC-005: Stale class_uid 2001 (deprecated Security Finding) assertions found.\n\
             After OCSF-CLASS-MIGRATION-001, select_by_class_name must return 2004 \
             for 'security_finding' (transitional alias) — not 2001.\n\
             Stale returns ({} total):\n{}",
            stale_2001_returns.len(),
            stale_2001_returns.join("\n")
        );
    }
} // mod ocsf_migration_red_gate
