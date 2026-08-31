# Demo Evidence Report — S-CLAROTY-OT-EVENTS-001

**Story:** Claroty xDome OT Activity Events Table — TOML `[[tables]]` block, 21-column Tier-1/Tier-2 spec, live structural tests (Wave A G2)
**Story version:** v1.9
**Evidence date:** 2026-08-31
**Recorder:** demo-recorder
**Product type:** CLI (Rust workspace) + live MCP (prism-live, monroe client)
**Recording tools:** VHS 0.11.0 (terminal session recordings) + annotated wire-JSON MCP transcripts

---

## Coverage Summary

All 9 acceptance criteria covered. AC-004 (live wire shape) is partially covered by
live transcript + mock-based wire shape test. The monroe OT network was quiescent at
capture time (0 OT activity events); structural shape is confirmed by the passing
`test_BC_2_16_016_claroty_ot_activity_events_wire_shape_class_uid_2004_mock` test (RG-004 wire companion).

| AC | Red Gate(s) | Evidence Artifact | Evidence Type | Status |
|----|-------------|-------------------|---------------|--------|
| AC-001 | RG-001 | AC-001-002-toml-parse (.tape/.gif/.webm) | VHS (test suite) | PASS |
| AC-002 | RG-002 | AC-001-002-toml-parse (.tape/.gif/.webm) | VHS (test suite) | PASS |
| AC-003 | RG-003 | AC-003-005-plan-gate-tests (.tape/.gif/.webm) + AC-003-005-error-paths.txt | VHS + live transcript | PASS |
| AC-004 | RG-004 wire | AC-004-wire-shape-mock (.tape/.gif/.webm) + AC-004-005-006-live-queries.txt | VHS + live transcript | PASS (quiescent caveat) |
| AC-005 | RG-009 | AC-003-005-plan-gate-tests (.tape/.gif/.webm) + AC-003-005-error-paths.txt + AC-004-005-006-live-queries.txt | VHS + live transcript | PASS |
| AC-006 | RG-005, RG-010 | AC-004-wire-shape-mock (.tape/.gif/.webm) + AC-004-005-006-live-queries.txt | VHS + live transcript | PASS (quiescent caveat) |
| AC-007 | RG-006, RG-011 | AC-007-008-null-passthrough (.tape/.gif/.webm) | VHS (test suite) | PASS |
| AC-008 | RG-007, RG-012 | AC-007-008-null-passthrough (.tape/.gif/.webm) | VHS (test suite) | PASS |
| AC-009 | RG-008 | AC-009-sap2-marker (.tape/.gif/.webm) | VHS (test suite) | PASS |

---

## Live MCP Transcript Evidence (prism-live, client: monroe)

The three transcript files capture direct JSON-RPC wire responses from the live prism binary
at `/Users/jmagady/Dev/test-soc/bin/prism` with config at `/Users/jmagady/Dev/test-soc/.prism-live/`
using the `prism-live-mcp-wrapper.sh` wrapper. Credentials are handled opaquely (AD-017).

### AC-001-002-schema-describe.txt

**Covers:** AC-001 (table registration), AC-002 (4 Tier-1 columns with OCSF field names)

**What it proves:**

