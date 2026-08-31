# Demo Evidence Report — S-CLAROTY-DEVVULNREL-001

**Story:** Claroty xDome Device-Vulnerability Relations Table — TOML `[[tables]]` block, 13-column Tier-1/Tier-2 spec, composite PK (vulnerability_name + device_uid), live structural tests (Wave B G3)
**Story version:** v1.8
**Evidence date:** 2026-08-31
**Recorder:** demo-recorder
**Product type:** CLI (Rust workspace) + live MCP (prism-live, monroe client)
**Recording tools:** VHS 0.11.0 (terminal session recordings) + annotated wire-JSON MCP transcripts

---

## Coverage Summary

All 9 acceptance criteria covered. AC-004 and AC-005 are covered by BOTH live MCP transcripts
(with real data — monroe has device_vulnerability_relations data) AND mock-based wire shape tests
(RG-010, RG-011). AC-009 empty-array passthrough is covered by mock-based tests; live data shows
null for `device_vulnerability_relevance_reasons` (valid: open vulnerabilities at capture time).

| AC | Red Gate(s) | Evidence Artifact | Evidence Type | Status |
|----|-------------|-------------------|---------------|--------|
| AC-001 | RG-001 | AC-001-002-toml-parse (.tape/.gif/.webm) + AC-001-002-schema-describe.txt | VHS (test suite) + live transcript | PASS |
| AC-002 | RG-002 | AC-001-002-toml-parse (.tape/.gif/.webm) + AC-001-002-schema-describe.txt | VHS (test suite) + live transcript | PASS |
| AC-003 | RG-003 | AC-003-plan-gate-tests (.tape/.gif/.webm) + AC-003-005-error-paths.txt | VHS + live transcript | PASS |
| AC-004 | RG-011, RG-010 | AC-004-wire-shape-mock (.tape/.gif/.webm) + AC-004-005-009-live-queries.txt | VHS (Arrow-level + serialized-JSON) + live transcript | PASS |
| AC-005 | RG-010, RG-011 | AC-004-wire-shape-mock (.tape/.gif/.webm) + AC-003-005-error-paths.txt + AC-004-005-009-live-queries.txt | VHS + live transcripts | PASS |
| AC-006 | RG-006, RG-010 | AC-006-009-edge-cases (.tape/.gif/.webm) + AC-004-wire-shape-mock (.tape/.gif/.webm) | VHS (test suite) + VHS (production-path serialized-JSON) | PASS |
| AC-007 | RG-007 | AC-006-009-edge-cases (.tape/.gif/.webm) | VHS (test suite) | PASS |
| AC-008 | RG-008 | AC-006-009-edge-cases (.tape/.gif/.webm) | VHS (test suite) | PASS |
| AC-009 | RG-009, RG-010 | AC-006-009-edge-cases (.tape/.gif/.webm) + AC-004-wire-shape-mock (.tape/.gif/.webm) + AC-004-005-009-live-queries.txt | VHS + live transcript | PASS (mock-coverage + live-null observation) |

---

## Live MCP Transcript Evidence (prism-live, client: monroe)

The three transcript files capture direct JSON-RPC wire responses from the live prism binary
at `/Users/jmagady/Dev/test-soc/bin/prism` with config at `/Users/jmagady/Dev/test-soc/.prism-live/`
using the `prism-live-mcp-wrapper.sh` wrapper. Credentials are handled opaquely (AD-017).

### AC-001-002-schema-describe.txt

**Covers:** AC-001 (table registration), AC-002 (2 Tier-1 columns with OCSF field names)

**What it proves:**

- `prism_describe` for client `monroe` returns `claroty_device_vulnerability_relations` as a
  registered table with `description = "vulnerability_finding"` and `sensor_type = "claroty"`.
- 5 visible Arrow columns exposed:
  - `finding_info_title` (string, nullable=true) — Tier-1, OCSF description: `finding_info.title`
    — maps from `vulnerability_name` TOML column with `ocsf_field="finding_info.title"`, REQUIRED
  - `time` (datetime, nullable=true) — Tier-1, OCSF description: `time`
    — maps from `device_vulnerability_detection_date` TOML column with `ocsf_field="time"`
  - `raw_extensions` (json, nullable=true) — Tier-2 aggregate; description lists all 11 source columns:
    `device_uid, device_name, device_asset_id, vulnerability_id, vulnerability_cvss_v3_score,
    vulnerability_is_known_exploited, vulnerability_epss_score, device_vulnerability_resolution_date,
    device_vulnerability_relevance_reasons, patch_install_date, device_site_name`
  - `class_uid` (integer, nullable=false) — synthesized OCSF class identifier (value: 2002)
  - `_sensor` (string, nullable=false) — synthesized sensor identifier (value: "claroty")

