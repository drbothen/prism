# Evidence Report — S-DEMO-PRISMQL-GRAMMAR-REMEDIATION-001

**Story:** PrismQL Grammar + MCP-Surface Remediation  
**Branch:** `feature/S-DEMO-PRISMQL-GRAMMAR-REMEDIATION-001`  
**HEAD at recording:** `71b196ad`  
**Recording date:** 2026-06-25  
**Tool:** VHS 0.11.0 (via tmux PTY) — all tapes in this directory  

---

## Coverage Summary

| Area | ACs Covered | Recording Mechanism |
|------|-------------|---------------------|
| A — SQL-to-Pipe Grammar (BC-2.11.020) | AC-001, AC-002, AC-003 | VHS: AC-001-003-sqlpipe-grammar |
| A — FORBID-BOTH message detail (BC-2.11.020) | AC-002 (message text) | VHS: AC-002-forbid-both-message |
| B — Temporal Grammar (BC-2.11.021) | AC-004, AC-005 | VHS: AC-004-005-temporal-grammar |
| C — Reference Content (BC-2.11.022) | AC-006, AC-007, AC-008, AC-023, AC-026 | VHS: AC-006-008-reference-content |
| D — Mode-Bridge D1+D2 (BC-2.11.023) | AC-009, AC-027 | VHS: AC-009-027-mode-bridge-d1-d2 |
| D — normalized_pql (BC-2.11.023) | AC-010 | VHS: AC-010-normalized-pql |
| D — Filter mode + D7 grammar | AC-011, AC-012 | VHS: AC-011-012-filter-mode-d7 |
| E — OrgRegistry Arc-DI (BC-2.10.015) | AC-013, AC-014 | VHS: AC-013-014-list-capabilities |
| E — Prompts fast-return (BC-2.10.016) | AC-015, AC-016 | VHS: AC-015-016-prompts-fast-return |
| E — NOT_YET_AVAILABLE fast-fail (BC-2.10.017) | AC-017, AC-018 | VHS: AC-017-018-not-yet-available |
| F — Enrich guided errors (GRAMMAR-005/015) | AC-022, AC-025 | VHS: AC-022-025-enrich-guidance |
| F — Area F implementation ACs | AC-019, AC-020, AC-021, AC-024 | Test citation (no live sensor required) |

**Total ACs covered:** 27/27 (23 via VHS recording, 4 via test citation per Area F)

---

## Recording Artifacts

### AC-001 / AC-002 / AC-003 — SQL-to-Pipe Grammar (BC-2.11.020)

- `AC-001-003-sqlpipe-grammar.gif` (114 KB) / `AC-001-003-sqlpipe-grammar.webm` (168 KB)
- `AC-001-003-sqlpipe-grammar.tape` — tape source

**Demonstrates:**
- AC-001: `SELECT ... | enrich ... | limit N` parses to `Ast::SqlPipe` with 2 stages  
- AC-002: `SELECT ... LIMIT 5 | ... | limit 3` → `E-QUERY-040 FORBID-BOTH` at plan-time  
- AC-003: Pure SQL (`SELECT` without `|`) and pure Pipe (`FROM t | ...`) modes unchanged  

**Red Gate tests:** `test_bc_2_11_020_sqlpipe_ast_round_trip`, `test_bc_2_11_020_forbid_both_dual_limit_e_query_040`, `test_bc_2_11_020_pure_modes_unchanged`

---

### AC-002 (message detail) — FORBID-BOTH E-QUERY-040 text

- `AC-002-forbid-both-message.gif` (73 KB) / `AC-002-forbid-both-message.webm` (70 KB)
- `AC-002-forbid-both-message.tape`

**Demonstrates:** error message contains `E-QUERY-040`, both limit counts (`5` and `3`) interpolated, verbatim "PrismQL requires exactly one row cap" guidance.

**Red Gate tests:** `test_bc_2_11_020_forbid_both_dual_limit_e_query_040`

---

