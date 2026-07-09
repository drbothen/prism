---
document_type: story
story_id: S-DEMO-PRISMQL-GRAMMAR-REMEDIATION-001
title: "Demo-Readiness Grammar & MCP Remediation — T13 Capstone Demo (2026-06-24)"
wave: null
# Wave assignment: schedule immediately after S-5.04 merges to clear prism-mcp crate conflict.
# This story and S-5.04 both touch prism-mcp; S-5.04 must merge first to provide a clean
# develop base. Wave scheduler should assign to the wave immediately following S-5.04 merge.
target_module: prism-query
# Primary crate is prism-query (grammar, engine, error_recovery). prism-mcp and
# prism-security are secondary crates touched by this story.
subsystems: [SS-10, SS-11]
# Subsystem anchor justifications:
#   SS-11 (Query Execution Engine) owns the prism-query grammar work (BC-2.11.020/021/023):
#     SqlPipe AST, NOW()/INTERVAL temporal grammar, mode-bridge error diagnostics,
#     normalized_pql field on StructuredErrorFields (prism-mcp; D-1110 — no ParseErrorDetails type
#     exists), Filter mode execution tests.
#   SS-10 (MCP Interface) owns the prism-mcp work (BC-2.11.022/BC-2.10.015/016/017):
#     build_reference_content() function, list_capabilities OrgRegistry wiring,
#     MCP prompt hang fix, not-yet-available tools fast-fail re-ordering.
#   Both subsystems are touched; SS-11 is primary (larger scope); SS-10 is co-owner.
priority: P0
# P0: ALL findings targeted by this story are DEMO-BLOCKER or BLOCKER class per
# demo-pre-flight-audit-2026-06-24.md and prismql-grammar-usability-audit-2026-06-24.md.
# T13 capstone demo cannot proceed without these fixes.
depends_on:
  - S-5.04
  # S-5.04 (TDD-ready) must merge first to avoid prism-mcp crate conflict.
  # S-5.04 touches prism-mcp extensively (sensor health tool, structured error responses,
  # BC-2.08.007 wiring). This story also modifies prism-mcp (resources.rs, server.rs, tools).
  # Merging S-5.04 first provides a clean prism-mcp base on develop.
  # Dependency anchor: build-order requirement, not just conceptual relatedness.
blocks: []
estimated_days: 4
# Estimate reflects multi-area TDD work: grammar extension (2 areas), MCP reference overhaul,
# MCP tool/prompt fixes (3 areas), E-QUERY error improvements, plus 2–3 BLOCKER investigative fixes.
points: 13
# Points breakdown:
#   BC-2.11.020 SQL→Pipe Ast::SqlPipe + FORBID-BOTH E-QUERY-040: 2 pts
#   BC-2.11.021 NOW()/INTERVAL temporal grammar + plan-time injection: 2 pts
#   BC-2.11.022 build_reference_content() + CI 3-tier gate: 3 pts
#   BC-2.11.023 mode-bridge error + normalized_pql on StructuredErrorFields + D7 filter tests: 2 pts
#   BC-2.10.015 FeatureFlagEvaluator Arc<OrgRegistry> DI wiring: 1 pt
#   BC-2.10.016 BLOCKER-003 prompt hang investigation + fix: 1.5 pts
#   BC-2.10.017 BLOCKER-004 fast-fail guard reorder (before emit_tool_audit await; D-1110 — not try_send): 0.5 pts
#   Implementer-AC (non-BC) items (BLOCKER-001, BLOCKER-002, polish): 1 pt
#   Total: 13 pts
level: "L4"
status: merged
# BC status: all 8 BCs promoted draft→active per POL-14 at D-1367 (PR #203 squash-merged develop@7e60df03 2026-06-26).
version: "1.16"
updated: "2026-07-08"
# v1.13: BC-2.11.020 v1.1→v1.2 version-pin propagation (ADV-FIX-P1-HIGH-001, POL-29/POL-23).
# v1.12: POL-14 post-merge promotion (D-1367 2026-06-26) — status draft→merged; all 8 BCs draft→active.
# PR #203 squash-merged to develop@7e60df03 (CI 43/43 green; 9-round PR-LEVEL 3-CLEAN(strict)
# cascade on frozen HEAD 356e0573; security CLEAN). No AC/scope change.
# v1.1: dclaude:remove-uncertainty pass 1 (D-1110) — 8 tech-assumption corrections applied
# (versions pinned to Cargo.lock; ParseErrorDetails→StructuredErrorFields; OrgRegistry::slug_exists;
# emit_tool_audit guard-reorder not try_send; DTU routes/oauth.rs path; DataFusion plan-time
# soundness confirmed; chrono custom interval parse; rmcp prompt-dispatch investigation reaffirmed).
# No AC count / BC trace change. See Changelog v1.1 row.
# v1.2: dclaude:remove-uncertainty pass 2 (D-1110, pre-TDD-delivery) — re-validated against the
# post-S-5.04 develop tip (903c8fcb). All 4 pass-1 code-grounded corrections RE-CONFIRMED on
# 903c8fcb (StructuredErrorFields shape, OrgRegistry::slug_exists(&OrgSlug), emit_tool_audit
# guard-reorder-before-audit-await, DTU oauth.rs/plugin internals). S-5.04 introduced NO new
# conflicts (check_sensor_health is a LIVE_TOOLS handler, not NOT_YET_AVAILABLE; emit_tool_audit
# + not-yet-available stub structure unchanged). 2 NEW corrections applied: (a) Area C location —
# the prismql://reference include_str! is PQL_REFERENCE_CONTENT in resources/schema.rs (served by
# render_pql_reference_resource), NOT in resources.rs as v1.1 stated; (b) BLOCKER-001 — no dedicated
# "force-refresh entrypoint" in the plugin; acquire_token is the unconditional fresh-acquire fn,
# get_token re-acquires only on cache-miss/stale. No AC count / BC trace change. See Changelog v1.2 row.
# v1.4: version-pin sync (MED-1 / POL-23) — BC-2.11.023 v1.1→v1.2 and BC-2.10.015 v1.0→v1.1
# in Behavioral Contracts table. POL-25 sweep: no other live-narrative pins at old versions.
# No AC/scope/code change.
# v1.3: story-writer spec-sync burst (D-1326 adjudication) — 4 corrections: (1) AC-019 re-scoped:
# BLOCKER-001 root cause is connect-timeout (not KV staleness — PluginKvStore is in-memory/fresh per
# prism start); deferred to S-RESILIENCE-FEDERATED-001; demo unblocked via runbook DTU health-check
# Fix B; BC citation corrected BC-2.06.001→BC-2.01.005. (2) OBS-3: File Structure RowLimit MCP
# mapping path corrected prism-core→prism-mcp/src/error_mapping.rs. (3) BC-2.11.023 v1.0→v1.1 in
# Behavioral Contracts table. (4) S-RESILIENCE-FEDERATED-001 stub registered as deferral anchor.
# No AC count / BC trace count change.
producer: story-writer
timestamp: "2026-06-24T00:00:00Z"
input-hash: "TBD"
inputs:
  - ".factory/specs/architecture/demo-readiness-remediation-design-2026-06-24.md"
  - ".factory/specs/behavioral-contracts/BC-2.11.020-sql-to-pipe-composition-sqlpipe-ast-and-forbid-both-dual-limit.md"
  - ".factory/specs/behavioral-contracts/BC-2.11.021-temporal-grammar-now-interval-planning-time-constant-injection.md"
  - ".factory/specs/behavioral-contracts/BC-2.11.022-auto-generated-prismql-reference-content-contract-and-ci-parity-gate.md"
  - ".factory/specs/behavioral-contracts/BC-2.11.023-three-mode-correctness-mode-bridge-error-normalized-pql-and-d7-graduation-invariant.md"
  - ".factory/specs/behavioral-contracts/BC-2.10.015-list-capabilities-consults-org-registry-for-client-registered-check.md"
  - ".factory/specs/behavioral-contracts/BC-2.10.016-mcp-prompts-fast-return-guarantee-no-hang.md"
  - ".factory/specs/behavioral-contracts/BC-2.10.017-not-yet-available-tools-fast-fail-audit-channel-non-blocking.md"
  - ".factory/specs/behavioral-contracts/BC-2.11.002-prismql-filter-mode.md"
  - ".factory/specs/architecture/decisions/ADR-043-true-sql-to-pipe-composition-select-from-t-stage-head-lowers-to-pipe-source.md"
  - ".factory/specs/architecture/decisions/ADR-044-temporal-grammar-now-and-interval-relative-duration-literals.md"
  - ".factory/specs/architecture/decisions/ADR-045-auto-generated-prismql-reference-resource-grammar-registry-parity-gate.md"
  - ".factory/specs/architecture/decisions/ADR-046-three-mode-correctness-filter-sql-pipe-mode-bridge-error-and-execution-validation.md"
  - ".factory/research/prismql-grammar-usability-audit-2026-06-24.md"
  - ".factory/research/demo-pre-flight-audit-2026-06-24.md"
traces_to: [D-1308]
cycle: "v1.0.0-greenfield"
epic_id: "E-5"
# Epic E-5 (MCP Interface / Query Engine). Remediation story targeting T13 capstone demo.
phase: 2
acceptance_criteria_count: 27
# 27 ACs: 20 BC-traced ACs + 7 implementer/doc ACs with finding IDs.
red_gate_tests: 20
# 20 Red Gate tests corresponding to the 20 BC-traced ACs.
# Implementer/doc ACs (7) are verified by integration tests, timing assertions, or doc
# inspection — not classic Red Gate unit tests.
tdd_mode: strict
behavioral_contracts:
  [BC-2.11.020, BC-2.11.021, BC-2.11.022, BC-2.11.023, BC-2.10.015, BC-2.10.016, BC-2.10.017, BC-2.11.002]
# BC array propagation (bc_array_changes_propagate_to_body_and_acs):
# BC-2.11.020 — SQL→Pipe SqlPipe AST + FORBID-BOTH E-QUERY-040 (cited in AC-001, AC-002, AC-003)
# BC-2.11.021 — NOW()/INTERVAL temporal grammar (cited in AC-004, AC-005)
# BC-2.11.022 — auto-generated prismql://reference + CI 3-tier gate (cited in AC-006, AC-007, AC-008)
# BC-2.11.023 — mode-bridge error + normalized_pql + D7 filter execution (cited in AC-009, AC-010, AC-011, AC-012, AC-027)
# BC-2.10.015 — list_capabilities OrgRegistry Arc-DI (cited in AC-013, AC-014)
# BC-2.10.016 — MCP prompts fast-return (cited in AC-015, AC-016)
# BC-2.10.017 — not-yet-available tools fast-fail (cited in AC-017, AC-018)
# BC-2.11.002 — filter mode execution tests (cited in AC-011, AC-012)
# All 8 BCs cited in at least one AC body trace.
verification_properties: [VP-021]
# VP-021 (PrismQL parser never panics on arbitrary input — fuzz) applies to grammar changes
# made by this story: Ast::SqlPipe, Expr::Now/Interval, mode-bridge error paths all add new
# parser code paths. The fuzz target covers the complete parser.
assumption_validations: []
risk_mitigations: []
crates_touched:
  - prism-query
  # Grammar: ast.rs (Ast::SqlPipe, Expr::Now/Interval/TimestampArithmetic), filter_parser.rs
  # (mode detection tristate, expression parser extension),
  # lib.rs `inject_now` + `plan_sqlpipe_query` (plan-time NOW injection, FORBID-BOTH check),
  # materialization.rs `run_materialization_pipeline` Step 1a/1b (wires both plan-time gates),
  # execute_against_session `Ast::SqlPipe` arm in materialization.rs (execution lowering),
  # error_recovery.rs (mode-bridge diagnostic),
  # error_recovery.rs (normalized_pql rewrite STRING producer); PrismError::RedundantRowLimit
  # NOTE (D-1110): the normalized_pql FIELD lives on StructuredErrorFields in prism-mcp, not on
  # a prism-query struct. There is no `ParseErrorDetails` type in the codebase.
  - prism-mcp
  # resources.rs (build_reference_content function + static constants + example array; repoint the
  # read_resource prismql://reference arm), resources/schema.rs (D-1110 pass-2: remove
  # PQL_REFERENCE_CONTENT include_str! + retire/repoint render_pql_reference_resource — the
  # include_str! lives in resources/schema.rs, NOT resources.rs),
  # error_mapping.rs (StructuredErrorFields.normalized_pql field — D-1110),
  # server.rs (NOT_YET_AVAILABLE guard reorder BEFORE emit_tool_audit await — D-1110: emit_tool_audit
  # awaits AuditWriter::write_tool_call, it is NOT an mpsc try_send path; list_capabilities handler
  # wiring also in server.rs — no tools/list_capabilities.rs exists), prompts.rs / #[prompt_handler] macro expansion investigation
  - prism-core
  # error.rs (PrismError::RedundantRowLimit variant). NOTE (D-1110): org_registry.rs needs NO new
  # method — OrgRegistry::slug_exists(&OrgSlug) already covers the BC-2.10.015 existence check.
  - prism-security
  # feature_flag.rs (FeatureFlagEvaluator: add Arc<OrgRegistry> field + change client_exists(&str)
  # to build an OrgSlug from the &str and call OrgRegistry::slug_exists per BC-2.10.015 — D-1110)
  - prism-dtu-crowdstrike
  # BLOCKER-001 DEFERRED to S-RESILIENCE-FEDERATED-001 (D-1326): no changes to this crate
  # in this story. Root cause is connect-timeout, not token cache. DTU routes/oauth.rs unchanged.