The 13-column TOML spec collapses to 5 Arrow columns under `ocsf_column_naming=true` (ADR-058 §B2):
2 Tier-1 OCSF-named + `raw_extensions` (11 Tier-2 aggregate) + `class_uid` + `_sensor`.

### AC-003-005-error-paths.txt

**Covers:** AC-003 (Tier-2 plan-gate E-QUERY-038), AC-005 error path (OCSF rename enforcement)

**What it proves:**

- `SELECT device_uid FROM claroty_device_vulnerability_relations LIMIT 1` raises E-QUERY-038:
  ```
  E-QUERY-038: column 'device_uid' not found in table 'claroty_device_vulnerability_relations'
  for client 'monroe'; available: [_sensor, class_uid, finding_info_title, raw_extensions, time]
  ```
  `device_uid` is absent from `available_columns`; `raw_extensions` is present.

- `SELECT device_vulnerability_detection_date FROM claroty_device_vulnerability_relations LIMIT 1`
  raises E-QUERY-038:
  ```
  E-QUERY-038: column 'device_vulnerability_detection_date' not found in table
  'claroty_device_vulnerability_relations' for client 'monroe'; available:
  [_sensor, class_uid, finding_info_title, raw_extensions, time]
  ```
  `device_vulnerability_detection_date` is the raw TOML column name; the Arrow field name is `time`.

### AC-004-005-009-live-queries.txt

**Covers:** AC-004 (live wire shape with real data), AC-005 success paths, AC-009 live observation

**What it proves:**

Live data from the monroe Claroty xDome instance — 3 device-vulnerability relation rows captured:

- `SELECT * FROM claroty_device_vulnerability_relations LIMIT 3` returns 3 rows with:
  - `class_uid = 2002` (vulnerability_finding OCSF class) — all 3 rows ✓
  - `finding_info_title`: `"CVE-2024-38213"`, `"CVE-2024-38193"`, `"CVE-2024-38107"` (real CVE IDs) ✓
  - `time`: `"2024-10-16T17:19:30Z"` (Tier-1 OCSF Arrow name for detection date) ✓
  - `raw_extensions`: JSON string present (non-null) containing:
    - `device_uid`: `"00020bd4-f2fd-453b-9351-5b560dff0f55"` (composite PK join key to claroty_devices) ✓
    - `vulnerability_id`: `"ABCQDNLZ"`, `"ABEASJGT"`, `"ABFLWBIP"` (opaque Claroty IDs) ✓
    - `vulnerability_cvss_v3_score`: 6.5, 7.8, 7.8 (CVSS v3 risk triage) ✓
    - `vulnerability_epss_score`: 0.13626, 0.28529, 0.01635 (EPSS exploit probability) ✓
    - `vulnerability_is_known_exploited`: `true` (CISA KEV indicator) ✓
    - `device_vulnerability_resolution_date`: `null` (open vulnerabilities — EC-016-017-002) ✓
    - `patch_install_date`: `null` (unpatched devices — EC-016-017-003) ✓
  - Tier-2 columns NOT at row root: `device_uid`, `vulnerability_cvss_v3_score`, `device_name`
    absent as standalone top-level keys ✓

- `SELECT raw_extensions LIMIT 3`: accepted without E-QUERY-038; returns all 11 Tier-2 keys ✓
- `SELECT time LIMIT 1`: accepted; returns `"2024-10-16T17:19:30Z"` ✓
- `SELECT finding_info_title LIMIT 2`: accepted; returns `"CVE-2024-38213"`, `"CVE-2024-38193"` ✓

AC-009 live observation: `device_vulnerability_relevance_reasons = null` in live rows (valid:
open-vulnerability rows at capture time). Empty-array [] passthrough confirmed by mock-based
tests RG-009 (prism-sensors unit) and RG-010 (prism-bin wire-level).

