---
document_type: story
story_id: S-DEMO-CLAROTY-DAR-001
title: "prism-dtu-claroty + claroty.sensor.toml: Add device_alert_relations table — DTU route and TOML spec (closes DTU-EXT-006)"
wave: 5
epic_id: E-DTU-FIDELITY
priority: P1
status: draft
# BC status: BC-2.16.013 active (v1.36 as of 2026-08-11); BC-2.01.013 active.
# S-7.01 gate: behavioral_contracts non-empty; story may be dispatched to ready after PO confirms no new-BC flags.
version: "1.1"
acceptance_criteria_count: 7
level: "L4"
producer: story-writer
timestamp: "2026-08-11T00:00:00Z"
modified: "2026-08-11"
tdd_mode: strict
subsystems: [SS-17, SS-16]
# Subsystem anchor justifications:
#   SS-17 (DTU Clones) owns crates/prism-dtu-claroty; the new device_alert_relations route,
#     ClarotyDeviceAlertRelation struct, GetDeviceAlertsBody, GetDeviceAlertsResponse,
#     and fixture file are all SS-17 DTU work.
#   SS-16 (Spec Engine) is the consumer of the DTU route — BC-2.16.013 §Postconditions §1
#     governs the claroty.sensor.toml spec entry; spec-driven pipeline executes against this DTU.
#     claroty.sensor.toml lives under crates/prism-sensors/specs/ which is SS-16 territory.
crates_touched: [prism-dtu-claroty, prism-sensors]
target_module: prism-dtu-claroty
capabilities: [CAP-001, CAP-029]
behavioral_contracts:
  - BC-2.16.013  # Bundled Sensor Spec Authoring and DTU-Parity Verification — §Postconditions §1
                 # claroty.sensor.toml device_alert_relations entry; §Known Gaps DTU-EXT-006;
                 # INV-HARNESS-ROUTE-PARITY response envelope shape.
                 # BC-2.16.013 v1.36 active as of 2026-08-11. This story closes DTU-EXT-006.
  - BC-2.01.013  # DataSource Trait — the device_alert_relations table is a sensor table;
                 # it must be reachable by the spec-driven pipeline and must enforce bearer auth.
verification_properties:
  - VP-148  # VP-PLUGIN-003 DTU parity — parity verification exercises the claroty spec tables;
            # VP-148 anchor story is PLUGIN-MIGRATION-001-D. This story extends coverage to
            # the device_alert_relations table once parity tests are added.
depends_on:
  - PLUGIN-MIGRATION-001-A  # BehavioralClone trait surface and ClarotyClone scaffolding
                             # (build_router, ClarotyState, NormalizePathLayer) established there.
                             # PR #156 merged to develop.
  - S-DEMO-CLAROTY-AUDIT-DTU-001  # Established the route-handler pattern (check_bearer_auth,
                                   # load_fixture, JSON envelope) and the NormalizePathLayer
                                   # outer-service wrap in clone.rs. PR #167 merged to develop.
blocks: []
# Dependency anchor justifications:
#   depends_on PLUGIN-MIGRATION-001-A: ClarotyClone scaffolding including build_router() and
#   ClarotyState are required for route registration. Already merged.
#   depends_on S-DEMO-CLAROTY-AUDIT-DTU-001: Establishes the handler pattern and confirms
#   NormalizePathLayer is already wrapping the outer service (ADR-031 §D8-b). Already merged.
#   blocks []: no known downstream blockers at registration time; follow-up harness story will
#   depend_on this story once it is created.
points: 5
# Points justification:
#   - Define ClarotyDeviceAlertRelation struct + GetDeviceAlertsBody + GetDeviceAlertsResponse
#     in types.rs: ~1 pt
#   - Implement routes/device_alert_relations.rs handler (load_fixture + bearer auth + pagination): ~1 pt
#   - Register route in clone.rs build_router() + routes/mod.rs pub mod: ~0.5 pts
#   - Create fixtures/device-alert-relations.json (5+ synthetic entries, no real customer data): ~0.5 pts
#   - Red Gate tests (6 tests: route registered, response key, auth, column parity, TOML columns,
#     fields param): ~1.5 pts
#   - TOML spec table block in claroty.sensor.toml (10 columns + steps block): ~0.5 pts
#   Total: 5 points (~1 day of focused TDD work)
estimated_days: 1
risk: LOW
# Risk justification:
#   DTU-only addition plus TOML spec extension. No prism-spec-engine or prism-query changes needed.
#   Route handler pattern is established (mirrors audit_log.rs exactly). TOML pattern is
#   established (mirrors audit_logs table block). The silent-failure risk (wrong response key)
#   is guarded by RG-002 specifically; early red state is guaranteed. No existing routes modified.
assumption_validations: []
risk_mitigations: []
phase: 3
cycle: "v1.0.0-brownfield"
inputs:
  - ".factory/specs/behavioral-contracts/BC-2.16.013-bundled-sensor-spec-dtu-parity.md"
  - "crates/prism-dtu-claroty/src/clone.rs"
  - "crates/prism-sensors/specs/claroty.sensor.toml"
