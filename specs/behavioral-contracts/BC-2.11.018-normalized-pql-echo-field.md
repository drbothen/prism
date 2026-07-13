---
document_type: behavioral-contract
level: L3
version: "1.5"
status: active
producer: product-owner
timestamp: 2026-06-19T00:00:00Z
phase: 1a
inputs: [".factory/specs/domain-spec/capabilities.md", ".factory/specs/domain-spec/invariants.md", ".factory/specs/architecture/decisions/ADR-041-prismql-llm-auto-onboarding-4-layer-teaching-surface-for-automatic-agent-query-authoring.md", ".factory/specs/architecture/decisions/ADR-047-prismql-case-sensitivity-policy-ieq-iin-and-adapter-boundary-normalization.md"]
input-hash: "TBD"
traces_to: ["CAP-015"]
extracted_from: null
origin: greenfield
subsystem: "SS-11"
capability: "CAP-015"
lifecycle_status: active
introduced: 2026-06-19
modified: "2026-07-13"
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.11.018: `normalized_pql` Field on Successful Query Responses (L4 Echo / OPD-1)

## Description

On every successful `query` tool execution (parse + plan + execute all pass), the response includes an optional `normalized_pql` field containing the Chumsky-normalized PQL query string — the query in the form the planner accepted after parsing, normalization, and whitespace/alias canonicalization. This is the string the server would replay verbatim to reproduce the identical query plan. The field is absent on ALL error responses. This is the "echo-normalized-PQL-back" pattern (ADR-041 v1.1 OPD-1, adopted per human product decision 2026-06-19) — the LLM agent accrues grounded exemplars over a session, improving the accuracy of subsequent queries.

## Preconditions

1. A `query` MCP tool call has been submitted and has completed the full pipeline: Chumsky parse → plan validation (E-QUERY-037/038 gates) → DataFusion execution.
2. All pipeline stages returned successfully (no parse error, no plan-time rejection, no execution failure).
3. The Chumsky parser/normalizer in `prism-query` has produced a normalized query string from the submitted input.

## Postconditions

### Field presence invariants

**Present (required):** `normalized_pql` MUST be present as a non-empty string on every successful `query` tool response. "Successful" means: no E-QUERY-NNN error was returned; results were produced (including the case of zero matching rows — zero results is a success, not an error).

**Absent (required):** `normalized_pql` MUST be absent (the field must not appear in the JSON object) on ALL error responses. This includes:
- E-QUERY-001 (parse error)
- E-QUERY-002 (type error)
- E-QUERY-003 (security limits)
- E-QUERY-004 (timeout)
- E-QUERY-005 (materialization limit)
- E-QUERY-006 (scope too broad)
- E-QUERY-037 (table unavailable)
- E-QUERY-038 (column not found)
- Any other E-QUERY-NNN or E-MCP-NNN error
- Partial failures in `sensor_errors` do NOT count as "error response" — if the query succeeded with some sensor errors, `normalized_pql` is still present.

The field must be absent, not null, not empty string — it must not appear in the response JSON at all for error cases.

### Wire field name

The field name is `normalized_pql` (snake_case). This is the adopted name per ADR-041 v1.1 §Response Envelope Placement. The BC author has chosen `normalized_pql` as the wire name (ADR-041 deferred to the BC author; other candidate names — `planned_query`, `echoed_query` — are NOT used).

### Field content — what is included

`normalized_pql` contains the **Chumsky-normalized PQL string** — the query as the server's parse/normalization pipeline produced it from the model's input. Specifically:

- **Normalized form:** the query after Chumsky parsing and canonicalization — normalized whitespace, canonicalized keyword casing (e.g., `select` → `SELECT`), alias-expanded to the canonical form the planner used.
- **Round-trips through Chumsky:** the value is the server-emitted normalized form, not raw model input verbatim. If the model submitted `select * from foo where x  =  1`, the normalized form might be `SELECT * FROM foo WHERE x = 1`. The normalized form MUST parse to the same AST as the original — this is the "round-trips to the same plan" guarantee.
- **Injection-safe:** the `normalized_pql` value is emitted by the Prism Chumsky parser and normalizer pipeline — trusted server code. It is a normalized re-emission of the model's own syntactically-valid PQL input, not raw model input verbatim. The injection-safety boundary established in ADR-041 §Cross-Cutting holds.

### Field content — what is EXCLUDED

The following MUST NOT appear in `normalized_pql`:

1. **DataFusion physical plan internals** — no execution plan node-type strings (e.g., `HashJoin`, `TableScan`, `SortExec`), no physical operator annotations.
2. **Cost estimates** — no optimizer cost annotations, no row count estimates, no selectivity annotations.
3. **Alias-expansion internals beyond the PQL canonical form** — how column aliases were resolved internally to DataFusion projection expressions is NOT exposed.
4. **Join-order decisions** — which join side was chosen as probe vs. build is NOT in the normalized PQL string.
5. **Partition/pushdown details** — filter pushdown decisions, partition pruning annotations are NOT exposed.
6. **Sensor API URLs or credentials** — the field contains PQL, not HTTP request URLs.

The `normalized_pql` value should look like a valid PQL query string — something the model could submit as input on its next call and get the same results.

### Response envelope placement

`normalized_pql` is an ADDITIVE, OPTIONAL field on the `query` tool's JSON response. The existing fields (`rows`, `returned_results`, `total_available`, `is_truncated`, `sensor_errors`) are UNCHANGED. The new field is appended after the existing fields:

```json
{
  "rows": [...],
  "returned_results": 42,
  "total_available": 42,
  "is_truncated": false,
  "sensor_errors": [],
  "normalized_pql": "SELECT host_name, COUNT(*) FROM crowdstrike_detections WHERE timestamp > NOW() - INTERVAL '1h' GROUP BY host_name ORDER BY COUNT(*) DESC LIMIT 10"
}
```

The response type carrying `normalized_pql` MUST be `#[non_exhaustive]` per CLAUDE.md conventions. If it is not already `#[non_exhaustive]`, it must be marked so before the PR merges. The `ci.yml EXPECTED` non-exhaustive gate count is incremented if this type is newly non-exhaustive.

### IEQ / IIN / INE round-trip in normalized_pql (ADR-047 D.4)

`IEQ`, `IIN`, and `INE` case-insensitive predicate operators (BC-2.11.024) are reflected in `normalized_pql` on successful query responses. The Chumsky normalizer emits operator keywords in uppercase canonical form: a query submitted as `severity ieq 'high'` appears in `normalized_pql` as `severity IEQ 'high'`. The round-trip guarantee applies: the value in `normalized_pql` parses back to the same AST as the original, preserving the `case_insensitive: true` flag on the `Predicate` node.

### Token cost acceptance

Adding `normalized_pql` costs approximately +50–200 tokens per successful query response (proportional to query complexity). This overhead is explicitly accepted per the human product decision (ADR-041 v1.1 OPD-1 resolution). No token budget optimization is required.

### In-session self-teaching benefit

A model that receives `normalized_pql` in a successful query response has a grounded exemplar it can use as a template for subsequent queries in the same session. This composes with L4 (pedagogical error loop): if the model bases its next query on the echoed normalized form, it already knows this shape is valid for this server, reducing parse-time rejection probability. The benefit is strongest in multi-step analyst workflows — the core Prism demo scenario.

## Invariants

- DI-006: `normalized_pql` is server-emitted Chumsky-normalized PQL. It is NOT a verbatim echo of model input, NOT sensor API response data, NOT user free-text. The injection-safety boundary is maintained: the normalization pipeline is trusted server code.
- DI-004: The `query` tool already emits an `AuditEntry` per call (BC-2.11.001). The `normalized_pql` field does not change audit behavior — no additional audit event is required for the echo field.
- DI-019: The normalized PQL string does not expose query security limit internals. Its presence does not relax any security limit.

## Error Cases

