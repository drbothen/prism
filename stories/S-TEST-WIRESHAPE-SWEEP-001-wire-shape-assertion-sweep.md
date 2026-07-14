---
document_type: story
story_id: "S-TEST-WIRESHAPE-SWEEP-001"
title: "Retroactive wire-shape assertion sweep — all 14 MCP-visible tool surfaces + 6 resource surfaces"
wave: maintenance
epic_id: maintenance
priority: P1
status: draft
version: "0.20"
spec_version: "v0.20"
level: ops
producer: product-owner
timestamp: "2026-07-13"
modified: "2026-07-14"
input-hash: ""
inputs:
  - crates/prism-mcp/src/server.rs
  - crates/prism-mcp/src/resources.rs
  - crates/prism-mcp/src/tools/prism_describe.rs
  - crates/prism-mcp/src/tools/query.rs
  - crates/prism-mcp/src/tools/sensor_health.rs
  - .factory/specs/behavioral-contracts/BC-2.11.001-query-mcp-tool.md
  - .factory/specs/behavioral-contracts/BC-2.10.007-structured-error-responses.md
origin_finding: "T13 live-audit escapes [C3][H20][H8b] — suite asserts internal structures not wire bytes"
origin_cascade: "AUDIT-COVERAGE-001 D-hardening; D-1715 live-audit run; D-1716 human-approved triage 2026-07-13"
cycle: "v1.0.0-greenfield"
phase: 3
tdd_mode: strict
track: "Platform Engineering"
subsystems: [SS-10, SS-11]
# Subsystem anchor justifications:
#   SS-10 (MCP Interface) owns the 14 LIVE_TOOLS and 6 resources in prism-mcp/src/.
#   SS-11 (Query Execution Engine) co-owns because BC-2.11.001 governs query tool
#   row-shape (with_explicit_nulls) — the root cause of [C3] and [H20] escapes.
#   SS-10 is primary (all surfaces touched are MCP-layer); SS-11 is co-owner for the
#   query tool row-shape assertion subset.
crates_touched:
  - prism-mcp
target_module: "crates/prism-mcp"
behavioral_contracts: [BC-2.11.001, BC-2.10.007]
# BC status: both BCs are active.
#   BC-2.11.001 v1.22 (null-not-absent postcondition codified at v1.16; current pin v1.22): null-not-absent row-shape postcondition
#   added (DEFECT-MCP-ROWSHAPE-NULLS-001); EC-11-079. This is the primary anchor for
#   query tool wire-shape tests.
#   BC-2.10.007 v1.19: structured error response wire shape — all 9 required fields,
#   retry_after_seconds null-not-absent. Governs error-path assertions across all tools.
#   Additional tool-specific BCs (prism_describe, check_sensor_health, resources) govern
#   those surfaces; they are referenced per AC below. S-7.01 gate is satisfied by the
#   two listed BCs; additional BCs may be added when PO amends tool-specific contracts.
verification_properties: []
depends_on: []
blocks: []
points: 8
estimated_days: 2.5
risk: P1
acceptance_criteria_count: 12
red_gate_tests: 20
estimated_passes: "2-3"
holdout_scenarios: []
assumption_validations: []
risk_mitigations: []
---

# S-TEST-WIRESHAPE-SWEEP-001: Retroactive wire-shape assertion sweep — 14 MCP tool surfaces + 6 resource surfaces

## §Origin — T13 live-audit escapes [C3][H20][H8b]

**Cascade:** AUDIT-COVERAGE-001 D-hardening; D-1715 live audit (98 PASS / 8 FAIL)
**Session record:** D-1715 (live audit 2026-07-13); D-1716 (human-approved triage + story authorization)
**Authorization:** human-approved 2026-07-13 (D-1716 triage autonomy grant)

The T13 106-check live audit exposed three escapes that share a common root cause: the test
suite asserts in-memory data structures and composed text strings but NOT the serialized JSON
wire bytes that LLM agents actually receive:

- **[C3]** `| fields` pipe-mode query did not restrict projection. The audit check probed the
  wire-level `rows` array (key presence per projected column) and caught an escape that no
  existing unit test would have detected — the test suite asserted `RecordBatch` row counts, not
  the JSON-serialized envelope.
- **[H20]** `threat_score` column ABSENT from all rows (ADR-051 §D2 NULL output expected). The
  enrich stage produced no column at all instead of a `null`-valued column key. The existing
  enrich tests verified the `InfusionResult` shape, not the serialized `structuredContent.results.rows`
  JSON.
- **[H8b]** bare-col JOIN returned code `UNKNOWN` message `'Internal error; see audit log. See
  audit log for details.'` (doubled suffix) — the composed `content[0].text` string was asserted
  to contain an error code, but no test validated the `structuredContent.error.code` field directly.

Root cause: prism-mcp has 14 live-implemented tools and 6 resources, but their test suites assert
internal data shapes. For an LLM-agent consumer, the wire IS the product. This story adds at least
one wire-shape assertion per surface.

**SID-2 — Implementer discipline: wire-shape assertions for MCP tool and resource responses**

This story establishes SID-2 as a standing implementer discipline to be added to CLAUDE.md
§Standing Adversary Probes & Implementer Disciplines alongside SID-1. The complete definition:

> **SID-2 — Wire-shape assertions for MCP tool and resource responses**
>
> When writing tests for MCP tool or resource responses, the following rules apply:
>
> 1. **Serialize to JSON before asserting** — parse the tool response to JSON
>    (`serde_json::to_value` or equivalent) and assert on the JSON tree, not on
>    Rust struct fields. The wire contract is defined by the serialized JSON, not by
>    the Rust type layout.
> 2. **Multi-field composed text (`content[0].text`)** — when the response includes a
>    `content[0].text` string composed from multiple fields (e.g., error code + message
>    + suggestion), ALSO assert the `structuredContent` or the individual JSON field keys.
>    Asserting only the prose text string passes even when the underlying structured
>    contract is broken.
> 3. **Explicit-null key presence (EC-11-079)** — for each entry in `structuredContent.results.rows`, assert that
>    every projected column key is present in every row. For NULL-valued cells, assert
>    the row has `"column_name": null`, not merely that the row parses without error.
>    The `WriterBuilder.with_explicit_nulls(true)` invariant (BC-2.11.001 v1.22) must be
>    exercised by at least one test.
> 4. **Error-code anchor presence** — for error responses, assert `structuredContent.error.code`
>    matches the expected `E-XXX-NNN` string from the taxonomy. Asserting `isError: true` or
>    `content[0].text` alone is insufficient.
> 5. **Null-not-absent fields** — for optional-but-required-null fields (e.g.,
>    `retry_after_seconds` per BC-2.10.007, `did_you_mean` per BC-2.11.001), assert the key
>    IS present in the JSON with value `null` when appropriate — not merely absent.
>
> Source: T13 live-audit escapes [C3]/[H20]/[H8b] (2026-07-13);
> AUDIT-COVERAGE-001 B/C/D-hardening cascade.

## Narrative

As a Prism test maintainer running the T13 pre-flight audit harness, I want every MCP-visible
tool and resource surface to have at least one test that asserts the SERIALIZED JSON output
shape (wire bytes) — including explicit-null key presence, structured error field completeness,
and composed-string anchor presence in `structuredContent` — so that regressions in
agent-harness-facing serialization are caught by the unit test suite before they reach the live
audit stage.

## Behavioral Contracts

| BC | Title | Version | Relevance |
|----|-------|---------|-----------|
| BC-2.11.001 | `query` MCP Tool Accepts Scoping + PrismQL Query String | v1.22 | Postcondition: row-shape null-not-absent — `WriterBuilder.with_explicit_nulls(true)` required; EC-11-079 (NULL column → key present as `null`). Primary anchor for AC-002. |
| BC-2.10.007 | Structured Error Responses | v1.19 | Postcondition: nested wire shape with 9 required fields; `retry_after_seconds: null` (not absent) for non-rate-limit errors. Primary anchor for AC-003 and all error-path ACs. |

## Acceptance Criteria

### AC-001 — SID-2 codified in CLAUDE.md
(traces to BC-2.11.001 v1.22 invariant — agent-harness schema stability rationale)

The full SID-2 definition (see §Origin above) is added to `CLAUDE.md`
§Standing Adversary Probes & Implementer Disciplines immediately after SID-1, with section
header `### SID-2 — Implementer discipline: wire-shape assertions for MCP tool and resource
responses`. The source line reads: `Source: T13 live-audit escapes [C3]/[H20]/[H8b] (2026-07-13);
AUDIT-COVERAGE-001 B/C/D-hardening cascade.`

No Red Gate test for this AC (doc-only change).

### AC-002 — `query` tool: explicit-null key presence (EC-11-079)
(traces to BC-2.11.001 v1.22 postcondition — row-shape null-not-absent)

`crates/prism-mcp/tests/wire_shape.rs` (new file) adds:

`test_BC_2_11_001_query_tool_explicit_null_key_present` — executes a query against a DTU
fixture that returns at least one row with a NULL-valued column (e.g., `sensor_ip` or
`threat_score`). Asserts on the JSON-serialized response that every row in
`structuredContent.results.rows` contains the column key with value `null`, not key-omission.
Implementation: call the `query` tool handler, serialize the result to `serde_json::Value`,
walk `result["structuredContent"]["results"]["rows"]`, assert each row has the projected column key.

Red Gate: the `WriterBuilder.with_explicit_nulls(true)` fix has landed in DEFECT-MCP-ROWSHAPE-NULLS-001
(pending merge). On a build without that fix this test fails with the column key absent from null rows.

`test_BC_2_11_001_query_tool_enrich_null_column_key_present` — pipe-mode `| enrich` query
where the enrichment UDF returns NULL for some rows. Asserts the enriched column key is
present as `null` in those rows (ADR-051 §D2). Targets T13 [H20] escape.

### AC-003 — `query` tool: error-path BC-2.10.007 wire shape
(traces to BC-2.10.007 v1.19 postcondition — structured error wire shape)

`test_BC_2_10_007_query_error_structured_content_fields` — calls `query` with an invalid
PrismQL string to trigger `E-QUERY-001`. Serializes the error response to JSON and asserts:
- `result["isError"]` == `true`
- `result["structuredContent"]["error"]["code"]` == `"E-QUERY-001"` (error-code anchor)
- All 9 BC-2.10.007 fields present in `structuredContent.error`: `code`, `message`,
  `category`, `retryable`, `retry_after_seconds`, `suggestion`, `source`,
  `original_params_valid`, `upstream_message`
