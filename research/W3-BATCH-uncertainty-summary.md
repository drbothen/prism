---
document_type: uncertainty-batch-summary
batch_id: W3-2026-05-04
version: "1.0"
producer: dclaude-uncertainty-scanner
timestamp: "2026-05-04T00:00:00Z"
status: draft
scope: "S-3.02 through S-3.13 (12 stories) — pre-implementation tech-uncertainty scan per directive D-223"
out_of_scope: "S-3.01 (handled by separate scanner)"
---

# Wave 3 Batch Uncertainty Summary

## Overall W3 Verdict

**RED** — Two stories carry RED-level blockers (S-3.05 cross-reference
conflict; S-3.07 self-flagged DataFusion DML API uncertainty). Most other
stories are YELLOW pending confirmation of DataFusion 53.x API surfaces
that have shifted across the 30→53 evolution. None are GREEN-clean except
S-3.08 which is mechanically simple.

The dominant single risk is **DataFusion 53.x API drift** since ADR-015
pinned `datafusion = "53.1"` for Wave 4 detection-rule work. 12 of 12
stories cite DataFusion APIs without confirming current minor-patch
behavior. Recommendation: a single-pass DataFusion 53.x API verification
sweep before any Tier 3 story begins.

## Verdicts Per Story

| Story | Tier | Verdict | Critical findings | Important findings | Notes |
|---|---|---|---|---|---|
| S-3.02 | 2 (path) | YELLOW | 0 | 4 | DataFusion API self-flagged (line 387) |
| S-3.03 | 3 | YELLOW | 0 | 2 | `create_logical_plan` API drift |
| S-3.04 | 3 | YELLOW (near GREEN) | 0 | 1 | Internal RocksDB-vs-toml inconsistency |
| S-3.05 | 3 | **RED** | 1 | 4 | `lru` workspace-pin conflict (0.12 vs 0.17) |
| S-3.06 | 2 | YELLOW | 0 | 3 | Chumsky 0.12 dynamic-dispatch self-flagged |
| S-3.07 | 4 | **RED** | 2 | 2 | DataFusion `insert_into` self-flagged; UPDATE/DELETE trait surface unclear |
| S-3.08 | 3 | GREEN | 0 | 0 | Mechanical Arrow projection |
| S-3.09 | 3 | YELLOW | 0 | 1 | `MemoryPool::reserved()` API drift |
| S-3.10 | 4 | YELLOW | 0 | 2 | Join-order hint behavior unconfirmed |
| S-3.11 | 3 | YELLOW | 0 | 2 | Coherence with S-3.05 cache scope |
| S-3.12 | 3 | YELLOW | 0 | 2 | `OptimizerRule` trait shape drift |
| S-3.13 | 3 | YELLOW | 0 | 2 | `register_table` concurrency semantics |

## Cross-Story Patterns

### Pattern 1 — DataFusion 53.x version-pin & API drift (10 stories)

S-3.02, S-3.03, S-3.07, S-3.08, S-3.09, S-3.10, S-3.11, S-3.12, S-3.13 all
cite "datafusion 53" or "datafusion 53.x" without minor pin, while ADR-015
pins exactly `datafusion = "53.1"` for Wave 4. Workspace
`crates/prism-query/Cargo.toml` does not yet contain a datafusion dep —
S-3.02 is the first to introduce it.

**Specific 53.x API claims that need verification:**

- `SessionContext::new_with_config_rt` (S-3.02) — constructor name
- `SessionContext::create_logical_plan` (S-3.03) — may live on `state()` now
- `LogicalPlan::display_indent` (S-3.03)
- `MemoryPool::reserved()` (S-3.09) — vs `consumed()` / `current()`
- `TableProvider::insert_into` signature with `InsertOp` enum (S-3.07) — self-flagged
- `OptimizerRule::try_optimize` vs `rewrite()` with `TreeNode` (S-3.12)
- `SessionContext::register_table` / `deregister_table` thread-safety (S-3.13)
- Built-in `PushDownProjection` rule presence (S-3.12)

