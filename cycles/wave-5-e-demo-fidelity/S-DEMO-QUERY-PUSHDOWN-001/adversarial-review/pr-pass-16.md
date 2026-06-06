---
document_type: adversarial-review-report
scope: PR-LEVEL
story_id: S-DEMO-QUERY-PUSHDOWN-001
pr_number: 173
pass_number: 16
cascade: PR-LEVEL (distinct from LOCAL; LOCAL converged at pass 11 @69aafcc7)
base_develop: "752e407a"
feature_head_at_review: "6835e4fa"
feature_head_after_fix_burst: "6835e4fa"
clean_strict: false
clean_pr_merge: true
streak_after: "0/3"
produced: 2026-06-06
authority: BC-5.39.001 D-779
---

# PR-LEVEL Adversary Pass 16 — S-DEMO-QUERY-PUSHDOWN-001

**Story:** S-DEMO-QUERY-PUSHDOWN-001 — Push-Down Query Fidelity (Phase B Lane 2)
**PR:** #173 (base develop@752e407a, head 6835e4fa at review)
**Pass:** PR-LEVEL pass 16
**Date:** 2026-06-06

## Pass-15 Closure Verification

Pass-15 was CLEAN(strict)=yes at HEAD 6835e4fa; zero findings. Streak 0/3 → 1/3.
All recurring hygiene classes (dangling-AC, draft-comment, vacuous-assertion,
evidence-SHA) confirmed closed and durable.

## Adversary Pass 16 Findings

### F-P16-LOW-001 (LOW) — 5 stale volatile line-number pins in STORY narrative (TD-VSDD-091)

**Finding ID:** F-P16-LOW-001
**Severity:** LOW
**Category:** Spec artifact / TD-VSDD-091 anti-volatile-pin

**Description:** The story narrative (S-DEMO-QUERY-PUSHDOWN-001 story file) contained
5 volatile line-number pins citing `materialization.rs ~434-440` (or equivalent
`~NNN` approximate-line annotations) in the story body's §Implementation Notes or
§Tasks section. These pins reference line numbers in the production implementation
that were correct at the time of initial authorship but will decay on subsequent
diffs — any refactor touching materialization.rs will silently invalidate them.

TD-VSDD-091 (anti-volatile-pin discipline) prohibits line-number pins in narrative
spec content. Justified citations (Red Gate test tables, AC source-of-truth tables,
pass-report changelogs) are excepted, but implementation-note `~NNN` line references
in story narrative are exactly the pattern TD-VSDD-091 prohibits.

**CLEAN(strict):** no (1 LOW finding)
**CLEAN(PR-merge):** yes (no CRIT/HIGH/MED)

**Root cause:** Story narrative was authored with `~NNN` line annotations as
navigational aids during initial story writing. The TD-VSDD-091 sweep at spec-
crystallization time did not cover story narrative body text (sweep focused on
.factory/specs/ artifacts). Story body pins persisted through 15 PR-LEVEL passes
until this fresh-context pass caught them via holistic spec-artifact sweep.

**Closure:** CLOSED by story-writer. COMPLETE SWEEP of all story body content for
`~\d+` and `:\d+` pattern line pins:
- 5 volatile `~434-440` (and similar) pins in §Implementation Notes anchored to
  function names and section identifiers (e.g., `build_pushdown_context()` in
  `materialization.rs`, `predicate_tree_to_filter_map()` in `pipeline.rs`).
- Zero residual line-number pins after sweep (grep-verified).
- Story v2.7 → v2.8 (changelog entry added per POL-32).
- STORY-INDEX Full Story List row updated: v2.7 → v2.8.
- STORY-INDEX v2.289 → v2.290.
- Feature code HEAD UNCHANGED at 6835e4fa (spec-only fix; no code change).

## Axes Checked

| Axis | Result | Notes |
|------|--------|-------|
| Correctness | PASS | All correctness findings remain closed |
| Story volatile line-pin sweep (TD-VSDD-091) | FAIL → FIXED | F-P16-LOW-001: 5 `~NNN` pins in §Impl Notes → function-name anchors; story v2.7→v2.8; STORY-INDEX v2.289→v2.290 |
| SAP-1 (catalog) | PASS | 71 rows; no unregistered event_type |
| SAP-2 (DTU↔TOML) | PASS | CrowdStrike + Armis confirmed |
| AC traceability (SAP-5) | PASS | ZERO dangling ACs |
| Draft-comment class | PASS | Confirmed closed (pass 11) |
| Evidence SHA class | PASS | Confirmed closed (pass 14 de-pin) |
| Vacuous-assertion class | PASS | Confirmed closed (pass 13) |
| POLICY 32 (BC-2.16.002) | PASS | Monotonic; no duplicates |
| Security | PASS (CLEAR-TO-MERGE) | No new security surface |

## Summary

**CLEAN(strict):** no (1 LOW F-P16-LOW-001 — 5 volatile line-pins in story narrative)
**CLEAN(PR-merge):** yes (no CRIT/HIGH/MED)
**Streak:** 0/3 (RESET — finding resets streak from 1/3 → 0/3 per BC-5.39.001 D-779)
**Feature HEAD:** 6835e4fa (UNCHANGED — spec-only fix)
**Story version:** v2.7 → v2.8 (story-writer; TD-VSDD-091 line-pin sweep; changelog added)
**STORY-INDEX version:** v2.289 → v2.290
**Next step:** PR-LEVEL pass 17 (fresh streak on 6835e4fa + story v2.8; need 3 strict-clean)

## Observation — Strict-3-CLEAN Tail Pattern (Codification Candidate)

Passes 10-16 demonstrate a recurring pattern in the PR-LEVEL cascade tail:

After substantive code convergence (all correctness, security, and spec-semantics
findings closed), fresh-context adversary passes surface a continuing tail of single
cosmetic spec/test-hygiene LOWs. Each fix gives the next reviewer new surface (e.g.,
the pass-13 evidence refresh introduced a new volatile SHA pin caught at pass-14).

Observable trajectory for this story's PR-LEVEL cascade:
- Passes 1-9: substantive findings (MEDs, security, dangling-AC class x2)
- CLEAN(PR-merge)=yes held on ALL 16 passes
- CLEAN(strict)=yes achieved only at passes 10, 12, 15

The strict-3-CLEAN requirement caught 4 LOW-class hygiene items across passes 10-16.
While each individual fix is correct and justified, the pattern suggests a systematic
opportunity: a pre-cascade hygiene-sweep gate (covering line-pins, draft-comments,
vacuous-asserts, dangling-ACs, volatile-SHAs) run ONCE before the PR-LEVEL cascade
begins could drain the cosmetic tail upfront — allowing the cascade to focus on
substantive behavioral review from pass 1. This is recorded as a codification
candidate in lessons.md.

[process-gap][codification-candidate] Tagged for session-reviewer.
