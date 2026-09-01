---
document_type: behavioral-contract
level: L3
version: "1.3"
status: active
producer: product-owner
timestamp: 2026-08-24T00:00:00Z
phase: 3
origin: brownfield
subsystem: "SS-16"
capability: "CAP-029"
lifecycle_status: active
inputs:
  - ".factory/objectives/xdome-endpoint-expansion-plan.md"
  - ".factory/objectives/xdome-v1-validation/endpoint-spike-findings.md"
  - ".factory/objectives/xdome-v1-validation/endpoint-schema-extract.md"
  - ".factory/specs/domain-spec/capabilities.md"
  - ".factory/specs/architecture/decisions/ADR-058-v1-column-naming-col-name-as-arrow-field-identifier.md"
  - "crates/prism-sensors/specs/claroty.sensor.toml"
input-hash: "150e0ec"
traces_to: ["CAP-029"]
extracted_from: ".factory/reference/api-specs/xdome_openapi_06.20.2026.json"
introduced: "2026-08-24"
modified: "2026-08-31"
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.16.018: Claroty xDome Collection Servers Table — Queryable Surface and OCSF inventory_info Mapping (No DTU)

## Description

The `claroty_servers` TOML table block in `claroty.sensor.toml` exposes Claroty xDome collection
server appliance records as a queryable PrismQL table. The table follows the standard Claroty
POST-for-read pattern with offset/limit pagination against `POST /api/v1/servers/` (envelope
`{count, servers}`), using `inventory_info` (class_uid 5001, existing arm) as its OCSF class.
Under `ocsf_column_naming = true`, `server_name` maps to the Tier-1 OCSF column `device.name`
(Arrow `device_name`, REQUIRED) and `server_status` maps to `status_code` (Arrow `status_code`);
the remaining 15 scalar columns are Tier-2 aggregated into `raw_extensions`. No DTU exists for
this endpoint; near-term tests run against the live monroe sensor only.

## Preconditions

- `claroty.sensor.toml` includes the `claroty_servers` [[tables]] block as specified in
  S-CLAROTY-SERVERS-001
- `ocsf_column_naming = true` is declared at the sensor level in `claroty.sensor.toml`
- The `inventory_info` / class_uid 5001 arm exists in
  `prism-ocsf/src/class_selector.rs::select_by_class_name` (existing arm — no new arm required
  per spike findings §Overall Verdict)
- The Claroty bearer token credential is configured for the requesting client
- S-PLUGIN-PREREQ-A through S-PLUGIN-PREREQ-E have all merged (spec-engine pipeline active)

## Postconditions

### 1. TOML Table Contract

The `claroty_servers` table MUST be declared in `claroty.sensor.toml` with:

```toml
[[tables]]
table_name = "servers"  # bare name; TableRegistry derives the registered/queryable name as {sensor_id}_{table_name} = "claroty_servers"
ocsf_class = "inventory_info"   # class_uid 5001 (existing arm)
```

**Step definition:**

```toml
[[tables.steps]]
name = "fetch_servers"
method = "POST"
path_template = "/api/v1/servers/"
body_template = '{"fields": ["server_name", "server_location", "server_status", "site_id", "model", "os_version", "serial_number", "num_of_interfaces", "management_ip", "idrac_ip", "management_mac", "uptime_days", "avg_traffic_past_month_mbps", "avg_traffic_past_week_mbps", "avg_traffic_past_hour_mbps", "num_of_open_incidents", "notes"]}'
response_path = "$.servers"
variables_produced = []
[tables.steps.pagination]
type = "offset_limit"
page_size = 1000
```

**Pagination note:** The `servers` envelope carries a `count` field per the schema extract. If
`count` is null or absent, pagination halts via empty-page check (EC-016-018-004).

### 2. Column Tier Classification (ADR-058)

Under `ocsf_column_naming = true`, columns are classified as follows:

**Tier-1 columns** (have `ocsf_field`; exposed as Arrow field name =
`ocsf_field_to_arrow_name(ocsf_field)`):

| Column (TOML name) | ColumnType | ocsf_field | Arrow Field Name | Options |
|--------------------|-----------|------------|-----------------|---------|
| `server_name` | String | `device.name` | `device_name` | REQUIRED |
| `server_status` | String | `status_code` | `status_code` | — |

**Tier-2 columns** (no `ocsf_field`; values aggregate into `raw_extensions` JSON object):