---

# S-DEMO-PRISMQL-GRAMMAR-REMEDIATION-001: Demo-Readiness Grammar & MCP Remediation

## Narrative

As a Prism developer preparing the T13 capstone demo, I want all BLOCKER and MAJOR grammar,
error-message, and MCP infrastructure findings resolved, so that the demo runs without hangs,
correct grammar examples are auto-generated in `prismql://reference`, SQL→Pipe composition
works end-to-end, temporal queries parse, and MCP tools/prompts respond within time bounds.

## Behavioral Contracts

| BC ID | Version | Title |
|-------|---------|-------|
| BC-2.11.020 | v1.5 | SQL→Pipe Composition — `SqlPipe` AST Variant and FORBID-BOTH Dual-Limit Rule |
| BC-2.11.021 | v1.0 | Temporal Grammar — `NOW()` and `INTERVAL` Planning-Time Constant Injection |
| BC-2.11.022 | v1.0 | Auto-Generated `prismql://reference` Content Contract and CI Parity Gate |
| BC-2.11.023 | v1.2 | Three-Mode Correctness — Mode-Bridge Error, `normalized_pql`, and D7 Graduation Invariant |
| BC-2.10.015 | v1.2 | `list_capabilities` Consults `OrgRegistry` for `client_registered` Check |
| BC-2.10.016 | v1.1 | MCP Prompts Fast-Return Guarantee — No Indefinite Hang |
| BC-2.10.017 | v1.1 | Not-Yet-Available Tools Fast-Fail — Audit Channel Non-Blocking |
| BC-2.11.002 | v1.4 | PrismQL Filter Mode Parsing |

---

## Acceptance Criteria

> NOTE: This story is large by explicit human directive (consolidate all T13 demo-blocking
> findings into one story). ACs are grouped into numbered areas to enable area-by-area TDD
> cascade. The implementer MUST work through areas sequentially: Area A → B → C → D → E → F.
> Each area is independently Red-Gate-able and independently mergeable only if the orchestrator
> grants scope split. Default is deliver as one PR.

---

### Area A — SQL→Pipe Composition (BC-2.11.020, ADR-043)

**AC-001** (traces to BC-2.11.020 postcondition 1): A query beginning with `SELECT` and
containing at least one unquoted `|` followed by a pipe stage keyword (e.g., `| enrich`,
`| where`, `| limit`, `| sort`, `| stats`, `| dedup`, `| fields`) parses as
`Ast::SqlPipe(SqlPipeQuery { head: SqlQuery, stages: Vec<PipeStage> })`. The `head` is the
full SQL preamble before the first `|`; `stages` are the ordered pipe stage sequence.

**Red Gate test:** `test_BC_2_11_020_sqlpipe_ast_round_trip` — parse
`SELECT * FROM crowdstrike_detections | enrich threat_score(src_ip) | limit 10` and assert
the result is `Ast::SqlPipe(_)` with `stages.len() == 2`.

**AC-002** (traces to BC-2.11.020 postcondition 5 — FORBID-BOTH): A SQL→Pipe composed query
specifying both a SQL `LIMIT N` in the `SELECT … FROM t LIMIT N` head AND a row-capping
`| limit M` or `| tail M` pipe stage returns
`Err(PrismError::RedundantRowLimit { sql_limit: N, pipe_limit: M })` at plan time (after
parse, before any DataFusion execution). MCP mapping: `-32602 INVALID_PARAMS`. The error
message includes both limit values per error-taxonomy E-QUERY-040.

**Red Gate test:** `test_BC_2_11_020_forbid_both_dual_limit_e_query_040` — plan
`SELECT * FROM t LIMIT 5 | enrich fn(x) | limit 3`; assert
`Err(PrismError::RedundantRowLimit { sql_limit: 5, pipe_limit: 3 })`.

**AC-003** (traces to BC-2.11.020 invariant — additive): Pure SQL queries (no `|` outside
string literals) continue to parse as `Ast::Sql`. Pure pipe queries (`FROM t | …`) continue
to parse as `Ast::Pipe`. `Ast::SqlPipe` is strictly additive — no existing parser path changes.

**Red Gate test:** `test_BC_2_11_020_pure_modes_unchanged` — assert `SELECT * FROM t LIMIT 5`
parses as `Ast::Sql(_)` and `FROM t | where severity = 'HIGH'` parses as `Ast::Pipe(_)`.

---

### Area B — Temporal Grammar: NOW() and INTERVAL (BC-2.11.021, ADR-044)

**AC-004** (traces to BC-2.11.021 postconditions — Expr::Now, Expr::Interval,
Expr::TimestampArithmetic, planning-time constant injection): The expression
`WHERE timestamp > NOW() - INTERVAL '24h'` (SQL mode), `| where timestamp > NOW() - 24h`
(Pipe mode), and `timestamp > NOW() - INTERVAL '1h'` (Filter mode) all parse successfully.
At plan time, `Expr::Now` is evaluated to the current UTC timestamp and substituted as a
`Literal::Timestamp` constant before DataFusion execution. The `build_example_query` output
from `prism_describe.rs` (which generates `WHERE timestamp > NOW() - INTERVAL '1h'`) is
valid PrismQL after this AC is implemented.

**Red Gate test:** `test_BC_2_11_021_now_interval_parses_all_three_modes` — assert parse
success for all three mode variants; additionally assert `PrismQlParser::parse_and_plan`
on a `SELECT * FROM t WHERE timestamp > NOW() - INTERVAL '24h'` query succeeds (planning-
time substitution applied; DataFusion receives a concrete timestamp literal).

**AC-005** (traces to BC-2.11.021 invariants and error cases): `NOW()` with any argument
(e.g., `NOW(1)`) returns `Err(E-QUERY-001)` with message "NOW() takes no arguments."
`NOW() + INTERVAL '1h'` (addition) returns `Err(E-QUERY-001)` with message indicating
subtraction-only in v1. `INTERVAL 'bogus'` returns `Err(E-QUERY-001)` with message
indicating a valid duration literal is required.

**Red Gate test:** `test_BC_2_11_021_now_error_cases` — assert three distinct E-QUERY-001
errors for: `NOW(1)`, `NOW() + INTERVAL '1h'`, and `INTERVAL 'bogus'`.

---

### Area C — Auto-Generated prismql://reference (BC-2.11.022, ADR-045)

**AC-006** (traces to BC-2.11.022 postconditions — content requirements): The MCP resource
`prismql://reference` is served by `build_reference_content(infusion_registry: Option<&InfusionRegistry>)`.
The assembled string includes ALL of the following sections: (1) mode overview covering
Filter / SQL / Pipe / SqlPipe, (2) SQL mode BNF, (3) Pipe mode BNF + stage list, (4)
SQL→Pipe composition with FORBID-BOTH note, (5) operators table (including `CONTAINS`,
`=~`, `IN CIDR`, `HAS`, `MISSING`, `BETWEEN`, `IS NULL`, `IS NOT NULL`), (6) aggregates/stats,
(7) temporal grammar (NOW, INTERVAL, subtraction-only), (8) virtual fields and scope model,
(9) case-sensitivity note, (10) column naming note, (11) LIMIT/head/limit equivalence note,
(12) E-QUERY error quick-reference, (13) enrichment section. The static `pql_reference.md`
file is RETIRED.
> **CORRECTION (remove-uncertainty pass-2, D-1110, develop@903c8fcb):** v1.1 said the
> `include_str!` for the reference lives in `resources.rs`. **It does not.** Verified on
> 903c8fcb: the embedded constant is
> `PQL_REFERENCE_CONTENT: &str = include_str!("../pql_reference.md")` in
> `crates/prism-mcp/src/resources/schema.rs`, and `prismql://reference` is served by
> `schema::render_pql_reference_resource()` (also in `resources/schema.rs`), which `resources.rs`
> dispatches to (the `prismql://reference` arm of the `read_resource` handler calls
> `schema::render_pql_reference_resource()`). The implementer adds `build_reference_content`
> (and `REFERENCE_EXAMPLES` / `ExampleKind`) to `resources.rs`, removes the `include_str!` of
> `PQL_REFERENCE_CONTENT` from `resources/schema.rs` (and retires `render_pql_reference_resource`
> or repoints it), and changes the `resources.rs` `read_resource` `prismql://reference` arm to
> call `build_reference_content(Some(&registry))` instead of `schema::render_pql_reference_resource()`.
> The markdown file path (`crates/prism-mcp/src/pql_reference.md`) is unchanged — the `../` in the
> `include_str!` resolves there from `resources/schema.rs`.

**Red Gate test:** `test_BC_2_11_022_reference_content_completeness` — call
`build_reference_content(None)` and assert the returned string contains key phrases from
each required section (mode names, operator names, NOW(), virtual field names, error codes).

**AC-007** (traces to BC-2.11.022 postconditions — CI 3-tier gate): A shared
`const REFERENCE_EXAMPLES: &[(ExampleKind, &'static str, &'static str)]` (i.e., `(kind, title, snippet)`) in `resources.rs` drives three
CI gate test assertions: (1) **positive round-trip gate** — every `ExampleKind::Positive`
example parses to `Ok(_)` via `PrismQlParser::parse`; (2) **negative E-QUERY-040 gate** —
every `ExampleKind::NegativeE040` example returns
`Err(PrismError::RedundantRowLimit { .. })`; (3) **registry-parity gate** — a test registry
with known infusions produces an enrichment section that contains exactly those infusion
names, no more, no less.

**Red Gate test:** `test_BC_2_11_022_ci_3tier_gate` — assert all three gate assertions
pass using the shared `REFERENCE_EXAMPLES` constant.

**AC-008** (traces to BC-2.11.022 invariants): `build_reference_content(None)` completes
synchronously without panicking when `infusion_registry` is `None`. The returned string's
enrichment section contains the placeholder: "Call `list_infusions` to see available
enrichment functions for your deployment." The `read_resource` handler passes the live
`Arc<ArcSwap<InfusionRegistry>>` value (reload-aware per ADR-042); no caching of the
assembled string.

**Red Gate test:** `test_BC_2_11_022_none_registry_placeholder` — assert
`build_reference_content(None)` returns a string containing the placeholder text and does
not panic.

---

### Area D — Three-Mode Correctness (BC-2.11.023, BC-2.11.002, ADR-046)

**AC-009** (traces to BC-2.11.023 postconditions — mode-bridge D1): When a SQL-mode parse
fails at a `|` token that is NOT a valid SQL→Pipe composition trigger, the generic Chumsky
expectation dump is REPLACED with the verbatim mode-bridge message from BC-2.11.023 §D1:

```
E-QUERY-001: parse error near '|': pipe stages are not valid after a SQL SELECT query in SQL mode.
To use pipe stages (enrich, where, limit, sort, stats, dedup, fields), use one of:
  1. SQL+pipe composition:  SELECT <cols> FROM <table> | <pipe_stage> …
  2. Pipe mode only:        FROM <table> | where <predicate> | <stage> …
See prismql://reference for the complete grammar.
```

The test MUST assert ALL THREE of the following substrings are present in the error string:
(a) `(enrich, where, limit, sort, stats, dedup, fields)` — the stage-keyword enumeration,
(b) `1. SQL+pipe composition:` and `2. Pipe mode only:` — the numbered alternatives,
(c) `See prismql://reference for the complete grammar.` — the reference pointer.
A raw Chumsky token list (e.g. `expected one of ...`) MUST NOT appear in the error string (negative
control).

