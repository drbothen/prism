---
document_type: holdout-scenario
level: L3
id: "HS-018"
category: "negative-validation"
must_pass: true
priority: P0
epic_id: "PLUGIN-MIGRATION-001"
version: "1.4"
status: draft
producer: product-owner
timestamp: 2026-05-20T00:00:00Z
phase: 3
inputs:
  - ".factory/specs/behavioral-contracts/BC-2.16.013-bundled-sensor-spec-dtu-parity.md"
  - ".factory/specs/behavioral-contracts/BC-2.16.001-sensor-spec-file-loading.md"
input-hash: null
traces_to: prd.md
behavioral_contracts:
  - BC-2.16.013
  - BC-2.16.001
lifecycle_status: active
introduced: "2026-05-20"
last_evaluated: null
last_eval_satisfaction: null
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
notes: "PLUGIN-MIGRATION-001-D holdout — negative: spec_id/filename mismatch rejected at load time (E-SPEC-017). Authored FB-IMPL-P1-PO fix-burst-1 2026-05-20. Corrected E-SPEC-009→E-SPEC-017 in FB-IMPL-P2-PO fix-burst-2 2026-05-20 (E-SPEC-009 covers duplicate-sensor_id only; filename-stem mismatch is E-SPEC-017 per error-taxonomy.md v1.44)."
---

# HS-018: Negative — Spec sensor_id / Filename Mismatch Rejected at Load Time

**Group:** DTU Parity — Plugin Migration (PLUGIN-MIGRATION-001-D) — Negative Coverage
**Date:** 2026-05-20
**Priority:** P0
**BC Anchor:** BC-2.16.013 §Error Conditions (E-SPEC-017), BC-2.16.001 §Postconditions

---

## Scenario

Validates that a bundled sensor spec file where the `sensor_id` value does not case-sensitively
match the filename stem is rejected at load time with `E-SPEC-017`. This enforces the
`{sensor_id}.sensor.toml` naming convention (INV-PARITY-002) and prevents silent namespace
collisions in the DataFusion table registry.

Example violation: file `crowdstrike.sensor.toml` with `sensor_id: "falcon"` — the file name
says `crowdstrike` but the spec declares a different sensor identity. This would register tables
as `falcon.detections` instead of `crowdstrike.detections`, silently breaking any query that
targets `crowdstrike.*`.

---

## Sub-Scenarios

### HS-018-01: Filename Stem Mismatch (crowdstrike file, falcon sensor_id)

**Preconditions:**
- A test fixture file `crowdstrike-mismatch.sensor.toml` is prepared with `sensor_id: "falcon"`
- BC-2.16.001 spec loading is active; filename stem vs `sensor_id` validation is implemented

**Steps:**
1. Load spec directory containing `crowdstrike-mismatch.sensor.toml` via
   `parse_spec_directory()` or equivalent
2. Collect returned errors

**Expected Outcome:**
- The file is rejected with `E-SPEC-017` per error-taxonomy.md v1.44 (filename-stem-vs-sensor_id
  mismatch; distinct from `E-SPEC-009` which covers duplicate-sensor_id only per BC-2.16.013
  §Error Conditions)
- Error message names both the filename and the declared `sensor_id`
- No partial registration: `falcon.*` tables are NOT registered in DataFusion
- Other valid spec files in the directory continue loading (DI-030 partial-failure isolation)

### HS-018-02: Case-Mismatch (crowdstrike file, CrowdStrike sensor_id)

**Preconditions:**
- A test fixture file `crowdstrike-case.sensor.toml` is prepared with `sensor_id: "CrowdStrike"`
  (different case than filename stem `crowdstrike`)

**Steps:**
1. Load spec via `parse_spec_directory()`

**Expected Outcome:**
- File is rejected — `sensor_id` must case-sensitively match filename stem
- `E-SPEC-017` returned per error-taxonomy.md v1.44 (case-mismatch is a filename-stem-vs-sensor_id failure, not a duplicate-sensor_id; E-SPEC-009 does not apply here)
- Production convention requires `sensor_id: "crowdstrike"` (all lowercase, matching filename)

### HS-018-03: Valid Convention (crowdstrike file, crowdstrike sensor_id)

**Preconditions:**
- Production bundled spec `crowdstrike.sensor.toml` with `sensor_id: "crowdstrike"` (matching)

**Steps:**
1. Load spec via `parse_spec_directory()`

**Expected Outcome:**
- Spec loads successfully: `Ok(SensorSpec)` with `sensor_id == "crowdstrike"`
- DataFusion table namespace registered as `crowdstrike.detections`, `crowdstrike.devices`,
  `crowdstrike.incidents`
- This is the control case confirming the validation does NOT reject valid specs

---

## Behavioral Contract Linkage

| BC | Title | Sub-Scenarios |
|----|-------|---------------|
| BC-2.16.013 | Bundled Sensor Spec Authoring and DTU-Parity Verification — 4 Initial Sensors | HS-018-01, HS-018-02, HS-018-03 |
| BC-2.16.001 | Sensor Spec File Loading | HS-018-01, HS-018-02, HS-018-03 |