### AC-004 / AC-005 — Temporal Grammar: NOW() and INTERVAL (BC-2.11.021)

- `AC-004-005-temporal-grammar.gif` (93 KB) / `AC-004-005-temporal-grammar.webm` (137 KB)
- `AC-004-005-temporal-grammar.tape`

**Demonstrates:**
- AC-004: `NOW() - INTERVAL '24h'` parses in SQL, Pipe, and Filter modes  
- AC-005: `NOW(1)`, `NOW() + INTERVAL`, `INTERVAL 'bogus'` each return `E-QUERY-001`  

**Red Gate tests:** `test_bc_2_11_021_now_interval_parses_all_three_modes`, `test_bc_2_11_021_now_error_cases`

---

### AC-006 / AC-007 / AC-008 / AC-023 / AC-026 — prismql://reference content (BC-2.11.022)

- `AC-006-008-reference-content.gif` (193 KB) / `AC-006-008-reference-content.webm` (413 KB)
- `AC-006-008-reference-content.tape`

**Demonstrates:**
- AC-006: `build_reference_content()` produces all 13 required sections  
- AC-007: CI 3-tier gate (positive / negative / registry parity) passes  
- AC-008: `build_reference_content(None)` returns placeholder text without panicking  
- AC-023: Reference includes `IS NOT NULL` on JSON-list column semantics  
- AC-026: Reference aggregates section documents `percentile`, `distinct_count`  

**Red Gate tests:** entire `prism-mcp::reference_content` test suite (7 tests)

---

### AC-009 / AC-027 — Mode-Bridge D1 and D2 Diagnostics (BC-2.11.023)

- `AC-009-027-mode-bridge-d1-d2.gif` (108 KB) / `AC-009-027-mode-bridge-d1-d2.webm` (165 KB)
- `AC-009-027-mode-bridge-d1-d2.tape`

**Demonstrates:**
- AC-009: D1 — `SELECT * FROM t | invalid_stage` produces mode-bridge message with all 3 required BC-2.11.023 §D1 substrings: (a) stage-keyword enumeration, (b) numbered alternatives, (c) `prismql://reference` pointer  
- AC-027: D2 — `FROM t | ORDER BY` produces "SQL clauses not valid as pipe stages" message  

**Red Gate tests:** `test_bc_2_11_023_mode_bridge_d1_sql_pipe_diagnostic`, `test_bc_2_11_023_mode_bridge_d2_sql_keyword_in_pipe_position`

---

### AC-010 — normalized_pql on StructuredErrorFields

- `AC-010-normalized-pql.gif` (77 KB) / `AC-010-normalized-pql.webm` (144 KB)
- `AC-010-normalized-pql.tape`

**Demonstrates:** D1 mode-bridge error on `SELECT * FROM t WHERE severity = 'HIGH' | limit 10` populates `normalized_pql = Some("FROM t | where severity = 'HIGH' | limit 10")` in MCP structured error envelope.

**Red Gate tests:** `test_bc_2_11_023_normalized_pql_on_mode_bridge_error` (prism-mcp::mcp_infrastructure)

---

### AC-011 / AC-012 — Filter-mode + D7 Shared Predicate Grammar

- `AC-011-012-filter-mode-d7.gif` (90 KB) / `AC-011-012-filter-mode-d7.webm` (135 KB)
- `AC-011-012-filter-mode-d7.tape`

**Demonstrates:**
- AC-011: Filter mode — bare predicate `severity = 'HIGH'` and source-qualified `crowdstrike.severity = 'HIGH'` both produce `Ast::Filter`  
- AC-012: D7 — single `build_predicate_parser()` function used across SQL WHERE, Pipe `| where`, and Filter mode  

**Red Gate tests:** `test_bc_2_11_023_filter_mode_end_to_end_execution`, `test_bc_2_11_023_d7_shared_predicate_grammar`

---

### AC-013 / AC-014 — list_capabilities consults OrgRegistry (BC-2.10.015)

