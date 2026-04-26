# Red Gate Log — S-2.04 (prism-audit: Audit Entry Construction and Compliance)

**Date:** 2026-04-25
**Story version:** v1.5
**Worktree:** `.worktrees/S-2.04-audit-construction`
**Commit:** `66bce09f`

---

## Summary

72 tests written across 6 behavioral contracts (BC-2.05.001/002/003/004/006/008).
Red Gate result: **54 PASS, 18 FAIL**. All 18 failures are intentional, driven by
the two S-2.04 v1.5 spec corrections noted below.

---

## Red Gate Results

| Metric | Value |
|--------|-------|
| Tests compiled | YES |
| Tests written | 72 |
| Red (fail) | 18 |
| Green by design | 54 |
| Workspace baseline | 1058 PASS / 0 FAIL |
| Workspace after Red Gate | 1112 PASS / 18 FAIL |
| Net new passing tests | 54 |
| Clippy | PASS (clean) |
| rustfmt | PASS (clean) |

---

## Spec-Correction-Driven Failures (v1.4 stub → v1.5 canonical)

### 1. Redaction sentinel: `"***REDACTED***"` → `"[REDACTED]"`

**Affected tests:** 18 tests in `bc_2_05_003.rs` (all BC-2.05.003 redaction tests)

The stub (`20b4a12a`, generated against v1.4) used `REDACTED_SENTINEL = "***REDACTED***"`.
S-2.04 v1.5 corrects this to `"[REDACTED]"` per BC-2.05.003 canonical definition.

All 18 BC-2.05.003 tests assert `"[REDACTED]"` and fail because the stub's
`redaction.rs` still defines `REDACTED_SENTINEL = "***REDACTED***"`.

**Implementer action:** Change `REDACTED_SENTINEL` in
`crates/prism-audit/src/redaction.rs` from `"***REDACTED***"` to `"[REDACTED]"`.

### 2. RiskTier → AuditRiskLevel in WriteAuditDetail

**Resolution:** Fixed in this dispatch (test infrastructure change).

The stub used `prism_core::RiskTier` (Reversible|Irreversible) in
`WriteAuditDetail.risk_tier`. S-2.04 v1.5 mandates `prism_core::AuditRiskLevel`
(Low|Medium|High|Critical).

This dispatch:
- Declared `AuditRiskLevel` in `crates/prism-core/src/audit_risk.rs`
- Re-exported from `prism-core/src/lib.rs`
- Updated `write_audit.rs` to use `AuditRiskLevel` (required for tests to compile)

BC-2.05.004 tests for `AuditRiskLevel` variants now PASS (the type exists and the
field type is correct). These tests confirm the v1.5 correction is in place.

---

## Test Files Written

| File | BC | Tests | Red | Green |
|------|----|-------|-----|-------|
| `crates/prism-audit/src/tests/bc_2_05_001.rs` | BC-2.05.001 | 6 | 0 | 6 |
| `crates/prism-audit/src/tests/bc_2_05_002.rs` | BC-2.05.002 | 11 | 0 | 11 |
| `crates/prism-audit/src/tests/bc_2_05_003.rs` | BC-2.05.003 | 21 | 18 | 3 |
| `crates/prism-audit/src/tests/bc_2_05_004.rs` | BC-2.05.004 | 16 | 0 | 16 |
| `crates/prism-audit/src/tests/bc_2_05_006.rs` | BC-2.05.006 | 4 | 0 | 4 |
| `crates/prism-audit/src/tests/bc_2_05_008.rs` | BC-2.05.008 | 14 | 0 | 14 |
| **Total** | | **72** | **18** | **54** |

Infrastructure also added:
- `crates/prism-audit/src/tests/helpers.rs` — `MemBackend`, `FailingBackend`,
  `AlwaysSucceedService`, `make_request`, `count_audit_entries`

---

## Green-by-Design Tests (54)

These tests pass because the stub's implementation happens to satisfy the assertion:

- **BC-2.05.001 (6 tests):** `AuditEmitter` fail-closed and fail-open behaviour is
  correctly implemented in the stub's `audit_emitter.rs`. All 6 tests pass.
- **BC-2.05.002 (11 tests):** `AuditEntry` struct + serialisation fully implemented.
  All 11 field completeness and sentinel tests pass.
- **BC-2.05.003 (3 of 21 tests):** `is_credential_key` returns true/false correctly;
  key-name-preserved and non-credential field tests pass. The 18 value-assertion
  tests fail on the sentinel mismatch.
- **BC-2.05.004 (16 tests):** `WriteAuditDetail` + `AuditRiskLevel` types are correct
  (thanks to the v1.5 fix applied in this dispatch). All 16 pass.
- **BC-2.05.006 (4 tests):** Key format, concurrent uniqueness, correction pattern,
  and source scanner all pass.
- **BC-2.05.008 (14 tests):** SOC2/ISO27001 field completeness tests pass.

---

## AuditRiskLevel Declaration

**Declared:** YES
**File:** `crates/prism-core/src/audit_risk.rs`
**Re-exported:** `crates/prism-core/src/lib.rs` (`pub use audit_risk::AuditRiskLevel;`)
**Variants:** `Low | Medium | High | Critical`
**Derives:** `Debug, Clone, PartialEq, Eq, Serialize, Deserialize`

---

## Implementer Handoff

To make all 72 tests pass, the implementer must:

1. **Fix `REDACTED_SENTINEL`** in `crates/prism-audit/src/redaction.rs`:
   ```
   pub const REDACTED_SENTINEL: &str = "[REDACTED]";
   ```
   This unblocks all 18 BC-2.05.003 test failures.

2. Everything else is already correct (the `AuditRiskLevel` v1.5 correction was applied
   in this test dispatch). No other changes should be needed to pass the test suite.

Make each test pass, one at a time, with minimum code. The single remaining change
is the sentinel constant.
