---
document_type: story
story_id: S-CLAROTY-SERVERS-001
title: "Claroty xDome Collection Servers + Server Interfaces Tables — TOML [[tables]] blocks, 17-column + 10-column Tier-1/Tier-2 spec, composite PK for server_interfaces, live structural tests (Wave C G4)"
level: "L4"
wave: xdome-wave-c
epic_id: E-XDOME-EXPANSION
priority: P0
status: ready
# BC status: BC-2.16.018 v1.0 draft + BC-2.16.019 v1.0 draft — pre-delivery remove-uncertainty pass complete 2026-08-31; promoted to ready (D-2385).
producer: story-writer
timestamp: "2026-08-24T00:00:00Z"
version: "1.5"
modified: "2026-08-31"
phase: 3
cycle: v1.0.0-brownfield
inputs:
  - ".factory/specs/behavioral-contracts/BC-2.16.018-claroty-servers-table.md"
  - ".factory/specs/behavioral-contracts/BC-2.16.019-claroty-server-interfaces-table.md"
  - ".factory/objectives/xdome-endpoint-expansion-plan.md"
  - ".factory/objectives/xdome-v1-validation/endpoint-schema-extract.md"
  - ".factory/specs/architecture/decisions/ADR-058-v1-column-naming-col-name-as-arrow-field-identifier.md"
  - "crates/prism-sensors/specs/claroty.sensor.toml"
input-hash: "250d51e"
# input-hash: refreshed 2026-08-31 (G4 spec-prose corrections v1.4); recomputed by validate-input-hash hook
traces_to: "BC-2.16.018"
# traces_to covers primary BC; BC-2.16.019 is the companion BC; both wired via behavioral_contracts
points: 5
estimated_days: 1
tdd_mode: strict
subsystems: [SS-01, SS-16]
# Subsystem anchor justifications (ARCH-INDEX Subsystem Registry):
#   SS-01 (Sensor Adapters) owns this story's scope because
#     `crates/prism-sensors/specs/claroty.sensor.toml` — the TOML spec file being
#     modified — lives in the prism-sensors crate, which is listed under SS-01 per
#     ARCH-INDEX. Both `claroty_servers` and `claroty_server_interfaces` [[tables]] blocks
#     are sensor-adapter configuration artifacts, exactly the surface SS-01 governs.
#   SS-16 (Spec Engine) owns this story's scope because
#     `crates/prism-spec-engine/src/spec_parser.rs` must parse both new [[tables]]
#     blocks without validation error. RG-001, RG-002, RG-009, RG-010 are spec-parser
#     unit tests that exercise SS-16's ColumnSpec and FetchStep deserialization.
#     SS-16 is the canonical owner of prism-spec-engine per ARCH-INDEX Subsystem Registry.
target_module: prism-sensors
crates_touched: [prism-sensors, prism-spec-engine, prism-bin]
# crates_touched:
#   prism-sensors: claroty.sensor.toml — two new [[tables]] blocks (servers,
#     server_interfaces)
#   prism-spec-engine: RG-001/RG-002/RG-009/RG-010 spec-parser unit tests; no production
#     code changes
#   prism-bin: authoritative RG-003/RG-011 end-to-end E-QUERY-038 gates + RG-017 wire-shape assertions
capabilities:
  - CAP-029
behavioral_contracts:
  - BC-2.16.018
  # BC-2.16.018 v1.0 — Claroty xDome Collection Servers Table: TOML table contract
  # (§Postconditions §1), 17-column Tier-1/Tier-2 classification (§Postconditions §2),
  # PK rationale (§Postconditions §3), SAP-2 N/A (§Postconditions §4),
  # EC-016-018-001..006 edge cases. ACs 001–008 trace to this BC.
  - BC-2.16.019
  # BC-2.16.019 v1.0 — Claroty xDome Server Interfaces Table: TOML table contract
  # (§Postconditions §1), 10-column Tier-1/Tier-2 classification (§Postconditions §2),
  # composite PK (server_name + interface_name) rationale (§Postconditions §3),
  # SAP-2 N/A (§Postconditions §4), EC-016-019-001..006 edge cases.
  # ACs 009–016 trace to this BC.
verification_properties: []
holdout_scenarios:
  - HS-027
# holdout_scenarios: HS-027 registered by PO at BC-2.16.018 §Changelog and
# BC-2.16.019 §Changelog (3 P0 hidden scenarios covering both tables).
# Scenarios live under the holdout-scenarios directory that test-writer and implementer
# MUST NOT read (contamination control). The story-level holdout gate (human-approved
# 2026-07-13) is BLOCKING before demo recording / push to origin.
depends_on: []
# depends_on justification: No cross-table join dependency for claroty_servers.
# Both tables are independent POST-for-read queries. S-CLAROTY-DEVVULNREL-001 (Wave B G3)
# does not block this story — the servers/server_interfaces tables do not join to
# vulnerabilities in this first-cut 17/10-column spec. S-ADR058-OCSF-ROUTING-001
# (which activated ocsf_column_naming=true) is already MERGED (PR #242,
# develop@3f1e66179). No delivery-time scheduling dependency remains.
blocks: []
acceptance_criteria_count: 16
risk: MEDIUM
# Risk justification:
#   Both tables have no DTU; RG-005/RG-006/RG-013/RG-014 (live Variant-1 tests) are
#   #[ignore]'d until live validation against monroe. The composite PK for
#   claroty_server_interfaces is semantic only — no TOML REQUIRED option on
#   interface_name. The server_interfaces endpoint is SEPARATE from /api/v1/servers/
#   (confirmed from OpenAPI spec; endpoint correction documented in BC-2.16.019 §Description).
#   uptime_days ColumnType is Float — CONFIRMED (pre-delivery remove-uncertainty pass 2026-08-31):
#   the GetServersResponse §example in xdome_openapi_06.20.2026.json carries uptime_days = 667.233661
#   (fractional), so Float is positively grounded in the schema example; the earlier "verify before
#   asserting Integer" caution is resolved (live re-confirmation welcome but no longer gating).
#   SAP-2 DTU-parity probe is N/A per D-2200 for both tables.
assumption_validations: []
risk_mitigations: []
---

# S-CLAROTY-SERVERS-001: Claroty xDome Collection Servers + Server Interfaces Tables — TOML Blocks + Live Structural Tests

## Authority

**BC-2.16.018 v1.0 §Postconditions §1 — TOML Table Contract (claroty_servers)** governs the
exact `[[tables]]` block structure: `table_name = "servers"` (bare name; `{sensor_id}_{table_name}` derives the registered/queryable name `claroty_servers`),
`ocsf_class = "inventory_info"`, step name `"fetch_servers"`, `method = "POST"`,
`path_template = "/api/v1/servers/"`, `response_path = "$.servers"`,
pagination `type = "offset_limit"` / `page_size = 1000`, and the 17-field `body_template`.
Read §Postconditions §1 in full before authoring the TOML.

**BC-2.16.018 v1.0 §Postconditions §2 — Tier-1/Tier-2 Column Classification (claroty_servers)**
governs Arrow field naming under `ocsf_column_naming = true`:
- Tier-1: `server_name` (`ocsf_field = "device.name"` → Arrow `device_name`, options REQUIRED),
  `server_status` (`ocsf_field = "status_code"` → Arrow `status_code`).
- Tier-2 (15 columns): all remaining columns aggregate into `raw_extensions`.

**BC-2.16.019 v1.0 §Postconditions §1 — TOML Table Contract (claroty_server_interfaces)** governs
the exact `[[tables]]` block: `table_name = "server_interfaces"` (bare name; `{sensor_id}_{table_name}` derives the registered/queryable name `claroty_server_interfaces`),
`ocsf_class = "inventory_info"`, step name `"fetch_server_interfaces"`, `method = "POST"`,
`path_template = "/api/v1/server_interfaces/"`, `response_path = "$.server_interfaces"`,
pagination `type = "offset_limit"` / `page_size = 1000`, and the 10-field `body_template`.
**Endpoint correction from initial plan:** this is a SEPARATE endpoint from `/api/v1/servers/`
(operationId `get_servers_api_v1_server_interfaces__post`, confirmed from OpenAPI spec).
Read §Postconditions §1 in full before authoring the TOML.

**BC-2.16.019 v1.0 §Postconditions §2 — Tier-1/Tier-2 Column Classification (claroty_server_interfaces)**:
- Tier-1: `server_name` (`ocsf_field = "device.name"` → Arrow `device_name`, options REQUIRED),
  `interface_status` (`ocsf_field = "status_code"` → Arrow `status_code`).
- Tier-2 (8 columns, including composite PK element `interface_name`): all aggregate into
  `raw_extensions`.

**BC-2.16.019 v1.0 §Postconditions §3 — Composite PK (claroty_server_interfaces)**:
Composite PK is (`server_name`, `interface_name`). `server_name` carries `REQUIRED`;
`interface_name` does NOT — a row with null `interface_name` is degraded but not dropped.
`interface_name` is Tier-2 (in `raw_extensions`) and is the primary join key for cross-table
server enrichment.

**ADR-058 §B2** — Tier-2 columns (those without `ocsf_field`) MUST aggregate into `raw_extensions`
under `ocsf_column_naming = true`. The `inventory_info` OCSF class maps to class_uid 5001 —
the existing arm in `class_selector.rs::select_by_class_name` used without modification.

**ADR-058 §C** — `ocsf_field_to_arrow_name("device.name")` = `"device_name"` (dot → underscore
flattening); `ocsf_field_to_arrow_name("status_code")` = `"status_code"` (no change).

**spike-findings §Overall Verdict** confirms `"inventory_info"` arm at class_uid 5001 exists
in `class_selector.rs::select_by_class_name`. No new arm required for either table.

