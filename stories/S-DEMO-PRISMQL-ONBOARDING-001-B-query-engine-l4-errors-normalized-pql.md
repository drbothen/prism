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
#   Architectural Surface table. The prism-mcp response envelope for normalized_pql (conditional
#   Value key insertion into PrismServer::query inline payload) is a thin wire between SS-11 output and SS-10 surface;
#   it is included in this sub-story because the prism-query normalizer is the hard work.
priority: P0
# P0: DEMO-BLOCKING per D-1243 (S-DEMO-PRISMQL-ONBOARDING-001 set). The pedagogical E-QUERY-038
# error + normalized_pql field are required for Claude to self-correct PrismQL queries in the
# multi-client SOC demo capstone.
depends_on:
  - S-5.03
  # S-5.03 (MERGED — provides ServerHandler override patterns for MCP tools/resources/prompts;
  # the prism-mcp wire edits in this sub-story (error_mapping.rs arm, normalized_pql field)
  # follow the patterns S-5.03 established).
  - S-3.13
  # S-3.13 (MERGED — wires `TableRegistry` into `QueryEngine`; establishes the
  # `check_availability_gate(query_str, org_scope, resolved_spec_map)` pattern and org-scope filter
  # helpers that E-QUERY-038 extends for column-level checking. The `resolved_spec_map` parameter
  # already flows through `check_table_availability` and into `check_availability_gate` —
  # E-QUERY-038 reads column data from this same parameter, not from `TableRegistry` itself).
# Dependency anchors:
#   S-DEMO-PRISMQL-ONBOARDING-001-B depends on S-5.03 because S-5.03 completed the MCP server
#     handler patterns (ServerHandler overrides) that inform this sub-story's prism-mcp wire edits.
#   S-DEMO-PRISMQL-ONBOARDING-001-B depends on S-3.13 because the E-QUERY-038 plan-time column
#     gate extends the `check_availability_gate` function and org-scope filter pattern established
#     by S-3.13; column data flows from `resolved_spec_map` (already a parameter in that gate),
#     not from `TableRegistry` itself.
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
#         conditional Value key insertion into PrismServer::query inline payload: 1.5 pts
#   Total: 6 pts
level: "L4"
status: draft
# BC status: behavioral_contracts is non-empty (3 BCs). Status remains draft until
# orchestrator schedules into a wave (Spec-First Gate S-7.01 met — all ACs trace to BCs).
document_type: story
version: "2.2"
updated: "2026-07-08"
producer: story-writer
timestamp: "2026-06-28T00:00:00Z"
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
     impl in prism-query/src/ast.rs as of develop@f6739764 — VERIFIED 2026-06-20: a workspace
     grep for `impl Display`/`to_pql`/`normalize`/`to_canonical` on ast.rs AST node types returns
     ZERO matches; chumsky 0.12.0 provides NO built-in AST pretty-printer/re-serializer, confirmed
     via docs.rs + Perplexity research 2026-06-20). This is the largest single technical task in
     this sub-story. The implementer MUST read filter_parser.rs, pipe_parser.rs, sql_parser.rs,
     and ast.rs before beginning the normalizer task. NOTE: prior spec text cited 'partial display
     affordances at ast.rs:681, ast.rs:1099' — that citation is RETRACTED (remove-uncertainty pass
     2026-06-20): those lines are doc-comments on `SourceRef` and `TimestampLiteral`, not Display
     affordances; there are no AST-node Display impls to leverage. A full canonicalizing
     re-serializer (whitespace + keyword casing + alias expansion) is required net-new per
     BC-2.11.018."
  - "E-QUERY-038 gate ordering: must fire AFTER E-QUERY-037 (table exists → then check columns).
     Gate ordering: E-QUERY-001 (parse) → E-QUERY-037 (table not found) → E-QUERY-038 (column
     not found). If E-QUERY-038 fires when the table is also absent, DataFusion internals may
     leak. Colocate both gates and enforce the ordering explicitly."
  - "normalized_pql MUST be absent (not null, not present) on ALL error responses.
     Mechanism: conditional key insertion into the inline `serde_json::Value` payload
     in `PrismServer::query` — when `normalized_pql_str` is `None`, no key is inserted
     and the field is absent from the JSON output. `#[serde(skip_serializing_if)]` is
     a serde STRUCT attribute and does NOT apply to `serde_json::Value` construction.
     The absent-on-error guarantee is structurally enforced: the error path returns via
     `prism_error_to_structured_call_result` before the payload `Value` is built.
     AC-006 test: deserialize the response as `serde_json::Value` and assert
     `value.get(\"normalized_pql\").is_none()` — semantics and test wording UNCHANGED."
  - "PrismError::ColumnNotFound arm in error_mapping.rs MUST be explicit (-32602 INVALID_PARAMS).
     Must NOT fall through to the #[non_exhaustive] catch-all -32000 arm. AC-001 integration
     test verifies the MCP error code is -32602, not -32000."
  - "available_columns in E-QUERY-038 sourced from operator TOML specs via `resolved_spec_map →
     ResolvedSensorSpec.spec.tables → TableSpec.columns → ColumnSpec.name` (the same TOML specs
     that populate TableRegistry's table-name strings). `ColumnSpec.name` is an operator-defined
     schema field name from the TOML spec (e.g., `\"severity\"`, `\"host_name\"`). MUST NOT
     contain API keys, bearer tokens, URL paths, or credentials — which it cannot, because column
     names are operator-specified strings in TOML, not API response data. The proptest-style VP
     candidate (E-QUERY-038 available_columns contains no credential-pattern strings) should be
     implemented as part of AC-002."
