---
document_type: story
story_id: "S-TEST-WIRESHAPE-SWEEP-001"
title: "Retroactive wire-shape assertion sweep — all 14 MCP-visible tool surfaces + 6 resource surfaces"
wave: maintenance
epic_id: maintenance
priority: P1
status: draft
version: "0.6"
spec_version: "v0.6"
level: ops
producer: story-writer
timestamp: "2026-07-13"
modified: "2026-07-13"
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
#   row-shape (explicit_nulls) — the root cause of [C3] and [H20] escapes.
#   SS-10 is primary (all surfaces touched are MCP-layer); SS-11 is co-owner for the
#   query tool row-shape assertion subset.
crates_touched:
  - prism-mcp
target_module: "crates/prism-mcp"
behavioral_contracts: [BC-2.11.001, BC-2.10.007]
# BC status: both BCs are active.
#   BC-2.11.001 v1.18 (modified 2026-07-13): null-not-absent row-shape postcondition
#   added (DEFECT-MCP-ROWSHAPE-NULLS-001); EC-11-068. This is the primary anchor for
#   query tool wire-shape tests.
#   BC-2.10.007 v1.11: structured error response wire shape — all 9 required fields,
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
- **[H20]** `threat_score` column ABSENT from all rows (ADR-051 §D4 NULL output expected). The
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
> 3. **Explicit-null key presence (EC-11-068)** — for each entry in `structuredContent.results.rows`, assert that
>    every projected column key is present in every row. For NULL-valued cells, assert
>    the row has `"column_name": null`, not merely that the row parses without error.
>    The `WriterBuilder.explicit_nulls(true)` invariant (BC-2.11.001 v1.18) must be
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
| BC-2.11.001 | `query` MCP Tool Accepts Scoping + PrismQL Query String | v1.18 | Postcondition: row-shape null-not-absent — `WriterBuilder.explicit_nulls(true)` required; EC-11-068 (NULL column → key present as `null`). Primary anchor for AC-002. |
| BC-2.10.007 | Structured Error Responses | v1.11 | Postcondition: nested wire shape with 9 required fields; `retry_after_seconds: null` (not absent) for non-rate-limit errors. Primary anchor for AC-003 and all error-path ACs. |

## Acceptance Criteria

### AC-001 — SID-2 codified in CLAUDE.md
(traces to BC-2.11.001 v1.18 invariant — agent-harness schema stability rationale)

The full SID-2 definition (see §Origin above) is added to `CLAUDE.md`
§Standing Adversary Probes & Implementer Disciplines immediately after SID-1, with section
header `### SID-2 — Implementer discipline: wire-shape assertions for MCP tool and resource
responses`. The source line reads: `Source: T13 live-audit escapes [C3]/[H20]/[H8b] (2026-07-13);
AUDIT-COVERAGE-001 B/C/D-hardening cascade.`

No Red Gate test for this AC (doc-only change).

### AC-002 — `query` tool: explicit-null key presence (EC-11-068)
(traces to BC-2.11.001 v1.18 postcondition — row-shape null-not-absent)

`crates/prism-mcp/tests/wire_shape.rs` (new file) adds:

`test_BC_2_11_001_query_tool_explicit_null_key_present` — executes a query against a DTU
fixture that returns at least one row with a NULL-valued column (e.g., `sensor_ip` or
`threat_score`). Asserts on the JSON-serialized response that every row in
`structuredContent.results.rows` contains the column key with value `null`, not key-omission.
Implementation: call the `query` tool handler, serialize the result to `serde_json::Value`,
walk `result["structuredContent"]["results"]["rows"]`, assert each row has the projected column key.

Red Gate: before the `WriterBuilder.explicit_nulls(true)` fix ships (if not already applied
by DEFECT-MCP-ROWSHAPE-NULLS-001), this test fails with the column key absent from null rows.

`test_BC_2_11_001_query_tool_enrich_null_column_key_present` — pipe-mode `| enrich` query
where the enrichment UDF returns NULL for some rows. Asserts the enriched column key is
present as `null` in those rows (ADR-051 §D4). Targets T13 [H20] escape.

