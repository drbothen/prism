---
document_type: architecture-scoping
title: "Demo-Readiness Remediation Design — T13 Capstone Demo (2026-06-24)"
producer: architect
date: "2026-06-24"
version: "1.1"
modified: "2026-06-26"
status: approved
traces_to: ARCH-INDEX.md
adrs_produced: [ADR-043, ADR-044, ADR-045, ADR-046]
input_audits:
  - .factory/research/prismql-grammar-usability-audit-2026-06-24.md
  - .factory/research/demo-pre-flight-audit-2026-06-24.md
upstream_story: S-DEMO-PRISMQL-ONBOARDING-001-C
subsystems: [SS-10, SS-11]
---

# Demo-Readiness Remediation Design

> **Editorial correction — v1.1 (2026-06-26):** Five citations of `ParseErrorDetails`
> in this document (GRAMMAR-001, GRAMMAR-014, GRAMMAR-016 rows and the ADRs Produced
> summary) have been corrected to `StructuredErrorFields` / `StructuredErrorFields.normalized_pql`.
> Per D-1110: there is no `ParseErrorDetails` type in the codebase. The `normalized_pql`
> field lives on `prism_mcp::error_mapping::StructuredErrorFields`. Source: F-P2-LOW-001
> POL-25 sibling sweep; verified against worktree source at
> `crates/prism-mcp/src/error_mapping.rs`.

## Purpose

This document maps ALL 24 findings from the grammar usability audit (19 GRAMMAR-NNN)
and pre-flight audit (5 BLOCKER/MAJOR findings) to their architectural fix bucket,
affected components, contract surface required, root-cause hypothesis, ownership
(architect / product-owner / implementer), and disposition relative to
S-DEMO-PRISMQL-ONBOARDING-001-C.

This is the contract-layer handoff from architect to product-owner (BC authoring) and
subsequently to story-writer (story decomposition). All BCs and stories flow downstream
from this document.

---

## Scope Boundary: 001-C Coverage vs New Work

S-DEMO-PRISMQL-ONBOARDING-001-C is in-flight. Its BC layer was finalized at D-1308
(2026-06-23) and covers:

| Covered by 001-C | Finding it closes |
|-----------------|------------------|
| BC-2.10.012 v1.4 `from_name` / `qualified_name` field in TableDescriptor | GRAMMAR-010 / AUDIT-001 |
| BC-2.11.019 prompt bodies regenerated from real table registry | AUDIT-004 |
| BC-2.10.014 v1.2 `available_infusions` in prism_describe | GRAMMAR-012 |
| BC-2.11.001 v1.13 `clients` parameter name documentation | GRAMMAR-003 (partial) |

Findings NOT covered by 001-C and requiring NEW stories / BCs are flagged `[NEW-STORY]`
in the table below. Findings covered by 001-C are flagged `[001-C]` and need no
additional story.

---

## Remediation Design Map

### BLOCKER Findings (6 BLOCKER across both audits)

