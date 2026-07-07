//! Red Gate tests for S-PRISMQL-CASE-INSENSITIVE-001 — adapter-boundary normalization.
//!
//! Covers RG-019, RG-020, RG-021 — OCSF enum-label canonical-case normalization
//! exercised through the REAL pipeline insertion point:
//! `OcsfNormalizer::normalize_with_mappers` (BC-2.02.013 F-CRIT-001).
//!
//! ## Why the previous tests were TD-VSDD-059 paper-fixes
//!
//! The old RG-019/020/021 called `OcsfEnumMap::normalize_label` directly — a helper
//! that was already fully implemented. Tests passed without the production pipeline
//! being wired. BC-2.02.013 F-CRIT-001 mandates normalization happen INSIDE
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
//! - BC-2.02.013 — Adapter-Boundary OCSF Enum-Label Canonical-Case Normalization
//!   - F-CRIT-001: insertion point = `OcsfNormalizer::normalize_with_mappers`
//!   - F-HIGH-003: keying contract = string field name (e.g., "severity"), not "_id" companion
//!   - F-HIGH-002: in-scope fields = severity (confirmed), status (v1.1 addition)
//! - BC-2.02.002 v1.5 — DynamicMessage Creation
//! - BC-2.02.010 v1.5 — OcsfEnumMap as sole canonical casing authority
//!
//! ## Production caller status (OBS-003)
//!
//! `normalize_with_mappers` has zero production callers today — it is defined
//! in `normalizer.rs` but called only from test code and integration fixtures.
//! These tests lock the contract for the future protobuf-export path where a
//! real `SensorMapper` will be wired (per BC-2.02.013).  Until that wiring
//! lands the tests act as forward-compatibility guardrails.

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
        // Transfer status → DynamicMessage (BC-2.02.013 F-HIGH-002: status in-scope)
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
/// - `status`   = `"New"` (BC-2.02.013 F-HIGH-002: status is in-scope)
///
/// Regression guard: PASSES — normalization wired in F-CRIT-001 fix-burst.
/// `normalize_with_mappers` applies `OcsfEnumMap::normalize_label` to string enum
/// fields and rewrites them in the DynamicMessage. This test guards against regression
/// (re-breaking the normalization wiring).
///
/// Traces to: BC-2.02.013 postconditions "Severity (guaranteed)", "Status (guaranteed)";
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

    // BC-2.02.013: severity field normalized to canonical OCSF Title-case
    let severity_val = extract_string_field(&msg, "severity");
    assert_eq!(
        severity_val, "Critical",
        "RG-019: severity='CRITICAL' must normalize to 'Critical' via normalize_with_mappers \
         (BC-2.02.013 F-CRIT-001); got: {severity_val:?}"
    );

    // BC-2.02.013 F-HIGH-002: status is also in-scope and must be normalized
    let status_val = extract_string_field(&msg, "status");
    assert_eq!(
        status_val, "New",
        "RG-019: status='NEW' must normalize to 'New' via normalize_with_mappers \
         (BC-2.02.013 F-HIGH-002 status in-scope); \
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
/// Regression guard: PASSES — normalization is idempotent for already-canonical values.
/// The normalization wiring added by F-CRIT-001 does NOT corrupt "High" to something else.
/// Any future regression in normalization that corrupts already-canonical values will cause
/// this test to fail.
///
/// Traces to: BC-2.02.013 postcondition "idempotent: canonical value unchanged";
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
         (BC-2.02.013 idempotent postcondition); got: {severity_val:?}"
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
/// Strengthened for LOCAL-pass-6 (F-P6-CRIT-001 / F-P6-HIGH-003):
/// In addition to the event_type check, this test now validates the full
/// BC-2.16.002 catalog row 91 schema: field_name, value, sensor_type.
/// The SECONDARY (normalizer.rs) already emits all three fields correctly —
/// these schema assertions PASS at HEAD (regression guards).
///
/// Regression guard: PASSES — `normalize_with_mappers` correctly emits all
/// catalog row 91 fields.
///
/// Traces to: BC-2.02.013 error case "Warning (non-fatal): unrecognized value";
/// BC-2.16.002 Canonical Structured Event Catalog (ocsf.enum_label_unrecognized);
/// AC-018; EC-02-021 (Armis 'UNHANDLED' vendor-specific value).
#[test]
fn test_S_PRISMQL_CASE_INSENSITIVE_001_adapter_normalization_unrecognized_value_left_as_received() {
    use std::sync::{Arc, Mutex};

    // ── Tracing event capture — catalog-complete field capture ─────────────────
    // Captures ALL BC-2.16.002 catalog row 91 fields: event_type, field_name,
    // value, sensor_type.

    #[derive(Default, Clone, Debug)]
    struct WarnEvent {
        event_type: Option<String>,
        field_name: Option<String>,
        value: Option<String>,
        sensor_type: Option<String>,
    }

    #[derive(Default)]
    struct WarnFieldVisitor {
        event: WarnEvent,
    }

    impl tracing::field::Visit for WarnFieldVisitor {
        fn record_str(&mut self, field: &tracing::field::Field, val: &str) {
            // Handles string literal fields (e.g. `event_type = "ocsf.enum_label_unrecognized"`).
            match field.name() {
                "event_type" => self.event.event_type = Some(val.to_owned()),
                "field_name" => self.event.field_name = Some(val.to_owned()),
                "value" => self.event.value = Some(val.to_owned()),
                "sensor_type" => self.event.sensor_type = Some(val.to_owned()),
                _ => {}
            }
        }
        fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
            // Handles `%`-formatted fields (e.g. `field_name = %field`, `value = %current`).
            // tracing routes Display-formatted values through record_debug; the
            // dyn Debug impl for format_args!("{}", x) delegates to Display, so
            // `format!("{value:?}")` gives the Display representation without extra quoting.
            let s = format!("{value:?}");
            match field.name() {
                "event_type" => {
                    if self.event.event_type.is_none() {
                        self.event.event_type = Some(s);
                    }
                }
                "field_name" => {
                    if self.event.field_name.is_none() {
                        self.event.field_name = Some(s);
                    }
                }
                "value" => {
                    if self.event.value.is_none() {
                        self.event.value = Some(s);
                    }
                }
                "sensor_type" => {
                    if self.event.sensor_type.is_none() {
                        self.event.sensor_type = Some(s);
                    }
                }
                _ => {}
            }
        }
    }

    struct WarnCapture {
        events: Arc<Mutex<Vec<WarnEvent>>>,
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
                if visitor.event.event_type.is_some() {
                    self.events.lock().unwrap().push(visitor.event);
                }
            }
        }
    }

    // ── Test body ─────────────────────────────────────────────────────────────
    let captured: Arc<Mutex<Vec<WarnEvent>>> = Arc::new(Mutex::new(Vec::new()));
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
         (BC-2.02.013 non-fatal error case); got: {severity_val:?}"
    );

    // (2) event_type = "ocsf.enum_label_unrecognized" must be emitted.
    // BC-2.02.013 warn contract + BC-2.16.002 catalog row 91.
    let warns = captured.lock().unwrap();
    assert!(
        warns
            .iter()
            .any(|e| e.event_type.as_deref() == Some("ocsf.enum_label_unrecognized")),
        "RG-021: normalize_with_mappers must emit tracing::warn!(event_type = \
         \"ocsf.enum_label_unrecognized\", ...) for unrecognized OCSF enum values \
         (BC-2.02.013 error case; BC-2.16.002 catalog); \
         captured events: {warns:?}"
    );

    // Locate the event for catalog-schema validation.
    let evt = warns
        .iter()
        .find(|e| e.event_type.as_deref() == Some("ocsf.enum_label_unrecognized"))
        .expect(
            "ocsf.enum_label_unrecognized event not found \
             (assertion 2 should have caught this)",
        );

    // (3) SECONDARY catalog schema: field_name must be "severity".
    // PASSES NOW: normalizer.rs already emits `field_name = %field`.
    // Regression guard: if field is accidentally renamed, this fails.
    assert_eq!(
        evt.field_name.as_deref(),
        Some("severity"),
        "RG-021 catalog schema: warn must carry field_name='severity' \
         (BC-2.16.002 catalog row 91); got: {:?}",
        evt.field_name
    );

    // (4) SECONDARY catalog schema: sensor_type must be "crowdstrike".
    // PASSES NOW: normalizer.rs already emits `sensor_type = %sensor`.
    // Regression guard: if sensor_type is accidentally dropped, this fails.
    assert_eq!(
        evt.sensor_type.as_deref(),
        Some("crowdstrike"),
        "RG-021 catalog schema: warn must carry sensor_type='crowdstrike' \
         (BC-2.16.002 catalog row 91); got: {:?}",
        evt.sensor_type
    );

    // (5) SECONDARY catalog schema: value must be "UNHANDLED".
    // PASSES NOW for this short value; see separate 50-cap test for the cap invariant.
    assert_eq!(
        evt.value.as_deref(),
        Some("UNHANDLED"),
        "RG-021 catalog schema: warn must carry value='UNHANDLED'; got: {:?}",
        evt.value
    );
}

