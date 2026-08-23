---
document_type: holdout-scenario
level: L3
id: "HS-ROUTING-001-B-001"
title: "Zero-Tier-1 OCSF table A+W: ocsf.zero_tier1_table WARN emitted once at spec-load; raw_extensions in available set; raw col.name rejected"
category: "behavioral-correctness"
must_pass: true
priority: P0
epic_id: "EPIC-OCSF-ROUTING"
story_source: "S-ADR058-OCSF-ROUTING-001"
version: "1.0"
status: active
used: false
single_use: true
producer: product-owner
timestamp: "2026-08-23T00:00:00Z"
modified: "2026-08-23"
phase: 3
inputs:
  - ".factory/specs/behavioral-contracts/BC-2.11.016-e-query-038-column-not-found.md"
  - ".factory/specs/behavioral-contracts/BC-2.16.002-multi-step-fetch-pipeline.md"
  - ".factory/specs/architecture/decisions/ADR-058-v1-column-naming-col-name-as-arrow-field-identifier.md"
input-hash: "cbf6dae"
traces_to: "BC-2.11.016"
behavioral_contracts:
  - BC-2.11.016
  - BC-2.16.002
verification_properties:
  - VP-017
lifecycle_status: active
introduced: "S-ADR058-OCSF-ROUTING-001"
last_evaluated: null
last_eval_satisfaction: null
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
notes: "HIDDEN, SINGLE-USE story-level holdout re-gate for S-ADR058-OCSF-ROUTING-001 (HS-023 group — A+W amendment coverage; HS-022 consumed at D-2270). Tests BC-2.11.016 EC-11-080 A+W sub-case: zero-Tier-1 OCSF table emits ocsf.zero_tier1_table WARN ONCE at spec-load; plan-gate available set is exactly {class_uid, _sensor, raw_extensions}; raw col.name rejected with E-QUERY-038. Uses synthetic test sensor TOML (provided inline). BLOCKING. Test-writer and implementer must NOT read this file."
---

# HS-ROUTING-001-B-001: Zero-Tier-1 OCSF table A+W: ocsf.zero_tier1_table WARN emitted once at spec-load; raw_extensions in available set; raw col.name rejected

> **WARNING:** This file is stored in `.factory/holdout-scenarios/` and must
> NEVER be shown to the implementer or test-writer agents. The information
> asymmetry between builder and evaluator is the core quality mechanism.

**Story:** S-ADR058-OCSF-ROUTING-001 (HS-023 re-gate group — A+W amendment; HS-022 consumed at D-2270)
**Must Pass:** YES (P0 — blocks story merge)
**BC Traced:** BC-2.11.016 EC-11-080 A+W sub-case (zero Tier-1 + ≥1 Tier-2: available set = [class_uid, _sensor, raw_extensions] + ocsf.zero_tier1_table WARN once at spec-load; raw col.name rejected E-QUERY-038); BC-2.16.002 Canonical Structured Event Catalog `ocsf.zero_tier1_table` row (SAP-1/PG-LP11-001 obligation); ADR-058 §J6 A+W rule (human decision 2026-08-23)
**Gate:** Story-level holdout re-gate (HS-023) — runs after LOCAL 3-CLEAN convergence at code @8aeaf06c4, before demo recording and PR push. SINGLE-USE. HIDDEN from test-writer and implementer.

---

## Scenario

This scenario validates the A+W (Option A+Warning) behavior introduced by the human decision on 2026-08-23 — a new requirement that HS-022 predated and therefore did NOT cover. When `ocsf_column_naming = true` is active for an OCSF sensor table with **zero Tier-1 columns** (no column declares a non-`None` `ocsf_field`) and **at least one Tier-2 column**, the specification mandates:

**A (Keep):** The table loads successfully. `register_sensor` (`prism-query::table_registry`) registers exactly three columns: `class_uid` (Integer), `_sensor` (String), and `raw_extensions` (Json). Tier-2 data is preserved in `raw_extensions`, NOT dropped.

