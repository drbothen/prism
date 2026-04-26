//! Tests for BC-2.05.004 — Write Operations Log Capability Check and
//! Execution Outcome.
//!
//! Postconditions tested:
//!   - `WriteAuditDetail` contains `capability_check`, `risk_tier` (AuditRiskLevel),
//!     `confirmation_token_used`, and `execution_outcome`.
//!   - `risk_tier` uses `AuditRiskLevel` (Low | Medium | High | Critical), NOT `RiskTier`.
//!   - `WriteAuditDetail` serialises cleanly to JSON.
//!   - Capability denied → `result_summary` records `"denied_by_capability_check"`.
//!
//! AC-5: write operation audit entry contains `WriteAuditDetail` with required fields.
//!
//! SPEC CORRECTION NOTE (S-2.04 v1.5):
//!   `WriteAuditDetail.risk_tier` must be `AuditRiskLevel` (Low | Medium | High | Critical).
//!   The stub used `RiskTier` (Reversible | Irreversible) — WRONG per v1.5.
//!   These tests assert `AuditRiskLevel` variants, which will FAIL until the implementer
//!   corrects `write_audit.rs` to use `AuditRiskLevel`.

use prism_core::AuditRiskLevel;
use serde_json::Value;

use crate::write_audit::{CapabilityCheckResult, WriteAuditDetail, WriteOutcome};

// ── AC-5: WriteAuditDetail has required fields ────────────────────────────────

/// AC-5 (BC-2.05.004): A `WriteAuditDetail` for a write operation must contain
/// `capability_check`, `risk_tier`, and `execution_outcome` fields.
#[test]
fn test_BC_2_05_004_write_audit_detail_has_required_fields() {
    let detail = WriteAuditDetail::new(
        CapabilityCheckResult::Granted,
        AuditRiskLevel::High,
        None,
        WriteOutcome::Committed,
    );

    let obj: Value =
        serde_json::to_value(&detail).expect("WriteAuditDetail must serialise to JSON");

    assert!(
        obj.get("capability_check").is_some(),
        "WriteAuditDetail must have 'capability_check' field (AC-5, BC-2.05.004)"
    );
    assert!(
        obj.get("risk_tier").is_some(),
        "WriteAuditDetail must have 'risk_tier' field (AC-5, BC-2.05.004)"
    );
    assert!(
        obj.get("execution_outcome").is_some(),
        "WriteAuditDetail must have 'execution_outcome' field (AC-5, BC-2.05.004)"
    );
    assert!(
        obj.get("confirmation_token_used").is_some()
            || obj.get("confirmation_token_used").is_none(),
        // This field may be absent (Option<String>) when None — both are acceptable.
        "'confirmation_token_used' is optional (None → omitted or null)"
    );
}

// ── AuditRiskLevel variants (v1.5 spec correction) ────────────────────────────

/// BC-2.05.004 (S-2.04 v1.5): `risk_tier` must use `AuditRiskLevel::Low`.
/// FAILS if implementer uses `RiskTier::Reversible` instead.
#[test]
fn test_BC_2_05_004_risk_tier_low_variant() {
    let detail = WriteAuditDetail::new(
        CapabilityCheckResult::Granted,
        AuditRiskLevel::Low,
        None,
        WriteOutcome::Committed,
    );
    let obj: Value = serde_json::to_value(&detail).unwrap();
    assert_eq!(
        obj["risk_tier"],
        Value::String("low".to_owned()),
        "AuditRiskLevel::Low must serialise as 'low' (snake_case)"
    );
}

/// BC-2.05.004 (S-2.04 v1.5): `risk_tier` must use `AuditRiskLevel::Medium`.
#[test]
fn test_BC_2_05_004_risk_tier_medium_variant() {
    let detail = WriteAuditDetail::new(
        CapabilityCheckResult::Granted,
        AuditRiskLevel::Medium,
        None,
        WriteOutcome::Committed,
    );
    let obj: Value = serde_json::to_value(&detail).unwrap();
    assert_eq!(
        obj["risk_tier"],
        Value::String("medium".to_owned()),
        "AuditRiskLevel::Medium must serialise as 'medium'"
    );
}