input-hash: "e13bafe"
traces_to:
  - "BC-2.16.013"
  - "BC-2.01.013"
---

# S-DEMO-CLAROTY-DAR-001: Add `device_alert_relations` Table — DTU Route + TOML Spec (closes DTU-EXT-006)

## Authority

BC-2.16.013 §Postconditions §1, claroty.sensor.toml `device_alert_relations` entry (v1.36,
`status: active`) is the primary behavioral authority. Read the full entry before implementing:
`.factory/specs/behavioral-contracts/BC-2.16.013-bundled-sensor-spec-dtu-parity.md` §Postconditions §1.

BC-2.16.013 §Postconditions §1 specifies:
- Endpoint: `POST /api/v1/device_alert_relations/` (trailing-slash form, POST-for-read)
- Response top-level key: `devices_alerts` (NOT the path stem `device_alert_relations`)
- `fields` array: REQUIRED in request body (`GetDeviceAlertsParameters.fields`, `minItems: 1`)
- Pagination: offset/limit, API maximum `limit: 5000`
- Contracted column subset: exactly 10 columns from the 92-value `AlertedDevicesPairs__fields_enum` (10 contracted + 82 excluded = 92)

BC-2.16.013 §Invariants INV-HARNESS-ROUTE-PARITY establishes that once this route lands in the
standalone DTU, `prism-dtu-harness::clones::claroty::router()` MUST also register
`POST /api/v1/device_alert_relations/` — this obligation is tracked in AC-007 and is a
follow-up story, not in-scope for this delivery.

BC-2.16.013 §Known Gaps DTU-EXT-006 is the tracking row for this story. The gap entry reads:
"Story ID unassigned as of 2026-08-11 — see amendment report. Gap closes on story merge to develop."
This story (S-DEMO-CLAROTY-DAR-001) is the assigned story; DTU-EXT-006 closes on merge.

BC-2.01.013 §Postconditions §2 governs bearer-auth enforcement (AC-002). The canonical auth
helper is `routes/devices.rs::check_bearer_auth`.

ADR-031 §D1 governs DTU isolation (`prism-dtu-claroty` must not depend on `prism-spec-engine`,
`prism-sensors`, or `prism-query`). ADR-031 §D2 authorizes synthetic fixture data.
ADR-031 §D8-b governs the trailing-slash normalization: NormalizePathLayer is already wired
in `clone.rs` `start_on()` outer service; the route is registered WITHOUT trailing slash.

CLAUDE.md §SAP-2 is the governing probe for DTU-TOML schema parity. SAP-2 Rule 3 (column in
TOML with no DTU field = P1 CRITICAL) and SAP-2 Rule 6 (emission-site authority) both apply.

BC-2.16.013 `status: active`. BC-2.01.013 `status: active`. ADR-031 `status: accepted`.

---

## Narrative

As a demo operator and MSSP analyst using the Prism MCP server,
I want the Claroty xDome sensor to expose a `device_alert_relations` table,
so that I can query `FROM claroty_device_alert_relations` to investigate the
alert-to-device linkage and retrieve the risk/severity signals that are absent
from the `claroty_alerts` surface (which carries no `severity` field).

## Background

The `claroty_alerts` table exposes the 20-value `Alert__fields_enum` from the xDome API.
That enum contains no `severity` field. Risk signal (`device_risk_score`,
`network_signature_severity`, `network_signature_confidence`, `malicious_ip_severity`)
exists exclusively on `device_alert_relations` rows. This makes `device_alert_relations`
both the alert→device investigation path and the sole prioritization source for the
Claroty sensor surface (BC-2.16.013 §Postconditions §1 table rationale).

As of develop HEAD on 2026-08-11:
- `crates/prism-sensors/specs/claroty.sensor.toml` (268 lines) declares three tables:
  `alerts`, `audit_logs`, `devices`. There is no `device_alert_relations` block.
- `crates/prism-dtu-claroty/src/clone.rs` `build_router()` registers routes for
  `/api/v1/devices`, `/api/v1/alerts`, `/api/v1/audit_log/get`, alert-device and
  vulnerability routes, tag write routes, and DTU control routes. No
  `/api/v1/device_alert_relations/` route exists.
- `crates/prism-dtu-claroty/src/routes/mod.rs` exports: `alerts`, `audit_log`,
  `devices`, `tags`, `vulnerabilities`. No `device_alert_relations` module.

Both the TOML spec entry and the DTU route handler need to be created from scratch.
This story delivers both in the same PR (they must land together to satisfy ADR-028 §D1
URL grounding: the TOML spec entry for a path must have a corresponding DTU route).

## Behavioral Contracts