- `retry_after_seconds` == `null` (not absent — null-not-absent invariant)
- `content[0]["text"]` contains the error code string (composed-string anchor, SID-2 step 2)

### AC-004 — `prism_describe` tool: wire-shape assertion
(traces to BC-2.10.007 v1.19 postcondition for error path; success path traces to BC-2.10.001
  once that BC is confirmed; pending PO confirmation)

`test_prism_describe_success_wire_shape` — calls `prism_describe` with a valid client ID.
Serializes to JSON and asserts:
- `structuredContent` is present and contains a `tables` array
- Each table entry has at minimum `name` (string key) and `columns` (array key) present
- `content[0]["text"]` is a non-empty string (composed text produced from the table list)

`test_prism_describe_invalid_client_error_wire_shape` — calls `prism_describe` with an
invalid client ID. Asserts `structuredContent.error.code` == `"E-MCP-001"` (error-code anchor,
SID-2 step 4).

### AC-005 — `check_sensor_health` tool: wire-shape assertion
(traces to BC-2.10.007 v1.19 postcondition for error path)

`test_check_sensor_health_success_wire_shape` — calls `check_sensor_health` with a valid
sensor. Serializes to JSON and asserts `structuredContent` contains a health result with at
minimum `client_id` and `sensors` keys present. This is a key-presence assertion (SID-2 step 1).

`test_check_sensor_health_invalid_client_error_wire_shape` — invalid client_id returns
`structuredContent.error.code` == `"E-MCP-001"` (error-code anchor).

### AC-006 — `explain_query` tool: wire-shape assertion
(traces to BC-2.10.007 v1.19 postcondition)

`test_explain_query_success_wire_shape` — calls `explain_query` with a valid PrismQL string.
Serializes to JSON and asserts `structuredContent` contains at minimum `plan_steps` (array) or
equivalent key; `content[0]["text"]` is non-empty.

`test_explain_query_parse_error_wire_shape` — invalid PrismQL returns `structuredContent.error.code`
== `"E-QUERY-001"` (error-code anchor, SID-2 step 4).

### AC-007 — `list_capabilities` tool: wire-shape assertion
(traces to BC-2.10.007 v1.19 postcondition)

`test_list_capabilities_wire_shape` — calls `list_capabilities`. Serializes to JSON and asserts
`structuredContent` contains a `tools` array where each entry has at minimum `name` (string)
and `available` (bool) keys present. Asserts `content[0]["text"]` is non-empty (composed-string
from the capabilities list, SID-2 step 2).

### AC-008 — Alias family tools wire-shape assertions (create_alias, list_aliases, delete_alias, explain_alias)
(traces to BC-2.10.007 v1.19 postcondition for error paths)

Four tests, one per tool:
- `test_create_alias_wire_shape`: calls `create_alias`; asserts `structuredContent` contains
  an `alias` field on success, or `structuredContent.error.code` on validation failure.
- `test_list_aliases_wire_shape`: calls `list_aliases`; asserts `structuredContent` has an
  `aliases` array key present (may be empty).
- `test_delete_alias_wire_shape`: calls `delete_alias` for a non-existent alias; asserts
  `structuredContent.error.code` is present (error-code anchor).
- `test_explain_alias_wire_shape`: calls `explain_alias` for a non-existent alias; asserts
  `structuredContent.error.code` is present.

### AC-009 — `confirm_action` tool: wire-shape assertion
(traces to BC-2.10.007 v1.19 postcondition)

`test_confirm_action_invalid_token_wire_shape` — calls `confirm_action` with an invalid token.
Asserts `structuredContent.error.code` is present and non-empty (error-code anchor, SID-2 step 4);
`retry_after_seconds` == `null` (null-not-absent invariant, SID-2 step 5).

### AC-010 — Config tools wire-shape assertions (reload_config, add_sensor_spec, list_sensor_specs, validate_config)
(traces to BC-2.10.007 v1.19 postcondition for error paths)

Four tests, one per tool:
- `test_reload_config_wire_shape`: calls `reload_config`; asserts the response has
  `structuredContent` with a success indicator key present.
- `test_add_sensor_spec_invalid_wire_shape`: calls `add_sensor_spec` with an invalid TOML
  body; asserts `structuredContent.error.code` is present (error-code anchor).
- `test_list_sensor_specs_wire_shape`: calls `list_sensor_specs`; asserts `structuredContent`
  has a `specs` array key present (may be empty; key presence asserted regardless).
- `test_validate_config_wire_shape`: calls `validate_config`; asserts `structuredContent`
  has a validation result key present.

### AC-011 — Static resource wire-shape assertions (prism://config/clients, prism://sensors/health, prismql://reference)
(traces to BC-2.10.007 v1.19 postcondition for error paths)

Three tests, one per static resource:
- `test_resource_config_clients_wire_shape`: reads `prism://config/clients`; asserts the
  response body parses as valid JSON containing a top-level `clients` array. Key-presence
  assertion (SID-2 step 1).
- `test_resource_sensors_health_wire_shape`: reads `prism://sensors/health`; asserts the
  response body parses as valid JSON containing at minimum a `sensors` or `last_check`
  key present.