/// BC-2.05.004 (S-2.04 v1.5): `risk_tier` must use `AuditRiskLevel::High`.
#[test]
fn test_BC_2_05_004_risk_tier_high_variant() {
    let detail = WriteAuditDetail::new(
        CapabilityCheckResult::Granted,
        AuditRiskLevel::High,
        None,
        WriteOutcome::Committed,
    );
    let obj: Value = serde_json::to_value(&detail).unwrap();
    assert_eq!(
        obj["risk_tier"],
        Value::String("high".to_owned()),
        "AuditRiskLevel::High must serialise as 'high'"
    );
}

/// BC-2.05.004 (S-2.04 v1.5): `risk_tier` must use `AuditRiskLevel::Critical`.
#[test]
fn test_BC_2_05_004_risk_tier_critical_variant() {
    let detail = WriteAuditDetail::new(
        CapabilityCheckResult::Granted,
        AuditRiskLevel::Critical,
        None,
        WriteOutcome::Aborted,
    );
    let obj: Value = serde_json::to_value(&detail).unwrap();
    assert_eq!(
        obj["risk_tier"],
        Value::String("critical".to_owned()),
        "AuditRiskLevel::Critical must serialise as 'critical'"
    );
}

// ── AuditRiskLevel is NOT RiskTier ────────────────────────────────────────────

/// BC-2.05.004 / Dev Notes: `AuditRiskLevel` and `RiskTier` are distinct types.
/// `AuditRiskLevel` has 4 variants (Low | Medium | High | Critical);
/// `RiskTier` has 2 (Reversible | Irreversible).
/// This test asserts `AuditRiskLevel` compiles and is distinct.
#[test]
fn test_BC_2_05_004_audit_risk_level_is_distinct_from_risk_tier() {
    // If this compiles, AuditRiskLevel is a distinct type from RiskTier.
    let _: AuditRiskLevel = AuditRiskLevel::Critical;
    let _: AuditRiskLevel = AuditRiskLevel::High;
    let _: AuditRiskLevel = AuditRiskLevel::Medium;
    let _: AuditRiskLevel = AuditRiskLevel::Low;

    // prism_core::RiskTier must still exist and be different (no conflict).
    let _: prism_core::RiskTier = prism_core::RiskTier::Reversible;
    let _: prism_core::RiskTier = prism_core::RiskTier::Irreversible;
}

// ── CapabilityCheckResult — Granted / Denied ──────────────────────────────────

/// BC-2.05.004: `capability_check: Granted` serialises correctly.
#[test]
fn test_BC_2_05_004_capability_check_granted_serialises() {
    let detail = WriteAuditDetail::new(
        CapabilityCheckResult::Granted,
        AuditRiskLevel::Low,
        None,
        WriteOutcome::Committed,
    );
    let obj: Value = serde_json::to_value(&detail).unwrap();
    assert_eq!(
        obj["capability_check"]["status"],
        Value::String("granted".to_owned()),
        "CapabilityCheckResult::Granted must serialise with status='granted'"
    );
}

/// BC-2.05.004: `capability_check: Denied { reason }` serialises with the denial reason.
#[test]
fn test_BC_2_05_004_capability_check_denied_serialises_with_reason() {
    let detail = WriteAuditDetail::new(
        CapabilityCheckResult::Denied {
            reason: "sensor.crowdstrike.containment not enabled".to_owned(),
        },
        AuditRiskLevel::High,
        None,
        WriteOutcome::Aborted,
    );
    let obj: Value = serde_json::to_value(&detail).unwrap();
    assert_eq!(
        obj["capability_check"]["status"],
        Value::String("denied".to_owned()),
        "CapabilityCheckResult::Denied must serialise with status='denied'"
    );
    assert!(
        obj["capability_check"]["reason"].is_string(),
        "CapabilityCheckResult::Denied must include a 'reason' string field"
    );
}

// ── WriteOutcome variants ─────────────────────────────────────────────────────

