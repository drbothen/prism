---
document_type: adversarial-review-pass
story: S-DEMO-PRISMQL-GRAMMAR-REMEDIATION-001
cascade_type: LOCAL
pass_number: 1
frozen_head: "329fa519"
diff_range: "903c8fcb..329fa519"
adversary_agent_id: a21b900e798cac0ec
date: 2026-06-24
clean_strict: false
clean_pr_merge: false
streak_after: "0/3"
streak_reset: true
---
# LOCAL Adversarial Cascade — Pass 1

## Identity

- **Story:** S-DEMO-PRISMQL-GRAMMAR-REMEDIATION-001
- **Cascade type:** LOCAL (BC-5.39.001)
- **Pass:** 1 of 3 required for convergence
- **Frozen HEAD:** `329fa519` (full story diff `903c8fcb..329fa519`)
- **Adversary agent ID:** a21b900e798cac0ec
- **Date:** 2026-06-24

## Verdict

| Criterion | Result |
|-----------|--------|
| CLEAN (strict) — ZERO findings of any severity | **NO** |
| CLEAN (PR-merge) — ZERO CRIT/HIGH/MED findings | **NO** |
| 3-CLEAN streak advance | **NO — streak RESET to 0/3** |

Pass 1 found a **systemic "implemented-but-unwired" pattern**: functions and handlers implemented with isolated tests but NOT wired into the production `QueryEngine::execute` / rmcp dispatch / host plugin path.

---

## Findings

### CRIT-1 — BC-2.11.020 / ADR-043 D4: `plan_sqlpipe_query` has ZERO production callers

**Severity:** CRITICAL
**BC anchor:** BC-2.11.020, ADR-043 §D4
**Location:** `lib.rs:208` — FORBID-BOTH + E-QUERY-040 validator

`plan_sqlpipe_query` implements the FORBID-BOTH E-QUERY-040 validator but has **zero production callers**. It is not wired into `QueryEngine::execute`. Composed queries do not receive the E-QUERY-040 validation at runtime. The headline composition feature parses correctly but the enforcement gate is dead code in production.

**Confirmed by orchestrator git/grep:** function exists in lib.rs; `grep` across workspace finds no caller in the execute path.

---

### CRIT-2 — BC-2.11.020: `Ast::SqlPipe` has NO execution arm in `execute_against_session`

**Severity:** CRITICAL
**BC anchor:** BC-2.11.020
**Location:** `materialization.rs` — `execute_against_session`

`Ast::SqlPipe` falls to the `_ => Ok(Vec::new())` catch-all arm in `execute_against_session`. Composed SQL→pipe queries silently return empty at runtime. No execution arm materializes the composed query.

**Confirmed by orchestrator git/grep:** `Ast::SqlPipe` appears only in `filter_parser.rs` (construct) and `explain.rs:692` (no-op). No materialization arm.

---

### CRIT-1b — AC-004/005: `NOW()`/`INTERVAL` plan-time injection not applied on production execute path

**Severity:** CRITICAL
**BC anchor:** AC-004, AC-005
**Location:** `parse_and_plan` / `inject_now`

The `parse_and_plan`/`inject_now` pipeline for plan-time `NOW()` and `INTERVAL` substitution is **also not applied on the production execute path**. Un-substituted `Expr::Now` reaches DataFusion. The injection exists as an isolated function with tests but the production wiring is absent.

---

### CRIT-3 — AC-019 BLOCKER-001: `reset_token_cache` unreachable in production

**Severity:** CRITICAL
**BC anchor:** AC-019, BLOCKER-001
**Location:** CrowdStrike OAuth2 plugin

`reset_token_cache` in the crowdstrike-oauth2 plugin is `pub(crate)` and gated `#[cfg(test/wasm32)]`. It is NOT in the sensor-auth WIT interface and has zero host callers. It is unreachable in production. Production `acquire-token` may always force-acquire; eviction may address a non-issue.

**Root-cause re-derivation routed to implementer** (architect consult required if WIT/interface change is needed).

---

### HIGH-1 — AC-015/016 BLOCKER-003: Prompt dispatch partial gap

**Severity:** HIGH
**BC anchor:** AC-015, AC-016, BLOCKER-003
**Location:** Prompt tests

The `get_prompt` full-transport test covers `query_tutorial`, but the two AC-015/016 tests (`prompts_fast_return`, `missing_required_arg`) still call `render_*` directly rather than via `PromptRouter`/`ServerHandler` dispatch. `investigate_host` and the missing-arg path are not covered via dispatch. This is a partial gap — not a full reflag of the prior HIGH-001 fix, but the coverage gap remains.

---

### HIGH-2 — AC-021: `did_you_mean Some(...)` strsim branch untested

**Severity:** HIGH
**BC anchor:** AC-021
**Location:** `resolve_source_refs`

The `did_you_mean Some(...)` strsim branch in `resolve_source_refs` is untested. Only the `None` path is tested (the `unknown_table` test case is Levenshtein distance > 3 from all sensors). The `available_tables` population is also unasserted.

---