**W (Warning):** A `tracing::warn!` event with `event_type = "ocsf.zero_tier1_table"` MUST be emitted ONCE at spec-load/registration time (emission site: `register_sensor` in `prism-query::table_registry` — the common sensor-table load chokepoint reached by both boot loading and dynamic add-sensor). Fields: `sensor_id`, `table_name`, `tier2_column_count` (integer count of Tier-2 columns). The warning must NOT be emitted per-query or per-record — only once per table at registration.

The scenario uses a synthetic test sensor TOML (provided inline in §Setup Instructions) that has `ocsf_column_naming = true` and a table with two columns, both without `ocsf_field` declarations (both Tier-2). No mock HTTP server is needed because the assertions target spec-load behavior (WARN in stderr) and plan-gate behavior (E-QUERY-038 fires at plan time, before any HTTP fetch).

**Behavioral assertions:**

1. prism starts successfully (exit code 0 within a few seconds; MCP stdio is ready for input).
2. The startup stderr contains a structured log line with `event_type = "ocsf.zero_tier1_table"`, `sensor_id = "test-zero-tier1"` (or the sensor_id value from the provided TOML), `table_name = "events"`, and `tier2_column_count = 2`.
3. The startup stderr contains the `ocsf.zero_tier1_table` event AT MOST ONCE for the `events` table — NOT zero times (absence is a FAIL) and NOT two or more times (repeated emission is a FAIL).
4. A `query` MCP tool call with `SELECT raw_extensions FROM test-zero-tier1.events` does NOT return an E-QUERY-038 error (`raw_extensions` is in the available set for a zero-Tier-1-with-Tier-2 table). The query may return an execution-time error (sensor HTTP endpoint unreachable) — that is acceptable. The absence of E-QUERY-038 is the PASS signal.
5. A `query` MCP tool call with `SELECT severity FROM test-zero-tier1.events` returns an E-QUERY-038 error. The `available_columns` field in the error JSON payload MUST contain `"raw_extensions"`, `"class_uid"`, and `"_sensor"`. The `available_columns` MUST NOT contain the string `"severity"` (the raw col.name — not in the OCSF-mode available set).
6. After the two queries in assertions 4 and 5, the `ocsf.zero_tier1_table` WARN is still present exactly ONCE in stderr (not replicated per-query).

**BDD supplement:**

**Given** prism MCP stdio is started with a test sensor TOML having `ocsf_column_naming = true` and a table `events` with two columns (`severity` and `message`) that both lack `ocsf_field` declarations (both Tier-2)
**Then** the startup stderr contains exactly ONE `ocsf.zero_tier1_table` WARN with `sensor_id = "test-zero-tier1"`, `table_name = "events"`, and `tier2_column_count = 2`
**When** `SELECT raw_extensions FROM test-zero-tier1.events` is issued via the MCP `query` tool
**Then** the response is NOT an E-QUERY-038 column-not-found error (plan gate accepts `raw_extensions`)
**When** `SELECT severity FROM test-zero-tier1.events` is issued via the MCP `query` tool
**Then** the response is an E-QUERY-038 error whose `available_columns` payload contains `"raw_extensions"`, `"class_uid"`, and `"_sensor"` but NOT `"severity"`
**And** the `ocsf.zero_tier1_table` WARN in stderr remains exactly ONE occurrence (not per-query)

---

## Setup Instructions

1. Write the following sensor TOML to a temp file (e.g., `/tmp/test-zero-tier1.sensor.toml`):

