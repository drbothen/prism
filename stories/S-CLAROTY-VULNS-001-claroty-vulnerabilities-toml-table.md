---
document_type: story
story_id: S-CLAROTY-VULNS-001
title: "Claroty xDome vulnerabilities table — TOML [[tables]] block, 19-column Tier-1/Tier-2 spec, live structural tests (Wave A G1)"
level: "L4"
wave: xdome-wave-a
epic_id: E-XDOME-EXPANSION
priority: P0
status: ready
# BC status: BC-2.16.015 v1.2 draft (promotes to active on PR merge per POL-14). Pre-TDD remove-uncertainty CLEAN (D-1110, 2nd pass, 2026-08-24); status draft→ready.
producer: story-writer
timestamp: "2026-08-24T00:00:00Z"
version: "1.3"
modified: "2026-08-25"
phase: 3
cycle: v1.0.0-brownfield
inputs:
  - ".factory/specs/behavioral-contracts/BC-2.16.015-claroty-vulnerabilities-table.md"
  - ".factory/objectives/xdome-endpoint-expansion-plan.md"
  - ".factory/objectives/xdome-v1-validation/endpoint-spike-findings.md"
  - ".factory/specs/architecture/decisions/ADR-058-v1-column-naming-col-name-as-arrow-field-identifier.md"
  - "crates/prism-sensors/specs/claroty.sensor.toml"
input-hash: "a325727"
# input-hash: updated 2026-08-25 — compute-input-hash reported a325727 (inputs include BC-2.16.015, updated v1.0→v1.2 by pass-2/pass-3 fix-bursts; claroty.sensor.toml, modified by S-ADR058-OCSF-ROUTING-001 PR #242)
traces_to: "BC-2.16.015"
points: 5
estimated_days: 1
tdd_mode: strict
subsystems: [SS-01, SS-16]
# Subsystem anchor justifications (ARCH-INDEX Subsystem Registry):
#   SS-01 (Sensor Adapters) owns this story's scope because
#     `crates/prism-sensors/specs/claroty.sensor.toml` — the TOML spec file being
#     modified — lives in the prism-sensors crate, which is listed under SS-01 per
#     ARCH-INDEX. The `claroty_vulnerabilities` [[tables]] block is a sensor-adapter
#     configuration artifact, exactly the surface SS-01 governs.
#   SS-16 (Spec Engine) owns this story's scope because
#     `crates/prism-spec-engine/src/spec_parser.rs` must parse the new [[tables]]
#     block without validation error. RG-001 and RG-002 are spec-parser unit tests
#     that exercise SS-16's ColumnSpec and FetchStep deserialization. SS-16 is the
#     canonical owner of prism-spec-engine per ARCH-INDEX Subsystem Registry.
target_module: prism-sensors
crates_touched: [prism-sensors, prism-bin]
# crates_touched (F-VULNS-011 corrected @62f1c6379):
#   prism-sensors: claroty.sensor.toml (new [[tables]] block) + Cargo.toml (test dep) +
#     tests/bc_2_16_015_claroty_vulnerabilities.rs
#     (RG-003b ocsf_projected_column_names proxy; RG-005 live; RG-006/007/008 unit tests)
#   prism-bin: tests/bc_2_16_015_claroty_vulnerabilities_wire_shape.rs
#     (RG-003a QueryEngine::execute e2e gate; RG-004 live Variant-1; RG-004b mock wire-shape) +
#     Cargo.toml + Cargo.lock
#   NOTE: prism-spec-engine has ZERO modified files in the feature diff; RG-001/RG-002
#     are in prism-sensors/tests and call SpecLoader::parse via the public prism-sensors API.
#     subsystems: [SS-01, SS-16] anchors remain correct (subsystem anchors, not a file list)
capabilities:
  - CAP-029
behavioral_contracts:
  - BC-2.16.015
  # BC-2.16.015 v1.2 — Claroty xDome Vulnerability Findings Table: TOML table contract
  # (§Postconditions §1), 19-column Tier-1/Tier-2 classification (§Postconditions §2),
  # PK rationale (§Postconditions §3), SAP-2 DTU parity (§Postconditions §4),
  # EC-016-015-001..006 edge cases. All 8 ACs trace to this BC.
verification_properties: []
holdout_scenarios: []
# holdout_scenarios: PO authors 2–4 hidden SINGLE-USE scenarios during remove-uncertainty
# pass (same touchpoint as remove-uncertainty); scenarios live under the holdout-scenarios
# directory that test-writer and implementer MUST NOT read (contamination control).
# The story-level holdout gate (human-approved 2026-07-13) is BLOCKING before demo/push.
depends_on: []
# depends_on justification: S-ADR058-OCSF-ROUTING-001 (which established
# ocsf_column_naming=true for claroty.sensor.toml) is already MERGED (PR #242,
# develop@3f1e66179). No delivery-time scheduling dependency remains.
blocks: []
acceptance_criteria_count: 8
risk: MEDIUM
# Risk justification:
#   The `id` column uses source_path = "$.id" — an API field outside the fields_enum
#   that must be confirmed present on the live monroe sensor before being relied upon.
#   Mark as optional (no REQUIRED option) until confirmed. Live tests must run against
#   monroe; if not accessible at test-authoring time, RG-004/RG-005 (the live Variant-1
#   tests) are #[ignore]'d until live validation. RG-003a is a non-live plan-time
#   e2e integration test (prism-bin, QueryEngine::execute) and is NOT #[ignore]'d.
#   RG-003b is the spec-parse proxy companion, also NOT #[ignore]'d. SAP-2 DTU-parity probe is deferred per D-2200.
assumption_validations: []
risk_mitigations: []
---