**Red Gate test:** `test_BC_2_11_023_mode_bridge_d1_sql_pipe_diagnostic` — trigger mode-bridge
on a query like `SELECT * FROM t | INVALID_KEYWORD`; assert the error string contains all three
required substrings listed above AND does NOT contain a raw Chumsky token-expectation dump.

**AC-010** (traces to BC-2.11.023 postconditions — normalized_pql field): The
**`StructuredErrorFields`** struct (in `crates/prism-mcp/src/error_mapping.rs`, the canonical
MCP structured-error payload — it is `#[non_exhaustive]` and already carries `near_text`,
`reference_pointer`, `available_columns`, `did_you_mean`, all via the same
`#[serde(skip_serializing_if = "Option::is_none")]` pattern) gains a
`normalized_pql: Option<String>` field. On a D1 mode-bridge error where a best-effort pipe-mode
rewrite is derivable (simple `FROM/WHERE/LIMIT` cases), `normalized_pql` is `Some(rewrite_string)`.
When not derivable (JOINs, subqueries), `normalized_pql` is `None`. The field serializes to JSON
when `Some` and is absent from the JSON payload when `None`
(`#[serde(skip_serializing_if = "Option::is_none")]`).

> **CORRECTION (remove-uncertainty pass, D-1110, 2026-06-24):** v1.0 of this story named a
> struct `ParseErrorDetails` and placed it in `crates/prism-query/src/error.rs`. **No such
> struct exists in the codebase** (verified by grep across `crates/`). The actual MCP-facing
> structured-error payload is `StructuredErrorFields` in `crates/prism-mcp/src/error_mapping.rs`.
> Because `prism-query` MUST NOT depend on `prism-mcp` (dependency-direction rule), the
> `normalized_pql` rewrite STRING is computed in `prism-query`'s error-recovery path (it already
> hosts the `normalized_pql` Chumsky re-serializer for SUCCESS responses, per
> `engine.rs` `normalized_pql` re-serializer / S-DEMO-PRISMQL-ONBOARDING-001-B) and the FIELD is
> carried on `StructuredErrorFields` in `prism-mcp`. The implementer adds the field to
> `StructuredErrorFields` (and its `new`/builder constructors) and populates it from the
> `prism-query` rewrite when mapping `PrismError::QueryParseFailed` in `error_mapping.rs`.
> Since `StructuredErrorFields` is already `#[non_exhaustive]`, adding a field does NOT change
> the `ci.yml` non-exhaustive `EXPECTED` count.

**Red Gate test:** `test_BC_2_11_023_normalized_pql_on_mode_bridge_error` — trigger a D1
mode-bridge on a simple query (`SELECT * FROM t WHERE severity = 'HIGH' | limit 10`); assert
the resulting `StructuredErrorFields.normalized_pql` (the MCP error payload) is
`Some("FROM t | where severity = 'HIGH' | limit 10")` (or equivalent pipe rewrite).

**AC-011** (traces to BC-2.11.023 postconditions D4 + BC-2.11.002 postconditions): Filter
mode end-to-end execution is validated. `test_filter_mode_simple_predicate` exists:
executes `severity='HIGH'` as `Ast::Filter` against a mocked/DTU sensor source and verifies
rows matching the predicate are returned. `test_filter_mode_with_source` exists: executes
a source-qualified filter (e.g., `crowdstrike_detections | severity='HIGH'`) and verifies
rows returned. Both tests use `QueryEngine::execute`, not just `PrismQlParser::parse`.

**Red Gate test:** `test_BC_2_11_023_filter_mode_end_to_end_execution` (wraps or aliases
`test_filter_mode_simple_predicate` and `test_filter_mode_with_source`).

**AC-012** (traces to BC-2.11.023 invariants — D7 shared-predicate-grammar): A predicate
that parses in Filter mode parses identically in SQL `WHERE` and pipe `| where`. There is
exactly one `build_predicate_parser` function consumed by all three modes — no mode-specific
predicate grammar extensions. The graduation path works: `severity = 'HIGH'` (filter) ↔
`SELECT * FROM t WHERE severity = 'HIGH'` (SQL) ↔ `FROM t | where severity = 'HIGH'` (pipe)
— all share identical predicate grammar.

**Red Gate test:** `test_BC_2_11_023_d7_shared_predicate_grammar` — parse the predicate
`severity = 'HIGH' AND risk_score > 50` in all three entry forms and assert each produces
an AST with equivalent predicate semantics.

**AC-027** (traces to BC-2.11.023 postconditions — mode-bridge D2 / ADR-046 D2): When a
pipe-mode parse fails because an uppercase SQL clause keyword (`SELECT` or `ORDER BY`) appears
in stage position (i.e., not preceded by a valid `|` introducing a pipe stage), the generic
Chumsky expectation dump is REPLACED with the verbatim mode-bridge message from BC-2.11.023 §D2:

```
E-QUERY-001: parse error near '<keyword>': SQL clauses are not valid as pipe stages.
In pipe mode, use lowercase stage keywords: 'where', 'sort', 'limit', 'stats'.
Example: FROM <table> | where severity = 'HIGH' | sort time DESC | limit 10
```

The test MUST assert both a positive and a negative control:
- **Positive control:** a pipe-mode query with an uppercase SQL clause keyword in stage position
  (e.g. `FROM crowdstrike_detections | WHERE severity = 'HIGH'` or
  `FROM crowdstrike_detections | ORDER BY time DESC`) returns `Err(E-QUERY-001)` and the error
  string contains ALL of: `SQL clauses are not valid as pipe stages`, `'where', 'sort', 'limit',
  'stats'`, and the example line `FROM <table> | where severity = 'HIGH' | sort time DESC | limit 10`.
- **Negative control:** the error string MUST NOT contain a raw Chumsky token-expectation dump
  (e.g. `expected one of ...`).

Note per BC-2.11.023 §D2: `WHERE` and `LIMIT` (uppercase) already parse in pipe mode because
keywords are case-insensitive; D2 fires specifically when `SELECT` or `ORDER BY` appears in
pipe stage position.

**Red Gate test:** `test_BC_2_11_023_mode_bridge_d2_sql_keyword_in_pipe_position` — trigger D2
on `FROM crowdstrike_detections | WHERE severity = 'HIGH'` (or `| ORDER BY ...`); assert
positive and negative controls above.

---

### Area E — MCP Infrastructure Fixes (BC-2.10.015, BC-2.10.016, BC-2.10.017)

**AC-013** (traces to BC-2.10.015 postconditions — OrgRegistry Arc-DI wiring): The
`FeatureFlagEvaluator` constructor signature changes from the current
`FeatureFlagEvaluator::new(client_capabilities: BTreeMap<String, ClientCapabilities>)` to
`FeatureFlagEvaluator::new(client_capabilities: BTreeMap<String, ClientCapabilities>, org_registry: Arc<OrgRegistry>)`.
The existing `client_exists(client_id: &str) -> bool` method (currently
`self.client_capabilities.contains_key(client_id)`) is changed to consult the `OrgRegistry`.

> **CORRECTION (remove-uncertainty pass, D-1110, 2026-06-24):** v1.0 named the OrgRegistry
> accessor `OrgRegistry::contains(client_id)`. **No such method exists.** The real type is
> `prism_core::org_registry::OrgRegistry` (`crates/prism-core/src/org_registry.rs`), and its
> existence accessor is `slug_exists(&self, slug: &OrgSlug) -> bool` (a thin wrapper over
> `resolve(slug).is_some()`). Note the signature takes `&OrgSlug`, NOT `&str` — so
> `client_exists(client_id: &str)` must construct/validate an `OrgSlug` from the `&str` first
> (e.g. via the `OrgSlug` `TryFrom`/parse path). A malformed `client_id` that cannot form a
> valid `OrgSlug` returns `client_registered: false` (it cannot be registered if it cannot be a
> valid slug). The implementer MUST NOT use `OrgSlug::new_unchecked` here (forbidden in
> production paths per CLAUDE.md / AD-017). No new `OrgRegistry` method is required —
> `slug_exists` already covers the need, so the `crates_touched` note about possibly adding
> `OrgRegistry::contains` is RETRACTED (see crates_touched correction below).

**Red Gate test:** `test_BC_2_10_015_client_registered_true_from_org_registry` — construct
a `FeatureFlagEvaluator` with an `OrgRegistry` containing `org-c` but with an empty
`client_capabilities` map; call `list_capabilities("org-c")`; assert
`client_registered: true`.

**AC-014** (traces to BC-2.10.015 postconditions — demo provisioning path): An org
provisioned ONLY via spec overlays (no `[clients.*]` entry in `prism.toml`) returns
`client_registered: true` from `list_capabilities`. An org NOT in `OrgRegistry` returns
`client_registered: false`. The capability matrix (`capabilities` field) continues to
reflect `prism.toml` write-capability config exactly as before.

**Red Gate test:** `test_BC_2_10_015_demo_provisioned_org_registered` — assert `org-c`
(in OrgRegistry, not in prism.toml `[clients]`) returns `client_registered: true`; assert
`non-existent-org` returns `client_registered: false`.

**AC-015** (traces to BC-2.10.016 postconditions — prompts fast-return): All registered
MCP prompts (`triage_alerts`, `investigate_host`, `client_overview`, `cross_client_status`,
`query_tutorial`) return a `prompts/get` response within 5 seconds. The fix resolves the
BLOCKER-003 hang in `#[prompt_handler]` macro expansion / `PromptRouter` dispatch
(investigation protocol per ADR-046 D6). Implementer MUST run `cargo expand -p prism-mcp`
to locate the blocking point before writing any fix code.

**Red Gate test:** `test_BC_2_10_016_prompts_fast_return_within_5s` — start a real rmcp
server instance; send `prompts/get query_tutorial` with `client_id: "test-org"` and
`prompts/get investigate_host` with `client_id: "test-org"` + `hostname: "host-001"`;
assert both return within 5 seconds (use `tokio::time::timeout`).

**AC-016** (traces to BC-2.10.016 invariants — INV-PROMPT-REQUIRED-ARGS option (a)): For a
prompt with a required argument (`investigate_host.hostname`), when that argument is missing,
the dispatch machinery MUST NOT hang — it substitutes the literal string `(unknown)` for the
missing argument and returns **Ok** within 5 seconds. No structured MCP error is returned;
the key contract is the no-hang fast-return guarantee.

**Red Gate test:** `test_BC_2_10_016_missing_required_arg_fast_error` — send
`prompts/get investigate_host` with `client_id` but no `hostname`; assert `result.is_ok()`
and that the response is returned within 5 seconds (no hang; no structured MCP error).

**AC-017** (traces to BC-2.10.017 postconditions — fast-fail guard order): For tools in
`NOT_YET_AVAILABLE_TOOLS` (`list_infusions`, `plugin_status`, `infusion_status`), the
fast-fail short-circuit (returning `-32003 not_yet_available`) fires BEFORE the
`emit_tool_audit(...).await` call in the handler body. The JSON-RPC `-32003` error response
is returned within 1 second of request receipt.

> **CORRECTION (remove-uncertainty pass, D-1110, 2026-06-24):** v1.0 framed this as
> "`emit_tool_audit` uses `mpsc::Sender::send` vs `try_send`." **That mechanism does not
> match the code.** Verified in `crates/prism-mcp/src/server.rs`:
> `async fn emit_tool_audit(...)` does NOT use a `tokio::sync::mpsc::Sender` at all — it calls
> `writer.write_tool_call(...).await` directly on an `Arc<dyn AuditWriter>` and returns
> `Result<Option<String>, ErrorData>`. The not-yet-available stub handlers (e.g.
> `create_schedule`, `list_schedules`) currently call `scan_inputs_audited(...).await?` then
> `emit_tool_audit(...).await?` and ONLY THEN `Err(not_yet_available_msg(...))`. The blocking
> risk is the `.await` on the durable audit write, NOT an mpsc full-channel block.
>
> **Therefore the fix is the GUARD-REORDER path, not the try_send path:** in each
> NOT_YET_AVAILABLE tool handler, return the `-32003` short-circuit BEFORE the
> `emit_tool_audit(...).await` (and before `scan_inputs_audited`, since no scan is needed when
> the tool is unavailable). The `try_send` framing in AC-018 is RETRACTED — there is no mpsc
> sender on this path to convert. If a future audit-channel-backed `AuditWriter` is introduced,
> the non-blocking concern would re-apply at the `AuditWriter` impl, not at `emit_tool_audit`.

