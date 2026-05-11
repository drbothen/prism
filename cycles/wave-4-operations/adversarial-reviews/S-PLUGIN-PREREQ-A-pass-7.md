---
document_type: adversarial-review
level: ops
version: "1.0"
status: complete
producer: adversary
timestamp: 2026-05-11T06:00:00Z
phase: 3
inputs: []
input-hash: "[live-state]"
traces_to: ""
pass: 7
previous_review: S-PLUGIN-PREREQ-A-pass-6.md
target_artifact: S-PLUGIN-PREREQ-A
review_layer: LOCAL
target_sha: bc57c80d
base_sha: c6dd6602
prior_passes: [1, 2, 3, 4, 5, 6]
verdict: BLOCKED-soft
streak: 0/3
finding_counts:
  CRITICAL: 0
  HIGH: 0
  MED: 2
  LOW: 2
  OBS: 0
trajectory: "14 → 12 → 6 → 4 → 2 → 6 → 4"
---

# Adversarial Review: S-PLUGIN-PREREQ-A (Pass 7)

## Finding ID Convention

Finding IDs use the format: `F-LP7-<SEV>-<SEQ>` (LOCAL pass, pass 7).

- `F`: Fixed prefix for LOCAL layer findings
- `LP7`: LOCAL Pass 7
- `<SEV>`: Severity abbreviation (`CRIT`, `HIGH`, `MED`, `LOW`, `OBS`)
- `<SEQ>`: Three-digit sequence within the pass

---

## Part A — Fix Verification (Pass 6 Closures — 7/7 PASS)

All seven pass-6 findings verified CLEAN at bc57c80d. Zero paper-closes detected.

| ID | Previous Severity | Status | Notes |
|----|-------------------|--------|-------|
| F-LP6-HIGH-001 | HIGH | RESOLVED | SensorId::new renamed to `new_unchecked`; `try_from_str` is the only validated public constructor. Doc-comment marks new_unchecked as test-only footgun. Textbook closure. |
| F-LP6-MED-001 | MED | RESOLVED | All mock SensorAdapter impls in prism-sensors/src/tests/ and prism-query/tests/ now use `sensor_id` field name uniformly. |
| F-LP6-MED-002 | MED | RESOLVED | BC-2.01.013 v1.5 amended (state-manager D-386 burst). Adapter Identity Method postcondition documents the sensor_type() rename rationale. |
| F-LP6-MED-003 | MED | RESOLVED | Proptest strategy extended to non-ASCII rejection class; hardcoded vector `validate_sensor_id_string("crowdstr\u{00EF}ke")` returns Err. Two-layer coverage (randomized + deterministic). |
| F-LP6-MED-004 | MED | RESOLVED | Orchestrator OPTION B decision adopted: `&SensorId` is canonical API. Story task 7 + AC-4 amended to v1.3 in this state-burst. F-LP7-MED-002 cross-files the partial-close pattern codification. |
| F-LP6-LOW-001 | LOW | RESOLVED | `validate_sensor_id_string` changed to `pub(crate)` at sensor_id.rs. |
| F-LP6-LOW-002 | LOW | RESOLVED | Story v1.2 input-hash recomputed to 6954524 in D-386 state-burst. |

**Pass-6 closure quality:** TEXTBOOK. HIGH-001 (`new_unchecked` rename) is a model closure — renames the footgun instead of papering over with doc-comments. Non-ASCII proptest extension (MED-003) adds hardcoded boundary vector plus strategy extension. BC-2.01.013 v1.5 amendment (MED-002) clean and concise. Zero sibling-site misses detected across all seven closures.

---

## Part B — New Findings

### CRITICAL

_None._

### HIGH

_None._

### MEDIUM

#### F-LP7-MED-001: Missing Red Gate Test `test_sensor_id_borrow_str_lookup` (AC-9(b) + Task 11)

