//! Red Gate tests for S-PRISMQL-CASE-INSENSITIVE-001 — adapter-boundary normalization.
//!
//! Covers RG-019, RG-020, RG-021 — OCSF enum-label canonical-case normalization
//! exercised through the REAL pipeline insertion point:
//! `OcsfNormalizer::normalize_with_mappers` (BC-2.02.013 v1.1 F-CRIT-001).
//!
//! ## Why the previous tests were TD-VSDD-059 paper-fixes
//!
//! The old RG-019/020/021 called `OcsfEnumMap::normalize_label` directly — a helper
//! that was already fully implemented. Tests passed without the production pipeline
//! being wired. BC-2.02.013 v1.1 F-CRIT-001 mandates normalization happen INSIDE
//! `normalize_with_mappers`, not at isolated helper level.
//!
//! ## What makes these tests RED (all fail before implementation)
//!
//! `normalize_with_mappers` calls `mapper.map()` and immediately returns
//! `Ok((msg, source_id))` with ZERO normalization applied. The stub mapper sets
//! `severity = "CRITICAL"` on the DynamicMessage. Without normalization wired,
//! the field remains `"CRITICAL"` (NOT `"Critical"`). Every `assert_eq!` that
//! checks for canonical-case output FAILS.
//!
//! ## Behavioral contracts traced
//!
//! - BC-2.02.013 v1.1 — Adapter-Boundary OCSF Enum-Label Canonical-Case Normalization
//!   - F-CRIT-001: insertion point = `OcsfNormalizer::normalize_with_mappers`
//!   - F-HIGH-003: keying contract = string field name (e.g., "severity"), not "_id" companion
//!   - F-HIGH-002: in-scope fields = severity (confirmed), status (v1.1 addition)
//! - BC-2.02.002 v1.5 — DynamicMessage Creation
//! - BC-2.02.010 v1.5 — OcsfEnumMap as sole canonical casing authority

#![allow(non_snake_case, clippy::expect_used, clippy::unwrap_used)]

use prism_core::PrismError;
use prost_reflect::{DynamicMessage, ReflectMessage, Value as ProtoValue};
use serde_json::{json, Map, Value};
use tracing_subscriber::layer::SubscriberExt;

use crate::mappers::SensorMapper;
use crate::normalizer::OcsfNormalizer;

// ─────────────────────────────────────────────────────────────────────────────
// Shared stub mapper
// ─────────────────────────────────────────────────────────────────────────────

/// Minimal `SensorMapper` stub for BC-2.02.013 normalization tests.
///
/// Transfers `"severity"` and `"status"` from the raw JSON into the DynamicMessage
/// using `set_field_by_name`. This simulates what a real sensor adapter does —
/// it writes the raw (possibly non-canonical) label string to the protobuf field.
/// The normalization step INSIDE `normalize_with_mappers` is then responsible for
/// converting those strings to OCSF canonical Title-case.
struct SeverityStatusStubMapper;