| BC | Title | Version | Role |
|----|-------|---------|------|
| BC-2.16.013 | Bundled Sensor Spec Authoring and DTU-Parity Verification — 4 Initial Sensors | v1.36 | §Postconditions §1 defines the device_alert_relations contracted surface (endpoint, response key, fields requirement, pagination, 10 columns, SAP-2 exclusion); §Known Gaps DTU-EXT-006 tracks this work; INV-HARNESS-ROUTE-PARITY mandates harness follow-up |
| BC-2.01.013 | DataSource Trait Eliminates Per-Sensor Code Duplication | active | The device_alert_relations table is a sensor table; it must be reachable by the spec-driven pipeline and must enforce bearer auth per §Postconditions §2 |

## Red Gate Tests (SAC-1 — tdd_mode: strict)

| ID | Test name | Test type | What it gates |
|----|-----------|-----------|---------------|
| RG-001 | `test_BC_2_16_013_claroty_dar_dtu_route_registered` | Unit (against DTU HTTP) | AC-001: route returns HTTP 200 (not 404) with valid bearer |
| RG-002 | `test_BC_2_16_013_claroty_dar_dtu_response_key_is_devices_alerts` | Unit (against DTU HTTP) | AC-003: response body top-level key is `devices_alerts`; test MUST fail if key is `device_alert_relations` |
| RG-003 | `test_BC_2_16_013_claroty_dar_dtu_auth_enforced` | Unit (against DTU HTTP) | AC-002: missing bearer returns HTTP 401 with exact error string |
| RG-004 | `test_BC_2_16_013_claroty_dar_dtu_column_parity_ten_fields` | Unit (Rust struct validation) | AC-005: all 10 TOML contracted columns present in `ClarotyDeviceAlertRelation` struct AND present in the fixture wire-emission (SAP-2 Rule 3 + Rule 6) |
| RG-005 | `test_BC_2_16_013_claroty_dar_toml_table_has_ten_columns` | Unit (parse TOML spec) | AC-004: `device_alert_relations` table block in claroty.sensor.toml declares exactly 10 columns matching the contracted subset |
| RG-006 | `test_BC_2_16_013_claroty_dar_dtu_accepts_fields_body_param` | Unit (against DTU HTTP) | AC-004: request body with `fields: [...]` is accepted; DTU processes it without error |

### BC-5.38.001 Density Check

Red Gate test count: **6**. Acceptance criteria count: **7** (AC-007 is a traceability/obligation AC with no Red Gate test, as the harness update is a follow-up story). Testable AC count: **6**.
Density: **6 / 6 = 1.0** — exceeds the 0.5 minimum threshold from BC-5.38.001. All six testable ACs are covered by exactly one named Red Gate test each. No story-split is required.

## Acceptance Criteria

### AC-001: DTU route registration (traces to BC-2.16.013 postcondition §1 URL grounding)

`POST /api/v1/device_alert_relations/` is registered in `ClarotyClone::build_router()` at path
`/api/v1/device_alert_relations` (without trailing slash — NormalizePathLayer on the outer
service strips inbound trailing-slash requests per ADR-031 §D8-b, already wired since
S-DEMO-CLAROTY-TRAILING-SLASH-001). A request to this path with a valid
`Authorization: Bearer` header returns HTTP 200, not HTTP 404.

### AC-002: Bearer auth enforcement (traces to BC-2.01.013 postcondition §2 auth enforcement)

A request to `POST /api/v1/device_alert_relations/` without a valid `Authorization: Bearer`
header returns HTTP 401 with `{"error": "missing or invalid Authorization header", "code": 401}`.
This is the verbatim output of `routes/devices.rs::check_bearer_auth` (POL-24
error_message_template_verbatim). **The Red Gate test `test_BC_2_16_013_claroty_dar_dtu_auth_enforced`
MUST assert the response body contains the exact string `"missing or invalid Authorization header"` —
a substring or regex match is insufficient.**

### AC-003: Response envelope uses `devices_alerts` key, NOT path stem (traces to BC-2.16.013 EC-016-013-009)

The DTU response body top-level key is `devices_alerts` (not `device_alert_relations`).
The corresponding TOML `response_path` in `claroty.sensor.toml` is `$.devices_alerts`.
**This is the critical silent-failure guard:** EC-016-013-009 in BC-2.16.013 documents that
using the path stem as the key causes all relation rows to be silently discarded at normalization
time with no error. The Red Gate test `test_BC_2_16_013_claroty_dar_dtu_response_key_is_devices_alerts`
MUST deserialize the response body and assert:
1. The key `devices_alerts` is present in the top-level JSON object.
2. The key `device_alert_relations` is NOT present in the top-level JSON object.

Both assertions are required; asserting only (1) is insufficient.

### AC-004: TOML spec `device_alert_relations` table block with 9 contracted columns (traces to BC-2.16.013 postcondition §1 contracted column subset)

`crates/prism-sensors/specs/claroty.sensor.toml` declares a `[[tables]]` block with
`table_name = "device_alert_relations"` containing exactly these 10 columns (in any order):