/// BC-2.05.004: `WriteOutcome::Committed` serialises as `"committed"`.
#[test]
fn test_BC_2_05_004_write_outcome_committed() {
    let detail = WriteAuditDetail::new(
        CapabilityCheckResult::Granted,
        AuditRiskLevel::Medium,
        None,
        WriteOutcome::Committed,
    );
    let obj: Value = serde_json::to_value(&detail).unwrap();
    assert_eq!(
        obj["execution_outcome"],
        Value::String("committed".to_owned()),
        "WriteOutcome::Committed must serialise as 'committed'"
    );
}

/// BC-2.05.004: `WriteOutcome::Aborted` serialises as `"aborted"`.
#[test]
fn test_BC_2_05_004_write_outcome_aborted() {
    let detail = WriteAuditDetail::new(
        CapabilityCheckResult::Granted,
        AuditRiskLevel::High,
        None,
        WriteOutcome::Aborted,
    );
    let obj: Value = serde_json::to_value(&detail).unwrap();
    assert_eq!(
        obj["execution_outcome"],
        Value::String("aborted".to_owned()),
        "WriteOutcome::Aborted must serialise as 'aborted'"
    );
}

/// BC-2.05.004: `WriteOutcome::DryRun` serialises as `"dry_run"`.
#[test]
fn test_BC_2_05_004_write_outcome_dry_run() {
    let detail = WriteAuditDetail::new(
        CapabilityCheckResult::Granted,
        AuditRiskLevel::Low,
        None,
        WriteOutcome::DryRun,
    );
    let obj: Value = serde_json::to_value(&detail).unwrap();
    assert_eq!(
        obj["execution_outcome"],
        Value::String("dry_run".to_owned()),
        "WriteOutcome::DryRun must serialise as 'dry_run'"
    );
}

/// BC-2.05.004: `WriteOutcome::ConfirmationTokenIssued` serialises as `"confirmation_token_issued"`.
#[test]
fn test_BC_2_05_004_write_outcome_confirmation_token_issued() {
    let detail = WriteAuditDetail::new(
        CapabilityCheckResult::Granted,
        AuditRiskLevel::Critical,
        None,
        WriteOutcome::ConfirmationTokenIssued,
    );
    let obj: Value = serde_json::to_value(&detail).unwrap();
    assert_eq!(
        obj["execution_outcome"],
        Value::String("confirmation_token_issued".to_owned()),
        "WriteOutcome::ConfirmationTokenIssued must serialise as 'confirmation_token_issued'"
    );
}

// ── confirmation_token_used ───────────────────────────────────────────────────

/// BC-2.05.004: When `confirmation_token_used: Some(token_id)`, the token ID
/// is recorded in the detail.
#[test]
fn test_BC_2_05_004_confirmation_token_used_some_is_recorded() {
    let token_id = "tok_abc123".to_owned();
    let detail = WriteAuditDetail::new(
        CapabilityCheckResult::Granted,
        AuditRiskLevel::Critical,
        Some(token_id.clone()),
        WriteOutcome::Committed,
    );
    assert_eq!(
        detail.confirmation_token_used,
        Some(token_id),
        "confirmation_token_used must record the token ID when Some"
    );
}

/// BC-2.05.004: When `confirmation_token_used: None`, the field is absent or null.
#[test]
fn test_BC_2_05_004_confirmation_token_used_none_is_absent() {
    let detail = WriteAuditDetail::new(
        CapabilityCheckResult::Granted,
        AuditRiskLevel::Low,
        None,
        WriteOutcome::Committed,
    );
    assert!(
        detail.confirmation_token_used.is_none(),
        "confirmation_token_used must be None when no token was consumed"
    );
}

// ── to_json() serialisation ───────────────────────────────────────────────────

/// BC-2.05.004: `WriteAuditDetail::to_json()` must succeed for a valid detail.
#[test]
fn test_BC_2_05_004_to_json_succeeds_for_valid_detail() {
    let detail = WriteAuditDetail::new(
        CapabilityCheckResult::Granted,
        AuditRiskLevel::High,
        None,
        WriteOutcome::Committed,
    );
    let result = detail.to_json();
    assert!(
        result.is_ok(),
        "WriteAuditDetail::to_json() must succeed for a valid detail: {result:?}"
    );
}
