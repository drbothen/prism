---
document_type: behavioral-contract
level: L3
version: "1.2"
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

# BC-2.16.019: Claroty xDome Server Interfaces Table — Queryable Surface, Composite PK, and OCSF inventory_info Mapping (No DTU)

## Description

The `claroty_server_interfaces` TOML table block in `claroty.sensor.toml` exposes Claroty xDome
collection server network interface records as a queryable PrismQL table. **Endpoint correction
from initial plan:** the server interfaces are NOT a sub-resource of `/api/v1/servers/` — they
are exposed at a **separate** endpoint `POST /api/v1/server_interfaces/` (operationId
`get_servers_api_v1_server_interfaces__post`, confirmed from the xDome OpenAPI spec). The table
uses `inventory_info` (class_uid 5001, existing arm — same as `claroty_servers`) with a composite
primary key (`server_name`, `interface_name`). Under `ocsf_column_naming = true`, `server_name`
maps to the Tier-1 OCSF column `device.name` (Arrow `device_name`, REQUIRED) and `interface_status`
maps to `status_code` (Arrow `status_code`); the remaining 8 scalar columns are Tier-2 aggregated
into `raw_extensions`. No DTU exists for this endpoint; near-term tests run against the live monroe
sensor only.

## Preconditions

- `claroty.sensor.toml` includes the `claroty_server_interfaces` [[tables]] block as specified in
  S-CLAROTY-SERVERS-001
- `ocsf_column_naming = true` is declared at the sensor level in `claroty.sensor.toml`
- The `inventory_info` / class_uid 5001 arm exists in
  `prism-ocsf/src/class_selector.rs::select_by_class_name` (existing arm — same arm used by
  `claroty_servers` and `claroty_devices`; no new arm required per spike findings §Overall Verdict)
- The Claroty bearer token credential is configured for the requesting client
- S-PLUGIN-PREREQ-A through S-PLUGIN-PREREQ-E have all merged (spec-engine pipeline active)

## Postconditions

### 1. TOML Table Contract

The `claroty_server_interfaces` table MUST be declared in `claroty.sensor.toml` with:

```toml
[[tables]]
table_name = "server_interfaces"  # bare name; TableRegistry derives the registered/queryable name as {sensor_id}_{table_name} = "claroty_server_interfaces"
ocsf_class = "inventory_info"   # class_uid 5001 (existing arm; same as claroty_servers)
```

**Step definition:**

```toml
[[tables.steps]]
name = "fetch_server_interfaces"
method = "POST"
path_template = "/api/v1/server_interfaces/"
body_template = '{"fields": ["server_name", "interface_name", "interface_status", "interface_type", "interface_connection_type", "site_id", "avg_traffic_past_month_mbps", "avg_traffic_past_week_mbps", "avg_traffic_past_hour_mbps", "notes"]}'
response_path = "$.server_interfaces"
variables_produced = []
[tables.steps.pagination]
type = "offset_limit"
page_size = 1000
```

**Endpoint correction:** The xDome OpenAPI spec defines a SEPARATE endpoint `/api/v1/server_interfaces/`
(operationId: `get_servers_api_v1_server_interfaces__post`). The initial plan description of this
as "sub-table via the servers endpoint" was incorrect; the endpoint is independent. This is confirmed
by the targeted `jq` query against the OpenAPI spec which shows `"/api/v1/server_interfaces/"` as
a distinct path with its own POST handler, request schema (`GetServerInterfacesParameters`), and
response schema (`GetServerInterfacesResponse` with envelope `{count, server_interfaces}`).

**Pagination note:** The `server_interfaces` response envelope carries a `count` field per the
OpenAPI spec. If `count` is null or absent, pagination halts via empty-page check (EC-016-019-005).

### 2. Column Tier Classification (ADR-058)

Under `ocsf_column_naming = true`, columns are classified as follows:

**Tier-1 columns** (have `ocsf_field`; exposed as Arrow field name =
`ocsf_field_to_arrow_name(ocsf_field)`):

| Column (TOML name) | ColumnType | ocsf_field | Arrow Field Name | Options |
|--------------------|-----------|------------|-----------------|---------|
| `server_name` | String | `device.name` | `device_name` | REQUIRED |
| `interface_status` | String | `status_code` | `status_code` | — |

**Tier-2 columns** (no `ocsf_field`; values aggregate into `raw_extensions` JSON object):

