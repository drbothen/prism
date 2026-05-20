---
document_type: holdout-scenario
level: L3
id: "HS-013"
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
notes: "PLUGIN-MIGRATION-001-D holdout — CrowdStrike two-step parity (DTU). Authored FB-IMPL-P1-PO fix-burst-1 2026-05-20."
---

# HS-013: CrowdStrike Two-Step DTU Parity

**Group:** DTU Parity — Plugin Migration (PLUGIN-MIGRATION-001-D)
**Date:** 2026-05-20
**Priority:** P0
**BC Anchor:** BC-2.16.013 §Postconditions §2, §Canonical Test Vectors

---

## Scenario

Validates that the spec-driven dispatch path (`crowdstrike.sensor.toml` + `PipelineExecutor`)
against the CrowdStrike DTU clone produces OCSF-normalized output semantically equivalent to
the reference output from the prior hardcoded Rust adapter path for the same raw API response,
per TS-PLUGIN-PARITY-001 canonicalization rules.

The CrowdStrike two-step pipeline is the highest-risk parity case: the spec must orchestrate a
QueryV2 step (GET `/detects/queries/detects/v1`) to retrieve detection IDs, then a PostEntities
step (POST `/detects/entities/summaries/GET/v1`) batching those IDs (≤ 100 per batch, per
CROWDSTRIKE_BATCH_SIZE). A single-step spec or incorrect variable forwarding breaks parity.

---

## Sub-Scenarios

### HS-013-01: CrowdStrike Detections Happy Path (3 detections)

**Preconditions:**
- `prism-dtu-crowdstrike` DTU clone is running (started via `BehavioralClone::start_on`)
- `crowdstrike.sensor.toml` is loaded and passes BC-2.16.009 validation
- Fixture: QueryV2 returns 3 detection IDs; PostEntities returns 3 full detection records
- Holdout evaluator has NOT seen the reference OCSF output before evaluation

**Steps:**
1. Start `CrowdstrikeClone` via `BehavioralClone::start_on("127.0.0.1:0", shutdown, None)`;
   override `crowdstrike.sensor.toml` `base_url` to the returned `SocketAddr`
2. Load fixture: DTU returns 3 detection IDs from QueryV2, 3 full records from PostEntities
3. Execute `PipelineExecutor::execute(spec, &detections_table, &context, &http_client, &null_auth)`
4. Apply TS-PLUGIN-PARITY-001 Rules A–I canonicalization
5. Compare spec-driven output against reference output from `CrowdStrikeAdapter::fetch()`
   (`SensorAdapter::fetch` trait method, `crates/prism-sensors/src/auth/crowdstrike.rs`)
   applied to the same fixture payload

**Expected Outcome:**
- Parity verdict: PASS for all 3 detection records
- `request_count == 2` (one QueryV2 + one PostEntities)
- OCSF mandatory fields present: `class_uid`, `severity_id`, `finding_info.uid`, `time`, `metadata`
- Zero FAILs; WARN allowed per TS-PLUGIN-PARITY-001 Rule B (null vs absent)

### HS-013-02: CrowdStrike Batch Cap at CROWDSTRIKE_BATCH_SIZE (100 IDs)

**Preconditions:**
- Same as HS-013-01
- Fixture: QueryV2 returns exactly 100 detection IDs in one page

**Steps:**
1. Same setup as HS-013-01 but with 100-ID fixture
2. Execute `PipelineExecutor::execute` for the `detections` table
3. Verify batch boundary: PostEntities receives all 100 IDs in exactly one batch (not 101+)

**Expected Outcome:**
- Parity verdict: PASS — spec produces one PostEntities batch of exactly 100 records
- `batch_size` cap of 100 respected; no batch exceeds CROWDSTRIKE_BATCH_SIZE
- `request_count == 2` (one QueryV2 + one PostEntities with 100 IDs)

---

## Behavioral Contract Linkage

| BC | Title | Sub-Scenarios |
|----|-------|---------------|
| BC-2.16.013 | Bundled Sensor Spec Authoring and DTU-Parity Verification — 4 Initial Sensors | HS-013-01, HS-013-02 |

---

## Known-Good / Known-Problematic Corpus Note

- **Known-good corpus:** CrowdStrike DTU clone with standard fixture payloads from
  `crates/prism-dtu-crowdstrike/fixtures/parity/` — expected result: parity PASS for all 3+ real-sensor recordings and 3 synthesized cases per BC-2.16.013 §Postconditions §2 minimum coverage.
- **Known-problematic corpus:** CrowdStrike DTU clone with a fixture where `batch_size > 100` IDs
  are returned in a single QueryV2 page — expected result: spec correctly batches into multiple
  PostEntities calls of ≤ 100 each; failure to batch is a FAIL verdict.