- `AC-013-014-list-capabilities.gif` (98 KB) / `AC-013-014-list-capabilities.webm` (193 KB)
- `AC-013-014-list-capabilities.tape`

**Demonstrates:**
- AC-013: `FeatureFlagEvaluator` consults `OrgRegistry::slug_exists()` — org-c registered → `client_registered: true`  
- AC-014: Non-existent org → `client_registered: false`  

**Red Gate tests:** `test_bc_2_10_015_client_registered_true_from_org_registry`, `test_bc_2_10_015_demo_provisioned_org_registered`

---

### AC-015 / AC-016 — MCP Prompts Fast-Return (BC-2.10.016)

- `AC-015-016-prompts-fast-return.gif` (145 KB) / `AC-015-016-prompts-fast-return.webm` (295 KB)
- `AC-015-016-prompts-fast-return.tape`

**Demonstrates:**
- AC-015: All `prompts/get` calls return within 5 seconds (BLOCKER-003 hang fixed)  
- AC-016: Missing required argument `investigate_host.hostname` → structured error within 5s, no hang  

**Red Gate tests:** `test_bc_2_10_016_prompts_fast_return_within_5s`, `test_bc_2_10_016_missing_required_arg_fast_error`, `test_bc_2_10_016_get_prompt_full_transport_dispatch`, `test_high1_investigate_host_full_transport_dispatch`, `test_high1_missing_required_arg_via_full_transport_no_hang`

---

### AC-017 / AC-018 — NOT_YET_AVAILABLE Fast-Fail (BC-2.10.017)

- `AC-017-018-not-yet-available.gif` (125 KB) / `AC-017-018-not-yet-available.webm` (251 KB)
- `AC-017-018-not-yet-available.tape`

**Demonstrates:**
- AC-017: `list_infusions`, `plugin_status`, `infusion_status` return `-32003` within 1s even with a slow `AuditWriter` injected  
- AC-018: Guard fires BEFORE `emit_tool_audit().await` — no NOT_YET_AVAILABLE path blocks on audit I/O  

**Red Gate tests:** `test_bc_2_10_017_not_yet_available_fast_fail_under_1s`, `test_bc_2_10_017_not_yet_available_guard_precedes_audit`, `test_bc_2_10_017_sibling_handlers_guard_precedes_audit`

---

### AC-022 / AC-025 — Enrich Parse Error Guidance (GRAMMAR-005/015)

- `AC-022-025-enrich-guidance.gif` (131 KB) / `AC-022-025-enrich-guidance.webm` (200 KB)
- `AC-022-025-enrich-guidance.tape`

**Demonstrates:**
- AC-022: `FROM t | enrich threat_score` → guided error "enrich requires a column argument: `| enrich <infusion>(<column>)`" (not raw Chumsky token dump)  
- AC-025: Multi-stage `FROM t | where severity = 'HIGH' | enrich threat_score` → same guided error (heuristic fires at all pipeline positions)  

**Red Gate tests:** `test_bc_2_11_grammar005_enrich_missing_column_arg_guidance`, `test_bc_2_11_grammar015_enrich_missing_column_arg_multi_stage_guidance`, `test_bc_2_11_obs1_sqlpipe_enrich_missing_column_arg_guided_error`

---

## Area F — Test Citations (No Live Sensor Required)

These ACs are implementer/doc ACs verified by inspection or existing test assertions, not Red Gate tests. Demonstrations via live terminal are not applicable.

### AC-019 — CrowdStrike OAuth no session-start hang (BLOCKER-001)

**D-1326 Adjudication:** Root cause is connectivity `connect_timeout` (no separate from `PLUGIN_HTTP_CLIENT_TIMEOUT_SECS=30`), not KV staleness. `PluginKvStore` is in-memory and fresh per `prism start` — cross-session staleness is impossible. The runtime fix is deferred to `S-RESILIENCE-FEDERATED-001` (per-sensor TOML timeouts, boot-degraded mode). Demo unblocked via runbook DTU health-check (Fix B). No code change required in this story.