| Column (TOML name) | ColumnType | Notes |
|--------------------|-----------|-------|
| `server_location` | String | Physical location of the collection server appliance |
| `site_id` | Integer | Unique site identifier; numeric comparison operators supported |
| `model` | String | Server model string, e.g. "MCS R340" or "R640" |
| `os_version` | String | Ubuntu OS version string |
| `serial_number` | String | Server serial number |
| `num_of_interfaces` | Integer | Count of network interfaces on the server |
| `management_ip` | String | Data/Management port IP address; supports in_subnet operations at API level |
| `idrac_ip` | String | Integrated Dell Remote Access Controller IP address |
| `management_mac` | String | Data/Management port MAC address |
| `uptime_days` | Float | Days the server has been up; confirmed fractional from xDome OpenAPI example (e.g., 667.233661); Float type resolved |
| `avg_traffic_past_month_mbps` | Float | Average traffic volume to the server in the past month (Mbps) |
| `avg_traffic_past_week_mbps` | Float | Average traffic volume to the server in the past week (Mbps) |
| `avg_traffic_past_hour_mbps` | Float | Average traffic volume to the server in the past hour (Mbps) |
| `num_of_open_incidents` | Integer | Count of open incidents associated with the server |
| `notes` | String | Free-text analyst notes about the server |

**Total declared columns:** 17 (2 Tier-1, 15 Tier-2). All 17 fields are from the Server
`fields_enum` confirmed in the schema extract §Server (field count: 17).

### 3. Primary Key and OCSF Mapping Rationale

**Primary key: `server_name` (String, REQUIRED, single-column)**

`server_name` is the collection server's deployment-time name ("collection server name that is set
during deployment" per the OpenAPI field description). It uniquely identifies each Claroty
collection server appliance. No opaque internal ID field was identified in the Server fields_enum —
`server_name` is the canonical identifier.

**OCSF Tier-1 mapping rationale:**

- `server_name` → `device.name` (Arrow: `device_name`): OCSF `inventory_info` class (5001)
  represents a device inventory record. The `device` object is a required attribute of
  `inventory_info`, and `device.name` is the human-readable device identifier. A Claroty collection
  server appliance is precisely the kind of managed device described by `inventory_info`, and its
  deployment name maps to `device.name`.

- `server_status` (values: "Up", "Down", "Pending") → `status_code` (Arrow: `status_code`): OCSF
  `inventory_info` has `status_code` as a class-level attribute for the operational status of the
  managed device. This is the same pattern used in the existing `claroty_devices` table where
  `retired → status_code`. The server's up/down/pending status is the closest semantic match.

**All 15 remaining scalar columns are Tier-2** — none have a direct, semantically correct OCSF
`inventory_info` field equivalent that justifies a Tier-1 mapping. Traffic metrics, interface
counts, IP/MAC addresses, hardware model, OS version, and serial number all aggregate cleanly into
`raw_extensions` where they remain queryable via JSON extraction.

### 4. SAP-2 DTU Parity Status

SAP-2 probe is **N/A** for this table (no DTU exists for `/api/v1/servers/` per
xdome-endpoint-expansion-plan.md §Governing Directive and §Deferred DTU-Creation Stories).
The deferred DTU creation story is tracked as D-2200. Once the DTU story for `claroty_servers`
executes, SAP-2 probe applies retroactively and this BC MUST be amended with:
- DTU route file references (`crates/prism-dtu-claroty/src/routes/servers.rs`)
- DTU types.rs field equivalencies for all 17 contracted columns
- SAP-2 exclusion documentation if any fields are explicitly excluded

Until the DTU story executes, near-term tests run against the live monroe sensor only.

## Invariants

- DI-005: OCSF schema validity — `inventory_info` class_uid 5001 is a valid OCSF class
- `server_name` (String, REQUIRED) is the primary key; absent `server_name` produces a null row per
  spec-engine REQUIRED semantics (not a hard error; subsequent rows continue)
- Tier-2 columns are NOT exposed as standalone Arrow columns; a PrismQL query referencing them by
  raw TOML name (e.g., `WHERE server_location = 'x'`) MUST raise E-QUERY-038 with
  `available_columns` containing `raw_extensions`, `device_name`, `status_code`, `class_uid`,
  `_sensor` but NOT the raw Tier-2 column name
- All 17 columns are scalar (String, Integer, or Float) — no Json columns exist in this table;
  no array or object fields in the Server fields_enum require Json type