- `test_resource_pql_reference_wire_shape`: reads `prismql://reference`; asserts the
  response MIME type is `text/markdown` and the body contains the string `PrismQL` (non-empty
  reference content anchor).

### AC-012 — Resource template wire-shape assertions (prism://config/clients/{}/sensors, prism://schema/{}/{}, prismql://schema/{})
(traces to BC-2.10.007 v1.19 postcondition for error paths)

Three tests, one per template:
- `test_resource_client_sensors_wire_shape`: reads `prism://config/clients/{valid_id}/sensors`;
  asserts the JSON body has a `sensors` array key present.
- `test_resource_schema_wire_shape`: reads `prism://schema/{valid_sensor}/{valid_table}`;
  asserts JSON body has `columns` array key present.
- `test_resource_pql_schema_wire_shape`: reads `prismql://schema/{valid_client_id}`;
  asserts JSON body has `tables` array key present; each table entry has `name` and
  `columns` keys present (schema-catalog structure per BC-2.10.013).

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| `wire_shape.rs` test module (new) | `crates/prism-mcp/tests/wire_shape.rs` | Effectful (integration tests; drives tool handlers via DTU fixture) |
| `query` tool handler (`PrismServer::query`) | `crates/prism-mcp/src/server.rs` | Effectful (existing; `.with_explicit_nulls(true)` fix landed in DEFECT-MCP-ROWSHAPE-NULLS-001, pending merge). Note: `tools/query.rs` is a documentation facade with no handler code. |
| `prism_describe` handler | `crates/prism-mcp/src/tools/prism_describe.rs` | Effectful (existing) |
| `check_sensor_health` handler | `crates/prism-mcp/src/tools/sensor_health.rs` | Effectful (existing) |
| Resource dispatch | `crates/prism-mcp/src/server.rs` + `resources.rs` | Effectful (existing) |
| CLAUDE.md `§SID-2` addition | `/CLAUDE.md` | Doc (pure — no code change) |

Architecture section references:
- `architecture/module-decomposition.md` §SS-10 MCP Interface
- `architecture/module-decomposition.md` §SS-11 Query Execution Engine

**Anchor justifications (POL-4/POL-5):**
- SS-10 is the primary subsystem because all 14 tools and 6 resources are SS-10 artifacts
  (prism-mcp/src/server.rs, resources.rs, tools/) per the ARCH-INDEX Subsystem Registry.
- SS-11 is co-owner because BC-2.11.001 (query tool null-not-absent) is an SS-11 contract.
- No `depends_on`: wire-shape assertions are additive tests over existing behavior; no
  prerequisite story needed. If DEFECT-MCP-ROWSHAPE-NULLS-001 is in flight concurrently,
  AC-002's Red Gate tests may temporarily fail on develop until that fix merges.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | DTU fixture returns a row where ALL projected columns are non-NULL | Test still exercises the JSON serialization path; every key present with non-null value |
| EC-002 | `check_sensor_health` called with DTU offline | Returns structured error with `code` present (error-code anchor assertion still fires) |
| EC-003 | `list_aliases` called with no aliases configured | `aliases` array key present with value `[]`; key-presence assertion passes even for empty array |
| EC-004 | `prismql://reference` resource read in a cold boot state | Resource content is statically embedded; returns non-empty markdown body |
| EC-005 | Resource read for `prism://sensors/health` before any `check_sensor_health` call | Response body may contain stale/empty snapshot; key-presence assertion still exercises the wire shape |
| EC-006 | Concurrent wire-shape tests sharing the same DTU fixture | Tests must use per-test fixture isolation or read-only fixture access; no fixture mutation |

## Token Budget Estimate

| Item | Lines | Tokens (est.) |
|------|-------|--------------|
| Story spec (this file) | ~300 | ~4,200 |
| BC-2.11.001 v1.22 (query BC with null-not-absent postcondition) | ~160 | ~2,300 |
| BC-2.10.007 v1.19 (structured error BC) | ~120 | ~1,700 |
| crates/prism-mcp/src/server.rs (tool dispatch + resource dispatch sections) | ~400 | ~5,600 |
| crates/prism-mcp/src/tools/ (prism_describe, query, sensor_health) | ~300 | ~4,200 |
| crates/prism-mcp/src/resources.rs | ~200 | ~2,800 |
| New test file (crates/prism-mcp/tests/wire_shape.rs) | ~350 | ~4,900 |
| CLAUDE.md §SID-2 addition | ~40 | ~600 |
| DTU fixture setup code | ~100 | ~1,400 |
| **Total estimate** | | **~27,700 tokens** |

Fits within a 100k-token agent context window (~28%). No split required. If DTU fixture code
is large, split by loading only the tool-handler test utilities, not the full fixture module.

## Tasks