```toml
# Synthetic test sensor for holdout scenario HS-ROUTING-001-B-001.
# Zero Tier-1 columns: both columns have no ocsf_field — both are Tier-2.
# Used to verify ocsf.zero_tier1_table warning and A+W available-set behavior.

sensor_id = "test-zero-tier1"
name = "Test Zero Tier-1 Sensor"
auth_type = "bearer_static"
base_url = "http://127.0.0.1:19999"
ocsf_column_naming = true
version = "1.0.0"

[[credential_refs]]
name = "bearer_token"

[[tables]]
table_name = "events"
ocsf_class = "api_activity"

  [[tables.columns]]
  name = "severity"
  column_type = "string"

  [[tables.columns]]
  name = "message"
  column_type = "string"

  [[tables.steps]]
  name = "fetch_events"
  method = "GET"
  path_template = "/api/v1/events"
  response_path = "$.events"
  variables_produced = []
```

Note: `base_url = "http://127.0.0.1:19999"` deliberately uses an unreachable port. No mock server is needed — the assertions are plan-time (E-QUERY-038) and spec-load-time (WARN), both of which fire before any HTTP fetch attempt.

2. Determine how to configure prism to load a sensor spec from `/tmp/test-zero-tier1.sensor.toml` at startup. Look at the prism binary's config file format and command-line options. The sensor spec directory or explicit TOML path must point to the temp file written in step 1. Do NOT point at the real Claroty sensor — this test requires the synthetic zero-tier1 sensor in isolation. A configuration with just the synthetic sensor is sufficient.

3. Start prism in MCP stdio mode. Capture all stderr output. Wait for the MCP server to be ready (expect a startup completion log line or the first JSON-RPC read prompt).

4. First query: issue MCP `query` tool call with `{"sql": "SELECT raw_extensions FROM \"test-zero-tier1\".events"}`. Capture the full response.

5. Second query: issue MCP `query` tool call with `{"sql": "SELECT severity FROM \"test-zero-tier1\".events"}`. Capture the full response.

6. Collect the full stderr output after both queries complete.

---

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.11.016 | EC-11-080 A+W sub-case: zero-Tier-1 + ≥1-Tier-2 table MUST register `raw_extensions` in available set | Assertion 4: `raw_extensions` query does NOT get E-QUERY-038 |
| BC-2.11.016 | EC-11-080 A+W sub-case: raw TOML `col.name` MUST be rejected as-if-absent; `available_columns` MUST be exactly `["_sensor", "class_uid", "raw_extensions"]` | Assertion 5: `severity` query gets E-QUERY-038 with correct available_columns |
| BC-2.16.002 | Canonical Structured Event Catalog `ocsf.zero_tier1_table` row: event_type, sensor_id, table_name, tier2_column_count fields; emission ONCE at spec-load (not per-query) | Assertions 2, 3, 6: WARN present exactly once with correct field values |
| ADR-058 §J6 | A+W rule (human decision 2026-08-23): zero-Tier-1-with-Tier-2 table preserves Tier-2 data via raw_extensions; spec-load WARN emitted | End-to-end path from TOML registration to plan-gate acceptance and warning emission |
| ADR-058 §J6 | Emission site: `register_sensor` (`prism-query::table_registry`) — fires ONCE per offending table at registration | WARN emitted at startup (not per-query after assertions 4 and 5) |

---

## Verification Approach

1. Build the prism binary (`cargo build --release -p prism-bin` or `just build`) from the story branch at commit @8aeaf06c4.
2. Write the synthetic sensor TOML as specified in §Setup Instructions.
3. Configure and start prism with the synthetic sensor in MCP stdio mode. Capture stderr.
4. Assert (assertion 2): Scan captured stderr for a line containing ALL of:
   - `ocsf.zero_tier1_table` (event_type value)
   - `test-zero-tier1` (sensor_id value)
   - `events` (table_name value)
   - `tier2_column_count = 2` (or equivalent structured field format: `tier2_column_count=2`)
   If this line is absent, record FAIL on "WARN emitted at spec-load" dimension.
