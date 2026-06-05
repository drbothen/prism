# LOCAL Adversary Pass 6 — S-DEMO-QUERY-PUSHDOWN-001

**Story:** S-DEMO-QUERY-PUSHDOWN-001 — prism-spec-engine: Thread QueryParams push-down into PipelineExecutor via FetchContext
**Pass:** LOCAL adversary pass 6
**Feature HEAD at pass (frozen):** `ed27b5ff`
**Feature HEAD after pass:** `ed27b5ff` (NO fix applied — lane PAUSED for human scope decision)
**Date:** 2026-06-05
**Lens:** Correctness — production call path from query plan through materialization to adapter
**Authority:** BC-5.39.001 D-779 | SAP-2 | CLAUDE.md Canonical Principle

---

## Verdict

**CLEAN(strict): no**
**CLEAN(PR-merge): no**
**Streak: 0/3**

2 CRITICAL (HIGH confidence) + 2 MED findings. Lane PAUSED for human scope decision.

**ROOT-CAUSE SYNTHESIS (deepest layer — pass 6 correctness angle):**

Pass 6 identified the single root cause that subsumes all sensor-level push-down inertia findings from pass 5:

`prism-query/src/materialization.rs` lines ~434–440 (the sole production callsite that builds `QueryParams` for `SpecDrivenSensorAdapter::fetch`) **hardcodes** `start_time: None`, `end_time: None`, `cursor: None`. A workspace-wide grep of `prism-query/src` for any non-`None` assignment to `start_time` or `end_time` returns **zero matches**.

This means ALL time-window and cursor translation code in `apply_push_down_to_request()` and `apply_push_down_to_json_body()` is unreachable dead code in production. The AC-002/003/004 time-window tests validate only a `FetchContext` constructed by the test harness directly — never the real `materialization.rs → adapter` path.

The per-sensor shape mismatches (F-PUSHDOWN5-CRIT-001 Cyberint, F-PUSHDOWN5-CRIT-002 Armis `maxResults`/`timeFrame`, F-PUSHDOWN5-CRIT-003 Claroty body limit) compound on top of this fundamental wiring gap: even if the shapes were correct, the values would never be set.

The fix for the root cause lives in `prism-query/src/materialization.rs` — which is **outside** this story's declared `crates_touched` (`[prism-spec-engine, prism-bin]`). This is why the lane is PAUSED for a human scope decision.

**NO FIX APPLIED.** Feature HEAD unchanged at `ed27b5ff`.

---

## Findings

### F-P6-CRIT-001 — CRITICAL — time-window dead code: materialization.rs hardcodes None

**Severity:** CRITICAL
**Confidence:** HIGH
**Category:** Unwired feature gap | production call path disconnected from implementation

**Description:**
`prism-query/src/materialization.rs` lines ~434–440 is the sole production callsite constructing `QueryParams` before calling `SpecDrivenSensorAdapter::fetch`. This code hardcodes:

```rust
QueryParams {
    start_time: None,
    end_time: None,
    cursor: None,
    // limit and filter are populated from the plan
}
```

Workspace grep: `grep -rn 'start_time\s*:' prism-query/src/` — the ONLY assignment sites use `None`. Same for `end_time` and `cursor`.

**Consequence:** Every time-window and cursor translation inside `SpecDrivenSensorAdapter` is unreachable in production. ACs 002/003/004 validate this code by constructing `FetchContext` directly in tests (bypassing `materialization.rs`). These are unit tests that prove the translation logic exists but NOT that it is ever invoked.

**Fix location:** `prism-query/src/materialization.rs` — **outside** this story's `crates_touched: [prism-spec-engine, prism-bin]`. This is the scope-decision trigger.

**Status:** OPEN — NO FIX APPLIED (lane paused for human scope decision D-1004)

---

### F-P6-CRIT-002 — CRITICAL — per-sensor shape mismatches (SAP-2 class, compounds on CRIT-001)

**Severity:** CRITICAL
**Confidence:** HIGH
**Category:** SAP-2 DTU↔TOML schema parity | silently wrong data

**Description:**
Same findings class as pass-5 F-PUSHDOWN5-CRIT-001/002/003 — the per-sensor translation targets are wrong even if `materialization.rs` were to be wired:

- **Armis `maxResults`/`timeFrame`:** `SearchQueryParams` in `prism-dtu-armis` has no such fields. SAP-2 §3 P1 CRITICAL class.
- **Claroty body `limit`:** `claroty.sensor.toml` `body_template: '{}'`; body injection is a no-op against the DTU.
- **Cyberint time-window:** `fetch_alerts` is `GET` with no POST body; body injection is unreachable for the production spec (F-PUSHDOWN5-CRIT-001).

These findings are confirmed at pass 6 under the correctness lens independent of pass 5.

**Status:** OPEN — NO FIX APPLIED (lane paused for human scope decision D-1004)

---

### F-P6-MED-001 — MED — AC-005 invariant tested only for limit, not time-window, not on production path

**Severity:** MED
**Confidence:** MED
**Category:** Test coverage gap | AC completeness

**Description:**
AC-005 `test_BC_2_11_007_push_down_result_equivalence_invariant` demonstrates result equivalence for the `limit` push-down only, against a single-step Wiremock mock. It does not:
1. Test time-window push-down result equivalence (which would also fail since `materialization.rs` hardcodes `None`)
2. Exercise the real two-step materialization → adapter production call path

**Status:** OPEN — NO FIX APPLIED (lane paused for human scope decision D-1004)

---

### F-P6-MED-002 — MED — PipelineExecutor::execute_step hardcodes is_first_step=true

**Severity:** MED (latent trap, not currently triggered)
**Confidence:** MED
**Category:** Latent correctness gap | production wiring