| Finding | Severity | Root-Cause Hypothesis | Fix Bucket | ADR | Affected Crate(s) | BC/Contract Surface Needed | Expected Post-Fix Behavior | Ownership | 001-C? |
|---------|----------|-----------------------|------------|-----|-------------------|---------------------------|---------------------------|-----------|--------|
| **GRAMMAR-001** Mode disjunction: SQL mode has no `\|`; pipe mode has no SQL clauses | BLOCKER | Two entirely separate Chumsky grammars chosen by first-token heuristic; no diagnostic at mode boundary | grammar/code + error-messages | ADR-043 (SQL→Pipe composition) + ADR-046 (mode-bridge error) | prism-query | BC for `Ast::SqlPipe` variant + E-QUERY-001 `normalized_pql` in `StructuredErrorFields` (D-1110) | `SELECT … FROM t \| limit 5` parses as `SqlPipe`; or when it fails, the error says "use `FROM t \| limit 5` instead" with `normalized_pql` rewrite | architect (done: ADR-043, ADR-046) → product-owner (BC) → implementer | [NEW-STORY] |
| **GRAMMAR-008** Reference has zero enrichment content | BLOCKER | `pql_reference.md` predates the enrichment pivot and was never updated | docs | ADR-045 (auto-generated reference) | prism-mcp | BC-2.11.019 (reference content) amended by ADR-045 | `prismql://reference` includes pipe mode grammar, `enrich fn(col)` syntax, available infusion pointer, and all pipe stage keywords | architect (done: ADR-045) → product-owner (BC) → implementer | [001-C] (partial — DISCOVERABILITY-GAP-001) / new work for grammar completeness |
| **GRAMMAR-009** Reference BNF and multi-stage example are actively wrong (teach broken syntax) | BLOCKER | `pql_reference.md` BNF documents aspirational SQL-clauses-after-pipe model; the real grammar is two disjoint parsers | docs | ADR-045 (auto-generated reference) | prism-mcp | ADR-045 D1: `build_reference_content()` generates BNF from live grammar constants | `prismql://reference` BNF matches `pipe_parser.rs` stage list; `SELECT * FROM t \| WHERE …` example replaced with correct pipe mode example | architect (done: ADR-045) → implementer | [NEW-STORY] |
| **GRAMMAR-010** `prism_describe` reports `name:"alerts"` but FROM requires `cyberint_alerts` | BLOCKER | `TableDescriptor.name` surfaces the short unqualified table name; query engine keys on `{sensor}_{table}` underscore form | code | n/a (existing BC-2.10.012 v1.4) | prism-mcp | BC-2.10.012 v1.4 — adds `from_name`/`qualified_name` field | `prism_describe` returns `from_name: "cyberint_alerts"` as the primary query-ready token | product-owner (done: BC-2.10.012 v1.4) → implementer | [001-C] |
| **GRAMMAR-011** `build_example_query` autogenerates `NOW() - INTERVAL` — invalid syntax | BLOCKER | `NOW()` and `INTERVAL` were documented and generated but never implemented in grammar | grammar/code | ADR-044 (temporal grammar) | prism-query, prism-mcp | New BCs: `Expr::Now`, `Expr::Interval`, `TimestampArithmetic` postconditions | `SELECT COUNT(*) FROM t WHERE timestamp > NOW() - INTERVAL '1h'` parses as valid `Ast::Sql` | architect (done: ADR-044) → product-owner (new BC) → implementer | [NEW-STORY] |
| **GRAMMAR-014** Mode-mixing parse errors dump raw Chumsky expectation list instead of naming fix | BLOCKER | Default Chumsky `Rich` error formatting surfaced verbatim; no mode-aware diagnostic layer in `error_recovery.rs` | error-messages | ADR-046 (three-mode correctness D1) | prism-query | E-QUERY-001 `StructuredErrorFields.normalized_pql: Option<String>` field added (D-1110); product-owner updates error-taxonomy E-QUERY-001 row | When `SELECT … FROM t \| …` fails in SQL mode at `\|`, error includes: "pipe stages not valid in SQL mode; use `FROM t \| …`" + `normalized_pql` rewrite | architect (done: ADR-046) → product-owner (error-taxonomy update) → implementer | [NEW-STORY] |
| **BLOCKER-001** CrowdStrike OAuth plugin corrupts state across prism sessions | DEMO-BLOCKER | `crowdstrike-oauth2.prx` caches token in RocksDB; DTU OAuth simulation does not accept refresh from a new session (different DTU state); token becomes stale; plugin awaits HTTP response for `PLUGIN_HTTP_CLIENT_TIMEOUT_SECS` (30s) | code (WASM plugin + DTU) | n/a (no ADR; implementer investigation required) | prism-dtu-crowdstrike, crowdstrike-oauth2.prx | No new BC — existing BC-2.06.001 (CrowdStrike OAuth) covers the correct token flow | Plugin detects stale/different-session token and forces reauth instead of hanging; OR DTU `/oauth2/token` refresh endpoint accepts refresh requests from new sessions | implementer (investigate OAuth2 plugin + DTU mock); route to DTU-validator for DTU-side fix | [NEW-STORY] |
| **BLOCKER-002** Runbook §5.5 pipe syntax is syntactically invalid | DEMO-BLOCKER | Runbook was written against aspirational BNF (GRAMMAR-009); `FROM t WHERE … LIMIT N` is invalid pipe mode | docs | n/a | scripts/ or demo runbook | n/a (doc-only fix) | Demo runbook §5.5 uses `FROM t \| where … \| enrich fn(col) \| limit N` — valid pipe syntax | implementer / demo-preparer (no BC needed; doc fix only) | [NEW-STORY] (trivial — 1-line runbook fix) |
| **BLOCKER-003** `query_tutorial` and `investigate_host` prompts hang indefinitely | DEMO-BLOCKER | Render functions `render_query_tutorial` / `render_investigate_host` are synchronous pure functions (confirmed by code read) and return immediately; hang is in `PromptRouter` dispatch layer or `#[prompt_handler]` macro expansion | code | ADR-046 D6 (investigation protocol) | prism-mcp | No new BC until root cause confirmed; existing prompt BCs (BC-2.10.009) must remain true | Both prompts return within 5s; no change to prompt content | implementer (investigate `#[prompt_handler]` macro expansion / rmcp 1.7 PromptRouter) | [NEW-STORY] |
| **BLOCKER-004** `list_infusions`, `plugin_status`, `infusion_status` hang indefinitely | BLOCKER | All three tools call `emit_tool_audit` BEFORE `not_yet_available_msg`; if `audit_writer` channel is blocked/saturated, tools hang before reaching the fast-fail Err path | code | n/a | prism-mcp | Existing BC for `not_yet_available_msg` fast-fail pattern; implementer must verify the audit channel is not the block | All three tools return fast JSON-RPC -32003 within 1s (not 30s hang) | implementer (audit channel saturation diagnosis; route to separate audit-channel fix if needed) | [NEW-STORY] |

