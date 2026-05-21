---
document_type: holdout-scenario
level: L3
id: "HS-016"
category: "dtu-parity"
must_pass: true
priority: P0
epic_id: "PLUGIN-MIGRATION-001"
version: "1.2"
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
notes: "PLUGIN-MIGRATION-001-D holdout — Armis AQL forwarding + timestamp fallback chain parity (DTU). Authored FB-IMPL-P1-PO fix-burst-1 2026-05-20."
---

# HS-016: Armis AQL Forwarding and Timestamp Fallback Chain — DTU Parity

**Group:** DTU Parity — Plugin Migration (PLUGIN-MIGRATION-001-D)
**Date:** 2026-05-20
**Priority:** P0
**BC Anchor:** BC-2.16.013 §Postconditions §1 (armis.sensor.toml), §2, §Edge Cases EC-016-013-005, §Canonical Test Vectors

---

## Scenario

Validates that `armis.sensor.toml` + `PipelineExecutor` against the Armis DTU clone produces
OCSF output equivalent to the reference OCSF fixture, per TS-PLUGIN-PARITY-001.

**Auth (ADR-028 §D2):** `armis.sensor.toml` declares `auth_type = "bearer_static"` — grounded
against the Armis DTU's `Authorization: Bearer {non-empty}` header enforcement
(per `crates/prism-dtu-armis/src/lib.rs` module documentation — HTTP 403 on missing/invalid token per Armis Centrix
API spec). The legacy `ArmisAuth::auth_type_name()` return `"api_key"` was incorrect per DTU;
deleted by 001-A.

**URL / DTU Gap Note (ADR-028 §D1 + §D5):** Armis DTU has `/api/v1/devices` (GET) and
`/api/v1/alerts` (GET) — NOT `/api/v1/search`. This scenario tests AQL forwarding against the
DTU's actual device/alert routes. See BC-2.16.013 §Known Gaps DTU-EXT-003 and DTU-EXT-004 for
the full gap analysis. The sub-scenarios below exercise the `devices` table against
`GET /api/v1/devices` and verify AQL parameter forwarding via the DTU AQL log endpoint.

**Reference OCSF (ADR-028 §D3):** Reference loaded from committed fixture JSON at
`crates/prism-dtu-armis/fixtures/parity/reference-ocsf/devices.json`.

Two Armis-specific risks are tested:

1. **AQL Forwarding:** The Armis API accepts an AQL (Armis Query Language) expression as a
   query parameter (`aql`). The spec must forward the caller's AQL expression verbatim via
   `${query.filter.aql}` interpolation (corrected from the non-existent `${query.aql}` per
   BC-2.16.013 §Preconditions O-001 Grammar Verification). The holdout evaluator verifies
   the DTU clone received the verbatim AQL expression via `GET /dtu/aql-log`.

2. **Timestamp Fallback Chain:** `firstSeen` → `lastSeen` → `DateTime::now()` fallback
   is NOT declaratively expressible in the current TOML grammar (per BC-2.16.013 §Preconditions
   O-001 Grammar Verification). The implementer's chosen mechanism (grammar extension or WASM
   plugin) is tested here. The fallback-to-`now()` path must emit a `tracing::warn!` audit signal.

---

## Sub-Scenarios

### HS-016-01: Armis Devices — AQL Expression Forwarding

**Preconditions:**
- `prism-dtu-armis` DTU clone is running (started via `BehavioralClone::start_on`)
- `armis.sensor.toml` loaded and passes BC-2.16.009 validation
- Timestamp fallback mechanism (grammar extension or WASM plugin) implemented
- `FetchContext::query_filters` contains `{"aql": "in:devices timeFrame:\"1 Day\""}` (example)
- Holdout evaluator has NOT seen the AQL log before evaluation

**Steps:**
1. Start `ArmisClone` via `BehavioralClone::start_on("127.0.0.1:0", shutdown, None)`
2. Execute `PipelineExecutor::execute` for the `devices` table with the AQL context;
   auth via `Authorization: Bearer {token}` header (bearer_static; DTU enforces 403 on missing token)
3. Load reference OCSF from `crates/prism-dtu-armis/fixtures/parity/reference-ocsf/devices.json`
   (per ADR-028 §D3)
4. Check DTU AQL log (`GET /dtu/aql-log`) — Armis DTU records received AQL expressions

**Expected Outcome:**
- DTU AQL log contains the verbatim AQL expression `"in:devices timeFrame:\"1 Day\""`
- Parity verdict: PASS — DTU receives AQL expression unmodified; response normalized correctly
- `request_count >= 1`

### HS-016-02: Armis Devices — Timestamp Fallback to firstSeen

**Preconditions:**
- Same as HS-016-01
- Fixture: device record with `firstSeen: "2026-01-15T10:00:00Z"` and `lastSeen` absent

**Steps:**
1. Execute `PipelineExecutor::execute` for `devices` table
2. Verify `time` OCSF field is set from `firstSeen`

**Expected Outcome:**
- Parity verdict: PASS — `time` field matches `firstSeen` timestamp from reference output
- No WARN logged (firstSeen is present; fallback not triggered)

### HS-016-03: Armis Devices — Timestamp Fallback to now() with WARN

**Preconditions:**
- Same as HS-016-01
- Fixture: device record with `firstSeen` absent AND `lastSeen` absent

**Steps:**
1. Execute `PipelineExecutor::execute` for `devices` table

**Expected Outcome:**
- Parity verdict: PASS by TS-PLUGIN-PARITY-001 Rule C convention ("both took same fallback path")
  — both spec-driven and reference use fetch-time timestamp when all preferred fields absent
- WARN logged: `event_type = "timestamp_fallback_to_now"` (or equivalent audit signal)
- `tracing::warn!` emitted — audit signal preserved from prior Rust adapter behavior

---

## Behavioral Contract Linkage

| BC | Title | Sub-Scenarios |
|----|-------|---------------|
| BC-2.16.013 | Bundled Sensor Spec Authoring and DTU-Parity Verification — 4 Initial Sensors | HS-016-01, HS-016-02, HS-016-03 |

---

## Known-Good / Known-Problematic Corpus Note

- **Known-good corpus:** Armis DTU with `firstSeen` present on all records — expected parity PASS,
  no WARN.
- **Known-problematic corpus:** Armis DTU with `firstSeen` and `lastSeen` both absent — expected
  parity PASS by Rule C convention AND WARN emitted. If WARN is absent, the audit signal from
  the prior Rust adapter is lost — FAIL.

---

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.2 | FB-IMPL-P6-PO fix-burst-6 | 2026-05-20 | product-owner | Closes pass-6 finding F-LP6-LOW-001 (TD-VSDD-091 anti-volatile-pin sibling-asymmetric): replaced line-pinned cite `lib.rs:16-17` with module-doc anchor `crates/prism-dtu-armis/src/lib.rs module documentation` in §Scenario auth note. POL-25 multi-cite sweep — BC-2.16.013 updated in same burst. HOLDOUT-INDEX v1.6→v1.7. |
| 1.1 | FB-IMPL-P4-PO fix-burst-4 | 2026-05-20 | product-owner | auth corrected to `bearer_static` per ADR-028 §D2; DTU gap noted for AQL routes DTU-EXT-003/004; fixture reference added; bearer auth step added. |
| 1.0 | D-731 PLUGIN-MIGRATION-001-D PO authoring | 2026-05-20 | product-owner | Initial draft — HS anchor for PLUGIN-MIGRATION-001-D; 3 sub-scenarios covering AQL forwarding, timestamp resolution, and timestamp fallback to now() with WARN audit signal. |
