---
document_type: po-adjudication
finding_id: F-LP8-MED-001
burst: D-866
date: 2026-05-30
author: product-owner
bc_affected: BC-2.01.017
bc_version_before: "1.3"
bc_version_after: "1.4"
bc_index_version_before: "5.59"
bc_index_version_after: "5.60"
semantic_change: false
---

# PO Adjudication — F-LP8-MED-001

## Finding Statement

Pass 8 LOCAL adversary identified that `.factory/specs/behavioral-contracts/BC-2.01.017-static-cookie-auth-provider-no-login-roundtrip.md` Changelog table contains two byte-identical v1.2 rows with chronological ordering broken: 1.3 → 1.2 → 1.0 → 1.2(dup) → 1.1.

## Investigation

Read BC-2.01.017 lines 230–239. Confirmed:

- Line 234: v1.3 row (D-857 F-LP3-HIGH-001) — canonical, correct
- Line 235: v1.2 row (D-854) — canonical, correct
- Line 236: v1.0 row (D-849) — out of order (should be last)
- Line 237: v1.2 row (D-854) — byte-identical duplicate of line 235
- Line 238: v1.1 row (D-852) — correct content, wrong position

The duplicate was introduced during the D-857 v1.3 burst: when the v1.3 row was prepended at line 234, the v1.0 row was displaced below the v1.2 row, and the original v1.2 position was retained alongside a re-inserted copy. The result was: 1.3 → 1.2 → 1.0 → 1.2(dup) → 1.1, breaking both uniqueness and monotonic descending order.

No semantic content in any row is incorrect. This is a structural/ordering defect only.

## Decision

1. Delete the duplicate v1.2 row (was at line 237 — byte-identical to canonical v1.2 row at line 235).
2. Reorder remaining rows to monotonic descending: 1.4 → 1.3 → 1.2 → 1.1 → 1.0.
3. Add v1.4 row documenting this cleanup burst.
4. Bump BC version 1.3 → 1.4 (frontmatter).
5. Bump BC-INDEX version 5.59 → 5.60 (frontmatter + index row + changelog entry).

No semantic content changed. No implementer follow-on required. No story spec update required (story already references BC-2.01.017 by ID without version pin per POL-29; v1.4 is a hygiene-only bump that does not affect AC or implementation scope).

## Self-Audit Checklist (CLAUDE.md §Self-Audit Checklist)

- [x] Did I rationalize any decision with "MVP," "for now," "good enough," or "we can fix later"? — NO. Deleted duplicate, reordered, bumped version. Complete fix.
- [x] Did I add a new tech-debt-register entry without all three: explicit human direction, concrete future dependency, and a specific future story/wave anchor? — NO. No TD entry created.
- [x] Did I leave any "pending architect review," "TODO for architect," or "Placeholder for architect" in a spec artifact for a question I could have answered in scope? — NO.
- [x] Did I find a bug or gap in another AI's output and surface it as a question/advisory instead of fixing it in scope? — NO. Fixed in scope.
- [x] Did I default to the cheapest mechanism instead of the correct mechanism? — NO. Correct fix: delete dup, reorder, bump version, add v1.4 row, update BC-INDEX.
- [x] If I added an ADVISORY-severity finding to a report, did I evaluate whether it should be a BLOCKER under the production-grade lens? — N/A. This IS the fix.
- [x] Did I paper-fix a finding by renaming, doc-commenting, or asserting-only when the real fix is structural? — NO. Structural fix: table rows deleted and reordered.
- [x] Did I sibling-sweep all callsites when I changed a function signature, constant, or canonical identifier? — N/A. Changelog hygiene only; no identifier change. BC-INDEX row updated (the one downstream reference to BC-2.01.017 version status).