---

### MAJOR Findings (7 MAJOR from grammar audit + 1 MAJOR from pre-flight)

| Finding | Severity | Root-Cause Hypothesis | Fix Bucket | ADR | Affected Crate(s) | BC/Contract Surface Needed | Expected Post-Fix Behavior | Ownership | 001-C? |
|---------|----------|-----------------------|------------|-----|-------------------|---------------------------|---------------------------|-----------|--------|
| **GRAMMAR-002** `LIMIT`/`limit` means three different things across modes | MAJOR | SQL mode trailing `LIMIT`; pipe `\| limit`; pipe `\| head` all map to "first N rows" — three spellings for one concept | docs | n/a | prism-mcp (reference) | Reference note added (see ADR-045 build_reference_content) | Reference states: "`head N == limit N` in pipe mode; `LIMIT N` is trailing clause in SQL mode; all are case-insensitive" | implementer (reference generation) | [NEW-STORY] (covered under reference overhaul) |
| **GRAMMAR-003** `client_id` / scope is not query syntax; `_client`/`_sensor` virtual fields invisible | MAJOR | `clients` tool parameter docs are thin; virtual fields (`_client`, `_sensor`, `_source_table`, `_safety_flags`) are entirely undocumented in any teaching surface | docs | n/a | prism-mcp (reference), BC-2.11.001 | BC-2.11.001 v1.13 updated with `clients` param + virtual field docs | Reference and `query_tutorial` explicitly document scope-via-tool-param and virtual fields | product-owner (001-C covers `clients` param), implementer | [001-C] (partial) + [NEW-STORY] for virtual fields |
| **GRAMMAR-005** `enrich fn` (without parens) fails with bare `expected '('` | MAJOR | Chumsky `enrich_stage` parser requires `(col)` form; error message does not name the column arg or give an example | error-messages | ADR-046 (D1/D2 parse-time error improvements) | prism-query | No new BC — E-QUERY parse error format improvement | Parse error says: "`enrich` requires a column argument: `\| enrich <infusion>(<column>)`. Example: `\| enrich threat_score(iocs_value)`" | implementer | [NEW-STORY] (as part of three-mode correctness story) |
| **GRAMMAR-007** Rich operators (CONTAINS, =~, IN CIDR, HAS, MISSING, percentile, dedup, fields±) are hidden | MAJOR | Reference Operators table was never updated from early SQL-only conception; `ast.rs::Predicate` is the maintained source of truth | docs | ADR-045 (reference generation) | prism-mcp | Reference operators section regenerated from `ast.rs` doc-comment operator table | Reference lists all operators including CONTAINS/ICONTAINS, `=~`/MATCHES, IN CIDR, HAS, MISSING, BETWEEN, wildcard, percentile, distinct_count, stats BY, dedup, fields± | implementer (reference generation) | [NEW-STORY] (under reference overhaul) |
| **GRAMMAR-012** Enrichment function names not discoverable from any working surface | MAJOR | `list_infusions` hangs (BLOCKER-004); `list_capabilities` wrong (MAJOR-001); reference has no UDF list; only E-QUERY-039 error carries `available_infusions` | code + docs | n/a | prism-mcp | BC-2.10.014 v1.2 — `available_infusions` in `prism_describe` | `prism_describe` includes `available_infusions: ["threat_score", "cvss_base_score", …]` per-client; discovery no longer requires guessing wrong name first | product-owner (done: BC-2.10.014 v1.2) → implementer | [001-C] |
| **GRAMMAR-013** Aggregate discoverability gap (summary of GRAMMAR-003/006/007/008/010/011/012) | MAJOR | Aggregate root: teaching surfaces document aspirational grammar, not real grammar | mixed | ADR-045 (reference overhaul) | prism-mcp | Acceptance: analyst writes correct enrich query from teaching surfaces alone | After remediation, all 10 facts in GRAMMAR-013's "must know out-of-band" list are surfaced by `prism_describe` + `prismql://reference` + errors | architect (ADR-045) → implementer | [NEW-STORY] (acceptance test) |
| **GRAMMAR-017** Reference completeness gap — pipe stages, operators, aggregates, virtual fields, composites, internal tables all absent | MAJOR | Reference was written against early SQL-only conception and predates pipe/enrich pivot | docs + CI gate | ADR-045 (D3 CI parse-round-trip gate) | prism-mcp, prism-query | ADR-045 CI gate: every `prismql://reference` code-fence example must parse via `PrismQlParser::parse` | CI fails on `prl_reference.md` examples that don't parse; reference content covers full pipe-stage list + operators + aggregates | implementer (CI gate in build.rs or dedicated test), devops-engineer (CI job) | [NEW-STORY] (CI gate; separate from reference content) |
| **MAJOR-001** `list_capabilities` returns `client_registered: False` for all orgs | MAJOR | `client_exists()` in `feature_flag.rs` line 263 checks `self.client_capabilities` map; this map is populated from `prism.toml` `[clients.*.capabilities]` write-capability config entries, NOT from the `OrgRegistry` populated by spec overlays (ADR-029). Demo provisioning uses spec overlays, not `prism.toml` client entries — so `client_capabilities` is always empty | code | ADR-046 MAJOR-001 ruling | prism-security, prism-mcp | No new BC — existing BC for `list_capabilities` must return correct data; **HRG-4 ratified Path B: `list_capabilities` consults `OrgRegistry`; Path A rejected** | `list_capabilities(org-c)` returns `client_registered: True` for all demo-provisioned orgs | implementer (Path B: wire `Arc<OrgRegistry>` to `FeatureFlagEvaluator` per ADR-022 §C Arc-DI) | [NEW-STORY] |
| **AUDIT-004** Prompts teach dot-syntax table names (triage_alerts, cross_client_status use `crowdstrike.alerts`) | MAJOR | Prompt bodies were hardcoded; dot-syntax (`crowdstrike.alerts`) triggers E-QUERY-036 at plan time | code | n/a | prism-mcp | BC-2.11.019 (prompt body regeneration from real table registry) | Prompts use `crowdstrike_detections` (underscore form); no E-QUERY-036 triggered by own prompt bodies | product-owner (done: BC-2.11.019) → implementer | [001-C] |