---

## Known-Good / Known-Problematic Corpus Note

- **Known-good corpus:** All four production bundled specs (filename stems match `sensor_id` in
  each file) — expected: all four load without E-SPEC-017 rejection.
- **Known-problematic corpus:** Test fixture with `sensor_id` value differing from filename stem —
  expected: E-SPEC-017 rejection, no partial registration.

---

## Evaluation Criteria

### Coverage Mapping (F-LP13-MED-002 clarification — v1.1)

The three sub-scenarios are covered as follows by the Red Gate test suite:

| Sub-Scenario | Coverage | Test | Rationale |
|---|---|---|---|
| HS-018-01 | RG-09 | `sensor_id: "falcon"` vs filename stem `crowdstrike` — exact string mismatch | Direct coverage; RG-09 asserts E-SPEC-017 on filename-stem-vs-sensor_id inequality |
| HS-018-02 | RG-09 (via case-sensitive string equality assertion) | `sensor_id: "CrowdStrike"` vs filename stem `crowdstrike` — case differs | RG-09's E-SPEC-017 assertion uses exact case-sensitive string equality (`sensor_id != filename_stem`). `"CrowdStrike" != "crowdstrike"` — this IS a mismatch by the same string-equality predicate. No separate test required: the convention that bundled spec filenames are lowercase means `CrowdStrike` (mixed case) will never equal the filename stem `crowdstrike` (lowercase). HS-018-02 is therefore covered by RG-09's logic as a natural consequence of case-sensitive string comparison. |
| HS-018-03 | RG-09 (control case) | `sensor_id: "crowdstrike"` vs filename stem `crowdstrike` — exact match | RG-09 includes a control variant confirming no E-SPEC-017 is raised for matching values |

**Option A rationale (F-LP13-MED-002):** The adversary found no anchored test for HS-018-02. Option A is applied here: HS-018-02 expects exact `sensor_id` string equality with the file stem (case-sensitive). Since `parse_spec_directory()` / `load_all()` perform a case-sensitive byte-equality check between the `sensor_id` TOML field and the filename stem (lowercase by convention), HS-018-02's case-mismatch input (`"CrowdStrike"` vs `"crowdstrike"`) falls into the same E-SPEC-017 code path as HS-018-01. RG-09 already exercises this code path with its primary test fixture; the case-mismatch variant is structurally identical. A separate RG-10 test is NOT required unless the implementer discovers that case-insensitive comparison is applied (which would be a BC violation — this BC and BC-2.16.001 require case-sensitive enforcement). If case-insensitive comparison is discovered in implementation, escalate to orchestrator and route to story-writer for RG-10 creation.

---

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.4 | FB-IMPL-9 | 2026-05-21 | state-manager | FB-IMPL-9 transitive cite-pin sweep: `error-taxonomy.md v1.42` → `v1.44` at 3 active-prose sites (frontmatter notes line 31, HS-018-01 §Expected Outcome line 71, HS-018-02 §Expected Outcome line 89). Additional discovery — pass-10 adversary enumerated only BC-2.16.013:357/358 + story:1017; grep sweep found HS-018 also carries live-narrative current-authority v1.42 pins. Swept per task instruction: "if grep finds OTHER LIVE-narrative v1.42 cites, sweep each one." 5th POL-29 axis recurrence (transitive cite-pin chain). No semantic content change. |
| 1.3 | FB-IMPL-P22-PO | 2026-05-21 | product-owner | F-LP22-MED-001 closure (16th coherence-axis: same-line dual-format cite-pin escape): swept `error-taxonomy.md v1.41` → `v1.42` at 3 active-prose sites (frontmatter notes line 31, HS-018-01 §Expected Outcome line 71, HS-018-02 §Expected Outcome line 89). HS-018 v1.2→v1.3. |
| 1.2 | FB-IMPL-P21-PO | 2026-05-21 | product-owner | F-LP21-MED-001 closure (15th coherence-axis: section-versioned cite-pin format): HS-018-01 §Expected Outcome line 73 — stripped `v1.2` from `BC-2.16.013 §Error Conditions v1.2` → `BC-2.16.013 §Error Conditions` per Option A (unversioned style). Historical context preserved by error-taxonomy.md "Introduced FB-IMPL-P2-PO" clause and this changelog row. |
| 1.1 | FB-IMPL-P13-PO | 2026-05-20 | product-owner | F-LP13-MED-002 closure: Added §Evaluation Criteria section with coverage mapping for HS-018-01/02/03. Applied Option A — clarified HS-018-02 (case-mismatch) is covered by RG-09's existing case-sensitive string-equality E-SPEC-017 enforcement; no separate RG-10 required unless implementer uses case-insensitive comparison. Added §Changelog per POL-26 changelog discipline. |
| 1.0 | FB-IMPL-P1-PO fix-burst-1 | 2026-05-20 | product-owner | Initial draft — HS-018 spec_id/filename mismatch holdout for PLUGIN-MIGRATION-001-D; 3 sub-scenarios; E-SPEC-009 corrected to E-SPEC-017 in FB-IMPL-P2-PO fix-burst-2. |