# S-CLAROTY-VULNS-001: Claroty xDome Vulnerabilities Table — TOML Block + Live Structural Tests

## Authority

**BC-2.16.015 v1.2 §Postconditions §1 — TOML Table Contract** governs the exact `[[tables]]`
block structure: `table_name = "claroty_vulnerabilities"`, `ocsf_class = "vulnerability_finding"`,
step name `"fetch_vulnerabilities"`, `path_template = "/api/v1/vulnerabilities/"`,
`response_path = "$.vulnerabilities"`, pagination `type = "offset_limit"` / `page_size = 1000`,
and the 18-field `body_template` (excludes `id` which is NOT in the fields_enum and is captured
via `source_path = "$.id"` only). Read §Postconditions §1 in full before authoring the TOML.

**BC-2.16.015 v1.2 §Postconditions §2 — Tier-1/Tier-2 Column Classification** governs Arrow
field naming under `ocsf_column_naming = true`:
- Tier-1: `name` (`ocsf_field = "finding_info.title"` → Arrow `finding_info_title`, options REQUIRED),
  `description` (`ocsf_field = "message"` → Arrow `message`).
- Tier-2 (17 columns, including `id` via source_path): all aggregate into `raw_extensions`.

**ADR-058 §B2** — Tier-2 columns (those without `ocsf_field`) MUST aggregate into `raw_extensions`
under `ocsf_column_naming = true`. The `name` column's `ocsf_field_to_arrow_name("finding_info.title")`
produces `finding_info_title` per ADR-058 §C underscore-flattening convention.

**ADR-028 §D8-B** — `published_date` column (Datetime type) omits `timestamp_formats`; the
implicit iso8601 default applies (`effective_formats` returns `["iso8601"]`). This is not an error.

**spike-findings §Spike 1** is the authority for the PK decision (`name` over `id`), the column
set (18 fields in fields projection + optional `id` via source_path), and the SAP-2 exclusion
list of 14 deliberately excluded fields. Read §Spike 1 Decision rationale before implementing.

