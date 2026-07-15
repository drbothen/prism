---
document_type: stage-transition
defect: DEFECT-PQL-FNCALL-LHS-001
event: PR-OPEN
date: 2026-07-14
decision_id: D-1775
---
# DEFECT-PQL-FNCALL-LHS-001 — Push + PR #223 Created (D-1775)

**Stage transition: LOCAL-ONLY CONVERGED → PUSHED + PR OPEN**

## Branch

- Branch: `fix/DEFECT-PQL-FNCALL-LHS-001`
- HEAD: `973aedcf` (first push to origin; 39 commits ahead of develop `@5f1b5771`)
- Pre-push hook: PASSED (68.41s)
- LOCAL-ONLY status: ENDED as of this push

## PR

- PR #223: https://github.com/drbothen/prism/pull/223
- Base: `develop`
- Files changed: 13; 7490 insertions / 79 deletions

## CI Status (at PR open)

- 42/43 checks PASS
- `x86_64-unknown-linux-gnu` run 2 FAILED on transient runner disk exhaustion (not code; identical job run 1 passed)
- Re-run in flight: run `87265140790`

## Security Review

- APPROVE — 0 CRIT/HIGH/MED
- 2 LOW non-blocking:
  - SEC-001: bidi override chars not stripped in `sanitize_for_log` — acceptable per threat model
  - SEC-002: server-controlled `ColumnNotFoundDetails` Display fields — intentional, requires admin TOML access to set

## pr-manager 9-Step Status

| Step | Status |
|------|--------|
| 1–4, 6, 7 | DONE |
| 5 (review-convergence) | DEFERRED → PR-LEVEL cascade |
| 8–9 | HALT (human-gated) |

## Merge Gate Asks (carried on PR)

1. DRIFT-PQLFN-OD7 Gap-1 (E-QUERY-038 DML fail-open → S-3.07) — human ratification required
2. DRIFT-PQLFN-OD7 Gap-2 (source_select projections/JOIN/HAVING ungated → S-3.07) — human ratification required
3. BC-2.11.019 cross-branch sequencing: POL-14 auto-promotion fires on PR #223 merge (not PR #222)

## Human Sequencing Decision (2026-07-14)

PR #222 (DEFECT-MCP-ROWSHAPE-NULLS-001, CONVERGED 3/3) WAITS for PR #223.
Rationale: BC-2.11.019 injection-safety fix rides this branch; MCP disclosure item 1 eliminated once #223 merges first.

## Next Step

PR-LEVEL adversarial cascade on frozen pushed HEAD `973aedcf`.
- Fresh streak: 0/3 (BC-5.39.001)
- DRIFT-ORCH-PRLEVEL-PUSH-001: no pushes mid-cascade
- Handoff items: see `adversarial-review/local-pass-49.md §CONVERGENCE`

---

## PR Body Append — 2026-07-15 (pass-7)

Section appended to PR #223 body after PR-LEVEL adversarial pass-7 surfaced F-PQLFN-PR7-OBS-001.
Appended to the `## MERGE-GATE DISCLOSURES` section; corresponding pre-merge checklist row added.

### Merge-Gate Disclosure — LOW-006 DataFusion-name collision cost (F-PQLFN-PR7-OBS-001)

- The LOW-006 reserved-keyword gate (BC-2.11.004 v1.48, 21 keywords) rejects PrismQL operator keywords used as function-call LHS in predicates (E-QUERY-001 parse-time). This is a ratified, documented scope limit.
- Cost surfaced by PR-LEVEL adversarial pass-7: the list includes `CONTAINS` / `STARTSWITH` / `ENDSWITH` / `ICONTAINS` / `ISTARTSWITH` / `IENDSWITH`. Of these, `contains` collides with a same-spelled DataFusion 53.1 scalar function — so `WHERE contains(hostname, 'malware') = TRUE` is rejected at parse time even though `contains(...)` works in SELECT projection position. The underscore-spelled DataFusion variants (`starts_with`, `ends_with`) are NOT in the keyword list and remain usable as predicate-LHS fn-calls.
- This is NOT a regression (pre-branch, all fn-call LHS predicates failed with a generic parse error) and is fail-safe (clear keyword-rejection message).
- **HUMAN FEATURE-DECISION requested at merge gate** (per BC-2.11.004 EC-11-004-006 "surface in PR body for human feature-decision"): keep the 21-keyword list as-is (recommended — conservative, message is clear), or open a follow-up story to re-adjudicate the exclusion list (e.g., permit `contains`/`startswith`/`endswith` as fn-call LHS since the parenthesized form is unambiguous vs the operator form). No code change rides this PR either way.

Pre-merge checklist row added:
`- [ ] MERGE-GATE FEATURE-DECISION: LOW-006 keyword list adjudication — keep 21-keyword list as-is or open follow-up story (F-PQLFN-PR7-OBS-001)`