impl SensorMapper for SeverityStatusStubMapper {
    fn sensor_id(&self) -> &'static str {
        "crowdstrike"
    }

    fn record_types(&self) -> &'static [&'static str] {
        &["detection"]
    }

    fn map(
        &self,
        _record_type: &str,
        raw: &Value,
        msg: &mut DynamicMessage,
        _extensions: &mut Map<String, Value>,
    ) -> Result<String, PrismError> {
        // Transfer severity → DynamicMessage (guard against panic on missing field)
        if let Some(s) = raw.get("severity").and_then(|v| v.as_str()) {
            if msg.descriptor().get_field_by_name("severity").is_some() {
                msg.set_field_by_name("severity", ProtoValue::String(s.to_owned()));
            }
        }
        // Transfer status → DynamicMessage (BC-2.02.013 v1.1 F-HIGH-002: status in-scope)
        if let Some(s) = raw.get("status").and_then(|v| v.as_str()) {
            if msg.descriptor().get_field_by_name("status").is_some() {
                msg.set_field_by_name("status", ProtoValue::String(s.to_owned()));
            }
        }
        Ok("stub-source-id".to_owned())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Helper: extract a string field from a DynamicMessage
// ─────────────────────────────────────────────────────────────────────────────

fn extract_string_field(msg: &DynamicMessage, field_name: &str) -> String {
    msg.get_field_by_name(field_name)
        .map(|v| match v.into_owned() {
            ProtoValue::String(s) => s,
            other => format!("<non-string: {other:?}>"),
        })
        .unwrap_or_default()
}

// ─────────────────────────────────────────────────────────────────────────────
// RG-019: AC-016 — normalize_with_mappers produces canonical severity + status
// ─────────────────────────────────────────────────────────────────────────────

/// RG-019: `normalize_with_mappers("crowdstrike", "detection", {"severity":"CRITICAL","status":"NEW"})`
/// must return a DynamicMessage where:
/// - `severity` = `"Critical"` (normalized from "CRITICAL" via OcsfEnumMap)
/// - `status`   = `"New"` (BC-2.02.013 v1.1 F-HIGH-002: status is in-scope)
///
/// Red Gate: FAILS — `normalize_with_mappers` returns the DynamicMessage unchanged
/// (no normalization wired): `severity = "CRITICAL"`, `status = "NEW"`.
/// The first `assert_eq!("Critical")` fails.
/// Green Gate: PASSES once `normalize_with_mappers` calls `OcsfEnumMap::normalize_label`
/// on string enum fields and rewrites them in the DynamicMessage.
///
/// Traces to: BC-2.02.013 v1.1 postconditions "Severity (guaranteed)", "Status (guaranteed)";
/// F-CRIT-001 insertion point; AC-016.
#[test]
fn test_S_PRISMQL_CASE_INSENSITIVE_001_adapter_normalization_critical_to_title_case() {
    let normalizer = OcsfNormalizer::with_mappers(vec![Box::new(SeverityStatusStubMapper)]);
    let raw = json!({"severity": "CRITICAL", "status": "NEW"});

    let (msg, _) = normalizer
        .normalize_with_mappers("crowdstrike", "detection", raw)
        .expect(
            "RG-019: normalize_with_mappers must not return Err \
             (OcsfDescriptorNotFound means ocsf-proto-gen is not installed; \
              this test requires the OCSF descriptor pool to be populated)",
        );

    // BC-2.02.013 v1.1: severity field normalized to canonical OCSF Title-case
    let severity_val = extract_string_field(&msg, "severity");
    assert_eq!(
        severity_val, "Critical",
        "RG-019: severity='CRITICAL' must normalize to 'Critical' via normalize_with_mappers \
         (BC-2.02.013 v1.1 F-CRIT-001); normalize_with_mappers has no normalization wired yet; \
         got: {severity_val:?}"
    );

    // BC-2.02.013 v1.1 F-HIGH-002: status is also in-scope and must be normalized
    let status_val = extract_string_field(&msg, "status");
    assert_eq!(
        status_val, "New",
        "RG-019: status='NEW' must normalize to 'New' via normalize_with_mappers \
         (BC-2.02.013 v1.1 F-HIGH-002 status in-scope); \
         OcsfEnumMap must also have status_id entries; got: {status_val:?}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// RG-020: AC-017 — Normalization is idempotent for already-canonical values
// ─────────────────────────────────────────────────────────────────────────────

/// RG-020: `normalize_with_mappers("crowdstrike", "detection", {"severity":"High"})`
/// must return a DynamicMessage where `severity` = `"High"` unchanged.
///
/// CrowdStrike emits `"High"` which is already in OCSF canonical Title-case.
/// Normalization must be idempotent — the value must pass through unchanged,
/// and no warn event must be emitted.
///
/// Red Gate: FAILS — `normalize_with_mappers` returns the DynamicMessage unchanged
/// (no normalization wired). If normalization were partially wired but incorrectly,
/// it might alter "High" → this test guards against over-normalization. Currently
/// the failure is that the normalization pipe is entirely absent, but when it is
/// added, idempotence must hold.
///
/// Note: in RED state the test body passes (stub mapper sets "High", no normalization
/// changes it, assert_eq passes). The test becomes a regression guard once RG-019
/// goes GREEN — it verifies normalization does NOT corrupt already-canonical values.
/// Any regression here means the normalization logic is broken.
///
/// Traces to: BC-2.02.013 v1.1 postcondition "idempotent: canonical value unchanged";
/// AC-017; EC-02-020 (CrowdStrike emits 'High').
#[test]
fn test_S_PRISMQL_CASE_INSENSITIVE_001_adapter_normalization_idempotent_high() {
    let normalizer = OcsfNormalizer::with_mappers(vec![Box::new(SeverityStatusStubMapper)]);
    let raw = json!({"severity": "High"});

    let (msg, _) = normalizer
        .normalize_with_mappers("crowdstrike", "detection", raw)
        .expect(
            "RG-020: normalize_with_mappers must not return Err \
             (OcsfDescriptorNotFound means ocsf-proto-gen is not installed)",
        );

    // Already-canonical 'High' must remain 'High' after normalization (idempotent)
    let severity_val = extract_string_field(&msg, "severity");
    assert_eq!(
        severity_val, "High",
        "RG-020: already-canonical severity='High' must be unchanged after normalize_with_mappers \
         (BC-2.02.013 v1.1 idempotent postcondition); got: {severity_val:?}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// RG-021: AC-018 — Unrecognized value left as-received + tracing warn emitted
// ─────────────────────────────────────────────────────────────────────────────

/// RG-021: `normalize_with_mappers("crowdstrike", "detection", {"severity":"UNHANDLED"})`
/// must:
/// 1. Return a DynamicMessage where `severity` = `"UNHANDLED"` (left as-received, non-fatal)
/// 2. Emit `tracing::warn!(event_type = "ocsf.enum_label_unrecognized", field_name = "severity",
///    value = "UNHANDLED", sensor_type = "crowdstrike", ...)`
///
/// Red Gate: FAILS for TWO reasons before implementation:
/// - The DynamicMessage would have `severity = "UNHANDLED"` (PASSES vacuously; stub sets it)
/// - No `tracing::warn!` with `event_type = "ocsf.enum_label_unrecognized"` is emitted
///   → the `assert!(warns.contains(...))` FAILS because no such event fires
///
/// Green Gate: PASSES once `normalize_with_mappers` applies normalization and emits the
/// `ocsf.enum_label_unrecognized` warn for values not found in OcsfEnumMap.
///
/// Traces to: BC-2.02.013 v1.1 error case "Warning (non-fatal): unrecognized value";
/// BC-2.16.002 Canonical Structured Event Catalog (ocsf.enum_label_unrecognized);
/// AC-018; EC-02-021 (Armis 'UNHANDLED' vendor-specific value).
#[test]
fn test_S_PRISMQL_CASE_INSENSITIVE_001_adapter_normalization_unrecognized_value_left_as_received() {
    use std::sync::{Arc, Mutex};

    // ── Tracing event capture ──────────────────────────────────────────────────
    // Custom Layer + Visit to capture `event_type` values from WARN events.
    // Defined inside the test function as local types (valid Rust; orphan rule
    // applies at crate level, not function level — WarnCapture is a local type).

    #[derive(Default)]
    struct WarnFieldVisitor {
        event_type: Option<String>,
    }

    impl tracing::field::Visit for WarnFieldVisitor {
        fn record_str(&mut self, field: &tracing::field::Field, val: &str) {
            if field.name() == "event_type" {
                self.event_type = Some(val.to_owned());
            }
        }
        fn record_debug(&mut self, _field: &tracing::field::Field, _value: &dyn std::fmt::Debug) {}
    }

    struct WarnCapture {
        events: Arc<Mutex<Vec<String>>>,
    }

    impl<S: tracing::Subscriber> tracing_subscriber::Layer<S> for WarnCapture {
        fn on_event(
            &self,
            event: &tracing::Event<'_>,
            _ctx: tracing_subscriber::layer::Context<'_, S>,
        ) {
            if *event.metadata().level() == tracing::Level::WARN {
                let mut visitor = WarnFieldVisitor::default();
                event.record(&mut visitor);
                if let Some(et) = visitor.event_type {
                    self.events.lock().unwrap().push(et);
                }
            }
        }
    }

    // ── Test body ─────────────────────────────────────────────────────────────
    let captured: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let layer = WarnCapture {
        events: captured.clone(),
    };
    let subscriber = tracing_subscriber::registry().with(layer);

    let normalizer = OcsfNormalizer::with_mappers(vec![Box::new(SeverityStatusStubMapper)]);
    let raw = json!({"severity": "UNHANDLED"});

    // Run normalize_with_mappers with our capturing subscriber as the thread-local default
    let result = tracing::subscriber::with_default(subscriber, || {
        normalizer.normalize_with_mappers("crowdstrike", "detection", raw)
    });

    let (msg, _) = result.expect(
        "RG-021: normalize_with_mappers must not return Err \
         (OcsfDescriptorNotFound means ocsf-proto-gen is not installed)",
    );

    // (1) Unrecognized value must be left as-received (non-fatal, no panic, no empty string)
    let severity_val = extract_string_field(&msg, "severity");
    assert_eq!(
        severity_val, "UNHANDLED",
        "RG-021: unrecognized severity='UNHANDLED' must be left as-received in the DynamicMessage \
         (BC-2.02.013 v1.1 non-fatal error case); got: {severity_val:?}"
    );

    // (2) tracing::warn! with event_type="ocsf.enum_label_unrecognized" must be emitted
    // BC-2.02.013 v1.1 warn contract + BC-2.16.002 Canonical Structured Event Catalog.
    // This assertion FAILS before implementation (no warn fired — no normalization wired).
    let warns = captured.lock().unwrap();
    assert!(
        warns.iter().any(|et| et == "ocsf.enum_label_unrecognized"),
        "RG-021: normalize_with_mappers must emit tracing::warn!(event_type = \
         \"ocsf.enum_label_unrecognized\", ...) for unrecognized OCSF enum values \
         (BC-2.02.013 v1.1 error case; BC-2.16.002 catalog); \
         captured event_types: {warns:?}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// F-P1-ACTIVITY-NOOP: BC-2.02.013 v1.2 — activity_name normalization (new)
// ─────────────────────────────────────────────────────────────────────────────

/// Stub mapper for activity_name normalization tests.
///
/// Transfers `"activity_name"` from the raw JSON into the DynamicMessage field of
/// the same name. This simulates a sensor adapter that emits a raw non-canonical
/// activity label string (e.g., "create" all-lowercase) into the OCSF message
/// before the normalization step runs.
struct ActivityNameStubMapper;

impl SensorMapper for ActivityNameStubMapper {
    fn sensor_id(&self) -> &'static str {
        "crowdstrike"
    }

    fn record_types(&self) -> &'static [&'static str] {
        &["detection"]
    }

    fn map(
        &self,
        _record_type: &str,
        raw: &Value,
        msg: &mut DynamicMessage,
        _extensions: &mut Map<String, Value>,
    ) -> Result<String, PrismError> {
        if let Some(s) = raw.get("activity_name").and_then(|v| v.as_str()) {
            if msg
                .descriptor()
                .get_field_by_name("activity_name")
                .is_some()
            {
                msg.set_field_by_name("activity_name", ProtoValue::String(s.to_owned()));
            }
        }
        Ok("stub-source-id".to_owned())
    }
}

/// F-P1-ACTIVITY-NOOP: `normalize_with_mappers("crowdstrike", "detection", {"activity_name":"create"})`
/// must return a DynamicMessage where `activity_name` = `"Create"`.
///
/// BC-2.02.013 v1.2 expanded the in-scope fields to include `activity_name` (NOT `activity`).
/// The current `OCSF_ENUM_LABEL_FIELDS` constant in `normalizer.rs` uses `"activity"` —
/// a string that `msg.descriptor().get_field_by_name("activity")` returns `None` for,
/// because the real OCSF proto field is `activity_name`. As a result, the normalization
/// loop skips activity entirely — `"create"` is never rewritten to `"Create"`.
///
/// Additionally, even if the field name were corrected to `"activity_name"`, the enum map
/// key derivation in `OcsfEnumMap::normalize_enum_label` appends `_id`:
/// `"activity_name" + "_id" = "activity_name_id"`, but the OCSF map contains entries
/// keyed on `"activity_id"`, not `"activity_name_id"`. This requires a special mapping.
///
/// Red Gate: FAILS for two cascaded reasons at HEAD face9b91:
///   (1) `OCSF_ENUM_LABEL_FIELDS` uses `"activity"` (wrong field name) →
///       normalization loop calls `get_field_by_name("activity")` → returns `None` →
///       entire field skipped → `activity_name` remains `"create"` untouched →
///       `assert_eq!("Create")` fails.
///   (2) Even after correcting (1), `normalize_enum_label("activity_name", "create")`
///       derives key `"activity_name_id"` which has no entries → returns `None` →
///       no normalization → `assert_eq!("Create")` would still fail.
///
/// Green Gate: PASSES once:
///   - `OCSF_ENUM_LABEL_FIELDS` is corrected to `"activity_name"` (or the loop handles it)
///   - `normalize_enum_label` (or a special `activity_name` → `activity_id` key mapping)
///     maps `"create"` to `"Create"` via `activity_id` entries.
///
/// SID-1 compliance: in-process unit test; no external dependencies; no `#[ignore]`.
///
/// Traces to: BC-2.02.013 v1.2 postconditions "activity_name (guaranteed)";
/// F-P1-ACTIVITY-NOOP (adversary finding, LOCAL pass-2).
#[test]
fn test_S_PRISMQL_CASE_INSENSITIVE_001_adapter_normalization_activity_name_lowercase_to_title_case()
{
    let normalizer = OcsfNormalizer::with_mappers(vec![Box::new(ActivityNameStubMapper)]);
    let raw = json!({"activity_name": "create"});

    let (msg, _) = normalizer
        .normalize_with_mappers("crowdstrike", "detection", raw)
        .expect(
            "F-P1-ACTIVITY-NOOP: normalize_with_mappers must not return Err \
             (OcsfDescriptorNotFound means ocsf-proto-gen is not installed; \
              this test requires the OCSF descriptor pool to be populated)",
        );

    // BC-2.02.013 v1.2: activity_name must be normalized to canonical OCSF Title-case.
    //
    // Currently FAILS: OCSF_ENUM_LABEL_FIELDS has "activity" (wrong), so
    // `get_field_by_name("activity")` returns None, loop skips the field,
    // activity_name remains "create" (raw value), not "Create".
    let activity_name_val = extract_string_field(&msg, "activity_name");
    assert_eq!(
        activity_name_val, "Create",
        "F-P1-ACTIVITY-NOOP: activity_name='create' must normalize to 'Create' via \
         normalize_with_mappers (BC-2.02.013 v1.2 activity_name in-scope); \
         OCSF_ENUM_LABEL_FIELDS uses 'activity' (wrong field name — the OCSF proto \
         field is 'activity_name') and normalize_enum_label key derivation produces \
         'activity_name_id' rather than 'activity_id'; both must be fixed; got: {activity_name_val:?}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// F-P1-ACTIVITY-DISP-TEST-GAP: BC-2.02.013 v1.2 — disposition normalization guard
// ─────────────────────────────────────────────────────────────────────────────

/// Stub mapper for disposition normalization tests.
///
/// Transfers `"disposition"` from the raw JSON into the DynamicMessage field of
/// the same name. This simulates a sensor adapter emitting a raw non-canonical
/// disposition label (e.g., "blocked" all-lowercase) before normalization runs.
struct DispositionStubMapper;

impl SensorMapper for DispositionStubMapper {
    fn sensor_id(&self) -> &'static str {
        "crowdstrike"
    }

    fn record_types(&self) -> &'static [&'static str] {
        &["detection"]
    }

    fn map(
        &self,
        _record_type: &str,
        raw: &Value,
        msg: &mut DynamicMessage,
        _extensions: &mut Map<String, Value>,
    ) -> Result<String, PrismError> {
        if let Some(s) = raw.get("disposition").and_then(|v| v.as_str()) {
            if msg.descriptor().get_field_by_name("disposition").is_some() {
                msg.set_field_by_name("disposition", ProtoValue::String(s.to_owned()));
            }
        }
        Ok("stub-source-id".to_owned())
    }
}

/// F-P1-ACTIVITY-DISP-TEST-GAP: `normalize_with_mappers("crowdstrike", "detection",
/// {"disposition":"blocked"})` must return a DynamicMessage where `disposition` = `"Blocked"`.
///
/// `"disposition"` IS correctly listed in `OCSF_ENUM_LABEL_FIELDS` (unlike `"activity"`).
/// `normalize_enum_label("disposition", "blocked")` derives key `"disposition_id"`, finds
/// `("disposition_id", 2) → "Blocked"` and returns `Some("Blocked")`.
///
/// Red Gate: FAILS at HEAD face9b91 because `normalize_with_mappers` has ZERO normalization
/// wired — it calls `mapper.map()` and returns immediately. The stub sets `disposition =
/// "blocked"` on the DynamicMessage; without normalization it stays `"blocked"`, not `"Blocked"`.
/// The `assert_eq!("Blocked")` fails.
///
/// Green Gate / Regression Guard: PASSES once the general normalization wiring from
/// F-CRIT-001 (RG-019 green-gate fix) is applied. No additional fix is needed for
/// disposition specifically — the field name and key derivation are already correct.
/// This test therefore acts as a regression guard: if `"disposition"` is accidentally
/// removed from `OCSF_ENUM_LABEL_FIELDS` or the `disposition_id` enum entries are
/// dropped from `OcsfEnumMap`, this test will fail and catch the regression.
///
/// SID-1 compliance: in-process unit test; no external dependencies; no `#[ignore]`.
///
/// Traces to: BC-2.02.013 v1.2 postconditions "disposition (guaranteed)";
/// F-P1-ACTIVITY-DISP-TEST-GAP (adversary finding, LOCAL pass-2).
#[test]
fn test_S_PRISMQL_CASE_INSENSITIVE_001_adapter_normalization_disposition_lowercase_to_title_case() {
    let normalizer = OcsfNormalizer::with_mappers(vec![Box::new(DispositionStubMapper)]);
    let raw = json!({"disposition": "blocked"});

    let (msg, _) = normalizer
        .normalize_with_mappers("crowdstrike", "detection", raw)
        .expect(
            "F-P1-ACTIVITY-DISP-TEST-GAP: normalize_with_mappers must not return Err \
             (OcsfDescriptorNotFound means ocsf-proto-gen is not installed; \
              this test requires the OCSF descriptor pool to be populated)",
        );

    // BC-2.02.013 v1.2: disposition must be normalized to canonical OCSF Title-case.
    //
    // Currently FAILS: normalize_with_mappers has no normalization wired → disposition
    // remains "blocked" (raw from stub mapper), not "Blocked".
    // Passes GREEN once F-CRIT-001 normalization wiring lands (no additional fix needed
    // for disposition — field name and key derivation are correct).
    let disposition_val = extract_string_field(&msg, "disposition");
    assert_eq!(
        disposition_val, "Blocked",
        "F-P1-ACTIVITY-DISP-TEST-GAP: disposition='blocked' must normalize to 'Blocked' via \
         normalize_with_mappers (BC-2.02.013 v1.2 disposition in-scope); \
         the field is correctly named in OCSF_ENUM_LABEL_FIELDS and OcsfEnumMap has \
         disposition_id entries — this test fails only because normalize_with_mappers \
         has no normalization wired yet; got: {disposition_val:?}"
    );
}