- No Datetime columns exist — `timestamp_formats` is not required and E-SPEC-018 is not applicable
  to this table

## Error Cases

| Error | Condition | Behavior |
|-------|-----------|----------|
| `E-SENSOR-001` | Claroty API returns non-200 HTTP for POST /api/v1/servers/ | Structured error with sensor=claroty, status, body; no data loss for previously fetched pages |
| `E-QUERY-038` | Query references `server_location`, `model`, `management_ip` or any other Tier-2 column by its raw TOML name | Column-not-found at plan time; `available_columns` contains `raw_extensions`, `device_name`, `status_code`, `class_uid`, `_sensor` |

No new error codes are required for this table. All failure modes are covered by existing codes.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-016-018-001 | Row missing `server_name` field (REQUIRED) | Null row produced; no hard error; subsequent rows continue |
| EC-016-018-002 | `server_status` is null or absent | Null `status_code` Arrow cell; not an error |
| EC-016-018-003 | `uptime_days` returns fractional value (e.g., 1.5) | Float cell stored in `raw_extensions.uptime_days`; spec-engine Float type handles fractional days correctly |
| EC-016-018-004 | `count` is null or absent in the response envelope | Pagination continues via empty-page check (empty page → halt); not an error; same as EC-016-015-003 in BC-2.16.015 |
| EC-016-018-005 | Query references Tier-2 column `management_ip` by raw name | E-QUERY-038; `available_columns` contains `raw_extensions`, `device_name`, `status_code` but NOT `management_ip` |
| EC-016-018-006 | `server_name` values contain spaces (e.g., "Monroe Collector 1") | Preserved as-is in `device_name` Arrow column; no normalization |

## Related BCs

- BC-2.16.013: Bundled Sensor Spec Authoring — parent spec for the Claroty sensor; this BC adds
  the `claroty_servers` table to the Claroty sensor surface (depends on)
- BC-2.16.003: Column-to-OCSF Mapping at Query Time — `inventory_info` class_uid 5001 is the
  same OCSF class used by the existing `claroty_devices` table; `status_code` Tier-1 mapping
  follows the same pattern (composes with)
- BC-2.01.007: Claroty Bearer Token Auth — auth mechanism unchanged; preconditions satisfied
  (depends on)
- BC-2.16.019: Claroty xDome Server Interfaces Table — companion table at a separate endpoint
  (`/api/v1/server_interfaces/`); both share `inventory_info/5001` and are delivered in the
  same story S-CLAROTY-SERVERS-001 (sibling)

## Architecture Anchors

- `crates/prism-sensors/specs/claroty.sensor.toml` — TOML spec file authoring target
- `crates/prism-spec-engine/src/spec_parser.rs` — ColumnSpec, FetchStep deserialization
- `crates/prism-spec-engine/src/pipeline.rs` — OffsetLimit POST-body injection
- `crates/prism-ocsf/src/class_selector.rs::select_by_class_name` — `"inventory_info"` arm
  (existing; resolves to class_uid 5001 per ADR-058 §I5(b) KF-02 fix)
- `crates/prism-bin/src/spec_driven_adapter.rs` — `pipeline_result_to_record_batch`
- `.factory/reference/api-specs/xdome_openapi_06.20.2026.json §/api/v1/servers/` — endpoint
  authority (confirmed path `/api/v1/servers/`; Server fields_enum 17 fields)
- `.factory/objectives/xdome-endpoint-expansion-plan.md §Gap Table G4` — table scope authority

## Story Anchor

S-CLAROTY-SERVERS-001 (draft — Wave C)

## VP Anchors

(none — no formal verification properties defined; structural tests via story RG list per
S-CLAROTY-SERVERS-001; holdout evaluator exercises live monroe surface via HS-027)

## Canonical Test Vectors