| Condition | Behavior |
|-----------|----------|
| Query fails with any E-QUERY-NNN error | `normalized_pql` field is ABSENT from the response JSON |
| Query succeeds but produces zero rows | `normalized_pql` is PRESENT (zero rows is a successful execution) |
| Query uses an alias that is expanded during normalization | `normalized_pql` contains the expanded (canonical) form — the alias is resolved to its definition |
| Chumsky normalization produces an empty string (should not happen for valid parse) | Implementation MUST NOT emit `normalized_pql: ""` — if for any reason the normalized string is empty after normalization, omit the field rather than include an empty value |

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-11-051 | Model submits a query with non-canonical whitespace: `select  *  from   foo  where x=1` | `normalized_pql` returns the canonical form: `SELECT * FROM foo WHERE x = 1` (or equivalent canonical form per Chumsky) |
| EC-11-052 | Model submits a query that uses an alias defined via `create_alias` | `normalized_pql` contains the alias-expanded form (the PQL the planner executed, with alias replaced by its definition) |
| EC-11-053 | Query times out (E-QUERY-004) | `normalized_pql` is ABSENT (timeout is an error response) |
| EC-11-054 | Query produces partial results (some sensors errored, some succeeded) | `normalized_pql` is PRESENT — partial sensor failure is surfaced in `sensor_errors`, not as a query-level error |
| EC-11-055 | Model submits a pipe-mode query: `FROM foo | where severity = 'high' | head 10` | `normalized_pql` contains the normalized pipe-mode string — the canonical pipe form, not SQL-mode translation |
| EC-11-056 | `normalized_pql` value contains DataFusion plan internals (e.g., `HashJoin` or `TableScan` text) | This is a test failure — the normalization pipeline MUST NOT emit DataFusion plan strings. If this occurs, it is a bug in the normalization implementation. |
| EC-11-057 | Model submits a query using IEQ/IIN with lowercase keyword: `FROM crowdstrike_detections \| where severity ieq 'high' \| head 10` | `normalized_pql` reflects the operator in uppercase canonical form: `... \| where severity IEQ 'high' \| head 10`. The round-trip guarantee applies: the `normalized_pql` value parses back to the same AST (with `case_insensitive: true` on the predicate). |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Successful `query("SELECT * FROM crowdstrike_alerts WHERE severity = 'high' LIMIT 10")` | Response contains `normalized_pql` as a non-empty string that starts with `SELECT` and contains `crowdstrike_alerts`; field is absent on error responses | happy-path (field-present) |
| Failed `query("SELECT * FROM nonexistent_table")` → E-QUERY-037 | Response does NOT contain `normalized_pql` field | field-absent-on-error |
| Successful `query("select * from crowdstrike_alerts limit 5")` (lowercase) | `normalized_pql` contains uppercase `SELECT ... FROM ... LIMIT` (or equivalent canonical form) — different from raw input | normalization |
| `normalized_pql` value does not contain any of: `"HashJoin"`, `"TableScan"`, `"SortExec"`, `"Aggregate"` (DataFusion plan node names) | Pass — these are internal plan node type strings; normalized PQL must not contain them | exclusion-invariant |
| Zero-row successful query | `normalized_pql` present; `returned_results: 0`; `rows: []` | zero-rows-success |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| (VP-TBD) | `normalized_pql` is absent on ALL E-QUERY-NNN error responses for the `query` tool | integration test suite covering all E-QUERY error codes |
| (VP-TBD) | `normalized_pql` value does not contain any DataFusion physical plan node type strings (proptest random queries) | proptest |
| (VP-TBD) | `normalized_pql` value parses successfully when resubmitted to the Chumsky parser (round-trip property) | proptest |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-015 |
| Capability Anchor Justification | CAP-015 ("Ephemeral OCSF Query Engine") per capabilities.md §CAP-015 — this BC adds an optional `normalized_pql` field to the `query` tool's success response envelope. CAP-015 is the authoritative capability for the PQL query engine and its output format: "Results include `query_context` echo-back (original query, expanded query after alias resolution, clients/sensors queried, execution time)." The `normalized_pql` echo-back is analogous to the existing `original_query` and `expanded_query` fields in `query_context` — it is a query-result metadata field owned by the query engine layer (CAP-015). |
| L2 Invariants | DI-004, DI-006, DI-019 |
| ADR | ADR-041 v1.1 §Echo-Normalized-PQL-Back (Pattern B — Cortex-Analyst): ADOPTED IN V1; §Open Product Decisions OPD-1: RESOLVED — Adopted v1 (human decision, 2026-06-19) |
| Architecture Module | SS-11 (Query Execution) — `prism-query` provides the normalized string; SS-10 (MCP Interface) — `prism-mcp` places it in the response envelope |
| Priority | P1 |

## Related BCs

- BC-2.11.001 — depends on: `query` tool is the surface carrying `normalized_pql`; BC-2.11.001 `query_context` already carries related echo fields (`original_query`, `expanded_query`); story-writer should add `normalized_pql` as a postcondition note to BC-2.11.001 (under `bc_array_changes_propagate_to_body_and_acs`)
- BC-2.11.016 — composes with: E-QUERY-038 errors (column not found) suppress `normalized_pql`; successful queries after column correction carry `normalized_pql`
- BC-2.11.017 — composes with: all four pedagogical error enrichments also suppress `normalized_pql` (they are error paths); `normalized_pql` is the success-path complement

## Architecture Anchors

- `architecture/decisions/ADR-041` §Echo-Normalized-PQL-Back — "What Is Echoed", "What Is Excluded", "Response Envelope Placement", "Token Cost", "Composition with L4"
- `architecture/decisions/ADR-041` §Alternatives Considered Option D — adopted in v1 with scoping

## Story Anchor

S-DEMO-PRISMQL-ONBOARDING-001-B — the BC's behavior (`normalized_pql` echo field on successful query responses) is implemented by this story per its `anchor_bcs: [BC-2.11.018, ...]` frontmatter (POL-4/POL-5).

## VP Anchors