5. Assert (assertion 3): Count the number of lines in stderr matching `ocsf.zero_tier1_table`. If count is 0 (absent) or ≥2 (per-query emission), record FAIL on "WARN emitted exactly once" dimension.
6. Send first MCP `query` tool call: `SELECT raw_extensions FROM "test-zero-tier1".events`.
7. Assert (assertion 4): The response is NOT an E-QUERY-038 error. Specifically, the response JSON must NOT contain a field with value `"E-QUERY-038"` or an error code corresponding to column-not-found. If the response is any error OTHER than E-QUERY-038 (e.g., sensor unreachable, execution timeout), that is a PASS on this dimension — the plan gate accepted `raw_extensions`.
8. Send second MCP `query` tool call: `SELECT severity FROM "test-zero-tier1".events`.
9. Assert (assertion 5): The response IS an E-QUERY-038 error. Parse the `available_columns` field from the error JSON. Assert:
   - `"raw_extensions"` is present in `available_columns`.
   - `"class_uid"` is present in `available_columns`.
   - `"_sensor"` is present in `available_columns`.
   - `"severity"` is NOT present in `available_columns`.
10. Assert (assertion 6): After both queries, rescan stderr and confirm `ocsf.zero_tier1_table` still appears exactly once (not once-per-query).

---

## Evaluation Rubric

Rate each dimension 0.0–1.0; take weighted average. Satisfying threshold: ≥ 0.75.

- **WARN present at spec-load with correct fields** (weight: 0.35): Does stderr contain an `ocsf.zero_tier1_table` WARN with `sensor_id = "test-zero-tier1"`, `table_name = "events"`, and `tier2_column_count = 2`?
  Full credit (1.0): all three field values present in the WARN line.
  Partial credit (0.5): WARN present but missing `tier2_column_count` field or wrong count.
  Zero credit (0.0): WARN absent entirely from stderr — the A+W obligation was not implemented.

- **WARN emitted exactly once (not per-query)** (weight: 0.20): Is there exactly ONE `ocsf.zero_tier1_table` occurrence in stderr after both queries?
  Full credit (1.0): exactly one occurrence.
  Zero credit (0.0): zero (absent) or two or more (per-query emission — NOT acceptable per ADR-058 §J6).

- **raw_extensions query NOT rejected by E-QUERY-038** (weight: 0.25): Does `SELECT raw_extensions FROM "test-zero-tier1".events` return a non-E-QUERY-038 response?
  Full credit (1.0): response is any error OTHER than E-QUERY-038 (or possibly success if a mock were running; any non-E-QUERY-038 outcome is a PASS).
  Zero credit (0.0): response IS E-QUERY-038 — plan gate incorrectly excludes `raw_extensions` from the zero-Tier-1-with-Tier-2 available set.

- **raw col.name rejected by E-QUERY-038 with correct available_columns** (weight: 0.20): Does `SELECT severity FROM "test-zero-tier1".events` return E-QUERY-038 with `available_columns` containing `raw_extensions`, `class_uid`, `_sensor` but NOT `severity`?
  Full credit (1.0): E-QUERY-038 fires with all three expected names present and `severity` absent.
  Partial credit (0.5): E-QUERY-038 fires but `available_columns` is incomplete (missing one of the three expected names, or incorrectly includes `severity`).
  Zero credit (0.0): E-QUERY-038 does not fire — raw col.name `severity` is treated as queryable.

---

## Edge Conditions

- **Prism fails to start (SETUP-FAILURE):** If prism exits with a non-zero code on startup due to the synthetic TOML (e.g., TOML parse error, missing required field), record as SETUP-FAILURE. Check that the TOML matches the format expected by the version of prism built from the story branch. Do NOT mark as behavioral FAIL.

- **ocsf.zero_tier1_table WARN absent from stderr:** This is a FAIL (not SETUP-FAILURE) because it means the A+W emission obligation from AC-019 (A+W sub-case) / T-31 was not implemented. Record with observation "ocsf.zero_tier1_table WARN absent from stderr at spec-load — T-31 not implemented."

- **Both `raw_extensions` and `severity` trigger E-QUERY-038:** Indicates the entire zero-Tier-1 A+W projection was not implemented — the `raw_extensions` column was not added to the available set. Score zero on both plan-gate dimensions.