**S-ADR058-OCSF-ROUTING-001** (merged PR #242, develop@3f1e66179) activated
`ocsf_column_naming = true` at the sensor level in `claroty.sensor.toml`. Both new tables
inherit this setting automatically — no per-table flag needed.

---

## Narrative

As a SOC analyst querying Claroty xDome infrastructure data via PrismQL,
I want `claroty_servers` and `claroty_server_interfaces` tables with OCSF `inventory_info`
class,
so that I can query Claroty collection server appliance records and their network interface
inventory — with OCSF field routing (`device_name` for server identity, `status_code` for
operational state) and Tier-2 details (traffic metrics, IP/MAC addresses, interface type,
connection type) available via `raw_extensions`, enabling infrastructure health queries and
cross-table joins between server appliances and their interfaces via `server_name`.

## Background

As of develop@3f1e66179 the committed `crates/prism-sensors/specs/claroty.sensor.toml`
contains 4 tables — `alerts`, `audit_logs`, `devices`, `device_alert_relations` (verified by
direct inspection of the TOML during the remove-uncertainty pass; exactly 4 `table_name =`
declarations, one per named table). The Wave A/B G1–G3 expansion tables referenced by
prior-story intelligence (`claroty_vulnerabilities`, `claroty_ot_activity_events`,
`claroty_device_vulnerability_relations`) are NOT present in the committed TOML at this
story's authoring time. The implementer MUST re-verify the actual baseline table count at
implementation time and treat the post-story total as **baseline + 2** (6 if the baseline is
still the 4-table set at implementation time). See §Notes for Implementer item 9 for the
merge-status residual. The `/api/v1/servers/` and `/api/v1/server_interfaces/` endpoints
(Gap G4) are the Wave C priority.

This story delivers the complete Wave C G4 addition (two TOML blocks):

1. **`claroty_servers`** — TOML `[[tables]]` block with 17 columns (2 Tier-1 + 15 Tier-2),
   single-column PK (`server_name` REQUIRED), offset_limit pagination, response_path `$.servers`.

2. **`claroty_server_interfaces`** — TOML `[[tables]]` block with 10 columns (2 Tier-1 + 8 Tier-2,
   including composite PK element `interface_name` in Tier-2), composite PK
   (`server_name`, `interface_name`), offset_limit pagination, response_path `$.server_interfaces`,
   at the SEPARATE endpoint `/api/v1/server_interfaces/`.

3. **Tests** — TOML parse unit tests + live structural Variant-1 tests against monroe (wire-level
   JSON assertions) for BOTH tables.

**Live-test approach (per xdome-endpoint-expansion-plan.md §Per-Story Pipeline):**

- **Variant-1 (structural, required):** Live `#[ignore]`'d integration tests against the
  monroe sensor. Assertions are wire-level on the serialized JSON response (class_uid, field
  presence, raw_extensions keys). Tests marked `#[ignore]` with comment:
  `// LIVE-MONROE-001: requires CLAROTY_INSTANCE_URL env var pointing to monroe; run manually or in live-validation CI job`.
- **Variant-2 (agent, optional):** PrismQL agent-level test exercising the full LLM agent
  reasoning path. Deferred to live-validation milestone if not complete before holdout gate.
- **DTU note:** SAP-2 DTU-parity probe is **N/A** for both tables per BC-2.16.018 §PC4,
  BC-2.16.019 §PC4, and D-2200 governing decision (no DTU exists for either endpoint; DTU
  creation is a separate deferred story). Do NOT run SAP-2 checks against
  `crates/prism-dtu-claroty/src/` — neither `servers` nor `server_interfaces` routes exist
  there yet and their absence is expected.

**Composite PK note (claroty_server_interfaces):** The composite PK (`server_name`,
`interface_name`) is a semantic identity, not a TOML-level declaration. Only `server_name`
carries `options = ["REQUIRED"]`; `interface_name` does NOT. `interface_name` is Tier-2 (in
`raw_extensions`) — a row with null `interface_name` is degraded (server identified, interface
lost) but not dropped. Cross-table joins with `claroty_servers` use `server_name = device_name`.

**Story-level holdout gate:** After LOCAL 3-CLEAN adversary convergence and BEFORE demo
recording / push to origin, the holdout-evaluator runs HS-027 (3 hidden SINGLE-USE scenarios
authored by PO at remove-uncertainty time; stored under the holdout directory; contamination-
controlled — test-writer and implementer MUST NOT read the HS-027 scenario files). The gate
is BLOCKING: unsatisfied scenarios reset the LOCAL streak per BC-5.39.001.

## Behavioral Contracts

| BC | Title | Version | Role |
|----|-------|---------|------|
| BC-2.16.018 | Claroty xDome Collection Servers Table — Queryable Surface and OCSF inventory_info Mapping (No DTU) | v1.0 | §Postconditions §1 TOML table contract (step, path, body_template, pagination, response_path `$.servers`); §Postconditions §2 Tier-1/Tier-2 classification (2 Tier-1: device_name REQUIRED + status_code; 15 Tier-2 into raw_extensions); §Postconditions §3 PK rationale (server_name → device_name single-column); §Postconditions §4 SAP-2 N/A; EC-016-018-001..006 edge cases. ACs 001–008 trace to this BC. |
| BC-2.16.019 | Claroty xDome Server Interfaces Table — Queryable Surface, Composite PK, and OCSF inventory_info Mapping (No DTU) | v1.0 | §Postconditions §1 TOML table contract (step, path `/api/v1/server_interfaces/`, body_template, pagination, response_path `$.server_interfaces`; SEPARATE endpoint confirmed); §Postconditions §2 Tier-1/Tier-2 classification (2 Tier-1: device_name REQUIRED + status_code; 8 Tier-2 into raw_extensions incl. composite PK element interface_name); §Postconditions §3 composite PK (server_name + interface_name); §Postconditions §4 SAP-2 N/A; EC-016-019-001..006 edge cases. ACs 009–016 trace to this BC. |

## Acceptance Criteria

### — claroty_servers (BC-2.16.018) —

### AC-001: TOML block parses without validation error; 17 columns declared; pagination offset_limit 1000 (traces to BC-2.16.018 postcondition 1 — TOML Table Contract)

`crates/prism-sensors/specs/claroty.sensor.toml` declares a `[[tables]]` block with
`table_name = "servers"` (bare name; registered/queryable name `claroty_servers` derived as `{sensor_id}_{table_name}`), `ocsf_class = "inventory_info"`,
a step named `"fetch_servers"` with `method = "POST"`,
`path_template = "/api/v1/servers/"`,
`response_path = "$.servers"`, pagination `type = "offset_limit"` / `page_size = 1000`,
and `body_template` containing all 17 contracted fields.

`SpecLoader::parse` on the modified TOML returns `Ok(SensorSpec)` without validation error.
The parsed spec reports 17 `ColumnSpec` entries for `claroty_servers`.

**Test:** `test_BC_2_16_018_claroty_servers_toml_block_parses`

### AC-002: Two Tier-1 columns declared with correct ocsf_field; Arrow names are `device_name` (REQUIRED) and `status_code` (traces to BC-2.16.018 postcondition 2 — Tier-1 column classification)

The `[[tables.columns]]` block for `server_name` declares:
- `column_type = "string"`, `ocsf_field = "device.name"`, `options = ["REQUIRED"]`

The `[[tables.columns]]` block for `server_status` declares:
- `column_type = "string"`, `ocsf_field = "status_code"`

Under `ocsf_column_naming = true`, `ocsf_field_to_arrow_name("device.name")` = `"device_name"`
and `ocsf_field_to_arrow_name("status_code")` = `"status_code"`. Exactly 2 of 17 columns have
a non-None `ocsf_field`. Exactly 15 columns have no `ocsf_field` (aggregate into `raw_extensions`).

**Test:** `test_BC_2_16_018_claroty_servers_tier1_columns_two_with_ocsf_field`

### AC-003: Tier-2 column query raises E-QUERY-038; `available_columns` contains `raw_extensions` not raw Tier-2 name (traces to BC-2.16.018 invariant — Tier-2 not exposed as standalone Arrow column; error case E-QUERY-038; EC-016-018-005)

A PrismQL query `SELECT server_location FROM claroty.claroty_servers LIMIT 1`
raises E-QUERY-038 (column-not-found) at plan time. The error's `available_columns`
MUST contain `raw_extensions`, `device_name`, `status_code`, `class_uid`, `_sensor`
and MUST NOT contain `server_location` as a standalone column name.

Same applies for any other Tier-2 column (`model`, `management_ip`, `os_version`,
`serial_number`, `uptime_days`, etc.).

**Test:** `test_BC_2_16_018_claroty_servers_tier2_column_raises_e_query_038`
(drives through the plan-time validation path, not just a spec-parse assertion)

### AC-004 (WIRE-SHAPE rename): SELECT server_status (raw Tier-1 TOML name) raises E-QUERY-038; `available_columns` contains `status_code` but NOT `server_status` (traces to BC-2.16.018 invariant — raw Tier-1 TOML name rejected; Arrow name status_code is the accepted form; TV-BC-2.16.018-003 pattern)

A PrismQL query `SELECT server_status FROM claroty.claroty_servers LIMIT 1` raises
E-QUERY-038 at plan time. The error's `available_columns` MUST contain `status_code`
(the Arrow form) but MUST NOT contain `server_status` (the raw TOML column name).
Similarly, `SELECT server_name FROM claroty.claroty_servers LIMIT 1` raises E-QUERY-038;
`available_columns` MUST contain `device_name` but NOT `server_name`.

**Test:** `test_BC_2_16_018_claroty_servers_tier1_raw_toml_name_raises_e_query_038`
(plan-time validation; asserts server_status → E-QUERY-038, available_columns has status_code)

### AC-005 (WIRE-SHAPE): Live Variant-1 wire-shape — `SELECT * LIMIT 1` serialized JSON contains class_uid=5001, device_name present, status_code present, raw_extensions present (traces to BC-2.16.018 postcondition 1 class_uid; postcondition 2 Tier-1/Tier-2 wire representation; TV-BC-2.16.018-002)

Against the live monroe sensor, `SELECT * FROM claroty.claroty_servers LIMIT 1`
serialized JSON response (MCP-visible wire shape per 2026-07-13 wire-shape discipline):
1. `class_uid` key is present with value `5001`
2. `device_name` key is present (non-null string — collection server name)
3. `status_code` key is present (value in {"Up", "Down", "Pending"}) — see casing note below
4. `raw_extensions` key is present as a JSON object (not null, not absent)
5. None of `server_name`, `server_status`, `server_location`, `management_ip` etc. appear as
   standalone top-level keys (all Tier-2 columns are inside raw_extensions; Tier-1 raw names
   are not present as root keys)

**Status-value casing note (pre-delivery remove-uncertainty pass 2026-08-31):** the
`GetServersResponse` §example in `xdome_openapi_06.20.2026.json` renders `server_status` in
lowercase (`"up"`) alongside other clearly-synthetic placeholders (`model="model"`,
`notes="note"`, `serial_number="serial"`). The capitalized set `{"Up","Down","Pending"}` above
reflects the expected live xDome values, but exact live casing is UNCONFIRMED from the schema
example. The `#[ignore]`'d live test MUST compare `status_code` case-insensitively (or confirm
exact casing against monroe at live-validation) and MUST NOT fail on casing alone. This applies
equally to the raw `status_code` value assertion in RG-005.