| Column (TOML name) | ColumnType | Notes |
|--------------------|-----------|-------|
| `interface_name` | String | Composite PK element; name of the network interface (e.g., "eth0", "ens3") |
| `interface_type` | String | Interface type: "SPAN" or "Management" |
| `interface_connection_type` | String | Physical connection type: "SFP+" or "RJ45 (Copper)" |
| `site_id` | Integer | Unique site identifier for the site to which the interface belongs |
| `avg_traffic_past_month_mbps` | Float | Average traffic volume to the server via this interface in the past month (Mbps) |
| `avg_traffic_past_week_mbps` | Float | Average traffic volume via this interface in the past week (Mbps) |
| `avg_traffic_past_hour_mbps` | Float | Average traffic volume via this interface in the past hour (Mbps) |
| `notes` | String | Free-text notes about the interface |

**Total declared columns:** 10 (2 Tier-1, 8 Tier-2). All 10 fields are from the ServerInterfaces
`fields_enum` confirmed in the schema extract §ServerInterfaces (field count: 10).

### 3. Primary Key, Composite Identity, and OCSF Mapping Rationale

**Primary key: COMPOSITE (`server_name`, `interface_name`)**

No single column uniquely identifies a server interface row. Each row represents a specific network
interface on a specific collection server:

1. `server_name` identifies the server appliance (same PK field as `claroty_servers`).
2. `interface_name` identifies the interface on that server.
3. The composite (`server_name`, `interface_name`) is the unique identifier for a server interface
   record — the semantic join key for cross-table operations (e.g., JOIN with `claroty_servers`
   ON `server_name`).

`server_name` carries `REQUIRED` because it is the Tier-1 column (Arrow `device_name`). The
`interface_name` is Tier-2 (in `raw_extensions`) but is also a composite PK element; a row with
a null `interface_name` is degraded (server identified, interface lost) but not dropped.

**OCSF Tier-1 mapping rationale:**

- `server_name` → `device.name` (Arrow: `device_name`): The owning collection server is the
  primary device being inventoried. `device.name` in OCSF `inventory_info` represents the
  human-readable device identifier. Each interface row represents the inventory state of a
  specific interface on that named device. This is the same mapping as BC-2.16.018 §PC3 for
  `claroty_servers` — consistent semantics.

- `interface_status` (values: "Up", "No Carrier") → `status_code` (Arrow: `status_code`): OCSF
  `inventory_info` has `status_code` at the class level for the operational status of the managed
  entity. The interface availability status ("Up" / "No Carrier") is the operational state of the
  inventory entity being described in this row. This follows the same class-level `status_code`
  pattern as `claroty_devices` (`retired → status_code`) and `claroty_servers`
  (`server_status → status_code`).

**All 8 remaining scalar columns are Tier-2.** `interface_name` has no direct OCSF `inventory_info`
top-level mapping — network interface names are nested under `device.network_interfaces[]` as an
array element in OCSF, not a scalar `inventory_info` field. Mapping it as `device.name` would
conflict with `server_name`. All traffic metrics, connection type, interface type, and site_id
aggregate cleanly into `raw_extensions`.

### 4. SAP-2 DTU Parity Status

SAP-2 probe is **N/A** for this table (no DTU exists for `/api/v1/server_interfaces/` per
xdome-endpoint-expansion-plan.md §Governing Directive and §Deferred DTU-Creation Stories).
The deferred DTU creation story is tracked as D-2200. Once the DTU story for
`claroty_server_interfaces` executes, SAP-2 probe applies retroactively and this BC MUST be
amended with:
- DTU route file references (`crates/prism-dtu-claroty/src/routes/server_interfaces.rs`)
- DTU types.rs field equivalencies for all 10 contracted columns
- SAP-2 exclusion documentation if any fields are explicitly excluded

Until the DTU story executes, near-term tests run against the live monroe sensor only.

## Invariants

- DI-005: OCSF schema validity — `inventory_info` class_uid 5001 is a valid OCSF class
- `server_name` (String, REQUIRED) is the composite PK anchor; absent `server_name` produces a
  null row per spec-engine REQUIRED semantics (not a hard error; subsequent rows continue)
- `interface_name` and `server_name` together form the composite semantic PK; `interface_name` is
  Tier-2 (in `raw_extensions`) and is NOT declared REQUIRED in the TOML spec; a row with null
  `interface_name` but non-null `server_name` is valid (degraded, not dropped)
- Tier-2 columns are NOT exposed as standalone Arrow columns; a PrismQL query referencing them by
  raw TOML name MUST raise E-QUERY-038 with `available_columns` containing `raw_extensions`,
  `device_name`, `status_code`, `class_uid`, `_sensor` but NOT the raw Tier-2 column name
- `interface_status` raw column name is rejected at plan time — `SELECT interface_status` raises
  E-QUERY-038; the OCSF Arrow column name `status_code` is accepted
