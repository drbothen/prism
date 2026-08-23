---
document_type: holdout-scenario
level: L3
id: "HS-ROUTING-001-B-002"
title: "Spec-load §J4 intra-table duplicate collision rejected with E-SPEC-030 [§J4]; prism exits 2; hot-reload keeps prior spec"
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
  - ".factory/specs/behavioral-contracts/BC-2.16.003-column-to-ocsf-mapping.md"
  - ".factory/specs/architecture/decisions/ADR-058-v1-column-naming-col-name-as-arrow-field-identifier.md"
input-hash: "f82613c"
traces_to: "BC-2.16.003"
behavioral_contracts:
  - BC-2.16.003
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
notes: "HIDDEN, SINGLE-USE story-level holdout re-gate for S-ADR058-OCSF-ROUTING-001 (HS-023 group — A+W amendment coverage; HS-022 consumed at D-2270). Tests BC-2.16.003 EC-016-013-032 §J7 spec-load collision validation: §J4 intra-table duplicate (two Tier-1 columns flattening to the same Arrow name) causes E-SPEC-030 [§J4] rejection at spec-load; boot ConfigInvalid → exit 2; hot-reload keeps prior spec. NOT covered by HS-022. BLOCKING. Test-writer and implementer must NOT read this file."
---

# HS-ROUTING-001-B-002: Spec-load §J4 intra-table duplicate collision rejected with E-SPEC-030 [§J4]; prism exits 2; hot-reload keeps prior spec

> **WARNING:** This file is stored in `.factory/holdout-scenarios/` and must
> NEVER be shown to the implementer or test-writer agents. The information
> asymmetry between builder and evaluator is the core quality mechanism.

**Story:** S-ADR058-OCSF-ROUTING-001 (HS-023 re-gate group — A+W amendment + §J7 collision detection coverage; HS-022 consumed at D-2270)
**Must Pass:** YES (P0 — blocks story merge)
**BC Traced:** BC-2.16.003 EC-016-013-032 (§J7 spec-load collision validation: `parse_and_validate_spec_toml` via `validate_ocsf_column_collisions` Validation Rule 8 rejects §J4 intra-table duplicate with E-SPEC-030 [§J4]; boot ConfigInvalid → exit 2; hot-reload keeps prior spec; runtime `pipeline_result_to_record_batch` §J guard remains as defense-in-depth); ADR-058 §J7
**Gate:** Story-level holdout re-gate (HS-023) — runs after LOCAL 3-CLEAN convergence at code @8aeaf06c4, before demo recording and PR push. SINGLE-USE. HIDDEN from test-writer and implementer.

---

## Scenario

This scenario validates the §J7 spec-load §J collision validation behavior introduced by ADR-058 §J7 / BC-2.16.003 EC-016-013-032 — new behavior that HS-022 did NOT cover. When a sensor TOML with `ocsf_column_naming = true` is loaded and two Tier-1 columns in the same table produce the same Arrow field name after applying `ocsf_field_to_arrow_name`, this is a **§J4 intra-table duplicate** — a collision that would cause an `ArrowError::SchemaError` at runtime. The spec must be REJECTED at load time (fail-fast, fail-closed) rather than accepted and failing later at query time.

**The §J4 intra-table duplicate:** Two `ocsf_field` values that produce the same Arrow name after dot→underscore flattening constitute a §J4 collision. Example from this scenario: a TOML with `ocsf_field = "src.ip"` (Arrow name: `src_ip`) and `ocsf_field = "src_ip"` (Arrow name: `src_ip`) on two different columns in the same table. Both flatten to `src_ip` — the collision is unambiguous.

**Spec-load rejection behavior:**
- Boot path: prism exits with exit code 2 (ConfigInvalid) and stderr contains `"E-SPEC-030"` and `"[§J4]"`. No MCP I/O is produced.
- Hot-reload path: prism starts successfully with a valid sensor TOML, then a TOML with a §J4 collision is submitted via the MCP `add_sensor` tool (if available) or config-update path. The MCP response returns an error containing `"E-SPEC-030"` and `"[§J4]"`. The original sensor spec remains active (hot-reload keeps prior spec).

This scenario tests the **boot path** (two separate sub-assertions):
1. prism fails to start with the collision TOML → exit code 2 + E-SPEC-030 [§J4] in stderr.
2. A subsequent prism startup with a valid TOML + the same collision TOML submitted dynamically (if MCP add_sensor is available) → the valid sensor remains queryable; the collision TOML is rejected.