**Red Gate test:** `test_BC_2_10_017_not_yet_available_fast_fail_under_1s` — invoke
`list_infusions`, `plugin_status`, and `infusion_status` via the MCP server with an
`AuditWriter` whose `write_tool_call` blocks/sleeps (simulating a slow durable audit write);
assert all three return `-32003` within 1 second because the guard short-circuits before the
audit `.await`. (The test substitutes a deliberately-slow `Arc<dyn AuditWriter>` rather than a
"saturated audit channel" — see correction above.)

**AC-018** (traces to BC-2.10.017 invariants — INV-AUDIT-NON-BLOCKING): No
NOT_YET_AVAILABLE tool's `-32003` response path is gated behind an `await` on a durable audit
write or any other potentially-blocking I/O. The guard short-circuit precedes
`emit_tool_audit`. (Verified mechanism: `emit_tool_audit` awaits `AuditWriter::write_tool_call`;
the invariant is that this await is never on the fast-fail critical path.)

> **CORRECTION (D-1110):** v1.0's `mpsc::Sender::try_send` / `Err(Full)` framing is RETRACTED
> (see AC-017 correction). The invariant is restated as "guard precedes any blocking audit await."

**Red Gate test:** `test_BC_2_10_017_not_yet_available_guard_precedes_audit` — assert (by code
inspection or by injecting a panicking/blocking `AuditWriter`) that the `-32003` short-circuit
for each NOT_YET_AVAILABLE tool returns WITHOUT invoking the blocking audit `.await`.

---

### Area F — Implementer / Doc ACs (finding IDs — no BC trace required)

> These ACs are correctness requirements with finding IDs from the audits. They are
> verified by inspection, integration tests, or timing assertions rather than classic
> Red Gate unit tests. They do NOT count toward the 20 Red Gate test total.

**AC-019** [BLOCKER-001 / implementer-AC — D-1326 adjudication]: CrowdStrike OAuth plugin
does not hang on second Prism session start.

> **D-1326 ROOT-CAUSE ADJUDICATION:** The cross-session hang was architecturally misdiagnosed
> in the original story draft. The investigation showed PluginKvStore is **in-memory** and fresh
> per `prism start` — cross-session KV staleness is **impossible** by construction. The removed
> `reset_token_cache` function and its test (found at code HEAD 3fa69207) confirmed this.
>
> The real root cause is a **connectivity connect-timeout** in the plugin HTTP client:
> `PLUGIN_HTTP_CLIENT_TIMEOUT_SECS=30` sets a total request timeout but there is no separate
> `connect_timeout`. If the DTU is not yet up when Prism starts, the plugin waits the full 30s
> before failing. The structural fix (per-sensor TOML-tuneable connect/request timeouts,
> boot-degraded mode, connectivity diagnostic, retry-with-backoff) is a day-2 resilience
> concern that requires new ADR/BC/TOML schema work.
>
> **DEFERRAL (D-1326 / CLAUDE.md Rule 3 feature-ordering):** The runtime connect-timeout fix is
> **deferred to S-RESILIENCE-FEDERATED-001** (day-2 resilience epic). This is a legitimate
> feature-ordering deferral: the scope (new TOML schema, ADRs, boot-degraded model) exceeds
> T13 demo story budget, and a concrete anchor story exists. For the T13 demo, the hang is
> avoided operationally via the **runbook DTU health-check** (demo-pre-flight runbook Fix B:
> verify DTU is healthy before starting Prism). No code change is required in this story for
> BLOCKER-001.

- BC-2.01.005 (CrowdStrike OAuth2 Authentication and Two-Step Fetch) governs the correct
  token acquisition behavior in the production path. No new BC required for this story.
- Verification: the demo runbook DTU health-check step (Fix B) is confirmed present and
  tested in the demo pre-flight sequence; no session-start hang occurs when the DTU is
  healthy before `prism start`.

**AC-020** [BLOCKER-002 / implementer-AC]: Demo runbook §5.5 pipe syntax is valid PrismQL.
- The pipe syntax example in demo runbook §5.5 currently uses invalid syntax
  (`FROM t WHERE … LIMIT N` — aspirational BNF). Replace with valid pipe syntax:
  `FROM t | where <predicate> | enrich fn(col) | limit N`.
- Verification: the corrected example parses successfully via `PrismQlParser::parse`.
- This is a trivial 1-line runbook/script fix with no BC. Location: `scripts/` or demo
  runbook file (implementer reads the current file and corrects the example).

**AC-021** [GRAMMAR-004 / implementer-AC]: `E-QUERY-036` (`UnknownSourceTable` error) carries
`available_tables: Vec<String>` and `did_you_mean: Option<String>`, achieving parity with
E-QUERY-037/038/039.
- The `UnknownSourceTable` error variant (or its equivalent structured details struct) gains
  `available_tables` and `did_you_mean` fields following the same pattern as
  `TableNotAvailableDetails` (E-QUERY-037) and `ColumnNotFoundDetails` (E-QUERY-038).
- `did_you_mean` uses `strsim::levenshtein` ≤ 3 best candidate, consistent with E-QUERY-037.
- Verification: trigger E-QUERY-036 on a query with a typo'd table name; assert the error
  includes available tables and a did_you_mean suggestion.

**AC-022** [GRAMMAR-005 / GRAMMAR-015 / implementer-AC]: Parse-time `enrich` errors provide
actionable guidance.
- When `| enrich fn_name` is missing the `(col)` argument form, the error message says:
  "`enrich` requires a column argument: `| enrich <infusion>(<column>)`. Example:
  `| enrich threat_score(iocs_value)`" — NOT the bare "expected '('".
- Parse-time enrich shape errors (GRAMMAR-015: raw token-expectation output) are replaced
  with the plan-time quality bar. Implementer MUST add a post-parse heuristic in
  `error_recovery.rs` for `enrich`-position errors.
- Verification: parse `FROM t | enrich threat_score`; assert the error message contains
  the column argument guidance text.

**AC-023** [GRAMMAR-006 / implementer-AC]: `IS NOT NULL` semantics on JSON-list columns
are documented in `prismql://reference`.
- The reference includes a note: "`IS NOT NULL` on a JSON-list field returns `true` if the
  field is present and non-null (empty list `[]` is NOT null; `null` value is null)."
- Implementer MUST verify this is the actual code behavior (`prism-query` execution path
  for `IS NOT NULL` on a JSON column) before adding the doc note. If the behavior differs,
  document the actual behavior.
- Verification: the assembled reference string (from `build_reference_content(None)`)
  contains the IS NOT NULL / JSON-list note.

**AC-024** [GRAMMAR-013 / implementer-AC]: After all other ACs in this story are implemented,
a Prism analyst can discover and correctly write a `| enrich` query from the teaching
surfaces alone (no out-of-band knowledge required).
- The 10 "must know out-of-band" items from GRAMMAR-013 are ALL discoverable via
  `prism_describe` output + `prismql://reference` + error messages.
- This is an acceptance test AC: implementer runs through the GRAMMAR-013 checklist and
  documents in the PR description that each item is now surfaced.
- Verification: PR description includes a table mapping each GRAMMAR-013 item to the
  teaching surface that now covers it.

**AC-025** [GRAMMAR-015 / implementer-AC — extension of AC-022]: `enrich` stage parse
errors in nested pipe positions (multi-stage pipelines) also produce the guided error
message, not a bare Chumsky token dump. The fix is complete when error_recovery.rs's
`enrich`-position heuristic fires in all pipeline positions, not just the first stage.
- Verification: parse `FROM t | where severity = 'HIGH' | enrich threat_score`; assert
  the error message contains the column argument guidance (same as AC-022).

**AC-026** [GRAMMAR-013-AGG / implementer-AC]: Aggregate discoverability — the `prismql://reference`
aggregates/stats section documents `count`, `sum`, `avg`, `min`, `max`, `percentile`,
`distinct_count`, and the `stats <agg> [by <field>]` form. At least one test asserts the
reference string contains these aggregate names (covered by the AC-006 completeness test
if it checks the aggregates section).
- Verification: `test_BC_2_11_022_reference_content_completeness` (AC-006) asserts the
  assembled reference contains "percentile" and "distinct_count" as key phrases.

---

## Architecture Mapping

| Component | File | Pure/Effectful |
|-----------|------|----------------|
| `Ast` enum (add `SqlPipe` variant) | `crates/prism-query/src/ast.rs` | Pure |
| `Expr` enum (add `Now`, `Interval`, `TimestampArithmetic`) | `crates/prism-query/src/ast.rs` | Pure |
| Mode detection tristate (`SqlPipeMode`) | `crates/prism-query/src/filter_parser.rs` | Pure |
| `build_expr_parser` — temporal grammar extension | `crates/prism-query/src/filter_parser.rs` | Pure |
| SQL→Pipe composition combinator `parse_sql_pipe` | `crates/prism-query/src/filter_parser.rs` | Pure |
| `execute_against_session` — `Ast::SqlPipe` execution arm (CTE + pipe SQL lowering) | `crates/prism-query/src/materialization.rs` | Effectful |
| `inject_now` — NOW() constant injection (replaces `Expr::Now` with `Literal::Timestamp`); called from `run_materialization_pipeline` Step 1a | `crates/prism-query/src/lib.rs` | Pure |
| `plan_sqlpipe_query` — FORBID-BOTH E-QUERY-040 plan-time check; called from `run_materialization_pipeline` Step 1b | `crates/prism-query/src/lib.rs` | Pure |
| `error_recovery.rs` — mode-bridge heuristic (D1/D2) + `normalized_pql` rewrite string | `crates/prism-query/src/error_recovery.rs` | Pure |
| `StructuredErrorFields` — add `normalized_pql: Option<String>` (NOT `ParseErrorDetails`/prism-query; D-1110 correction) | `crates/prism-mcp/src/error_mapping.rs` | Pure |
| `PrismError::RedundantRowLimit { sql_limit: u64, pipe_limit: u64 }` | `crates/prism-core/src/error.rs` | Pure |
| `build_reference_content` + static constants + example array | `crates/prism-mcp/src/resources.rs` | Pure (fn); Effectful (server) |
| Retire `pql_reference.md` (remove `PQL_REFERENCE_CONTENT include_str!` in `resources/schema.rs`; repoint `read_resource` arm in `resources.rs`; D-1110 pass-2 — `include_str!` is in `resources/schema.rs`, NOT `resources.rs`) | `crates/prism-mcp/src/pql_reference.md` + `crates/prism-mcp/src/resources/schema.rs` + `resources.rs` | n/a |
| `FeatureFlagEvaluator` — add `Arc<OrgRegistry>` field | `crates/prism-security/src/feature_flag.rs` | Pure |
| `client_exists` — build `OrgSlug` from `&str` and consult `OrgRegistry::slug_exists(&OrgSlug)` (D-1110: `contains` does not exist) | `crates/prism-security/src/feature_flag.rs` | Pure |
| `list_capabilities` handler — wire `Arc<OrgRegistry>` (function `list_capabilities` in `server.rs`) | `crates/prism-mcp/src/server.rs` | Effectful |
| `#[prompt_handler]` macro expansion investigation | `crates/prism-mcp/src/prompts.rs` + `server.rs` | Effectful |
| `emit_tool_audit` — reorder NOT_YET_AVAILABLE guard BEFORE the audit `.await` (D-1110: it awaits `AuditWriter::write_tool_call`, NOT an mpsc try_send) | `crates/prism-mcp/src/server.rs` | Effectful |
| Filter mode execution tests | `crates/prism-query/tests/filter_mode.rs` | Effectful |

**Subsystem SS-11** owns grammar and engine work (prism-query, prism-core error variant).
**Subsystem SS-10** owns MCP surface work (prism-mcp, prism-security wiring).
All grammar changes are Pure (no I/O); execution arm additions are Effectful (query
fan-out, DataFusion planning). All MCP handler changes are Effectful.