- All 10 columns are scalar (String, Integer, or Float) — no Json columns; no array or object
  fields in the ServerInterfaces fields_enum require Json type
- No Datetime columns exist — `timestamp_formats` is not required and E-SPEC-018 is not applicable

## Error Cases

| Error | Condition | Behavior |
|-------|-----------|----------|
| `E-SENSOR-001` | Claroty API returns non-200 HTTP for POST /api/v1/server_interfaces/ | Structured error with sensor=claroty, status, body; no data loss for previously fetched pages |
| `E-QUERY-038` | Query references `interface_name`, `interface_type`, `avg_traffic_past_month_mbps`, or any Tier-2 column by raw TOML name | Column-not-found at plan time; `available_columns` contains `raw_extensions`, `device_name`, `status_code`, `class_uid`, `_sensor` |

No new error codes are required for this table. Endpoint path independence from `/api/v1/servers/`
is a TOML spec-authoring constraint (use correct `path_template = "/api/v1/server_interfaces/"`),
not a new runtime error mode.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-016-019-001 | Row missing `server_name` field (REQUIRED) | Null row produced; no hard error; subsequent rows continue |
| EC-016-019-002 | `interface_name` is null or absent (composite PK degraded) | `server_name` still resolves to non-null `device_name`; `interface_name` is null in `raw_extensions`; row is degraded but not dropped |
| EC-016-019-003 | `interface_status` is null or absent | Null `status_code` Arrow cell; not an error |
| EC-016-019-004 | Query references Tier-2 column `interface_name` by raw name | E-QUERY-038; `available_columns` contains `raw_extensions`, `device_name`, `status_code` but NOT `interface_name` |
| EC-016-019-005 | `count` is null or absent in the response envelope | Pagination continues via empty-page check (empty page → halt); not an error |
| EC-016-019-006 | `SELECT interface_status` (raw col.name) attempted | E-QUERY-038; `available_columns` includes `status_code` but NOT `interface_status`; the Tier-1 rename is enforced |

## Related BCs

- BC-2.16.013: Bundled Sensor Spec Authoring — parent spec for the Claroty sensor; this BC adds
  the `claroty_server_interfaces` table to the Claroty sensor surface (depends on)
- BC-2.16.018: Claroty xDome Collection Servers Table — companion table; both use
  `inventory_info/5001`, share `server_name → device.name` Tier-1 mapping, and are delivered
  in the same story S-CLAROTY-SERVERS-001; JOIN via `server_name = device_name` (sibling)
- BC-2.16.003: Column-to-OCSF Mapping at Query Time — `inventory_info` class_uid 5001 and
  `status_code` Tier-1 mapping; same class-level patterns (composes with)
- BC-2.01.007: Claroty Bearer Token Auth — auth mechanism unchanged; preconditions satisfied
  (depends on)

## Architecture Anchors

- `crates/prism-sensors/specs/claroty.sensor.toml` — TOML spec file authoring target
- `crates/prism-spec-engine/src/spec_parser.rs` — ColumnSpec, FetchStep deserialization
- `crates/prism-spec-engine/src/pipeline.rs` — OffsetLimit POST-body injection
- `crates/prism-ocsf/src/class_selector.rs::select_by_class_name` — `"inventory_info"` arm
  (existing; same arm as `claroty_servers` and `claroty_devices`)
- `crates/prism-bin/src/spec_driven_adapter.rs` — `pipeline_result_to_record_batch`
- `.factory/reference/api-specs/xdome_openapi_06.20.2026.json §/api/v1/server_interfaces/` —
  endpoint authority (operationId: `get_servers_api_v1_server_interfaces__post`; confirmed
  SEPARATE endpoint from `/api/v1/servers/`; ServerInterfaces fields_enum 10 fields)
- `.factory/objectives/xdome-endpoint-expansion-plan.md §Gap Table G4` — table scope authority
  (note: plan listed both tables under G4; endpoint independence confirmed from OpenAPI spec)

## Story Anchor

S-CLAROTY-SERVERS-001 (draft — Wave C)

## VP Anchors

(none — no formal verification properties defined; structural tests via story RG list per
S-CLAROTY-SERVERS-001; holdout evaluator exercises live monroe surface via HS-027)

## Canonical Test Vectors