---

## VHS Recording Files

All VHS recordings run tests against the story worktree at
`/Users/jmagady/Dev/prism/.worktrees/S-CLAROTY-DEVVULNREL-001/`
using `cargo nextest`. Compilation artifacts are pre-warmed; expected runtime per tape: 30–120s.

### AC-001-002-toml-parse

**Covers:** AC-001 (RG-001), AC-002 (RG-002)

- `AC-001-002-toml-parse.tape` — VHS script source
- `AC-001-002-toml-parse.gif` — PR-embeddable recording
- `AC-001-002-toml-parse.webm` — archival recording

**What it proves:**

- **AC-001 / RG-001:** `test_BC_2_16_017_claroty_device_vulnerability_relations_toml_block_parses`
  PASS. `SpecLoader::parse` on `claroty.sensor.toml` returns `Ok(SensorSpec)`. The parsed spec
  reports 13 `ColumnSpec` entries for `claroty_device_vulnerability_relations`. Pagination is
  `offset_limit` with `page_size=1000`. Traces to BC-2.16.017 §Postconditions §1.

- **AC-002 / RG-002:** `test_BC_2_16_017_claroty_device_vulnerability_relations_tier1_columns_two_with_ocsf_field`
  PASS. Exactly 2 columns have `ocsf_field == Some(_)`:
  - `vulnerability_name` → `"finding_info.title"` with `options = ["REQUIRED"]`
  - `device_vulnerability_detection_date` → `"time"`
  Exactly 11 columns have `ocsf_field == None` (Tier-2, all aggregate into `raw_extensions`).
  Traces to BC-2.16.017 §Postconditions §2.

### AC-003-plan-gate-tests

**Covers:** AC-003 (RG-003)

- `AC-003-plan-gate-tests.tape` — VHS script source
- `AC-003-plan-gate-tests.gif` — PR-embeddable recording
- `AC-003-plan-gate-tests.webm` — archival recording

**What it proves:**

- **AC-003 / RG-003 (authoritative):**
  `test_BC_2_16_017_claroty_device_vulnerability_relations_e2e_e_query_038_tier2_device_uid`
  PASS. End-to-end via `QueryEngine::execute` (prism-bin). `SELECT device_uid` raises E-QUERY-038;
  `available_columns` includes `raw_extensions`, `finding_info_title`, `time`; excludes `device_uid`.
  Traces to BC-2.16.017 §Invariants, EC-016-017-006.

### AC-004-wire-shape-mock

**Covers:** AC-004 wire shape (mock, Arrow-level via RG-011 + serialized-JSON via RG-010), AC-006 null-not-absent (production path, RG-010), AC-009 EC-004-WIRE (RG-010)

- `AC-004-wire-shape-mock.tape` — VHS script source
- `AC-004-wire-shape-mock.gif` — PR-embeddable recording
- `AC-004-wire-shape-mock.webm` — archival recording

**What it proves:**

- **AC-004 wire shape / RG-011:**
  `test_BC_2_16_017_claroty_device_vulnerability_relations_wire_shape_class_uid_2002_mock`
  PASS. Mock response with seeded device-vulnerability row. Arrow RecordBatch-level assertions
  (NOT serialized JSON — no `arrow_json` writer in this test):
  - `class_uid == 2002` (Int32Array downcast) — OCSF class vulnerability_finding.
  - `finding_info_title` column present by name — Tier-1 OCSF Arrow field.
  - `raw_extensions` present as StringArray (DataType::Utf8), valid JSON object.
  - `device_vulnerability_relevance_reasons` inside `raw_extensions` is a native JSON array
    `["Software","Configuration"]` (NOT stringified) — load-bearing SAP-4 assertion on the
    `build_column_array` ColumnType::Json arm.
  - No Tier-2 column name at RecordBatch top level (all 11 Tier-2 names checked).
  Production code path: `SpecDrivenSensorAdapter::fetch → pipeline_result_to_record_batch → build_column_array`.
  Traces to BC-2.16.017 §PC1 (class_uid=2002), §PC2 (Tier-1/Tier-2), §PC3 (composite PK).