**Test:** `test_BC_2_16_018_claroty_servers_live_wire_shape_class_uid_and_tier1`
(`#[ignore]` — requires `CLAROTY_INSTANCE_URL` env var pointing to monroe)

### AC-006: `SELECT raw_extensions LIMIT 5` succeeds; raw_extensions JSON object contains expected Tier-2 keys (traces to BC-2.16.018 postcondition 2 — Tier-2 source columns in raw_extensions; TV-BC-2.16.018-005)

Against the live monroe sensor, `SELECT raw_extensions FROM claroty.claroty_servers LIMIT 5`
returns rows where `raw_extensions` is a non-null JSON object. The deserialized JSON object
contains at minimum `management_ip`, `model`, `os_version` keys (or null values for those keys)
when the live API returns them. No E-QUERY-038 is raised on `raw_extensions` itself.

**Test:** `test_BC_2_16_018_claroty_servers_live_raw_extensions_contains_tier2_keys`
(`#[ignore]` — requires `CLAROTY_INSTANCE_URL` env var pointing to monroe)

### AC-007: Missing REQUIRED `server_name` field → null row, no hard error, subsequent rows unaffected (traces to BC-2.16.018 invariant — server_name MUST be present; EC-016-018-001)

The `server_name` column carries `options = ["REQUIRED"]` in the TOML. When the API response
contains a server row where `server_name` is absent or null, the spec-engine produces a null
row (REQUIRED semantics) without raising a hard error. Subsequent rows in the page continue
to be materialized normally.

**Test:** `test_BC_2_16_018_claroty_servers_required_server_name_absent_produces_null_row`
(unit test with mock response payload containing a row missing `server_name`)

### AC-008: Nullable count envelope — empty-page halt triggers correctly; no error when count is null (traces to BC-2.16.018 postcondition 1 pagination note; EC-016-018-004)

When the `servers` response envelope contains `count: null` or omits `count` entirely,
the spec-engine pagination logic uses the empty-page check (halts when returned page is empty),
not a null-pointer dereference on `count`. No error is raised. Consistent with the established
pattern in `claroty_vulnerabilities` (BC-2.16.015 EC-016-015-003) and
`claroty_device_vulnerability_relations` (BC-2.16.017 EC-016-017-005).

**Test:** `test_BC_2_16_018_claroty_servers_nullable_count_uses_empty_page_halt`
(unit test with mock response containing `{"servers": [], "count": null}`)

### — claroty_server_interfaces (BC-2.16.019) —

### AC-009: TOML block parses without validation error; 10 columns declared; pagination offset_limit 1000; path is `/api/v1/server_interfaces/` (SEPARATE endpoint) (traces to BC-2.16.019 postcondition 1 — TOML Table Contract; endpoint correction)

`crates/prism-sensors/specs/claroty.sensor.toml` declares a `[[tables]]` block with
`table_name = "server_interfaces"` (bare name; registered/queryable name `claroty_server_interfaces` derived as `{sensor_id}_{table_name}`), `ocsf_class = "inventory_info"`,
a step named `"fetch_server_interfaces"` with `method = "POST"`,
`path_template = "/api/v1/server_interfaces/"` (SEPARATE endpoint — NOT sub-path of /servers/),
`response_path = "$.server_interfaces"`, pagination `type = "offset_limit"` / `page_size = 1000`,
and `body_template` containing all 10 contracted fields.

`SpecLoader::parse` on the modified TOML returns `Ok(SensorSpec)` without validation error.
The parsed spec reports 10 `ColumnSpec` entries for `claroty_server_interfaces`.

**Test:** `test_BC_2_16_019_claroty_server_interfaces_toml_block_parses`

### AC-010: Two Tier-1 columns declared with correct ocsf_field; Arrow names are `device_name` (REQUIRED) and `status_code` (traces to BC-2.16.019 postcondition 2 — Tier-1 column classification)

The `[[tables.columns]]` block for `server_name` declares:
- `column_type = "string"`, `ocsf_field = "device.name"`, `options = ["REQUIRED"]`

The `[[tables.columns]]` block for `interface_status` declares:
- `column_type = "string"`, `ocsf_field = "status_code"`

Under `ocsf_column_naming = true`, `ocsf_field_to_arrow_name("device.name")` = `"device_name"`
and `ocsf_field_to_arrow_name("status_code")` = `"status_code"`. Exactly 2 of 10 columns have
a non-None `ocsf_field`. Exactly 8 columns have no `ocsf_field` (all aggregate into `raw_extensions`),
including the composite PK element `interface_name`.

**Test:** `test_BC_2_16_019_claroty_server_interfaces_tier1_columns_two_with_ocsf_field`

### AC-011: Tier-2 column `interface_name` (composite PK element) query raises E-QUERY-038; composite PK element is in raw_extensions, not a standalone Arrow column (traces to BC-2.16.019 invariant — interface_name Tier-2 despite PK role; EC-016-019-004)

A PrismQL query `SELECT interface_name FROM claroty.claroty_server_interfaces LIMIT 1`
raises E-QUERY-038 (column-not-found) at plan time. The error's `available_columns` MUST
contain `raw_extensions`, `device_name`, `status_code`, `class_uid`, `_sensor` and MUST NOT
contain `interface_name` as a standalone column name.

Despite `interface_name` being a composite PK element, it is classified Tier-2 (no `ocsf_field`)
and lives in `raw_extensions`. Cross-table access to `interface_name` requires
`SELECT raw_extensions` and JSON extraction.

Same applies for any other Tier-2 column (`interface_type`, `interface_connection_type`,
`site_id`, `avg_traffic_past_month_mbps`, etc.).

**Test:** `test_BC_2_16_019_claroty_server_interfaces_tier2_column_raises_e_query_038`
(drives through the plan-time validation path, not just a spec-parse assertion)

### AC-012 (WIRE-SHAPE rename): SELECT interface_status (raw Tier-1 TOML name) raises E-QUERY-038; `available_columns` contains `status_code` but NOT `interface_status` (traces to BC-2.16.019 invariant — Tier-1 rename enforced; EC-016-019-006)

A PrismQL query `SELECT interface_status FROM claroty.claroty_server_interfaces LIMIT 1`
raises E-QUERY-038 at plan time. The error's `available_columns` MUST contain `status_code`
(the Arrow form of the Tier-1 rename) but MUST NOT contain `interface_status` (the raw TOML
column name). The Arrow column `status_code` IS accessible; `interface_status` IS NOT.

**Test:** `test_BC_2_16_019_claroty_server_interfaces_interface_status_raw_name_raises_e_query_038`
(plan-time validation; asserts interface_status → E-QUERY-038, available_columns has status_code)

### AC-013 (WIRE-SHAPE): Live Variant-1 wire-shape — `SELECT * LIMIT 1` serialized JSON contains class_uid=5001, device_name, status_code (Up/No Carrier), raw_extensions with composite PK keys (traces to BC-2.16.019 postcondition 1 class_uid; postcondition 2 Tier-1/Tier-2 wire representation; postcondition 3 composite PK join keys in raw_extensions; TV-BC-2.16.019-002)

Against the live monroe sensor, `SELECT * FROM claroty.claroty_server_interfaces LIMIT 1`
serialized JSON response (MCP-visible wire shape per 2026-07-13 wire-shape discipline):
1. `class_uid` key is present with value `5001`
2. `device_name` key is present (non-null string — collection server name)
3. `status_code` key is present (value in {"Up", "No Carrier"}) — see casing note below
4. `raw_extensions` key is present as a JSON object (not null, not absent)
5. The `raw_extensions` JSON object contains `interface_name` and `interface_type` keys
   (the composite PK join key and interface type must be accessible via raw_extensions)
6. None of `server_name`, `interface_status`, `interface_name`, `interface_type`,
   `avg_traffic_past_month_mbps` etc. appear as standalone top-level keys in the row

**Status-value casing note (pre-delivery remove-uncertainty pass 2026-08-31):** the
`GetServerInterfacesResponse` §example in `xdome_openapi_06.20.2026.json` renders
`interface_status` in lowercase (`"up"`), `interface_type` as `"span"`, and
`interface_connection_type` as `"RJ45"` — synthetic placeholders. The capitalized set
`{"Up","No Carrier"}` above reflects expected live xDome values, but exact live casing is
UNCONFIRMED from the schema example. The `#[ignore]`'d live test MUST compare `status_code`
case-insensitively (or confirm exact casing against monroe at live-validation) and MUST NOT
fail on casing alone. This applies equally to the raw `status_code` value assertion in RG-013.

**Test:** `test_BC_2_16_019_claroty_server_interfaces_live_wire_shape_class_uid_and_tier1`
(`#[ignore]` — requires `CLAROTY_INSTANCE_URL` env var pointing to monroe)

### AC-014: `SELECT raw_extensions LIMIT 5` succeeds; raw_extensions JSON object contains expected Tier-2 keys including composite PK join element `interface_name` (traces to BC-2.16.019 postcondition 2 — Tier-2 source columns in raw_extensions; TV-BC-2.16.019-005)

Against the live monroe sensor, `SELECT raw_extensions FROM claroty.claroty_server_interfaces LIMIT 5`
returns rows where `raw_extensions` is a non-null JSON object. The deserialized JSON object
contains at minimum `interface_name`, `interface_type`, `interface_connection_type` keys (or null
values for those keys) when the live API returns them. No E-QUERY-038 is raised on
`raw_extensions` itself.