| Test Vector ID | Description | Expected |
|----------------|-------------|----------|
| TV-BC-2.16.019-001 | `SELECT device_name FROM claroty.claroty_server_interfaces LIMIT 5` against live monroe | Succeeds (no E-QUERY-038); rows have non-null `device_name` strings (collection server names) |
| TV-BC-2.16.019-002 | `SELECT * FROM claroty.claroty_server_interfaces LIMIT 1` | Wire JSON contains `class_uid = 5001`; `device_name` present; `status_code` present (Up/No Carrier); `raw_extensions` object present with `interface_name`, `interface_type`, `interface_connection_type` keys |
| TV-BC-2.16.019-003 | `SELECT interface_status FROM claroty.claroty_server_interfaces LIMIT 1` | E-QUERY-038; `available_columns` contains `status_code`, `raw_extensions`; does NOT contain `interface_status` |
| TV-BC-2.16.019-004 | `SELECT interface_name FROM claroty.claroty_server_interfaces LIMIT 1` | E-QUERY-038; `available_columns` contains `raw_extensions` but NOT `interface_name` |
| TV-BC-2.16.019-005 | `SELECT raw_extensions FROM claroty.claroty_server_interfaces LIMIT 5` | Succeeds; `raw_extensions` JSON contains `interface_name`, `interface_type`, `interface_connection_type` keys |
| TV-BC-2.16.019-006 | Response envelope with null `count` field | Pagination terminates on empty page; no error |

## Verification Properties

| VP | Verification Aspect |
|----|---------------------|
| (none) | No VP directly verifies this BC — structural tests cover via story RG list per S-CLAROTY-SERVERS-001; holdout evaluator exercises live monroe surface |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-029 |
| Capability Anchor Justification | CAP-029 ("Config-Driven Sensor Adapters") per capabilities.md §CAP-029 — this BC specifies the TOML table contract for the Claroty xDome `claroty_server_interfaces` table, defining 10 columns (typed with ColumnOptions and OCSF mappings), multi-step fetch pipeline (POST-for-read, offset_limit pagination, `server_interfaces` envelope key at `/api/v1/server_interfaces/`), Tier-1/Tier-2 OCSF column classification per ADR-058 (2 Tier-1: device_name REQUIRED [server_name→device.name] + status_code [interface_status→status_code]; 8 Tier-2 into raw_extensions), composite PK rationale (server_name + interface_name), and SAP-2 N/A documentation (no DTU; D-2200 deferred DTU anchor). This is exactly what CAP-029 defines: sensor adapters defined in TOML spec files with tables, columns, pipelines, and pagination config. |
| L2 Invariants | DI-005 |
| Priority | P0 |
| Story | S-CLAROTY-SERVERS-001 |
| DTU Status | NONE — no DTU exists; near-term tests against live monroe sensor only; DTU deferred to D-2200 |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.2 | g4-obs2-body-template-single-line | 2026-08-31 | product-owner | OBS-2: §Postconditions §1 `body_template` re-rendered as valid single-line TOML literal string; prior multi-line backslash-continuation form is invalid in TOML literal (single-quoted) strings. No semantic change — all 10 fields unchanged. |
| 1.1 | g3-g4-g5-spec-prose-corrections | 2026-08-31 | product-owner | MED-1: §Postconditions §1 TOML bare table_name corrected from `"claroty_server_interfaces"` to `"server_interfaces"`; added derivation note (`{sensor_id}_{table_name}` = registered/queryable name `"claroty_server_interfaces"`). Architecture anchor: §Architecture Anchors `spec_driven_adapter.rs` crate corrected `crates/prism-spec-engine` → `crates/prism-bin` (ground truth: `pipeline_result_to_record_batch` lives in `crates/prism-bin/src/spec_driven_adapter.rs`). FIX 2 not applicable — no `ColumnMapper::map_record` attribution present. |
| 1.0 | xdome-wave-c-f2-spec-evolution | 2026-08-24 | product-owner | Initial authoring — Claroty xDome server interfaces queryable surface contract per xdome-endpoint-expansion-plan.md Wave C G4. TOML table contract, 10-column Tier-1/Tier-2 classification per ADR-058 (2 Tier-1: device_name REQUIRED [server_name→device.name] + status_code [interface_status→status_code]; 8 Tier-2 into raw_extensions). Composite PK: (server_name, interface_name). OCSF class: inventory_info/5001 (existing arm; same as claroty_servers). ENDPOINT CORRECTION: `/api/v1/server_interfaces/` is a SEPARATE endpoint from `/api/v1/servers/` (confirmed from OpenAPI spec; operationId: get_servers_api_v1_server_interfaces__post). No new error codes. SAP-2 N/A (no DTU; D-2200 deferred DTU anchor). All 10 fields from ServerInterfaces fields_enum confirmed in schema-extract §ServerInterfaces. HS-027 holdout group registered with 3 P0 scenarios for S-CLAROTY-SERVERS-001. |