If the MCP dynamic-add path is not available or cannot be exercised without source access, the evaluator tests only the boot path (sub-assertion 1). Sub-assertion 2 is bonus coverage.

**Behavioral assertions:**

**Boot path (primary — P0):**
1. Start prism with a configuration that includes ONLY the collision TOML (provided inline in §Setup Instructions). Prism MUST exit with a non-zero exit code (exit 2 is expected; ConfigInvalid).
2. The stderr output from the failed prism startup MUST contain the string `"E-SPEC-030"`.
3. The stderr output from the failed prism startup MUST contain the string `"[§J4]"` (or `"[S-J4]"` — verify the exact format against the error taxonomy; the discriminator tag for intra-table duplicate is `§J4`).
4. Prism does NOT enter an MCP I/O ready state — no JSON-RPC output on stdout.

**Hot-reload path (secondary — P1 bonus if MCP add_sensor is available):**
5. Start prism with a valid sensor TOML (e.g., a minimal single-column sensor with no collision). Verify prism starts successfully and is MCP-ready.
6. Submit the collision TOML via the MCP `add_sensor` tool (tool name may vary; look for the sensor-management MCP tool). The MCP response MUST be an error containing `"E-SPEC-030"` and `"[§J4]"`.
7. Verify the original valid sensor is still accessible — issue a `query` MCP call against the valid sensor's table. The query should proceed normally (passes E-QUERY-038 plan gate for the original sensor's columns). This confirms hot-reload keeps the prior spec.

**BDD supplement (boot path):**

**Given** a sensor TOML with `ocsf_column_naming = true` and two Tier-1 columns in the same table whose `ocsf_field` values flatten to the same Arrow name (§J4 intra-table duplicate)
**When** prism is started with this TOML as the sensor configuration
**Then** prism exits with exit code 2 (ConfigInvalid)
**And** stderr contains `"E-SPEC-030"` and `"[§J4]"`
**And** prism does NOT reach MCP stdio ready state (no JSON-RPC output)

---

## Setup Instructions

1. Write the following collision sensor TOML to a temp file (e.g., `/tmp/test-j4-collision.sensor.toml`). This TOML has a §J4 intra-table duplicate: column `source_ip` has `ocsf_field = "src.ip"` (flattens to `src_ip`) and column `source_plain_ip` has `ocsf_field = "src_ip"` (no dots; Arrow name is `src_ip` — same as the other column):

```toml
# Synthetic sensor with §J4 intra-table duplicate for holdout scenario HS-ROUTING-001-B-002.
# Column "source_ip" ocsf_field "src.ip" → ocsf_field_to_arrow_name → "src_ip"
# Column "source_plain_ip" ocsf_field "src_ip" → ocsf_field_to_arrow_name → "src_ip"
# Both flatten to "src_ip" — §J4 intra-table duplicate collision.
# Expected: parse_and_validate_spec_toml returns Err with "E-SPEC-030 [§J4]"; boot exits 2.

sensor_id = "test-j4-collision"
name = "Test J4 Collision Sensor"
auth_type = "bearer_static"
base_url = "http://127.0.0.1:19999"
ocsf_column_naming = true
version = "1.0.0"

[[credential_refs]]
name = "bearer_token"

[[tables]]
table_name = "network_events"
ocsf_class = "network_activity"

  [[tables.columns]]
  name = "source_ip"
  column_type = "string"
  ocsf_field = "src.ip"

  [[tables.columns]]
  name = "source_plain_ip"
  column_type = "string"
  ocsf_field = "src_ip"

  [[tables.steps]]
  name = "fetch_network_events"
  method = "GET"
  path_template = "/api/v1/network-events"
  response_path = "$.events"
  variables_produced = []
```

2. Configure prism to load ONLY this collision TOML (no other sensors). The prism configuration must point to the temp file as the single sensor spec.

3. Start prism with this configuration. Capture all stderr output and record the process exit code.

4. Assert: the process exits with a non-zero code (expected: 2 = ConfigInvalid). If prism starts and enters MCP stdio ready state, record as FAIL.

5. Assert: stderr contains `"E-SPEC-030"`. Search the captured stderr for this exact string.