**Test:** `test_BC_2_16_019_claroty_server_interfaces_live_raw_extensions_contains_tier2_keys`
(`#[ignore]` — requires `CLAROTY_INSTANCE_URL` env var pointing to monroe)

### AC-015: Missing REQUIRED `server_name` → null row; null `interface_name` (composite PK degraded) → row not dropped (traces to BC-2.16.019 invariant — server_name REQUIRED; interface_name degraded-but-valid; EC-016-019-001 + EC-016-019-002)

When the API response contains a server-interface row where `server_name` is absent or null,
the spec-engine produces a null row (REQUIRED semantics) without raising a hard error.

When `interface_name` is absent or null (composite PK degraded), the row is NOT dropped —
`server_name` still resolves to a non-null `device_name` Arrow cell; `interface_name` is null
in `raw_extensions`. This is valid per BC-2.16.019 §Invariants: only `server_name` carries
REQUIRED; `interface_name` does NOT.

**Tests:**
- `test_BC_2_16_019_claroty_server_interfaces_required_server_name_absent_produces_null_row`
  (unit test with mock response containing a row missing `server_name`)
- `test_BC_2_16_019_claroty_server_interfaces_null_interface_name_row_not_dropped`
  (unit test with mock response containing a row where `interface_name` is null but
   `server_name` is non-null; verifies row is materialized with null in raw_extensions)

### AC-016: Nullable count envelope — empty-page halt triggers correctly; no error when count is null (traces to BC-2.16.019 postcondition 1 pagination note; EC-016-019-005)

When the `server_interfaces` response envelope contains `count: null` or omits `count` entirely,
the spec-engine pagination logic uses the empty-page check (halts when returned page is empty),
not a null-pointer dereference on `count`. No error is raised. Consistent with the established
pattern in `claroty_vulnerabilities` and `claroty_servers`.

**Test:** `test_BC_2_16_019_claroty_server_interfaces_nullable_count_uses_empty_page_halt`
(unit test with mock response containing `{"server_interfaces": [], "count": null}`)

## Red Gate Tests

| ID | Test name | Test type | What it gates |
|----|-----------|-----------|---------------|
| RG-001 | `test_BC_2_16_018_claroty_servers_toml_block_parses` | Unit (SpecLoader::parse) | AC-001: TOML block parses Ok; 17 column entries returned for claroty_servers; pagination offset_limit 1000 |
| RG-002 | `test_BC_2_16_018_claroty_servers_tier1_columns_two_with_ocsf_field` | Unit (ColumnSpec inspection) | AC-002: exactly 2 Tier-1 columns (ocsf_field == Some); server_name→device.name REQUIRED; server_status→status_code; 15 Tier-2 have None |
| RG-003 | `test_BC_2_16_018_claroty_servers_e2e_e_query_038_tier2_column` | Integration end-to-end (prism-bin, via QueryEngine::execute — authoritative; prism-sensors version is defense-in-depth per SAP-3 rule 3) | AC-003: SELECT server_location raises E-QUERY-038; available_columns excludes server_location; includes raw_extensions, device_name, status_code |
| RG-004 | `test_BC_2_16_018_claroty_servers_tier1_raw_toml_name_raises_e_query_038` | Integration (plan-time validation) | AC-004 (WIRE-SHAPE rename): SELECT server_status raises E-QUERY-038; available_columns has status_code but NOT server_status; SELECT server_name raises E-QUERY-038; available_columns has device_name but NOT server_name |
| RG-005 | `test_BC_2_16_018_claroty_servers_live_wire_shape_class_uid_and_tier1` | Live Variant-1 (`#[ignore]`) | AC-005 (WIRE-SHAPE): wire JSON class_uid=5001, device_name present, status_code present, raw_extensions present; no Tier-2 as standalone root keys |
| RG-006 | `test_BC_2_16_018_claroty_servers_live_raw_extensions_contains_tier2_keys` | Live Variant-1 (`#[ignore]`) | AC-006: raw_extensions JSON object contains management_ip, model, os_version keys; no E-QUERY-038 on raw_extensions |
| RG-007 | `test_BC_2_16_018_claroty_servers_required_server_name_absent_produces_null_row` | Unit (mock response) | AC-007: row missing server_name → null row; no hard error; subsequent rows continue |
| RG-008 | `test_BC_2_16_018_claroty_servers_nullable_count_uses_empty_page_halt` | Unit (mock response) | AC-008: count=null in servers envelope → empty-page halt; no error; no null-ptr deref |
| RG-009 | `test_BC_2_16_019_claroty_server_interfaces_toml_block_parses` | Unit (SpecLoader::parse) | AC-009: TOML block parses Ok; 10 column entries for claroty_server_interfaces; path /api/v1/server_interfaces/ (SEPARATE); response_path $.server_interfaces |
| RG-010 | `test_BC_2_16_019_claroty_server_interfaces_tier1_columns_two_with_ocsf_field` | Unit (ColumnSpec inspection) | AC-010: exactly 2 Tier-1 columns; server_name→device.name REQUIRED; interface_status→status_code; 8 Tier-2 (incl. interface_name) have None |
| RG-011 | `test_BC_2_16_019_claroty_server_interfaces_e2e_e_query_038_tier2_column` | Integration end-to-end (prism-bin, via QueryEngine::execute — authoritative; prism-sensors version is defense-in-depth per SAP-3 rule 3) | AC-011: SELECT interface_name raises E-QUERY-038; available_columns excludes interface_name; includes raw_extensions (composite PK element correctly Tier-2) |
| RG-012 | `test_BC_2_16_019_claroty_server_interfaces_interface_status_raw_name_raises_e_query_038` | Integration (plan-time validation) | AC-012 (WIRE-SHAPE rename): SELECT interface_status raises E-QUERY-038; available_columns has status_code but NOT interface_status |
| RG-013 | `test_BC_2_16_019_claroty_server_interfaces_live_wire_shape_class_uid_and_tier1` | Live Variant-1 (`#[ignore]`) | AC-013 (WIRE-SHAPE): wire JSON class_uid=5001, device_name present, status_code (Up/No Carrier), raw_extensions with interface_name+interface_type; no Tier-2 as standalone root keys |
| RG-014 | `test_BC_2_16_019_claroty_server_interfaces_live_raw_extensions_contains_tier2_keys` | Live Variant-1 (`#[ignore]`) | AC-014: raw_extensions JSON object contains interface_name, interface_type, interface_connection_type keys; no E-QUERY-038 on raw_extensions |
| RG-015 | `test_BC_2_16_019_claroty_server_interfaces_required_server_name_absent_produces_null_row` + `test_BC_2_16_019_claroty_server_interfaces_null_interface_name_row_not_dropped` | Unit (mock response) — two sub-tests | AC-015: (1) server_name absent → null row; (2) interface_name null, server_name non-null → row materialized with null in raw_extensions (composite PK degraded, not dropped) |
| RG-016 | `test_BC_2_16_019_claroty_server_interfaces_nullable_count_uses_empty_page_halt` | Unit (mock response) | AC-016: count=null in server_interfaces envelope → empty-page halt; no error; no null-ptr deref |
| RG-017 | `test_BC_2_16_018_claroty_servers_wire_shape_class_uid_5001_mock` | Integration (prism-bin, wire-shape via SpecDrivenSensorAdapter::fetch — authoritative path; no DTU per D-2200) | SAP-4 production-path: class_uid=5001; device_name present; raw_extensions present as JSON object; Tier-2 NOT as standalone root keys |
| RG-018 | `test_BC_2_16_018_claroty_servers_null_passthrough_server_name_absent_null_not_absent` | Integration (prism-bin, wire-shape via SpecDrivenSensorAdapter::fetch — authoritative path) | null-not-absent wire discipline: absent server_name produces explicit null cell in wire output (explicit_nulls=true); key present with null value, not absent |
| RG-019 | `test_BC_2_16_018_claroty_servers_ec016_018_004_count_null_empty_page_halt_ok_zero_rows` | Integration (prism-bin, production-path via SpecDrivenSensorAdapter::fetch — authoritative) | EC-016-018-004: count=null in servers envelope → empty-page halt, zero rows returned, no error |
| RG-020 | `test_BC_2_16_019_claroty_server_interfaces_wire_shape_class_uid_5001_mock` | Integration (prism-bin, wire-shape via SpecDrivenSensorAdapter::fetch — authoritative path; no DTU per D-2200) | SAP-4 production-path: class_uid=5001; server_interfaces table; device_name present; raw_extensions present; Tier-2 NOT as standalone root keys |
| RG-021 | `test_BC_2_16_019_claroty_server_interfaces_null_interface_name_row_not_dropped_wire` | Integration (prism-bin, wire-shape via SpecDrivenSensorAdapter::fetch — authoritative path) | null-not-absent wire discipline: null interface_name row not dropped; interface_name key appears as explicit null in wire output; row materialized with null in raw_extensions |
| RG-022 | `test_BC_2_16_019_claroty_server_interfaces_ec016_019_005_count_null_empty_page_halt_ok_zero_rows` | Integration (prism-bin, production-path via SpecDrivenSensorAdapter::fetch — authoritative) | EC-016-019-005: count=null in server_interfaces envelope → empty-page halt, zero rows returned, no error |

**BC-5.38.001 density check:** 22 Red Gate tests / 16 acceptance criteria = 1.375 ≥ 0.5 threshold. PASS.
(Note: RG-015 gates two sub-tests under AC-015; counted as 1 RGT per 1 AC. RG-017..RG-022 are authoritative fetch-path wire-shape and production-path tests via SpecDrivenSensorAdapter::fetch.)

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| `claroty_servers` TOML block | `crates/prism-sensors/specs/claroty.sensor.toml` | Static data (TOML spec) |
| `claroty_server_interfaces` TOML block | `crates/prism-sensors/specs/claroty.sensor.toml` | Static data (TOML spec) |
| TOML parse validation (both tables) | `crates/prism-spec-engine/src/spec_parser.rs §spec_parser` | Pure (TOML deserialization; no I/O) |
| Tier-1/Tier-2 Arrow schema computation (both tables) | `crates/prism-spec-engine/src/column_mapping.rs §ocsf_field_to_arrow_name` | Pure (string transformation; no I/O) |
| OffsetLimit POST-body injection (both tables) | `crates/prism-spec-engine/src/pipeline.rs §PipelineExecutor::execute` | Effectful (HTTP POST to xDome; merges offset/limit into body_template) |
| response_path extraction (both tables) | `crates/prism-bin/src/spec_driven_adapter.rs §pipeline_result_to_record_batch` | Effectful (processes HTTP response; builds Arrow RecordBatch) |
| `inventory_info` class arm (shared by both tables) | `crates/prism-ocsf/src/class_selector.rs::select_by_class_name` | Pure (constant → u32 lookup; arm already exists; returns 5001) |