**Description:**
`crates/prism-query/src/pipeline.rs` line ~741: `PipelineExecutor::execute_step` hardcodes `is_first_step: true`. This is a latent trap if a hydration step ever uses this path. Parameterizing `is_first_step` based on step index would prevent silent misclassification.

This is secondary to F-P6-CRIT-001 but represents a correctness gap in the surrounding production path.

**Status:** OPEN — NO FIX APPLIED (lane paused for human scope decision D-1004)

---

## Summary Table

| Finding | Severity | Conf | Status |
|---------|----------|------|--------|
| F-P6-CRIT-001 | CRITICAL | HIGH | OPEN — paused |
| F-P6-CRIT-002 | CRITICAL | HIGH | OPEN — paused |
| F-P6-MED-001 | MED | MED | OPEN — paused |
| F-P6-MED-002 | MED | MED | OPEN — paused |

---

## Root-Cause Synthesis: Fixture-Level Paper-Test + Unwired Feature

Passes 1–4 gave a false convergence signal because:

1. **Test fixtures are fabricated.** `make_crowdstrike_like_spec`, `make_cyberint_like_spec`, `make_armis_like_spec` construct spec shapes that do not match the production `*.sensor.toml` files. The real Cyberint spec uses GET (no body template). The real Armis spec uses AQL passthrough, not `maxResults`/`timeFrame`. The real Claroty spec has `body_template: '{}'`.

2. **`materialization.rs` hardcodes `None` for time-window.** The production call path from the query plan through `PipelineExecutor::execute_step` → `QueryParams` construction → `SpecDrivenSensorAdapter::fetch` never supplies a non-`None` `start_time`, `end_time`, or `cursor`. The translation code is written but unreachable.

These two defects compound: even if the translation logic were correct per the fabricated fixtures, the production path would never invoke it.

**Only CrowdStrike push-down has verified real effect:** DTU `DetectionListParams` accepts `filter` + `limit` + `offset`. The test fixture shape happens to match (GET query params). However, the `start_time`/`end_time` → `filter` translation in `materialization.rs` is still hardcoded `None`, so CrowdStrike time-window push-down is also not wired in production — only the `limit` push-down is verified.

---

## Open Decision (D-1004) — Human Scope Decision Required

The fix to F-P6-CRIT-001 requires editing `prism-query/src/materialization.rs`, which is **outside** this story's `crates_touched: [prism-spec-engine, prism-bin]`.

Two options for human adjudication:

**Option A — Full wire into prism-query (expand scope):**
- Expand `crates_touched` to include `prism-query`
- Fix `materialization.rs` to read `start_time`, `end_time`, `cursor` from the query plan and populate `QueryParams`
- Fix per-sensor translation targets to match production DTU specs (Armis AQL passthrough per BC-2.11.007; Cyberint GET-compatible; Claroty deferred per S-DEMO-CLAROTY-PAGINATION-001)
- Fix BC-2.01.013 per-sensor table false "implemented" claims (F-PUSHDOWN5-HIGH-001)
- All 5 AC tests become load-bearing against the real production path
- Scope expands by ~1 session; architect confirmation of `materialization.rs` wiring approach needed

**Option B — Narrow to limit-only, defer time-window:**
- Explicitly scope the story to `limit` push-down ONLY (the only verified real-effect push-down)
- Remove or qualify AC-002/003/004 time-window ACs as "future scope" with explicit anchoring to a new story (e.g., S-DEMO-QUERY-TIMEWINDOW-001)
- Fix BC-2.01.013 per-sensor table to remove false "implemented" claims for time-window
- Fix Armis translation to use AQL passthrough (removes `maxResults`/`timeFrame` confusion) per BC-2.11.007
- `materialization.rs` wiring for time-window deferred to the new story with explicit story anchor
- Scope stays within `crates_touched: [prism-spec-engine, prism-bin]` (Armis AQL fix is in spec-engine)
- Faster path; demo-goal depends on whether time-window push-down is required for the Wave 5 demo

**Canonical Principle Rule 3 exception applies:** The two scope options both require either architect approval (Option A — scope expansion requiring `prism-query` architecture change) or product-owner direction (Option B — AC scope reduction). This is a D-989 exception (2): "genuine product/scope decision + scope-expansion-requiring-architecture."

**Lane status:** PAUSED_PENDING_HUMAN_SCOPE_DECISION at D-1004.

---

## Convergence Trajectory (through pass 6)

| Pass | Feature HEAD | Findings | Delta | CLEAN(strict) | CLEAN(PR-merge) | Streak |
|------|-------------|----------|-------|--------------|-----------------|--------|
| 1 | 19184786→a75fada4 | 8 (2H+2M+2L+2OBS) | — | no | no | 0/3 |
| 2 | a75fada4→688f82b5 | 3 (1M+1L+1OBS) | -5 | no | no | 0/3 |
| 3 | 688f82b5→ed27b5ff | 1 (1 LOW) | -2 | no | yes | 0/3 |
| 4 | ed27b5ff | 0 (fixture-only lens) | -1 | yes (limited lens) | yes | **1/3** |
| 5 | ed27b5ff (frozen) | 5 (3C+1H+1M) | +5 REGRESSION | no | no | **0/3 RESET** |
| 6 | ed27b5ff (frozen) | 4 (2C+2M) | -1 | no | no | 0/3 |

**Note:** Pass 4 CLEAN was a false signal (fixture-only lens; fabricated fixtures hide production shape mismatches + `materialization.rs` wiring gap). Passes 5 and 6 independently confirmed the same root cause using SAP-2 extended (fabricated-fixture parity) and production call-path tracing respectively.