- **Severity:** MED
- **Category:** coverage-gap
- **Location:** `crates/prism-core/src/sensor_id.rs` (tests module); story §Red Gate Test Set item 3; AC-9(b); Task 11
- **Description:** `test_sensor_id_borrow_str_lookup` does not exist in the codebase. Task 11 and AC-9(b) explicitly mandate this test. The Red Gate test set (story §Red Gate Test Set item 3) lists it as a required pre-implementation failing test. The `Borrow<str>` impl is present in production code but its behavior for `HashMap` lookup is not regression-tested.
- **Evidence:** Grep for `test_sensor_id_borrow_str_lookup` in `crates/prism-core/` returns zero results at bc57c80d. Task 11 body: "insert `SensorId::from("armis")` in `HashMap<SensorId, u32>`, then look up via `map.get("armis" as &str)`." AC-9(b) body: "`Borrow<str>` lookup behavior: `SensorId::from("armis")` inserted into `HashMap<SensorId, u32>` is retrievable via `map.get("armis" as &str)`." Both mandate this test by name.
- **Survival explanation:** Passes 1-3 focused on CRIT/HIGH findings. Passes 4-6 focused on sibling-rename, validator-parity, and pub-API enumeration axes. No prior pass dispatched with a Red Gate test materialization audit checklist axis. PG-LP7-002 below codifies this gap.
- **Note on F-LP7-MED-002 interaction:** OPTION B adoption relaxes the Borrow<str> mandate from the *registry API* in task 7/AC-4. It does not remove the unit-test obligation for the `Borrow<str>` trait impl itself — AC-9(b) and Task 11 still mandate `test_sensor_id_borrow_str_lookup`.
- **Proposed Fix:** Add `test_sensor_id_borrow_str_lookup` to `crates/prism-core/src/sensor_id.rs` tests module. ~8 lines. Verify via `just iter prism-core test_sensor_id_borrow_str_lookup`.

#### F-LP7-MED-002: Partial-Close Pattern — F-LP6-MED-004 Spec-Mismatch Root Cause

- **Severity:** MED
- **Category:** spec-fidelity
- **Location:** Story task 7; AC-4; F-LP6-MED-004 closure record
- **Description:** F-LP6-MED-004's root cause was a spec-vs-implementation mismatch (task 7 mandated `Borrow<str>`; implementation used `&SensorId`). Code-only blast-radius fix (fanout.rs) was a partial close. Full closure required orchestrator adjudication (OPTION B) + story v1.3 spec amendment. This finding records the partial-close pattern for audit trail integrity and codifies PG-LP7-003.
- **Evidence:** F-LP6-MED-004 cited blast radius `fanout.rs:337,490`. Pass-6 closure record shows code fix at fanout.rs. Story task 7 still contained the `Borrow<str>` mandate at the time of pass-6 closure verification — the spec mismatch was unresolved until this state-burst applied the v1.3 amendment.
- **Status:** SELF-CLOSING — story v1.3 amendment already applied in this state-burst (D-387). No further implementer action required. Recorded for audit trail integrity and PG-LP7-003 codification.
- **Proposed Fix:** N/A — already resolved by state-burst story v1.3 amendment.

### LOW

#### F-LP7-LOW-001: 5th Sibling-Rename Recurrence — Loop Variable Names in Test Code (SCOPE-OUT)

- **Severity:** LOW
- **Category:** code-quality
- **Location:** Test bodies in various crates
- **Description:** Loop variable bindings of the form `for sensor_type in [SensorId::from("crowdstrike"), ...]` appear in test bodies. The variable name `sensor_type` holds a value of type `SensorId`. Cosmetic residue of the SensorType → SensorId rename.
- **Evidence:** Pattern `for sensor_type in` in `#[cfg(test)]` blocks at bc57c80d.
- **ORCHESTRATOR ADJUDICATION:** Local variable names in test code are OUT-OF-SCOPE for the rename axis. Rename axis covers struct fields, function names, trait method names (public and private) — but NOT local variable / loop binding identifiers in test bodies. Blast radius is zero; scope is ~10-line function. Strict enforcement would force cosmetic churn on every type rename in any newtype-keystone story. Adjudication codified in PG-LP7-001.
- **Proposed Fix:** None required. SCOPE-OUT.

#### F-LP7-LOW-002: Cross-Newtype Audit Pattern — OrgSlug::new_unchecked Pub Validation Bypass

- **Severity:** LOW
- **Category:** security-surface
- **Location:** `crates/prism-core/src/tenant.rs:77-86`
- **Description:** `OrgSlug::new_unchecked` is `pub` with a doc-comment prohibition ("MUST NOT be called from production code"). Same defect class as pre-fix-burst-6 `SensorId::new`. Paper-fence: compiler does not enforce the doc-comment. Other newtypes in prism-core (AnalystId, CredentialName, etc.) should be audited for similar bypasses.
- **Evidence:** `tenant.rs:77-86` — `pub fn new_unchecked(s: impl Into<Arc<str>>) -> Self { OrgSlug(s.into()) }` with doc-comment prohibition but pub visibility.
- **Scope note:** Not in S-PLUGIN-PREREQ-A scope (story touches sensor_id.rs + sensor adapter layer, not tenant.rs). Escalated to TD-S-PLUGIN-PREREQ-A-006 P3.
- **Proposed Fix:** File TD-S-PLUGIN-PREREQ-A-006 P3. Defer to post-PREREQ-A maintenance pass or dedicated cross-newtype hardening story. No fix-burst-7 action required.