Architecture section references:
- `architecture/module-decomposition.md` §SS-01 Sensor Adapters (prism-sensors; claroty.sensor.toml)
- `architecture/module-decomposition.md` §SS-16 Spec Engine (prism-spec-engine; spec_parser, pipeline, column_mapping)
- ADR-058 §B2 (Tier-2 raw_extensions aggregation), §C (Arrow field naming convention: device.name → device_name), §D (ocsf_column_naming per-sensor flag)

## Purity Classification

- **Pure functions (no I/O, deterministic):** `SpecLoader::parse` (TOML deserialization);
  `ocsf_field_to_arrow_name` (string → string, deterministic);
  `select_by_class_name("inventory_info")` (constant lookup, returns 5001);
  RG-001/RG-002/RG-009/RG-010 TOML parse + column inspection assertions.
- **Effectful functions (I/O, network):** `PipelineExecutor::execute` (HTTP POST to
  `/api/v1/servers/` and `/api/v1/server_interfaces/`; pagination loops);
  `pipeline_result_to_record_batch` (HTTP response to Arrow RecordBatch);
  RG-005/RG-006/RG-013/RG-014 live integration tests (require running monroe sensor).

---

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Row where `server_name` is absent (REQUIRED, claroty_servers) | Null row produced per spec-engine REQUIRED semantics; no hard error; pagination continues (EC-016-018-001) |
| EC-002 | `server_status` is null or absent | Null `status_code` Arrow cell; not an error (EC-016-018-002) |
| EC-003 | `uptime_days` returns fractional value (confirmed: OpenAPI §example shows `667.233661`) | Float cell stored in `raw_extensions.uptime_days`; spec-engine Float type handles fractional days correctly (EC-016-018-003) |
| EC-004 | `count` is null or absent in `servers` response envelope | Pagination halts on empty page; no null-deref; consistent with other Claroty tables (EC-016-018-004) |
| EC-005 | Query references Tier-2 column `management_ip` by raw name | E-QUERY-038; `available_columns` contains `raw_extensions`, `device_name`, `status_code` but NOT `management_ip` (EC-016-018-005) |
| EC-006 | `server_name` values contain spaces (e.g., "Monroe Collector 1") | Preserved as-is in `device_name` Arrow column; no normalization (EC-016-018-006) |
| EC-007 | Row where `server_name` is absent (REQUIRED, claroty_server_interfaces) | Null row produced; no hard error; pagination continues (EC-016-019-001) |
| EC-008 | `interface_name` is null (composite PK degraded) | `device_name` resolves to non-null; `interface_name` null in `raw_extensions`; row materialized as degraded (EC-016-019-002) |
| EC-009 | `interface_status` is null or absent | Null `status_code` Arrow cell; not an error (EC-016-019-003) |
| EC-010 | Query references Tier-2 column `interface_name` by raw name | E-QUERY-038; `available_columns` contains `raw_extensions`, `device_name`, `status_code` but NOT `interface_name` (EC-016-019-004) |
| EC-011 | `count` is null or absent in `server_interfaces` response envelope | Pagination halts on empty page; no null-deref (EC-016-019-005) |
| EC-012 | `SELECT interface_status` (raw Tier-1 TOML name) attempted | E-QUERY-038; `available_columns` includes `status_code` but NOT `interface_status`; the Tier-1 rename is enforced (EC-016-019-006) |
| EC-013 | API returns non-200 HTTP for POST /api/v1/servers/ | E-SENSOR-001 structured error; sensor=claroty, status, body excerpt; previously fetched pages remain valid |
| EC-014 | API returns non-200 HTTP for POST /api/v1/server_interfaces/ | E-SENSOR-001 structured error; same pattern as EC-013 |

## TOML Column-Block Specification

The complete `[[tables]]` blocks for both tables as specified by BC-2.16.018 §PC1/§PC2
and BC-2.16.019 §PC1/§PC2:

```toml
# Wave C G4 — claroty_servers
# POST /api/v1/servers/ → envelope key: servers (count, servers)
# OCSF class: inventory_info (class_uid 5001; existing arm in class_selector.rs)
# PK: server_name (String, REQUIRED, single-column)
# DTU status: NONE — SAP-2 probe N/A; near-term tests against live monroe only (D-2200 deferred)
[[tables]]
table_name = "servers"
# registered/queryable name = {sensor_id}_{table_name} = "claroty_servers"
ocsf_class = "inventory_info"   # class_uid 5001 (existing arm; same as claroty_devices)

# Tier-1: server_name → device_name (REQUIRED; primary key)
[[tables.columns]]
name = "server_name"
column_type = "string"
ocsf_field = "device.name"
options = ["REQUIRED"]

# Tier-1: server_status → status_code ("Up" / "Down" / "Pending")
[[tables.columns]]
name = "server_status"
column_type = "string"
ocsf_field = "status_code"

# Tier-2: physical location of the collection server appliance
[[tables.columns]]
name = "server_location"
column_type = "string"

# Tier-2: unique site identifier; numeric comparison operators supported
[[tables.columns]]
name = "site_id"
column_type = "integer"

# Tier-2: server model string (e.g. "MCS R340" or "R640")
[[tables.columns]]
name = "model"
column_type = "string"

# Tier-2: Ubuntu OS version string
[[tables.columns]]
name = "os_version"
column_type = "string"

# Tier-2: server serial number
[[tables.columns]]
name = "serial_number"
column_type = "string"

# Tier-2: count of network interfaces on the server
[[tables.columns]]
name = "num_of_interfaces"
column_type = "integer"

# Tier-2: data/management port IP address
[[tables.columns]]
name = "management_ip"
column_type = "string"

# Tier-2: iDRAC IP address
[[tables.columns]]
name = "idrac_ip"
column_type = "string"

# Tier-2: data/management port MAC address
[[tables.columns]]
name = "management_mac"
column_type = "string"

# Tier-2: days the server has been up; may be fractional (BC-2.16.018 §PC2 note — verify on live)
[[tables.columns]]
name = "uptime_days"
column_type = "float"

# Tier-2: avg traffic past month (Mbps)
[[tables.columns]]
name = "avg_traffic_past_month_mbps"
column_type = "float"

# Tier-2: avg traffic past week (Mbps)
[[tables.columns]]
name = "avg_traffic_past_week_mbps"
column_type = "float"

# Tier-2: avg traffic past hour (Mbps)
[[tables.columns]]
name = "avg_traffic_past_hour_mbps"
column_type = "float"

# Tier-2: count of open incidents associated with this server
[[tables.columns]]
name = "num_of_open_incidents"
column_type = "integer"

# Tier-2: free-text analyst notes
[[tables.columns]]
name = "notes"
column_type = "string"

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

```toml
# Wave C G4 — claroty_server_interfaces
# SEPARATE endpoint POST /api/v1/server_interfaces/ → envelope key: server_interfaces
# (note: SEPARATE from /api/v1/servers/; operationId: get_servers_api_v1_server_interfaces__post)
# OCSF class: inventory_info (class_uid 5001; existing arm; same as claroty_servers)
# Composite PK: (server_name, interface_name) — server_name REQUIRED; interface_name Tier-2
# DTU status: NONE — SAP-2 probe N/A; near-term tests against live monroe only (D-2200 deferred)
[[tables]]
table_name = "server_interfaces"
# registered/queryable name = {sensor_id}_{table_name} = "claroty_server_interfaces"
ocsf_class = "inventory_info"   # class_uid 5001 (existing arm; same as claroty_servers)

# Tier-1: server_name → device_name (REQUIRED; composite PK anchor)
[[tables.columns]]
name = "server_name"
column_type = "string"
ocsf_field = "device.name"
options = ["REQUIRED"]

# Tier-1: interface_status → status_code ("Up" / "No Carrier")
[[tables.columns]]
name = "interface_status"
column_type = "string"
ocsf_field = "status_code"

# Tier-2: composite PK join key — interface name (e.g. "eth0", "ens3")
# No REQUIRED option per BC-2.16.019 §Invariants: null interface_name is degraded, not dropped
[[tables.columns]]
name = "interface_name"
column_type = "string"

# Tier-2: interface type ("SPAN" or "Management")
[[tables.columns]]
name = "interface_type"
column_type = "string"

# Tier-2: physical connection type ("SFP+" or "RJ45 (Copper)")
[[tables.columns]]
name = "interface_connection_type"
column_type = "string"

# Tier-2: unique site identifier for the site to which the interface belongs
[[tables.columns]]
name = "site_id"
column_type = "integer"

# Tier-2: avg traffic past month via this interface (Mbps)
[[tables.columns]]
name = "avg_traffic_past_month_mbps"
column_type = "float"

# Tier-2: avg traffic past week via this interface (Mbps)
[[tables.columns]]
name = "avg_traffic_past_week_mbps"
column_type = "float"

# Tier-2: avg traffic past hour via this interface (Mbps)
[[tables.columns]]
name = "avg_traffic_past_hour_mbps"
column_type = "float"

# Tier-2: free-text notes about the interface
[[tables.columns]]
name = "notes"
column_type = "string"

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

## Token Budget Estimate