crates_touched: [prism-core, prism-query, prism-mcp]
# prism-core: new PrismError::ColumnNotFound variant + Display
# prism-query: E-QUERY-038 plan-time gate + E-QUERY-001/002/003/037 pedagogical enrichments +
#              Chumsky normalized_pql string production (net-new re-serializer)
# prism-mcp: error_mapping.rs arm for ColumnNotFound (-32602) + normalized_pql conditional
#             Value key insertion into existing inline json! payload in PrismServer::query
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
| BC-2.11.016 v1.7 | E-QUERY-038 Column-Not-Found Plan-Time Gate (L4) | Gate at plan time after E-QUERY-037; available_columns always present; did_you_mean present when Levenshtein ≤ 3; DI-008 org-scoped available_columns; -32602 MCP error code; gate expanded to twelve positions (Filter/Pipe/SqlPipe predicate + sort/stats/project positions) incl. HAVING (6th position, parity with WHERE/GROUP BY) |
| BC-2.11.017 v1.3 | E-QUERY Pedagogical Enrichments (L4 — Codes 001, 002, 003, 037) | E-QUERY-001: near_text ≤50 chars + reference_pointer; E-QUERY-002: valid_operators_for_type list; E-QUERY-003: how_to_fix string; E-QUERY-037: suggestion contains prism_describe reference |
| BC-2.11.018 v1.2 | `normalized_pql` Field on Successful Query Responses (L4 Echo / OPD-1) | Present (non-empty) on every successful execution incl. zero-row; absent (not null, not present) on all errors; Chumsky-normalized form; excludes DataFusion plan internals |

---

## Token Budget Estimate

| Artifact | Estimated Tokens |
|----------|-----------------|
| This story spec | ~3,500 |
| BC-2.11.016 v1.7 | ~1,200 |
| BC-2.11.017 v1.3 | ~1,000 |
| BC-2.11.018 v1.2 | ~800 |
| ADR-041 v1.1 §L4 (pedagogical errors + normalized_pql sections) | ~3,000 |
| `crates/prism-core/src/error.rs` (PrismError; ColumnNotFound variant) | ~500 |
| `crates/prism-query/src/engine.rs` (E-QUERY-037 gate area; E-QUERY-038 gate addition) | ~3,000 |
| `crates/prism-query/src/ast.rs` (AST types; NO existing Display impls — net-new normalizer) | ~2,000 |
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
- [ ] Read `crates/prism-query/src/ast.rs` fully — VERIFIED 2026-06-20: NO Display/to_pql/normalize
  impls exist on any AST node type (zero grep matches); a full net-new canonicalizing re-serializer
  is required (prior "lines 681/1099 display affordances" text was RETRACTED in v1.1 — those are
  doc-comments on SourceRef/TimestampLiteral, not Display impls)
- [ ] Read `crates/prism-query/src/filter_parser.rs`, `pipe_parser.rs`, `sql_parser.rs` — understand
  the chumsky 0.12.0 parse output shape; confirm the parsed AST is the same type for all three modes.
  NOTE (remove-uncertainty 2026-06-20): the E-QUERY-001 parse-error variant is
  `PrismError::QueryParseFailed { offset: usize, detail: String }` (NOT `{ pos, message }` — the
  taxonomy Message-Format prose says "at position {pos}: {message}" but the live Display is
  `"E-QUERY-001: query parse error at offset {offset}: {detail}"`). The E-QUERY-001 `near_text`
  enrichment is ADDITIVE (does not change the Display); source the offending token slice from the
  chumsky error span (Simple/Rich `span()` → start/end indices over the input — confirmed docs.rs
  2026-06-20).
- [ ] Read `crates/prism-mcp/src/error_mapping.rs` — confirm the catch-all `-32000` arm uses
  `#[non_exhaustive]` match; identify insertion point for the explicit `-32602 ColumnNotFound` arm
- [ ] Read `crates/prism-query/Cargo.toml` — confirm `strsim = "0.11"` is a direct dep
  (D-1163 precedent; verified present 2026-06-20, resolves 0.11.1 per Cargo.lock); confirm
  `chumsky = "0.12"` (resolves 0.12.0 per Cargo.lock — verified 2026-06-20)

### Phase 1 — PrismError::ColumnNotFound variant

- [ ] Write failing test (FAIL first): implicit via test_BC_2_11_016_e_query_038_did_you_mean —
  the test expects E-QUERY-038 JSON; the variant must compile to run the test
- [ ] Add to `crates/prism-core/src/error.rs`:

  **Step 1 — new `pub struct ColumnNotFoundDetails`** (add before the `TableNotAvailableDetails`
  struct, matching its position pattern in error.rs):
  ```rust
  /// Details for E-QUERY-038: column not found in the queried table for this client.
  ///
  /// Boxed inside `PrismError::ColumnNotFound(Box<ColumnNotFoundDetails>)` to keep
  /// `PrismError` within the `clippy::result_large_err` 128-byte threshold.
  /// 3× `String` + `Vec<String>` + `Option<String>` inline would equal or exceed
  /// the 128-byte limit — boxing reduces the variant to 8 bytes (pointer width).
  /// Follows `TableNotAvailableDetails` precedent from S-3.13 LOW-1.
  #[derive(Debug, Clone, PartialEq, Eq)]
  #[non_exhaustive]
  pub struct ColumnNotFoundDetails {
      pub column: String,
      pub table: String,
      pub client_id: String,
      pub available_columns: Vec<String>,
      pub did_you_mean: Option<String>,
  }
  ```
  Implement `Display` for `ColumnNotFoundDetails` delegating:
  `"E-QUERY-038: column '{}' not found in table '{}' for client '{}'"`
  Implement `ColumnNotFoundDetails::new(column, table, client_id, available_columns, did_you_mean)`
  constructor.

  **Step 2 — enum variant** (boxed, NOT inline fields):
  ```rust
  /// E-QUERY-038: Column not found in the queried table for this client.
  ///
  /// The inner fields are boxed (`Box<ColumnNotFoundDetails>`) to keep `PrismError`
  /// within the `clippy::result_large_err` threshold.
  ///
  /// Construct via `PrismError::ColumnNotFound(Box::new(ColumnNotFoundDetails::new(...)))`.
  /// Match via `PrismError::ColumnNotFound(ref d)` or `PrismError::ColumnNotFound(..)`.
  #[error("{0}")]
  ColumnNotFound(Box<ColumnNotFoundDetails>),
  ```

  REASON: `Vec<String>` + `Option<String>` + 3× `String` inline equals or exceeds the
  `result_large_err` 128-byte threshold (total ~128 bytes plus enum tag); boxing following
  `TableNotAvailableDetails` precedent (S-3.13 LOW-1) reduces the variant to 8 bytes.
  `ColumnNotFoundDetails` is a pub prism-core type and requires `#[non_exhaustive]` per
  CLAUDE.md conventions (adds 1 to EXPECTED — see Phase 5 and Phase 6 below).

  NOTE: `PrismError` enum is already `#[non_exhaustive]` — do NOT add the annotation again
  at enum level.