### AC-003 — `query` tool: error-path BC-2.10.007 wire shape
(traces to BC-2.10.007 v1.11 postcondition — structured error wire shape)

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
(traces to BC-2.10.007 v1.11 postcondition for error path; success path traces to BC-2.10.001
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
(traces to BC-2.10.007 v1.11 postcondition for error path)

`test_check_sensor_health_success_wire_shape` — calls `check_sensor_health` with a valid
sensor. Serializes to JSON and asserts `structuredContent` contains a health result with at
minimum `client_id` and `sensors` keys present. This is a key-presence assertion (SID-2 step 1).

`test_check_sensor_health_invalid_client_error_wire_shape` — invalid client_id returns
`structuredContent.error.code` == `"E-MCP-001"` (error-code anchor).

### AC-006 — `explain_query` tool: wire-shape assertion
(traces to BC-2.10.007 v1.11 postcondition)

`test_explain_query_success_wire_shape` — calls `explain_query` with a valid PrismQL string.
Serializes to JSON and asserts `structuredContent` contains at minimum `plan_steps` (array) or
equivalent key; `content[0]["text"]` is non-empty.

`test_explain_query_parse_error_wire_shape` — invalid PrismQL returns `structuredContent.error.code`
== `"E-QUERY-001"` (error-code anchor, SID-2 step 4).

### AC-007 — `list_capabilities` tool: wire-shape assertion
(traces to BC-2.10.007 v1.11 postcondition)

`test_list_capabilities_wire_shape` — calls `list_capabilities`. Serializes to JSON and asserts
`structuredContent` contains a `tools` array where each entry has at minimum `name` (string)
and `available` (bool) keys present. Asserts `content[0]["text"]` is non-empty (composed-string
from the capabilities list, SID-2 step 2).

### AC-008 — Alias family tools wire-shape assertions (create_alias, list_aliases, delete_alias, explain_alias)
(traces to BC-2.10.007 v1.11 postcondition for error paths)

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
(traces to BC-2.10.007 v1.11 postcondition)

`test_confirm_action_invalid_token_wire_shape` — calls `confirm_action` with an invalid token.
Asserts `structuredContent.error.code` is present and non-empty (error-code anchor, SID-2 step 4);
`retry_after_seconds` == `null` (null-not-absent invariant, SID-2 step 5).

### AC-010 — Config tools wire-shape assertions (reload_config, add_sensor_spec, list_sensor_specs, validate_config)
(traces to BC-2.10.007 v1.11 postcondition for error paths)

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
(traces to BC-2.10.007 v1.11 postcondition for error paths)

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
(traces to BC-2.10.007 v1.11 postcondition for error paths)

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
| `query` tool handler | `crates/prism-mcp/src/tools/query.rs` | Effectful (existing; `.explicit_nulls(true)` fix may be in DEFECT story) |
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
| BC-2.11.001 v1.18 (query BC with null-not-absent postcondition) | ~160 | ~2,300 |
| BC-2.10.007 v1.11 (structured error BC) | ~120 | ~1,700 |
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

- [ ] Read BC-2.11.001 v1.18 postcondition (null-not-absent) and BC-2.10.007 postcondition (structured error wire shape) before writing any test.
- [ ] Read `crates/prism-mcp/src/server.rs` (LIVE_TOOLS list, dispatch table) and `resources.rs` (build_resource_list, build_resource_template_list) to confirm the 14+6 surface inventory.
- [ ] Add `### SID-2` to CLAUDE.md §Standing Adversary Probes & Implementer Disciplines immediately after SID-1 (full definition from §Origin above). Commit this change in the same PR.
- [ ] Create `crates/prism-mcp/tests/wire_shape.rs` (new file). Add all 20 Red Gate tests (AC-002 through AC-012, counting sub-tests).
- [ ] Run `cargo nextest run -p prism-mcp -E 'test(wire_shape)'` — confirm all tests fail RED before production changes (TDD strict mode per BC-5.38.001).
- [ ] If DEFECT-MCP-ROWSHAPE-NULLS-001 has not yet merged: AC-002's `explicit_nulls` tests will be RED; they are the TDD Red Gate for that defect fix. Confirm the fix is either in scope of this story or deferred to DEFECT-MCP-ROWSHAPE-NULLS-001 with a story dependency note.
- [ ] Implement any missing production changes needed to make tests GREEN (likely limited to `explicit_nulls(true)` if not already applied).
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
  `explicit_nulls(true)` issue. This story is the TEST coverage companion. Coordinate merge
  order: the Red Gate tests from this story should go RED on a build without the defect fix
  and GREEN after.
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
  `explicit_nulls`), NOT `file.rs:NNN` line numbers, in test comments.
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
| `crates/prism-mcp/src/tools/query.rs` | Modify (if not done by DEFECT story) | Ensure `WriterBuilder.explicit_nulls(true)` at all RecordBatch-to-JSON serialization sites in query response path |
| `crates/prism-mcp/tests/mod.rs` (or `lib.rs`) | Modify | Register `wire_shape` module if `tests/` uses explicit module declarations |