---

### MINOR Findings (5 MINOR)

| Finding | Severity | Root-Cause Hypothesis | Fix Bucket | ADR | Affected Crate(s) | Expected Post-Fix Behavior | Ownership | 001-C? |
|---------|----------|-----------------------|------------|-----|-------------------|---------------------------|-----------|--------|
| **GRAMMAR-004** E-QUERY-036 lacks `did_you_mean` / `available_tables` (parity gap with 037/038/039) | MINOR | 036 is a coarse materialization-layer check; structured error enrichment was added to 037/038/039 but not backported to 036 | error-messages | n/a | prism-query (error.rs) | E-QUERY-036 `UnknownSourceTable` carries `available_tables: Vec<String>` and `did_you_mean: Option<String>` matching 037/038/039 pattern | implementer | [NEW-STORY] (part of three-mode correctness story or separate polish story) |
| **GRAMMAR-006** `IS NOT NULL` semantics on JSON-list columns undocumented | MINOR | Reference only says "Structural equality only" for JSON; null semantics of empty list vs absent field vs `[null]` not specified | docs (+verify code) | n/a | prism-mcp (reference), prism-query (data-engineer verification) | Reference documents: `IS NOT NULL` on a JSON-list field returns `true` if field is present and non-null; behavior for empty list documented | implementer (doc), data-engineer (behavior verification) | [NEW-STORY] (low priority; docs subsection) |
| **GRAMMAR-015** Parse-time `enrich` shape errors far below plan-time quality bar | MINOR | Plan-time errors (E-QUERY-039) are exemplary; parse-time Chumsky errors are raw token-expectation | error-messages | ADR-046 D1/D2 | prism-query | Parse-time `enrich` errors name the correct form with an example, not just `expected '('` | implementer | [NEW-STORY] (part of three-mode correctness story) |
| **GRAMMAR-016** `normalized_pql` field documented but absent in practice | MINOR | `StructuredErrorFields` did not have a `normalized_pql` field (D-1110: `ParseErrorDetails` is a phantom type that does not exist); ADR-041 `OPD-1` planned it but it was not implemented | code | ADR-046 D3 | prism-query | `StructuredErrorFields` carries `normalized_pql: Option<String>`; mode-bridge errors populate it with a best-effort rewrite | product-owner (error-taxonomy E-QUERY-001 update) → implementer | [NEW-STORY] (part of three-mode correctness story) |
| **GRAMMAR-018** Case convention inconsistent and undocumented | MINOR | Parser accepts all keywords case-insensitively (`kw_ci`); teaching surfaces use UPPER SQL / lowercase pipe, creating a visual false rule | docs | ADR-046 D5 | prism-mcp (reference) | Reference states: "All PrismQL keywords are case-insensitive. Convention: UPPER for SQL-mode keywords, lowercase for pipe stage names" | implementer (reference generation) | [NEW-STORY] (covered under reference overhaul) |

