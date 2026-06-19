---
document_type: story
story_id: S-DEMO-PRISMQL-ONBOARDING-001-B
title: "PrismQL LLM Auto-Onboarding — Query Engine L4 (E-QUERY-038 Gate + Pedagogical Enrichments + normalized_pql)"
wave: null
# Wave assignment deferred — schedules after PIVOT-003 merges (prism-query crate conflict
# avoidance per D-1244). PIVOT-003 also touches prism-query; pipeline B behind it.
target_module: prism-query
subsystems: [SS-11]
# Subsystem anchor justifications:
#   SS-11 (Query Execution Engine) owns this sub-story's scope per ARCH-INDEX Subsystem Registry.
#   E-QUERY-038 plan-time gate, E-QUERY-001/002/003/037 pedagogical error enrichments, and the
#   normalized_pql Chumsky normalizer all live in prism-query/prism-core per ADR-041 v1.1
#   Architectural Surface table. The prism-mcp response envelope for normalized_pql (field
#   declaration + #[non_exhaustive] mark) is a thin wire between SS-11 output and SS-10 surface;
#   it is included in this sub-story because the prism-query normalizer is the hard work.
priority: P0
# P0: DEMO-BLOCKING per D-1243 (S-DEMO-PRISMQL-ONBOARDING-001 set). The pedagogical E-QUERY-038
# error + normalized_pql field are required for Claude to self-correct PrismQL queries in the
# multi-client SOC demo capstone.
depends_on:
  - S-5.03
  # S-5.03 (MERGED — provides TableRegistry injection into PrismServer; the same TableRegistry
  # instance that this sub-story's E-QUERY-038 column gate reads from at plan time).
  - S-3.13
  # S-3.13 (MERGED — provides Arc<dyn TableRegistry> wired into PrismServer; E-QUERY-038 reads
  # the registry's per-org column schema for the table being queried).
# Dependency anchors:
#   S-DEMO-PRISMQL-ONBOARDING-001-B depends on S-5.03 because S-5.03 completed the TableRegistry
#     injection pattern into PrismServer that this sub-story's column gate reads from.
#   S-DEMO-PRISMQL-ONBOARDING-001-B depends on S-3.13 because the E-QUERY-038 plan-time column
#     gate uses the same TableRegistry org-scoped filter pattern as E-QUERY-037, which was
#     established by S-3.13.
#   NOTE: no hard functional dependency on PIVOT-003. For smooth merge sequencing (D-1244 crate-
#     conflict avoidance in prism-query), this sub-story SHOULD pipeline after PIVOT-003.
#     PIVOT-003 adds IOC columns to Cyberint/CrowdStrike TOML specs which expand the column set
#     visible to E-QUERY-038; that expansion is a data dependency at demo time, not a code
#     dependency. The orchestrator should merge PIVOT-003 first where possible.
blocks:
  - S-DEMO-PRISMQL-ONBOARDING-001-A
  # 001-A's server.rs wire for normalized_pql response field depends on prism-query exposing
  # the normalized string. 001-B must be deliverable before 001-A declares normalized_pql
  # response wiring complete. This is a soft blocks relationship — 001-A can merge its L1/L2/L3
  # surface first; normalized_pql response envelope in 001-A should land after 001-B merges
  # (or 001-A can omit the normalized_pql server.rs wire and add it at 001-B merge via a
  # micro-PR — orchestrator decides at delivery time).
estimated_days: 2
points: 6
# Points justification (ADR-041 L4 scope):
#   L4 — PrismError::ColumnNotFound variant + Display (prism-core): 0.5 pts
#   L4 — E-QUERY-038 plan-time column gate in prism-query engine (colocated with E-QUERY-037): 1.5 pts
#   L4 — E-QUERY-038 MCP error mapping in error_mapping.rs (-32602 INVALID_PARAMS arm): 0.5 pts
#   L4 — E-QUERY-001 near_text + reference_pointer pedagogical enrichment: 0.5 pts
#   L4 — E-QUERY-002 valid_operators_for_type pedagogical enrichment: 0.5 pts
#   L4 — E-QUERY-003 how_to_fix pedagogical enrichment: 0.5 pts
#   L4 — E-QUERY-037 suggestion field update with prism_describe reference: 0.5 pts
#   L4 — normalized_pql: Chumsky AST re-serializer (net-new; largest single task) + prism-mcp
#         response envelope field + #[non_exhaustive] mark: 1.5 pts
#   Total: 6 pts
level: "L4"
status: draft
# BC status: behavioral_contracts is non-empty (3 BCs). Status remains draft until
# orchestrator schedules into a wave (Spec-First Gate S-7.01 met — all ACs trace to BCs).
document_type: story
version: "1.0"
producer: story-writer
timestamp: "2026-06-19T00:00:00Z"
input-hash: "TBD"
traces_to: [D-1241, D-1243, D-1244]
cycle: "v1.0.0-greenfield"
epic_id: "E-5"
# Epic E-5 (MCP Interface / Query Engine). Sub-story of S-DEMO-PRISMQL-ONBOARDING-001 per D-1244.
phase: 2
acceptance_criteria_count: 6
red_gate_tests: 6
tdd_mode: strict
behavioral_contracts:
  [BC-2.11.016, BC-2.11.017, BC-2.11.018]