- **AC-004 serialized-JSON + AC-006 null-not-absent + AC-009 EC-004-WIRE / RG-010:**
  `test_BC_2_16_017_claroty_device_vulnerability_relations_wire_shape_native_json_array_and_null_passthrough`
  PASS. Two-record mock; full production serialization path via
  `arrow_json::writer::WriterBuilder::new().with_explicit_nulls(true)`.

  **Row 0 assertions (serialized JSON level):**
  - `class_uid = 2002` in row0 wire output.
  - `finding_info_title = "CVE-2024-EMPTY-REASONS"` in row0 wire output.
  - `time` key present, non-null, ISO-8601 string starting `"2024-04-01T08:00:00"` —
    Tier-1 `device_vulnerability_detection_date` → Arrow Timestamp(Microsecond, UTC) →
    wire string. **MED-1 LOAD-BEARING assertion** (added at HEAD bc5732ecf): verifies the
    Datetime→time column propagates through the production `arrow_json` serialization path.
  - `raw_extensions` JSON string with `device_vulnerability_relevance_reasons = []`
    (native empty JSON array, NOT null, NOT the string `"[]"`; wire form is exactly `"[]"`).
  - No Tier-2 column at row0 root (11 names checked).

  **Row 1 assertions (serialized JSON level):**
  - `finding_info_title = null` present (null-not-absent); `vulnerability_name` absent in the
    API record → Arrow null cell → `{"finding_info_title": null}` with `explicit_nulls=true`.
    Production-path companion to RG-006. BC-2.11.001 EC-11-079.
  - `time = null` present (null-not-absent); row1 has no `device_vulnerability_detection_date`
    → Arrow null Timestamp cell → `{"time": null}` with `explicit_nulls=true`.
    **MED-1 LOAD-BEARING assertion** (added at HEAD bc5732ecf). ADR-028 §D8-B.
  - `raw_extensions` present; `device_vulnerability_relevance_reasons = ["Network"]` native array.
  - `device_uid` present inside `raw_extensions`.

  Traces to BC-2.16.017 §PC1 (class_uid=2002), §PC2 (Tier-1/Tier-2), AC-004, AC-006;
  BC-2.11.001 EC-11-079 (null-not-absent); EC-016-017-004 (empty array passthrough).

### AC-006-009-edge-cases

**Covers:** AC-006 (RG-006), AC-007 (RG-007), AC-008 (RG-008), AC-009 (RG-009)

- `AC-006-009-edge-cases.tape` — VHS script source
- `AC-006-009-edge-cases.gif` — PR-embeddable recording
- `AC-006-009-edge-cases.webm` — archival recording

**What it proves:**

- **AC-006 / RG-006:**
  `test_BC_2_16_017_claroty_device_vulnerability_relations_required_vulnerability_name_absent_produces_null_row`
  PASS. Row missing `vulnerability_name` (REQUIRED column) → null row produced; no hard error;
  subsequent rows continue to materialize. Traces to BC-2.16.017 §Invariants, EC-016-017-001.

- **AC-007 / RG-007:**
  `test_BC_2_16_017_claroty_device_vulnerability_relations_nullable_count_uses_empty_page_halt`
  PASS. `devices_vulnerabilities` envelope with `count: null` → empty-page halt; no null-deref;
  pagination terminates gracefully. Matches claroty_vulnerabilities BC-2.16.015 EC-016-015-003 pattern.
  Traces to BC-2.16.017 §PC1 pagination note, EC-016-017-005.

- **AC-008 / RG-008:**
  `test_BC_2_16_017_claroty_device_vulnerability_relations_nullable_datetime_fields_pass_through`
  PASS. `device_vulnerability_resolution_date = null` and `patch_install_date = null` → null
  cells in `raw_extensions`; no E-SPEC-018; pagination continues. ADR-028 §D8-B null passthrough
  confirmed. Traces to BC-2.16.017 §Invariants, EC-016-017-002, EC-016-017-003.

- **AC-009 / RG-009:**
  `test_BC_2_16_017_claroty_device_vulnerability_relations_empty_relevance_reasons_array_serialized`
  PASS. `device_vulnerability_relevance_reasons = []` → stored as native JSON array `[]` in
  `raw_extensions` (NOT a JSON string `"[]"`); not null; no error.
  Traces to BC-2.16.017 EC-016-017-004.

### AC-009-sap2-marker

**Covers:** SAP-2 N/A documentation