| Column name | TOML column_type | Rationale |
|-------------|-----------------|-----------|
| `device_uid` | `string` | UUID-form device identifier |
| `alert_id` | `string` | Polymorphic ID (EC-016-013-004 pattern; integer normalized to string) |
| `device_alert_detected_time` | `datetime` | Alert detection timestamp; ISO 8601 string from xDome |
| `device_risk_score` | `string` | Risk score (numeric-string or label, same pattern as devices.risk_score) |
| `network_signature_severity` | `string` | Severity label for network signature |
| `network_signature_confidence` | `string` | Confidence label for network signature |
| `malicious_ip_severity` | `string` | Severity label for malicious IP match |
| `alert_note` | `string` | Human-authored note on the alert-device relation |
| `external_ip` | `string` | External IP address associated with the alert |
| `device_alert_status` | `string` | Alert resolution status (e.g. "Unresolved", "Resolved") |

The `[[tables.steps]]` block declares:
- `method = "POST"`, `path_template = "/api/v1/device_alert_relations/"` (trailing slash per ADR-031 §D8-b)
- `body_template = '{"fields": ["device_uid", "alert_id", "device_alert_detected_time", "device_risk_score", "network_signature_severity", "network_signature_confidence", "malicious_ip_severity", "alert_note", "external_ip", "device_alert_status"]}'`
  (required `fields` parameter per `GetDeviceAlertsParameters.fields`, `minItems: 1`)
- `response_path = "$.devices_alerts"`
- `[tables.steps.pagination] type = "offset_limit"`, `page_size = 1000`

### AC-005: DTU struct column parity with TOML spec (traces to BC-2.16.013 postcondition §2 DTU-TOML column parity / SAP-2)

`ClarotyDeviceAlertRelation` in `crates/prism-dtu-claroty/src/types.rs` has struct fields
corresponding to all 10 contracted columns. SAP-2 Rule 6 (emission-site authority) applies:
the fixture JSON emitted by the route handler MUST include all 10 fields as keys (with
`Option<_>` fields present as JSON `null` when not populated, not absent from the object).
A TOML column with no corresponding key in the wire-emission is a P1 CRITICAL (SAP-2 Rule 3).

Recommended struct field types (derived from field semantics and the polymorphic ID pattern):

```
device_uid: String,
alert_id: u32,                       // integer from xDome; TOML string-normalizes via column_type = "string"
device_alert_detected_time: Option<String>,  // ISO 8601 datetime string
device_risk_score: Option<String>,
network_signature_severity: Option<String>,
network_signature_confidence: Option<String>,
malicious_ip_severity: Option<String>,
alert_note: Option<String>,
external_ip: Option<String>,
device_alert_status: String,         // "Unresolved" / "Resolved"; always present
```

The `GetDeviceAlertsResponse` struct uses `devices_alerts: Vec<ClarotyDeviceAlertRelation>`
as its primary field. The route handler serializes the response as
`json!({"devices_alerts": entries, "count": total_u32})` per INV-HARNESS-ROUTE-PARITY
response envelope shape (`count` is optional but MUST be emitted for harness consistency).

### AC-006: DTU-EXT-006 gap closes on merge (traces to BC-2.16.013 §Known Gaps DTU-EXT-006)

On merge to develop: `crates/prism-dtu-claroty/src/clone.rs` `build_router()` registers
`POST /api/v1/device_alert_relations/` and `crates/prism-sensors/specs/claroty.sensor.toml`
declares the matching `device_alert_relations` table block. DTU-EXT-006 in BC-2.16.013
§Known Gaps transitions from "Story ID unassigned" to "CLOSED by S-DEMO-CLAROTY-DAR-001"
(state-manager updates BC-2.16.013 in the post-merge burst).

### AC-007: INV-HARNESS-ROUTE-PARITY obligation is trackable (traces to BC-2.16.013 invariant INV-HARNESS-ROUTE-PARITY)

Once S-DEMO-CLAROTY-DAR-001 merges to develop, `prism-dtu-harness::clones::claroty::router()`
MUST register `POST /api/v1/device_alert_relations/` with a response envelope shape of
`{"devices_alerts": [...], "count": N}` per INV-HARNESS-ROUTE-PARITY `Claroty device_alert_relations`
clause. This AC documents the obligation and requires that a follow-up story (separate from
this delivery, following the `S-DEMO-HARNESS-CLONE-PARITY-001` precedent) be created and
anchored to INV-HARNESS-ROUTE-PARITY before this obligation is considered closed.
This AC closes when the follow-up story ID is registered in STORY-INDEX and
INV-HARNESS-ROUTE-PARITY in BC-2.16.013 names `POST /api/v1/device_alert_relations/`
with this story as the anchor.

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| `ClarotyDeviceAlertRelation` struct | `crates/prism-dtu-claroty/src/types.rs` | Pure (data struct) |
| `GetDeviceAlertsBody` struct | `crates/prism-dtu-claroty/src/types.rs` | Pure (data struct) |
| `GetDeviceAlertsResponse` struct | `crates/prism-dtu-claroty/src/types.rs` | Pure (data struct) |
| `list_device_alert_relations` handler | `crates/prism-dtu-claroty/src/routes/device_alert_relations.rs` | Effectful (HTTP handler, loads fixture) |
| `build_router()` — route registration | `crates/prism-dtu-claroty/src/clone.rs` | Effectful (router mutation) |
| `routes/mod.rs` — module export | `crates/prism-dtu-claroty/src/routes/mod.rs` | Pure (module declaration) |
| `fixtures/device-alert-relations.json` | `crates/prism-dtu-claroty/fixtures/` | Static data |
| `claroty.sensor.toml` — table block | `crates/prism-sensors/specs/claroty.sensor.toml` | Static configuration |

