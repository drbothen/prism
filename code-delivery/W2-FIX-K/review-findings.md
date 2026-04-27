# W2-FIX-K Review Findings

**PR:** #71
**Branch:** fix/W2-FIX-K-token-id-audit-shape
**Merge SHA:** cf4fb34b
**Merged:** 2026-04-27T09:22:19Z

## Convergence Table

| Cycle | Reviewer | Findings | Blocking | Fixed | Verdict |
|-------|----------|----------|----------|-------|---------|
| 1 | pr-manager code review | 1 (CI fmt) | 0 (CI-gate, not blocking spec) | 1 | APPROVE |

**Cycles to converge:** 1

## Finding Log

### CI-FMT-001 (Non-blocking — CI gate)

- **Severity:** CI gate (non-blocking spec finding)
- **Location:** `crates/prism-audit/src/token_events.rs:132` and `:291`
- **Description:** Stable rustfmt 1.95.0 on CI required the `let mut detail_json = detail_to_json(...).map_err(...)` chains to break onto the next line. Local rustfmt accepted single-line form.
- **Fix:** Commit `451b9019` — reformatted both bindings to multi-line style.
- **Status:** RESOLVED

## Security Review

Verdict: CLEAN
- CWE-200: Resolved (privacy improvement — token_id removed from persisted bytes)
- No injection, no auth changes, no new attack surface
- `bincode` dev-dep only (not in production graph)

## Process Gap TD (from Pass 7)

**Filed as:** Recommended TD post-merge per dispatch
**Recommendation:** Extend `validate-consistency` skill with:
1. Tautology-detector: flag `test_BC_*` functions that don't call the corresponding `emit_*` function
2. BC-TV consistency check: parse canonical TV tables for field-level exclusion markers, cross-reference struct definitions

State-manager to file as formal TD entry.
