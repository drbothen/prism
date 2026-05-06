---
document_type: adversarial-review-pass
pass_number: 2
pr_number: 130
story_id: S-3.06
branch_sha: 84c65574
verdict: BLOCKED
convergence_window: 0/3
reviewer: adversary
timestamp: 2026-05-06
producer: adversary
inputs:
  - .factory/stories/S-3.06-prismql-write-parser.md
  - .factory/specs/behavioral-contracts/BC-2.11.006-query-security-limits.md
  - .factory/specs/behavioral-contracts/BC-2.11.004-prismql-pipe-mode.md
  - .factory/code-delivery/S-3.06/pr-description.md
input-hash: "[live-adv-review pass-2]"
traces_to: PR-130
---

# PR #130 Adversarial Pass-2 — BLOCKED (Convergence Reset)

## Verdict: BLOCKED — 1 HIGH + 2 MEDIUM + 6 OBS (3 KUDOs + 1 process-gap)

Convergence window: **0 / 3** clean passes. Severity decay positive (pass-1: 15 findings → pass-2: 9 findings, all documentation).

The pass-1 → pass-2 fix bundle correctly closed 7 of 8 findings (HIGH-001, HIGH-002, MED-001, MED-002, MED-003, LOW-002, LOW-003, OBS-2). LOW-001 was only partially closed — 1 of 3 affected lines fixed. This recurrence of partial-fix-regression in factory-artifacts blocks convergence.

## High Findings

### F-PR130-P2-HIGH-001 — Pass-1 LOW-001 fix is partially closed
- **Where:** `.factory/code-delivery/S-3.06/pr-description.md:74-75`
- **What:** Pass-1 instructed replacing `BC010[BC-2.11.010]` with `BC004` for AC-3/AC-7/AC-8. Commit `36f18fae` updated only line 73 (AC-3); lines 74-75 (AC-7, AC-8) still cite `BC010`.
- **Why fails:** Mermaid auto-creates undefined nodes; rendered graph shows AC-7/AC-8 originating from a phantom `BC010`. Violates POL-04 (semantic_anchoring_integrity) and POL-08 (BC array changes propagate).
- **Fix:** Replace `BC010` at lines 74, 75 with `BC004`.

## Medium Findings

### F-PR130-P2-MED-001 — PR description BC version + symbol count + E-error count are stale
- **Where:** `.factory/code-delivery/S-3.06/pr-description.md:6,20,209`
- **What:** Stale references to BC-2.11.006 v1.11 / 9 new symbols / 27 expected E-errors. Per BC v1.14, current values are 10 symbols / 28 E-errors.
- **Fix:** Update line 6, 20, 209 to v1.14 / 10 symbols / 28 E-errors.

### F-PR130-P2-MED-002 — BC-2.11.006 v1.14 changelog burst tag references wrong story
- **Where:** `.factory/specs/behavioral-contracts/BC-2.11.006-query-security-limits.md:164`
- **What:** v1.14 changelog row tag is `S-3.02-pr130-pass1` but PR #130 is for S-3.06.
- **Fix:** Change to `S-3.06-pr130-pass1`.

## Observations

- **OBS-1:** MED-001 regression test verifies signature change but doesn't exercise actual race window. Structural code change is sufficient; no action.
- **OBS-2 [process-gap]:** Recurrence of partial-fix-regression — 1 of 3 lines fixed. Recommend orchestrator add explicit `grep <old-string>` check before declaring LOW closure.
- **OBS-3:** BC v1.14 frontmatter places parse_sql_dml_with_limits between sql_parser and filter_parser blocks. Cosmetic only.
- **KUDO OBS-4:** HIGH-001 fix (`check_denied_keywords` shared helper) — exemplary single source of truth.
- **KUDO OBS-5:** MED-002 fix (`visit_dml_node`) — well-shaped Visitor extension.
- **KUDO OBS-6:** OBS-2 closure (api_surface.rs) — added denylist parity test cross-checking HIGH-001 closure at integration layer.

## Pass-1 → Pass-2 Closure Matrix

| Finding | Status |
|---------|--------|
| F-PR130-P1-HIGH-001 (denylist) | CLOSED |
| F-PR130-P1-HIGH-002 (perimeter symbol) | CLOSED |
| F-PR130-P1-MED-001 (parse_pipe_with_write race) | CLOSED |
| F-PR130-P1-MED-002 (Visitor DML) | CLOSED |
| F-PR130-P1-MED-003 (test coverage) | CLOSED |
| F-PR130-P1-LOW-001 (Mermaid traceability) | PARTIAL → re-opened as P2-HIGH-001 |
| F-PR130-P1-LOW-002 (write stage in count) | CLOSED |
| F-PR130-P1-LOW-003 (bare-verb doc) | CLOSED |
| F-PR130-P1-OBS-2 (api_surface.rs) | CLOSED |

7 closed / 1 partial-then-re-opened-as-HIGH / 0 still open from original list.

## 7-Lens Verification Matrix

| Lens | Status |
|------|--------|
| 1. Fix-bundle validation | PASS with caveats (LOW-001 partial closure) |
| 2. BC-2.11.004 v1.4 invariants | PASS |
| 3. BC-2.11.006 v1.14 perimeter integrity | PASS (27 symbols, 28 E-errors) |
| 4. AST evolution safety | PASS |
| 5. Cross-cutting concerns | PASS |
| 6. Story↔AC↔Test traceability | FAIL (HIGH-001 Mermaid) |
| 7. Documentation drift | FAIL (MED-001 + MED-002) |

## Convergence Window State

**0 / 3** clean passes. Pass-1 BLOCKED. Pass-2 BLOCKED (1 HIGH + 2 MEDIUM). Window does NOT advance.

## Process-Gap Findings

- **OBS-2 [process-gap]:** Recurrence of partial-fix-regression. Recommend grep-sweep before LOW-finding closure.

## Novelty Assessment

**Novelty: MEDIUM.** All 3 new findings are genuinely new (partial-closure regression of pass-1 LOW-001 + downstream propagation drift + typo). Severity decay is positive — fewer and less severe findings, all documentation.

**Required fixes before pass-3:**
1. Lines 74-75 in pr-description.md (BC010 → BC004)
2. Lines 6, 20, 209 in pr-description.md (v1.11 → v1.14, 9 → 10, 27 → 28)
3. BC-2.11.006 v1.14 changelog burst tag (S-3.02 → S-3.06)