6. Assert: stderr contains `"[§J4]"` (with the section-sign character `§` — Unicode U+00A7). If the error tag uses a different format (e.g., `[J4]` without the section sign), search for the pattern that matches the E-SPEC-030 [§J4] tag format defined in the story's error taxonomy. The discriminator must indicate the intra-table duplicate class.

7. (Bonus — hot-reload path, if MCP add_sensor is available): Write a valid minimal sensor TOML (no collision) to `/tmp/test-valid-minimal.sensor.toml`. Start prism with this valid TOML. Then submit `/tmp/test-j4-collision.sensor.toml` via the MCP `add_sensor` tool call. Assert the MCP response is an error containing `"E-SPEC-030"` and `"[§J4]"`. Assert the original valid sensor remains queryable.

---

## Behavioral Contract Linkage

| BC ID | Clause Tested | Scenario Aspect |
|-------|--------------|-----------------|
| BC-2.16.003 | EC-016-013-032: `parse_and_validate_spec_toml` MUST reject §J4 intra-table duplicate collisions via `validate_ocsf_column_collisions` (Validation Rule 8); error MUST contain E-SPEC-030 + [§J4] tag | Assertions 1–4: prism exits 2; E-SPEC-030 + [§J4] in stderr |
| BC-2.16.003 | EC-016-013-032: boot ConfigInvalid → exit 2; hot-reload keeps prior spec | Boot path: exit 2. Hot-reload path: prior spec remains active |
| ADR-058 §J7 | `validate_ocsf_column_collisions(spec: &SensorSpec) -> Vec<String>` enforces §J4 (intra-table duplicate) — two Tier-1 columns in the same table flatten to same Arrow name | §J4 collision TOML triggers the validator; exit 2 confirms fail-closed |
| ADR-058 §J7 | Runtime `pipeline_result_to_record_batch` §J guard remains as defense-in-depth; spec-load check is additive, not a replacement | Spec-load validation fires at startup; runtime guard remains unchanged |

---

## Verification Approach

1. Build the prism binary from the story branch at commit @8aeaf06c4.
2. Write the collision TOML as specified in §Setup Instructions.
3. Configure prism with the collision TOML as the only sensor spec.
4. Launch prism (without MCP stdio mode if possible, or with it); capture stderr; wait for exit or timeout (up to 15 seconds).
5. Record the exit code. Assert: exit code is non-zero. If exit code is 0 (startup succeeded), record FAIL on "boot rejects collision" dimension.
6. Scan stderr for the string `"E-SPEC-030"`. If absent, record FAIL on "E-SPEC-030 present in stderr" dimension.
7. Scan stderr for the string `"[§J4]"` (or the equivalent tag format used by the implementation). If absent, record FAIL on "§J4 discriminator present" dimension.
8. Confirm no JSON-RPC output on stdout — prism must not have entered MCP I/O mode before exiting.
9. (Bonus) Repeat with hot-reload path as described in §Setup Instructions step 7.

---

## Evaluation Rubric

Rate each dimension 0.0–1.0; take weighted average. Satisfying threshold: ≥ 0.80.

- **Boot exits non-zero on collision TOML** (weight: 0.40): Does prism exit with a non-zero code when loaded with the §J4 collision TOML?
  Full credit (1.0): non-zero exit (specifically exit 2 is the expected ConfigInvalid code; any non-zero code is a PASS on this dimension).
  Zero credit (0.0): exit code 0 (startup succeeded despite the collision) OR prism hangs indefinitely.

- **E-SPEC-030 present in stderr** (weight: 0.30): Does the stderr output contain the string `"E-SPEC-030"`?
  Full credit (1.0): `"E-SPEC-030"` present.
  Zero credit (0.0): `"E-SPEC-030"` absent — spec-load error code not emitted; the collision may be detected but the error is not tagged with the canonical code.

- **§J4 discriminator present in stderr** (weight: 0.20): Does the stderr output contain the §J4 discriminator tag (e.g., `"[§J4]"` or equivalent)?
  Full credit (1.0): §J4 discriminator present.
  Partial credit (0.5): error detected but discriminator is absent or uses a different format (e.g., `"duplicate"` without the canonical `§J4` tag).
  Zero credit (0.0): no mention of the §J4 class in the error output.

- **No MCP I/O on stdout** (weight: 0.10): Does prism exit without producing JSON-RPC output on stdout?
  Full credit (1.0): stdout is empty or contains only non-JSON-RPC lines; prism did not enter ready state.
  Zero credit (0.0): JSON-RPC output on stdout (prism entered ready state despite the collision) — fail-closed violated.