---

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `SELECT * FROM t | unknown_stage` — mode-bridge fires | E-QUERY-001 with mode-bridge message; `normalized_pql` may be set |
| EC-002 | `SELECT * FROM t LIMIT 5 | limit 3` — FORBID-BOTH with `| limit` pipe stage | E-QUERY-040 with sql_limit=5, pipe_limit=3 |
| EC-002b | `SELECT * FROM t LIMIT 5 | enrich fn(x) | tail 3` — FORBID-BOTH with `| tail` pipe stage (BC-2.11.020 v1.5 EC-11-020-008) | E-QUERY-040 with sql_limit=5, pipe_limit=3; `| tail M` is a row-capping stage in the same FAMILY as `| limit M` |
| EC-003 | `SELECT * FROM t LIMIT 10 | enrich fn(x)` — SQL LIMIT, no pipe `| limit` or `| tail` | Valid Ast::SqlPipe; SQL LIMIT 10 applies |
| EC-004 | `NOW() - INTERVAL '0s'` | Valid; lowers to current timestamp |
| EC-005 | `NOW() + INTERVAL '1h'` | E-QUERY-001: subtraction-only in v1 |
| EC-006 | `build_reference_content(None)` called before WASM plugins load | Enrichment section shows placeholder; all other sections accurate |
| EC-007 | `list_capabilities("org-c")` after demo-setup.sh (no prism.toml [clients.*]) | `client_registered: true` from OrgRegistry |
| EC-008 | `prompts/get investigate_host` with `hostname` argument MISSING | Substitutes `(unknown)` for missing arg, returns Ok within 5s; MUST NOT hang (no structured MCP error per BC-2.10.016 v1.1 INV-PROMPT-REQUIRED-ARGS option (a)) |
| EC-009 | `list_infusions` invoked while the durable audit write (`AuditWriter::write_tool_call`) is slow/blocking (D-1110: not a "saturated channel" — emit_tool_audit awaits the writer directly) | JSON-RPC -32003 within 1s; guard short-circuits BEFORE the audit await |
| EC-010 | CrowdStrike OAuth: second Prism session (DTU state reset between sessions) | Query completes without 30s hang |
| EC-011 | `IS NOT NULL` on JSON-list column | Returns true if field present and non-null; documented in reference |

---

## Token Budget Estimate

| Component | Estimated Tokens |
|-----------|----------------|
| Story spec (this file) | ~8,000 |
| BC files (8 BCs) | ~24,000 |
| ADR files (4: ADR-043/044/045/046) | ~20,000 |
| `ast.rs` (current grammar) | ~6,000 |
| `filter_parser.rs` + `pipe_parser.rs` + `sql_parser.rs` | ~12,000 |
| `engine.rs` (query engine, executor, planner) | ~8,000 |
| `error_recovery.rs` + `error.rs` | ~4,000 |
| `resources.rs` + `server.rs` (MCP surface) | ~8,000 |
| `feature_flag.rs` (prism-security) | ~3,000 |
| `prompts.rs` (MCP prompts) | ~3,000 |
| Test files to write (20 Red Gate tests) | ~10,000 |
| Tool outputs (cargo expand, cargo nextest) | ~5,000 |
| **Total estimate** | **~111,000** |

**Context window budget guidance:** This story is large (111k token estimate). It MUST be
worked area-by-area. The TDD cascade operator MUST dispatch one area at a time:
Area A (grammar), Area B (temporal), Area C (reference), Area D (modes), Area E (MCP infra),
Area F (polish). Each area is independently verifiable with `just iter prism-query` or
`just iter prism-mcp`. All areas land in a SINGLE PR per human directive.

**WARNING:** At 111k tokens this story is at the upper bound for a single implementer dispatch.
The orchestrator SHOULD consider area-by-area dispatch with a single PR aggregation. If the
orchestrator decides to split, the recommended sub-groupings are:
- Sub-A: Areas A+B+D (prism-query grammar work, one crate)
- Sub-B: Areas C+E+F (prism-mcp + prism-security + polish)
Human authorization required before any split — this is flagged per story-writer mandate.

---

## Tasks

### Phase 1 — Red Gate Test Stubs (test-writer)

Write failing tests for all 20 BC-traced ACs (Areas A–E). The test-writer MUST read all
8 BC files before writing tests. For each area:

- **Area A:** `test_BC_2_11_020_sqlpipe_ast_round_trip`, `test_BC_2_11_020_forbid_both_dual_limit_e_query_040`, `test_BC_2_11_020_pure_modes_unchanged`
- **Area B:** `test_BC_2_11_021_now_interval_parses_all_three_modes`, `test_BC_2_11_021_now_error_cases`
- **Area C:** `test_BC_2_11_022_reference_content_completeness`, `test_BC_2_11_022_ci_3tier_gate`, `test_BC_2_11_022_none_registry_placeholder`
- **Area D:** `test_BC_2_11_023_mode_bridge_d1_sql_pipe_diagnostic`, `test_BC_2_11_023_normalized_pql_on_mode_bridge_error`, `test_BC_2_11_023_filter_mode_end_to_end_execution`, `test_BC_2_11_023_d7_shared_predicate_grammar`, `test_BC_2_11_023_mode_bridge_d2_sql_keyword_in_pipe_position`
- **Area E:** `test_BC_2_10_015_client_registered_true_from_org_registry`, `test_BC_2_10_015_demo_provisioned_org_registered`, `test_BC_2_10_016_prompts_fast_return_within_5s`, `test_BC_2_10_016_missing_required_arg_fast_error`, `test_BC_2_10_017_not_yet_available_fast_fail_under_1s`, `test_BC_2_10_017_not_yet_available_guard_precedes_audit` (D-1110: renamed from `..._emit_tool_audit_try_send_non_blocking` — no try_send path exists)

All test bodies are `todo!()` stubs. `just iter prism-query` and `just iter prism-mcp`
must both fail (Red Gate requirement BC-5.38.001).

### Phase 2 — Area A Implementation (implementer)

1. Add `Ast::SqlPipe(SqlPipeQuery)` variant to `ast.rs`. Add `SqlPipeQuery { head: SqlQuery, stages: Vec<PipeStage> }` struct. Mark `Ast` with `#[non_exhaustive]` (already present; new variant is additive). Add wildcard `_ => {}` arm to any external `Ast` match sites.
2. Extend mode detection in `filter_parser.rs` to tristate: `SqlPipeMode::Sql`, `SqlPipeMode::Pipe`, `SqlPipeMode::SqlPipe`. Dispatch to `parse_sql_pipe` when `SqlPipeMode::SqlPipe` detected.
3. Implement `parse_sql_pipe` Chumsky combinator: parse SQL preamble as `SqlQuery`, then parse `| <stage>*` as `Vec<PipeStage>`.
4. Add `Ast::SqlPipe` arm to `QueryEngine::execute`: execute head as SQL → pass Arrow batch through pipe stages.
5. Add FORBID-BOTH plan-time check: if `sq.head.limit.is_some()` AND any stage matches `PipeStage::Limit(_) | PipeStage::Tail(_)` (i.e., any row-capping pipe stage per BC-2.11.020 v1.5 FAMILY rule), return `Err(PrismError::RedundantRowLimit { sql_limit, pipe_limit })`.
6. Add `PrismError::RedundantRowLimit { sql_limit: u64, pipe_limit: u64 }` to `prism-core/src/error.rs`. Add explicit `-32602 INVALID_PARAMS` arm in `map_prism_error`.
7. Run `just iter prism-query` — AC-001/AC-002/AC-003 tests must turn GREEN.

### Phase 3 — Area B Implementation (implementer)

1. Add `Expr::Now`, `Expr::Interval(Duration)`, `Expr::TimestampArithmetic { base: Box<Expr>, op: BinaryOp, offset: Duration }` to `ast.rs`. (`Expr` already exists at `ast.rs` — verified; the additions are new variants on the existing enum, additive.) Use `chrono::Duration` for the `Duration` field.
2. Extend `build_expr_parser` in `filter_parser.rs` (the real shared `build_expr_parser` at `filter_parser.rs`, distinct from `build_predicate_parser` which all three modes also share — both verified present) to recognize `NOW()` → `Expr::Now` (idiomatic chumsky 0.12: `just(NOW).then_ignore(LParen).then_ignore(RParen).map(|_| Expr::Now)`) and `INTERVAL '<dur>'` → `Expr::Interval` (`just(INTERVAL).ignore_then(select!(string-literal)).try_map(parse_interval_string)`). Parse `<expr> - <expr>` as `Expr::TimestampArithmetic` when one side is `Expr::Now`.
3. Add `NOW()` with args → `Err(E-QUERY-001)`; `NOW() + <duration>` (addition) → `Err(E-QUERY-001)` "subtraction-only in v1"; `INTERVAL 'bogus'` → `Err(E-QUERY-001)` via the `try_map` returning a custom error. The interval-string parser is CUSTOM code (chrono 0.4.44 has no duration-string parser — verified D-1110); parse `<int><unit>` (s/m/h/d) and map to `chrono::Duration::{seconds,minutes,hours,days}`.
4. Add planning-time constant injection in `plan_query`: in PrismQL's OWN AST→logical-plan lowering (BEFORE handing the plan to DataFusion), replace `Expr::Now` with a concrete timestamp literal (`chrono::Utc::now()` captured once per query → an Arrow/DataFusion `ScalarValue::TimestampNanosecond` / `Literal`).
   > **DataFusion soundness (verified D-1110):** because PrismQL substitutes its OWN `Expr::Now`
   > node at PrismQL lowering time, it never emits a DataFusion `now()` scalar call — so there is
   > NO collision with DataFusion 53's built-in `now()` and no dependency on DataFusion's
   > `SimplifyExpressions`/`ConstEvaluator` folding behavior (which is documented but NOT
   > guaranteed to fold `now()` at plan time). Rewriting an expression to `Expr::Literal(ScalarValue)`
   > is a first-class, supported DataFusion operation; the prism-query approach (substitute before
   > the plan reaches DataFusion) is the soundest path and sidesteps the documented timezone/cast
   > nuances of DataFusion's own `now()`. Capture the timestamp ONCE per query so all `NOW()`
   > occurrences in a single query observe the same instant (matches DataFusion's documented
   > per-query-constant `now()` semantics).
5. Run `just iter prism-query` — AC-004/AC-005 tests must turn GREEN.

### Phase 4 — Area C Implementation (implementer)

1. In `crates/prism-mcp/src/resources.rs`, add `pub const REFERENCE_EXAMPLES: &[(ExampleKind, &'static str, &'static str)] = &[…]` (i.e., `(kind, title, snippet)`) with positive and negative examples covering all required sections.
2. Add `ExampleKind` enum: `Positive`, `NegativeE040`, `NegativeOther(&'static str)`.
3. Implement `build_reference_content(infusion_registry: Option<&InfusionRegistry>) -> String`. Write each section as a `&'static str` constant. Assemble the enrichment section from the live registry (or placeholder if `None`).
4. Remove the `PQL_REFERENCE_CONTENT = include_str!("../pql_reference.md")` constant from
   `crates/prism-mcp/src/resources/schema.rs` (NOT `resources.rs` — D-1110 pass-2 location
   correction) and retire/repoint `schema::render_pql_reference_resource()`. Update the
   `prismql://reference` arm of the `read_resource` handler in `resources.rs` (which currently
   calls `schema::render_pql_reference_resource()`) to call `build_reference_content(Some(&registry))`.
5. Write the 3-tier CI gate test driven by `REFERENCE_EXAMPLES`.
6. Run `just iter prism-mcp` — AC-006/AC-007/AC-008 tests must turn GREEN.

### Phase 5 — Area D Implementation (implementer)

1. In `error_recovery.rs`, add a heuristic: if `rich_to_parse_error` detects a `|` token in SQL mode context, replace the Chumsky error with the verbatim D1 mode-bridge message from BC-2.11.023 §D1 (all three required substrings: stage-keyword enumeration, numbered alternatives, reference pointer).
2. In `error_recovery.rs`, add a second heuristic (D2): if `rich_to_parse_error` detects `SELECT` or `ORDER BY` in pipe-stage position, replace the Chumsky error with the verbatim D2 mode-bridge message from BC-2.11.023 §D2 (SQL-clauses-not-valid, lowercase-keywords guidance, and the example line).
3. In `error_recovery.rs`, produce the best-effort `normalized_pql` rewrite STRING on D1 mode-bridge errors for simple cases: `SELECT * FROM t WHERE <pred> LIMIT N` → `FROM t | where <pred> | limit N`. Add the `normalized_pql: Option<String>` FIELD to `StructuredErrorFields` (`crates/prism-mcp/src/error_mapping.rs`, `#[serde(skip_serializing_if = "Option::is_none")]`, update `new`/builder) and populate it from the prism-query rewrite in the `QueryParseFailed` mapping arm. (D-1110: there is no `ParseErrorDetails` type; do NOT create one.)
4. Implement or verify `test_filter_mode_simple_predicate` and `test_filter_mode_with_source` use `QueryEngine::execute` (not just parse).
5. Verify shared predicate grammar invariant (D7) — no predicate extension in any mode's local grammar.
6. Run `just iter prism-query` — AC-009/AC-010/AC-011/AC-012/AC-027 tests must turn GREEN.