**Recommended action:** Single research pass using
`mcp__perplexity__deep_research` (or equivalent) to validate the latest
datafusion 53.x patch and the eight API surfaces above; produce a single
"datafusion-53-api-confirmed.md" memo that all W3 stories reference.

### Pattern 2 — Chumsky 0.12 (1 story, but architecture-load-bearing)

Only S-3.06 directly uses Chumsky for write-parser extensions. Self-flagged
two specific patterns: dynamic verb dispatch and `recover_with` strategies.
Chumsky 0.13 may have stabilized since the AD-003 pin.

### Pattern 3 — Cache crate fragmentation (S-3.05 + S-3.11)

S-3.05 mentions both `lru 0.12.x` (dep table) and `moka` (body text) for
the response cache. S-3.11 builds an in-query dedup cache with
`RecordBatch` values. Three different caching contexts; should
explicitly select one library per context and pin it.

The active workspace already declares `lru = "0.17"` as a dev-dep in
`crates/prism-query/Cargo.toml` — story S-3.05's `0.12.x` pin is **stale
relative to the workspace by ~5 majors** (lru API redesigned multiple times
since 0.12).

### Pattern 4 — ADR-008 org_id threading

Three stories touch state that should carry an `{org_id}:` prefix per
ADR-008 (universal re-keying):

- S-3.04 alias persistence (file-based — partially exempt)
- S-3.05 cache key derivation (uses `TenantId` — confirm canonicality)
- S-3.07 audit intent log (writes via prism-audit — confirm caller populates org_id)
- S-3.11 in-query cache (multi-client fan-out within one query)

None are critical alone, but a sweep to confirm the `TenantId == org_id`
contract (or whether `client_id` is a separate concept) would close
ambiguity. ADR-013 (60s tick) and ADR-016 (action_state CF) are not
relevant to W3 query-engine work.

### Pattern 5 — Cross-story AST coherence (S-3.06 + S-3.07)

S-3.06 produces AST nodes (`WriteNode/DmlNode/WriteArg`); S-3.07 consumes
them. Neither story explicitly states the AST module location. Risk: the
8-way Tier 3 parallel may produce a second copy of the AST in S-3.07's
work unless the dependency direction is enforced (S-3.06 → S-3.07).

## Implementation Order Risk Assessment (Tier 3 parallelism)

The Tier 3 set (8-way parallel: S-3.03, S-3.04, S-3.05, S-3.08, S-3.09,
S-3.11, S-3.12, S-3.13) has the following coordination risks:

| Risk | Stories | Severity | Mitigation |
|---|---|---|---|
| Multiple cache implementations co-evolving | S-3.05, S-3.11 | MEDIUM | Single owner for `cache_key.rs` design; agree on `lru = "0.17"` baseline before kickoff |
| Optimizer-rule + dynamic-table-registration race | S-3.12, S-3.13 | MEDIUM | Both touch SessionContext lifecycle; ensure read-then-modify ordering documented |
| Profiling instrumentation ↔ explain output | S-3.03, S-3.09 | LOW | S-3.09 emits metrics; S-3.03 reads "estimated vs actual" — agree on QueryMetrics struct shape |
| Hidden columns + column pruning interaction | S-3.08, S-3.12 | LOW | Pruning must NOT remove hidden columns from the schema before post-filter; spec is correct in S-3.08 line 92 |
| Alias expander runs pre-parse | S-3.04 | LOW | Independent; runs before Chumsky parser |

S-3.07 (Tier 4) and S-3.10 (Tier 4) wait for prerequisites; they each carry
their own RED/YELLOW issues (above) but no parallel-coordination risk.

## Recommended Pre-Implementation Spec Updates

Consolidated bullet list (story-writer / orchestrator action items):

1. **S-3.02:** Pin DataFusion to `=53.1.x` in dep table; remove the warning
   "API has changed" by performing the verification now and updating with
   confirmed signatures for `SessionContext::new_with_config_rt`,
   `MemTable::try_new`, `GreedyMemoryPool::new`.