---

### NIT Finding

| Finding | Severity | Root-Cause Hypothesis | Fix Bucket | Affected Crate(s) | Expected Post-Fix Behavior | Ownership | 001-C? |
|---------|----------|-----------------------|------------|-------------------|---------------------------|-----------|--------|
| **GRAMMAR-019** Column naming convention (clean identifiers vs source paths) undocumented | NIT | No guidance on when to use dot-path vs flat snake_case column name | docs | prism-mcp (reference) | Reference note: "Column names come verbatim from `prism_describe`; use the name as shown; do not construct dot-path names" | implementer (reference generation) | [NEW-STORY] (low priority; one reference line) |

---

## Summary: New Story Requirements

The following findings require new stories (not covered by 001-C). Product-owner
authors the BCs; story-writer decomposes into implementable stories.

### Priority 1 — Demo-blockers (must fix before T13)

| Finding(s) | Story Theme | Key BC Contract Needed |
|------------|-------------|----------------------|
| BLOCKER-001 | CrowdStrike OAuth plugin session-isolation fix | BC for OAuth plugin retry-vs-reauth decision; DTU `/oauth2/token` refresh behavior |
| BLOCKER-002 | Demo runbook §5.5 pipe syntax fix | No BC needed — doc fix; 1-line change to demo runbook/scripts |
| BLOCKER-003 | Prompt hang investigation + fix | BC-2.10.009 existing; investigate `#[prompt_handler]` macro; no new BC until root cause |
| BLOCKER-004 | `list_infusions`/`plugin_status`/`infusion_status` hang fix | Fast-fail within 1s; `emit_tool_audit` must not block the fast-fail path |
| GRAMMAR-011 | `NOW()` + `INTERVAL` grammar implementation | New BCs: `Expr::Now`/`Expr::Interval`/`TimestampArithmetic` postconditions + planning-time constant injection |
| GRAMMAR-001 / GRAMMAR-014 | Mode-bridge error on `\|` in SQL mode + `normalized_pql` field | E-QUERY-001 error-taxonomy update: `normalized_pql` field; mode-bridge message spec |
| MAJOR-001 | `list_capabilities` returns correct org registration state | Either `demo-setup.sh` fix or `client_exists()` consults OrgRegistry |

