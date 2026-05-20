---
document_type: holdout-scenario
level: L3
id: "HS-014"
category: "dtu-parity"
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
  - ".factory/specs/test-strategy/TS-PLUGIN-PARITY-001-dtu-canonicalization.md"
input-hash: null
traces_to: prd.md
behavioral_contracts:
  - BC-2.16.013
lifecycle_status: active
introduced: "2026-05-20"
last_evaluated: null
last_eval_satisfaction: null
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
notes: "PLUGIN-MIGRATION-001-D holdout — Claroty POST-for-read polymorphic ID parity (DTU). Authored FB-IMPL-P1-PO fix-burst-1 2026-05-20."
---

# HS-014: Claroty POST-for-Read with Polymorphic ID — DTU Parity

**Group:** DTU Parity — Plugin Migration (PLUGIN-MIGRATION-001-D)
**Date:** 2026-05-20
**Priority:** P0
**BC Anchor:** BC-2.16.013 §Postconditions §1 (claroty.sensor.toml), §2, §Edge Cases EC-016-013-004

---

## Scenario

Validates that `claroty.sensor.toml` + `PipelineExecutor` against the Claroty DTU clone produces
OCSF-normalized output equivalent to the reference output from the prior hardcoded Rust adapter
path, per TS-PLUGIN-PARITY-001. The Claroty `assets` table uses a POST-for-read pattern
(POST `/api/v1/assets` with a JSON body to retrieve asset records), which is the
least-common pattern in the spec grammar and the highest risk for spec-authoring error.
URL pattern from `claroty.rs:endpoint_from_spec()` (claroty.rs:238-244): strips `"claroty_"`
prefix and prepends `"/api/v1/"` — NO `/xdome` prefix present in the production code.

The polymorphic ID case (`ClarotyId` — integer or UUID string) is the primary edge case:
the spec column must be typed `string` to handle both ID forms; the OCSF output must normalize
both forms correctly.

---

## Sub-Scenarios

### HS-014-01: Claroty Assets POST-for-Read — Integer ID

**Preconditions:**
- `prism-dtu-claroty` DTU clone is running (started via `BehavioralClone::start_on`)
- `claroty.sensor.toml` loaded and passes BC-2.16.009 validation
- Fixture: asset record with `"id": 12345` (integer)
- Holdout evaluator has NOT seen the reference OCSF output before evaluation

**Steps:**
1. Start `ClarotyClone` via `BehavioralClone::start_on("127.0.0.1:0", shutdown, None)`
2. Execute `PipelineExecutor::execute` for the `assets` table
3. Apply TS-PLUGIN-PARITY-001 canonicalization

**Expected Outcome:**
- Parity verdict: PASS — `id` column value `"12345"` (string-normalized); matches reference
- POST body correctly formed per `claroty.sensor.toml` step definition targeting `/api/v1/assets`
- Offset pagination advances correctly; `request_count >= 1`

### HS-014-02: Claroty Assets POST-for-Read — UUID String ID

**Preconditions:**
- Same as HS-014-01
- Fixture: asset record with `"id": "550e8400-e29b-41d4-a716-446655440000"` (UUID string)

**Steps:**
1. Same setup as HS-014-01 but with UUID-string-ID fixture

**Expected Outcome:**
- Parity verdict: PASS — `id` column value `"550e8400-..."` matches reference
- String normalization is a no-op for UUID string IDs

---

## Behavioral Contract Linkage

| BC | Title | Sub-Scenarios |
|----|-------|---------------|
| BC-2.16.013 | Bundled Sensor Spec Authoring and DTU-Parity Verification — 4 Initial Sensors | HS-014-01, HS-014-02 |

---

## Known-Good / Known-Problematic Corpus Note

- **Known-good corpus:** Claroty DTU with standard fixture payloads (integer IDs, offset pagination
  advancing correctly) — expected parity PASS.
- **Known-problematic corpus:** Claroty DTU with mixed-ID fixture (half integer, half UUID in same
  response) — expected parity PASS if spec correctly normalizes both forms; FAIL if spec hardcodes
  integer parsing only.