- `prism_describe` for client `monroe` returns `claroty_ot_activity_events` as a registered table.
- OCSF class = `detection_finding` (confirmed by table's `description` field).
- 7 visible Arrow columns exposed:
  - `finding_info_uid` (integer, nullable=true) — Tier-1, OCSF desc: `finding_info.uid`
  - `time` (datetime, nullable=true) — Tier-1, OCSF desc: `time`
  - `activity_name` (string, nullable=true) — Tier-1, OCSF desc: `activity_name`
  - `message` (string, nullable=true) — Tier-1, OCSF desc: `message`
  - `raw_extensions` (json, nullable=true) — Tier-2 aggregate; description lists all 17 source columns:
    `source_ip, dest_ip, protocol, dest_port, source_port, ip_protocol, source_asset_id, dest_asset_id,
    source_device_name, dest_device_name, source_device_type, dest_device_type, source_site_name,
    dest_site_name, source_username, related_alert_ids, mode`
  - `class_uid` (integer, nullable=false) — synthesized OCSF class identifier
  - `_sensor` (string, nullable=false) — synthesized sensor identifier

The 21-column TOML spec collapses to 7 Arrow columns under `ocsf_column_naming=true` (ADR-058 §B2):
4 Tier-1 OCSF-named + `raw_extensions` (17 Tier-2 aggregate) + `class_uid` + `_sensor`.

### AC-003-005-error-paths.txt

**Covers:** AC-003 (Tier-2 plan-gate E-QUERY-038), AC-005 error path (detection_time E-QUERY-038)

**What it proves:**

- `SELECT source_ip FROM claroty_ot_activity_events LIMIT 1` raises E-QUERY-038 at plan time:
  ```
  E-QUERY-038: column 'source_ip' not found in table 'claroty_ot_activity_events' for client 'monroe';
  available: [_sensor, activity_name, class_uid, finding_info_uid, message, raw_extensions, time]
  ```
  `source_ip` is absent from `available_columns`; `raw_extensions` is present.

- `SELECT detection_time FROM claroty_ot_activity_events LIMIT 1` raises E-QUERY-038:
  ```
  E-QUERY-038: column 'detection_time' not found in table 'claroty_ot_activity_events' for client 'monroe';
  available: [_sensor, activity_name, class_uid, finding_info_uid, message, raw_extensions, time]
  ```
  `detection_time` is the raw TOML column name; the Arrow field name is `time`.

### AC-004-005-006-live-queries.txt

**Covers:** AC-004 (live wire shape, quiescent), AC-005 success path (SELECT time), AC-006 (SELECT raw_extensions)

**What it proves:**

- `SELECT * FROM claroty_ot_activity_events LIMIT 1` executes without error; returns well-formed
  empty result set (`rows=[], total_available=0, is_truncated=false`). The OT network was quiescent
  (0 events) at capture time 2026-08-31T15:57:15Z.
- `SELECT time FROM claroty_ot_activity_events LIMIT 1` executes without E-QUERY-038; `time` is
  the valid Arrow field name for the `detection_time` Tier-1 column.
- `SELECT raw_extensions FROM claroty_ot_activity_events LIMIT 1` executes without E-QUERY-038;
  `raw_extensions` is a valid Tier-2 aggregate Arrow column. Returns 0 rows due to quiescence.

---

## VHS Recording Files

All VHS recordings run tests against the story worktree at
`/Users/jmagady/Dev/prism/.worktrees/S-CLAROTY-OT-EVENTS-001/`
using `cargo nextest`. Compilation artifacts are pre-warmed; expected runtime per tape: 30–90s.

### AC-001-002-toml-parse

**Covers:** AC-001 (RG-001), AC-002 (RG-002)

- `AC-001-002-toml-parse.tape` — VHS script source
- `AC-001-002-toml-parse.gif` — PR-embeddable recording
- `AC-001-002-toml-parse.webm` — archival recording

**What it proves:**

- **AC-001 / RG-001:** `test_BC_2_16_016_claroty_ot_activity_events_toml_block_parses` PASS.
  `SpecLoader::parse` on `claroty.sensor.toml` returns `Ok(SensorSpec)`. The parsed spec
  reports 21 `ColumnSpec` entries for `claroty_ot_activity_events`. `related_alert_ids`
  confirmed as `ColumnType::Json` (not stringified). Traces to BC-2.16.016 §Postconditions §1.

- **AC-002 / RG-002:** `test_BC_2_16_016_claroty_ot_activity_events_four_tier1_columns` PASS.
  Exactly 4 columns have `ocsf_field == Some(_)`: `event_id`→`"finding_info.uid"` (with REQUIRED),
  `detection_time`→`"time"`, `event_type`→`"activity_name"`, `description`→`"message"`.
  Exactly 17 columns have `ocsf_field == None` (Tier-2). Traces to BC-2.16.016 §Postconditions §2.

### AC-003-005-plan-gate-tests

**Covers:** AC-003 (RG-003), AC-005 error path (RG-009)

- `AC-003-005-plan-gate-tests.tape` — VHS script source
- `AC-003-005-plan-gate-tests.gif` — PR-embeddable recording
- `AC-003-005-plan-gate-tests.webm` — archival recording

**What it proves:**

- **AC-003 / RG-003:** `test_BC_2_16_016_claroty_ot_activity_events_tier2_source_ip_raises_e_query_038`
  PASS (authoritative prism-bin end-to-end gate via `QueryEngine::execute`). E-QUERY-038 raised;
  `available_columns` excludes `source_ip`; includes `raw_extensions`, `finding_info_uid`, `time`,
  `activity_name`, `message`. Traces to BC-2.16.016 §Invariants, EC-016-016-006.

- **AC-005 error path / RG-009:** `test_BC_2_16_016_claroty_ot_activity_events_raw_toml_name_detection_time_raises_e_query_038`
  PASS (prism-bin via `QueryEngine::execute`). `SELECT detection_time` raises E-QUERY-038; use `time`
  instead. Traces to BC-2.16.016 §Postconditions §2, EC-009.

### AC-004-wire-shape-mock

**Covers:** AC-004 wire shape (mock companion), AC-006 EC-002-WIRE (RG-010)

- `AC-004-wire-shape-mock.tape` — VHS script source
- `AC-004-wire-shape-mock.gif` — PR-embeddable recording
- `AC-004-wire-shape-mock.webm` — archival recording

**What it proves:**

- **AC-004 wire shape / mock companion:** `test_BC_2_16_016_claroty_ot_activity_events_wire_shape_class_uid_2004_mock`
  PASS. Mock response with seeded OT activity event row. Serialized JSON wire output contains
  `class_uid=2004`, `finding_info_uid` present (integer or null), `time` present, `activity_name`
  present, `message` present, `raw_extensions` present as JSON object with network 5-tuple keys.
  No Tier-2 keys at top level. Traces to BC-2.16.016 §Postconditions §1 (class_uid), §2.

- **AC-006 EC-002-WIRE / RG-010:** `test_BC_2_16_016_claroty_ot_activity_events_ec002_related_alert_ids_native_json_array`
  PASS. `related_alert_ids` serialized as native JSON array (e.g., `[1,2,3]` or `[]`), NOT as
  a stringified JSON string. `column_type="json"` pass-through confirmed at wire level.
  Traces to BC-2.16.016 EC-016-016-002.

### AC-007-008-null-passthrough

**Covers:** AC-007 (RG-006, RG-011 SAP-3 gate), AC-008 (RG-007, RG-012 SAP-3 gate)

- `AC-007-008-null-passthrough.tape` — VHS script source
- `AC-007-008-null-passthrough.gif` — PR-embeddable recording
- `AC-007-008-null-passthrough.webm` — archival recording

**What it proves:**

- **AC-007 / RG-011 (SAP-3 production-path gate):**
  `test_BC_2_16_016_claroty_ot_activity_events_ac007_absent_event_id_null_finding_info_uid_production_path`
  PASS. Row missing `event_id` traverses `build_column_array` within `pipeline_result_to_record_batch`
  (reached via `SpecDrivenSensorAdapter::fetch` in `crates/prism-bin/src/spec_driven_adapter.rs`).
  Null `finding_info_uid` cell produced; row NOT dropped; `time` and `raw_extensions` remain populated;
  subsequent rows continue. No hard error. Traces to BC-2.16.016 §Invariants, EC-016-016-001.

- **AC-007 defense-in-depth / RG-006:**
  `test_BC_2_16_016_claroty_ot_activity_events_required_event_id_absent_produces_null_row` PASS.
  Unit test (mock response). Same behavior via `build_column_array` absent-field passthrough.

- **AC-008 / RG-012 (SAP-3 production-path gate):**
  `test_BC_2_16_016_claroty_ot_activity_events_ac008_absent_detection_time_null_time_production_path`
  PASS. Row with absent/null `detection_time` traverses same production path; null `time` cell
  produced; no E-SPEC-018; pagination continues. Traces to BC-2.16.016 §Invariants, EC-016-016-003.

- **AC-008 defense-in-depth / RG-007:**
  `test_BC_2_16_016_claroty_ot_activity_events_detection_time_null_passthrough` PASS.
  Unit test (mock response). Implicit ISO-8601 default (ADR-028 §D8-B) confirmed: null passthrough,
  no E-SPEC-018.

### AC-009-sap2-marker

**Covers:** AC-009 (RG-008)

- `AC-009-sap2-marker.tape` — VHS script source
- `AC-009-sap2-marker.gif` — PR-embeddable recording
- `AC-009-sap2-marker.webm` — archival recording

**What it proves:**

- **AC-009 / RG-008:** `test_BC_2_16_016_claroty_ot_activity_events_sap2_na_documented` PASS.
  Marker test asserts `SAP2_STATUS: &str = "N/A: no DTU; deferred D-2200"` constant present
  in test file. Adversarial review MUST NOT file SAP-2 parity findings against this story.
  Traces to BC-2.16.016 §Postconditions §4.

---

## Live-Data Quiescence Caveat (AC-004, AC-006)

AC-004 requires wire JSON with `class_uid=2004`, `finding_info_uid`, `raw_extensions` with
network 5-tuple keys. The live monroe OT network was **quiescent** at capture time
(2026-08-31T15:57:15Z) — no OT activity events returned 0 rows.

Evidence strategy:
1. **Live structural proof:** `SELECT * LIMIT 1`, `SELECT raw_extensions LIMIT 1` both execute
   without error — structural registration is confirmed by the live MCP transcript.
2. **Wire shape proof:** The mock-based test `test_BC_2_16_016_claroty_ot_activity_events_wire_shape_class_uid_2004_mock`
   uses a seeded mock row to assert the wire JSON shape end-to-end through the full
   `SpecDrivenSensorAdapter::fetch → pipeline_result_to_record_batch → arrow-json serialization` path.
   This is the production code path — the mock replaces the HTTP transport only.
3. **AC-006 network 5-tuple:** `SELECT raw_extensions` accepts the column without error; network
   5-tuple presence in `raw_extensions` is asserted in the mock-based wire shape test (RG-004/RG-005).

When live OT events are present on monroe, re-running `SELECT * FROM claroty_ot_activity_events LIMIT 1`
will confirm `class_uid=2004`, Tier-1 fields, and `raw_extensions` in the live wire output.

---

## BC Traceability

| Evidence Artifact | AC | BC | EC |
|-------------------|----|----|----|
| AC-001-002-schema-describe.txt | AC-001, AC-002 | BC-2.16.016 §1, §2 | — |
| AC-001-002-toml-parse (.gif/.webm) | AC-001 (RG-001) | BC-2.16.016 §1 | — |
| AC-001-002-toml-parse (.gif/.webm) | AC-002 (RG-002) | BC-2.16.016 §2 | — |
| AC-003-005-error-paths.txt | AC-003, AC-005 err | BC-2.16.016 §Invariants | EC-016-016-006, EC-009 |
| AC-003-005-plan-gate-tests (.gif/.webm) | AC-003 (RG-003) | BC-2.16.016 §Invariants | EC-016-016-006 |
| AC-003-005-plan-gate-tests (.gif/.webm) | AC-005 err (RG-009) | BC-2.16.016 §2 | EC-009 |
| AC-004-005-006-live-queries.txt | AC-004 (quiescent), AC-005 ok, AC-006 | BC-2.16.016 §1, §2 | — |
| AC-004-wire-shape-mock (.gif/.webm) | AC-004 mock (RG-004 wire) | BC-2.16.016 §1, §2 | — |
| AC-004-wire-shape-mock (.gif/.webm) | AC-006 EC-002-WIRE (RG-010) | BC-2.16.016 §2 | EC-016-016-002 |
| AC-007-008-null-passthrough (.gif/.webm) | AC-007 (RG-006, RG-011) | BC-2.16.016 §Invariants | EC-016-016-001 |
| AC-007-008-null-passthrough (.gif/.webm) | AC-008 (RG-007, RG-012) | BC-2.16.016 §Invariants | EC-016-016-003 |
| AC-009-sap2-marker (.gif/.webm) | AC-009 (RG-008) | BC-2.16.016 §4 | — |