### Priority 2 — Reference overhaul (highest discoverability leverage)

| Finding(s) | Story Theme | Key BC Contract Needed |
|------------|-------------|----------------------|
| GRAMMAR-009, GRAMMAR-008, GRAMMAR-007, GRAMMAR-017, GRAMMAR-002, GRAMMAR-003 (virtual fields), GRAMMAR-018, GRAMMAR-019 | `build_reference_content()` replaces static `pql_reference.md` | ADR-045 implementation BC: function signature, static constants, runtime infusions section, CI parse-round-trip gate |

### Priority 3 — Filter mode execution validation + parse-time error improvements

| Finding(s) | Story Theme | Key BC Contract Needed |
|------------|-------------|----------------------|
| ADR-046 D4 | Filter mode end-to-end integration tests | BC-2.11.002 (Filter mode execution) — existing contract; tests verify it end-to-end |
| GRAMMAR-005 / GRAMMAR-015 / GRAMMAR-016 | Parse-time enrich error improvements + `normalized_pql` implementation | E-QUERY-001 `StructuredErrorFields.normalized_pql` field addition (D-1110); error-taxonomy row update |
| GRAMMAR-004 | E-QUERY-036 parity with 037/038/039 | E-QUERY-036 `UnknownSourceTable` struct adds `available_tables`/`did_you_mean` |
| GRAMMAR-006 | JSON/list `IS NOT NULL` documentation | Reference doc section; data-engineer to verify code behavior |

---

## Root-Cause Hypotheses (Blocker-class, for implementer briefing)

### BLOCKER-001: CrowdStrike OAuth session corruption

**Hypothesis (HIGH CONFIDENCE):** `crowdstrike-oauth2.prx` stores the OAuth2 token in
RocksDB (CF `plugin_state`). On session 2, the plugin reads the cached token and attempts
a refresh against the DTU `POST /oauth2/token` endpoint. The DTU's mock responds only to
initial `grant_type=client_credentials` requests; it does not implement
`grant_type=refresh_token` (or the token it issued in session 1 is not in its in-memory
token store after restart). The plugin waits for a successful token response that never
arrives, blocking for exactly `PLUGIN_HTTP_CLIENT_TIMEOUT_SECS` = 30s.

**Implementer action:** (a) Read `crowdstrike-oauth2.prx` token refresh flow; determine
whether it does `refresh_token` or re-does `client_credentials` on cache miss. (b) Read
`crates/prism-dtu-crowdstrike/src/routes/oauth2.rs` — verify which token flows are
implemented. (c) Fix: either the plugin detects a stale token (different DTU session) and
forces a full reauth, OR the DTU implements the missing token refresh endpoint. Path (c-DTU)
is faster; path (c-plugin) is more robust for production.

### BLOCKER-003: Prompt hang

**Hypothesis (HIGH CONFIDENCE):** Hang is NOT in `render_query_tutorial` or
`render_investigate_host` — both are synchronous pure functions (confirmed by reading
`crates/prism-mcp/src/prompts.rs`). The hang is in the rmcp 1.7 `PromptRouter` dispatch
layer or the `#[prompt_handler]` macro expansion. Candidate causes in priority order:
1. The `#[prompt_handler]` macro generates an async closure that awaits on a future that
   never resolves (e.g., awaiting on a `recv()` from a channel that was never populated).
2. The rmcp 1.7 `PromptRoute::new_dyn` closure captures the router lock and does not
   release it before the handler returns.
