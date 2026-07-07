//! Red Gate test stubs for S-PRISMQL-CASE-INSENSITIVE-001.
//!
//! Covers RG-019, RG-020, RG-021 — adapter-boundary OCSF enum-label canonical-case
//! normalization via `OcsfEnumMap::normalize_label` + normalization pipeline.
//! All bodies are `todo!()` per BC-5.38.001 Red Gate discipline.
//!
//! Behavioral contracts traced:
//!   BC-2.02.013 v1.0 — Adapter-Boundary OCSF Enum-Label Canonical-Case Normalization
//!   BC-2.02.002 v1.5 — DynamicMessage Creation (normalization applied before construction)
//!   BC-2.02.010 v1.5 — OcsfEnumMap as sole canonical casing authority
//!
//! Self-Check Rule (BC-5.38.005 invariant 1):
//! "If I include this non-todo!() function body, will the test for this function
//! pass trivially without any implementer work?"
//! Applied to every function below — all answer YES, so all bodies are `todo!()`.

// ─────────────────────────────────────────────────────────────────────────────
// RG-019: AC-016 — OCSF enum-label fields normalized to canonical Title-case
// ─────────────────────────────────────────────────────────────────────────────

/// RG-019: A sensor record with `severity='CRITICAL'` enters the normalization pipeline;
/// after adapter-boundary normalization, `severity='Critical'` (OCSF canonical Title-case).
///
/// Also covers: `severity='low'` → `'Low'`, `severity='MEDIUM'` → `'Medium'`, etc.
///
/// Traces to: BC-2.02.013 v1.0 postconditions "Severity (guaranteed): 'HIGH' → 'High'";
/// BC-2.02.002 v1.5 amendment (normalization before DynamicMessage creation);
/// BC-2.02.010 v1.5 (enum_map.rs sole casing authority).
#[test]
fn test_S_PRISMQL_CASE_INSENSITIVE_001_adapter_normalization_critical_to_title_case() {
    todo!(
        "S-PRISMQL-CASE-INSENSITIVE-001 RG-019: invoke adapter-boundary normalization pipeline \
         with severity='CRITICAL', assert resulting DynamicMessage has severity='Critical'; \
         also assert severity='low' → 'Low', severity='MEDIUM' → 'Medium' \
         (BC-2.02.013 v1.0 postconditions, BC-2.02.002 v1.5, BC-2.02.010 v1.5)"
    )
}

// ─────────────────────────────────────────────────────────────────────────────
// RG-020: AC-017 — Normalization is idempotent: already-canonical values unchanged
// ─────────────────────────────────────────────────────────────────────────────

/// RG-020: CrowdStrike sensor data with `severity='High'` (already OCSF Title-case)
/// passes through the normalization pipeline unchanged — value stays `'High'`, no warning emitted.
///
/// Traces to: BC-2.02.013 v1.0 postcondition "idempotent: if field already contains canonical-case
/// value, value unchanged"; EC-02-020 (CrowdStrike emits 'High' already canonical).
#[test]
fn test_S_PRISMQL_CASE_INSENSITIVE_001_adapter_normalization_idempotent_high() {
    todo!(
        "S-PRISMQL-CASE-INSENSITIVE-001 RG-020: invoke adapter-boundary normalization pipeline \
         with severity='High' (already canonical), assert resulting DynamicMessage has \
         severity='High' unchanged AND no tracing::warn! emitted (BC-2.02.013 v1.0 idempotent \
         postcondition, EC-02-020)"
    )
}

// ─────────────────────────────────────────────────────────────────────────────
// RG-021: AC-018 — Unrecognized vendor values left as-received with warning logged
// ─────────────────────────────────────────────────────────────────────────────

/// RG-021: Armis vendor-specific value `severity='UNHANDLED'` (not in OCSF severity captions
/// in `enum_map.rs`) passes through unchanged, and a `tracing::warn!` is emitted.
///
/// The normalization is non-fatal — it does not fail or return an error.
///
/// Traces to: BC-2.02.013 v1.0 error cases "Warning (non-fatal): value has no matching caption";
/// EC-02-021 (Armis 'UNHANDLED' vendor-specific).
#[test]
fn test_S_PRISMQL_CASE_INSENSITIVE_001_adapter_normalization_unrecognized_value_left_as_received() {
    todo!(
        "S-PRISMQL-CASE-INSENSITIVE-001 RG-021: invoke adapter-boundary normalization pipeline \
         with severity='UNHANDLED', assert: (1) DynamicMessage has severity='UNHANDLED' unchanged, \
         (2) tracing::warn! emitted with field name + value + sensor type, \
         (3) normalization does NOT return Err (non-fatal) (BC-2.02.013 v1.0 error case, EC-02-021)"
    )
}
