---
document_type: holdout-scenario
level: L3
id: "HS-017"
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
  - ".factory/specs/behavioral-contracts/BC-2.16.009-spec-file-validation.md"
input-hash: null
traces_to: prd.md
behavioral_contracts:
  - BC-2.16.013
  - BC-2.16.009
lifecycle_status: active
introduced: "2026-05-20"
last_evaluated: null
last_eval_satisfaction: null
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
notes: "PLUGIN-MIGRATION-001-D holdout — negative: bundled spec fails BC-2.16.009 validation CI gate. Authored FB-IMPL-P1-PO fix-burst-1 2026-05-20."
---

# HS-017: Negative — Bundled Spec Fails BC-2.16.009 Validation at CI Time

**Group:** DTU Parity — Plugin Migration (PLUGIN-MIGRATION-001-D) — Negative Coverage
**Date:** 2026-05-20
**Priority:** P0
**BC Anchor:** BC-2.16.013 §Error Conditions (E-SPEC-001), BC-2.16.009

---

## Scenario

Validates that a deliberately malformed bundled sensor spec file is correctly rejected by the
BC-2.16.009 validation pipeline at CI time. This is the **negative correctness gate**: if a
broken spec can be committed without triggering a CI failure, the CI gate is broken.

This holdout scenario exercises the CI validation path, not the parity path. The holdout
evaluator simulates what would happen if a developer committed a malformed spec file and verifies
the CI job would catch it.

---

## Sub-Scenarios

### HS-017-01: Spec with Invalid Column Type Fails E-SPEC-002

**Preconditions:**
- CI job `validate-bundled-specs` is configured (PLUGIN-MIGRATION-001-D story task)
- A malformed test spec file `crowdstrike-invalid.sensor.toml` is prepared with column
  `type: "not_a_real_type"` (invalid — not one of string/integer/float/boolean/datetime/json)
- This spec is NOT in the production specs directory; it is in a test-only fixture path

**Steps:**
1. Run spec validation: `spec_parser::parse_spec_file("crowdstrike-invalid.sensor.toml")`
2. Collect returned error

**Expected Outcome:**
- Returns `Err(SpecEngineError)` — NOT `Ok(SensorSpec)`
- Error message contains `E-SPEC-002` (invalid column type)
- CI job exits non-zero; bundled spec file cannot be committed without fix

### HS-017-02: Spec with Undefined Variable Reference Fails E-SPEC-003

**Preconditions:**
- A malformed test spec file is prepared with `path_template: "/detects/${undefined_step.id}"`
  where `undefined_step` is not a declared step in the spec

**Steps:**
1. Run spec validation: `spec_parser::parse_spec_file("crowdstrike-undefined-var.sensor.toml")`
2. Collect returned error

**Expected Outcome:**
- Returns `Err(SpecEngineError)` containing `E-SPEC-003` (undefined variable reference)
- CI gate correctly blocks the commit

---

## Behavioral Contract Linkage

| BC | Title | Sub-Scenarios |
|----|-------|---------------|
| BC-2.16.013 | Bundled Sensor Spec Authoring and DTU-Parity Verification — 4 Initial Sensors | HS-017-01, HS-017-02 |
| BC-2.16.009 | Spec File Validation | HS-017-01, HS-017-02 |

---

## Known-Good / Known-Problematic Corpus Note

- **Known-good corpus:** All four production bundled specs (`crowdstrike.sensor.toml`,
  `claroty.sensor.toml`, `cyberint.sensor.toml`, `armis.sensor.toml`) parsed through
  `spec_parser::parse_spec_file()` — expected result: `Ok(SensorSpec)` for all four; zero
  validation errors.
- **Known-problematic corpus:** The malformed test fixtures above — expected result: `Err`
  with specific E-SPEC-NNN codes; zero `Ok` returns.