Architecture section references:
- `architecture/module-decomposition.md` §SS-17 DTU Clones (ClarotyClone structure)
- `architecture/dependency-graph.md` §Wave-5 DTU fidelity stories

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Missing `Authorization` header | HTTP 401 `{"error": "missing or invalid Authorization header", "code": 401}` — verbatim `check_bearer_auth` output (POL-24) |
| EC-002 | Empty `Authorization: Bearer ` (no token value) | HTTP 401 — same as EC-001; `check_bearer_auth` treats empty token as missing |
| EC-003 | Request body omits `fields` key | HTTP 200 with full fixture (DTU is permissive — `GetDeviceAlertsBody.fields` is `Vec<String>` defaulting to empty; DTU does not enforce `minItems: 1` server-side; the real API does, but the DTU returns fixture data regardless of field selection) |
| EC-004 | Malformed request body (non-JSON) | HTTP 200 with full fixture (body is `Option<Json<GetDeviceAlertsBody>>` — unrecognized body ignored, fixture returned) |
| EC-005 | Response path used as `device_alert_relations` instead of `devices_alerts` | All rows silently lost at pipeline normalization — this is the EC-016-013-009 silent-failure mode; guarded by RG-002 |
| EC-006 | Route accessed with GET instead of POST | HTTP 405 Method Not Allowed (axum default for unmatched method on registered path) |
| EC-007 | Org-isolation: X-Org-Id header mismatch | HTTP 401 — org-isolation pattern established across all Claroty list endpoints (W3-FIX-SEC-001 / AC-007 of S-DEMO-CLAROTY-AUDIT-DTU-001); apply the same `validate_org_id` pattern |

## Token Budget Estimate

| Item | Estimated tokens |
|------|-----------------|
| This story spec | ~4,000 |
| `crates/prism-dtu-claroty/src/clone.rs` (existing) | ~7,000 |
| `crates/prism-dtu-claroty/src/types.rs` (existing) | ~5,000 |
| `crates/prism-dtu-claroty/src/routes/audit_log.rs` (handler pattern reference) | ~2,000 |
| `crates/prism-dtu-claroty/src/routes/alerts.rs` (body + response pattern) | ~2,500 |
| `crates/prism-dtu-claroty/src/routes/devices.rs` (auth + org-isolation pattern) | ~4,500 |
| `crates/prism-sensors/specs/claroty.sensor.toml` (TOML pattern reference) | ~3,500 |
| BC files (2 BCs: BC-2.16.013, BC-2.01.013) | ~9,000 |
| New files to write (device_alert_relations.rs + fixture + types additions + TOML block) | ~3,500 |
| **Total estimate** | **~41,000 tokens** |

Well within 20-30% of a 200K context window (~40-60K token budget per story). No split needed.

## Tasks

**Red-then-green ordering: test tasks (1-3) MUST precede implementation tasks (4-8) per SAC-1.**

- [ ] **Task 1: Write Red Gate stubs** — Create `crates/prism-dtu-claroty/src/routes/device_alert_relations.rs`
  with `todo!()` handler bodies. Write the 6 Red Gate tests in `#[cfg(test)] mod tests`:
  RG-001 through RG-006. All 6 tests MUST compile and FAIL (red state) before Task 4 begins.
  Self-verify red state: `cargo nextest run -p prism-dtu-claroty --no-fail-fast`. Red Gate
  density check: 6/6 tests failing confirms the minimum density threshold is met.

- [ ] **Task 2: Verify red state** — Confirm all 6 Red Gate tests fail with the expected
  compilation error or assertion failure. Do NOT proceed to Task 4 until red state is confirmed.
  Record the failing test names and failure messages in the PR description.

- [ ] **Task 3: Stub-architect pass** — Ensure `list_device_alert_relations` handler
  signature compiles (returns `todo!()`) so that the test can reach the assertion failure
  rather than a compilation error.

- [ ] **Task 4: Define types** — Add to `crates/prism-dtu-claroty/src/types.rs`:
  - `ClarotyDeviceAlertRelation` struct (10 fields matching contracted columns; see AC-005)
  - `GetDeviceAlertsBody` struct (`fields: Vec<String>`, `offset: Option<u32>`, `limit: Option<u32>`,
    `filter_by: Option<ApiQueryFilter>`, `sort_by: Option<Vec<ApiSortClause>>`, `include_count: Option<bool>`)
  - `GetDeviceAlertsResponse` struct (`devices_alerts: Vec<ClarotyDeviceAlertRelation>`, `count: Option<u32>`)
  All three structs need `#[derive(Debug, Serialize, Deserialize)]` and `#[serde(deny_unknown_fields)]`
  on `GetDeviceAlertsResponse` (to catch phantom key bugs), but NOT on `GetDeviceAlertsBody`
  (DTU request bodies are permissive per existing convention — EC-004).
  All three structs need `#[non_exhaustive]` per CLAUDE.md `#[non_exhaustive]` discipline
  (they are pub types added to a TOML-deserialized / pub-API surface crate). Register each
  symbol in `scripts/check-non-exhaustive-per-symbol.py` `EXPECTED_SYMBOLS`.