- [ ] Read BC-2.11.001 v1.22 postcondition (null-not-absent) and BC-2.10.007 postcondition (structured error wire shape) before writing any test.
- [ ] Read `crates/prism-mcp/src/server.rs` (LIVE_TOOLS list, dispatch table) and `resources.rs` (build_resource_list, build_resource_template_list) to confirm the 14+6 surface inventory.
- [ ] Add `### SID-2` to CLAUDE.md §Standing Adversary Probes & Implementer Disciplines immediately after SID-1 (full definition from §Origin above). Commit this change in the same PR.
- [ ] Create `crates/prism-mcp/tests/wire_shape.rs` (new file). Add all 20 Red Gate tests (AC-002 through AC-012, counting sub-tests).
- [ ] Run `cargo nextest run -p prism-mcp -E 'test(wire_shape)'` — confirm all tests fail RED before production changes (TDD strict mode per BC-5.38.001).
- [ ] DEFECT-MCP-ROWSHAPE-NULLS-001 has landed the `with_explicit_nulls(true)` fix (pending merge). AC-002's `with_explicit_nulls` tests will be RED on a build without that branch and GREEN after merge. No in-scope production change needed for this story on this point.
- [ ] Implement any missing production changes needed to make tests GREEN (the `with_explicit_nulls(true)` fix is handled by DEFECT-MCP-ROWSHAPE-NULLS-001; ensure that branch is merged before running AC-002 tests in this story's CI).
- [ ] Run `just iter prism-mcp` — full crate test suite GREEN.
- [ ] Run `just check` (full workspace) before declaring done.
- [ ] Update `acceptance_criteria_count`, `red_gate_tests` frontmatter fields if test count changes during implementation.

## Previous Story Intelligence

This is the first story targeting a systematic wire-shape assertion sweep of prism-mcp. Prior
context from the AUDIT-COVERAGE-001 cascade:

- Passes 1–44 of the LOCAL cascade fixed 38 audit-script issues. Three escapes ([C3], [H20],
  [H8b]) were not caught by the unit test suite because tests asserted internal Rust struct
  fields, not JSON wire bytes.
- BC-2.11.001 v1.16 (2026-07-13) added the null-not-absent postcondition for query `rows`
  specifically in response to [C3] and [H20]. This story adds the tests that enforce that
  postcondition via TDD.
- BC-2.10.007 v1.8 already specified the null-not-absent pattern for `retry_after_seconds`; this
  story adds tests that confirm the pattern is enforced at the wire level.
- DEFECT-MCP-ROWSHAPE-NULLS-001 (concurrent worktree) is the PRODUCT defect fix for the
  `with_explicit_nulls(true)` issue. That fix has landed (pending merge). This story is the
  TEST coverage companion. The Red Gate tests from this story will be RED on a build without
  DEFECT-MCP-ROWSHAPE-NULLS-001 merged and GREEN after.
- S-AUDIT-URI-VALIDATION-001 (draft, maintenance epic) targets MCP resource URI validation;
  it does not overlap with the wire-shape assertions in this story.

## Architecture Compliance Rules

- **SID-2 compliance (primary rule for this story):** Every test in `wire_shape.rs` MUST
  serialize the tool response to `serde_json::Value` before asserting (SID-2 step 1). Tests
  that assert on Rust struct fields and not on JSON paths are non-compliant and will be flagged
  by the adversary.
- **`#[non_exhaustive]` discipline:** No new public types are introduced in this story (tests
  only + CLAUDE.md doc). If a test helper struct is introduced, confirm it is in `#[cfg(test)]`
  scope and does not require `#[non_exhaustive]`.
- **No `unwrap()` in test helpers:** Use `expect("…")` with descriptive messages for test
  assertions to produce useful failure output when a field is missing.
- **TD-VSDD-091:** Cite function names (`build_resource_list`, `dispatch_read_resource`,
  `with_explicit_nulls`), NOT `file.rs:NNN` line numbers, in test comments.
- **Forbidden dependencies:** `prism-mcp` MUST NOT gain new dependencies on `prism-query` or
  `prism-sensors` crates. Test-only imports must be gated `#[cfg(test)]`.
- **SAP-1:** No new `event_type =` tracing emissions are expected in this story. If test
  infrastructure adds tracing, confirm it does not add new `event_type` values without a
  BC-2.16.002 catalog row.
- **Test isolation:** Wire-shape tests in `wire_shape.rs` must not assume a specific DTU
  fixture port or shared mutable state. Use the existing `HarnessBuilder` or equivalent
  per-test fixture isolation pattern established in S-3.3.05.

## Library & Framework Requirements

| Library | Version | Notes |
|---------|---------|-------|
| `serde_json` | workspace-pinned | `serde_json::to_value` for serializing tool responses to JSON tree |
| `nextest` | workspace-pinned | `just iter prism-mcp` for fast inner loop |
| `rmcp` | workspace-pinned | `CallToolResult` / `ReadResourceResult` types being serialized |
| DTU harness fixture utilities | internal | Reuse existing `HarnessBuilder` pattern from S-3.3.05; do NOT introduce a new fixture framework |

No new dependencies. Wire-shape tests use the same test infrastructure as existing prism-mcp
integration tests.

**Forbidden dependencies (build-time enforcement):** `prism-mcp` MUST NOT import `prism-query`
or `prism-sensors` crates in production code. Test files may use DTU harness crates under
`[dev-dependencies]`.

## File Structure Requirements

| File | Action | Notes |
|------|--------|-------|
| `crates/prism-mcp/tests/wire_shape.rs` | Create | New file containing all 20 wire-shape Red Gate tests (AC-002 through AC-012) |
| `CLAUDE.md` | Modify | Add `### SID-2` section after `### SID-1` in §Standing Adversary Probes & Implementer Disciplines |
| `crates/prism-mcp/src/server.rs` (`PrismServer::query`) | No code change needed (fix landed in DEFECT-MCP-ROWSHAPE-NULLS-001, pending merge) | `WriterBuilder.with_explicit_nulls(true)` already applied at all RecordBatch-to-JSON serialization sites. Note: `tools/query.rs` is a documentation facade with no handler code. |
| `crates/prism-mcp/tests/mod.rs` (or `lib.rs`) | Modify | Register `wire_shape` module if `tests/` uses explicit module declarations |

## Changelog

| Version | Date | Change | Source |
|---------|------|--------|--------|
| v0.20 | 2026-07-14 | Pin refresh (POL-23 follow-up): BC-2.11.001 v1.21→v1.22 (EC-11-081 added — Float64 non-finite (NaN/±Inf) serialization-as-null boundary contract codified; additive, no semantic change to the null-not-absent invariant or EC-11-079 tested by this story). 7 live-prose locations updated (8 string occurrences): frontmatter comment ~line 42 (2 occurrences: BC ID pin and bare `current pin v1.21` phrase), SID-2 step-3 invariant cite (~line 113), Behavioral Contracts table version cell (~line 138), AC-001 traces annotation (~line 144), AC-002 traces annotation (~line 155), Token Budget BC row (~line 326), Tasks read instruction (~line 341). Story version bumped 0.19→0.20. Historical §Changelog rows referencing `v1.21` as a destination (v0.10 row) left untouched per TD-VSDD-091. | F-MCPRS-PRL16-LOW-001 POL-23 follow-up (BC-2.11.001 v1.21→v1.22, EC-11-081) |
| v0.19 | 2026-07-14 | Pin refresh (POL-23): BC-2.10.007 v1.18→v1.19 (F-MCPRS-PRL16-HIGH-001 — `CursorCapExceeded` reclassified from `"validation"`/`original_params_valid: false` to `"internal"`/`original_params_valid: true`; dedicated VariantMeta arm + `map_prism_error` INTERNAL_ERROR alignment in DEFECT-PQL-FNCALL-LHS-001 fix-burst-25; no semantic change to the 9 wire-shape fields or `retry_after_seconds` null-not-absent invariants asserted by this story; all 13 live BC-2.10.007 v1.18 pins updated). Story version bumped 0.18→0.19. Historical §Changelog rows referencing `v1.18` as a destination (v0.17 and v0.18 rows) left untouched per TD-VSDD-091. | F-MCPRS-PRL16-HIGH-001; DEFECT-MCP-ROWSHAPE-NULLS-001 PR-LEVEL pass 16; DEFECT-PQL-FNCALL-LHS-001 fix-burst-25; POL-23 |
| v0.18 | 2026-07-14 | Fix (F-MCPRS-PRL11-MED-001): Behavioral Contracts body table cell BC-2.10.007 version v1.13→v1.18 (line ~139 — cell missed by both the v0.13 sweep and the v0.17 sweep). TD-VSDD-060 residual grep confirms zero remaining live-prose stale v1.13/v1.14/v1.15/v1.16/v1.17 BC-2.10.007 pins outside the changelog. Story version bumped 0.17→0.18. Historical §Changelog rows left untouched per TD-VSDD-091. | F-MCPRS-PRL11-MED-001; DEFECT-MCP-ROWSHAPE-NULLS-001 PR-LEVEL pass 11 |
| v0.17 | 2026-07-14 | Pin refresh (POL-23): BC-2.10.007 v1.17→v1.18 (F-MCPRS-PRL10-OBS-003 — §Rule 2 catch-all now FUTURE-ONLY; §Category table synced with 28 explicit-arm groups; catch-all no longer applies to any of the 18 formerly-catch-all variants tested here; no semantic change to the 9 wire-shape fields this story asserts; all 12 live BC-2.10.007 v1.17 pins updated). Story version bumped 0.16→0.17. Historical §Changelog rows referencing `v1.17` as a destination (v0.16 row) left untouched per TD-VSDD-091. | POL-23; DEFECT-MCP-ROWSHAPE-NULLS-001 fix-burst 22 F-MCPRS-PRL10-OBS-003 |
| v0.16 | 2026-07-14 | Pin refresh (POL-23): BC-2.10.007 v1.16→v1.17 (F-MCPRS-PRL8-OBS-002 snippet parity — `.as_u16()` removed; no semantic change to retryable rule; 503-test-vector row unchanged; all 12 live BC-2.10.007 v1.16 pins updated). Story version bumped 0.15→0.16. Historical §Changelog rows referencing `v1.16` as a destination (v0.15 row) left untouched per TD-VSDD-091. | POL-23; DEFECT-MCP-ROWSHAPE-NULLS-001 fix-burst 20 F-MCPRS-PRL8-OBS-002 |
| v0.15 | 2026-07-14 | Pin refresh (POL-23): BC-2.10.007 v1.15→v1.16 (§RETRYABLE-503 rule corrected from overbroad `!matches!(status, 401\|403)` to transient-only `matches!(status.as_u16(), 408\|425\|429\|500\|502\|503\|504)` — coordinator-raised finding; 503-test-vector row unchanged under both rules; all 12 live BC-2.10.007 v1.15 pins updated). Story version bumped 0.14→0.15. Historical §Changelog rows referencing `v1.15` as a destination (v0.14 row) left untouched per TD-VSDD-091. | POL-23; DEFECT-MCP-ROWSHAPE-NULLS-001 fix-burst 18 RETRYABLE-503-RULE |
| v0.14 | 2026-07-14 | Pin refresh (POL-23): BC-2.10.007 v1.14→v1.15 (F-MCPRS-PRL6-MED-001 QueryDenylisted vector struct corrected + POL-29 sweep + RETRYABLE-503 adjudication; all 11 AC trace annotations + frontmatter comment + Token Budget BC row updated). Story version bumped 0.13→0.14; `modified:` updated 2026-07-13→2026-07-14. Historical §Changelog rows referencing `v1.14` as a destination (v0.13 row) left untouched per TD-VSDD-091. | POL-23; DEFECT-MCP-ROWSHAPE-NULLS-001 fix-burst 18 |
| v0.13 | 2026-07-13 | Pin refresh (POL-23): BC-2.10.007 v1.13→v1.14 (§MED-001 safety-category arm added — SafetyContextContamination/SafetyDataExfiltration now have dedicated arm with `category: "safety"`, per-variant `ec_code_override`, `original_params_valid: true`; §LOW-001 test vectors completed with 3 missing LOW-002 vectors + 2 safety vectors + catch-all regression guard; error-taxonomy v2.46→v2.47 E-SAFETY-001/002 descriptions corrected). Sites updated: frontmatter comment (~line 45), Behavioral Contracts table version cell (~line 139), 10 AC trace annotations, Token Budget BC row (~line 327). Historical §Changelog rows referencing `v1.13` as a destination (v0.12 row) left untouched per TD-VSDD-091. Story version bumped 0.12→0.13; `modified:` already 2026-07-13. | POL-23; DEFECT-MCP-ROWSHAPE-NULLS-001 fix-burst 16 |
| v0.12 | 2026-07-13 | Pin refresh (POL-23): BC-2.10.007 v1.12→v1.13 (§LOW-002 arm code corrected — per-variant `ec_code_override` via nested match; implementer-discovered: `map_prism_error` returns `"Internal error"` for all 6 variants; Rule 1 redaction prevents code inference; v1.12 claim `ec_code_override: None` was incorrect). Sites updated: frontmatter comment (~line 45), Behavioral Contracts table version cell (~line 139), 10 AC traces annotations, Token Budget BC row (~line 327). Historical §Changelog rows referencing `v1.12` as a destination (v0.11 row) left untouched per TD-VSDD-091. Story version bumped 0.11→0.12; `modified:` already 2026-07-13. | POL-23; DEFECT-MCP-ROWSHAPE-NULLS-001 fix-burst 15 |
| v0.11 | 2026-07-13 | Pin refresh (POL-23): BC-2.10.007 v1.11→v1.12 (6 query engine variants moved from catch-all upstream_error to dedicated internal arm — F-MCPRS-PRL2-LOW-002; additive/clarifying; cited error-path postconditions unchanged in substance). Sites updated: frontmatter comment (~line 45), Behavioral Contracts table version cell (~line 139), 10 AC traces annotations, Token Budget BC row (~line 327). Historical §Changelog rows referencing `v1.11` as a destination (v0.4 row) left untouched per TD-VSDD-091. Story version bumped 0.10→0.11; `modified:` already 2026-07-13. | POL-23; DEFECT-MCP-ROWSHAPE-NULLS-001 F-MCPRS-PRL2-LOW-002 |
| v0.10 | 2026-07-13 | BC-2.11.001 version-pin refresh v1.20→v1.21 (F-MCPNULL-P16-LOW-001 POL-23 pin sweep, pass 16): 8 live sites updated — frontmatter comment ~line 42 (2 occurrences: BC ID pin `BC-2.11.001 v1.20` and bare `current pin v1.20` phrase), SID-2 step-3 invariant cite (~line 113), Behavioral Contracts table version cell (~line 138), AC-001 traces annotation (~line 144), AC-002 traces annotation (~line 155), Token Budget BC row (~line 326), Tasks read instruction (~line 341). ADR-051 §D4→§D2 correction at 2 sites (Precedents attribution fix — NULL-partial-failure precedent belongs to §D2, not §D4; `not absent` attributed to EC-11-079's own codification): §Origin [H20] narrative (~line 83) and AC-002 pipe-mode enrich note (~line 171). Historical §Changelog rows referencing `v1.20` as a destination (v0.9 row) left untouched per TD-VSDD-091. Story version bumped 0.9→0.10; `modified:` already 2026-07-13. | F-MCPNULL-P16-LOW-001 (pass-16) |
| v0.9 | 2026-07-13 | BC-2.11.001 version-pin refresh v1.18→v1.20 (F-MCPNULL-P14-MED-001, pass 14): 7 live sites updated — frontmatter comment (attribution corrected: null-not-absent postcondition attributed to v1.16 codification, not v1.18 prose-harmonize), SID-2 step-3 invariant cite, Behavioral Contracts table version cell, AC-001 traces annotation, AC-002 traces annotation, Token Budget BC row, Tasks read instruction. Story version bumped 0.8→0.9; `modified:` already 2026-07-13 (POL-23). EC-11-068 grep confirmed 0 stale live instances (1 historical-only in v0.8 changelog row, append-only per TD-VSDD-091). Changelog rows referencing `v1.18` in v0.2 and v0.5 entries are historical and left intact per append-only policy (TD-VSDD-091); §Changelog reordered to monotonic descending per POL-32 (pre-existing ascending order normalized). | F-MCPNULL-P14-MED-001 (pass-14) |
| v0.8 | 2026-07-13 | EC ID renumbering (F-MCPNULL-P13-HIGH-001): BC-2.11.001 v1.20 @62d48f01 renumbered row-shape EC EC-11-068→EC-11-079. Updated 4 live sites: frontmatter comment (~line 43); SID-2 step-3 definition (~line 110); Behavioral Contracts table (~line 138); AC-002 heading (~line 154). No BC-2.11.016 FIELDS-TRANSITION keeper EC-11-068 present in this file. | F-MCPNULL-P13-HIGH-001 (pass-13) |
| v0.7 | 2026-07-13 | API-name propagation (F-MCPNULL-P10-OBS-001): replaced `explicit_nulls(true)` / `WriterBuilder.explicit_nulls(true)` with `with_explicit_nulls(true)` at all 10 sites (frontmatter comment ~line 34; SID-2 step-3 ~line 113; Behavioral Contracts table ~line 138; AC-002 Red Gate note ~lines 166-167; Architecture Mapping ~line 292; Tasks ~lines 346-347; Previous Story Intelligence ~line 366; Architecture Compliance Rules ~line 384; File Structure Requirements ~line 416). Anchor correction (F-MCPNULL-P10-OBS-002): re-anchored query-tool handler from `tools/query.rs` (documentation facade) to `server.rs::PrismServer::query` in Architecture Mapping (~line 292) and File Structure Requirements (~line 416). Updated all DEFECT story hedges ("may be in DEFECT story" / "if not done by DEFECT story") to reflect fix landed in DEFECT-MCP-ROWSHAPE-NULLS-001, pending merge. | F-MCPNULL-P10-OBS-001/002 (pass-10) |
| v0.6 | 2026-07-13 | Corrected dotpath depth at 4 sites: `structuredContent.rows` → `structuredContent.results.rows` (§Origin [H20] line ~82; SID-2 step-3 line ~110; AC-002 assertion prose line ~162) and `result["structuredContent"]["rows"]` → `result["structuredContent"]["results"]["rows"]` (AC-002 implementation instruction line ~164). Verified via grep: `structuredContent\.rows` and `structuredContent\["rows"\]` — 0 remaining instances after this pass. Wire path confirmed from `envelope_json` helper (`structured_content` field) + test navigation `v["results"]["rows"]` in DEFECT-MCP-ROWSHAPE-NULLS-001 worktree. Correction note for v0.5 arithmetic: v0.5 claimed '6 sites' but grep-verifiable dotpath count is 4 (the 4 sites fixed here); the two other v0.5 entries ('line ~77 wire-level key' and 'Previous Story Intelligence line ~360') were converted to neutral/non-dotpath prose in v0.5 rather than to `structuredContent.rows`, so they were not surviving dotpath instances — accurate v0.5 dotpath site count was 4, not 6. | F-MCPNULL-P9-MED-001 + OBS-002 (pass-9) |
| v0.5 | 2026-07-13 | Retired key `structuredContent.events` → canonical `structuredContent.rows` at 6 sites: §Origin [H20] narrative (line ~77 wire-level key, line ~82 dotpath), SID-2 step-3 definition (line ~110), AC-002 assertion prose (line ~162), AC-002 implementation instruction (line ~164), Previous Story Intelligence narrative (line ~360). Canonical key per BC-2.11.001 v1.18 / shipped server.rs payload. | F-MCPNULL-P8-MED-001 (pass-8); POL-25 full-file sweep |
| v0.4 | 2026-07-13 | Pin refresh (POL-23): BC-2.10.007 v1.10→v1.11 (Rule-1 exhaustive carve-out + McpSerializationError category ruling — additive/clarifying; cited postconditions unchanged in substance). | POL-23; DEFECT-MCP-ROWSHAPE-NULLS-001 F-MCPNULL-P7-MED-001/OBS-002 |
| v0.3 | 2026-07-13 | Pin refresh (POL-23): BC-2.10.007 v1.9→v1.10 (clarifying rewrite of split postcondition; cited postconditions unchanged in substance). Propagated from DEFECT-MCP-ROWSHAPE-NULLS-001 F-MCPNULL-P6-OBS-003. | POL-23; DEFECT-MCP-ROWSHAPE-NULLS-001 F-MCPNULL-P6-OBS-003 |
| v0.2 | 2026-07-13 | Pin refresh (POL-23): BC-2.10.007 v1.8→v1.9 (additive amendment, cited postconditions unchanged); BC-2.11.001 v1.16→v1.18 (additive amendment, null-not-absent postcondition unchanged). Propagated from DEFECT-MCP-ROWSHAPE-NULLS-001 F-MCPNULL-P4-OBS-001 adversarial finding. | POL-23; DEFECT-MCP-ROWSHAPE-NULLS-001 F-MCPNULL-P4-OBS-001 |
| v0.1 | 2026-07-13 | Initial draft — 12 ACs, 20 Red Gate tests, SID-2 codification | AUDIT-COVERAGE-001 D-hardening; D-1715/D-1716 |