## Changelog

| Version | Date | Change | Source |
|---------|------|--------|--------|
| v0.1 | 2026-07-13 | Initial draft — 12 ACs, 20 Red Gate tests, SID-2 codification | AUDIT-COVERAGE-001 D-hardening; D-1715/D-1716 |
| v0.2 | 2026-07-13 | Pin refresh (POL-23): BC-2.10.007 v1.8→v1.9 (additive amendment, cited postconditions unchanged); BC-2.11.001 v1.16→v1.18 (additive amendment, null-not-absent postcondition unchanged). Propagated from DEFECT-MCP-ROWSHAPE-NULLS-001 F-MCPNULL-P4-OBS-001 adversarial finding. | POL-23; DEFECT-MCP-ROWSHAPE-NULLS-001 F-MCPNULL-P4-OBS-001 |
| v0.3 | 2026-07-13 | Pin refresh (POL-23): BC-2.10.007 v1.9→v1.10 (clarifying rewrite of split postcondition; cited postconditions unchanged in substance). Propagated from DEFECT-MCP-ROWSHAPE-NULLS-001 F-MCPNULL-P6-OBS-003. | POL-23; DEFECT-MCP-ROWSHAPE-NULLS-001 F-MCPNULL-P6-OBS-003 |
| v0.4 | 2026-07-13 | Pin refresh (POL-23): BC-2.10.007 v1.10→v1.11 (Rule-1 exhaustive carve-out + McpSerializationError category ruling — additive/clarifying; cited postconditions unchanged in substance). | POL-23; DEFECT-MCP-ROWSHAPE-NULLS-001 F-MCPNULL-P7-MED-001/OBS-002 |
| v0.5 | 2026-07-13 | Retired key `structuredContent.events` → canonical `structuredContent.rows` at 6 sites: §Origin [H20] narrative (line ~77 wire-level key, line ~82 dotpath), SID-2 step-3 definition (line ~110), AC-002 assertion prose (line ~162), AC-002 implementation instruction (line ~164), Previous Story Intelligence narrative (line ~360). Canonical key per BC-2.11.001 v1.18 / shipped server.rs payload. | F-MCPNULL-P8-MED-001 (pass-8); POL-25 full-file sweep |
| v0.6 | 2026-07-13 | Corrected dotpath depth at 4 sites: `structuredContent.rows` → `structuredContent.results.rows` (§Origin [H20] line ~82; SID-2 step-3 line ~110; AC-002 assertion prose line ~162) and `result["structuredContent"]["rows"]` → `result["structuredContent"]["results"]["rows"]` (AC-002 implementation instruction line ~164). Verified via grep: `structuredContent\.rows` and `structuredContent\["rows"\]` — 0 remaining instances after this pass. Wire path confirmed from `envelope_json` helper (`structured_content` field) + test navigation `v["results"]["rows"]` in DEFECT-MCP-ROWSHAPE-NULLS-001 worktree. Correction note for v0.5 arithmetic: v0.5 claimed '6 sites' but grep-verifiable dotpath count is 4 (the 4 sites fixed here); the two other v0.5 entries ('line ~77 wire-level key' and 'Previous Story Intelligence line ~360') were converted to neutral/non-dotpath prose in v0.5 rather than to `structuredContent.rows`, so they were not surviving dotpath instances — accurate v0.5 dotpath site count was 4, not 6. | F-MCPNULL-P9-MED-001 + OBS-002 (pass-9) |