- [ ] **Task 5: Create fixture** — Author `crates/prism-dtu-claroty/fixtures/device-alert-relations.json`
  with at least 5 synthetic entries. Each entry MUST include ALL 10 contracted columns as keys
  (with `null` for optional fields that are not populated, not absent from the object) per
  SAP-2 Rule 6 wire-emission authority. Timestamps in ISO 8601 format. `alert_id` as integer.
  No real customer data (ADR-031 §D2).

- [ ] **Task 6: Implement route handler** — Implement `list_device_alert_relations` in
  `crates/prism-dtu-claroty/src/routes/device_alert_relations.rs`. Pattern:
  identical to `routes/audit_log.rs` and `routes/alerts.rs` — call `check_bearer_auth`,
  call `validate_org_id`, load fixture via `prism_dtu_common::load_fixture(env!("CARGO_MANIFEST_DIR"), "device-alert-relations")`,
  return `json!({"devices_alerts": entries, "count": total_u32})`.
  Handler signature (following audit_log.rs pattern):
  `pub async fn list_device_alert_relations(State(state): State<Arc<ClarotyState>>, headers: HeaderMap, _body: Option<Json<GetDeviceAlertsBody>>) -> (StatusCode, Json<Value>)`.

- [ ] **Task 7: Register module and route** — Add `pub mod device_alert_relations;` to
  `crates/prism-dtu-claroty/src/routes/mod.rs`. Add to `ClarotyClone::build_router()` in `clone.rs`:
  `.route("/api/v1/device_alert_relations", post(device_alert_relations::list_device_alert_relations))`
  (WITHOUT trailing slash — NormalizePathLayer handles stripping). Update the
  `use crate::routes::{..., device_alert_relations};` import.

- [ ] **Task 8: Add TOML spec table block** — Add the `[[tables]]` block for `device_alert_relations`
  to `crates/prism-sensors/specs/claroty.sensor.toml` after the `devices` table. The block must
  include all 10 contracted columns with the correct `column_type` values (see AC-004), the
  `[[tables.steps]]` block with `response_path = "$.devices_alerts"` (CRITICAL — not `$.device_alert_relations`),
  and pagination with `type = "offset_limit"`, `page_size = 1000`.

- [ ] **Task 9: Confirm green state** — Run `just iter prism-dtu-claroty` and `just iter prism-sensors`
  to confirm all 6 Red Gate tests pass (green state). Verify SAP-2 self-check: for each of
  the 10 TOML columns, confirm the field exists in `ClarotyDeviceAlertRelation` AND appears
  in the fixture JSON as a key.

- [ ] **Task 10: BC-2.16.002 catalog check** — If any `tracing::*!(event_type = ...)` emission
  is added to the route handler, add a corresponding row to BC-2.16.002 §Postconditions
  Structured Event Catalog in the same commit (SAP-1 discipline). The `audit_log.rs` pattern
  does NOT emit a tracing event for normal responses — follow the same convention.

## Previous Story Intelligence

Closely related to `S-DEMO-CLAROTY-AUDIT-DTU-001` (the most recent Claroty DTU story). Key
patterns to carry forward:

1. **Route handler pattern:** `routes/audit_log.rs` is the canonical template. Call
   `check_bearer_auth` first, then `validate_org_id`, then load fixture, then return JSON
   envelope. Do NOT use `?` propagation in fixture loading (convention: SAFETY allow + expect).

2. **Org-isolation:** `validate_org_id` must be called on `list_device_alert_relations`
   following the W3-FIX-SEC-001 pattern. EC-007 is the edge case. All 6 list endpoints
   in the Claroty DTU use this pattern per AC-007 of S-DEMO-CLAROTY-AUDIT-DTU-001.

3. **Fixture naming convention:** Fixture file name is the hyphenated table name
   (`device-alert-relations.json`), loaded via
   `load_fixture(env!("CARGO_MANIFEST_DIR"), "device-alert-relations")`.

4. **TOML `body_template` includes `fields`:** Unlike `audit_logs` (which has no `fields`
   requirement), `device_alert_relations` REQUIRES the `fields` parameter in the request body
   (`GetDeviceAlertsParameters.fields`, `minItems: 1`). The `body_template` in the TOML spec
   must include the full 10-field projection list (see AC-004).

5. **SAP-2 Rule 6 wire-emission check:** Before any commit, verify that the fixture JSON
   emits ALL 10 contracted columns as top-level keys in each entry object. This is distinct
   from verifying the struct definition — the fixture is the wire-emission.