---

## Summary

| Severity | Count |
|----------|-------|
| CRITICAL | 0 |
| HIGH | 0 |
| MEDIUM | 2 |
| LOW | 2 |

**Overall Assessment:** block (BLOCKED-soft — no CRIT/HIGH; 2 MED present)
**Convergence:** findings remain — iterate (fix-burst-7 required for F-LP7-MED-001; F-LP7-MED-002 self-closing; F-LP7-LOW-001 scope-out; F-LP7-LOW-002 deferred to TD-006)
**Readiness:** not ready for next phase — requires fix-burst-7 (1 test addition) then pass-8

---

## Process-Gap Codifications

### PG-LP7-001 — Rename-Axis Layer Enumeration (Sibling-Rename Scope Boundary)

The rename axis for newtype-keystone stories covers:

| Layer | In Scope |
|-------|----------|
| Struct fields (public) | YES |
| Struct fields (private) | YES |
| Function names (public) | YES |
| Function names (private) | YES |
| Trait method names (public) | YES |
| Trait method names (private) | YES |
| Local variable / loop binding identifiers in test bodies | NO — out of scope |

Adversary checklist: do not flag local variable names in `#[cfg(test)]` blocks or `tests/` files during rename-axis sweeps.

### PG-LP7-002 — Red Gate Test Materialization Audit Axis

After fix-burst-N completes, adversary pass-(N+1) checklist MUST include a dedicated Red Gate test materialization audit axis:
1. Read story §Red Gate Test Set.
2. For each named test, grep codebase for the exact test function name.
3. If absent: file MED finding with exact test name, story reference, and AC reference.

This axis was absent from passes 1-6. F-LP7-MED-001 survived 6 passes as a result.

### PG-LP7-003 — Partial-Close Pattern Detection (Spec-Mismatch Root Cause)

When a finding's root cause is a spec-vs-implementation mismatch:
- Code-only fixes are PARTIAL closes. Finding not CLOSED until spec is also amended.
- Closure verification must check BOTH: (a) code at cited blast radius, AND (b) spec artifact that was out of sync.
- Orchestrator adjudication decisions must be recorded in STATE.md (D-row) and story changelog before finding is marked CLOSED.

---

## KUDOs

**KUDO-1 — F-LP6-HIGH-001 Textbook Closure.** `new_unchecked` rename sets a strong precedent for all future newtype hardening — renames the footgun instead of adding paper-fence doc-comments.

**KUDO-2 — Non-ASCII Proptest Diversity.** Two-layer approach (randomized strategy + hardcoded boundary vector) for MED-003 closure is the correct pattern for proptest extensions.

**KUDO-3 — BC-2.01.013 v1.5 Amendment Quality.** Concise, precisely scoped, does not overspecify implementation details.

**KUDO-4 — Comprehensive Deserialize Rejection Tests.** Empty string, digit-leading, hyphen-leading, non-ASCII, overly-long — full rejection-class coverage across all fix-bursts.

**KUDO-5 — `#[non_exhaustive]` Survival.** `SensorIdValidationError` retains `#[non_exhaustive]` through 7 passes. Correct forward-compatibility marker for an error type that may gain new variants.

---

## Novelty Assessment

| Field | Value |
|-------|-------|
| **Pass** | 7 |
| **New findings** | 2 (F-LP7-MED-001 missing Red Gate test; F-LP7-LOW-002 cross-newtype audit) |
| **Duplicate/variant findings** | 2 (F-LP7-MED-002 self-closing partial-close record; F-LP7-LOW-001 5th rename-axis recurrence — scope-out) |
| **Novelty score** | 2 / (2 + 2) = 0.50 |
| **Median severity** | MED (2 MED + 2 LOW; no CRIT/HIGH) |
| **Trajectory** | 14 → 12 → 6 → 4 → 2 → 6 → 4 |
| **Verdict** | FINDINGS_REMAIN — fix-burst-7 required (1 test add); pass-8 expected CLEAN |
