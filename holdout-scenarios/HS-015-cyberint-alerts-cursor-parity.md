---
document_type: holdout-scenario
level: L3
id: "HS-015"
category: "dtu-parity"
must_pass: true
priority: P0
epic_id: "PLUGIN-MIGRATION-001"
version: "1.1"
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
notes: "PLUGIN-MIGRATION-001-D holdout — Cyberint alerts cursor parity + multi-format timestamp (DTU). Authored FB-IMPL-P1-PO fix-burst-1 2026-05-20."
---

# HS-015: Cyberint Alerts Cursor Pagination — DTU Parity

**Group:** DTU Parity — Plugin Migration (PLUGIN-MIGRATION-001-D)
**Date:** 2026-05-20
**Priority:** P0
**BC Anchor:** BC-2.16.013 §Postconditions §1 (cyberint.sensor.toml), §2, §Canonical Test Vectors

---

## Scenario

Validates that `cyberint.sensor.toml` + `PipelineExecutor` against the Cyberint DTU clone
produces OCSF output equivalent to the reference OCSF fixture, per TS-PLUGIN-PARITY-001.

**Auth (ADR-028 §D2):** `cyberint.sensor.toml` declares `auth_type = "cookie_roundtrip"` —
grounded against the Cyberint DTU's cookie enforcement (`crates/prism-dtu-cyberint/src/routes/alerts.rs:43-46`
`extract_session_token()` extracting `cyberint_session` cookie). The legacy
`CyberintAuth::auth_type_name()` return `"bearer_static"` was a label bug; deleted by 001-A.

**URL (ADR-028 §D1):** Cyberint `alerts` table uses `GET /api/v1/alerts` — grounded against
DTU route registration in `crates/prism-dtu-cyberint/src/clone.rs` line 115.
(Note: prior BC v1.3 incorrectly cited `/api/alerts` from the legacy adapter; `/api/v1/alerts`
is the real endpoint.)

**Reference OCSF (ADR-028 §D3):** Reference loaded from committed fixture JSON at
`crates/prism-dtu-cyberint/fixtures/parity/reference-ocsf/alerts.json`.

The primary risk for Cyberint is multi-format timestamp parsing (`parse_timestamp()` — RFC3339,
no-timezone, microseconds, null/empty) which is NOT declaratively expressible in the current
TOML grammar (per BC-2.16.013 §Preconditions O-001 Grammar Verification). This scenario verifies
that the implementer's chosen mechanism (grammar extension or WASM plugin) correctly handles
all timestamp formats.

The `incidents` table parity test is in SKIP status per TS-PLUGIN-PARITY-001 Cyberint DTU
Gap Note; this scenario covers only the `alerts` table.

---

## Sub-Scenarios

### HS-015-01: Cyberint Alerts Happy Path — ISO-8601 Timestamps

**Preconditions:**
- `prism-dtu-cyberint` DTU clone is running (started via `BehavioralClone::start_on`)
- `cyberint.sensor.toml` loaded and passes BC-2.16.009 validation
- Timestamp handling mechanism (grammar extension or WASM plugin) implemented and active
- Fixture: 5 alert records with standard ISO-8601 timestamps
- Holdout evaluator has NOT seen the reference OCSF output before evaluation

**Steps:**
1. Start `CyberintClone` via `BehavioralClone::start_on("127.0.0.1:0", shutdown, None)`
2. Execute `PipelineExecutor::execute` for the `alerts` table via `GET /api/v1/alerts` with cursor pagination;
   auth via `cyberint_session` cookie (cookie_roundtrip; DTU enforces cookie extraction at `alerts.rs:43-46`)
3. Load reference OCSF from `crates/prism-dtu-cyberint/fixtures/parity/reference-ocsf/alerts.json`
   (per ADR-028 §D3)
4. Apply TS-PLUGIN-PARITY-001 canonicalization (Rule C: timestamps within ±1s tolerance)

**Expected Outcome:**
- Parity verdict: PASS — all 5 alert records match reference fixture; timestamps normalized to UTC
- Cursor pagination correctly extracts next-page cursor from response
- `request_count >= 1` (varies by page count)

### HS-015-02: Cyberint Alerts — Multi-Format Timestamp Edge Cases

**Preconditions:**
- Same as HS-015-01
- Fixture: alerts with mixed timestamp formats (one ISO-8601, one no-timezone, one microseconds,
  one null/empty — minimum 4 records, one per format)

**Steps:**
1. Same setup as HS-015-01 but with mixed-format fixture
2. Assert each record's normalized timestamp matches reference

**Expected Outcome:**
- Parity verdict: PASS for ISO-8601 and no-timezone records (TS-PLUGIN-PARITY-001 Rule C)
- Parity verdict: WARN for null/empty timestamp (TS-PLUGIN-PARITY-001 Rule B) — field absent from
  spec output where reference has null; acceptable per Rule B
- Zero FAILs

### HS-015-03: Cyberint Incidents SKIP Verdict

**Preconditions:**
- Same setup as HS-015-01
- Parity test targets the `cyberint.incidents` table

**Steps:**
1. Run parity test for `cyberint.incidents` table

**Expected Outcome:**
- Test returns SKIP verdict with message:
  `"cyberint incidents DTU gap — see TS-PLUGIN-PARITY-001 Cyberint DTU Gap Note"`
- Test does NOT fail; SKIP is the correct verdict until DTU `incidents` coverage is verified

---

## Behavioral Contract Linkage

| BC | Title | Sub-Scenarios |
|----|-------|---------------|
| BC-2.16.013 | Bundled Sensor Spec Authoring and DTU-Parity Verification — 4 Initial Sensors | HS-015-01, HS-015-02, HS-015-03 |

---

## Known-Good / Known-Problematic Corpus Note

- **Known-good corpus:** Cyberint DTU with ISO-8601 timestamps — expected parity PASS.
- **Known-problematic corpus:** Cyberint DTU with a null `created_at` field — expected parity WARN
  (not FAIL); if implementation raises FAIL, the parity rules are too strict (TS-PLUGIN-PARITY-001
  Rule B violation).
