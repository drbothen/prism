---
document_type: adversarial-review
level: ops
version: "1.0"
status: complete
producer: adversary
timestamp: 2026-05-11T08:00:00Z
phase: 3
inputs: []
input-hash: "c2521ed"
traces_to: S-PLUGIN-PREREQ-A-sensorid-newtype.md
pass: 9
previous_review: S-PLUGIN-PREREQ-A-pass-8.md
target_artifact: S-PLUGIN-PREREQ-A
review_layer: LOCAL
target_sha: cda9abf5
base_sha: c6dd6602
prior_passes: [1, 2, 3, 4, 5, 6, 7, 8]
verdict: BLOCKED-soft
streak_before: 1/3
streak_after: 0/3
finding_counts:
  CRITICAL: 0
  HIGH: 0
  MED: 2
  LOW: 2
  OBS: 1
trajectory: "14 → 12 → 6 → 4 → 2 → 6 → 4 → 0 → 4"
---

# Adversarial Review: S-PLUGIN-PREREQ-A (Pass 9)

**Verdict: BLOCKED-soft** — Pass-8 FALSE-CLEAN detected. Streak reset 1/3 → 0/3. Trajectory non-monotonic 14→12→6→4→2→6→4→0→4.

## Finding ID Convention

Finding IDs use the format: `F-LP<PASS>-<SEV>-<SEQ>` (local-pass convention for story-scoped LOCAL review cascade).

---

## Part A — Fix Verification (Pass-8 OBS Items)

| ID | Previous Severity | Status | Notes |
|----|-------------------|--------|-------|
| OBS-LP8-001 | OBS | RESOLVED | Pass-7 closure report narrative accurately described test additions and OPTION-B disposition. Non-blocking. |
| OBS-LP8-002 | OBS | RESOLVED | Story BC list and STORY-INDEX row are in alignment at cda9abf5. Non-blocking housekeeping. |
| OBS-LP8-003 | OBS | **FALSE-CLEAN DETECTED** | Pass-8 claimed "6/6 by exact name" for Red Gate tests. Actual count is 0/6 exact-name matches — all implemented tests use BC-prefixed convention. Pass-8 adversary accepted semantic equivalence without running literal grep and mis-reported "exact." This is the root cause of the false-CLEAN verdict. Streak reset 1/3 → 0/3. |

---

## Part B — New Findings (or all findings for pass 1)

### CRITICAL

_None._

### HIGH

_None._

### MEDIUM

#### F-LP9-MED-001: Doc-comment drift — `sensor_type` parameter name in invalidation.rs

- **Severity:** MED
- **Category:** spec-fidelity / code-quality
- **Location:** `crates/prism-query/src/invalidation.rs:125,129`
- **Description:** Doc-comment at lines 125 and 129 references `sensor_type` as a parameter name. The function signature at this site uses `sensor_id: &SensorId` (post-migration parameter name). The doc-comment retained the pre-migration concept reference.
- **Evidence:**
  ```
  grep -n "sensor_type" crates/prism-query/src/invalidation.rs
  125:/// * `sensor_type` - the sensor type to invalidate
  129:///   the `sensor_type` field
  ```
  The function parameter is `sensor_id`, not `sensor_type`. Doc-comment is factually incorrect post-rename.
- **Proposed Fix:** Replace `sensor_type` with `sensor_id` at both lines 125 and 129. 2-line edit.

---

#### F-LP9-MED-002: Story §Red Gate Test Set — 0/6 exact-name match against implementation

- **Severity:** MED
- **Category:** spec-fidelity
- **Location:** `.factory/stories/S-PLUGIN-PREREQ-A-sensorid-newtype.md:324–345`
- **Description:** The story §Red Gate Test Set (v1.3) lists 6 tests by design-phase names. The implementation uses BC-prefixed names per project convention. Exact-name grep returns 0 matches. This divergence caused OBS-LP8-003's false "6/6 exact" claim in pass-8 and is a durable audit-trail gap: any future audit using the story as canonical reference will find 0/6 evidence.
- **Evidence:**
  ```
  grep -rn "test_sensor_id_exists\|test_sensor_id_hash_eq_display_roundtrip\|test_sensor_id_borrow_str_lookup\|test_adapter_registry_sensorid_keyed\|test_sensortype_reintroduction_compile_fails\|test_prism_query_dispatch_uses_sensorid" crates/
  ```
  Returns **0 matches**. Implemented BC-prefixed names verified present:
  - `test_BC_2_01_013_001_sensorid_from_str_roundtrip` (covers design `test_sensor_id_exists`)
  - `test_BC_2_01_013_003_sensorid_hash_eq_invariant` (covers design `test_sensor_id_hash_eq_display_roundtrip`)
  - `test_BC_2_01_013_004_sensor_id_borrow_str_lookup` (covers design `test_sensor_id_borrow_str_lookup`, added fix-burst-7)
  - `test_BC_2_01_013_004_adapter_registry_sensorid_insert_lookup` (covers design `test_adapter_registry_sensorid_keyed`, in bc_2_01_013_sensorid.rs)
  - compile-fail assertion at `tests/external/perimeter-violation/src/main.rs` (covers design `test_sensorttype_reintroduction_compile_fails` — VP-PLUGIN-001 CI gate, not a `#[test]` function)
  - `test_BC_2_01_013_005_sensorid_lookup_at_virtual_fields_dispatch` (covers design `test_prism_query_dispatch_uses_sensorid`, in sensorid_dispatch_redgate.rs)