3. A deadlock between the MCP server's tool-handler lock and the prompt-handler lock when
   both are registered on the same `rmcp::Server` instance.

**Distinguishing test:** The three working prompts (`triage_alerts`, `client_overview`,
`cross_client_status`) all have simple string-only args or no args. The two hanging
prompts have: `query_tutorial` (`client_id: String, goal: Option<String>`) and
`investigate_host` (`client_id: String, hostname: String`). The `hostname` arg is
REQUIRED on `investigate_host` — if the macro expansion tries to resolve a required-arg
value from the MCP params map and the routing machinery has a bug with required args,
this would explain selective hanging.

**Implementer action:** Run `cargo expand -p prism-mcp` to inspect the `#[prompt_handler]`
macro expansion for `render_query_tutorial` and `render_investigate_host`. Identify the
blocking point from the expanded async closure.

### BLOCKER-004: list_infusions / plugin_status / infusion_status hang

**Hypothesis (HIGH CONFIDENCE):** All three tools call `emit_tool_audit(...)` before
reaching the `Err(not_yet_available_msg(...))` return. If `emit_tool_audit` writes to an
`audit_writer: mpsc::Sender` and that channel's buffer is full (or the receiver has been
dropped), the `send()` call will block. Since `NOT_YET_AVAILABLE_TOOLS` routes go through
the same server handler infrastructure as normal tools, the audit call happens before the
fast-fail guard.

**Implementer action:** Check `crates/prism-mcp/src/server.rs` `emit_tool_audit` —
determine whether it uses `try_send` (non-blocking) or `send` (blocking). If `send`, the
audit channel is the hang source. Fix: use `try_send` with a log-warn on `Err(Full)` (the
audit buffer full condition should not block the fast-fail path). Alternatively, re-order
the `not_yet_available_msg` check BEFORE the `emit_tool_audit` call — if the tool is
not-yet-available, there is no audit event to emit (no tool execution happened).

### MAJOR-001: list_capabilities returns client_registered: False

**Hypothesis (HIGH CONFIDENCE):** `feature_flag.rs::client_exists()` at line 263 checks
`self.client_capabilities.contains_key(client_id)`. This map is populated during
`FeatureFlagEvaluator` construction from the `prism.toml` `[clients.{id}.capabilities]`
sections (write capability configuration). Demo provisioning (`demo-setup.sh`) populates
per-org spec overlay files (ADR-029) under `~/.config/prism-demo/specs/customers/` but
does NOT write `[clients.org-*]` entries to `prism.toml`. Therefore `client_capabilities`
is empty for all demo orgs, and `client_exists()` returns `false` for all.

**Implementer action (two valid paths):**
- **Path A (faster, demo-ready):** Add `[clients.org-a]`, `[clients.org-b]`, `[clients.org-c]`
  entries to `prism.toml` or to `demo-setup.sh` output config. These entries need only
  exist (no write capabilities needed for the demo); `client_exists()` will return `true`.
- **Path B (architecturally correct):** Modify `list_capabilities` to consult the
  `OrgRegistry` (populated from spec overlays per ADR-006/ADR-029) for basic org
  existence check, independent of write-capability config. This decouples capability
  discovery from write-capability provisioning, which is the correct semantic.

Path B is the production-grade default per ADR-006. Path A is acceptable for the demo if
Path B is out-of-scope for the current sprint.

---

## ADRs Produced by This Design Burst