### Phase 6 — Area E Implementation (implementer)

1. **BC-2.10.015:** Read `crates/prism-security/src/feature_flag.rs` `FeatureFlagEvaluator`
   (current ctor: `new(client_capabilities: BTreeMap<String, ClientCapabilities>)`; current
   `client_exists(&str)` = `self.client_capabilities.contains_key(id)`). Add
   `org_registry: Arc<OrgRegistry>` field. Change constructor to accept it. Change
   `client_exists(&str)` to build an `OrgSlug` from the `&str` (validated path — NOT
   `OrgSlug::new_unchecked`, forbidden in prod) and call
   `self.org_registry.slug_exists(&slug)` (D-1110: the real method is `slug_exists(&OrgSlug)`,
   NOT `contains` — no new OrgRegistry method needed). A `&str` that cannot form a valid
   `OrgSlug` → `false`. Wire `Arc<OrgRegistry>` through `list_capabilities` handler →
   `FeatureFlagEvaluator::new`.
2. **BC-2.10.016:** Run `cargo expand -p prism-mcp` to inspect the `#[prompt_handler(router = self.prompt_router)]` macro expansion on the `ServerHandler for PrismServer` impl (`server.rs`) — this generates the `get_prompt`/`list_prompts` dispatch into `PromptRouter`. Note (D-1110 research): the registered routes in `prompts.rs` use `PromptRoute::new_dyn` closures that read args via `.unwrap_or("(unknown)")`, so the closures themselves do NOT block on missing required args — the hang (if any) is in the macro-generated dispatch / `PromptRouter::get` layer or the rmcp transport, NOT the closures. rmcp 1.7.0's `PromptRouter` is EXPECTED (by design parity with reference MCP SDKs) to validate required args before invoking the closure, but rmcp docs do NOT explicitly document this — so DO NOT assume; the `cargo expand` investigation (ADR-046 D6) is mandatory to locate the actual blocking point before writing any fix. Fix the hang. Write `test_BC_2_10_016_prompts_fast_return_within_5s` using `tokio::time::timeout`.
3. **BC-2.10.017:** Read `emit_tool_audit` in `server.rs`. **D-1110 VERIFIED:** it is an
   `async fn` that calls `writer.write_tool_call(...).await` on an `Arc<dyn AuditWriter>` —
   it does NOT use a `tokio::sync::mpsc::Sender`, so there is NO `send`/`try_send` to swap.
   The correct fix is to **reorder**: in each `NOT_YET_AVAILABLE_TOOLS` handler
   (e.g. `create_schedule`, `list_schedules`, plus `list_infusions`/`plugin_status`/`infusion_status`),
   return the `-32003 not_yet_available` short-circuit BEFORE the `emit_tool_audit(...).await`
   (and before `scan_inputs_audited`). Do NOT attempt a `try_send` conversion. Write the timing
   tests using a deliberately-slow `Arc<dyn AuditWriter>` (not a "saturated channel").
4. Run `just iter prism-mcp` and `just iter prism-security` — Area E tests must turn GREEN.

### Phase 7 — Area F Implementation (implementer)

1. **BLOCKER-001 — DEFERRED to S-RESILIENCE-FEDERATED-001 (D-1326):** No implementer action required. Root cause is a connectivity connect-timeout (not a KV staleness issue — PluginKvStore is in-memory + fresh per `prism start`). The runtime fix (per-sensor TOML connect/request timeouts, boot-degraded, retry) is out of T13 scope. The demo runbook DTU health-check (Fix B) operationally avoids the hang. See AC-019 for full adjudication.
2. **BLOCKER-002:** Find demo runbook §5.5 pipe syntax line. Replace with valid pipe syntax. Verify by parsing the corrected example.
3. **GRAMMAR-004 (AC-021):** Add `available_tables` + `did_you_mean` to E-QUERY-036 error struct.
4. **GRAMMAR-005/015 (AC-022/AC-025):** Add `enrich`-position heuristic to `error_recovery.rs` for guided error message.
5. **GRAMMAR-006 (AC-023):** Verify IS NOT NULL behavior on JSON columns; add reference note.
6. **GRAMMAR-013/AGG (AC-024/AC-026):** Verify all 10 GRAMMAR-013 checklist items are covered by teaching surfaces; document in PR.

### Phase 8 — Final Gate

1. Run `just check` (full workspace). All tests must pass.
2. Verify non-exhaustive gate count in `ci.yml` EXPECTED. NOTE (D-1110): this story adds the
   `normalized_pql` field to the EXISTING `#[non_exhaustive]` `StructuredErrorFields` (no new
   type) and a new VARIANT to the EXISTING `#[non_exhaustive]` `PrismError` enum
   (`RedundantRowLimit`) — neither adds a NEW `#[non_exhaustive]` TYPE, so EXPECTED does NOT
   change UNLESS a brand-new pub struct (e.g. `SqlPipeQuery`) is given `#[non_exhaustive]`. If
   `SqlPipeQuery` is added to a pub-API surface crate (prism-query) with `#[non_exhaustive]`,
   bump EXPECTED in BOTH `ci.yml` and CLAUDE.md in the same commit. Confirm by re-running the
   compile-fail gate at `tests/external/non-exhaustive-violation/`.
3. Run SAP-1: grep `event_type =` across `crates/` — verify any new emission sites have corresponding BC-2.16.002 catalog rows.
4. Run SAP-2: for any TOML spec touched (BLOCKER-001 deferred per D-1326 — no TOML spec changes in this story), verify DTU↔TOML schema parity if other TOML specs are modified.
5. Create PR targeting `develop`. Ensure PR description includes GRAMMAR-013 coverage table (AC-024).

---

## Previous Story Intelligence