# BC array propagation (bc_array_changes_propagate_to_body_and_acs):
# BC-2.11.016 — E-QUERY-038 column-not-found plan-time gate (cited in AC-001, AC-002)
# BC-2.11.017 — E-QUERY pedagogical enrichments (E-QUERY-001/002/003/037) (cited in AC-003, AC-004)
# BC-2.11.018 — normalized_pql field on successful query responses (cited in AC-005, AC-006)
# All 3 BCs cited in at least one AC body trace (bidirectional trace satisfied).
verification_properties: []
# VP assignments TBD — architect assigns after story decomposition.
assumption_validations: []
risk_mitigations:
  - "normalized_pql Chumsky re-serializer is net-new (no existing AST→PQL Display/normalize
     impl in prism-query/src/ast.rs as of develop@9114e028). This is the largest single
     technical task in this sub-story. The implementer MUST read filter_parser.rs, pipe_parser.rs,
     sql_parser.rs, and ast.rs before beginning Task 13. Partial display affordances exist
     (ast.rs:681, ast.rs:1099 raw display strings) — leverage but do not assume they produce
     canonicalized form. A full canonicalizing re-serializer (whitespace + keyword casing +
     alias expansion) is required per BC-2.11.018."
  - "E-QUERY-038 gate ordering: must fire AFTER E-QUERY-037 (table exists → then check columns).
     Gate ordering: E-QUERY-001 (parse) → E-QUERY-037 (table not found) → E-QUERY-038 (column
     not found). If E-QUERY-038 fires when the table is also absent, DataFusion internals may
     leak. Colocate both gates and enforce the ordering explicitly."
  - "normalized_pql MUST be absent (not null, not present) on ALL error responses. Implement via
     #[serde(skip_serializing_if = 'Option::is_none')] — NOT Option::None serializing to null.
     AC-006 test uses serde_json::Value deserialization and checks value.get('normalized_pql').is_none()."
  - "PrismError::ColumnNotFound arm in error_mapping.rs MUST be explicit (-32602 INVALID_PARAMS).
     Must NOT fall through to the #[non_exhaustive] catch-all -32000 arm. AC-001 integration
     test verifies the MCP error code is -32602, not -32000."
  - "available_columns in E-QUERY-038 sourced ENTIRELY from TableRegistry (operator TOML →
     registry). MUST NOT contain API keys, bearer tokens, URL paths, or credentials. The
     proptest-style VP candidate (E-QUERY-038 available_columns contains no credential-pattern
     strings) should be implemented as part of AC-002."
crates_touched: [prism-core, prism-query, prism-mcp]
# prism-core: new PrismError::ColumnNotFound variant + Display
# prism-query: E-QUERY-038 plan-time gate + E-QUERY-001/002/003/037 pedagogical enrichments +
#              Chumsky normalized_pql string production (net-new re-serializer)
# prism-mcp: error_mapping.rs arm for ColumnNotFound (-32602) + normalized_pql Option<String>
#             field in query response type + #[non_exhaustive] mark on response type
anchor_bcs: [BC-2.11.016, BC-2.11.017, BC-2.11.018]
anchor_subsystem: ["SS-11"]
parent_story: S-DEMO-PRISMQL-ONBOARDING-001
# This is a decomposed sub-story of S-DEMO-PRISMQL-ONBOARDING-001 (per D-1244).
# The parent story is marked superseded-by-sub-stories; see parent story for full context.
---

# S-DEMO-PRISMQL-ONBOARDING-001-B — PrismQL LLM Auto-Onboarding: Query Engine L4

**Decomposition context (D-1244):** This sub-story covers the **prism-query/prism-core side**
of the 4-layer teaching mechanism (ADR-041 v1.1): L4 (E-QUERY-038 column-not-found gate,
pedagogical enrichments for E-QUERY-001/002/003/037, and the `normalized_pql` Chumsky
re-serializer). It is separated from S-DEMO-PRISMQL-ONBOARDING-001-A (prism-mcp L1+L2+L3)
to eliminate the prism-query crate conflict with PIVOT-003 (D-1244 §Pairwise crate overlap).
This sub-story pipelines behind PIVOT-003 (also prism-query) for lowest merge friction;
S-DEMO-PRISMQL-ONBOARDING-001-A pipelines behind S-5.04 (also prism-mcp).

---

## Narrative

As a Claude Code AI agent authoring PrismQL queries for multi-client SOC investigations, I want
column-not-found errors (E-QUERY-038) that tell me exactly which columns are available and which
one I may have meant, enriched parse/type/limit errors with actionable fields, and my successful
queries echoed back in normalized form, so that I can self-correct failed queries in ≤3 retries
and build grounded query templates for the current session.

---

## Behavioral Contracts