| Item | Estimated tokens |
|------|-----------------|
| This story spec | ~10,000 |
| `crates/prism-sensors/specs/claroty.sensor.toml` (existing 4 tables on current develop; may be higher at implementation time if sibling expansion stories merge first per depends_on) | ~7,500 |
| BC-2.16.018 v1.0 (full) | ~5,500 |
| BC-2.16.019 v1.0 (full) | ~5,000 |
| ADR-058 §B2/§C/§D sections (ocsf_column_naming flag mechanism) | ~4,000 |
| prism-spec-engine/src/spec_parser.rs (ColumnSpec + FetchStep section) | ~3,000 |
| prism-spec-engine/src/column_mapping.rs (ocsf_field_to_arrow_name) | ~1,500 |
| Test files (16 RGTs; 12 unit + 4 live integration) | ~10,000 |
| endpoint-schema-extract.md §Server + §ServerInterfaces sections | ~1,500 |
| **Total estimate** | **~48,000 tokens** |

Well within 20-30% of a 200K window. If context is tight, load `claroty.sensor.toml` sections
by reading only the `alerts` table block first as the canonical pattern, then skip to the
pagination section. Load both BC files before writing tests — wire-shape assertions must be
derived from BC postconditions.

## Tasks

- [ ] **Task 1 (Red Gate — test first):** Write RG-001 and RG-009: `test_BC_2_16_018_claroty_servers_toml_block_parses` and `test_BC_2_16_019_claroty_server_interfaces_toml_block_parses` in `crates/prism-spec-engine/src/spec_parser.rs #[cfg(test)] mod tests` (or test fixtures). Call `SpecLoader::parse` on `claroty.sensor.toml` (or fixtures containing the new blocks). Assert `Ok(SensorSpec)` returned; `claroty_servers` with 17 ColumnSpec entries and `claroty_server_interfaces` with 10 ColumnSpec entries. MUST fail before Task 8 (blocks not yet in TOML).

- [ ] **Task 2 (Red Gate — test first):** Write RG-002 and RG-010: `test_BC_2_16_018_claroty_servers_tier1_columns_two_with_ocsf_field` and `test_BC_2_16_019_claroty_server_interfaces_tier1_columns_two_with_ocsf_field`. For servers: assert 2 Tier-1 (`server_name` → `device.name` REQUIRED; `server_status` → `status_code`; 15 Tier-2 None). For server_interfaces: assert 2 Tier-1 (`server_name` → `device.name` REQUIRED; `interface_status` → `status_code`; 8 Tier-2 None including `interface_name`). MUST fail before Task 8.

- [ ] **Task 3 (Red Gate — test first):** Write RG-007, RG-008, RG-015, RG-016 — unit tests using mock HTTP responses (no live sensor required). Tests: (a) servers required server_name absent → null row; (b) servers count=null → empty-page halt; (c) server_interfaces server_name absent → null row; (d) server_interfaces interface_name null, server_name non-null → row materialized with null in raw_extensions; (e) server_interfaces count=null → empty-page halt. Place in `crates/prism-sensors/tests/bc_2_16_018_claroty_servers.rs` and `crates/prism-sensors/tests/bc_2_16_019_claroty_server_interfaces.rs`. All MUST fail before Task 8.

- [ ] **Task 4 (Red Gate — test first):** Write RG-003 and RG-004 (servers plan-time validation): `test_BC_2_16_018_claroty_servers_tier2_column_raises_e_query_038` and `test_BC_2_16_018_claroty_servers_tier1_raw_toml_name_raises_e_query_038`. Drive `SELECT server_location` and `SELECT server_status` through the plan-time path. Assert E-QUERY-038 raised; assert available_columns memberships per AC-003 and AC-004. MUST fail before Task 8.

- [ ] **Task 5 (Red Gate — test first):** Write RG-011 and RG-012 (server_interfaces plan-time validation): `test_BC_2_16_019_claroty_server_interfaces_tier2_column_raises_e_query_038` (SELECT interface_name) and `test_BC_2_16_019_claroty_server_interfaces_interface_status_raw_name_raises_e_query_038` (SELECT interface_status). Drive through plan-time path. Assert E-QUERY-038 raised; assert available_columns has `raw_extensions` / `status_code` but NOT the raw column names. MUST fail before Task 8.

- [ ] **Task 6 (Red Gate — test first):** Write RG-005, RG-006, RG-013, RG-014 — live Variant-1 `#[ignore]`'d integration tests. Each test has comment: `// LIVE-MONROE-001: requires CLAROTY_INSTANCE_URL env var pointing to monroe; run manually or in live-validation CI job`. RG-005/006 assert claroty_servers wire-shape and raw_extensions per AC-005/006. RG-013/014 assert claroty_server_interfaces wire-shape and raw_extensions per AC-013/014. All four MUST fail when `#[ignore]` is removed if the TOML blocks are absent.

- [ ] **Task 7 (Pre-implementation endpoint check):** Confirm `path_template = "/api/v1/server_interfaces/"` is the SEPARATE endpoint (not a sub-path of `/api/v1/servers/`) against the live monroe response headers or response envelope key. The envelope key `server_interfaces` (from OpenAPI spec) must match what the live API returns. If the live API uses a different key, update the TOML accordingly and amend BC-2.16.019 §PC1 via a spec fix (route to product-owner). Do not implement with an unconfirmed envelope key for server_interfaces.

- [ ] **Task 8 (Implementation — both TOML blocks):** Add both `[[tables]]` blocks to `crates/prism-sensors/specs/claroty.sensor.toml`. Follow the exact structures from §TOML Column-Block Specification above. Add `claroty_servers` block first (after the current last `[[tables]]` block in the file — as of develop@3f1e66179 that is the `device_alert_relations` table block; do NOT assume a `claroty_device_vulnerability_relations` block exists, it is absent from the committed TOML), then `claroty_server_interfaces` block immediately after. Include comments per the existing Claroty TOML convention (Wave C G4 note, DTU deferred note, SAP-2 probe N/A note, composite PK rationale for server_interfaces, endpoint correction note for server_interfaces).

  After editing: run `just iter prism-spec-engine` — RG-001, RG-002, RG-009, RG-010 MUST turn GREEN.

- [ ] **Task 9 (Implementation — verify parse + unit tests green):** Run `just iter prism-spec-engine --no-fail-fast`. Confirm RG-001, RG-002, RG-003, RG-004, RG-007, RG-008, RG-009, RG-010, RG-011, RG-012, RG-015, RG-016 all GREEN. Confirm no existing tests regressed. Run `just iter prism-sensors` to confirm TOML file is syntactically valid.

- [ ] **Task 10 (SAP-2 self-check — N/A documented for both tables):** SAP-2 DTU-parity probe is N/A for both tables per BC-2.16.018 §PC4, BC-2.16.019 §PC4, and D-2200 (no DTU exists for `/api/v1/servers/` or `/api/v1/server_interfaces/`; neither route exists in `prism-dtu-claroty`). Record this explicitly in story comments. Do NOT create DTU routes in this story.

- [ ] **Task 11 (SAP-1 self-check):** Confirm no new `tracing::*!(event_type = ...)` emissions are added by this story (TOML-only change + unit tests). If any new emission appears during implementation, add a BC-2.16.002 catalog row per PG-LP11-001.

- [ ] **Task 12 (Final gate):** Run `just check` (full workspace). Confirm all non-`#[ignore]` Red Gate tests pass (RG-001..RG-004, RG-007..RG-012, RG-015..RG-016). Confirm no new `unwrap()`/`expect()` on `Result` in production code paths. Confirm `claroty.sensor.toml` gained exactly 2 new tables from this story (`claroty_servers` + `claroty_server_interfaces`) on top of the branch-time baseline. Baseline was 4 tables as of develop@3f1e66179, so the expected post-story total is baseline + 2 (6 unless the baseline changed at implementation time — re-verify the baseline `table_name =` count before asserting a fixed total). After `just check` passes, hold for story-level holdout gate (HS-027) before pushing to origin.

## Previous Story Intelligence