**S-ADR058-OCSF-ROUTING-001** (merged PR #242, develop@3f1e66179) activated `ocsf_column_naming = true`
at the sensor level in `claroty.sensor.toml`. This story builds on that foundation — the
`vulnerability_finding` / class_uid 2002 arm in `class_selector.rs::select_by_class_name`
already exists (no new arm required per spike-findings §Overall Verdict).

---

## Narrative

As a SOC analyst querying Claroty xDome vulnerability data via PrismQL,
I want a `claroty_vulnerabilities` table with OCSF `vulnerability_finding` class,
so that I can query CVE and advisory findings from the xDome sensor with proper
OCSF field routing (`finding_info_title`, `message`) and Tier-2 details
(`cvss_v3_score`, `is_known_exploited`, `affected_devices_count`) available
via `raw_extensions`.

## Background

The xDome sensor currently exposes 4 tables: `alerts`, `audit_logs`, `devices`,
`device_alert_relations`. The `POST /api/v1/vulnerabilities/` endpoint (Gap G1) is unaddressed.

This story delivers the complete Wave A G1 addition:
1. **`claroty.sensor.toml`** — add `[[tables]]` block for `claroty_vulnerabilities` (19 columns,
   offset_limit pagination, response_path `$.vulnerabilities`).
2. **Tests** — TOML parse unit test + live structural Variant-1 tests against monroe (wire-level
   JSON assertions). Variant-2 agent test is optional.

**Live-test approach (per xdome-endpoint-expansion-plan.md §Per-Story Pipeline):**

- **Variant-1 (structural, required):** Live `#[ignore]`'d integration tests against the
  monroe sensor. Assertions are wire-level on the serialized JSON response (class_uid, field
  presence). Tests marked `#[ignore]` with comment citing `// LIVE-MONROE-001: requires
  CLAROTY_INSTANCE_URL env var pointing to monroe; run manually or in live-validation CI job`.
- **Variant-2 (agent, optional):** PrismQL agent-level test exercising the full LLM
  agent reasoning path. Deferred to live-validation milestone if not complete before
  story-level holdout gate.
- **DTU note:** SAP-2 DTU-parity probe is deferred for this story delivery per the D-2200
  governing decision (all DTU-parity work PARKED post-first-release; the parity-migration
  story S-ADR058-DTU-PARITY-MIGRATION-001 is parked). Correction of prior note: the DTU
  (`prism-dtu-claroty`) DOES already have a registered `vulnerabilities` route —
  `clone.rs::build_router` wires `POST /api/v1/vulnerabilities` and
  `POST /api/v1/vulnerabilities/:vuln_id/devices`, with handlers in
  `routes/vulnerabilities.rs`. It currently serves a legacy stub envelope
  `{"vulnerabilities": [...], "total": N, "page": 1}` (uses `total`/`page`, not the real
  API's `count`) from a fixture that predates the 19-column `ocsf_column_naming=true`
  contract. Aligning that route/fixture to this table's contract is the parked parity work
  — it is the PARITY that is deferred, not the route's existence.

**Story-level holdout gate:** After LOCAL 3-CLEAN adversary convergence and BEFORE
demo recording / push to origin, the holdout-evaluator runs 2–4 hidden SINGLE-USE
scenarios (authored by PO at remove-uncertainty time; stored under the holdout directory;
contamination-controlled — test-writer and implementer MUST NOT read them). The gate is
BLOCKING: unsatisfied scenarios reset the LOCAL streak per BC-5.39.001.

## Behavioral Contracts

| BC | Title | Version | Role |
|----|-------|---------|------|
| BC-2.16.015 | Claroty xDome Vulnerability Findings Table — Queryable Surface and OCSF vulnerability_finding Mapping | v1.2 | §Postconditions §1 TOML table contract (step, path, body_template, pagination, response_path); §Postconditions §2 Tier-1/Tier-2 classification (2 Tier-1, 17 Tier-2 + source_path id); §Postconditions §3 PK rationale; §Postconditions §4 SAP-2 DTU parity deferred; EC-016-015-001..006 edge cases |

## Acceptance Criteria

### AC-001: TOML block parses without validation error; 19 columns declared; pagination offset_limit 1000 (traces to BC-2.16.015 postcondition 1 — TOML Table Contract)

`crates/prism-sensors/specs/claroty.sensor.toml` declares a `[[tables]]` block with
`table_name = "claroty_vulnerabilities"`, `ocsf_class = "vulnerability_finding"`,
a step named `"fetch_vulnerabilities"` with `method = "POST"`,
`path_template = "/api/v1/vulnerabilities/"`,
`response_path = "$.vulnerabilities"`, pagination `type = "offset_limit"` / `page_size = 1000`,
and `body_template` containing exactly the 18 fields in the fields projection (excludes `id`
which is NOT in the Vulnerability fields_enum).

`SpecLoader::parse` on the modified TOML returns `Ok(SensorSpec)` without validation error.
The parsed spec reports 19 `ColumnSpec` entries for `claroty_vulnerabilities`.

**Test:** `test_BC_2_16_015_claroty_vulnerabilities_toml_block_parses`

### AC-002: Two Tier-1 columns declared with correct ocsf_field; Arrow names are `finding_info_title` and `message` (traces to BC-2.16.015 postcondition 2 — Tier-1 column classification)

The `[[tables.columns]]` block for `name` declares:
- `column_type = "string"`, `ocsf_field = "finding_info.title"`, `options = ["REQUIRED"]`

The `[[tables.columns]]` block for `description` declares:
- `column_type = "string"`, `ocsf_field = "message"`

Under `ocsf_column_naming = true`, `ocsf_field_to_arrow_name("finding_info.title")` = `"finding_info_title"`
and `ocsf_field_to_arrow_name("message")` = `"message"`. Exactly 2 of 19 columns have
a non-None `ocsf_field`. Exactly 17 columns (including `id` via source_path) have no `ocsf_field`.

**Test:** `test_BC_2_16_015_claroty_vulnerabilities_tier1_columns_two_with_ocsf_field`

### AC-003: Tier-2 column query raises E-QUERY-038; `available_columns` contains `raw_extensions` not raw Tier-2 name (traces to BC-2.16.015 postcondition 2 — Tier-2 Tier classification invariant; error case E-QUERY-001)

A PrismQL query `SELECT vulnerability_type FROM claroty.claroty_vulnerabilities LIMIT 1`
raises E-QUERY-038 (column-not-found) at plan time. The error's `available_columns`
MUST contain `raw_extensions`, `finding_info_title`, `message`, `class_uid`, `_sensor`
and MUST NOT contain `vulnerability_type` as a standalone column name.

Same applies for any other Tier-2 column (`cvss_v3_score`, `is_known_exploited`, etc.).

**Primary test (plan-time e2e gate — SAP-3):** `test_BC_2_16_015_claroty_vulnerabilities_e2e_e_query_038_tier2_column`
(`crates/prism-bin/tests/bc_2_16_015_claroty_vulnerabilities_wire_shape.rs` — drives
`SELECT vulnerability_type` through `QueryEngine::execute()` at the public surface
→ E-QUERY-038; this is the SAP-3-compliant arm-reachability test)

**Defense-in-depth proxy:** `test_BC_2_16_015_claroty_vulnerabilities_tier2_column_raises_e_query_038`
(`crates/prism-sensors/tests/bc_2_16_015_claroty_vulnerabilities.rs` — calls
`ocsf_projected_column_names()` directly on the parsed spec; validates that
`vulnerability_type` is absent from the OCSF-projected column set at spec-parse time;
does NOT exercise the full query plan path)

### AC-004: Live Variant-1 wire-shape: `SELECT * LIMIT 1` serialized JSON contains class_uid=2002, finding_info_title, raw_extensions (traces to BC-2.16.015 postcondition 1 class_uid; postcondition 2 Tier-1/Tier-2 wire representation)

Against the live monroe sensor, `SELECT * FROM claroty.claroty_vulnerabilities LIMIT 1`
serialized JSON response (the MCP-visible wire shape per 2026-07-13 wire-shape discipline):
1. `class_uid` key is present with value `2002`
2. `finding_info_title` key is present (non-null for CVE-named vulnerability rows)
3. `raw_extensions` key is present as a JSON object (not null, not absent)
4. `message` key is present

None of `vulnerability_type`, `cvss_v3_score`, `is_known_exploited` etc. appear as standalone
top-level keys in the row (they are inside raw_extensions).

**Test:** `test_BC_2_16_015_claroty_vulnerabilities_live_wire_shape_class_uid_and_tier1`
(`#[ignore]` — requires `CLAROTY_INSTANCE_URL` env var pointing to monroe)

### AC-005: `SELECT raw_extensions LIMIT 5` succeeds; raw_extensions JSON object contains expected Tier-2 keys (traces to BC-2.16.015 postcondition 2 — Tier-2 source columns in raw_extensions)

Against the live monroe sensor, `SELECT raw_extensions FROM claroty.claroty_vulnerabilities LIMIT 5`
returns rows where `raw_extensions` is a non-null JSON object. The deserialized JSON object
contains at minimum `vulnerability_type`, `cvss_v3_score` keys (or null values for those keys)
when the live API returns them. No E-QUERY-038 is raised on `raw_extensions` itself.

**Test:** `test_BC_2_16_015_claroty_vulnerabilities_live_raw_extensions_contains_tier2_keys`
(`#[ignore]` — requires `CLAROTY_INSTANCE_URL` env var pointing to monroe)

### AC-006: Missing REQUIRED `name` field → null row, no hard error, subsequent rows unaffected (traces to BC-2.16.015 invariant — name MUST be present; edge case EC-016-015-001)

The `name` column carries `options = ["REQUIRED"]` in the TOML. When the API response contains
a vulnerability row where `name` is absent or null, the spec-engine produces a null row
(REQUIRED semantics) without raising a hard error. Subsequent rows in the page continue
to be materialized normally.

**Test:** `test_BC_2_16_015_claroty_vulnerabilities_required_name_absent_produces_null_row`
(unit test with mock response payload containing a row missing `name`)

### AC-007: Nullable count envelope: empty-page halt triggers correctly; no error when count is null (traces to BC-2.16.015 postcondition 1 pagination note; edge case EC-016-015-003)

When the `vulnerabilities` response envelope contains `count: null` or omits `count` entirely,
the spec-engine pagination logic uses the empty-page check (halts when returned page is empty),
not a null-pointer dereference on `count`. No error is raised. This matches the existing
behavior for `device_alert_relations` (`count` is also nullable there).

**Test:** `test_BC_2_16_015_claroty_vulnerabilities_nullable_count_uses_empty_page_halt`
(unit test with mock response containing `{"vulnerabilities": [], "count": null}`)

### AC-008: `id` column via source_path is null when absent from API response; does not block pagination (traces to BC-2.16.015 postcondition 2 — optional secondary id identifier; invariant; edge case EC-016-015-002)

The `id` column declares `source_path = "$.id"` (no `ocsf_field`, no `options = ["REQUIRED"]`).
When a vulnerability row in the API response does not include an `id` key in the envelope
(or `id` is null), the column value is null for that row. This does NOT cause a parse error,
does NOT raise E-SPEC-018, and does NOT halt pagination. The `id` value, when present, is
a string (CJYASHKR-format opaque Claroty identifier).

**Test:** `test_BC_2_16_015_claroty_vulnerabilities_source_path_id_null_when_absent`
(unit test with mock response where envelope has no `"id"` key)

## Red Gate Tests

| ID | Test name | Test type | What it gates |
|----|-----------|-----------|---------------|
| RG-001 | `test_BC_2_16_015_claroty_vulnerabilities_toml_block_parses` | Unit (SpecLoader::parse) | AC-001: TOML block parses Ok; 19 column entries returned for claroty_vulnerabilities |
| RG-002 | `test_BC_2_16_015_claroty_vulnerabilities_tier1_columns_two_with_ocsf_field` | Unit (ColumnSpec inspection) | AC-002: exactly 2 Tier-1 columns (ocsf_field == Some); name→finding_info.title REQUIRED; description→message |
| RG-003a | `test_BC_2_16_015_claroty_vulnerabilities_e2e_e_query_038_tier2_column` | Integration — prism-bin, `QueryEngine::execute()` (PRIMARY plan-time gate) | AC-003 PRIMARY: SELECT vulnerability_type drives through `QueryEngine::execute()` at the public surface → E-QUERY-038; `available_columns` includes `raw_extensions`, excludes `vulnerability_type`; SAP-3 arm-reachability gate |
| RG-003b | `test_BC_2_16_015_claroty_vulnerabilities_tier2_column_raises_e_query_038` | Unit proxy — prism-sensors, `ocsf_projected_column_names()` (defense-in-depth) | AC-003 proxy: spec-parse-derived check — `vulnerability_type` absent from OCSF-projected column names; does NOT exercise full query plan; acceptable only as a companion to RG-003a |
| RG-004 | `test_BC_2_16_015_claroty_vulnerabilities_live_wire_shape_class_uid_and_tier1` | Live Variant-1 (`#[ignore]`) | AC-004: wire JSON contains class_uid=2002, finding_info_title present, raw_extensions present, no Tier-2 standalone keys |
| RG-004b | `test_BC_2_16_015_claroty_vulnerabilities_wire_shape_class_uid_2002_mock` | Integration — prism-bin, mock (non-live) | AC-004 non-live coverage: mock wire-shape asserts class_uid=2002, finding_info_title present, raw_extensions present; does not require `CLAROTY_INSTANCE_URL` env var; defense-in-depth companion to live RG-004 |
| RG-005 | `test_BC_2_16_015_claroty_vulnerabilities_live_raw_extensions_contains_tier2_keys` | Live Variant-1 (`#[ignore]`) | AC-005: raw_extensions JSON object contains vulnerability_type, cvss_v3_score keys; no E-QUERY-038 on raw_extensions |
| RG-006 | `test_BC_2_16_015_claroty_vulnerabilities_required_name_absent_produces_null_row` | Unit (mock response) | AC-006: row missing name → null row; no hard error; subsequent rows continue |
| RG-007 | `test_BC_2_16_015_claroty_vulnerabilities_nullable_count_uses_empty_page_halt` | Unit (mock response) | AC-007: count=null in envelope → empty-page halt; no error; no null-ptr deref |
| RG-008 | `test_BC_2_16_015_claroty_vulnerabilities_source_path_id_null_when_absent` | Unit (mock response) | AC-008: id absent from envelope → null cell; no error; pagination unaffected |

**BC-5.38.001 density check:** 10 Red Gate tests (RG-001, RG-002, RG-003a, RG-003b, RG-004, RG-004b, RG-005, RG-006, RG-007, RG-008) / 8 acceptance criteria = 1.25 ≥ 0.5 threshold. PASS.

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| `claroty_vulnerabilities` TOML block | `crates/prism-sensors/specs/claroty.sensor.toml` | Static data (TOML spec) |
| TOML parse validation | `crates/prism-spec-engine/src/spec_parser.rs §spec_parser` | Pure (TOML deserialization; no I/O) |
| Tier-1/Tier-2 Arrow schema computation | `crates/prism-spec-engine/src/column_mapping.rs §ocsf_field_to_arrow_name` | Pure (string transformation; no I/O) |
| OffsetLimit POST-body injection | `crates/prism-spec-engine/src/pipeline.rs §PipelineExecutor::execute` | Effectful (HTTP POST to xDome; merges offset/limit into body_template) |
| response_path extraction | `crates/prism-bin/src/spec_driven_adapter.rs §pipeline_result_to_record_batch` | Effectful (processes HTTP response; builds Arrow RecordBatch) |
| `vulnerability_finding` class arm | `crates/prism-ocsf/src/class_selector.rs::select_by_class_name` | Pure (constant → u32 lookup; arm already exists) |

Architecture section references:
- `architecture/module-decomposition.md` §SS-01 Sensor Adapters (prism-sensors; claroty.sensor.toml)
- `architecture/module-decomposition.md` §SS-16 Spec Engine (prism-spec-engine; spec_parser, pipeline, column_mapping)
- ADR-058 §B2 (Tier-2 raw_extensions aggregation), §C (Arrow field naming), §D (ocsf_column_naming per-sensor flag)
- ADR-028 §D8-B (implicit iso8601 default for datetime without timestamp_formats)

## Purity Classification

- **Pure functions (no I/O, deterministic):** `SpecLoader::parse` (TOML deserialization);
  `ocsf_field_to_arrow_name` (string → string, deterministic); `select_by_class_name("vulnerability_finding")`
  (constant lookup, returns 2002); RG-001/RG-002 TOML parse + column inspection assertions.
- **Effectful functions (I/O, network):** `PipelineExecutor::execute` (HTTP POST to
  `/api/v1/vulnerabilities/`; pagination loop); `pipeline_result_to_record_batch` (HTTP response
  to Arrow RecordBatch); RG-004/RG-005 live integration tests (require running monroe sensor).

---

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Row where `name` is absent (REQUIRED column) | Null row produced per spec-engine REQUIRED semantics; no hard error; pagination continues (EC-016-015-001) |
| EC-002 | `id` absent from API envelope (outside fields projection) | `id` cell is null; no error; does not halt pagination; SAP-2 probe deferred (EC-016-015-002) |
| EC-003 | `count` field is null or absent in response envelope | Pagination halts on empty page; no null-deref; consistent with device_alert_relations pattern (EC-016-015-003) |
| EC-004 | CVE ID format varies (`CVE-YYYY-NNNNN` vs advisory title `ICSMA-21-161-01 (ZOLL...)`) | Preserved as-is in `finding_info_title`; no normalization; any valid string is a valid title (EC-016-015-004) |
| EC-005 | `cve_ids` field is an empty array `[]` | Serialized as `"[]"` JSON string in `raw_extensions`; not null; consistent with existing Json column behavior (EC-016-015-005) |
| EC-006 | `published_date` is null for a vulnerability row | Null Datetime cell; ADR-028 §D8-B null-passthrough; no E-SPEC-018 raised (EC-016-015-006) |
| EC-007 | `published_date` is a non-ISO-8601 string | E-SPEC-018 TimestampParseFailure — null demoted with warning; row continues; no pagination halt |
| EC-008 | API returns non-200 HTTP for POST /api/v1/vulnerabilities/ | E-SENSOR-001 structured error; sensor=claroty, status, body excerpt; previously fetched pages remain valid |
| EC-009 | `SELECT id FROM claroty.claroty_vulnerabilities` | E-QUERY-038 — `id` is captured via source_path into raw_extensions; not a standalone Arrow column (Tier-2, not Tier-1) |

## Token Budget Estimate

| Item | Estimated tokens |
|------|-----------------|
| This story spec | ~7,000 |
| `crates/prism-sensors/specs/claroty.sensor.toml` (existing 4 tables as pattern reference) | ~5,500 |
| BC-2.16.015 v1.2 (full) | ~5,000 |
| ADR-058 §B2/§C/§D sections (ocsf_column_naming flag mechanism) | ~4,000 |
| spike-findings §Spike 1 (PK decision, column set) | ~2,000 |
| prism-spec-engine/src/spec_parser.rs (ColumnSpec + FetchStep section) | ~3,000 |
| prism-spec-engine/src/column_mapping.rs (ocsf_field_to_arrow_name) | ~1,500 |
| Test files (10 RGTs; unit + live integration + mock wire-shape) | ~7,000 |
| ADR-028 §D8-B (implicit iso8601 default reference) | ~1,000 |
| **Total estimate** | **~36,000 tokens** |

Well within 20-30% of a 200K window. If context is tight, load `claroty.sensor.toml` sections
by reading only the `alerts` table block first as the canonical pattern, then skip to the
pagination section.

## Tasks

- [ ] **Task 1 (Red Gate — test first):** Write RG-001: `test_BC_2_16_015_claroty_vulnerabilities_toml_block_parses` in `crates/prism-spec-engine/src/spec_parser.rs #[cfg(test)] mod tests`. Call `SpecLoader::parse` on `claroty.sensor.toml` (or a test fixture containing the new block). Assert `Ok(SensorSpec)` returned, `claroty_vulnerabilities` table present in `spec.tables`, 19 `ColumnSpec` entries. MUST fail before Task 6 (block not yet in TOML).

- [ ] **Task 2 (Red Gate — test first):** Write RG-002: `test_BC_2_16_015_claroty_vulnerabilities_tier1_columns_two_with_ocsf_field` in same test module. Parse TOML; find `claroty_vulnerabilities` table; assert exactly 2 columns have `ocsf_field == Some(_)`: `name` → `"finding_info.title"` with `options = ["REQUIRED"]`, and `description` → `"message"`. Assert 17 columns have `ocsf_field == None`. MUST fail before Task 6.

- [ ] **Task 3 (Red Gate — test first):** Write RG-006, RG-007, RG-008 — unit tests using mock HTTP responses (no live sensor required). These test: REQUIRED name absent → null row; nullable count → empty-page halt; source_path id absent → null cell. Place in `crates/prism-sensors/tests/bc_2_16_015_claroty_vulnerabilities.rs` or `crates/prism-spec-engine/src/pipeline.rs #[cfg(test)]`. All MUST fail before Tasks 6–7.

- [ ] **Task 4 (Red Gate — test first):** Write RG-003a and RG-003b.

  **RG-003a** (`test_BC_2_16_015_claroty_vulnerabilities_e2e_e_query_038_tier2_column` in `crates/prism-bin/tests/bc_2_16_015_claroty_vulnerabilities_wire_shape.rs`): Drive `SELECT vulnerability_type FROM claroty.claroty_vulnerabilities LIMIT 1` through `QueryEngine::execute()` at the public surface. Assert E-QUERY-038 raised; `available_columns` includes `raw_extensions`, `finding_info_title`, `message`; excludes `vulnerability_type`. This is the PRIMARY plan-time gate and the SAP-3-compliant arm-reachability test. MUST fail before Task 6.

  **RG-003b** (`test_BC_2_16_015_claroty_vulnerabilities_tier2_column_raises_e_query_038` in `crates/prism-sensors/tests/bc_2_16_015_claroty_vulnerabilities.rs`): Call `ocsf_projected_column_names()` directly on the parsed spec for `claroty_vulnerabilities`. Assert that `"vulnerability_type"` is NOT in the projected set. This is a spec-parse-derived proxy / defense-in-depth companion to RG-003a — it does NOT exercise the full query plan and is not a substitute for RG-003a. MUST fail before Task 6.

- [ ] **Task 5 (Red Gate — test first):** Write RG-004 and RG-005 — live Variant-1 `#[ignore]`'d integration tests in `crates/prism-sensors/tests/bc_2_16_015_claroty_vulnerabilities.rs`. Each test has a comment: `// LIVE-MONROE-001: requires CLAROTY_INSTANCE_URL env var pointing to monroe; run manually or in live-validation CI job`. RG-004 asserts wire-level JSON shape (class_uid=2002, finding_info_title, raw_extensions keys). RG-005 asserts raw_extensions contains Tier-2 keys. Both MUST fail when `#[ignore]` is removed if the TOML block is absent.

- [ ] **Task 6 (Implementation — TOML block):** Add the `[[tables]]` block to `crates/prism-sensors/specs/claroty.sensor.toml`. Follow the exact structure from BC-2.16.015 §Postconditions §1. Columns in order: 2 Tier-1 first (`name` with REQUIRED, `description`), then 16 Tier-2 with fields projection, then `id` via source_path (no ocsf_field, no REQUIRED, no body_template field). Step: `fetch_vulnerabilities`, POST, path `/api/v1/vulnerabilities/`, body_template with 18-field projection (excludes `id`), response_path `$.vulnerabilities`, pagination offset_limit 1000. Include comments per the existing Claroty TOML convention (DTU deferred note, SAP-2 probe deferred note, `id` source_path rationale).

  After editing: run `just iter prism-spec-engine` — RG-001 and RG-002 MUST turn GREEN.

- [ ] **Task 7 (Implementation — verify parse + unit test green):** Run `just iter prism-spec-engine --no-fail-fast`. Confirm RG-001, RG-002, RG-006, RG-007, RG-008 all GREEN. Confirm no existing tests regressed. Run `just iter prism-sensors` to confirm TOML file is valid.

- [ ] **Task 8 (SAP-2 self-check — deferred):** SAP-2 DTU-parity probe is deferred for this delivery per the D-2200 governing decision (all DTU-parity work PARKED post-first-release; S-ADR058-DTU-PARITY-MIGRATION-001 parked). NOTE: a `vulnerabilities` route DOES already exist and is registered in `prism-dtu-claroty` (`clone.rs::build_router` → `routes/vulnerabilities.rs`), but it serves a legacy stub envelope (`total`/`page`, not `count`) whose fixture is not yet aligned to the 19-column `ocsf_column_naming=true` contract. Do NOT attempt to reconcile that route/fixture in this story — the alignment is the parked parity work. Record this as a known gap in the story comment.

- [ ] **Task 9 (SAP-1 self-check):** Confirm no new `tracing::*!(event_type = ...)` emissions are added by this story (TOML-only change + unit tests). If any new emission appears during implementation, add a BC-2.16.002 catalog row per PG-LP11-001.

- [ ] **Task 10 (Final gate):** Run `just check` (full workspace). Confirm all non-`#[ignore]` Red Gate tests pass: RG-001, RG-002, RG-003a, RG-003b, RG-004b, RG-006, RG-007, RG-008. (RG-004 and RG-005 are `#[ignore]`'d live tests — excluded from `just check`.) Confirm no new `unwrap()`/`expect()` on `Result` in production code paths. Confirm `claroty.sensor.toml` has 5 tables total (existing 4 + `claroty_vulnerabilities`). After `just check` passes, hold for story-level holdout gate before pushing to origin.

## Previous Story Intelligence

1. **S-ADR058-OCSF-ROUTING-001 (merged PR #242):** Activated `ocsf_column_naming = true` at the
   sensor level in `claroty.sensor.toml`. The Tier-1/Tier-2 routing mechanism (ADR-058 §B2/§C)
   is already active for all Claroty tables. The `vulnerability_finding` / class_uid 2002 arm
   was confirmed existing in `class_selector.rs::select_by_class_name` (spike-findings §Overall Verdict).
   No new class_selector arm is needed.

2. **S-ADR058-OCSF-COERCION-001 (merged PR #240):** Closed EC-016-013-007/008/009 (coercion
   path fixes). The `claroty_vulnerabilities` columns include `cve_ids` (Json type) and `id`
   (source_path). Verify that the Json column type and source_path mechanism both pass through
   the coercion path without hitting the now-closed bugs.

3. **S-DEMO-CLAROTY-DAR-001 (merged):** Added `device_alert_relations` to claroty.sensor.toml.
   Its response envelope uses `"devices_alerts"` (not `"device_alert_relations"`). For
   `claroty_vulnerabilities`, the response_path is `"$.vulnerabilities"` per BC-2.16.015 §1 —
   no analogous envelope-key divergence. Confirm on live monroe.

4. **S-DEMO-CLAROTY-TRAILING-SLASH-001 (merged):** Established that Claroty paths use trailing
   slash: `/api/v1/vulnerabilities/` (with trailing slash). The existing alerts/devices paths
   confirm this pattern. The BC-specified `path_template = "/api/v1/vulnerabilities/"` is correct.

5. **Existing TOML pattern (claroty.sensor.toml §alerts):** The `alerts` table is the canonical
   TOML pattern to mirror: `[[tables]]` header → `[[tables.columns]]` blocks → `[[tables.steps]]`
   block. Comments should follow the Gap-CL-NNN / DTU-route / body_template rationale style.
   Read the `alerts` block as the primary template before authoring `claroty_vulnerabilities`.

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
  The `vulnerability_finding` OCSF class maps to class_uid 2002 — the existing arm in
  `class_selector.rs::select_by_class_name` is used without modification.

From ADR-028 §D8-B:
- `published_date` column type `datetime` with NO `timestamp_formats` key is valid: `effective_formats`
  returns `["iso8601"]` as the implicit default. This is intentional, NOT a missing-field error.
  Do NOT add `timestamp_formats = ["iso8601"]` unnecessarily.

From spike-findings §Spike 1 §Decision:
- `name` MUST be the canonical PK (REQUIRED option). `id` MUST NOT be marked REQUIRED — it is
  an optional secondary identifier present outside the fields_enum. Do not swap PK to `id`.
- 14 fields are DELIBERATELY excluded from the first-cut column set (listed in BC-2.16.015 §4).
  Do NOT add those fields; do NOT treat their absence as a gap.

## Library & Framework Requirements

| Library | Version | Source |
|---------|---------|--------|
| `prism-spec-engine` | workspace path | `SpecLoader::parse`, `ColumnSpec`, `FetchStep`, `PaginationConfig::OffsetLimit` |
| `prism-ocsf` | workspace path | `class_selector.rs::select_by_class_name("vulnerability_finding")` → 2002 (existing arm — read only) |
| `serde_json` | per workspace Cargo.toml | Mock response construction in unit tests (RG-006/007/008) |
| `tokio` | per workspace Cargo.toml | Async test runtime for live integration tests (RG-004/005) |

Do NOT add new Cargo.toml production dependencies. The TOML spec addition requires no new
crate imports in production code.

## File Structure Requirements

| Action | File path | Notes |
|--------|-----------|-------|
| MODIFY | `crates/prism-sensors/specs/claroty.sensor.toml` | Add `[[tables]]` block for `claroty_vulnerabilities` after the existing `device_alert_relations` block |
| CREATE | `crates/prism-sensors/tests/bc_2_16_015_claroty_vulnerabilities.rs` | RG-003b (proxy: `ocsf_projected_column_names` check), RG-005, RG-006, RG-007, RG-008; `#[ignore]` live tests include `LIVE-MONROE-001` comment |
| CREATE | `crates/prism-bin/tests/bc_2_16_015_claroty_vulnerabilities_wire_shape.rs` | RG-003a (PRIMARY plan-time e2e: `QueryEngine::execute` → E-QUERY-038), RG-004 (`#[ignore]` live Variant-1 wire-shape), RG-004b (mock wire-shape asserting class_uid=2002) |

Files that MUST NOT be modified:
- `crates/prism-ocsf/src/class_selector.rs` — `vulnerability_finding` arm already exists; no changes
- `crates/prism-spec-engine/src/spec_parser.rs` — no production code changes needed; RG-001/RG-002
  may add unit tests in-module if easier, or inline in the test file above
- `crates/prism-dtu-claroty/` — read only (SAP-2 deferred); no production changes

## Forbidden Dependencies

`prism-sensors` MUST NOT gain any new production dependency on `prism-dtu-claroty` (dev-dep only
for future DTU-parity test if needed — not in this story). `prism-spec-engine` MUST NOT gain a
new dependency on `prism-sensors` (direction is prism-sensors → prism-spec-engine, not reverse).

## Notes for Implementer

1. **`id` is NOT in the body_template.** The `id` field is outside the Vulnerability
   fields_enum — it cannot be requested via the `fields` projection. The `body_template` lists
   exactly 18 fields. The `id` column uses `source_path = "$.id"` to extract from the raw
   API response envelope. Do not add `"id"` to the body_template fields list.

2. **SAP-2 DTU-parity probe is DEFERRED.** Do NOT run parity checks against
   `crates/prism-dtu-claroty/src/` in this delivery. The `vulnerabilities` route already
   exists and is registered (`clone.rs::build_router` → `routes/vulnerabilities.rs`), but it
   serves a legacy stub envelope (`total`/`page`, not `count`) with a fixture that predates
   the 19-column `ocsf_column_naming=true` contract. The parked parity-migration story
   (S-ADR058-DTU-PARITY-MIGRATION-001, deferred post-first-release per D-2200) will align
   that route/fixture and run SAP-2 at that time.

3. **14 fields are deliberately excluded.** The 14 fields listed in BC-2.16.015 §Postconditions §4
   (SAP-2 exclusion documentation) are NOT bugs — they are intentional first-cut exclusions.
   Do NOT add them; do NOT file adversarial findings against their absence.

4. **Live tests are `#[ignore]`'d.** RG-004 and RG-005 require the live monroe sensor. Mark
   them `#[ignore]` with comment `// LIVE-MONROE-001: requires CLAROTY_INSTANCE_URL env var`.
   Per SID-1 discipline: since RG-004/005 are `#[ignore]`'d, also add non-ignored unit tests
   that exercise the TOML parse path (RG-001, RG-002) as the non-live coverage.

5. **Holdout gate is BLOCKING.** After LOCAL adversary 3-CLEAN and BEFORE push to origin,
   the holdout-evaluator runs hidden scenarios. Do not push until the gate passes.

---

## References

- BC-2.16.015 v1.2 (draft) — §Postconditions §1 TOML contract; §Postconditions §2 19-column Tier-1/Tier-2; §Postconditions §3 PK rationale; §Postconditions §4 SAP-2 deferred; EC-016-015-001..006
- ADR-058 §B2 — Tier-2 columns aggregate into raw_extensions; §C — underscore-flattened Arrow names; §D — per-sensor ocsf_column_naming flag
- ADR-028 §D8-B — implicit iso8601 default for datetime columns without timestamp_formats
- spike-findings §Spike 1 — PK decision authority (name > id); first-cut 19-column set; source_path id rationale; 14-field exclusion list
- xdome-endpoint-expansion-plan.md §Near-Term Stories, §Per-Story Pipeline — Wave A G1 context, no-DTU live test approach
- `crates/prism-sensors/specs/claroty.sensor.toml §alerts` — canonical TOML block pattern to mirror
- S-ADR058-OCSF-ROUTING-001 (merged PR #242) — activated ocsf_column_naming=true; vulnerability_finding arm confirmed existing

---

## Changelog

| Version | Date | Author | Notes |
|---------|------|--------|-------|
| 1.3 | 2026-08-25 | story-writer | F-VULNS-ANCHOR-001 (story-side): §Architecture Mapping `response_path extraction` row crate corrected `prism-spec-engine` → `prism-bin`. F-VULNS-VER-001: BC-2.16.015 version pins refreshed v1.0→v1.2 at all 6 story-side locations (frontmatter BC comment, frontmatter behavioral_contracts comment, §Authority ×2, §Behavioral Contracts table, §Token Budget, §References). input-hash updated c3934ca→a325727 (BC-2.16.015 v1.2 now in inputs). |
| 1.2 | 2026-08-25 | story-writer + state-manager | F-VULNS-P1-003: RG-003 row reconciled to name the real plan-time e2e test (prism-bin, RG-003a: test_BC_2_16_015_claroty_vulnerabilities_e2e_e_query_038_tier2_column) + proxy defense-in-depth (prism-sensors, RG-003b); RG-004b added for non-live mock wire-shape coverage; RG-list↔test traceability restored (SAC-1); density updated to 10/8 = 1.25. F-VULNS-011 (state-manager): crates_touched synced [prism-sensors, prism-spec-engine]→[prism-sensors, prism-bin] — feature diff @62f1c6379 has zero prism-spec-engine file modifications; RG-001/RG-002 call SpecLoader::parse via prism-sensors public API; prism-bin carries e2e + wire-shape tests (RG-003a, RG-004b). |
| 1.1 | 2026-08-24 | state-manager | Pre-TDD remove-uncertainty gate CLEAN (D-1110, 2nd pass); status draft→ready; TDD delivery opened. |
| 1.0 | 2026-08-24 | story-writer | Initial authoring — F3 story materialization for S-CLAROTY-VULNS-001 (Wave A G1). BC-2.16.015 v1.0 traceability; 19-column Tier-1/Tier-2 spec; 8 ACs; 8 RGTs; density 1.0; SAC-1 compliant; SAC-2 N/A (no ADR authored by this story); SAP-2 deferred per D-2200; live-test approach per xdome-endpoint-expansion-plan.md §Per-Story Pipeline. |