| BC ID | Title | Key Clauses |
|-------|-------|-------------|
| BC-2.11.016 v1.0 | E-QUERY-038 Column-Not-Found Plan-Time Gate (L4) | Gate at plan time after E-QUERY-037; available_columns always present; did_you_mean present when Levenshtein ≤ 3; DI-008 org-scoped available_columns; -32602 MCP error code |
| BC-2.11.017 v1.0 | E-QUERY Pedagogical Enrichments (L4 — Codes 001, 002, 003, 037) | E-QUERY-001: near_text ≤50 chars + reference_pointer; E-QUERY-002: valid_operators_for_type list; E-QUERY-003: how_to_fix string; E-QUERY-037: suggestion contains prism_describe reference |
| BC-2.11.018 v1.0 | `normalized_pql` Field on Successful Query Responses (L4 Echo / OPD-1) | Present (non-empty) on every successful execution incl. zero-row; absent (not null, not present) on all errors; Chumsky-normalized form; excludes DataFusion plan internals |

---

## Token Budget Estimate

| Artifact | Estimated Tokens |
|----------|-----------------|
| This story spec | ~3,500 |
| BC-2.11.016 v1.0 | ~1,200 |
| BC-2.11.017 v1.0 | ~1,000 |
| BC-2.11.018 v1.0 | ~800 |
| ADR-041 v1.1 §L4 (pedagogical errors + normalized_pql sections) | ~3,000 |
| `crates/prism-core/src/error.rs` (PrismError; ColumnNotFound variant) | ~500 |
| `crates/prism-query/src/engine.rs` (E-QUERY-037 gate area; E-QUERY-038 gate addition) | ~3,000 |
| `crates/prism-query/src/ast.rs` (AST types; display affordances at lines 681, 1099) | ~2,000 |
| `crates/prism-query/src/filter_parser.rs` + `pipe_parser.rs` + `sql_parser.rs` (Chumsky parsers) | ~4,000 |
| `crates/prism-mcp/src/error_mapping.rs` (existing MCP error arms) | ~800 |
| `crates/prism-mcp/src/server.rs` query response type area (normalized_pql field) | ~1,000 |
| Test files (6 stubs × ~80 lines each) | ~1,500 |
| Tool outputs (nextest, clippy) | ~1,000 |
| **Total estimate** | **~23,300** |

At ~200k context window: ~11.7% — within the 20-30% ceiling.

---

## Tasks

### Pre-flight: read substrate before writing anything

- [ ] Read `crates/prism-core/src/error.rs` — locate `PrismError` enum; confirm `#[non_exhaustive]`
  already present at enum level; identify `result_large_err` clippy lint pattern (TableNotAvailableDetails
  precedent for boxing large variants)
- [ ] Read `crates/prism-query/src/engine.rs` — find where E-QUERY-037 plan-time gate fires;
  identify the colocated injection point for E-QUERY-038; understand the `TableRegistry` call pattern
- [ ] Read `crates/prism-query/src/ast.rs` fully — note display affordances at lines 681 and 1099;
  identify which AST nodes have Display impls and which need net-new canonicalizing re-serialization
- [ ] Read `crates/prism-query/src/filter_parser.rs`, `pipe_parser.rs`, `sql_parser.rs` — understand
  Chumsky 0.12.0 parse output shape; confirm the parsed AST is the same type for all three modes
- [ ] Read `crates/prism-mcp/src/error_mapping.rs` — confirm the catch-all `-32000` arm uses
  `#[non_exhaustive]` match; identify insertion point for the explicit `-32602 ColumnNotFound` arm
- [ ] Read `crates/prism-query/Cargo.toml` — confirm `strsim = "0.11"` is a direct dep
  (D-1163 precedent); confirm `chumsky` version is 0.12.0

### Phase 1 — PrismError::ColumnNotFound variant

- [ ] Write failing test (FAIL first): implicit via test_BC_2_11_016_e_query_038_did_you_mean —
  the test expects E-QUERY-038 JSON; the variant must compile to run the test
- [ ] Add to `crates/prism-core/src/error.rs`:
  ```rust
  ColumnNotFound {
      column: String,
      table: String,
      client_id: String,
      available_columns: Vec<String>,
      did_you_mean: Option<String>,
  }
  ```
  NOTE: `PrismError` enum is already `#[non_exhaustive]` — NO new annotation needed at enum level.
  Verify `result_large_err` clippy lint: if this variant triggers it, box the Vec fields following
  `TableNotAvailableDetails` precedent.
- [ ] Implement `Display` for the new variant:
  `"E-QUERY-038: column '{}' not found in table '{}' for client '{}'"`
- [ ] Verify crate compiles; check for result_large_err lint

### Phase 2 — E-QUERY-038 MCP error mapping

- [ ] In `crates/prism-mcp/src/error_mapping.rs`, add explicit arm BEFORE the `#[non_exhaustive]`
  catch-all:
  - `PrismError::ColumnNotFound { ... }` → MCP error code `-32602` (INVALID_PARAMS)
  - Structured response per BC-2.10.007 format: `code: "E-QUERY-038"`, `category: "validation"`,
    `severity: "broken"`, `retryable: false`
  - `suggestion`: "Call prism_describe('<client_id>') to see available columns, or use the
    available_columns field in this error to correct the column name."
  - Payload fields: `column`, `table`, `client_id`, `available_columns`, `did_you_mean`
    (omit `did_you_mean` if None — NOT null, NOT empty string)