- **Proposed Fix:** OPTION B (recommended) — amend story §Red Gate Test Set v1.3 → v1.4 with canonical BC-prefixed names, retaining original design-phase names as parentheticals for audit-trail. _Orchestrator decision: OPTION B adopted._

---

### LOW

#### F-LP9-LOW-001: Doc-comment drift — `sensor_type` concept reference in materialization.rs

- **Severity:** LOW
- **Category:** code-quality
- **Location:** `crates/prism-query/src/materialization.rs:775`
- **Description:** Doc-comment at line 775 references `sensor_type` (pre-migration concept) where surrounding code uses `SensorId`. Stale concept reference.
- **Evidence:**
  ```
  grep -n "sensor_type" crates/prism-query/src/materialization.rs
  775:    /// Returns the `sensor_type` for this materialized row, used to route
  ```
  The field and parameter are now `sensor_id`.
- **Proposed Fix:** Replace `sensor_type` with `sensor_id` in the doc-comment at line 775. 1-line edit.

---

#### F-LP9-LOW-002: Doc reference to trait method `sensor_type()` in registry.rs — adjudication clarity

- **Severity:** LOW
- **Category:** code-quality
- **Location:** `crates/prism-sensors/src/registry.rs:56`
- **Description:** Doc-comment references `adapter.sensor_type()` trait method name. The method exists and is named `sensor_type()` at cda9abf5 (returns `SensorId`; rename deferred to PREREQ-E per ADR-023 §C1). The reference is currently accurate but will become a doc-drift defect at PREREQ-E time. Flagged for adjudication clarity — INTENTIONAL per pass-7 preservation.
- **Evidence:**
  ```
  grep -n "sensor_type" crates/prism-sensors/src/registry.rs
  56:    /// Calls `adapter.sensor_type()` to derive the registration key.
  ```
- **Proposed Fix (optional):** Add `// NOTE: method renamed to sensor_id() in PREREQ-E (ADR-023 §C1 deferred)` inline comment to prevent future confusion. Non-blocking for this pass; deferred to PREREQ-E scope unless orchestrator directs closure now.

---

### OBS

#### OBS-LP9-001: Process gap — PG-LP7-002 Red Gate exact-name audit BYPASSED in pass-8

- **Severity:** OBS / Process-Gap
- **Description:** PG-LP7-002 (codified after pass-7) states: "Red Gate test materialization audit: adversary MUST grep for exact test names before asserting coverage." Pass-8 bypassed this rule — accepted semantic equivalence and reported "6/6 exact" without running the literal grep. This produced the false-CLEAN verdict and reset the streak from 1/3 → 0/3.
- **Codification required:** Amend the adversary dispatch prompt: "Each Red Gate audit MUST include the literal grep command AND the match-count output as evidence. Semantic equivalence alone is insufficient. If exact-name count is 0 but semantic coverage is present, this MUST be flagged as a finding (story spec divergence) requiring OPTION B amendment."

---

### KUDOs (5)

**KUDO-1 — Validator parity proptest exemplar:** The cross-crate proptest covering both `prism-core` and `prism-spec-engine` validation rules with 6 input strategies is an exemplar pattern for the project. Highest-quality cross-crate test infrastructure in the current cascade.

**KUDO-2 — Panic contract uniformity across 4 constructors:** All 4 `SensorId` construction paths (`from`, `try_from_str`, `try_from_string`, `Deserialize`) have consistent, documented panic vs. error semantics.

**KUDO-3 — `try_from_str` blanket-impl footgun warning:** Removing the accidental blanket `TryFrom` impl and documenting why it was removed is an excellent forward-compat decision. Prevents "why doesn't my type have TryFrom<SensorId>?" confusion for future plugin authors.

**KUDO-4 — CI `--color=never` with story citation:** The `ci.yml` E0432 detection step using `--color=never` was a precision CI engineering decision. Citing the story in the CI step comment (VP-PLUGIN-001 reference) is the correct documentation practice.

**KUDO-5 — VP-PLUGIN-001 dual-assertion E0432+per-symbol:** The perimeter-violation test asserting both the E0432 error code AND per-symbol absence is defense-in-depth at the compile-fail layer.

---

## Summary

| Severity | Count |
|----------|-------|
| CRITICAL | 0 |
| HIGH | 0 |
| MEDIUM | 2 |
| LOW | 2 |
| OBS | 1 |

**Overall Assessment:** block
**Convergence:** findings remain — iterate. False-CLEAN in pass-8 detected; streak reset 1/3 → 0/3. Fix-burst-9 required (3-line doc edits + story v1.4 amendment).
**Readiness:** requires revision. fix-burst-9 → pass-10 fresh-context for streak 1/3 attempt.

---

## Novelty Assessment

| Field | Value |
|-------|-------|
| **Pass** | 9 |
| **New findings** | 4 (F-LP9-MED-001, F-LP9-MED-002, F-LP9-LOW-001, F-LP9-LOW-002) |
| **Duplicate/variant findings** | 0 |
| **Novelty score** | 4 / (4 + 0) = 1.0 |
| **Median severity** | MED (3.0 on 1.0–5.0 scale) |
| **Trajectory** | 14 → 12 → 6 → 4 → 2 → 6 → 4 → 0 → 4 |
| **Verdict** | FINDINGS_REMAIN — non-monotonic trajectory; false-CLEAN detected in pass-8; streak reset 1/3 → 0/3 |
