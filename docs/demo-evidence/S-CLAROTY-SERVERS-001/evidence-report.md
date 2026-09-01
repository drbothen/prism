# Demo Evidence Report — S-CLAROTY-SERVERS-001

**Story:** Claroty xDome Collection Servers + Server Interfaces Tables — TOML `[[tables]]` blocks, 17-column + 10-column Tier-1/Tier-2 spec, live structural tests (Wave C G4)
**Story version:** v1.8
**Evidence date:** 2026-09-01
**Recorder:** demo-recorder
**Product type:** CLI (Rust workspace) + live MCP (prism-live, monroe client)
**Recording tools:** VHS 0.11.0 (terminal session recordings) + annotated wire-JSON MCP transcripts

---

## Coverage Summary

All 16 acceptance criteria covered across 2 tables (claroty_servers: AC-001..AC-008; claroty_server_interfaces: AC-009..AC-016). AC-005 and AC-013 (live wire shape) are covered by live MCP transcripts AND mock-based wire shape tests. Live data is active on monroe — 4 collection servers and multiple server interfaces returned real rows.

| AC | Red Gate(s) | Evidence Artifact | Evidence Type | Status |
|----|-------------|-------------------|---------------|--------|
| AC-001 | RG-001 | AC-001-002-009-010-toml-parse (.tape/.gif/.webm) + AC-001-002-009-010-schema-describe.txt | VHS (test suite) + live MCP transcript | PASS |
| AC-002 | RG-002 | AC-001-002-009-010-toml-parse (.tape/.gif/.webm) + AC-001-002-009-010-schema-describe.txt | VHS (test suite) + live MCP transcript | PASS |
| AC-003 | RG-003 | AC-003-004-011-012-plan-gate-tests (.tape/.gif/.webm) + AC-003-004-011-012-error-paths.txt | VHS (prism-bin E2E) + live MCP transcript | PASS |
| AC-004 | RG-004 | AC-003-004-011-012-plan-gate-tests (.tape/.gif/.webm) + AC-003-004-011-012-error-paths.txt | VHS (plan-time) + live MCP transcript | PASS |
| AC-005 | RG-017 wire | AC-005-013-wire-shape-mock (.tape/.gif/.webm) + AC-005-006-013-014-live-queries.txt | VHS (mock path) + live MCP transcript | PASS |
| AC-006 | RG-006 (#ignore) | AC-005-006-013-014-live-queries.txt | Live MCP transcript | PASS |
| AC-007 | RG-007 | AC-007-008-015-016-null-passthrough (.tape/.gif/.webm) | VHS (unit mock) | PASS |
| AC-008 | RG-008 | AC-007-008-015-016-null-passthrough (.tape/.gif/.webm) | VHS (unit mock) | PASS |
| AC-009 | RG-009 | AC-001-002-009-010-toml-parse (.tape/.gif/.webm) + AC-001-002-009-010-schema-describe.txt | VHS (test suite) + live MCP transcript | PASS |
| AC-010 | RG-010 | AC-001-002-009-010-toml-parse (.tape/.gif/.webm) + AC-001-002-009-010-schema-describe.txt | VHS (test suite) + live MCP transcript | PASS |
| AC-011 | RG-011 | AC-003-004-011-012-plan-gate-tests (.tape/.gif/.webm) + AC-003-004-011-012-error-paths.txt | VHS (prism-bin E2E) + live MCP transcript | PASS |
| AC-012 | RG-012 | AC-003-004-011-012-plan-gate-tests (.tape/.gif/.webm) + AC-003-004-011-012-error-paths.txt | VHS (plan-time) + live MCP transcript | PASS |
| AC-013 | RG-020 wire | AC-005-013-wire-shape-mock (.tape/.gif/.webm) + AC-005-006-013-014-live-queries.txt | VHS (mock path) + live MCP transcript | PASS |
| AC-014 | RG-014 (#ignore) | AC-005-006-013-014-live-queries.txt | Live MCP transcript | PASS |
| AC-015 | RG-015a, RG-015b | AC-007-008-015-016-null-passthrough (.tape/.gif/.webm) | VHS (unit mock, two sub-tests) | PASS |
| AC-016 | RG-016 | AC-007-008-015-016-null-passthrough (.tape/.gif/.webm) | VHS (unit mock) | PASS |

---

## Live MCP Transcript Evidence (prism-live, client: monroe)

All transcript files capture direct JSON-RPC wire responses from the live prism binary at
`/Users/jmagady/Dev/test-soc/bin/prism` with config at `/Users/jmagady/Dev/test-soc/.prism-live/`
using the `prism-live-mcp-wrapper.sh` wrapper. Credentials handled opaquely (AD-017).

**Live data status:** Both tables are active on monroe. 4 collection server rows were returned for
`claroty_servers`. Multiple interface rows (including eno4, eno2, eno3, enp59s0f0, enp59s0f2) were
returned for `claroty_server_interfaces`. No quiescence caveat applies to this story.

**raw_extensions wire-shape note:** `raw_extensions` is emitted as a JSON-serialized STRING on the
wire (not a native JSON object). This is the current observed behavior under separate architect
adjudication. Transcript files describe what is actually emitted — no claims are made about whether
this is the intended final form.

### AC-001-002-009-010-schema-describe.txt

**Covers:** AC-001, AC-002 (claroty_servers schema), AC-009, AC-010 (claroty_server_interfaces schema)

**What it proves:**

- `prism_describe` for client `monroe` returns both `claroty_servers` and `claroty_server_interfaces`
  as registered tables (9 total tables listed, up from 7 in G3). Both have `description=inventory_info`
  and `sensor_type=claroty`.

- `claroty_servers` exposes 5 Arrow columns:
  - `device_name` (string, device.name) — Tier-1, OCSF rename of `server_name`
  - `status_code` (string, status_code) — Tier-1, OCSF rename of `server_status`
  - `raw_extensions` (json) — Tier-2 aggregate of 15 source columns: server_location, site_id,
    model, os_version, serial_number, num_of_interfaces, management_ip, idrac_ip, management_mac,
    uptime_days, avg_traffic_past_month_mbps, avg_traffic_past_week_mbps, avg_traffic_past_hour_mbps,
    num_of_open_incidents, notes
  - `class_uid` (integer, nullable=false) — synthesized OCSF class identifier
  - `_sensor` (string, nullable=false) — synthesized sensor identifier

- `claroty_server_interfaces` exposes 5 Arrow columns:
  - `device_name` (string, device.name) — Tier-1, OCSF rename of `server_name`
  - `status_code` (string, status_code) — Tier-1, OCSF rename of `interface_status`
  - `raw_extensions` (json) — Tier-2 aggregate of 8 source columns: interface_name, interface_type,
    interface_connection_type, site_id, avg_traffic_past_month_mbps, avg_traffic_past_week_mbps,
    avg_traffic_past_hour_mbps, notes (includes composite PK element `interface_name`)
  - `class_uid` (integer, nullable=false)
  - `_sensor` (string, nullable=false)

### AC-003-004-011-012-error-paths.txt

**Covers:** AC-003, AC-004 (claroty_servers E-QUERY-038), AC-011, AC-012 (claroty_server_interfaces E-QUERY-038)

**What it proves:**

- `SELECT server_name FROM claroty_servers LIMIT 1` raises E-QUERY-038:
  ```
  E-QUERY-038: column 'server_name' not found in table 'claroty_servers' for client 'monroe';
  available: [_sensor, class_uid, device_name, raw_extensions, status_code]
  ```
  Raw TOML column name rejected; Arrow name `device_name` is in available_columns.

- `SELECT server_status FROM claroty_servers LIMIT 1` raises E-QUERY-038:
  ```
  E-QUERY-038: column 'server_status' not found in table 'claroty_servers' for client 'monroe';
  available: [_sensor, class_uid, device_name, raw_extensions, status_code]
  ```
  Raw TOML column name rejected; Arrow name `status_code` is in available_columns.

- `SELECT interface_name FROM claroty_server_interfaces LIMIT 1` raises E-QUERY-038:
  ```
  E-QUERY-038: column 'interface_name' not found in table 'claroty_server_interfaces' for client 'monroe';
  available: [_sensor, class_uid, device_name, raw_extensions, status_code]
  ```
  Composite PK element `interface_name` is Tier-2 (no `ocsf_field`); only accessible via raw_extensions.

- `SELECT interface_status FROM claroty_server_interfaces LIMIT 1` raises E-QUERY-038:
  ```
  E-QUERY-038: column 'interface_status' not found in table 'claroty_server_interfaces' for client 'monroe';
  available: [_sensor, class_uid, device_name, raw_extensions, status_code]
  ```
  Raw TOML column name rejected; Arrow name `status_code` is in available_columns.

### AC-005-006-013-014-live-queries.txt

**Covers:** AC-005 (claroty_servers live wire shape), AC-006 (raw_extensions keys), AC-013 (claroty_server_interfaces live wire shape), AC-014 (raw_extensions keys with composite PK join element)

**What it proves:**

- `SELECT * FROM claroty_servers LIMIT 1` returns 1 live row: `class_uid=5001`,
  `device_name="monroeenergy-chlstn-collection-1"`, `status_code="up"` (live lowercase value,
  per casing note in AC-005), `raw_extensions="{...}"` (JSON-serialized string; non-null).
  No standalone Tier-2 root keys. `_source_table="claroty_servers"`.

- `SELECT raw_extensions FROM claroty_servers LIMIT 5` returns 4 rows (total_available=4).
  raw_extensions JSON string contains management_ip, model, os_version, serial_number (all present
  with non-null values in rows 1-4). uptime_days fractional (e.g., 36.55728) confirms Float type.

- `SELECT * FROM claroty_server_interfaces LIMIT 1` returns 1 live row: `class_uid=5001`,
  `device_name="monroeenergy-chlstn-collection-1"`, `status_code="down"` (interface down state),
  `raw_extensions="{...}"` containing interface_name="eno4", interface_type="span",
  interface_connection_type="SFP+". `_source_table="claroty_server_interfaces"` confirms
  SEPARATE endpoint (distinct from claroty_servers).

- `SELECT raw_extensions FROM claroty_server_interfaces LIMIT 5` returns 5 rows.
  raw_extensions JSON string contains interface_name (eno4, eno2, eno3, enp59s0f0, enp59s0f2),
  interface_type (span/management), interface_connection_type (SFP+/RJ45). Composite PK join
  element `interface_name` accessible via raw_extensions.

---

## VHS Recording Files

All VHS recordings run tests against the story worktree at
`/Users/jmagady/Dev/prism/.worktrees/S-CLAROTY-SERVERS-001/`
using `cargo nextest`. Compilation artifacts are pre-warmed; expected runtime per tape: 30–90s.

### AC-001-002-009-010-toml-parse

**Covers:** AC-001 (RG-001), AC-002 (RG-002), AC-009 (RG-009), AC-010 (RG-010)

- `AC-001-002-009-010-toml-parse.tape` — VHS script source
- `AC-001-002-009-010-toml-parse.gif` — PR-embeddable recording
- `AC-001-002-009-010-toml-parse.webm` — archival recording

**What it proves:**

- **AC-001 / RG-001:** `test_BC_2_16_018_claroty_servers_toml_block_parses` PASS.
  `SpecLoader::parse` on `claroty.sensor.toml` returns `Ok(SensorSpec)`. 17 `ColumnSpec` entries
  for `claroty_servers`. Pagination `offset_limit` / page_size 1000. Traces to BC-2.16.018 §PC1.

- **AC-002 / RG-002:** `test_BC_2_16_018_claroty_servers_tier1_columns_two_with_ocsf_field` PASS.
  Exactly 2 columns have `ocsf_field == Some(_)`: `server_name`→`"device.name"` (REQUIRED),
  `server_status`→`"status_code"`. Exactly 15 columns have `ocsf_field == None` (Tier-2).
  Traces to BC-2.16.018 §PC2.

- **AC-009 / RG-009:** `test_BC_2_16_019_claroty_server_interfaces_toml_block_parses` PASS.
  `SpecLoader::parse` returns `Ok(SensorSpec)`. 10 `ColumnSpec` entries for
  `claroty_server_interfaces`. `path_template = "/api/v1/server_interfaces/"` (SEPARATE endpoint).
  `response_path = "$.server_interfaces"`. Pagination `offset_limit` / page_size 1000.
  Traces to BC-2.16.019 §PC1.

- **AC-010 / RG-010:** `test_BC_2_16_019_claroty_server_interfaces_tier1_columns_two_with_ocsf_field`
  PASS. Exactly 2 columns have `ocsf_field == Some(_)`: `server_name`→`"device.name"` (REQUIRED),
  `interface_status`→`"status_code"`. Exactly 8 columns have `ocsf_field == None` (Tier-2,
  including composite PK element `interface_name`). Traces to BC-2.16.019 §PC2.

### AC-003-004-011-012-plan-gate-tests

**Covers:** AC-003 (RG-003), AC-004 (RG-004), AC-011 (RG-011), AC-012 (RG-012)

- `AC-003-004-011-012-plan-gate-tests.tape` — VHS script source
- `AC-003-004-011-012-plan-gate-tests.gif` — PR-embeddable recording
- `AC-003-004-011-012-plan-gate-tests.webm` — archival recording

**What it proves:**

- **AC-003 / RG-003:** `test_BC_2_16_018_claroty_servers_e2e_e_query_038_tier2_column` PASS
  (authoritative prism-bin E2E gate via `QueryEngine::execute`). E-QUERY-038 raised for Tier-2
  column query; `available_columns` excludes Tier-2 name; includes `raw_extensions`, `device_name`,
  `status_code`, `class_uid`, `_sensor`. Traces to BC-2.16.018 §Invariants, EC-016-018-005.

- **AC-004 / RG-004:** `test_BC_2_16_018_claroty_servers_tier1_raw_toml_name_raises_e_query_038`
  PASS. `SELECT server_status` raises E-QUERY-038; available has `status_code` not `server_status`.
  `SELECT server_name` raises E-QUERY-038; available has `device_name` not `server_name`.
  Traces to BC-2.16.018 §Invariants.

- **AC-011 / RG-011:** `test_BC_2_16_019_claroty_server_interfaces_e2e_e_query_038_tier2_column`
  PASS (authoritative prism-bin E2E gate). E-QUERY-038 raised for `SELECT interface_name`;
  composite PK element is Tier-2 in raw_extensions, not a standalone Arrow column.
  `available_columns` excludes `interface_name`; includes `raw_extensions`, `device_name`,
  `status_code`, `class_uid`, `_sensor`. Traces to BC-2.16.019 §Invariants, EC-016-019-004.

- **AC-012 / RG-012:** `test_BC_2_16_019_claroty_server_interfaces_interface_status_raw_name_raises_e_query_038`
  PASS. `SELECT interface_status` raises E-QUERY-038; available has `status_code` not `interface_status`.
  Traces to BC-2.16.019 §Invariants, EC-016-019-006.

### AC-005-013-wire-shape-mock

**Covers:** AC-005 wire shape mock companion (RG-017), AC-013 wire shape mock companion (RG-020)

- `AC-005-013-wire-shape-mock.tape` — VHS script source
- `AC-005-013-wire-shape-mock.gif` — PR-embeddable recording
- `AC-005-013-wire-shape-mock.webm` — archival recording

**What it proves:**

- **AC-005 wire shape / RG-017:** `test_BC_2_16_018_claroty_servers_wire_shape_class_uid_5001_mock`
  PASS (SAP-4 production path via `SpecDrivenSensorAdapter::fetch → pipeline_result_to_record_batch`).
  Mock response with seeded claroty_servers row. Serialized JSON wire output contains `class_uid=5001`,
  `device_name` present, `status_code` present, `raw_extensions` present. No standalone Tier-2 root
  keys. Traces to BC-2.16.018 §PC1 (class_uid), §PC2.

- **AC-013 wire shape / RG-020:** `test_BC_2_16_019_claroty_server_interfaces_wire_shape_class_uid_5001_mock`
  PASS (SAP-4 production path). Mock response with seeded claroty_server_interfaces row. Serialized
  JSON wire output contains `class_uid=5001`, `device_name` present, `status_code` present,
  `raw_extensions` present with `interface_name` and `interface_type` keys (composite PK join keys
  accessible via raw_extensions). `_source_table="claroty_server_interfaces"` (SEPARATE endpoint
  confirmed by distinct source table). Traces to BC-2.16.019 §PC1, §PC2, §PC3.

### AC-007-008-015-016-null-passthrough

**Covers:** AC-007 (RG-007), AC-008 (RG-008), AC-015 (RG-015a, RG-015b), AC-016 (RG-016)

- `AC-007-008-015-016-null-passthrough.tape` — VHS script source
- `AC-007-008-015-016-null-passthrough.gif` — PR-embeddable recording
- `AC-007-008-015-016-null-passthrough.webm` — archival recording

**What it proves:**

- **AC-007 / RG-007:** `test_BC_2_16_018_claroty_servers_required_server_name_absent_produces_null_row`
  PASS. Row missing `server_name` → null row (REQUIRED semantics); no hard error; subsequent rows
  continue. Traces to BC-2.16.018 §Invariants, EC-016-018-001.

- **AC-008 / RG-008:** `test_BC_2_16_018_claroty_servers_nullable_count_uses_empty_page_halt`
  PASS. `count=null` in servers envelope → empty-page halt; no null-ptr dereference; no error.
  Traces to BC-2.16.018 §PC1 pagination note, EC-016-018-004.

- **AC-015 / RG-015a:** `test_BC_2_16_019_claroty_server_interfaces_required_server_name_absent_produces_null_row`
  PASS. Row missing `server_name` → null row; no hard error; pagination continues.
  Traces to BC-2.16.019 §Invariants, EC-016-019-001.

- **AC-015 / RG-015b:** `test_BC_2_16_019_claroty_server_interfaces_null_interface_name_row_not_dropped`
  PASS. Row with null `interface_name` but non-null `server_name` → row materialized (NOT dropped);
  `device_name` resolves to non-null Arrow cell; `interface_name` null inside `raw_extensions`.
  Composite PK degraded (server identified, interface lost) but row valid.
  Traces to BC-2.16.019 §Invariants, EC-016-019-002.

- **AC-016 / RG-016:** `test_BC_2_16_019_claroty_server_interfaces_nullable_count_uses_empty_page_halt`
  PASS. `count=null` in server_interfaces envelope → empty-page halt; no error.
  Traces to BC-2.16.019 §PC1 pagination note, EC-016-019-005.

---

## BC Traceability

| Evidence Artifact | AC(s) | BC | EC |
|-------------------|----|----|----|
| AC-001-002-009-010-schema-describe.txt | AC-001, AC-002 | BC-2.16.018 §1, §2 | — |
| AC-001-002-009-010-schema-describe.txt | AC-009, AC-010 | BC-2.16.019 §1, §2 | — |
| AC-001-002-009-010-toml-parse (.gif/.webm) | AC-001 (RG-001) | BC-2.16.018 §1 | — |
| AC-001-002-009-010-toml-parse (.gif/.webm) | AC-002 (RG-002) | BC-2.16.018 §2 | — |
| AC-001-002-009-010-toml-parse (.gif/.webm) | AC-009 (RG-009) | BC-2.16.019 §1 | — |
| AC-001-002-009-010-toml-parse (.gif/.webm) | AC-010 (RG-010) | BC-2.16.019 §2 | — |
| AC-003-004-011-012-error-paths.txt | AC-003, AC-004 | BC-2.16.018 §Invariants | EC-016-018-005 |
| AC-003-004-011-012-error-paths.txt | AC-011, AC-012 | BC-2.16.019 §Invariants | EC-016-019-004, EC-016-019-006 |
| AC-003-004-011-012-plan-gate-tests (.gif/.webm) | AC-003 (RG-003) | BC-2.16.018 §Invariants | EC-016-018-005 |
| AC-003-004-011-012-plan-gate-tests (.gif/.webm) | AC-004 (RG-004) | BC-2.16.018 §Invariants | — |
| AC-003-004-011-012-plan-gate-tests (.gif/.webm) | AC-011 (RG-011) | BC-2.16.019 §Invariants | EC-016-019-004 |
| AC-003-004-011-012-plan-gate-tests (.gif/.webm) | AC-012 (RG-012) | BC-2.16.019 §Invariants | EC-016-019-006 |
| AC-005-006-013-014-live-queries.txt | AC-005 (live) | BC-2.16.018 §1, §2 | — |
| AC-005-006-013-014-live-queries.txt | AC-006 (live) | BC-2.16.018 §2 | EC-016-018-003 (uptime_days float) |
| AC-005-006-013-014-live-queries.txt | AC-013 (live) | BC-2.16.019 §1, §2, §3 | — |
| AC-005-006-013-014-live-queries.txt | AC-014 (live) | BC-2.16.019 §2 | — |
| AC-005-013-wire-shape-mock (.gif/.webm) | AC-005 (RG-017) | BC-2.16.018 §1, §2 | — |
| AC-005-013-wire-shape-mock (.gif/.webm) | AC-013 (RG-020) | BC-2.16.019 §1, §2, §3 | — |
| AC-007-008-015-016-null-passthrough (.gif/.webm) | AC-007 (RG-007) | BC-2.16.018 §Invariants | EC-016-018-001 |
| AC-007-008-015-016-null-passthrough (.gif/.webm) | AC-008 (RG-008) | BC-2.16.018 §PC1 | EC-016-018-004 |
| AC-007-008-015-016-null-passthrough (.gif/.webm) | AC-015 (RG-015a, RG-015b) | BC-2.16.019 §Invariants | EC-016-019-001, EC-016-019-002 |
| AC-007-008-015-016-null-passthrough (.gif/.webm) | AC-016 (RG-016) | BC-2.16.019 §PC1 | EC-016-019-005 |