- [ ] Verify existing match arm ordering is preserved; `-32000` catch-all still last

### Phase 3 — E-QUERY-038 plan-time column gate

- [ ] Write failing tests 1, 2 (FAIL first):
  `test_BC_2_11_016_e_query_038_did_you_mean`
  `test_BC_2_11_016_e_query_038_org_scoped_available_columns`
- [ ] In `crates/prism-query/src/engine.rs` (or equivalent plan validation step), colocated with
  E-QUERY-037 gate:
  - Gate fires AFTER E-QUERY-037 passes (table exists → check columns)
  - Gate ordering: E-QUERY-001 (parse) → E-QUERY-037 (table not found) → E-QUERY-038 (column not found)
  - Column availability checked against `TableRegistry` for `(table, OrgId)` pair (same lookup
    pattern as E-QUERY-037 per D-1163)
  - `available_columns` is ALWAYS present (empty `[]` if table has zero columns); org-scoped per DI-008
  - `did_you_mean`: `strsim::levenshtein` against all available columns; present when distance ≤ 3
    (same crate as E-QUERY-037's did_you_mean per D-1163); absent (field omitted) when no match ≤ 3
  - Injection-safety: `available_columns` from `TableRegistry` only; MUST NOT contain credential
    values, API key substrings, or URL strings
  - Audit: rejection included in `AuditEntry` for the `query` tool call (`outcome: "rejected"`,
    `reason: "column_not_found"`)
- [ ] Verify tests 1, 2 pass

### Phase 4 — E-QUERY pedagogical enrichments (additive fields)

- [ ] Write failing test 3 (FAIL first): `test_BC_2_11_017_pedagogical_enrichments`
- [ ] E-QUERY-001 enrichment (parse error):
  - Additive fields: `near_text: String` (offending token from Chumsky error context, ≤50 chars;
    empty string `""` if parser cannot provide a token); `reference_pointer: "prismql://reference"`
  - Display string UNCHANGED; injection-safety: near_text is model's own PQL input, truncated ≤50 chars
- [ ] E-QUERY-002 enrichment (type error):
  - Additive field: `valid_operators_for_type: Vec<String>` (compile-time table per ColumnType)
    String → `["=", "!=", "LIKE", "IN", "NOT IN"]`; Integer → `["=", "!=", "<", ">", "<=", ">=", "BETWEEN", "IN", "NOT IN"]`;
    Float → `["=", "!=", "<", ">", "<=", ">=", "BETWEEN"]`; Boolean → `["=", "!="]`;
    Datetime → `["=", "!=", "<", ">", "<=", ">=", "BETWEEN"]`; Json → `["=", "!="]` minimum
  - Helper: `fn valid_operators_for_type(t: ColumnType) -> &'static [&'static str]`
  - Display string UNCHANGED
- [ ] E-QUERY-003 enrichment (security limits):
  - Additive field: `how_to_fix: String` from `limit_detail` category match:
    Query size > 64KB → "Shorten the query. Remove large IN (...) lists or break into multiple queries."
    Nesting depth > 64 → "Flatten nested conditions. Use AND/OR instead of deeply nested parentheses."
    Pipe stage count > 32 → "Reduce the number of pipe stages. Combine adjacent filter conditions."
    Regex > 1024 bytes → "Use a shorter regex pattern. Consider using LIKE instead of regex for simple pattern matching."
    Expanded query > 64KB → "The alias expansion produced a query over 64KB. Simplify the aliased query or use a narrower alias."
    Catch-all → "Simplify or shorten the query."
  - `PrismError::QuerySecurityLimitExceeded { detail }` variant UNCHANGED; `how_to_fix` computed at error-map time from `detail` string
- [ ] Write failing test 4 (FAIL first): `test_BC_2_11_017_e_query_037_suggestion_prism_describe`
- [ ] E-QUERY-037 suggestion field update (existing error handler):
  - When `did_you_mean` present: "Call prism_describe('<client_id>') to see available tables and columns. If you meant '<did_you_mean_value>', retry with that table name."
  - When `did_you_mean` absent: "Call prism_describe('<client_id>') to see available tables and columns for this client."
  - `available_sensors`, `available_tables`, `did_you_mean` fields UNCHANGED; only `suggestion` text updated
- [ ] Verify tests 3, 4 pass

### Phase 5 — normalized_pql Chumsky re-serializer

- [ ] Write failing test 5 (FAIL first): `test_BC_2_11_018_normalized_pql_present_on_success_absent_on_error`
  (combined: present on success including zero-row; absent on E-QUERY-037/038/001 error)
- [ ] Investigate AST re-serialization options:
  - Read ast.rs lines ~681 and ~1099 for existing display affordances
  - Determine if a `Display` impl on the top-level AST node can produce canonical PQL
  - If partial Display impls exist, extend them; if none, build a full canonicalizing
    `PqlNormalizer` that walks the AST producing whitespace-normalized, uppercase-keyword form
- [ ] Implement `normalized_pql` string production in `prism-query`:
  - Source: validated + canonicalized PQL string from Chumsky parse output
  - Keyword casing: SELECT, FROM, WHERE, GROUP BY, ORDER BY, LIMIT, AND, OR, IN, etc. — uppercase
  - Whitespace normalization: single space between tokens; no trailing whitespace
  - The normalized string MUST round-trip through Chumsky (parse to same AST as original)
  - EXCLUDED: DataFusion plan node strings (HashJoin, TableScan, SortExec, Aggregate, etc.)
    cost estimates, join-order, partition/pushdown details
  - If normalization produces empty string (should not happen for valid parse): return `None`
    (field omitted per BC-2.11.018 Error Cases)
- [ ] Add `normalized_pql: Option<String>` field to query response type in `crates/prism-mcp/src/server.rs`:
  - Response type MUST be `#[non_exhaustive]`; add `#[non_exhaustive]` if not already present
  - If adding `#[non_exhaustive]` to query response type: increment `ci.yml EXPECTED` by 1 more
    (beyond the +3 from 001-A; coordinate with 001-A on final EXPECTED count)
  - Field serialized with `#[serde(skip_serializing_if = "Option::is_none")]` — absent on error,
    not null
  - Field presence rules: PRESENT (non-empty) on every successful execution (incl. zero-row,
    partial sensor failure with query-level success); ABSENT on ALL error responses
- [ ] Verify test 5 passes

### Phase 6 — final gates

- [ ] SAP-1 probe: `rg 'event_type\s*=' crates/ --type rust` — verify new emissions have
  BC-2.16.002 catalog rows. Expected new event_type:
  `"column_not_found.rejected"` (E-QUERY-038 gate fired)
- [ ] Verify normalized_pql value does NOT contain DataFusion plan node substrings
  (`HashJoin`, `TableScan`, `SortExec`, `Aggregate`): add assertion in test 5
- [ ] Run `just check` — all 6 Red Gate tests pass; zero clippy warnings; fmt clean
- [ ] If query response type newly marked `#[non_exhaustive]`: update ci.yml EXPECTED and
  scripts/check-non-exhaustive.sh; coordinate with 001-A on final combined EXPECTED value

---

## Acceptance Criteria

### AC-001 — E-QUERY-038 gate payload shape (did_you_mean + absent-when-no-match + gate ordering)
(traces to BC-2.11.016 postconditions — Gate firing conditions, E-QUERY-038 payload shape; EC-11-039, EC-11-040, EC-11-043)

Given `query("SELECT sevrity FROM crowdstrike_alerts", clients=["acme"])` where `severity` is
a registered column but `sevrity` is not,
when executed,
then an `E-QUERY-038` error is returned as MCP `-32602 INVALID_PARAMS` with: `code: "E-QUERY-038"`,
`column: "sevrity"`, `table: "crowdstrike_alerts"`, `client_id: "acme"`,
`available_columns` is a non-empty array including `"severity"`,
`did_you_mean: "severity"` (Levenshtein distance 1);
when `query("SELECT completely_bogus_col FROM crowdstrike_alerts", clients=["acme"])` is executed
where no column is within distance 3,
then `E-QUERY-038` is returned with `available_columns` non-empty and `did_you_mean` field ABSENT
(not null — the field must not appear in the JSON);
when `query("SELECT * FROM nonexistent_table WHERE bogus_col = 1", clients=["acme"])` is executed,
then `E-QUERY-037` fires (not E-QUERY-038) — gate ordering is enforced.

Red Gate: `test_BC_2_11_016_e_query_038_did_you_mean`

### AC-002 — E-QUERY-038 org-scoped available_columns (DI-008)
(traces to BC-2.11.016 invariant DI-008; BC-2.11.016 Canonical Test Vectors — org-isolation)

Given `query("SELECT * FROM crowdstrike_alerts WHERE sevrity = 'high'", clients=["acme"])`
in a multi-tenant deployment where "globex" also has a `crowdstrike_alerts` table with a
`severity` column,
when the error is inspected,
then `available_columns` contains ONLY "acme"'s `crowdstrike_alerts` columns — "globex"'s column
names do not appear.

Red Gate: `test_BC_2_11_016_e_query_038_org_scoped_available_columns`

### AC-003 — E-QUERY-001/002/003 pedagogical enrichments
(traces to BC-2.11.017 postconditions — E-QUERY-001 near_text + reference_pointer; E-QUERY-002 valid_operators_for_type; E-QUERY-003 how_to_fix)

Given `query("SELCT * FROM crowdstrike_alerts")` (parse error — typo in SELECT),
when the error is inspected,
then E-QUERY-001 response contains additive fields: `near_text: "SELCT"` (the offending token,
≤50 chars) and `reference_pointer: "prismql://reference"` (literal string);
given `query("SELECT * FROM events WHERE severity > 5")` with `severity` as a `String` column,
when the error is inspected,
then E-QUERY-002 response contains additive field `valid_operators_for_type: ["=", "!=", "LIKE",
"IN", "NOT IN"]` (String operators);
given an E-QUERY-003 query size violation (query > 64KB),
when the error is inspected,
then E-QUERY-003 response contains additive field `how_to_fix` as a non-empty string
("Shorten the query. Remove large IN (...) lists or break into multiple queries.").

Red Gate: `test_BC_2_11_017_pedagogical_enrichments`

### AC-004 — E-QUERY-037 suggestion field update with prism_describe reference
(traces to BC-2.11.017 postcondition — E-QUERY-037 suggestion field update; EC-11-049, EC-11-050)

Given `query("SELECT * FROM crowdstrike_alert")` when `crowdstrike_alerts` is registered
(1-char table name typo),
when the error is inspected,
then E-QUERY-037 `suggestion` field contains the substring `"prism_describe"` AND a retry hint
referencing `"crowdstrike_alerts"`;
given `query("SELECT * FROM completely_made_up_table")` where no close match exists,
when the error is inspected,
then E-QUERY-037 `suggestion` field contains `"prism_describe"` but NO retry hint for a
specific table name.

Red Gate: `test_BC_2_11_017_e_query_037_suggestion_prism_describe`

### AC-005 — normalized_pql present on success (including zero-row and partial failure)
(traces to BC-2.11.018 postconditions — Field presence on success, Wire field name, Field content, normalization, zero-rows-success)

Given a successful `query("SELECT * FROM crowdstrike_alerts WHERE severity = 'high' LIMIT 10", clients=["acme"])`,
when the response is inspected,
then `normalized_pql` is present as a non-empty string that contains `"crowdstrike_alerts"` and
resembles a valid PQL query (starts with `SELECT` or `FROM`); the field value does NOT contain
any DataFusion plan node strings (`HashJoin`, `TableScan`, `SortExec`, `Aggregate`);
when `query("select * from crowdstrike_alerts limit 5")` (lowercase) is submitted and succeeds,
then `normalized_pql` contains uppercase canonicalized form (e.g., `SELECT * FROM
crowdstrike_alerts LIMIT 5`);
when a query returns zero rows (but succeeds),
then `normalized_pql` is still PRESENT.

Red Gate: `test_BC_2_11_018_normalized_pql_present_on_success_absent_on_error`

### AC-006 — normalized_pql absent on all error responses
(traces to BC-2.11.018 postconditions — Absent on error, partial failure treatment; EC-11-053, EC-11-054)

Given a failed query (E-QUERY-037 table unavailable OR E-QUERY-038 column not found OR
E-QUERY-001 parse error),
when the error response is inspected,
then `normalized_pql` field is ABSENT — the field does NOT appear in the JSON object
(not null, not empty string, not present-with-any-value);
given a partially successful query with `sensor_errors` non-empty but query-level success,
when the response is inspected,
then `normalized_pql` IS present.

Red Gate: combined in `test_BC_2_11_018_normalized_pql_present_on_success_absent_on_error`

---

## Red Gate Test Names

| # | Test Name | AC | Crate | Behavior Asserted |
|---|-----------|----|----|-------------------|
| 1 | `test_BC_2_11_016_e_query_038_did_you_mean` | AC-001 | prism-query | E-QUERY-038 with Levenshtein-1 typo → did_you_mean present; no-match typo → did_you_mean absent; table-not-found → E-QUERY-037 not E-QUERY-038 |
| 2 | `test_BC_2_11_016_e_query_038_org_scoped_available_columns` | AC-002 | prism-query | Multi-tenant: available_columns for acme contains only acme columns; globex columns absent |
| 3 | `test_BC_2_11_017_pedagogical_enrichments` | AC-003 | prism-query | E-QUERY-001 near_text + reference_pointer; E-QUERY-002 valid_operators_for_type; E-QUERY-003 how_to_fix |
| 4 | `test_BC_2_11_017_e_query_037_suggestion_prism_describe` | AC-004 | prism-query | E-QUERY-037 suggestion always contains "prism_describe"; with did_you_mean → retry hint included |
| 5 | `test_BC_2_11_018_normalized_pql_present_on_success_absent_on_error` | AC-005 + AC-006 | prism-mcp | normalized_pql present on success (incl. zero-row, partial failure); absent (not null, not present) on all error types; no DataFusion plan node strings in value |
| 6 | (implicit) | AC-001 | prism-mcp | E-QUERY-038 mapped to -32602 INVALID_PARAMS in error_mapping.rs; does not fall through to -32000 catch-all |

---

## Architecture Mapping

| Component | Module | Crate | Pure/Effectful |
|-----------|--------|-------|----------------|
| `PrismError::ColumnNotFound` variant | SS-11 | prism-core (`error.rs`) | Pure (type definition) |
| E-QUERY-038 plan-time column gate | SS-11 | prism-query (`engine.rs`) | Pure (plan-time validation against TableRegistry snapshot) |
| E-QUERY-038 MCP error mapping | SS-10 | prism-mcp (`error_mapping.rs`) | Pure (error translation) |
| E-QUERY-001/002/003/037 pedagogical enrichments | SS-11 | prism-query (error builder / error-map time) | Pure (additive field computation) |
| Chumsky AST re-serializer (`normalized_pql` source) | SS-11 | prism-query (AST Display or PqlNormalizer) | Pure (string production from parsed AST) |
| `normalized_pql` response envelope field | SS-10 | prism-mcp (`server.rs` query response type) | Effectful (MCP response construction) |

Subsystem anchor justification: SS-11 (Query Execution Engine) owns the query-engine L4 work
(E-QUERY-038 gate, error enrichments, Chumsky normalizer) per ARCH-INDEX Subsystem Registry.
The prism-mcp touches in this sub-story (`error_mapping.rs` arm + `normalized_pql` field) are
wiring concerns; they belong to SS-10 but are included here because the prism-query work is the
primary scope and the prism-mcp changes are thin wires driven by the query-engine changes.

---

## Previous Story Intelligence

**S-3.13 (MERGED — Dynamic Table Availability / TableRegistry):** `Arc<dyn TableRegistry>` is
wired into `PrismServer`. `TableRegistry::registered_tables()` returns `Vec<String>`.
`E-QUERY-037` (table-not-found gate) uses `TableRegistry::filter_to_org_visible()` — E-QUERY-038
column gate uses the same lookup pattern. `strsim = "0.11"` is a **direct** dependency of
`crates/prism-query/Cargo.toml` (line 84, D-1163; resolves 0.11.1) — NO new dependency needed
for E-QUERY-038's `did_you_mean`.

**PIVOT-003 (sibling story — prism-query):** PIVOT-003 adds IOC columns to Cyberint/CrowdStrike
TOML specs and updates the sensor column schemas. After PIVOT-003 merges, `TableRegistry` for
clients with those sensors will contain the new IOC column names. This expands the `available_columns`
set returned by E-QUERY-038 — a data dependency at demo time, not a code dependency. Pipeline
this sub-story behind PIVOT-003 for smooth rebase (D-1244).

**S-DEMO-PRISMQL-ONBOARDING-001-A (sibling sub-story — prism-mcp L1+L2+L3):** The `prism_describe`
pointer appearing in the E-QUERY-037 `suggestion` field (AC-004) and the `query_tutorial` Step 3
error field names (`near_text`, etc.) are cross-sub-story references. The server implements
`prism_describe` in 001-A; this sub-story adds the references in error text. Merge order is
flexible — error text referencing a not-yet-registered tool is acceptable at demo time as long
as both sub-stories merge before T13 capstone.

**ast.rs Chumsky-normalized PQL (critical pre-flight):** CONFIRMED 2026-06-19: `prism-query/src/ast.rs`
has NO existing `Display`/`to_pql`/`normalize` impl at the top-level AST node. Lines 681 and 1099
carry raw display strings for some sub-nodes. The implementer MUST build the canonicalizing
re-serializer (likely a new `impl Display for SelectStatement` or a `PqlNormalizer` visitor
pattern). This is the largest single task in this sub-story — budget appropriately.

---

## Architecture Compliance Rules

| Rule | Source | Enforcement |
|------|--------|-------------|
| E-QUERY-038 gate fires AFTER E-QUERY-037 (table must exist before checking columns) | BC-2.11.016 precondition | AC-001 "table-not-found → E-QUERY-037 not E-QUERY-038" test |
| Gate ordering explicit: E-QUERY-001 → E-QUERY-037 → E-QUERY-038 | ADR-041 §L4 + BC-2.11.016 | Adversary: read engine.rs validation step ordering |
| `PrismError::ColumnNotFound` arm MUST be explicit -32602 (not -32000 catch-all fallthrough) | BC-2.11.016 + BC-2.10.007 | Red Gate test 6 (MCP error code assertion) |
| `available_columns` sourced ENTIRELY from TableRegistry; MUST NOT contain credentials | BC-2.11.016 invariant + DI-008 | AC-002 multi-tenant test |
| `normalized_pql` MUST NOT contain DataFusion plan node type strings | BC-2.11.018 postcondition | AC-005 assertion in test 5 |
| `normalized_pql` MUST be absent (not null) on error responses | BC-2.11.018 invariant | AC-006: use `value.get("normalized_pql").is_none()` — not null check |
| `near_text` truncated to ≤50 chars (DI-006: prevents raw PQL relay as error context) | BC-2.11.017 postcondition | AC-003 near_text length assertion |
| `normalized_pql` round-trips through Chumsky (same AST) | BC-2.11.018 postcondition | Implementer verifies; proptest VP candidate |
| All new `event_type=` tracing emissions require BC-2.16.002 catalog rows (SAP-1) | CLAUDE.md §SAP-1 | Adversary SAP-1 probe |
| `PrismError` enum is already `#[non_exhaustive]` — do NOT add it again at enum level | CLAUDE.md §Conventions | Adversary: confirm `#[non_exhaustive]` at enum (not at variant) |

**Forbidden patterns:**
- `normalized_pql: null` in JSON (must be absent, not null — use `skip_serializing_if = "Option::is_none"`)
- E-QUERY-038 returning before E-QUERY-037 check (wrong gate ordering)
- `available_columns` containing API key, bearer token, or URL patterns
- `near_text` > 50 characters
- DataFusion plan node strings (`HashJoin`, `TableScan`, `SortExec`, `Aggregate`) in `normalized_pql`

---

## Library & Framework Requirements

| Library | Version | Usage |
|---------|---------|-------|
| prism-core | workspace | PrismError (new ColumnNotFound variant), ColumnType (valid_operators_for_type helper) |
| strsim | 0.11 (direct dep in prism-query/Cargo.toml line 84; resolves 0.11.1) | `strsim::levenshtein` for did_you_mean in E-QUERY-038 — NO new dep needed |
| chumsky | 0.12.0 (workspace) | Parsed AST source for normalized_pql re-serialization |
| serde / serde_json | 1.x (workspace) | `normalized_pql: Option<String>` with `skip_serializing_if = "Option::is_none"` |
| tracing | workspace | Structured event emission; `column_not_found.rejected` event_type must be in BC-2.16.002 |

**Version pinning note:** `strsim = "0.11"` CONFIRMED as direct dep at `crates/prism-query/Cargo.toml`
line 84 (resolves 0.11.1) — NO new dependency. `chumsky 0.12.0` is the resolved version for all three
parser files. `datafusion 53.1.0` is the backing execution engine — do NOT add DataFusion internal
import to get normalized query strings.

---

## File Structure Requirements

| File | Action | Purpose |
|------|--------|---------|
| `crates/prism-core/src/error.rs` | Modify | Add `PrismError::ColumnNotFound { column, table, client_id, available_columns, did_you_mean }` variant; implement Display |
| `crates/prism-query/src/engine.rs` | Modify | (1) E-QUERY-038 column-not-found plan-time gate (colocated with E-QUERY-037 gate); (2) E-QUERY-001 near_text + reference_pointer enrichment; (3) E-QUERY-002 valid_operators_for_type enrichment; (4) E-QUERY-003 how_to_fix enrichment; (5) E-QUERY-037 suggestion text update; (6) normalized_pql string production via Chumsky normalizer |
| `crates/prism-query/src/ast.rs` | Modify | Add `Display` impl or `PqlNormalizer` visitor on top-level AST node for canonicalized PQL re-serialization (net-new; leverage existing display affordances at lines 681, 1099) |
| `crates/prism-mcp/src/error_mapping.rs` | Modify | Add explicit `-32602 INVALID_PARAMS` arm for `PrismError::ColumnNotFound` before the `#[non_exhaustive]` catch-all |
| `crates/prism-mcp/src/server.rs` | Modify | Add `normalized_pql: Option<String>` to query response type; add `#[non_exhaustive]` to response type if not already present |
| `ci.yml` | Modify | Increment `EXPECTED` if query response type is newly `#[non_exhaustive]` (coordinate with 001-A which already increments to 82 for +3 types; this sub-story adds +0 or +1 depending on response type state) |
| `crates/prism-query/tests/e_query_pedagogical.rs` | Create | Tests for AC-001 through AC-004 (E-QUERY-038 gate, E-QUERY-001/002/003/037 enrichments) |
| `crates/prism-mcp/tests/normalized_pql.rs` | Create | Tests for AC-005, AC-006 (normalized_pql field presence/absence) |

---

## Edge Cases

| ID | Source | Description | Expected Behavior |
|----|--------|-------------|-------------------|
| EC-001 | BC-2.11.016 EC-11-039 | E-QUERY-038 — table has zero columns | `available_columns: []`; `did_you_mean` absent |
| EC-002 | BC-2.11.016 EC-11-044 | E-QUERY-038 — multiple invalid columns in same query | At minimum one E-QUERY-038 returned (fail-fast or collect-all — implementer choice) |
| EC-003 | BC-2.11.017 EC-11-046 | E-QUERY-001 at end-of-input (incomplete query) | `near_text: ""` (empty string); `reference_pointer: "prismql://reference"` still present |
| EC-004 | BC-2.11.017 EC-11-047 | E-QUERY-002 for `ColumnType::Json` operator | `valid_operators_for_type` includes at minimum `["=", "!="]` |
| EC-005 | BC-2.11.017 EC-11-048 | E-QUERY-003 with unrecognized limit category | `how_to_fix: "Simplify or shorten the query."` catch-all |
| EC-006 | BC-2.11.018 EC-11-055 | normalized_pql — pipe-mode query succeeds | Normalized pipe-mode string returned |
| EC-007 | BC-2.11.018 EC-11-053 | normalized_pql — query times out (E-QUERY-004) | `normalized_pql` ABSENT |
| EC-008 | BC-2.11.018 Error Cases | Chumsky normalization produces empty string (shouldn't happen for valid parse) | OMIT field (`None`) — do not emit `normalized_pql: ""` |

---

## Structured Event Catalog Obligation (BC-2.16.002 / PG-LP11-001)

New `event_type` values added by this story MUST have BC-2.16.002 catalog rows before PR merges:
- `event_type = "column_not_found.rejected"` (E-QUERY-038 gate fired)

Implementer: `rg 'event_type\s*=' crates/ --type rust` before declaring done (SAP-1).

---

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.0 | D-1244-decomposition-2026-06-19 | 2026-06-19 | story-writer | Initial sub-story decomposition — split from S-DEMO-PRISMQL-ONBOARDING-001 (13 pts) per D-1244 §Parallel Execution Plan. Covers L4 query-engine surfaces (prism-core + prism-query + prism-mcp wire). 3 BCs: BC-2.11.016, BC-2.11.017, BC-2.11.018. 6 ACs + 6 Red Gate tests. 6 pts. Pipelines behind PIVOT-003 for crate-conflict avoidance. |