2. **S-3.03:** Update line 88 to use the actual 53.x API
   (`SessionContext::state().create_logical_plan(sql)` if that is correct).
   Confirm `LogicalPlan::display_indent()` exists or substitute current method.
3. **S-3.04:** Resolve internal contradiction at line 412 (RocksDB CF claim
   vs aliases.toml file persistence). Pin Kani version.
4. **S-3.05:** **Critical** — change `lru 0.12.x` → `lru 0.17.x` to match
   workspace; OR adopt `moka` and remove `lru`. Decide single backend.
   Confirm `TenantId == org_id` per ADR-008.
5. **S-3.06:** Confirm Chumsky 0.12 dynamic-dispatch pattern with code
   sketch; remove "verify this pattern against Chumsky 0.12 API" caveat
   once confirmed. Validate `datafusion-sql::DmlStatement` is publicly
   exported in 53.x or remove the optional reference.
6. **S-3.07:** **Critical** — verify DataFusion 53.x `TableProvider`
   write-side API (`insert_into` with `InsertOp`; whether UPDATE/DELETE
   trait methods exist). Update story with confirmed signatures and adjust
   point estimate if extension-planner work is required.
7. **S-3.08:** No spec changes required (GREEN).
8. **S-3.09:** Confirm `MemoryPool::reserved()` is the high-water reader,
   adjust if needed.
9. **S-3.10:** Confirm DataFusion 53.x respects manual join-order hints
   absent table statistics; update story rationale if not.
10. **S-3.11:** Document boundary with S-3.05 explicitly (in-query lifetime
    vs process-global lifetime). Confirm cache_key includes `org_id`
    in addition to `client_id`.
11. **S-3.12:** Pick one pruning surface — DataFusion `OptimizerRule` OR
    pre-DataFusion AST walker for sensor field selection. If both, document
    the boundary. Confirm `OptimizerRule` trait shape in 53.x.
12. **S-3.13:** Confirm `register_table`/`deregister_table` concurrency
    semantics or implement a `CatalogProvider`-based approach.

13. **Cross-cutting:** Produce a single
    `.factory/research/datafusion-53-api-confirmed-2026-05-04.md` memo
    consolidating verified API surfaces across S-3.02/03/07/08/09/10/11/12/13.

14. **BC-INDEX cross-check:** Stories S-3.08, S-3.09, S-3.10, S-3.11,
    S-3.12, S-3.13 all carry empty `behavioral_contracts:` and `anchor_bcs:`
    in frontmatter. Per VSDD this is unusual for L4 stories — escalate to
    orchestrator whether this is intentional or whether BCs need to be
    generated/anchored before implementation.

## Quick reference — Files written

- `/Users/jmagady/Dev/prism/.factory/research/S-3.02/uncertainty-map.md`
- `/Users/jmagady/Dev/prism/.factory/research/S-3.03/uncertainty-map.md`
- `/Users/jmagady/Dev/prism/.factory/research/S-3.04/uncertainty-map.md`
- `/Users/jmagady/Dev/prism/.factory/research/S-3.05/uncertainty-map.md`
- `/Users/jmagady/Dev/prism/.factory/research/S-3.06/uncertainty-map.md`
- `/Users/jmagady/Dev/prism/.factory/research/S-3.07/uncertainty-map.md`
- `/Users/jmagady/Dev/prism/.factory/research/S-3.08/uncertainty-map.md`
- `/Users/jmagady/Dev/prism/.factory/research/S-3.09/uncertainty-map.md`
- `/Users/jmagady/Dev/prism/.factory/research/S-3.10/uncertainty-map.md`
- `/Users/jmagady/Dev/prism/.factory/research/S-3.11/uncertainty-map.md`
- `/Users/jmagady/Dev/prism/.factory/research/S-3.12/uncertainty-map.md`
- `/Users/jmagady/Dev/prism/.factory/research/S-3.13/uncertainty-map.md`

