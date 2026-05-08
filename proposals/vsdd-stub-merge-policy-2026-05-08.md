# Proposal: Stub-Merge Detection Policy for VSDD

**Status:** Draft
**Date:** 2026-05-08
**Source:** Discovered via plugin-system audit at downstream Prism project
**Apply to:** vsdd-factory plugin repository (or equivalent VSDD methodology codebase)

## How to Use This Proposal

This file is intentionally self-contained — paste it as the initial prompt to a fresh Claude session in the vsdd-factory repository. The prompt assumes no prior context; the receiving session will read your repo and implement the proposal.

---

## Problem Statement

VSDD currently allows stories to ship as "stub-phase" scaffolding — production code with `todo!()` / `unimplemented!()` bodies that downstream stories are expected to fill in. The `status: merged` frontmatter value is set when the PR lands, regardless of whether the implementation is real or scaffolding.

This creates a structural failure mode: stub-phase stories ship under `status: merged` with no graduation contract requiring downstream completion. Stubs persist indefinitely while their parent story shows as "merged".

A real-world audit at one VSDD-managed project (Prism, MSSP MCP server) discovered:

- **3 of 5 plugin-axis stories shipped as stub-merges**:
  - S-1.12 (hot-reload watcher): `HotReloadWatcher::start/stop` are `unimplemented!()` in production
  - S-1.14 (infusion framework): 100% `unimplemented!()` across 14 method bodies
  - S-1.15 (WASM plugin runtime): action-plugin dispatch (`fire_alert`/`fire_case`/`fire_report`) stubbed
- **Documentation in code claimed deliverables that were not implemented** (e.g. `// All method bodies are unimplemented!(). Implementation lives in S-1.14.` while S-1.14 status was merged)
- **14 P0/P1 deferrals were hidden behind merged status**
- A downstream story (S-3.09) attempting to consume the stubs hit the gap during integration testing months later — only then was the architectural debt surfaced
- The factory's adversarial review process did not catch any of this — adversary saw "all green tests" because the test surface was silent-shallow (tests verified scaffolding, not production behavior)

The methodology gap: VSDD has no enforcement that stub-phase scaffolding eventually graduates to real implementation, and no signal that distinguishes a "fully-delivered merged story" from a "scaffolding-only merged story".

## Proposed Fix

Implement five changes in vsdd-factory:

### 1. New status enum value: `partial-merge`

Extend the legitimate set of frontmatter `status:` values for stories:

```yaml
status: draft           # spec under development
status: ready           # spec frozen, ready for implementation
status: in-progress     # implementation underway
status: merged          # production code real; ZERO production-path stubs
status: partial-merge   # NEW — PR landed for spec/scaffolding; production stubs remain
status: deprecated      # superseded
status: retired         # removed
```

Update `frontmatter-validator` agent to recognize `partial-merge` as legitimate.

### 2. Hard rule: `status: merged` forbids production stub residue

A story can transition to `status: merged` ONLY IF all conditions hold:

- Every claimed delivery file (per the story's `File Structure Requirements` section / claimed BC delivery surface) contains zero `todo!()` / `unimplemented!()` / `panic!("not yet")` / `panic!("TODO")` outside `#[cfg(test)]` blocks
- No production code path that exercises the story's BCs panics with stub markers
- Every claimed BC has at least one integration test that invokes the BC's named SUT (not a hand-built struct shortcut)

Stories with production stubs MUST use `status: partial-merge` AND link a graduation contract (see #3).

Implementation: extend `consistency-validator` agent prompt to fail on `status: merged` stories where `rg 'todo!\(\)|unimplemented!\(\)|panic!\("not yet|panic!\("TODO' <delivery-files>` returns non-zero matches outside `#[cfg(test)]`.

### 3. Graduation contract requirement

For every `status: partial-merge` story, the spec must include a `## Graduation Contract` section listing:

- Which downstream story is expected to fill in each stub (must reference a real story ID, not "TBD")
- File:line of each stub
- BC postcondition currently unmet
- Estimated graduation timeline (e.g., "by S-X.YY merge")

The orchestrator must track partial-merge stories in a top-level `STUB-DEBT-INDEX.md` separate from `STORY-INDEX.md`. Each row: `partial-merge story | claimed-BCs-unmet | graduation-target-story | status (open/closing/graduated)`.

### 4. Adversary policy update — production-stub-residue check at merge gate

At every merge gate, the adversary must run a "production-stub-residue" check against the story's claimed delivery files:

```bash
# For each file in story.file_structure_requirements:
for file in $delivery_files; do
  rg -n 'todo!\(\)|unimplemented!\(\)|panic!\("not yet|panic!\("TODO' "$file" \
    | grep -v '#\[cfg(test)\]' \
    | grep -v 'mod tests' \
    || echo "OK: $file"
done
```

If any stubs remain, adversary blocks the `status: merged` transition. Story must either:
- (a) Implement the stubs (graduate to `merged`)
- (b) Reclassify to `status: partial-merge` with graduation contract

Add this check explicitly to the adversary agent prompt as a mandatory verification step (similar to existing POL-1..POL-11 policy rubric).

### 5. Periodic stub-debt sweep skill

New orchestrator skill `/vsdd-factory:audit-stub-debt` that scans:

- All `status: merged` stories' delivery files for stub residue (catch retroactive drift via untracked downstream edits)
- All `status: partial-merge` stories for graduation contract completeness
- All TOML / YAML / config files declared in delivery sections but not consumed by any production code path (orphan-config detection)
- All claimed BC postconditions verified to be tested at integration level (catch silent-shallow drift)

Output: structured findings report with file:line citations. Findings present to user; do NOT auto-file TDs (defer-without-confirmation is a separate methodology anti-pattern).

## Acceptance Criteria

- New `status: partial-merge` is a legitimate frontmatter value; `frontmatter-validator` agent recognizes it
- `consistency-validator` agent fails on `status: merged` stories with detected production stub residue (test against synthetic stub-merge fixture)
- Adversary agent prompt includes the production-stub-residue check at every merge gate (POL-12 or new policy ID)
- New `/vsdd-factory:audit-stub-debt` skill is implemented and produces structured findings report
- `STORY-INDEX.md` schema supports both `merged` and `partial-merge` with graduation contract reference
- New `STUB-DEBT-INDEX.md` is initialized for projects to track partial-merge debt
- The change is documented in `VSDD.md` (or equivalent methodology spec) under a new "Stub-Merge Detection" section
- Backwards compatibility: existing projects with `status: merged` stories that have stub residue are flagged on first audit run but not auto-reclassified — they require a maintenance burst to either implement stubs or reclassify

## Implementation Estimate

1-3 days of focused VSDD methodology PR. Treat as P0 priority — without it, future projects will recur the same pattern.

## Reference

Source: 2026-05-08 plugin-system audit at downstream Prism project (`/Users/jmagady/Dev/prism/.factory/cycles/wave-4-operations/plugin-system-audit-2026-05-08.md`). Audit cited 14 P0/P1 deferrals hidden behind `status: merged`. The "stub-phase convention" comment pattern (e.g., `// Stub module: all non-trivial bodies are todo!() pending implementation`) was a project-local workaround for this methodology gap.

Also reference: TD-S307-002 (test-name↔code coherence) and TD-S307-003 (catalog↔impl Display coherence) at Prism — silent-shallow test patterns that allowed stub-merge to slip past adversary review.

## Adjacent Methodology Concerns (Out of Scope for This Proposal)

These were also surfaced in the same audit but warrant separate proposals:

- **No-defer-without-confirmation policy**: orchestrator should not auto-defer adversary findings, process-gap codifications, or architectural findings to "next maintenance burst" without explicit user approval
- **Test-quality enforcement**: AC tests must invoke the AC's named SUT (not hand-built struct shortcuts) — codify TD-S307-002 cluster as POL-N
- **Silent-shallow detection at merge gate**: extend production-stub-residue check to also detect AC tests whose body manually constructs the struct under test instead of invoking the production SUT
- **Adapter ↔ DTU contract drift detection**: for projects with DTU clones, periodically verify adapter URLs match DTU routes (caught at Prism after stories had already merged)