- [ ] Verify crate compiles; run `just iter prism-core` and confirm zero `result_large_err` lint warnings

### Phase 2 — E-QUERY-038 MCP error mapping

- [ ] In `crates/prism-mcp/src/error_mapping.rs`, add explicit arm BEFORE the `#[non_exhaustive]`
  catch-all:
  - `PrismError::ColumnNotFound(ref d)` → MCP error code `-32602` (INVALID_PARAMS); access fields via `d.column`, `d.table`, `d.client_id`, `d.available_columns`, `d.did_you_mean`
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
  - Column availability checked via `resolved_spec_map` for `(table, OrgId)` pair.
    Specifically: filter `resolved_spec_map` to entries where `org_slug` is in `org_scope`
    (same org-scope rules as E-QUERY-037); among matching entries, find the `ResolvedSensorSpec`
    whose `spec.sensor_id + spec.tables[i].table_name` equals the requested table; read
    `spec.tables[i].columns.iter().map(|c| c.name.clone())` as `available_columns`. When
    `resolved_spec_map` is `None` (single-tenant / test mode), read from
    `ConfigSnapshot.sensor_specs.get(sensor_id)?.tables` via a helper that calls
    `config_manager` or `table_registry`'s registered name set as a fallback.
    `TableRegistry` itself does not hold column schema.
  - `available_columns` is ALWAYS present (empty `[]` if table has zero columns or if
    `resolved_spec_map` is None and ConfigSnapshot cannot be reached); org-scoped per DI-008
    using the same org-scope pattern as `filter_to_org_visible_sensors()` and
    `filter_to_org_visible_tables()` already in `table_registry.rs`.
  - `did_you_mean`: `strsim::levenshtein` against all available columns; present when distance ≤ 3
    (same crate as E-QUERY-037's did_you_mean per D-1163); absent (field omitted) when no match ≤ 3
  - Injection-safety: `available_columns` from `TableRegistry` only; MUST NOT contain credential
    values, API key substrings, or URL strings
  - Audit: rejection included in `AuditEntry` for the `query` tool call (`outcome: "rejected"`,
    `reason: "column_not_found"`)
- [ ] Verify tests 1, 2 pass

### Phase 4 — E-QUERY pedagogical enrichments (additive fields)

- [ ] Write failing tests 3a–3f (FAIL first): `test_BC_2_11_017_enrichment_helpers_valid_operators_for_type`, `test_BC_2_11_017_enrichment_helper_extract_near_text`, `test_BC_2_11_017_enrichment_helper_how_to_fix_for_security_limit` (prism-query); `test_BC_2_11_017_ac003_parse_error_response_carries_near_text`, `test_BC_2_11_017_ac003_type_error_response_carries_valid_operators`, `test_BC_2_11_017_ac003_security_limit_error_carries_how_to_fix` (prism-mcp)
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
  - VERIFIED 2026-06-20: ast.rs has NO existing Display/to_pql/normalize impls (zero grep matches);
    the prior "lines ~681/~1099 display affordances" note was retracted as factually incorrect
  - Build a full canonicalizing `PqlNormalizer` (visitor) or `impl std::fmt::Display` on the top-level
    AST node that walks the AST producing whitespace-normalized, uppercase-keyword form — net-new,
    no leverage points exist
- [ ] Implement `normalized_pql` string production in `prism-query`:
  - Source: validated + canonicalized PQL string from Chumsky parse output
  - Keyword casing: SELECT, FROM, WHERE, GROUP BY, ORDER BY, LIMIT, AND, OR, IN, etc. — uppercase
  - Whitespace normalization: single space between tokens; no trailing whitespace
  - The normalized string MUST round-trip through Chumsky (parse to same AST as original)
  - EXCLUDED: DataFusion plan node strings (HashJoin, TableScan, SortExec, Aggregate, etc.)
    cost estimates, join-order, partition/pushdown details
  - If normalization produces empty string (should not happen for valid parse): return `None`
    (field omitted per BC-2.11.018 Error Cases)
- [ ] Insert `normalized_pql` into the existing inline `json!` payload in
  `PrismServer::query` (`crates/prism-mcp/src/server.rs`), after the rows
  serialization block and before `SafetyEnvelopeBuilder::wrap` is called:
  - Pattern: construct `payload` as `serde_json::json!({...existing keys...})`
    then conditionally insert: `if let Some(ref s) = normalized_pql_str {`
    `payload["normalized_pql"] = serde_json::Value::String(s.clone()); }`
  - NO typed query-response struct is needed or permitted — the query handler
    uses an inline `serde_json::Value` payload per the BC-2.10.007 SafetyEnvelope
    pattern (shared by all tool handlers; ADR-022 wiring-not-redesign).
  - `#[serde(skip_serializing_if)]` is a struct attribute and does NOT apply to
    `serde_json::Value`. The correct equivalent is conditional key insertion (above)
    — when the key is absent from the `Value`, it is absent from the JSON output.
  - Field presence rules (UNCHANGED from BC-2.11.018): PRESENT (non-empty) on every
    successful execution (incl. zero-row, partial sensor failure with query-level
    success); ABSENT on ALL error responses. The absent-on-error guarantee is
    enforced structurally — on the error path, `PrismServer::query` returns early
    via `prism_error_to_structured_call_result` before `payload` is constructed.
  - `ColumnNotFoundDetails` (added in Phase 1) is a new `#[non_exhaustive]` pub struct —
    ci.yml EXPECTED must be bumped from **82 to 83** by the implementer during TDD green.
    Update the ci.yml type-list comment to include `ColumnNotFoundDetails` (adjacent to
    `TableNotAvailableDetails` in the `prism_core:` group — mirror the S-3.13 LOW-1 entry).
    CLAUDE.md non-exhaustive count must be updated from 82 to 83 at merge time (merge-time
    obligation). NOTE: the `normalized_pql` wire itself adds NO new pub struct — only
    Phase 1 (`ColumnNotFoundDetails`) drives the gate bump. Net story gate impact = +1.
- [ ] Verify test 5 passes

### Phase 6 — final gates

- [ ] SAP-1 probe: `rg 'event_type\s*=' crates/ --type rust` — verify new emissions have
  BC-2.16.002 catalog rows. Expected new event_type:
  `"column_not_found.rejected"` (E-QUERY-038 gate fired)
- [ ] Verify normalized_pql value does NOT contain DataFusion plan node substrings
  (`HashJoin`, `TableScan`, `SortExec`, `Aggregate`): add assertion in test 5
- [ ] Run `just check` — all 6 Red Gate tests pass; zero clippy warnings; fmt clean
- [ ] Confirm ci.yml EXPECTED is **83** (bumped from 82 during TDD green — `ColumnNotFoundDetails`
  from Phase 1 is one new `#[non_exhaustive]` pub struct). The non-exhaustive-violation crate's
  `tests/external/non-exhaustive-violation/src/struct_violations.rs` must include a
  `ColumnNotFoundDetails` violation function (struct-literal construction attempt, mirroring
  the `TableNotAvailableDetails` violation added by S-3.13 LOW-1) so the gate continues to
  compile-fail at the correct count. The `normalized_pql` wire adds NO new pub struct — only
  `ColumnNotFoundDetails` drives the bump (+1 total; EXPECTED 82→83).

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

Red Gate: `test_BC_2_11_017_enrichment_helpers_valid_operators_for_type`, `test_BC_2_11_017_enrichment_helper_extract_near_text`, `test_BC_2_11_017_enrichment_helper_how_to_fix_for_security_limit` (prism-query); `test_BC_2_11_017_ac003_parse_error_response_carries_near_text`, `test_BC_2_11_017_ac003_type_error_response_carries_valid_operators`, `test_BC_2_11_017_ac003_security_limit_error_carries_how_to_fix` (prism-mcp)

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
| 3 | `test_BC_2_11_017_enrichment_helpers_valid_operators_for_type` (prism-query), `test_BC_2_11_017_enrichment_helper_extract_near_text` (prism-query), `test_BC_2_11_017_enrichment_helper_how_to_fix_for_security_limit` (prism-query), `test_BC_2_11_017_ac003_parse_error_response_carries_near_text` (prism-mcp), `test_BC_2_11_017_ac003_type_error_response_carries_valid_operators` (prism-mcp), `test_BC_2_11_017_ac003_security_limit_error_carries_how_to_fix` (prism-mcp) | AC-003 | prism-query + prism-mcp | E-QUERY-001 near_text + reference_pointer; E-QUERY-002 valid_operators_for_type; E-QUERY-003 how_to_fix |
| 4 | `test_BC_2_11_017_e_query_037_suggestion_prism_describe` | AC-004 | prism-query | E-QUERY-037 suggestion always contains "prism_describe"; with did_you_mean → retry hint included |
| 5 | `test_BC_2_11_018_normalized_pql_present_on_success_absent_on_error` | AC-005 + AC-006 | prism-mcp | normalized_pql present on success (incl. zero-row, partial failure); absent (not null, not present) on all error types; no DataFusion plan node strings in value |
| 6 | (implicit) | AC-001 | prism-mcp | E-QUERY-038 mapped to -32602 INVALID_PARAMS in error_mapping.rs; does not fall through to -32000 catch-all |

---

## Architecture Mapping

| Component | Module | Crate | Pure/Effectful |
|-----------|--------|-------|----------------|
| `PrismError::ColumnNotFound` variant | SS-11 | prism-core (`error.rs`) | Pure (type definition) |
| E-QUERY-038 plan-time column gate | SS-11 | prism-query (`engine.rs`) | Pure (plan-time validation against `resolved_spec_map → TableSpec.columns`) |
| E-QUERY-038 MCP error mapping | SS-10 | prism-mcp (`error_mapping.rs`) | Pure (error translation) |
| E-QUERY-001/002/003/037 pedagogical enrichments | SS-11 | prism-query (error builder / error-map time) | Pure (additive field computation) |
| Chumsky AST re-serializer (`normalized_pql` source) | SS-11 | prism-query (AST Display or PqlNormalizer) | Pure (string production from parsed AST) |
| `normalized_pql` response envelope field | SS-10 | prism-mcp (`server.rs` `PrismServer::query` inline payload — conditional `Value` key insertion before `SafetyEnvelopeBuilder::wrap`; no typed response struct) | Effectful (MCP response construction) |

Subsystem anchor justification: SS-11 (Query Execution Engine) owns the query-engine L4 work
(E-QUERY-038 gate, error enrichments, Chumsky normalizer) per ARCH-INDEX Subsystem Registry.
The prism-mcp touches in this sub-story (`error_mapping.rs` arm + `normalized_pql` field) are
wiring concerns; they belong to SS-10 but are included here because the prism-query work is the
primary scope and the prism-mcp changes are thin wires driven by the query-engine changes.

---

## Previous Story Intelligence

**S-3.13 (MERGED — Dynamic Table Availability / TableRegistry):** VERIFIED 2026-06-20
(remove-uncertainty pass): `TableRegistry` is a **concrete struct** (`prism_query::table_registry::
TableRegistry`, `#[non_exhaustive]`), NOT a `dyn` trait — the prior "`Arc<dyn TableRegistry>` is
wired into PrismServer" phrasing is corrected: the engine receives `Option<&TableRegistry>` and the
plan-time gate `check_table_availability` (engine.rs) delegates to `TableRegistry::
check_availability_gate(query_str, org_scope, resolved_spec_map)`. `TableRegistry::registered_tables()`
returns `Vec<String>` (confirmed, table_registry.rs). Org-scoping uses the crate-private helpers
`filter_to_org_visible_sensors()` / `filter_to_org_visible_tables()` (table_registry.rs ~lines 565,
609) — the prior single-method name "`filter_to_org_visible()`" does not exist; E-QUERY-038's column
gate should follow the same org-scope pattern via `check_availability_gate`'s `resolved_spec_map`
parameter. `strsim = "0.11"` is a **direct** dependency of `crates/prism-query/Cargo.toml` (D-1163;
resolves to **0.11.1** per Cargo.lock; `strsim::levenshtein(a, b) -> usize` confirmed via docs.rs
2026-06-20) — NO new dependency needed for E-QUERY-038's `did_you_mean`.

**ARCHITECTURE FLAG RESOLVED (architect, 2026-06-20, onboarding-001-tableregistry-datapath-correction.md):**
E-QUERY-038 `available_columns` reads from `resolved_spec_map → ResolvedSensorSpec.spec.tables →
TableSpec.columns` (NOT from `TableRegistry`). The gate implementation extends
`check_availability_gate` in `table_registry.rs` or adds a colocated helper in `engine.rs`: after
table presence is confirmed by E-QUERY-037, look up the matching `TableSpec` in `resolved_spec_map`
using the validated `(org_slug, sensor_id)` key, then extract column names. When `resolved_spec_map`
is `None`, return `available_columns: []` (fail-open for single-tenant mode — the gate fires only
when resolved_spec_map is wired). AC-001 and AC-002 are satisfied entirely by this
`resolved_spec_map` read path.

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

**ast.rs Chumsky-normalized PQL (critical pre-flight):** CONFIRMED 2026-06-20 (remove-uncertainty
pass, develop@f6739764): `prism-query/src/ast.rs` has NO existing `Display`/`to_pql`/`normalize`/
`to_canonical` impl on ANY AST node type — a workspace grep returns ZERO matches. **RETRACTION:** the
prior text "Lines 681 and 1099 carry raw display strings for some sub-nodes" is FACTUALLY INCORRECT
and is removed per TD-VSDD-091 (anti-volatile-pin) — those line numbers point to doc-comments on the
`SourceRef` struct (line ~681) and `TimestampLiteral` struct (line ~1099), not Display impls or raw
display strings. There are no display affordances to leverage. The implementer MUST build the
canonicalizing re-serializer entirely net-new (likely a new `impl std::fmt::Display` on the top-level
AST node or a `PqlNormalizer` visitor pattern). chumsky 0.12.0 supplies no re-serialization helper
(confirmed via docs.rs + Perplexity 2026-06-20). This is the largest single task in this sub-story —
budget appropriately.

---

## Architecture Compliance Rules

| Rule | Source | Enforcement |
|------|--------|-------------|
| E-QUERY-038 gate fires AFTER E-QUERY-037 (table must exist before checking columns) | BC-2.11.016 precondition | AC-001 "table-not-found → E-QUERY-037 not E-QUERY-038" test |
| Gate ordering explicit: E-QUERY-001 → E-QUERY-037 → E-QUERY-038 | ADR-041 §L4 + BC-2.11.016 | Adversary: read engine.rs validation step ordering |
| `PrismError::ColumnNotFound` arm MUST be explicit -32602 (not -32000 catch-all fallthrough) | BC-2.11.016 + BC-2.10.007 | Red Gate test 6 (MCP error code assertion) |
| `available_columns` sourced from `resolved_spec_map → TableSpec.columns → ColumnSpec.name`; MUST NOT contain credentials (operator TOML column names are safe schema identifiers) | BC-2.11.016 invariant + DI-008 | AC-002 multi-tenant test |
| `normalized_pql` MUST NOT contain DataFusion plan node type strings | BC-2.11.018 postcondition | AC-005 assertion in test 5 |
| `normalized_pql` MUST be absent (not null) on error responses | BC-2.11.018 invariant | AC-006: use `value.get("normalized_pql").is_none()` — not null check |
| `near_text` truncated to ≤50 chars (DI-006: prevents raw PQL relay as error context) | BC-2.11.017 postcondition | AC-003 near_text length assertion |
| `normalized_pql` round-trips through Chumsky (same AST) | BC-2.11.018 postcondition | Implementer verifies; proptest VP candidate |
| All new `event_type=` tracing emissions require BC-2.16.002 catalog rows (SAP-1) | CLAUDE.md §SAP-1 | Adversary SAP-1 probe |
| `PrismError` enum is already `#[non_exhaustive]` — do NOT add it again at enum level | CLAUDE.md §Conventions | Adversary: confirm `#[non_exhaustive]` at enum (not at variant) |

**Forbidden patterns:**
- `normalized_pql: null` in JSON (must be absent, not null — use conditional key insertion into `serde_json::Value`; do NOT introduce a typed struct with `skip_serializing_if`, which is a struct attribute inapplicable to `serde_json::Value` construction)
- A new typed query-response struct to hold the `normalized_pql` field (ADR-022 wiring-not-redesign; the inline `json!` payload is the established BC-2.10.007 pattern)
- E-QUERY-038 returning before E-QUERY-037 check (wrong gate ordering)
- `available_columns` containing API key, bearer token, or URL patterns
- `near_text` > 50 characters
- DataFusion plan node strings (`HashJoin`, `TableScan`, `SortExec`, `Aggregate`) in `normalized_pql`

---

## Library & Framework Requirements

| Library | Version | Usage |
|---------|---------|-------|
| prism-core | workspace | PrismError (new ColumnNotFound variant), ColumnType (valid_operators_for_type helper) |
| strsim | `0.11` (direct dep in prism-query/Cargo.toml, D-1163; resolves **0.11.1** per Cargo.lock) | `strsim::levenshtein(a, b) -> usize` for did_you_mean in E-QUERY-038 — NO new dep needed (signature confirmed docs.rs 2026-06-20) |
| chumsky | `0.12` caret in Cargo.toml; resolves **0.12.0** per Cargo.lock (the only published 0.12.x release) | Parser-combinator only — NO built-in AST pretty-printer/re-serializer (confirmed docs.rs + Perplexity 2026-06-20). normalized_pql re-serializer is net-new. |
| ariadne | `0.4` caret; resolves **0.4.1** per Cargo.lock | Human-readable parse-error formatting (already a prism-query dep); span source for E-QUERY-001 near_text |
| datafusion | `53.1` caret; resolves **53.1.0** per Cargo.lock | Backing execution engine — do NOT import DataFusion internals for normalized_pql |
| serde / serde_json | `1` (workspace) | `normalized_pql` conditional `Value` key insertion into existing inline `serde_json::json!` payload in `PrismServer::query`; `serde_json::Value::String(s.clone())` insertion pattern |
| tracing | workspace | Structured event emission; `column_not_found.rejected` event_type must be in BC-2.16.002 |

**Version pinning note (verified 2026-06-20, remove-uncertainty pass, Cargo.lock @ develop@f6739764):**
`strsim` resolves to **0.11.1**, `chumsky` to **0.12.0** (only published 0.12.x), `datafusion` to
**53.1.0**, `ariadne` to **0.4.1**. All are existing prism-query deps — NO new dependency is added by
this story. `chumsky 0.12.0` is purely a parser-combinator library with NO AST pretty-printing or
re-serialization facility (confirmed via docs.rs/crates.io + Perplexity research 2026-06-20), so the
normalized_pql re-serializer is genuinely net-new. Do NOT add a DataFusion internal import to obtain
normalized query strings.

---

## File Structure Requirements

| File | Action | Purpose |
|------|--------|---------|
| `crates/prism-core/src/error.rs` | Modify | Add `pub struct ColumnNotFoundDetails` (`#[non_exhaustive]`) with fields column/table/client_id/available_columns/did_you_mean + Display + `::new()` constructor; add `PrismError::ColumnNotFound(Box<ColumnNotFoundDetails>)` enum variant with `#[error("{0}")]` (boxed to satisfy `result_large_err`; follows `TableNotAvailableDetails` precedent from S-3.13 LOW-1). |
| `crates/prism-query/src/engine.rs` | Modify | (1) E-QUERY-038 column-not-found plan-time gate (colocated with E-QUERY-037 gate); (2) E-QUERY-001 near_text + reference_pointer enrichment; (3) E-QUERY-002 valid_operators_for_type enrichment; (4) E-QUERY-003 how_to_fix enrichment; (5) E-QUERY-037 suggestion text update; (6) normalized_pql string production via Chumsky normalizer |
| `crates/prism-query/src/ast.rs` | Modify | Add `Display` impl or `PqlNormalizer` visitor on top-level AST node for canonicalized PQL re-serialization (fully net-new; VERIFIED 2026-06-20 no existing Display/to_pql/normalize impls — no leverage points) |
| `crates/prism-mcp/src/error_mapping.rs` | Modify | Add explicit `-32602 INVALID_PARAMS` arm for `PrismError::ColumnNotFound` before the `#[non_exhaustive]` catch-all |
| `crates/prism-mcp/src/server.rs` | Modify | In `PrismServer::query`, insert `normalized_pql` conditionally into the existing inline `serde_json::json!` payload after rows serialization; conditional key insertion (not a struct field — no typed response struct exists; ADR-022 wiring-not-redesign) |
| `ci.yml` | Modify | Bump EXPECTED from 82 to 83 — `ColumnNotFoundDetails` (from Phase 1) is one new `#[non_exhaustive]` pub struct added by this story. Update the type-list comment to include `ColumnNotFoundDetails` adjacent to `TableNotAvailableDetails` in the `prism_core:` group (mirror the S-3.13 LOW-1 row entry). Also update the error text listing at line ~679 area (see S-3.13 LOW-1 row for `TableNotAvailableDetails` as the model). The `normalized_pql` wire adds +0 to EXPECTED (no new pub struct). |
| `tests/external/non-exhaustive-violation/src/struct_violations.rs` | Modify | Add a violation function for `ColumnNotFoundDetails` (struct-literal construction attempt), mirroring the `TableNotAvailableDetails` violation function added by S-3.13 LOW-1. This brings the E0639 compile-fail count to 83 (matching the bumped ci.yml EXPECTED). |
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
| 2.2 | BC-2.11.016-v1.7-pin-propagation-2026-07-08 | 2026-07-08 | story-writer | **BC-2.11.016 v1.6→v1.7 version-pin propagation (POL-29/POL-23).** Product-owner keyword sweep bumped BC-2.11.016 v1.6→v1.7 (`\| project`→`\| fields` keyword). Two live version-pin cites updated: (1) §Behavioral Contracts table BC-2.11.016 row version cell v1.6→v1.7; (2) §Token Budget table BC-2.11.016 row v1.6→v1.7. Historical changelog row (2.1 v1.5→v1.6 propagation) left unchanged per POL-29. AC semantics UNCHANGED — AC-001 and AC-002 assert SELECT-position typo and org-scoped available_columns; neither asserts a specific clause-position count. Frontmatter version 2.1→2.2; updated 2026-07-08 (POL-23). |
| 2.1 | ADV-FIX-P1-HIGH-001-BC-2.11.016-v1.6-pin-propagation-2026-07-08 | 2026-07-08 | story-writer | **BC-2.11.016 v1.5→v1.6 version-pin propagation (ADV-FIX-P1-HIGH-001, POL-29/POL-23).** Product-owner amended BC-2.11.016 v1.5→v1.6 expanding E-QUERY-038 gate from six SQL positions to twelve positions (Filter/Pipe/SqlPipe predicate + sort/stats/project positions). Two live version-pin cites updated: (1) §Behavioral Contracts table BC-2.11.016 row version cell v1.5→v1.6; Key Clauses updated to note 12-position expansion. (2) §Token Budget table BC-2.11.016 row v1.5→v1.6. AC semantics UNCHANGED — AC-001 and AC-002 assert SELECT-position typo and org-scoped available_columns; neither asserts a specific clause-position count. Frontmatter version 2.0→2.1; updated 2026-07-08 (POL-23). |
| 2.0 | POL29-BC-2.11.016-V1.5-PROPAGATION-2026-06-28 | 2026-06-28 | story-writer | BC-2.11.016 v1.4→v1.5 POL-29 propagation (HAVING column-gate position added; F-PWL1-LOW-001). PO bumped BC-2.11.016 v1.4→v1.5 adding HAVING as the 6th column-gate position (same `Option<Predicate>` extraction path as WHERE). Two live cites updated: (1) §Behavioral Contracts body table version cell v1.4→v1.5 (Key Clauses extended to note HAVING 6th position); (2) §Token Budget table BC-2.11.016 version cell v1.4→v1.5. POL-7 title cell verified verbatim: `E-QUERY-038 Column-Not-Found Plan-Time Gate (L4)` — no change needed (prefix-stripped per prism convention). AC impact assessment: AC-001 and AC-002 assert SELECT-position typo and org-scoped available_columns respectively; neither asserts a specific clause-position count or set. The v1.5 HAVING addition does NOT materially affect AC assertions — version-pin bump only. No AC/BC-array/scope changes. |
| 1.9 | POL23-BC-VERSION-PROPAGATION-2026-06-22 | 2026-06-22 | story-writer | POL-4 Story-Anchor BC-version propagation (POL-23). PO bumped anchor BCs after Story-Anchor fix + exhaustive BC audit: BC-2.11.016 v1.3→v1.4, BC-2.11.017 v1.2→v1.3, BC-2.11.018 v1.1→v1.2. §Behavioral Contracts body table pins and §Token Budget table pins updated to match. Title cells verified prefix-stripped per POL-7 (match BC-INDEX canonical form — no change needed). No AC/BC-array/scope change. |
| 1.8 | POL20-BC-VERSION-PROPAGATION-2026-06-22 | 2026-06-22 | story-writer | POL-20 BC version-pin propagation (bc_array_changes_propagate_to_body_and_acs). §Behavioral Contracts table + §Token Budget table: BC-2.11.016 pin updated v1.2→v1.3 (PO introduced-field normalization burst); BC-2.11.017 pin updated v1.1→v1.2 (§E-QUERY-002 QueryTypeMismatch variant + Display correction); BC-2.11.018 pin updated v1.0→v1.1 (PO POL-20 normalization). Pins now match canonical BC-INDEX and on-disk BC files. No AC/scope/BC-array changes. |
| 1.7 | POL32-CHANGELOG-REORDER-2026-06-22 | 2026-06-22 | story-writer | POL-32 changelog ordering correction (F-001B-SC-FRESH-MED-001). Reordered §Changelog table to monotonic descending (v1.6 → … → v1.0) per POL-32 (changelog_monotonic_descending). Prior table was ascending with v1.6/v1.5 locally inverted at end. Frontmatter `version` bumped 1.6→1.7; `updated: 2026-06-22` added (POL-23 story-version-bump requirement). No AC/BC/scope changes. |
| 1.6 | LOCAL-ADVERSARY-FINDINGS-CLOSURE-2-2026-06-22 | 2026-06-22 | story-writer | LOCAL adversary findings closure (F-001B-P1-MED-001 + OBS-001B-P1-001). F-001B-P1-MED-001 (MED, POL-7 title-cell over-correction revert): BC-2.11.017 body-table title cell `"BC-2.11.017: E-QUERY Pedagogical Enrichments (L4 — Codes 001, 002, 003, 037)"` reverted to prefix-STRIPPED form `"E-QUERY Pedagogical Enrichments (L4 — Codes 001, 002, 003, 037)"` per BC-INDEX.md title column (v1.5 had incorrectly added the `BC-2.11.017:` prefix per a misapplication of POL-7 — the prism project norm strips the prefix; BC-2.11.016 and BC-2.11.018 sibling rows in same table are already prefix-stripped). OBS-001B-P1-001 (Red Gate test-name reconciliation, AC-003): row 3 of Red Gate table replaced phantom `test_BC_2_11_017_pedagogical_enrichments` (never existed in codebase) with the 6 real load-bearing tests confirmed present in worktree: `test_BC_2_11_017_enrichment_helpers_valid_operators_for_type`, `test_BC_2_11_017_enrichment_helper_extract_near_text`, `test_BC_2_11_017_enrichment_helper_how_to_fix_for_security_limit` (prism-query/tests/e_query_pedagogical.rs) + `test_BC_2_11_017_ac003_parse_error_response_carries_near_text`, `test_BC_2_11_017_ac003_type_error_response_carries_valid_operators`, `test_BC_2_11_017_ac003_security_limit_error_carries_how_to_fix` (prism-mcp/tests/normalized_pql.rs). AC-003 body Red Gate line and Phase 4 task line updated to match. BC array UNCHANGED (BC-2.11.016, BC-2.11.017, BC-2.11.018). No AC/BC/scope changes. |
| 1.5 | LOCAL-ADVERSARY-FINDINGS-CLOSURE-2026-06-22 | 2026-06-22 | story-writer | LOCAL adversary findings closure (F-001B-FRESH-MED-002 + OBS-001B-FRESH-001). F-001B-FRESH-MED-002 (MED, POL-23 BC version-pin propagation): BC-2.11.016 body table + Token Budget pins updated v1.0→v1.2 (canonical in BC-INDEX; v1.1 OBS-2 deferral prose + v1.2 OBS-001-B boxed-form prose; story already ships the boxed ColumnNotFound form — purely descriptive bump). BC-2.11.017 body table + Token Budget pins updated v1.0→v1.1 (PO amendment: QueryTypeMismatch variant + new Display propagated into BC body). BC-2.11.017 body table title cell updated to match BC H1 verbatim per POL-7: "BC-2.11.017: E-QUERY Pedagogical Enrichments (L4 — Codes 001, 002, 003, 037)". OBS-001B-FRESH-001 (cosmetic): File Structure row + Phase 6 gate step for ColumnNotFoundDetails violation corrected from `enum_violations.rs` → `struct_violations.rs` (ColumnNotFoundDetails is a struct → E0639; mirroring TableNotAvailableDetails which is also in struct_violations.rs per S-3.13 LOW-1). BC array unchanged (BC-2.11.016, BC-2.11.017, BC-2.11.018). No AC/scope changes. |
| 1.4 | CORRECTION-2-COLUMNNOTFOUND-BOXED-2026-06-21 | 2026-06-21 | story-writer | CORRECTION-2 adjudication applied (onboarding-001-B-columnnotfound-variant-shape-correction.md). `PrismError::ColumnNotFound` corrected from inline-field variant to `ColumnNotFound(Box<ColumnNotFoundDetails>)` — inline 3×String + Vec<String> + Option<String> equals or exceeds the `clippy::result_large_err` 128-byte threshold; boxing following `TableNotAvailableDetails` precedent (S-3.13 LOW-1) reduces variant to 8 bytes. Phase 1 task rewritten: now specifies (a) new `pub struct ColumnNotFoundDetails` with `#[non_exhaustive]` + `#[derive(Debug, Clone, PartialEq, Eq)]` + Display + `::new()` constructor, and (b) `ColumnNotFound(Box<ColumnNotFoundDetails>)` boxed variant with `#[error("{0}")]`. Phase 2 error_mapping.rs arm corrected from `ColumnNotFound { ... }` to `ColumnNotFound(ref d)` with field access via `d.*`. Phase 5 normalized_pql ci.yml note corrected: EXPECTED must be bumped 82→83 (ColumnNotFoundDetails from Phase 1); normalized_pql wire itself adds +0. Phase 6 final gates ci.yml step corrected: EXPECTED is 83 (not 82); enum_violations.rs must include ColumnNotFoundDetails violation function. File Structure ci.yml row corrected: "No change / EXPECTED remains 82" → "Modify / bump EXPECTED to 83". File Structure: new row added for `tests/external/non-exhaustive-violation/src/enum_violations.rs` (add ColumnNotFoundDetails violation function, mirroring TableNotAvailableDetails row from S-3.13 LOW-1). Net non-exhaustive gate impact = +1 (ColumnNotFoundDetails only; EXPECTED 82→83). CLAUDE.md count 82→83 is a merge-time obligation. AC behavioral semantics UNCHANGED. BCs UNCHANGED: BC-2.11.016, BC-2.11.017, BC-2.11.018. |
| 1.3 | NORMALIZED-PQL-ENVELOPE-CORRECTION-2026-06-21 | 2026-06-21 | story-writer | CORRECTION-1 adjudication applied (onboarding-001-B-normalized-pql-envelope-correction.md). Phase 5 task rewritten: "Add normalized_pql field to query response type" replaced with conditional `serde_json::Value` key insertion into the existing inline `json!` payload in `PrismServer::query` before `SafetyEnvelopeBuilder::wrap` — no typed query-response struct exists or is permitted (BC-2.10.007 SafetyEnvelope pattern; ADR-022 wiring-not-redesign). File Structure `server.rs` row updated to describe conditional key insertion. File Structure `ci.yml` row changed from "Modify / Increment EXPECTED" to "No change / EXPECTED remains 82" (001-A merged, +0 from 001-B). `risk_mitigations` skip_serializing_if bullet corrected: `#[serde(skip_serializing_if)]` is a struct attribute inapplicable to `serde_json::Value`; conditional key insertion is the correct equivalent. Architecture Mapping `normalized_pql` row updated to name `PrismServer::query` inline payload and omit "no typed response struct". Forbidden patterns block extended with typed-struct anti-pattern. Library & Framework serde row updated. Frontmatter `crates_touched` and points-justification comments corrected. AC-005/AC-006 behavioral semantics UNCHANGED. BCs UNCHANGED: BC-2.11.016, BC-2.11.017, BC-2.11.018. |
| 1.2 | TABLEREGISTRY-DATAPATH-CORRECTION-2026-06-20 | 2026-06-20 | story-writer | Architect adjudication applied (onboarding-001-tableregistry-datapath-correction.md, D-1259 FLAG-001). Wiring-not-redesign corrections for FLAG-001 from remove-uncertainty pass. Edits: (1) `depends_on` S-5.03 comment — corrected from fictional "TableRegistry injection into PrismServer" to actual ServerHandler override patterns; (2) `depends_on` S-3.13 comment — replaced "reads registry's per-org column schema" with `resolved_spec_map` parameter flow description; (3) dependency anchor comments corrected; (4) Tasks Phase 3 column gate — replaced "Column availability checked against TableRegistry" with `resolved_spec_map → ResolvedSensorSpec.spec.tables → TableSpec.columns` lookup with None fallback; updated `available_columns` presence note to reference `filter_to_org_visible_sensors/_tables` pattern; (5) `risk_mitigations` — replaced "sourced ENTIRELY from TableRegistry" with `resolved_spec_map → ColumnSpec.name` data path with TOML-source safety rationale; (6) Previous Story Intelligence CRITICAL FLAG paragraph — replaced with "ARCHITECTURE FLAG RESOLVED" citing architect adjudication doc, canonical `resolved_spec_map` read path, and AC-001/AC-002 satisfaction statement; (7) Architecture Compliance Rules `available_columns` sourcing rule corrected; (8) Architecture Mapping E-QUERY-038 row corrected. No AC-semantic changes. No BC array changes. BCs remain: BC-2.11.016, BC-2.11.017, BC-2.11.018. |
| 1.1 | remove-uncertainty-2026-06-20 | 2026-06-20 | research-agent | REMOVE-UNCERTAINTY pass (D-1110) pre-TDD. Verified all tech assertions against develop@f6739764 codebase + Cargo.lock + error-taxonomy.md v1.91 + docs.rs/Perplexity. LOW-RISK corrections applied: (1) confirmed version pins strsim 0.11.1 / chumsky 0.12.0 / datafusion 53.1.0 / ariadne 0.4.1 with citations; (2) RETRACTED false+volatile "ast.rs:681/1099 display affordances" citations (TD-VSDD-091 — those are doc-comments, not Display impls; ZERO AST Display impls exist; chumsky 0.12.0 has no built-in re-serializer); (3) corrected "Arc<dyn TableRegistry>" → concrete `TableRegistry` struct passed as `Option<&TableRegistry>`; gate = `check_table_availability` → `check_availability_gate`; (4) corrected non-existent `filter_to_org_visible()` → `filter_to_org_visible_sensors/_tables`; (5) noted E-QUERY-001 variant is `QueryParseFailed { offset, detail }`. FLAGGED for architect (not edited into ACs): `TableRegistry` has NO column-level schema — E-QUERY-038 `available_columns` must come from `resolved_spec_map` (ResolvedSensorSpec→TableSpec.columns), not TableRegistry; design path affects AC-001/AC-002 satisfaction. ColumnType variants (String/Integer/Float/Boolean/Datetime/Json) confirmed match AC-003 operator table. PrismError enum `#[non_exhaustive]` and error_mapping.rs `-32602` arm precedent (TableNotAvailable) confirmed. No AC/BC/scope text changed by this pass. |
| 1.0 | D-1244-decomposition-2026-06-19 | 2026-06-19 | story-writer | Initial sub-story decomposition — split from S-DEMO-PRISMQL-ONBOARDING-001 (13 pts) per D-1244 §Parallel Execution Plan. Covers L4 query-engine surfaces (prism-core + prism-query + prism-mcp wire). 3 BCs: BC-2.11.016, BC-2.11.017, BC-2.11.018. 6 ACs + 6 Red Gate tests. 6 pts. Pipelines behind PIVOT-003 for crate-conflict avoidance. |
