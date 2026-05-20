---
document_type: holdout-scenario
level: L3
id: "HS-018"
category: "negative-validation"
must_pass: true
priority: P0
epic_id: "PLUGIN-MIGRATION-001"
version: "1.0"
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
notes: "PLUGIN-MIGRATION-001-D holdout — negative: spec_id/filename mismatch rejected at load time (E-SPEC-017). Authored FB-IMPL-P1-PO fix-burst-1 2026-05-20. Corrected E-SPEC-009→E-SPEC-017 in FB-IMPL-P2-PO fix-burst-2 2026-05-20 (E-SPEC-009 covers duplicate-sensor_id only; filename-stem mismatch is E-SPEC-017 per error-taxonomy.md v1.41)."
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
- The file is rejected with `E-SPEC-017` per error-taxonomy.md v1.41 (filename-stem-vs-sensor_id
  mismatch; distinct from `E-SPEC-009` which covers duplicate-sensor_id only per BC-2.16.013
  §Error Conditions v1.2)
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
- `E-SPEC-017` returned per error-taxonomy.md v1.41 (case-mismatch is a filename-stem-vs-sensor_id failure, not a duplicate-sensor_id; E-SPEC-009 does not apply here)
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