| ADR | Title | Status | Key Decisions |
|-----|-------|--------|---------------|
| ADR-043 | True SQL→Pipe Composition | ACCEPTED v1.1 | New `Ast::SqlPipe(SqlPipeQuery)` variant; `QueryMode` tristate enum; SQL head then pipe stages execution; **HRG-1 FORBID-BOTH: dual SQL `LIMIT` + pipe `\| limit` is plan-time E-QUERY-040 error; pipe-wins is the only acceptable future relaxation** |
| ADR-044 | Temporal Grammar — NOW() + INTERVAL | PROPOSED v1.0 | `Expr::Now` / `Expr::Interval(Duration)` / `Expr::TimestampArithmetic`; planning-time constant injection; `build_example_query` becomes valid automatically |
| ADR-045 | Auto-Generated PrismQL Reference | ACCEPTED v1.1 | Replace static `pql_reference.md` with `build_reference_content(infusion_registry: Option<&InfusionRegistry>)`; **HRG-3 HYBRID: static constants + runtime infusions assembly + CI gate (shared example array: positive round-trip + negative E-QUERY-040 + registry-parity gates)**; build-time codegen rejected; amends ADR-041 §L3 |
| ADR-046 | Three-Mode Correctness | ACCEPTED v1.3 | Mode-bridge E-QUERY-001 heuristic on `\|` in SQL mode; `StructuredErrorFields.normalized_pql: Option<String>` (D-1110); Filter mode execution test mandate; BLOCKER-003 investigation protocol; **HRG-4/MAJOR-001 Path B: `list_capabilities` consults `OrgRegistry`**; **D7: Filter is bare-predicate sugar; only SQL→Pipe composes; pipe is canonical execution model; shared predicate grammar; three-way composition rejected**; amends ADR-041 §L4 |

---

## Human Ratification Gates (before story authoring)

The following decisions require explicit human approval before product-owner can author
the downstream BCs and story-writer can decompose:

| Gate | Decision | Ruling (2026-06-24) |
|------|----------|---------------------|
| **HRG-1 (ADR-043)** | LIMIT semantics: when `SELECT … LIMIT 5 FROM t \| limit 3` — which limit wins? Architect recommendation was pipe-wins. | **RATIFIED: FORBID-BOTH.** A composed query MAY NOT specify both SQL `LIMIT` and pipe `\| limit`. Plan-time pedagogical error **E-QUERY-040** allocated. Pipe-wins is the only acceptable future relaxation (forbid→permit is non-breaking; SQL-wins has zero precedent and is permanently ruled out). |
| **HRG-2 (ADR-043)** | Confirm Option 2 (full SQL→Pipe composition) over Option 3 (mode-bridge error only, no grammar extension) | **RATIFIED: Option 2 confirmed.** Human directed at session start; formalized here. |
| **HRG-3 (ADR-045)** | Reference generation strategy: static Rust string constants + runtime infusions vs build-time codegen. Architect recommendation: static constants + CI gate. | **RATIFIED: HYBRID.** Static `&'static str` constants for grammar/operator sections co-located with the Chumsky grammar + runtime assembly of infusions section from live `InfusionRegistry` + CI round-trip gate (shared example array: positive round-trip gate + negative E-QUERY-040 gate + registry-parity gate). Build-time codegen REJECTED. |
| **HRG-4 (MAJOR-001)** | Fix path: Path A (demo-setup.sh adds prism.toml entries, fast) vs Path B (`list_capabilities` consults OrgRegistry, architecturally correct). | **RATIFIED: Path B.** `list_capabilities` consults the authoritative `OrgRegistry` for org existence independently of write-capability config. Path A rejected as a demo-expedient workaround that papers over the architectural misalignment. `FeatureFlagEvaluator` gets `Arc<OrgRegistry>` via ADR-022 §C Arc-DI wiring. |

---

## Files Produced by This Design Burst

- `/Users/jmagady/Dev/prism/.factory/specs/architecture/decisions/ADR-043-true-sql-to-pipe-composition-select-from-t-stage-head-lowers-to-pipe-source.md`
- `/Users/jmagady/Dev/prism/.factory/specs/architecture/decisions/ADR-044-temporal-grammar-now-and-interval-relative-duration-literals.md`
- `/Users/jmagady/Dev/prism/.factory/specs/architecture/decisions/ADR-045-auto-generated-prismql-reference-resource-grammar-registry-parity-gate.md`
- `/Users/jmagady/Dev/prism/.factory/specs/architecture/decisions/ADR-046-three-mode-correctness-filter-sql-pipe-mode-bridge-error-and-execution-validation.md`
- `/Users/jmagady/Dev/prism/.factory/specs/architecture/demo-readiness-remediation-design-2026-06-24.md` (this file)
- `/Users/jmagady/Dev/prism/.factory/specs/architecture/ARCH-INDEX.md` (v2.143→v2.144; ADR-043..046 rows added)