**Load-bearing test:** None required — AC-019 is a structural/architectural verification. The dead `reset_token_cache` function and its test (found at HEAD `3fa69207`) confirmed PluginKvStore is in-memory. See `crates/prism-spec-engine/plugins/crowdstrike-oauth2/src/lib.rs`.

### AC-020 — Demo runbook §5.5 pipe syntax is valid PrismQL (BLOCKER-002)

**Verification:** The runbook `FROM t | where <predicate> | enrich fn(col) | limit N` form is valid pipe-mode PrismQL. Verified by `test_bc_2_11_020_pure_modes_unchanged` (confirms pure pipe mode unchanged) and `test_bc_2_11_023_filter_mode_end_to_end_execution` (confirms pipe-mode predicate execution). The old invalid syntax `FROM t WHERE … LIMIT N` would fail `PrismQlParser::parse` — the corrected form parses to `Ast::Pipe`.

**Load-bearing test:** `test_bc_2_11_020_pure_modes_unchanged` in `crates/prism-query/tests/grammar_remediation.rs`

### AC-021 — E-QUERY-036 carries available_tables + did_you_mean (GRAMMAR-004)

**Verification:** `UnknownSourceTableDetails` gains `available_tables: Vec<String>` and `did_you_mean: Option<String>`, matching the E-QUERY-037/038/039 parity requirement. Levenshtein ≤ 3 best-candidate algorithm consistent with `TableNotAvailableDetails`.

**Load-bearing tests:**
- `test_BC_2_11_007_grammar004_unknown_source_table_carries_available_tables_and_did_you_mean` in `crates/prism-query/src/materialization.rs`
- `test_unknown_source_table_display_with_did_you_mean` in `crates/prism-core/src/error.rs`
- `test_unknown_source_table_maps_to_invalid_params` in `crates/prism-mcp/src/error_mapping.rs`

### AC-024 — GRAMMAR-013 checklist: all 10 items discoverable from teaching surfaces alone

**Verification:** All 10 "must know out-of-band" items from GRAMMAR-013 are now discoverable via `prism_describe` output, `prismql://reference`, and error messages. Coverage:

| GRAMMAR-013 Item | Teaching Surface |
|-----------------|-----------------|
| Infusion names | `prism_describe` → `list_infusions` → enrichment paragraph (plain text following `## Error Code Quick-Reference` in `prismql://reference`) |
| Column argument form `fn(col)` | Error message (AC-022/025 enrich guidance) + enrichment paragraph: `\| enrich <fn>(<col>)` in `## Clause Grammar (BNF)` |
| Pipe stage keywords | D1 mode-bridge message (AC-009) + `## Clause Grammar (BNF)` (Pipe Mode BNF block) |
| SQL+pipe composition syntax | `## Clause Grammar (BNF)` (SqlPipe Mode block) + error D1 numbered alternatives |
| Pipe-mode `FROM t \| where` form | `## Clause Grammar (BNF)` (Pipe Mode BNF block) + D2 mode-bridge (AC-027) |
| Filter mode syntax | `## What is PrismQL` (mode summary table, Filter row) |
| INTERVAL literal form | `## Datetime Arithmetic` + E-QUERY-001 error (AC-005) |
| NOW() semantics | `## Datetime Arithmetic` |
| IS NOT NULL on JSON lists | `## Operators and Types` (null semantics paragraph) (AC-023) |
| Aggregate syntax `stats agg [by field]` | `## Operators and Types` (aggregate functions list) — `percentile`, `distinct_count` (AC-026) |

---

## Verification Run — All Red Gate Tests Pass

```
cargo nextest run -p prism-query --test grammar_remediation  → 13 tests: 13 passed
cargo nextest run -p prism-mcp --test reference_content      →  7 tests:  7 passed
cargo nextest run -p prism-mcp --test mcp_infrastructure     → 11 tests: 11 passed
cargo nextest run -p prism-mcp --test normalized_pql         → 20 tests: 20 passed
```

Total: 51 tests, 51 passed, 0 failed.