---

## Edge Conditions

- **TOML parse error instead of E-SPEC-030:** If the TOML itself is malformed (TOML parse error before semantic validation), prism may exit with a different error code or message. The evaluator should first verify the TOML is syntactically valid (try parsing it with `cargo test` or a TOML validator). If the TOML is valid but the error is a generic parse error (not E-SPEC-030), record as FAIL on "E-SPEC-030 present" and "§J4 discriminator" dimensions.

- **prism starts but then crashes after the collision:** If prism starts (exit code 0) but crashes before reaching ready state, record as FAIL on "boot rejects collision" dimension. The spec must be rejected at `parse_and_validate_spec_toml` before any downstream processing.

- **Section-sign encoding issue:** The `§` character (U+00A7) must be present in the error string. If the implementation encodes it differently (e.g., `[J4]` without the section sign), record as partial credit on "§J4 discriminator" dimension and note the exact format observed.

---

## Failure Guidance

If this scenario fails, send to the builder (one-line, no scenario specifics):

`"HOLDOUT FAIL: HS-ROUTING-001-B-002 (satisfaction: X.XX) — §J4 spec-load collision not rejected at boot; check validate_ocsf_column_collisions helper (Validation Rule 8) in prism-spec-engine::add_sensor_spec / parse_and_validate_spec_toml; E-SPEC-030 [§J4] must appear in stderr and prism must exit 2 (BC-2.16.003 EC-016-013-032; ADR-058 §J7; AC-021 / RG-Q-013)"`

Do NOT disclose: the specific column names or ocsf_field values in the collision TOML, the sensor_id value, or the exact assertion threshold.

---

## Category: real-world-corpus

This scenario is grounded in the real-world risk of a sensor author accidentally defining two OCSF columns that flatten to the same Arrow field name. Without spec-load rejection, this collision would produce an `ArrowError::SchemaError` at query execution time — a deferred failure that is hard to diagnose (the error appears at query time, not at spec registration time). The §J7 spec-load validation moves this failure to the earliest possible point (TOML load) so the operator can fix the configuration before any query is issued.

The §J4 collision is the intra-table duplicate case: two columns whose `ocsf_field` paths both flatten to the same Arrow name via `ocsf_field_to_arrow_name`. Example: `"src.ip"` and `"src_ip"` both produce Arrow name `"src_ip"` — not obvious to a spec author who may write them in separate sessions. The spec-load check makes this class of misconfiguration immediately visible.

| Field | Description |
|-------|-------------|
| corpus_source | Synthetic collision TOML (provided inline in §Setup Instructions); exercises parse_and_validate_spec_toml + validate_ocsf_column_collisions in prism-spec-engine |
| corpus_size | Single collision sensor with one table containing two conflicting Tier-1 columns |
| known_edge_cases | §J1 shadow collision (Tier-1 Arrow name equals another column's col.name) and §J2 reserved-name collision (Tier-1 flattens to class_uid/category_uid/_sensor/raw_extensions) are distinct defect classes covered by RG-Q-014 and RG-Q-012 respectively; this scenario covers only §J4 |
| false_positive_threshold | Zero: E-SPEC-030 [§J4] on a genuinely colliding TOML is an unambiguous spec-load validation signal |
| false_negative_threshold | Zero: absence of rejection on a colliding TOML means the §J4 guard was not implemented or is not invoked at spec-load time |

**Known-good corpus:** A sensor TOML with `ocsf_column_naming = true` where all `ocsf_field` values produce distinct Arrow names — expected: spec loads successfully (no E-SPEC-030). Tests that the collision check does NOT fire false positives for valid configurations.

**Known-problematic corpus:** A sensor TOML with two Tier-1 columns both mapping to the same Arrow name (this scenario's exact fixture) — expected: E-SPEC-030 [§J4] rejection at spec-load, prism exits 2.

---

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.0 | S-ADR058-OCSF-ROUTING-001-B-HS-023-re-gate | 2026-08-23 | product-owner | Initial authoring. HS-023 re-gate group for S-ADR058-OCSF-ROUTING-001 — §J4 intra-table duplicate collision rejected at spec-load with E-SPEC-030 [§J4]; boot exits 2; hot-reload keeps prior spec. NOT covered by consumed HS-022 group (D-2270). AC-021 / RG-Q-013 surface. SINGLE-USE. |