| Test Vector ID | Description | Expected |
|----------------|-------------|----------|
| TV-BC-2.16.018-001 | `SELECT device_name FROM claroty.claroty_servers LIMIT 5` against live monroe | Succeeds (no E-QUERY-038); rows have non-null `device_name` strings (collection server names) |
| TV-BC-2.16.018-002 | `SELECT * FROM claroty.claroty_servers LIMIT 1` | Wire JSON contains `class_uid = 5001`; `device_name` present; `status_code` present; `raw_extensions` object present with `server_location`, `model`, `management_ip` keys |
| TV-BC-2.16.018-003 | `SELECT server_name FROM claroty.claroty_servers LIMIT 1` | E-QUERY-038; `available_columns` contains `device_name`, `status_code`, `raw_extensions`; does NOT contain `server_name` |
| TV-BC-2.16.018-004 | `SELECT management_ip FROM claroty.claroty_servers LIMIT 1` | E-QUERY-038; `available_columns` contains `raw_extensions` but NOT `management_ip` |
| TV-BC-2.16.018-005 | `SELECT raw_extensions FROM claroty.claroty_servers LIMIT 5` | Succeeds; `raw_extensions` JSON contains `management_ip`, `model`, `os_version` keys |
| TV-BC-2.16.018-006 | Response envelope with null `count` field | Pagination terminates on empty page; no error |

## Verification Properties

| VP | Verification Aspect |
|----|---------------------|
| (none) | No VP directly verifies this BC — structural tests cover via story RG list per S-CLAROTY-SERVERS-001; holdout evaluator exercises live monroe surface |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-029 |
| Capability Anchor Justification | CAP-029 ("Config-Driven Sensor Adapters") per capabilities.md §CAP-029 — this BC specifies the TOML table contract for the Claroty xDome `claroty_servers` table, defining 17 columns (typed with ColumnOptions and OCSF mappings), multi-step fetch pipeline (POST-for-read, offset_limit pagination, `servers` envelope key), Tier-1/Tier-2 OCSF column classification per ADR-058 (2 Tier-1: device_name REQUIRED + status_code; 15 Tier-2 into raw_extensions), PK rationale (server_name → device.name), and SAP-2 N/A documentation (no DTU; D-2200 deferred DTU anchor). This is exactly what CAP-029 defines: sensor adapters defined in TOML spec files with tables, columns, pipelines, and pagination config. |
| L2 Invariants | DI-005 |
| Priority | P0 |
| Story | S-CLAROTY-SERVERS-001 |
| DTU Status | NONE — no DTU exists; near-term tests against live monroe sensor only; DTU deferred to D-2200 |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.3 | g4-obs2-body-template-single-line | 2026-08-31 | product-owner | OBS-2: §Postconditions §1 `body_template` re-rendered as valid single-line TOML literal string; prior multi-line backslash-continuation form is invalid in TOML literal (single-quoted) strings. No semantic change — all 17 fields unchanged. |
| 1.2 | g4-adversary-low2-uptime-caution | 2026-08-31 | product-owner | LOW-2: §Postconditions §2 Tier-2 table `uptime_days` row — removed stale "verify exact type on live monroe sensor before asserting Integer" caution; Float type confirmed fractional from xDome OpenAPI example (e.g., 667.233661) and propagated to TOML, story §risk, Notes, and EC-016-018-003. Row now states Float as resolved with OpenAPI-example confirmation. |
| 1.1 | g3-g4-g5-spec-prose-corrections | 2026-08-31 | product-owner | MED-1: §Postconditions §1 TOML bare table_name corrected from `"claroty_servers"` to `"servers"`; added derivation note (`{sensor_id}_{table_name}` = registered/queryable name `"claroty_servers"`). Architecture anchor: §Architecture Anchors `spec_driven_adapter.rs` crate corrected `crates/prism-spec-engine` → `crates/prism-bin` (ground truth: `pipeline_result_to_record_batch` lives in `crates/prism-bin/src/spec_driven_adapter.rs`). FIX 2 not applicable — no `ColumnMapper::map_record` attribution present. |
| 1.0 | xdome-wave-c-f2-spec-evolution | 2026-08-24 | product-owner | Initial authoring — Claroty xDome servers queryable surface contract per xdome-endpoint-expansion-plan.md Wave C G4. TOML table contract, 17-column Tier-1/Tier-2 classification per ADR-058 (2 Tier-1: device_name REQUIRED [server_name→device.name] + status_code [server_status→status_code]; 15 Tier-2 into raw_extensions). PK: server_name (String, REQUIRED, single-column). OCSF class: inventory_info/5001 (existing arm). No new error codes. SAP-2 N/A (no DTU; D-2200 deferred DTU anchor). Endpoint path `/api/v1/servers/` confirmed from OpenAPI spec. All 17 fields from Server fields_enum confirmed in schema-extract §Server. HS-027 holdout group registered with 3 P0 scenarios for S-CLAROTY-SERVERS-001. |