6. **Non-exhaustive discipline:** New public types in DTU crates require `#[non_exhaustive]`
   and registration in `EXPECTED_SYMBOLS`. Failure will cause CI to fail with a layer-1
   equality-check failure.

7. **`response_path = "$.devices_alerts"` is load-bearing:** This is the most failure-prone
   part of the story. The natural temptation is to use the path stem. Do not. RG-002 tests
   this specifically.

## Purity Classification

| Module | Classification | Justification |
|--------|----------------|---------------|
| `prism-dtu-claroty` | Effectful | Axum HTTP server; in-memory state via `Arc<ClarotyState>`; `load_fixture` reads from disk at startup. Effectful-shell per `architecture/purity-boundary-map.md`. Dev-dependency only. |
| `ClarotyDeviceAlertRelation`, `GetDeviceAlertsBody`, `GetDeviceAlertsResponse` (types) | Pure | Plain data structs; no I/O or side effects; derive-only logic. |
| `device_alert_relations.rs` handler | Effectful | HTTP handler; reads fixture data; produces HTTP response. Effectful-shell. |
| `claroty.sensor.toml` (TOML spec entry) | N/A | Static configuration file; no purity classification applies. |

---

## Architecture Compliance Rules

From `architecture/module-decomposition.md` §SS-17 DTU Clones:

- DTU clones are test infrastructure. They live in `crates/prism-dtu-*` and are NOT part
  of the `prism-bin` binary in production mode.
- Every route handler must call `check_bearer_auth` before processing.
- Fixture data must be synthetic (no real customer data, AD-017, ADR-031 §D2).
- `DtuConfigureBody` uses `#[serde(deny_unknown_fields)]`; route request bodies do NOT.

From `architecture/dependency-graph.md`:

- `prism-dtu-claroty` depends on `prism-dtu-common` for `BehavioralClone`, `load_fixture`,
  `StubConfig`, `FailureMode`. No new dependencies should be added for this story.
- `prism-dtu-claroty` MUST NOT depend on `prism-spec-engine`, `prism-sensors`, or `prism-query`.
- `prism-sensors` MUST NOT depend on `prism-dtu-claroty` (TOML spec changes are one-directional).

## Library & Framework Requirements

| Library | Version | Source |
|---------|---------|--------|
| `axum` | per `Cargo.toml` workspace pin | Route handler, `State`, `Json`, `HeaderMap` |
| `serde` | per `Cargo.toml` workspace pin | `Deserialize`, `Serialize` on new types |
| `serde_json` | per `Cargo.toml` workspace pin | `json!` macro, `Value` |
| `tokio` | per `Cargo.toml` workspace pin | Async test runtime |
| `prism-dtu-common` | workspace path | `load_fixture`, `BehavioralClone`, `StubConfig` |

Do NOT add `reqwest` as a new direct dependency — check `crates/prism-dtu-claroty/Cargo.toml`
first. The existing test pattern in `routes/alerts.rs` uses `reqwest` only if it is already
present in `[dev-dependencies]`.

## File Structure Requirements

| Action | File path | Notes |
|--------|-----------|-------|
| CREATE | `crates/prism-dtu-claroty/src/routes/device_alert_relations.rs` | New route handler module |
| MODIFY | `crates/prism-dtu-claroty/src/routes/mod.rs` | Add `pub mod device_alert_relations;` |
| MODIFY | `crates/prism-dtu-claroty/src/types.rs` | Add `ClarotyDeviceAlertRelation`, `GetDeviceAlertsBody`, `GetDeviceAlertsResponse` |
| MODIFY | `crates/prism-dtu-claroty/src/clone.rs` | Add route + import in `build_router()` |
| CREATE | `crates/prism-dtu-claroty/fixtures/device-alert-relations.json` | 5+ synthetic entries |
| MODIFY | `crates/prism-sensors/specs/claroty.sensor.toml` | Add `[[tables]]` block for `device_alert_relations` |
| MODIFY | `scripts/check-non-exhaustive-per-symbol.py` | Register 3 new `#[non_exhaustive]` symbols |

Files MUST NOT be modified:
- Any file in `crates/prism-spec-engine/` (DTU-only story)
- Any file in `crates/prism-bin/` (DTU-only story)
- Any file in `crates/prism-dtu-harness/` (harness update is a follow-up story, AC-007)
- `.factory/` artifact files (state-manager commits these; implementer does not commit factory files)

## Forbidden Dependencies

`prism-dtu-claroty` MUST NOT gain a dependency on:
- `prism-spec-engine` (build MUST fail if this dep appears — ADR-031 §D1)
- `prism-sensors` (build MUST fail if this dep appears — ADR-031 §D1)
- `prism-query` (build MUST fail if this dep appears — ADR-031 §D1)

`prism-sensors` MUST NOT gain a dependency on:
- `prism-dtu-claroty` (TOML spec changes are unidirectional)

The existing perimeter-violation compile-fail gate at `tests/external/perimeter-violation/`
is the enforcement template if a new gate is warranted.

## Notes for Implementer