1. **S-ADR058-OCSF-ROUTING-001 (merged PR #242):** Activated `ocsf_column_naming = true` at the
   sensor level in `claroty.sensor.toml`. The Tier-1/Tier-2 routing mechanism (ADR-058 §B2/§C)
   is already active for all Claroty tables. The `inventory_info` / class_uid 5001 arm was
   confirmed existing in `class_selector.rs::select_by_class_name` (spike-findings §Overall
   Verdict). No new class_selector arm is needed.

2. **S-CLAROTY-VULNS-001 (Wave A G1 — materialized draft, pending; not yet merged/implemented):**
   Establishes the baseline TOML block pattern for new Claroty tables — its [[tables]] block
   is NOT present in the committed TOML on develop as of develop@3f1e66179 (this story has no
   depends_on on VULNS-001; the TOML pattern reference is from existing committed tables such
   as `alerts`). Key lesson: the body_template must list ONLY fields in the API's `fields_enum`
   — extra fields cause server-side rejection. For `claroty_servers`, all 17 fields are confirmed
   in the Server fields_enum (schema-extract §Server). For `claroty_server_interfaces`, all 10
   fields are confirmed in the ServerInterfaces fields_enum (schema-extract §ServerInterfaces).

3. **S-CLAROTY-DEVVULNREL-001 (Wave B G3):** Established the composite PK pattern for
   Claroty tables where no single field uniquely identifies a row. The composite PK
   (`vulnerability_name`, `device_uid`) pattern from that story is mirrored here for
   (`server_name`, `interface_name`). Key difference: in DEV-VULNREL-001, both PK elements
   could have been argued for REQUIRED. In this story, only `server_name` is REQUIRED per BC
   — `interface_name` is Tier-2 without REQUIRED, so null `interface_name` is degraded-not-dropped.

4. **S-ADR058-OCSF-COERCION-001 (merged PR #240):** Closed EC-016-013-007/008/009 (coercion
   path fixes). The `claroty_servers` and `claroty_server_interfaces` columns include Float
   types (`uptime_days`, `avg_traffic_*_mbps`). Verify that Float columns pass through the
   coercion path without hitting the now-closed bugs.

5. **S-DEMO-CLAROTY-TRAILING-SLASH-001 (merged):** Established that Claroty paths use trailing
   slash. Both `path_template = "/api/v1/servers/"` and
   `path_template = "/api/v1/server_interfaces/"` use trailing slash. The BC-specified paths
   are correct.

6. **Existing TOML pattern (claroty.sensor.toml §alerts):** The `alerts` table is the canonical
   TOML pattern to mirror: `[[tables]]` header → `[[tables.columns]]` blocks → `[[tables.steps]]`
   block. Comments should follow the Wave-CL-NNN / DTU-route / body_template rationale style.
   Read the `alerts` block as the primary template before authoring.

7. **Tier-1-rename enforcement precedent (S-CLAROTY-DEVVULNREL-001 + S-CLAROTY-VULNS-001):**
   Both prior stories validated that raw TOML column names for Tier-1-renamed columns raise
   E-QUERY-038. This story extends the same pattern: `server_status` (raw TOML) vs `status_code`
   (Arrow) for servers, and `interface_status` (raw TOML) vs `status_code` (Arrow) for
   server_interfaces. RG-004 and RG-012 gate this behavior explicitly.

## Architecture Compliance Rules

From `architecture/module-decomposition.md` §SS-16 Spec Engine:
- `spec_parser.rs §spec_parser` owns TOML deserialization; `ColumnSpec`, `FetchStep`, `PaginationConfig`
  are the canonical data structures. New `[[tables.columns]]` blocks must produce valid `ColumnSpec`
  variants or `SpecParser` returns `Err(SpecEngineError::ConfigInvalid)`.
- `ocsf_field_to_arrow_name` lives in `column_mapping.rs` (ADR-058 §I1). Do NOT re-implement
  the helper in spec_parser or elsewhere.
- `PaginationConfig::OffsetLimit { page_size: 1000 }` is the correct deserialization target
  for `type = "offset_limit"` / `page_size = 1000`.

From ADR-058 §D (ocsf_column_naming flag mechanism):
- `ocsf_column_naming = true` is already declared at the sensor level in `claroty.sensor.toml`.
  New `[[tables]]` blocks inherit this setting automatically — no per-table flag needed.
- Per ADR-058 §B2: Tier-2 columns (those without `ocsf_field`) MUST aggregate into `raw_extensions`.
  The `inventory_info` OCSF class maps to class_uid 5001 — the existing arm in
  `class_selector.rs::select_by_class_name` is used without modification.

From BC-2.16.019 §Postconditions §3 — Composite PK:
- The composite primary key (`server_name`, `interface_name`) is a semantic identity, not a
  TOML-level declaration. Neither column requires special TOML markup for PK semantics.
  `server_name` carries `options = ["REQUIRED"]`; `interface_name` does NOT (a row with null
  `interface_name` but non-null `server_name` is valid per BC-2.16.019 §Invariants).
  Do not add `options = ["REQUIRED"]` to `interface_name`.

From xdome-endpoint-expansion-plan.md §Governing Directive:
- SAP-2 probe is N/A until DTU is created (D-2200). Do NOT run parity checks against
  `crates/prism-dtu-claroty/src/`. Neither `servers` nor `server_interfaces` routes exist there.

## Library & Framework Requirements

| Library | Version | Source |
|---------|---------|--------|
| `prism-spec-engine` | workspace path | `SpecLoader::parse`, `ColumnSpec`, `FetchStep`, `PaginationConfig::OffsetLimit` |
| `prism-ocsf` | workspace path | `class_selector.rs::select_by_class_name("inventory_info")` → 5001 (existing arm — read only) |
| `serde_json` | per workspace Cargo.toml | Mock response construction in unit tests (RG-007/008/015/016) |
| `tokio` | per workspace Cargo.toml | Async test runtime for live integration tests (RG-005/006/013/014) |

Do NOT add new Cargo.toml production dependencies. The TOML spec addition requires no new
crate imports in production code.

## File Structure Requirements

| Action | File path | Notes |
|--------|-----------|-------|
| MODIFY | `crates/prism-sensors/specs/claroty.sensor.toml` | Add TWO `[[tables]]` blocks: `servers` then `server_interfaces` (bare table_names; queryable as `claroty_servers` and `claroty_server_interfaces`) after the existing last table block |
| CREATE | `crates/prism-sensors/tests/bc_2_16_018_claroty_servers.rs` | RG-003..RG-008 tests for claroty_servers; `#[ignore]` live tests include `LIVE-MONROE-001` comment |
| CREATE | `crates/prism-sensors/tests/bc_2_16_019_claroty_server_interfaces.rs` | RG-009..RG-016 integration and unit tests for claroty_server_interfaces; `#[ignore]` live tests include `LIVE-MONROE-001` comment |
| CREATE | `crates/prism-bin/tests/bc_2_16_018_claroty_servers_wire_shape.rs` | Authoritative tests for BC-2.16.018: RG-003 E2E (SELECT server_location → E-QUERY-038 via QueryEngine::execute), RG-017 class_uid=5001 mock (fetch), RG-018 null-not-absent server_name (fetch), RG-019 EC-016-018-004 count=null (fetch); SAP-2 N/A comment at file header |
| CREATE | `crates/prism-bin/tests/bc_2_16_019_claroty_server_interfaces_wire_shape.rs` | Authoritative tests for BC-2.16.019: RG-011 E2E (SELECT interface_name → E-QUERY-038 via QueryEngine::execute), RG-020 class_uid=5001 mock (fetch), RG-021 null interface_name not dropped wire (fetch), RG-022 EC-016-019-005 count=null (fetch); SAP-2 N/A comment at file header |
| MODIFY | `crates/prism-bin/Cargo.toml` | Add `arrow-json` dev-dependency (wire-shape serialization in end-to-end tests); add two `[[test]]` entries: `bc_2_16_018_claroty_servers_wire_shape` and `bc_2_16_019_claroty_server_interfaces_wire_shape` |

Files that MUST NOT be modified:
- `crates/prism-ocsf/src/class_selector.rs` — `inventory_info` arm already exists; no changes
- `crates/prism-spec-engine/src/spec_parser.rs` — no production code changes needed; RG-001/RG-002/
  RG-009/RG-010 may add unit tests in-module if easier, or inline in the test files above
- `crates/prism-dtu-claroty/` — read only (SAP-2 N/A; no DTU routes for these endpoints)
- `crates/prism-sensors/specs/claroty.sensor.toml` §existing tables — do not modify existing tables

## Forbidden Dependencies

`prism-sensors` MUST NOT gain any new production dependency on `prism-dtu-claroty` (SAP-2 N/A;
no DTU routes exist for these endpoints). `prism-spec-engine` MUST NOT gain a new dependency on
`prism-sensors` (direction is prism-sensors → prism-spec-engine, not reverse). If the build
gains a new dependency in either of these forbidden directions, the build MUST fail via
dependency-direction enforcement.

## Notes for Implementer

1. **Two TOML blocks, one file.** Add both `claroty_servers` and `claroty_server_interfaces`
   blocks to `claroty.sensor.toml` in the same commit. The blocks are independent (separate
   endpoints, separate `[[tables]]` and `[[tables.columns]]` sections). The `claroty_servers`
   block goes first; `claroty_server_interfaces` block follows immediately after.

2. **SAP-2 DTU-parity probe is N/A for BOTH tables.** Do NOT run parity checks against
   `crates/prism-dtu-claroty/src/` in this delivery. Neither `/api/v1/servers/` nor
   `/api/v1/server_interfaces/` has a registered route in `prism-dtu-claroty`. The DTU
   creation stories are deferred per D-2200 (xdome-endpoint-expansion-plan.md §Deferred
   DTU-Creation Stories). Do NOT create DTU routes as part of this story.

3. **The server_interfaces endpoint is SEPARATE.** Do NOT use `/api/v1/servers/server_interfaces/`
   or any sub-path. The correct path is `/api/v1/server_interfaces/` (confirmed from OpenAPI spec,
   operationId `get_servers_api_v1_server_interfaces__post`). Confirm against the live API in
   Task 7 before finalizing tests.

4. **interface_name has NO `options = ["REQUIRED"]`.** Despite being a composite PK element,
   `interface_name` is Tier-2 without REQUIRED. A row with null `interface_name` but non-null
   `server_name` is valid — it is degraded (server identified, interface lost) but NOT dropped.
   Do not add REQUIRED to `interface_name`.

5. **uptime_days ColumnType is Float — CONFIRMED.** The pre-delivery remove-uncertainty pass
   (2026-08-31) confirmed Float directly from the `GetServersResponse` §example in
   `xdome_openapi_06.20.2026.json`, which carries `uptime_days = 667.233661` (fractional). The
   type is now positively grounded in the schema example — the earlier "verify before asserting
   Integer" caution is resolved. The TOML declares `column_type = "float"`; keep it Float. Live
   re-confirmation on monroe is welcome but is no longer a gating uncertainty.

6. **Live tests are `#[ignore]`'d.** RG-005, RG-006, RG-013, and RG-014 require the live monroe
   sensor. Mark them `#[ignore]` with comment `// LIVE-MONROE-001: requires CLAROTY_INSTANCE_URL
   env var`. Per SID-1 discipline: since live tests are `#[ignore]`'d, confirm non-ignored unit
   tests (RG-001, RG-002, RG-009, RG-010) exercise the TOML parse path as the non-live coverage.

7. **Holdout gate (HS-027) is BLOCKING.** After LOCAL adversary 3-CLEAN and BEFORE push to
   origin, the holdout-evaluator runs HS-027 (3 hidden scenarios). Do NOT read the HS-027
   scenario files — contamination control applies (test-writer and implementer MUST NOT read
   holdout scenario text). Hold for the holdout gate before pushing.

8. **All 17 servers columns and all 10 server_interfaces columns are in their respective body_templates.**
   Unlike the Wave A `claroty_vulnerabilities` design (which carries `id` via `source_path` outside
   the fields_enum), ALL contracted columns for both tables are in the `fields` projection. The
   body_template must list all field names exactly as shown in §TOML Column-Block Specification.

9. **Wave A/B sibling stories are materialized drafts — not committed TOML.** The sibling
   expansion stories (S-CLAROTY-VULNS-001, S-CLAROTY-OT-EVENTS-001, S-CLAROTY-DEVVULNREL-001)
   are materialized drafts (pending; not yet merged/implemented). Direct inspection of
   `crates/prism-sensors/specs/claroty.sensor.toml` at develop@3f1e66179 confirms 4 tables
   (`alerts`, `audit_logs`, `devices`, `device_alert_relations`); none of
   `claroty_vulnerabilities`, `claroty_ot_activity_events`, or
   `claroty_device_vulnerability_relations` exist in the committed TOML. This does NOT block
   this story — servers/server_interfaces do not join to Wave A/B tables, and the two new blocks
   append to whatever the current last table is (depends_on: [] is correct). The implementer
   MUST re-verify the actual baseline table count at implementation time and treat the post-story
   total as baseline + 2. If Wave A/B tables are expected to land before this story, that is a
   scheduling question for the orchestrator/human to confirm.

---

## References

- BC-2.16.018 v1.0 (draft) — §Postconditions §1 TOML contract (servers); §Postconditions §2 17-column Tier-1/Tier-2; §Postconditions §3 PK rationale; §Postconditions §4 SAP-2 N/A; EC-016-018-001..006
- BC-2.16.019 v1.0 (draft) — §Postconditions §1 TOML contract (server_interfaces; SEPARATE endpoint); §Postconditions §2 10-column Tier-1/Tier-2; §Postconditions §3 composite PK (server_name + interface_name); §Postconditions §4 SAP-2 N/A; EC-016-019-001..006
- ADR-058 §B2 — Tier-2 columns aggregate into raw_extensions; §C — dot-to-underscore Arrow names; §D — per-sensor ocsf_column_naming flag
- spike-findings §Overall Verdict — inventory_info/5001 arm confirmed existing; no new arm required
- xdome-endpoint-expansion-plan.md §Gap Table G4 — Wave C scope authority (claroty_servers + claroty_server_interfaces); §Per-Story Pipeline — no-DTU live test approach; §Governing Directive — DTU skip directive
- endpoint-schema-extract.md §Server + §ServerInterfaces — 17-field and 10-field enum confirmations; envelope keys `servers` and `server_interfaces`
- `crates/prism-sensors/specs/claroty.sensor.toml §alerts` — canonical TOML block pattern to mirror
- S-ADR058-OCSF-ROUTING-001 (merged PR #242) — activated ocsf_column_naming=true; inventory_info arm confirmed existing

---

## Changelog

| Version | Date | Author | Notes |
|---------|------|--------|-------|
| 1.5 | 2026-08-31 | story-writer | FIX A: §TOML Column-Block Specification — 27 `column_name =` occurrences (17 servers + 10 server_interfaces) changed to `name =`. FIX B: §Red Gate Tests — RG-003 test name corrected (`…e2e_e_query_038_tier2_column`); RG-011 test name corrected (`…e2e_e_query_038_tier2_column`); RG-017 (single combined wire-shape row) replaced with 6 authoritative fetch-path tests (RG-017..RG-022) across two delivered files; density updated 17→22 RGTs, ratio 1.06→1.375. FIX C: §File Structure Requirements prism-bin CREATE entry split from 1 combined file (`bc_2_16_018_019_…`) to 2 BC-scoped files (`bc_2_16_018_claroty_servers_wire_shape.rs` + `bc_2_16_019_claroty_server_interfaces_wire_shape.rs`); Cargo.toml MODIFY note updated to two `[[test]]` entries. |
| 1.4 | 2026-08-31 | story-writer | G4 spec-prose corrections (MED-1/MED-4 mirroring G2 S-CLAROTY-OT-EVENTS-001 v1.3). FIX 1 (MED-1): §Authority + AC-001 bare table_name for both tables: `"claroty_servers"` → `"servers"` and `"claroty_server_interfaces"` → `"server_interfaces"` (derivation notes added — `{sensor_id}_{table_name}` = registered/queryable names `claroty_servers` / `claroty_server_interfaces`); §TOML Column-Block Specification both table_name fields updated to bare form with derivation comments. FIX 2 (MED-3): N/A — no `ColumnMapper::map_record` references found. FIX 3 (MED-4): frontmatter `crates_touched` adds `prism-bin`; RG-003 and RG-011 updated to prism-bin end-to-end authoritative + prism-sensors defense-in-depth (SAP-3 rule 3); RG-017 added (wire-shape serialization assertion via prism-bin QueryEngine::execute); §File Structure Requirements adds CREATE `crates/prism-bin/tests/bc_2_16_018_019_...wire_shape.rs` and MODIFY `crates/prism-bin/Cargo.toml`; density check updated 16→17 RGTs / 16 ACs. FIX 4 (MED-4): §Architecture Mapping `spec_driven_adapter.rs` corrected from `crates/prism-spec-engine/src/` to `crates/prism-bin/src/`. |
| 1.3 | 2026-08-31 | research-agent | PRE-DELIVERY remove-uncertainty pass (D-1110 mandatory second pass, immediately before TDD delivery). Validated all load-bearing claims against ground truth (`xdome_openapi_06.20.2026.json`, endpoint-schema-extract.md, endpoint-spike-findings.md, `crates/prism-dtu-claroty/src/clone.rs`). CONFIRMED CLEAN: two-separate-endpoints (OpenAPI declares distinct top-level paths `/api/v1/servers/` and `/api/v1/server_interfaces/`, operationId `get_servers_api_v1_server_interfaces__post`); all 17 servers + 10 server_interfaces column names match the Server/ServerInterfaces `fields_enum` and appear in the response §examples; response_paths `$.servers`/`$.server_interfaces` match required envelope keys `servers`/`server_interfaces`; OCSF inventory_info/5001 + `device.name`→`device_name` (REQUIRED) + status→`status_code`; `count` is nullable (anyOf integer/null, `include_count` default false) so empty-page-halt ACs are grounded; no Datetime columns in either fields_enum so `timestamp_formats`/SAP-2 datetime arms a/b/c are N/A; SAP-2 DTU-absence confirmed (no `servers`/`server_interfaces` route in prism-dtu-claroty `build_router`). CORRECTIONS: (1) `uptime_days` Float now POSITIVELY CONFIRMED from the `GetServersResponse` §example (`uptime_days = 667.233661`, fractional) — resolved the prior "verify on live before asserting Integer" open uncertainty (risk note, Notes item 5, EC-003 updated); (2) added status-value casing notes to AC-005 §3 and AC-013 §3 — OpenAPI response §examples render status values in lowercase (`"up"`) as synthetic placeholders, so the capitalized value sets are UNCONFIRMED; the `#[ignore]`'d live tests (RG-005/RG-013) MUST assert `status_code` case-insensitively and MUST NOT fail on casing alone (live-validation confirms exact casing). Refreshed stale `input-hash` (ae98e4f→78a00bd; one or more `inputs:` files evolved since authoring). No load-bearing spec content changed (TOML blocks, columns, ColumnTypes, ACs count, RG list, BC set, depends_on unchanged). Story `status` left `draft` per pass scope. No volatile line cites introduced (TD-VSDD-091). |
| 1.2 | 2026-08-24 | story-writer | Sibling-sweep correction (TD-VSDD-060): fixed pipeline-state mischaracterization. §Previous Story Intelligence item 2 label changed from "merged" to "materialized draft, pending; not yet merged/implemented"; PSI text updated to reflect that S-CLAROTY-VULNS-001 [[tables]] block is NOT in committed TOML on develop, and that the TOML pattern reference comes from existing committed tables (e.g., alerts); Token Budget "existing 6 tables" corrected to "existing 4 tables on current develop"; Notes for Implementer item 9 tightened to remove stale "merged" framing now fixed in PSI. No load-bearing spec content (TOML blocks, columns, ColumnTypes, ACs, RG lists, BCs, depends_on) changed. |
| 1.1 | 2026-08-24 | research-agent | Remove-uncertainty pass (D-1110 mandatory post-materialization). Corrected baseline-table-count claims to ground truth: committed `claroty.sensor.toml` on develop@3f1e66179 has 4 tables (`alerts`, `audit_logs`, `devices`, `device_alert_relations`), not the 6 asserted in §Background — none of `claroty_vulnerabilities`/`claroty_ot_activity_events`/`claroty_device_vulnerability_relations` exist in any sensor spec. §Background reframed to ground truth; Task 8 positioning reference to the non-existent `claroty_device_vulnerability_relations` block corrected to the actual last table block; Task 12 total-count check made relative (baseline + 2 rather than a fixed 8); added §Notes for Implementer item 9 flagging the Wave A/B merge-status residual for orchestrator/human. CONFIRMED CLEAN via direct inspection: DTU-absence (no `servers`/`server_interfaces` route in `prism-dtu-claroty` build_router or routes/ — SAP-2 N/A correct); two-separate-endpoints (OpenAPI declares distinct top-level `/api/v1/servers/` and `/api/v1/server_interfaces/` paths, operationId `get_servers_api_v1_server_interfaces__post`); all 17 servers + 10 server_interfaces column names match the Server/ServerInterfaces enums in endpoint-schema-extract.md; response_paths `$.servers`/`$.server_interfaces` match envelope keys; OCSF class_uid 5001 + `device.name`→`device_name` (REQUIRED) + status→`status_code` mapping correct. No volatile line cites introduced (TD-VSDD-091). |
| 1.0 | 2026-08-24 | story-writer | Initial authoring — F3 story materialization for S-CLAROTY-SERVERS-001 (Wave C G4). BC-2.16.018 v1.0 + BC-2.16.019 v1.0 traceability; claroty_servers 17-column Tier-1/Tier-2 spec (2 Tier-1: device_name REQUIRED [server_name→device.name] + status_code [server_status→status_code]; 15 Tier-2 into raw_extensions); claroty_server_interfaces 10-column Tier-1/Tier-2 spec (2 Tier-1: device_name REQUIRED [server_name→device.name] + status_code [interface_status→status_code]; 8 Tier-2 into raw_extensions incl. composite PK element interface_name); composite PK (server_name + interface_name) per BC-2.16.019 §PC3; 16 ACs; 16 RGTs; density 1.0; SAC-1 compliant; SAC-2 N/A (no ADR authored by this story); SAP-2 N/A per D-2200 for both tables; live-test approach per xdome-endpoint-expansion-plan.md §Per-Story Pipeline; TOML column-block specs embedded per both BCs; HS-027 holdout gate BLOCKING; depends_on: []. |