/// F-P6-MED-001 (LOCAL pass-6): SECONDARY `ocsf.enum_label_unrecognized` warn in
/// `normalize_with_mappers` must cap the `value` field at 50 codepoints to bound log
/// volume for adversarially long vendor strings (SEC-002 pattern).
///
/// PRIMARY (`build_column_array` in `spec_driven_adapter.rs`) already caps at 50 via
/// `%s.chars().take(50).collect::<String>()`. SECONDARY (`normalizer.rs` line ~157)
/// emits `value = %current` WITHOUT any cap — the two emission sites must be symmetric.
///
/// ## Current behaviour (HEAD 8e4ec972) — RED
///
/// `normalize_with_mappers` receives `{"severity": "<60-char string>"}`. The warn fires
/// with `value = %current` where `current` is the full 60-char string. The assertion
/// `value.chars().count() <= 50` FAILS.
///
/// ## Green Gate
///
/// PASSES once `normalizer.rs` caps: `value = %current.chars().take(50).collect::<String>()`.
///
/// Traces to: BC-2.16.002 catalog row 91 (value: SEC-002 truncation);
/// F-P6-MED-001; LOCAL adversary pass-6.
#[test]
fn test_BC_2_02_013_normalizer_secondary_unrecognized_warn_value_capped_at_50_codepoints() {
    use std::sync::{Arc, Mutex};

    // 60 ASCII codepoints — 10 beyond the 50-codepoint cap.
    let long_value = "VENDOR".repeat(10); // "VENDORVENDOR...VENDOR" × 10 = 60 chars
    assert_eq!(
        long_value.chars().count(),
        60,
        "test precondition: input must be 60 codepoints"
    );

    #[derive(Default, Debug)]
    struct WarnValueVisitor {
        value: Option<String>,
        event_type: Option<String>,
    }

    impl tracing::field::Visit for WarnValueVisitor {
        fn record_str(&mut self, field: &tracing::field::Field, val: &str) {
            // Handles string literal fields.
            match field.name() {
                "event_type" => self.event_type = Some(val.to_owned()),
                "value" => self.value = Some(val.to_owned()),
                _ => {}
            }
        }
        fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
            // Handles `%`-formatted fields (e.g. `value = %current`).
            let s = format!("{value:?}");
            match field.name() {
                "event_type" => {
                    if self.event_type.is_none() {
                        self.event_type = Some(s);
                    }
                }
                "value" => {
                    if self.value.is_none() {
                        self.value = Some(s);
                    }
                }
                _ => {}
            }
        }
    }

    struct WarnCapture {
        captured_value: Arc<Mutex<Option<String>>>,
    }

    impl<S: tracing::Subscriber> tracing_subscriber::Layer<S> for WarnCapture {
        fn on_event(
            &self,
            event: &tracing::Event<'_>,
            _ctx: tracing_subscriber::layer::Context<'_, S>,
        ) {
            if *event.metadata().level() == tracing::Level::WARN {
                let mut visitor = WarnValueVisitor::default();
                event.record(&mut visitor);
                if visitor.event_type.as_deref() == Some("ocsf.enum_label_unrecognized") {
                    *self.captured_value.lock().unwrap() = visitor.value;
                }
            }
        }
    }

    let captured_value: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));
    let layer = WarnCapture {
        captured_value: captured_value.clone(),
    };
    let subscriber = tracing_subscriber::registry().with(layer);

    let normalizer = OcsfNormalizer::with_mappers(vec![Box::new(SeverityStatusStubMapper)]);
    let raw = json!({"severity": long_value.clone()});

    tracing::subscriber::with_default(subscriber, || {
        let _ = normalizer.normalize_with_mappers("crowdstrike", "detection", raw);
    });

    let captured = captured_value.lock().unwrap().clone();
    let captured_value_str = captured.as_deref().unwrap_or("");

    // Assert: warn value must be capped at 50 codepoints (symmetric with PRIMARY).
    // FAILS NOW: normalizer.rs emits `value = %current` (full 60-char string).
    // After fix: normalizer.rs caps with `.chars().take(50).collect::<String>()`.
    assert!(
        captured_value_str.chars().count() <= 50,
        "F-P6-MED-001 (LOCAL pass-6): SECONDARY ocsf.enum_label_unrecognized warn must cap \
         `value` at 50 codepoints (SEC-002 pattern, symmetric with PRIMARY in \
         build_column_array); \
         got {} codepoints in captured value {:?}. \
         Fix: change `value = %%current` to \
         `value = %%current.chars().take(50).collect::<String>()` in normalizer.rs.",
        captured_value_str.chars().count(),
        captured_value_str
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// OBS-1 SECONDARY: BC-2.02.013 — empty-string enum value passes through, no warn
// ─────────────────────────────────────────────────────────────────────────────

/// OBS-1 (LOCAL pass-12) — parity regression lock for the `!s.is_empty()` guard in
/// `normalizer.rs:146`.
///
/// An empty-string enum value passed through `normalize_with_mappers` MUST:
///
///  1. Pass through unchanged (the field remains `""` after normalization).
///  2. NOT emit an `ocsf.enum_label_unrecognized` warn event.
///
/// ## Why this is already GREEN
///
/// The guard `ProtoValue::String(s) if !s.is_empty() => s` at normalizer.rs:146 matches
/// an empty string as `_ => continue`, skipping the entire normalization block.
/// No warn is emitted; the field value is left as-is.
///
/// ## Why this test exists
///
/// This is a PARITY REGRESSION LOCK — mirror of the PRIMARY RG-047 (empty-string in
/// `build_column_array`). If the `!s.is_empty()` guard is accidentally removed in a
/// future refactor (e.g., while extending the field-type match), both the PRIMARY and
/// this SECONDARY test will fail immediately, preventing a silent regression where
/// every empty-string enum field triggers a spurious warn storm.
///
/// SID-1 compliance: in-process unit test; no external dependencies; no `#[ignore]`.
///
/// Traces to: BC-2.02.013 §Error Cases ("empty string: skip normalization, no warn");
/// OBS-1 (LOCAL adversary pass-12).
#[test]
fn test_BC_2_02_013_normalizer_secondary_empty_string_enum_value_no_warn() {
    use std::sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    };

    // Track whether ocsf.enum_label_unrecognized was emitted.
    let warn_fired = Arc::new(AtomicBool::new(false));

    struct WarnDetector {
        fired: Arc<AtomicBool>,
    }

    impl<S: tracing::Subscriber> tracing_subscriber::Layer<S> for WarnDetector {
        fn on_event(
            &self,
            event: &tracing::Event<'_>,
            _ctx: tracing_subscriber::layer::Context<'_, S>,
        ) {
            if *event.metadata().level() == tracing::Level::WARN {
                struct EventTypeVisitor(Option<String>);
                impl tracing::field::Visit for EventTypeVisitor {
                    fn record_str(&mut self, field: &tracing::field::Field, val: &str) {
                        if field.name() == "event_type" {
                            self.0 = Some(val.to_owned());
                        }
                    }
                    fn record_debug(
                        &mut self,
                        field: &tracing::field::Field,
                        value: &dyn std::fmt::Debug,
                    ) {
                        if field.name() == "event_type" && self.0.is_none() {
                            self.0 = Some(format!("{value:?}"));
                        }
                    }
                }
                let mut v = EventTypeVisitor(None);
                event.record(&mut v);
                if v.0.as_deref() == Some("ocsf.enum_label_unrecognized") {
                    self.fired.store(true, Ordering::SeqCst);
                }
            }
        }
    }

    let detector = WarnDetector {
        fired: warn_fired.clone(),
    };
    let subscriber = tracing_subscriber::registry().with(detector);

    // An empty-string value for "severity" — the stub mapper sets `severity = ""`.
    let normalizer = OcsfNormalizer::with_mappers(vec![Box::new(SeverityStatusStubMapper)]);
    let raw = json!({"severity": ""});

    let (msg, _) = tracing::subscriber::with_default(subscriber, || {
        normalizer
            .normalize_with_mappers("crowdstrike", "detection", raw)
            .expect(
                "OBS-1: normalize_with_mappers must not return Err for empty-string input \
                 (OcsfDescriptorNotFound means ocsf-proto-gen is not installed; \
                  this test requires the OCSF descriptor pool to be populated)",
            )
    });

    // Assert 1: no warn was emitted (the !s.is_empty() guard skips empty strings entirely).
    assert!(
        !warn_fired.load(Ordering::SeqCst),
        "OBS-1: normalize_with_mappers MUST NOT emit ocsf.enum_label_unrecognized for an \
         empty-string enum value. The `!s.is_empty()` guard at normalizer.rs:146 must be \
         preserved — if this test fails, the guard was removed or bypassed."
    );

    // Assert 2: the field passes through unchanged (empty string remains empty string).
    let severity_val = extract_string_field(&msg, "severity");
    assert_eq!(
        severity_val, "",
        "OBS-1: empty-string severity value must pass through unchanged after normalization; \
         got: {severity_val:?}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// F-P1-ACTIVITY-NOOP: BC-2.02.013 — activity_name normalization (new)
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
/// BC-2.02.013 expanded the in-scope fields to include `activity_name` (NOT `activity`).
/// The current `OCSF_ENUM_LABEL_FIELDS` constant in `normalizer.rs` uses `"activity"` —
/// a string that `msg.descriptor().get_field_by_name("activity")` returns `None` for,
/// because the real OCSF proto field is `activity_name`. As a result, the normalization
/// loop skips activity entirely — `"create"` is never rewritten to `"Create"`.
///
/// Regression guard: PASSES — both root causes were fixed:
///   (1) `OCSF_ENUM_LABEL_FIELDS` updated to `"activity_name"` (correct OCSF proto field name)
///   (2) `normalize_enum_label` uses a special `activity_name` → `activity_id` key mapping
///       so `"create"` maps to `"Create"` via `activity_id` entries in `OcsfEnumMap`.
/// Any future regression removing either fix will cause this test to fail.
///
/// SID-1 compliance: in-process unit test; no external dependencies; no `#[ignore]`.
///
/// Traces to: BC-2.02.013 postconditions "activity_name (guaranteed)";
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

    // BC-2.02.013: activity_name must be normalized to canonical OCSF Title-case.
    // Regression guard: normalization wired, OCSF_ENUM_LABEL_FIELDS uses "activity_name",
    // and the activity_id key mapping is correct.
    let activity_name_val = extract_string_field(&msg, "activity_name");
    assert_eq!(
        activity_name_val, "Create",
        "F-P1-ACTIVITY-NOOP: activity_name='create' must normalize to 'Create' via \
         normalize_with_mappers (BC-2.02.013 activity_name in-scope); \
         got: {activity_name_val:?}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// F-P1-ACTIVITY-DISP-TEST-GAP: BC-2.02.013 — disposition normalization guard
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
/// Regression guard: PASSES — the F-CRIT-001 normalization wiring correctly handles
/// disposition. The field name and key derivation were already correct;
/// the general normalization wiring from RG-019 was sufficient.
/// If `"disposition"` is accidentally removed from `OCSF_ENUM_LABEL_FIELDS` or the
/// `disposition_id` enum entries are dropped from `OcsfEnumMap`, this test will fail.
///
/// SID-1 compliance: in-process unit test; no external dependencies; no `#[ignore]`.
///
/// Traces to: BC-2.02.013 postconditions "disposition (guaranteed)";
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

    // BC-2.02.013: disposition must be normalized to canonical OCSF Title-case.
    // Regression guard: normalization wired by F-CRIT-001; field name and key derivation correct.
    let disposition_val = extract_string_field(&msg, "disposition");
    assert_eq!(
        disposition_val, "Blocked",
        "F-P1-ACTIVITY-DISP-TEST-GAP: disposition='blocked' must normalize to 'Blocked' via \
         normalize_with_mappers (BC-2.02.013 disposition in-scope); \
         got: {disposition_val:?}"
    );
}