VP assignments TBD — assigned after VP authoring pass.

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.5 | DEFECT-MCP-ROWSHAPE-NULLS-001-pass3-phantom-fields | 2026-07-13 | product-owner | **F-MCPNULL-P3-MED-001 closure — phantom-field sweep (POL-25 full-file).** Two fix sites: (1) §Canonical Test Vectors "Zero-row successful query" row: `events: []` → `rows: []` (retired key; the v1.4 sweep only fixed §Response envelope placement and missed the test-vector row — POL-25 incomplete-sweep gap); `row_count: 0` → `returned_results: 0` (`row_count` is a phantom field — no such key exists in the actual server.rs payload; the shipped field is `returned_results` per `prism-mcp/src/server.rs` lines 1994-1995). (2) §Response envelope placement JSON example: `row_count: 42` → `returned_results: 42`; `execution_time_ms: 1234` removed (not in top-level payload — `execution_time_ms` lives in `result.context` but is not surfaced in the `serde_json::json!({...})` payload block); `query_context: {...}` removed (no such nested object in the server-emitted payload). Correct top-level keys per server.rs: `rows`, `returned_results`, `total_available`, `is_truncated`, `sensor_errors`, `normalized_pql` (conditional). (3) §Response envelope placement prose "existing fields" list: `results`, `row_count`, `execution_time_ms`, `query_context` removed; updated to `rows`, `returned_results`, `total_available`, `is_truncated`, `sensor_errors`. **POL-25 full-file sweep — all sites checked:** §Description (lines 29-32): no `events` / `row_count` — clean. §Preconditions: clean. §Postconditions (field-presence invariants, wire-field-name, field-content, field-exclusions): clean. §Postconditions §Response envelope placement prose "existing fields" list — fixed (site 3 above). §Postconditions §Response envelope placement JSON example — fixed (site 2 above). §Postconditions §IEQ/IIN/INE round-trip: clean. §Postconditions §Token cost/in-session-self-teaching: clean. §Invariants: clean. §Error Cases table: clean. §Edge Cases (EC-11-051 through EC-11-057): clean. §Canonical Test Vectors: zero-row row fixed (site 1 above); all other rows clean. §Verification Properties: clean. §Traceability: clean. §Related BCs: clean. §Architecture Anchors: clean. §Story Anchor: clean. §VP Anchors: clean. Normalized_pql field semantics, round-trip guarantee, injection-safety boundary, and IEQ/IIN/INE extension ALL UNCHANGED. |
| 1.4 | DEFECT-MCP-ROWSHAPE-NULLS-001-events-to-rows | 2026-07-13 | product-owner | **POL-23 sibling sweep — `events` → `rows` in response envelope placement example.** §Response envelope placement: updated the existing-field list and the JSON example to use `"rows"` instead of `"events"`. The response array key was renamed from `events` to `rows` in S-5.01-FOLLOWUP-MCP-BOOT (PR #163); this BC's envelope example retained the old key. Brought into alignment with BC-2.11.001 v1.17 and shipped behavior per human adjudication 2026-07-13 (DEFECT-MCP-ROWSHAPE-NULLS-001 F-MCPNULL-P1-MED-001). `normalized_pql` field semantics and all other postconditions UNCHANGED. |
| 1.3 | S-PRISMQL-CASE-INSENSITIVE-001-bc-burst | 2026-07-06 | product-owner | **ADR-047 D.4 amendment: IEQ/IIN/INE round-trip in `normalized_pql`.** §Postconditions: added "IEQ / IIN / INE round-trip in normalized_pql (ADR-047 D.4)" sub-section — operator keywords uppercased in canonical form; round-trip guarantee applies to `case_insensitive: true` AST flag. §Edge Cases: EC-11-057 added (`severity ieq 'high'` → `normalized_pql` contains `severity IEQ 'high'` uppercase; round-trip parses same AST). inputs: ADR-047 added. |
| 1.2 | F-001B-SCFRESH-MED-001-story-anchor-fix | 2026-06-22 | product-owner | F-001B-SCFRESH-MED-001 closure (POL-4 story-anchor mis-anchoring): `## Story Anchor` corrected from placeholder `S-5.04 (or dedicated ADR-041 teaching story — to be assigned by story-writer)` to the actual implementing story `S-DEMO-PRISMQL-ONBOARDING-001-B`. Exhaustive BC metadata audit: all other surfaces clean. |
| 1.1 | F-001B-FRESH2-MED-001-pol20-normalization | 2026-06-22 | product-owner | POL-20 normalization: `introduced: ADR-041-teaching-burst-2026-06-19` → `introduced: 2026-06-19` (opaque burst-ID format prohibited by POL-20 anchored-regex; ISO date extracted). Also set `modified: 2026-06-22` (first amendment; POL-27). No body semantics changed. |
| 1.0 | ADR-041-teaching-burst-2026-06-19 | 2026-06-19 | product-owner | Initial draft — ADR-041 L4/echo `normalized_pql` field on successful query responses (OPD-1 adopted) |