### HIGH-3 — Capability partition: `check_sensor_health` in `NOT_YET_AVAILABLE_TOOLS`

**Severity:** HIGH
**BC anchor:** Capability partition contract
**Location:** `server.rs:1444` (`NOT_YET_AVAILABLE_TOOLS`), `server.rs:~3099` (real handler)

`check_sensor_health` is a real, working handler (`server.rs ~3099`) but is listed in `NOT_YET_AVAILABLE_TOOLS` (`server.rs:1444`). `list_capabilities` therefore reports it as unavailable even though it works.

**ORCHESTRATOR CONFIRMED via `git diff 903c8fcb..329fa519`:** This is **NOT introduced by this branch**. It is a **pre-existing condition from S-5.04 merge**. Cheap in-scope fix: move to `LIVE_TOOLS` per production-grade default. Routed to implementer; escalate if ripple is detected.

---

### OBS-1 (LOW) — Stale doc comment in `lib.rs ~195-197`

**Severity:** LOW / OBS
**Location:** `lib.rs ~195-197`

Stale "test-writer stubs ... `todo!()`" doc comment. Functions now have real bodies. Comment is misleading.

---

### OBS-2 (LOW, spec-drift) — BC-2.11.023 struct name / location mismatch

**Severity:** LOW / OBS (spec-drift)
**BC anchor:** BC-2.11.023

BC-2.11.023 still names struct `ParseErrorDetails` and locates `normalized_pql` in `prism-query/src/error.rs`, but code (correctly, per D-1110 + story precedence) uses `StructuredErrorFields` in `prism-mcp/src/error_mapping.rs`.

**Route:** product-owner to sync BC-2.11.023 prose. Non-blocking; code is correct.

---

### OBS-3 (LOW, spec-drift) — Story Architecture-Mapping file path error

**Severity:** LOW / OBS (spec-drift)
**Location:** Story §Architecture-Mapping/§File-Structure

Story says the `-32602`/`RedundantRowLimit`/`UnknownSourceTable` MCP mapping is in `prism-core/src/error_mapping.rs` but it is actually in `prism-mcp/src/error_mapping.rs`.

**Route:** story-writer for doc correction. Non-blocking; code is correct.

---

### OBS-4 / PG-1 (PROCESS-GAP) — Weak capability-partition test lets HIGH-3 pass CI

**Severity:** PROCESS-GAP (OBS)
**Location:** `test_MCP_01_capability_classification_partitions_tool_catalog`

The test only checks disjointness and coverage (no tool appears in both lists; all tools in exactly one list). It does NOT assert that a known-implemented tool appears in `LIVE_TOOLS`. This gap allowed HIGH-3 (`check_sensor_health` in `NOT_YET_AVAILABLE_TOOLS`) to pass CI undetected. Strengthen with positive-coverage assertions: "tool X which is implemented MUST be in LIVE_TOOLS."

---

## Genuinely Solid (No Findings)

The adversary found these areas clean — no findings raised:

- **MCP fast-fail BC-2.10.017:** 41-handler sweep confirmed; all non-permitted tools fast-fail correctly.
- **GRAMMAR-004 E-QUERY-036 production wiring:** Real `registered_sensor_ids` + strsim integrated; wiring confirmed.
- **Non-exhaustive count:** 87 types confirmed (correct; worktree EXPECTED=87 gate passes).

---

## Systemic Pattern Note

The dominant finding class in this pass is **"implemented-but-unwired"**: functions and handlers exist with isolated tests that pass, but the production execution path (`QueryEngine::execute`, rmcp dispatch, host plugin interface) was never wired. This pattern:

- Passed Group-1 and Group-2 TDD as "GREEN" because individual function tests pass.
- Was not caught until fresh-context cascade with full execution-path tracing.

**LESSON:** Red-Gate tests that call new `pub` fns directly (not via `QueryEngine::execute`) do not prove production wiring. The cascade must verify execution-path wiring, not just function existence.

---

## Routing Decisions

| Finding | Route |
|---------|-------|
| CRIT-1, CRIT-1b, CRIT-2 | Implementer: wire `plan_sqlpipe_query` + `Ast::SqlPipe` execution arm + `inject_now` into `QueryEngine::execute` |
| CRIT-3 | Implementer (root-cause re-derivation); architect consult if WIT/interface change required |
| HIGH-1, HIGH-2 | Implementer: add dispatch-path coverage for AC-015/016 + strsim Some(…) branch |
| HIGH-3 | Implementer: move `check_sensor_health` to `LIVE_TOOLS`; escalate if ripple |
| OBS-1 | Implementer: remove stale doc comment |
| OBS-4 / PG-1 | Implementer: strengthen partition test with positive-coverage assertions |
| OBS-2 | Product-owner: sync BC-2.11.023 prose |
| OBS-3 | Story-writer: correct §File-Structure path |

---

## Next Steps

After the fix-burst:

1. Re-freeze HEAD (no new commits after fix-burst before re-running cascade).
2. Re-run LOCAL adversarial cascade Pass 1 against the new frozen HEAD.
3. Streak resets from 0/3; re-gate on Pass 1 clean to start streak.