- **WARN emitted twice (once at startup, once after first query):** Per-query emission violates ADR-058 §J6 "ONCE per offending table at spec-load time, NOT per-query." Record as FAIL on "WARN emitted exactly once" dimension with observation "ocsf.zero_tier1_table WARN emitted per-query; expected once at spec-load."

---

## Failure Guidance

If this scenario fails, send to the builder (one-line, no scenario specifics):

`"HOLDOUT FAIL: HS-ROUTING-001-B-001 (satisfaction: X.XX) — A+W zero-Tier-1 OCSF table behavior gap; check register_sensor in prism-query::table_registry for ocsf.zero_tier1_table WARN emission (BC-2.16.002 catalog row) and raw_extensions registration in available set (BC-2.11.016 EC-11-080 A+W sub-case; ADR-058 §J6; AC-019 A+W sub-case T-31)"`

Do NOT disclose: the specific column names in the synthetic TOML, the exact assertion threshold, or the sensor_id value used.

---

## Category: real-world-corpus

This scenario is grounded in the ADR-058 §J6 A+W rule (human decision 2026-08-23): a sensor author who sets `ocsf_column_naming = true` on a table without completing the `ocsf_field` declarations for any column produces a zero-Tier-1 table. Without the A+W behavior, this misconfiguration was silently handled (prior to v2.29: `raw_extensions` was absent; prior to v2.31: spec loaded but Tier-2 data was dropped). The A+W rule ensures: (a) data is preserved via `raw_extensions` so analyst queries against `raw_extensions` succeed; (b) a WARN is emitted so the operator knows the configuration is incomplete. A zero-Tier-1 OCSF table is a probable misconfiguration — this scenario verifies both the diagnostic signal and the data preservation behavior.

| Field | Description |
|-------|-------------|
| corpus_source | Synthetic test sensor TOML (provided inline in §Setup Instructions); exercises prism-query::table_registry registration path and E-QUERY-038 plan-gate |
| corpus_size | Single synthetic sensor with one zero-Tier-1-with-Tier-2 table; two Tier-2 columns |
| known_edge_cases | Truly-empty (0 Tier-1, 0 Tier-2) sub-case also fires ocsf.zero_tier1_table WARN with tier2_column_count = 0 and available set = {class_uid, _sensor} only (no raw_extensions); this scenario covers the ≥1-Tier-2 sub-case only |
| false_positive_threshold | Zero: ocsf.zero_tier1_table WARN in stderr at startup is an unambiguous A+W obligation signal |
| false_negative_threshold | Zero: absence of the WARN means T-31 was not implemented; E-QUERY-038 on raw_extensions means raw_extensions was not added to the available set |

**Known-good corpus:** A sensor TOML with `ocsf_column_naming = true` where ALL columns have `ocsf_field` declarations (all Tier-1, zero Tier-2) — expected: no `ocsf.zero_tier1_table` WARN; available set = synthesized columns + all Tier-1 flattened names. Tests that the warning does NOT fire for a correctly configured OCSF table.

**Known-problematic corpus:** A sensor TOML with `ocsf_column_naming = true` and zero Tier-1 columns but ≥1 Tier-2 columns (this scenario's exact fixture) — expected: ocsf.zero_tier1_table WARN fires once; available set = {class_uid, _sensor, raw_extensions}. This is the A+W misconfiguration pattern that the amendment was designed to detect.

---

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.0 | S-ADR058-OCSF-ROUTING-001-B-HS-023-re-gate | 2026-08-23 | product-owner | Initial authoring. HS-023 re-gate group for S-ADR058-OCSF-ROUTING-001 — A+W zero-Tier-1 OCSF table: ocsf.zero_tier1_table WARN emission at spec-load + raw_extensions in available set + raw col.name E-QUERY-038 rejection. NOT covered by consumed HS-022 group (D-2270). AC-019 A+W sub-case / RG-Q-017 surface. SINGLE-USE. |