- `AC-009-sap2-marker.tape` — VHS script source
- `AC-009-sap2-marker.gif` — PR-embeddable recording
- `AC-009-sap2-marker.webm` — archival recording

**What it proves:**

- **SAP-2 N/A marker (prism-sensors):** `test_BC_2_16_017_..._sap2_na_documented` PASS.
  `SAP2_STATUS: &str = "N/A: no DTU; deferred D-2200"` constant present. Adversarial review
  MUST NOT file SAP-2 parity findings against this story.
- **SAP-2 N/A marker (prism-bin):** `test_BC_2_16_017_..._wire_shape_sap2_na_documented` PASS.
  Companion marker in the wire-shape test file.
  Traces to BC-2.16.017 §Postconditions §5 (SAP-2 DTU-parity N/A).

---

## Live Data Note (AC-004, AC-005)

Unlike the sibling G2 story (S-CLAROTY-OT-EVENTS-001 where OT network was quiescent),
the monroe `claroty_device_vulnerability_relations` table has **live data** with real CVE rows.
The live wire transcript confirms all AC-004 and AC-005 requirements against actual production data:

- `class_uid = 2002` ✓ (vulnerability_finding OCSF class)
- `finding_info_title = "CVE-2024-38213"` ✓ (real CVE ID — no mock needed)
- `time = "2024-10-16T17:19:30Z"` ✓ (Tier-1 OCSF Arrow name for detection date)
- `raw_extensions` contains `device_uid`, `vulnerability_id`, `vulnerability_cvss_v3_score`,
  `vulnerability_epss_score`, `vulnerability_is_known_exploited = true` ✓
- No Tier-2 keys at row root ✓

The mock-based test (RG-011) provides additional production-path wire serialization coverage
via the full `SpecDrivenSensorAdapter::fetch` path without network dependency.

---

## BC Traceability

| Evidence Artifact | AC | BC | EC |
|-------------------|----|----|----|
| AC-001-002-schema-describe.txt | AC-001, AC-002 | BC-2.16.017 §PC1, §PC2 | — |
| AC-001-002-toml-parse (.gif/.webm) | AC-001 (RG-001) | BC-2.16.017 §PC1 | — |
| AC-001-002-toml-parse (.gif/.webm) | AC-002 (RG-002) | BC-2.16.017 §PC2 | — |
| AC-003-005-error-paths.txt | AC-003, AC-005 err | BC-2.16.017 §Invariants | EC-016-017-006 |
| AC-003-plan-gate-tests (.gif/.webm) | AC-003 (RG-003) | BC-2.16.017 §Invariants | EC-016-017-006 |
| AC-004-005-009-live-queries.txt | AC-004 (live data), AC-005 ok, AC-009 obs | BC-2.16.017 §PC1, §PC2, §PC3 | EC-016-017-002, EC-016-017-003 |
| AC-004-wire-shape-mock (.gif/.webm) | AC-004 (RG-011, Arrow-level) | BC-2.16.017 §PC1, §PC2, §PC3 | — |
| AC-004-wire-shape-mock (.gif/.webm) | AC-004 (RG-010, serialized-JSON) | BC-2.16.017 §PC1, §PC2 | — |
| AC-004-wire-shape-mock (.gif/.webm) | AC-006 (RG-010, null-not-absent production path) | BC-2.16.017 §Invariants | BC-2.11.001 EC-11-079 |
| AC-004-wire-shape-mock (.gif/.webm) | AC-009 EC-004-WIRE (RG-010) | BC-2.16.017 §PC2 | EC-016-017-004 |
| AC-006-009-edge-cases (.gif/.webm) | AC-006 (RG-006) | BC-2.16.017 §Invariants | EC-016-017-001 |
| AC-006-009-edge-cases (.gif/.webm) | AC-007 (RG-007) | BC-2.16.017 §PC1 pagination | EC-016-017-005 |
| AC-006-009-edge-cases (.gif/.webm) | AC-008 (RG-008) | BC-2.16.017 §Invariants | EC-016-017-002, EC-016-017-003 |
| AC-006-009-edge-cases (.gif/.webm) | AC-009 (RG-009) | BC-2.16.017 §PC2 | EC-016-017-004 |
| AC-009-sap2-marker (.gif/.webm) | SAP-2 N/A (§PC5) | BC-2.16.017 §PC5 | — |