1. **Two-crate delivery in one PR.** Both the TOML spec change (`prism-sensors`) and the
   DTU route change (`prism-dtu-claroty`) must land in the same PR. ADR-028 §D1 prohibits
   a TOML spec entry for a path that has no corresponding DTU route. Do not split them.

2. **`response_path = "$.devices_alerts"` is the most important line in this story.**
   The xDome API response key is `devices_alerts`, not `device_alert_relations`. Using the
   path stem causes silent complete data loss (EC-016-013-009). RG-002 specifically guards
   this. Write RG-002 first so that the red state is unambiguous.

3. **SAP-2 self-check before commit.** For each of the 10 TOML columns
   (`device_uid`, `alert_id`, `device_alert_detected_time`, `device_risk_score`,
   `network_signature_severity`, `network_signature_confidence`, `malicious_ip_severity`,
   `alert_note`, `external_ip`, `device_alert_status`), verify:
   (a) A matching field exists in `ClarotyDeviceAlertRelation` (SAP-2 Rule 3),
   (b) The field key is present in the fixture JSON wire-emission (SAP-2 Rule 6 — emission-site authority).

4. **`body_template` must carry the `fields` projection.** Unlike `audit_log/get` (no `fields`
   parameter), this endpoint has `fields: required, minItems: 1`. The TOML `body_template`
   must provide all 10 column names as the `fields` value. The pipeline injects `offset` and
   `limit` separately via OffsetLimit POST-body injection; the `fields` projection is static
   and must be present in the template.

5. **DTU-EXT-006 tracking.** The commit message should reference `DTU-EXT-006` and
   `S-DEMO-CLAROTY-DAR-001`. State-manager updates BC-2.16.013 §Known Gaps in the post-merge
   burst — do not modify BC-2.16.013 in the implementation PR.

6. **Harness follow-up.** After merge, a follow-up story must be created to add
   `POST /api/v1/device_alert_relations/` to `prism-dtu-harness::clones::claroty::router()`.
   The story should mirror the `S-DEMO-HARNESS-CLONE-PARITY-001` pattern. This is NOT in
   scope for S-DEMO-CLAROTY-DAR-001.

---

## References

- BC-2.16.013 v1.36 (ACTIVE) — Bundled Sensor Spec Authoring and DTU-Parity Verification;
  §Postconditions §1 `device_alert_relations` entry; §Known Gaps DTU-EXT-006;
  INV-HARNESS-ROUTE-PARITY response envelope shape; EC-016-013-009 (response key mismatch)
- BC-2.01.013 (ACTIVE) — DataSource Trait; §Postconditions §2 auth enforcement
- ADR-031 §D1 — DTU clone isolation
- ADR-031 §D2 — permitted-divergence #1: synthetic fixture data
- ADR-031 §D8-b — trailing-slash normalization; NormalizePathLayer outer-service wrap
- ADR-028 §D1 — URL grounding from DTU routes
- `crates/prism-dtu-claroty/src/clone.rs` — ClarotyClone `build_router()` existing route registrations
- `crates/prism-dtu-claroty/src/routes/audit_log.rs` — handler pattern (check_bearer_auth + validate_org_id + load_fixture)
- `crates/prism-dtu-claroty/src/routes/alerts.rs` — body + response pattern
- `crates/prism-sensors/specs/claroty.sensor.toml` — existing table blocks (pattern reference)
- S-DEMO-CLAROTY-AUDIT-DTU-001 — preceding Claroty DTU fidelity story (PR #167 merged)
- S-DEMO-HARNESS-CLONE-PARITY-001 — harness parity precedent (PR #180 merged)

---

## Changelog

| Version | Date | Author | Notes |
|---------|------|--------|-------|
| 1.1 | 2026-08-11 | story-writer | Column-count correction: 9 → 10. Added `device_alert_status` (10th contracted column per BC-2.16.013 v1.36 §Postconditions §1 column-list reconciliation). Updated all stale count references across §Authority, Behavioral Contracts table, RG-004/RG-005 test names and descriptions, AC-004 column table + body_template + pagination page_size (100 → 1000 per shipped claroty.sensor.toml), AC-005, Task 4 struct example, Task 5, Task 8, Task 9, Previous Story Intelligence item 4, Notes for Implementer notes 3 and 4. Points justification comment updated. SAC-1 RG count and density ratio unchanged (6/6 = 1.0). Verifies S-DEMO-CLAROTY-HARNESS-DAR-001 carries no column-list content (clean). |
| 1.0 | 2026-08-11 | story-writer | Initial materialization. Closes DTU-EXT-006 registration; anchors INV-HARNESS-ROUTE-PARITY harness obligation to this story ID. Grounded against BC-2.16.013 v1.36 §Postconditions §1 device_alert_relations entry, clone.rs build_router(), routes/audit_log.rs (handler pattern), types.rs, claroty.sensor.toml (existing table blocks). 7 ACs; 6 Red Gate tests; 2 BCs: BC-2.16.013, BC-2.01.013; DTU: YES; tdd_mode: strict. |