**S-DEMO-PRISMQL-ONBOARDING-001-B** (last story in this epic area, merged PR #198 @5504c152):
- Key lessons for this story: `ColumnNotFoundDetails` used boxed form to avoid `result_large_err` — the new `normalized_pql` field is an `Option<String>` addition to the existing `StructuredErrorFields` struct (prism-mcp, D-1110 correction — NOT a `ParseErrorDetails` type, which does not exist); `StructuredErrorFields` already carries several `Option<...>` fields, so the `Option<String>` addition is consistent. If a `result_large_err` clippy lint trips on a `Result` returning this type, follow the `ColumnNotFoundDetails` boxing precedent.
- `normalized_pql` echo field was first designed in BC-2.11.018 for SUCCESSFUL query responses (a separate response struct). This story adds a `normalized_pql` field to `StructuredErrorFields` (ERROR responses) — these are two different structs; do not conflate. The prism-query `normalized_pql` re-serializer in `engine.rs` (from S-DEMO-PRISMQL-ONBOARDING-001-B) is the rewrite-string producer reused here.
- BC array propagation policy: after implementing, verify each BC in `behavioral_contracts` is cited in at least one AC body trace AND appears in the Behavioral Contracts table (POL-7 POL-8).
- The `#[prompt_handler]` macro investigation (BLOCKER-003) was flagged in BC-2.10.016 design notes. ADR-046 D6 provides the investigation protocol. Do NOT guess — expand the macro first.
- Non-exhaustive gate: current EXPECTED=84 on S-5.04 branch; whatever merges second after S-5.04 carries the union count. If this story adds new `#[non_exhaustive]` types, EXPECTED must be bumped in `ci.yml` and CLAUDE.md in the same commit.

**S-5.04** (in-flight, P0, depends_on = this story must wait for S-5.04 merge):
- S-5.04 adds `HealthSummary` and bumps EXPECTED to 84. When this story adds any new `#[non_exhaustive]` type, EXPECTED becomes 85+ (check at Phase 8 gate).

---

## Architecture Compliance Rules

From ADR-022 §C (Arc-DI wiring contract):
- **MANDATORY:** `FeatureFlagEvaluator::new` adds `Arc<OrgRegistry>` via constructor injection — NOT via `Default::default()` or a placeholder constructor. "Wiring, not redesign" (ADR-022 §C) means adding the Arc parameter to the constructor signature; it does NOT permit a placeholder.

From ADR-043 D4 (FORBID-BOTH):
- **MANDATORY:** The FORBID-BOTH ruling is permanent. SQL-wins (SQL `LIMIT` silently takes precedence) is NEVER acceptable. No code path may silently pick one limit over the other; the only valid response is `Err(E-QUERY-040)`.

From ADR-045 D3 (shared example array):
- **MANDATORY:** The CI 3-tier gate MUST use a shared `REFERENCE_EXAMPLES` constant — the doc and the test must consume the SAME constant. Writing two separate lists (one for doc, one for test) violates ADR-045 D3 and produces the exact drift it was designed to prevent.

From ADR-046 D6 (BLOCKER-003 investigation protocol):
- **MANDATORY:** Implementer MUST run `cargo expand -p prism-mcp` before writing any fix code for the prompt hang. Guessing the fix without understanding the macro expansion is forbidden by the investigation protocol.

From BC-2.11.020 invariant INV-FORBID-BOTH-PERMANENT:
- **MANDATORY:** No future "temporary" code silently resolves BOTH limits. E-QUERY-040 is the permanent behavior. Only a product-owner BC amendment can change this.

From CLAUDE.md conventions:
- New `pub` types added to `prism-query` or `prism-mcp` with `#[non_exhaustive]` require updating `ci.yml` EXPECTED count.
- All `event_type =` sites must be in BC-2.16.002 Structured Event Catalog (SAP-1).
- `PrismError::RedundantRowLimit` must have an explicit `-32602 INVALID_PARAMS` arm in `map_prism_error` — MUST NOT fall through to catch-all `-32000`.

---

## Library & Framework Requirements

> **Versions verified against `Cargo.lock` (resolved), not training data, during the
> remove-uncertainty pass (D-1110, 2026-06-24).** The repo is the source of truth; the
> caret pins in `Cargo.toml` resolve to the exact versions below.

| Library | Caret pin (`Cargo.toml`) | Resolved (`Cargo.lock`) | Usage | Source |
|---------|--------------------------|-------------------------|-------|--------|
| `chumsky` | `0.12` | **0.12.0** | PrismQL parser (grammar extension for SqlPipe, NOW(), mode-bridge) | `Cargo.lock` verified |
| `rmcp` | `1.7` (workspace, features `server`,`macros`,`transport-io`) | **1.7.0** | MCP server (`PromptRouter` + `#[prompt_handler]` dispatch) | `Cargo.lock` verified |
| `datafusion` | `53.1` | **53.1.0** (arrow 58) | Plan-time NOW() constant injection lowering | `Cargo.lock` verified |
| `strsim` | `0.11` | **0.11.1** | `did_you_mean` on E-QUERY-036 (parity with E-QUERY-037); use `strsim::levenshtein`, suggest only if distance ≤ 3 (D-1163) | `Cargo.lock` verified |
| `tokio` | `1` (workspace) | **1.52.1** | `tokio::time::timeout` for prompt timing tests (AC-015/AC-016); `mpsc::Sender::try_send`/`send` take `&self` in tokio 1.x | `Cargo.lock` verified |
| `chrono` | `0.4` (`default-features=false`, `std`,`serde`) | **0.4.44** | `DateTime<Utc>` for planning-time NOW() injection. NOTE: chrono 0.4 has **no** duration-string parser — `INTERVAL '24h'`/`'7d'`/`'1h'` literals MUST be parsed by a custom mini-parser, then mapped to `chrono::Duration::hours(n)` / `Duration::days(n)` / `Duration::minutes(n)` / `Duration::seconds(n)`. See AC-005 / Area B note. | `Cargo.lock` verified |
| `serde_json` | `1` (workspace) | (1.x) | `#[serde(skip_serializing_if = "Option::is_none")]` on `normalized_pql` | workspace |

**Forbidden dependencies:** `prism-query` MUST NOT depend on `prism-mcp`. `prism-security` MUST NOT depend on `prism-mcp`. Verify the dependency graph does not invert after Arc-DI wiring changes.

**INTERVAL-literal parsing (verified 2026-06-24):** chrono 0.4.44 provides `Duration::hours/days/minutes/seconds` constructors but NO text parser for `'24h'`-style literals. The idiomatic chumsky 0.12 approach is a `select!` on the string-literal token feeding a `.try_map(...)` that parses `<int><unit>` (units: `s`,`m`,`h`,`d`) and returns `Err` → `E-QUERY-001` for `INTERVAL 'bogus'` (AC-005). This is custom code, not a library call — implementer must not assume a chrono/serde parser exists.

---

## File Structure Requirements

| File | Action | Notes |
|------|--------|-------|
| `crates/prism-query/src/ast.rs` | Modify | Add `Ast::SqlPipe(SqlPipeQuery)`, `Expr::Now`, `Expr::Interval(Duration)`, `Expr::TimestampArithmetic` |
| `crates/prism-query/src/filter_parser.rs` | Modify | Mode detection tristate, `build_expr_parser` extension, `parse_sql_pipe` |
| `crates/prism-query/src/materialization.rs` | Modify | `run_materialization_pipeline` Step 1a (`inject_now` call) + Step 1b (`plan_sqlpipe_query` call); `execute_against_session` `Ast::SqlPipe` execution arm (shipped — wiring sites for new grammar; no net-new logic in this file beyond Step 3b bare-filter for `Ast::Filter`) |
| `crates/prism-query/src/lib.rs` | Modify | `inject_now` function (NOW() constant-folding traversal) + `plan_sqlpipe_query` function (FORBID-BOTH E-QUERY-040 check) + `parse_and_plan` (public planning API) |
| `crates/prism-query/src/error_recovery.rs` | Modify | Mode-bridge D1/D2 heuristic, enrich-position error messages |
| `crates/prism-query/src/error_recovery.rs` | Modify | Produce `normalized_pql` rewrite STRING on D1 mode-bridge (no struct named `ParseErrorDetails` exists — D-1110) |
| `crates/prism-mcp/src/error_mapping.rs` | Modify | Add `normalized_pql: Option<String>` field to `StructuredErrorFields` + update `new`/builder; populate in `QueryParseFailed` arm |
| `crates/prism-core/src/error.rs` | Modify | `PrismError::RedundantRowLimit { sql_limit: u64, pipe_limit: u64 }` variant |
| `crates/prism-mcp/src/error_mapping.rs` | Modify | Add `-32602 INVALID_PARAMS` arm for `RedundantRowLimit` (OBS-3 correction: MCP error mapping lives in prism-mcp, not prism-core) |
| `crates/prism-mcp/src/resources.rs` | Modify | `build_reference_content`, `REFERENCE_EXAMPLES`, `ExampleKind`; remove `include_str!` |
| `crates/prism-mcp/src/resources/schema.rs` | Modify | Remove `PQL_REFERENCE_CONTENT = include_str!("../pql_reference.md")` + retire/repoint `render_pql_reference_resource()` (D-1110 pass-2: the `include_str!` lives HERE, not in `resources.rs`) |
| `crates/prism-mcp/src/pql_reference.md` | Delete / retire | `include_str!` (in `resources/schema.rs`) removed; file may remain as documentation archive |
| `crates/prism-mcp/src/server.rs` | Modify | NOT_YET_AVAILABLE guard reorder BEFORE `emit_tool_audit(...).await` (D-1110: emit_tool_audit awaits AuditWriter::write_tool_call — no mpsc try_send path exists) |
| `crates/prism-mcp/src/prompts.rs` | Modify | Fix prompt hang (BLOCKER-003 root cause from `cargo expand` investigation) |
| `crates/prism-mcp/src/server.rs` (`list_capabilities` handler) | Modify | Wire `Arc<OrgRegistry>` through to `FeatureFlagEvaluator` (no `tools/list_capabilities.rs` exists — handler is in `server.rs`) |
| `crates/prism-security/src/feature_flag.rs` | Modify | `FeatureFlagEvaluator`: add `Arc<OrgRegistry>` field, change `client_exists` |
| `crates/prism-dtu-crowdstrike/src/routes/oauth.rs` | No change | BLOCKER-001 deferred to S-RESILIENCE-FEDERATED-001 (D-1326 adjudication); no DTU token endpoint change needed for T13 demo |
| `scripts/` (demo runbook §5.5) | Modify | BLOCKER-002: correct pipe syntax example |
| `crates/prism-query/tests/filter_mode.rs` | New | Filter mode end-to-end execution tests (AC-011) |
| `crates/prism-query/tests/grammar_remediation.rs` | New | Area A/B/D Red Gate tests |
| `crates/prism-mcp/tests/reference_content.rs` | New | Area C Red Gate tests |
| `crates/prism-mcp/tests/mcp_infrastructure.rs` | New | Area E Red Gate tests |
| `.ci.yml` | Modify | EXPECTED count bump if new `#[non_exhaustive]` types added |

---

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.16 | BC-2.11.020-v1.5-pin-propagation-2026-07-08 | 2026-07-08 | story-writer | **BC-2.11.020 v1.4→v1.5 version-pin propagation (sort-grammar fix round, POL-29/POL-23).** PO bumped BC-2.11.020 v1.4→v1.5 (sort-grammar micro-fix). Three live version-pin cites updated: (1) §Behavioral Contracts body table BC-2.11.020 version cell; (2) §Edge Cases table EC-002b cite; (3) §Tasks Phase 2 step 5 cite. Historical changelog rows left unchanged per POL-29. AC semantics UNCHANGED. No BC-array/AC-trace/scope change. Frontmatter version 1.15→1.16; updated 2026-07-08 (POL-23). |
| 1.15 | BC-2.11.020-v1.4-pin-propagation-2026-07-08 | 2026-07-08 | story-writer | **BC-2.11.020 v1.3→v1.4 version-pin propagation (pass-2 CRIT closure burst, POL-29/POL-23).** PO bumped BC-2.11.020 v1.3→v1.4 (14-position gate + derived-column binding rule). Three live version-pin cites updated: (1) §Behavioral Contracts body table BC-2.11.020 version cell v1.3→v1.4; (2) §Edge Cases table EC-002b cite `BC-2.11.020 v1.3 EC-11-020-008` → v1.4; (3) §Tasks Phase 2 step 5 cite `BC-2.11.020 v1.3 FAMILY rule` → v1.4. Historical changelog rows (1.14 v1.2→v1.3 propagation) left unchanged per POL-29. AC semantics UNCHANGED. No BC-array/AC-trace/scope change. Frontmatter version 1.14→1.15; updated 2026-07-08 (POL-23). |
| 1.14 | BC-2.11.020-v1.3-pin-propagation-2026-07-08 | 2026-07-08 | story-writer | **BC-2.11.020 v1.2→v1.3 version-pin propagation (POL-29/POL-23).** Product-owner keyword sweep bumped BC-2.11.020 v1.2→v1.3 (`\| project`→`\| fields` keyword). Three live version-pin cites updated: (1) §Behavioral Contracts body table BC-2.11.020 version cell v1.2→v1.3; (2) §Edge Cases table EC-002b cite `BC-2.11.020 v1.2 EC-11-020-008` → v1.3; (3) §Tasks Phase 2 step 5 cite `BC-2.11.020 v1.2 FAMILY rule` → v1.3. Historical changelog rows (1.13 v1.1→v1.2 propagation) left unchanged per POL-29. AC semantics UNCHANGED. No BC-array/AC-trace/scope change. Frontmatter version 1.13→1.14; updated 2026-07-08 (POL-23). |
| 1.13 | ADV-FIX-P1-HIGH-001-BC-2.11.020-v1.2-pin-propagation-2026-07-08 | 2026-07-08 | story-writer | **BC-2.11.020 v1.1→v1.2 version-pin propagation (ADV-FIX-P1-HIGH-001, POL-29/POL-23).** Product-owner amended BC-2.11.020 v1.1→v1.2 (sibling BC bumped in the same burst as BC-2.11.016 v1.6). Three live version-pin cites updated: (1) §Behavioral Contracts body table BC-2.11.020 version cell v1.1→v1.2; (2) §Edge Cases table EC-002b cite `BC-2.11.020 v1.1 EC-11-020-008` → v1.2; (3) §Tasks Phase 2 step 5 cite `BC-2.11.020 v1.1 FAMILY rule` → v1.2. Historical changelog rows (1.12 POL-14 promotion, 1.9 v1.0→v1.1 propagation) left unchanged per POL-29. AC semantics UNCHANGED. No BC-array/AC-trace/scope change. Frontmatter version 1.12→1.13; updated 2026-07-08 (POL-23). |
| 1.12 | PR-203-post-merge-POL-14 | 2026-06-26 | state-manager | **PR #203 MERGED (POL-13 status flip + POL-14 BC auto-promotion).** Squash-merged to develop@7e60df03 (2026-06-26; CI 43/43 green; 9-round PR-LEVEL cascade; 3-CLEAN(strict) on frozen HEAD 356e0573; security CLEAN). `status: draft → merged`. All 8 BCs promoted draft→active: BC-2.11.020 v1.1, BC-2.11.021 v1.0, BC-2.11.022 v1.0, BC-2.11.023 v1.2, BC-2.11.002 v1.4, BC-2.10.015 v1.2, BC-2.10.016 v1.1, BC-2.10.017 v1.1. develop_head 903c8fcb→7e60df03. non-exhaustive 87 (ExampleKind, SqlPipeQuery, UnknownSourceTableDetails added in this PR). |
| 1.11 | bc-pin-propagation-F-P2R2-HIGH-001-F-P2R2-LOW-001 | 2026-06-26 | story-writer | BC version-pin propagation (POL-8 bc_array_changes_propagate). F-P2R2-HIGH-001: BC-2.10.017 Behavioral Contracts table pin `v1.0 → v1.1` (product-owner bumped v1.0→v1.1: try_send/mpsc framing replaced by guard-reorder reality per D-1110). F-P2R2-LOW-001: BC-2.10.015 Behavioral Contracts table pin `v1.1 → v1.2` (product-owner bumped v1.1→v1.2: phantom `tools/list_capabilities.rs` anchor replaced by `server.rs::list_capabilities`). AC-017/AC-018/EC-009 prose verified unchanged (D-1110 guard-reorder/no-structured-error reality already correct — adversary confirmed). Frontmatter `behavioral_contracts:` array carries no version suffixes; no frontmatter array change required. No AC/scope/code/BC-trace change. |
| 1.10 | f-p2r2-med-001-ac016-ec008-bc-traceability-sync | 2026-06-26 | story-writer | F-P2R2-MED-001 closure: AC↔BC traceability desync. BC-2.10.016 v1.1 RETRACTED the "structured MCP error" behavior for the missing-required-arg case (INV-PROMPT-REQUIRED-ARGS option (a)); story prose was stale. (1) AC-016 prose updated: missing required arg → substitutes literal `(unknown)`, returns **Ok** within 5 seconds — no structured MCP error; the no-hang fast-return guarantee is the contract. (2) AC-016 Red Gate test description updated to match shipped test (`result.is_ok()`, not "assert structured MCP error"). (3) EC-008 expected-behavior updated to "substitutes `(unknown)`, returns Ok within 5s; MUST NOT hang (no structured MCP error)". No code change (shipped test `test_BC_2_10_016_missing_required_arg_fast_error` already asserts `result.is_ok()` — correct). No AC count / Red Gate count / BC-trace change. |
| 1.9 | bc-2.11.020-v1.1-propagation-F-P2-HIGH-001-F-P2-MED-001 | 2026-06-26 | story-writer | BC-2.11.020 v1.0→v1.1 propagation (F-P2-HIGH-001 / POL-8 bc_array_changes_propagate_to_body_and_acs): (1) Behavioral Contracts table pin `v1.0 → v1.1` (POL-27 version-pin-sync); (2) AC-002 condition updated: "AND a row-capping `\| limit M` or `\| tail M` pipe stage" to match BC-2.11.020 v1.1 FAMILY rule / error-taxonomy E-QUERY-040 wording; (3) Phase 2 step 5 updated to reflect `PipeStage::Limit(_) \| PipeStage::Tail(_)` two-arm check per shipped code; (4) EC-002 description clarified + EC-002b sibling row added for `\| tail 3` form (BC-2.11.020 v1.1 EC-11-020-008). F-P2-MED-001 closure: §Changelog rows reordered to strict monotonic descending (1.9→1.0); no rows deleted; no duplicates. |
| 1.8 | pol-23-bc-2.10.016-version-pin-sync | 2026-06-25 | story-writer | POL-23 sibling-sweep. BC-2.10.016 version pin updated `v1.0 → v1.1` in Behavioral Contracts table to match current BC frontmatter (product-owner bumped BC-2.10.016 v1.0→v1.1 for Error-Cases/EC reconciliation). POL-25 sweep confirmed no other live-narrative pins of BC-2.10.016 at v1.0 exist outside §Changelog historical rows (TD-VSDD-091-exempt). Frontmatter `behavioral_contracts:` array carries no version suffixes; no frontmatter change required. No AC/scope/code/BC-trace change. |
| 1.7 | obs-3-stale-path-closure | 2026-06-25 | story-writer | OBS-3 closure (LOW, process-gap). Corrected three stale `tools/list_capabilities.rs` path hints that the v1.6 TD-VSDD-091 sweep missed. Actual handler `list_capabilities` (BC-2.10.015 Arc-DI wiring) lives in `crates/prism-mcp/src/server.rs`; no `tools/list_capabilities.rs` exists (tools/ contains `config.rs`, `mod.rs`, `operations.rs`, `prism_describe.rs`, `query.rs`, `sensor_health.rs`, `write.rs` — verified against worktree and main codebase). Changes: (1) `crates_touched` frontmatter prism-mcp comment — replaced `tools/ (list_capabilities wiring)` with `list_capabilities handler wiring also in server.rs — no tools/list_capabilities.rs exists`; (2) Architecture Mapping row — changed `crates/prism-mcp/src/tools/list_capabilities.rs` to `crates/prism-mcp/src/server.rs` with function-name anchor per TD-VSDD-091; (3) File Structure row — same correction plus clarifying note. No AC count / BC trace / scope change. |
| 1.6 | td-vsdd-091-location-pin-remediation | 2026-06-25 | story-writer | TD-VSDD-091 anti-volatile-pin compliance. Replaced five stale `engine.rs` file-location hints that described FORBID-BOTH and NOW()-injection wiring. Verified against shipped code in worktree: `inject_now` lives in `lib.rs` (called from `run_materialization_pipeline` Step 1a in `materialization.rs`); `plan_sqlpipe_query` lives in `lib.rs` (called from Step 1b); `execute_against_session` `Ast::SqlPipe` arm lives in `materialization.rs`. Changed references: (1) `crates_touched` frontmatter comment — replaced `engine.rs (SqlPipe execution arm, plan-time NOW injection, FORBID-BOTH check)` with function-name anchors for `inject_now` / `plan_sqlpipe_query` / `run_materialization_pipeline` Step 1a/1b / `execute_against_session`; (2) Architecture Mapping rows — replaced three `engine.rs` rows with `materialization.rs::execute_against_session`, `lib.rs::inject_now`, `lib.rs::plan_sqlpipe_query`; (3) File Structure — replaced one `engine.rs` row with two rows for `materialization.rs` and `lib.rs`. Preserved: `engine.rs::normalize_pql` reference at AC-010 and Previous Story Intelligence (accurate — that function IS in engine.rs). No AC count / BC trace / scope change. |
| 1.5 | bc-2.11.023-d1-d2-coverage-gap-closure | 2026-06-25 | story-writer | HIGH-1 + HIGH-2 adversary gap closure. HIGH-1: AC-009 (mode-bridge D1) tightened — now asserts all three verbatim BC-2.11.023 §D1 substrings: (a) stage-keyword enumeration `(enrich, where, limit, sort, stats, dedup, fields)`, (b) numbered alternatives `1. SQL+pipe composition: …` / `2. Pipe mode only: …`, (c) reference pointer `See prismql://reference for the complete grammar.`; negative control (no raw Chumsky token list) added. HIGH-2: AC-027 added (new) — D2 mode-bridge diagnostic: pipe-mode query with uppercase SQL clause keyword in stage position (e.g. `FROM t \| WHERE …` or `\| ORDER BY …`) produces verbatim BC-2.11.023 §D2 message; positive and negative controls specified; Red Gate test `test_BC_2_11_023_mode_bridge_d2_sql_keyword_in_pipe_position` added. Bookkeeping: `acceptance_criteria_count` 26→27, `red_gate_tests` 19→20, BC-2.11.023 frontmatter comment updated with AC-027, Tasks §Phase 1 Area D list extended, Phase 5 steps updated to include D2 heuristic, Token Budget table updated (19→20 Red Gate tests), Area F header note updated (19→20). Changelog row v1.0 historical count unchanged (records original state). |
| 1.4 | version-pin-sync-MED-1 | 2026-06-25 | story-writer | Version-pin sync (MED-1 / POL-23 sibling-sweep gap closure). Behavioral Contracts table: BC-2.11.023 pin updated `v1.1 → v1.2` and BC-2.10.015 pin updated `v1.0 → v1.1` to match current BC frontmatter versions. POL-25 sweep confirmed no other live-narrative pins of these BCs at the old versions exist outside §Changelog historical rows (TD-VSDD-091-exempt). No AC/scope/code change. |
| 1.3 | story-writer-spec-sync-D1326 | 2026-06-24 | story-writer | Spec-sync burst (D-1326 adjudication). 4 changes: (1) **AC-019 re-scoped** — BLOCKER-001 root cause adjudicated as connectivity connect-timeout (PluginKvStore is in-memory/fresh per `prism start`, making cross-session KV staleness impossible; dead `reset_token_cache` + test removed at code HEAD 3fa69207); runtime fix deferred to S-RESILIENCE-FEDERATED-001 (per-sensor TOML timeouts, boot-degraded, retry-with-backoff); demo unblocked via runbook DTU health-check Fix B; wrong BC citation corrected (BC-2.06.001 "TOML Config Loading" → BC-2.01.005 "CrowdStrike OAuth2 Authentication and Two-Step Fetch"). (2) **OBS-3** — File Structure `-32602 INVALID_PARAMS` arm path corrected from `crates/prism-core/src/error_mapping.rs` to `crates/prism-mcp/src/error_mapping.rs` (MCP error mapping lives in prism-mcp; confirmed by `map_prism_error` function location). (3) **BC-2.11.023 version** — v1.0 → v1.1 in Behavioral Contracts table (PO bumped in same burst). (4) **S-RESILIENCE-FEDERATED-001 stub** registered (day-2 resilience epic anchor). No AC count / BC list / Red Gate test count change. |
| 1.2 | remove-uncertainty-pass2-D1110 | 2026-06-24 | research-agent | `dclaude:remove-uncertainty` pass 2 (D-1110, pre-TDD-delivery) — re-validated against the post-S-5.04 develop tip (develop@903c8fcb, contains merged S-5.04). **Pass-1 corrections RE-CONFIRMED on 903c8fcb:** (1) `StructuredErrorFields` in `prism-mcp/src/error_mapping.rs` — `#[non_exhaustive]`, carries `near_text`/`reference_pointer`/`available_columns`/`did_you_mean` via `skip_serializing_if`; no `ParseErrorDetails` exists. (2) `OrgRegistry::slug_exists(&self, slug: &OrgSlug) -> bool` (org_registry.rs) — no `contains`. (3) `emit_tool_audit` (server.rs) is `async fn` calling `writer.write_tool_call(...).await` on `Arc<dyn AuditWriter>` returning `Result<Option<String>, ErrorData>` — NO mpsc/try_send; not-yet-available stubs (`create_schedule`, `list_schedules`, etc.) call `scan_inputs_audited().await? → emit_tool_audit().await? → Err(not_yet_available_msg(...))`; guard-reorder-before-audit-await fix is correct. (4) DTU `routes/oauth.rs` `pub async fn token` is `client_credentials`-only / static `"dtu-fake-cs-token"` / 401 only on `auth_mode=="reject"`; plugin caches `token`+`expires_at_secs` in KV. **NEW S-5.04 conflict check:** NONE — S-5.04 added `check_sensor_health` as a `LIVE_TOOLS` handler (line in `LIVE_TOOLS` const, not `NOT_YET_AVAILABLE_TOOLS`), and left `emit_tool_audit` + the not-yet-available handler structure unchanged; Area E guard-reorder scope does not intersect S-5.04 code. EXPECTED=84 (S-5.04 HealthSummary) already correctly reflected in Previous Story Intelligence + Phase 8. **2 NEW corrections applied (mechanical/code-grounded, no architect adjudication):** (A) Area C location — the `prismql://reference` `include_str!` is `PQL_REFERENCE_CONTENT = include_str!("../pql_reference.md")` in `crates/prism-mcp/src/resources/schema.rs` (served by `render_pql_reference_resource()`), NOT in `resources.rs` as v1.1 said; `resources.rs` only dispatches to `schema::render_pql_reference_resource()` from the `read_resource` `prismql://reference` arm. Corrected AC-006, Phase 4 step 4, Architecture Mapping, File Structure (added `resources/schema.rs` row), crates_touched. The `pql_reference.md` path itself was already correct. (B) BLOCKER-001 / AC-019 — the plugin has NO dedicated "FORCED-REFRESH entrypoint"; `acquire_token(host, token_endpoint)` is the unconditional fresh-acquire fn, and `get_token` re-acquires only on cache-miss/stale (`now >= expires_at_secs` or empty cached token). To force a refresh the plugin path calls `acquire_token` directly or evicts the KV `token`/`expires_at_secs` keys. Corrected AC-019 plugin-internals + fix-path bullets. No AC count / BC trace change. |
| 1.1 | remove-uncertainty-pass-D1110 | 2026-06-24 | research-agent | `dclaude:remove-uncertainty` pass 1 (D-1110). Validated all version-sensitive tech assumptions against `Cargo.lock` (source of truth) + CURRENT (2026) authoritative docs (Perplexity sonar-deep-research + Context7-grade source verification). Corrections applied (no AC count / BC trace change): (1) **Pinned exact resolved versions** — rmcp 1.7.0, chumsky 0.12.0, datafusion 53.1.0 (arrow 58), strsim 0.11.1, tokio 1.52.1, chrono 0.4.44. (2) **`ParseErrorDetails` does not exist** — the `normalized_pql` field belongs on the existing `#[non_exhaustive]` `StructuredErrorFields` in `prism-mcp/src/error_mapping.rs`; rewrite STRING produced in `prism-query` error_recovery; corrected AC-010, Architecture Mapping, File Structure, crates_touched, Phase 5, Phase 8 EXPECTED note, Previous Story Intelligence. (3) **OrgRegistry API** — method is `slug_exists(&OrgSlug)` not `contains(&str)`; corrected AC-013 + Phase 6 + crates_touched; no new OrgRegistry method needed; `OrgSlug::new_unchecked` forbidden. (4) **BLOCKER-004 mechanism** — `emit_tool_audit` awaits `AuditWriter::write_tool_call`, it is NOT an mpsc `try_send` path; the fix is guard-reorder-before-audit-await, not try_send; corrected AC-017/AC-018 (test renamed `..._guard_precedes_audit`), EC-009, points note, Architecture Mapping, File Structure. (5) **BLOCKER-001 path** — DTU route is `routes/oauth.rs` (`fn token`, client_credentials-only, static token) not `routes/oauth2.rs`; plugin-side full-reauth is lower-risk default; corrected AC-019, Phase 7, File Structure, crates_touched. (6) **DataFusion 53 plan-time injection confirmed sound** — PrismQL substitutes its own `Expr::Now` before the plan reaches DataFusion, avoiding any collision with DataFusion's built-in `now()`; added soundness note to Phase 3. (7) **chrono 0.4 has no duration-string parser** — `INTERVAL` literals require custom `<int><unit>` parsing via chumsky `try_map`; added note to Library table + Phase 3. (8) **rmcp PromptRouter required-arg validation** — design-expected but NOT explicitly documented in rmcp 1.7 docs; reaffirmed `cargo expand` investigation mandate (ADR-046 D6) before any fix; routes use `new_dyn` closures with `.unwrap_or` so closures don't block — hang is in macro-dispatch/transport layer. No item required architect adjudication (all corrections mechanical/code-grounded per production-grade default). |
| 1.0 | demo-readiness-remediation-2026-06-24 | 2026-06-24 | story-writer | Initial story. 26 ACs (19 BC-traced + 7 implementer/doc). 8 BCs. Explicit human directive: single consolidated story for all T13 demo-readiness findings. Size flag included per story-writer mandate (111k token estimate). |
