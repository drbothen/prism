# Lessons Learned — wave-5-e-demo-fidelity

## Codification Candidates

### [recurrence] OCSF-CLASS-MIGRATION-001 cascade: cite-pin sweeps must be story-wide exhaustive, not incremental

**Date recorded:** 2026-06-06
**D-NNN anchor:** D-1032 (cycle-close discipline S-7.02; justified-deferred codification follow-up — not blocking OCSF merge)
**Story:** OCSF-CLASS-MIGRATION-001
**Tags:** [recurrence] [cite-pin-sweep] [sibling-sweep] [TD-VSDD-060] [streak-reset]
**Classification:** PROCESS-GAP — recurring cite-pin/sibling-sweep-miss class caused 2 streak resets in the same story's LOCAL cascade.

**Description:**

The OCSF-CLASS-MIGRATION-001 LOCAL cascade experienced 2 streak resets from the same root cause class:

- **Reset 1 at pass-6 (F-LP6-HIGH-001 HIGH):** Story-writer advanced BC-2.02.012 cite pins in an earlier burst (D-1026) by scanning for certain patterns. The sweep missed the `subsystems:` frontmatter field and justification comment bodies that referenced `SS-16` instead of the canonical `SS-02` (prism-ocsf is OCSF Normalization, not Spec Engine). Four sites were missed. Streak reset 2/3→0/3.

- **Reset 2 at pass-8 (F-LP8-MED-001 MED):** Story-writer advanced the BC-2.02.012 cite pin from v1.5→v1.6 in D-1026 by targeting the forward-looking cite sites. The trailing summary sentence `"BC is now v1.5"` in the frontmatter comment block (line 31) was not reached by the targeted sweep pattern. Streak reset 1/3→0/3.

Both resets involved the same root cause: the story-writer ran a **targeted/incremental cite-advance** (searching for specific known-stale patterns) rather than a **story-wide exhaustive sweep** (reading every occurrence of the BC name/version across the entire story file and adjacent spec files).

**Root cause of recurrence:**

Targeted sweeps (grep-and-replace on known patterns) are fast but incomplete. They find the sites the sweeper expected to exist but miss unexpected sites (comment bodies, trailing summary sentences, frontmatter fields) that hold the same staleness. The pass-8 fix-burst's response was correct: it performed a **full 35+ site version-pin audit** (22 BC-2.02.012 sites + 10 BC-2.01.013 sites), confirmed all present-tense pins, and explicitly distinguished historical hop narratives (immutable per TD-VSDD-091) from present-tense claims. Zero stale present-tense pins remained after the exhaustive sweep.

**Correct response (codified rule):**

When a story-writer performs a cite-pin advance (bumping a BC version reference in a story file), the sweep MUST:

1. Use a **story-wide exhaustive read**: read the full story file top-to-bottom, cataloging every occurrence of the BC name (e.g., `BC-2.02.012`) with its adjacent version string.
2. **Distinguish present-tense citations** (must be advanced) from **historical hop narratives** (immutable per TD-VSDD-091 — do not advance).
3. **Explicitly count and confirm**: record the total sites examined and the total sites advanced. e.g., "22 BC-2.02.012 present-tense pins confirmed v1.6; 10 BC-2.01.013 present-tense pins confirmed v1.14; 0 stale."
4. **Cross-check adjacent documents** in the story's perimeter (frontmatter comment blocks, justification comments, §Architecture Mapping tables, §Scope references) — these are frequently missed by pattern-search sweeps.
5. **Do not use grep-only as the primary sweep mechanism** for cite-pin advances when the story file is large or has multi-section structure. Read the file; grep is a verification step, not the sweep.

**Outcome:**

The pass-8 fix-burst (complete 35+ site audit) closed the recurring class. Passes 9/10/11 were all CLEAN(strict)=yes. LOCAL cascade CONVERGED.

**Codification direction (for future session-reviewer / VSDD process improvement):**

- This pattern should become a standing story-writer discipline: after any BC version bump, perform a full story read (not grep-only) and record the exhaustive count in the fix-burst commit message.
- Adversary SAP probe extension: after a cite-pin advance, adversary MUST grep the full story file for ALL versions of the BC name (including older version strings) and verify none remain as present-tense claims.
- Consider adding a POL amendment to POL-32 or a new process rule: "cite-pin sweeps must be story-wide exhaustive with explicit count record."

**Note:** This is a justified-deferred codification follow-up per Canonical Principle Rule 3. The codification work (SAP probe extension, possible POL amendment) requires session-reviewer adjudication. It does NOT block the OCSF-CLASS-MIGRATION-001 PR cycle.
