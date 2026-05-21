---
document_type: fix-burst-closure
story_id: PLUGIN-MIGRATION-001-D
pass_number: 17
closure_date: 2026-05-20
findings_total: 4
findings_closed: 4
findings_deferred: 0
decision_id: D-751
---

# Fix-Burst-17 Closure Record — PLUGIN-MIGRATION-001-D

## Per-Finding Closures

### F-LP17-HIGH-001 — STORY-INDEX row 399 intra-cell version-pin asymmetry (11th coherence-axis)

**Finding:** STORY-INDEX row 399 header `**draft** v1.8` was stale while embedded BC pin had already advanced to `BC-2.16.013(v1.9)`. FB-IMPL-P16-PO swept embedded BC pin but missed the `**draft** vX.Y` row-header token — a shape not enumerated in POL-29 fixed-point regex.

**Closure:** PO chain (state-manager initially scoped, handled by PO in chain):
- STORY-INDEX row 399 header token `**draft** v1.8` → `**draft** v1.10` (advancing to story v1.10 to match full PO chain bump)
- Embedded BC pin `BC-2.16.013(v1.9)` → `BC-2.16.013(v1.10)` (synchronized with concurrent F-LP17-HIGH-002 propagation)
- STORY-INDEX v2.167 → v2.168

**Verification:** Both version-token shapes in row 399 now agree: header `**draft** v1.10`, embedded `BC-2.16.013(v1.10)`. Fixed-point confirmed.

---

### F-LP17-HIGH-002 — ADR-028 §Changelog convention reversal vs ADR-022 6-precedent DESCENDING (12th coherence-axis)

**Finding:** FB-IMPL-P16-ARCH flipped ADR-028 §Changelog from descending to ascending based on 3-ADR sample (ADR-025/026/027). ADR-022 v1.6 §Changelog explicitly cites 6 prior POL-26 closures enforcing DESCENDING order (D-611/D-628/D-635/D-659/D-670/D-671). The 3-ADR sample was insufficient — ADR-022 has the earliest and largest precedent set.

**Closure:** Architect scope:
- ADR-028 v1.7 → v1.8: §Changelog REVERTED to descending order per per-file convention lock
- ADR-028 §Status self-cite updated to v1.8
- ADR-028 §D7 added: Per-File §Changelog Convention Lock rule — each ADR's §Changelog convention locks at authoring; fix-bursts preserve established order; POL-26 enforcement targets row position within convention, NOT convention itself. Convention enumeration table included (all current ADRs, their established conventions, and lock dates).
- ARCH-INDEX v2.94 → v2.95

**Verification:** ADR-028 §Changelog is descending. §D7 codifies per-file lock. No further sibling comparison needed — ADR-022 precedent is authoritative, ADR-028 per-file lock now documented.

---

### F-LP17-OBS-001 [process-gap] — POL-29 fixed-point regex token-form enumeration (4th manifestation)

**Finding:** POL-29 step 8 grep regex did not enumerate ALL version-bearing token shapes per artifact. Story has at minimum 4 shapes: `BC-2.16.013 v1.X` (cite), `BC-2.16.013(v1.X)` (compact embedded), `**draft** v1.X` (row-header status), `**Version:** v1.X` (body header). The `**draft** vX.Y` shape was not in the regex, causing F-LP17-HIGH-001 to survive multiple prior passes.

**Closure:** Captured as strong-codification candidate for next policy-add burst. 4th manifestation of this class (P14/P15/P16/P17). POL-29 policy text must be amended to require exhaustive token-form enumeration per artifact before declaring fixed-point. Orchestrator codification pending.

---

### F-LP17-OBS-002 [process-gap] — TD-VSDD-060 exhaustive sibling enumeration (3rd manifestation)

**Finding:** TD-VSDD-060 (sibling-site sweep on value changes) was fulfilled via 3-ADR sample (ADR-025/026/027) during FB-IMPL-P16-ARCH. This was insufficient — ADR-022 (earlier, larger precedent set) was missed. TD-VSDD-060 must require workspace-wide grep, not sampled subset. Earliest-canonical-precedent wins or per-file lock applies.

**Closure:** Captured as strong-codification candidate for next policy-add burst. 3rd manifestation of sibling-sample-bias (P15/P16/P17). CLAUDE.md CONVENTIONS §TD-VSDD-060 must be strengthened to explicitly forbid sampled-set adjudication for convention determinations. Orchestrator codification pending.

---

## Propagation Cascade (PO Chain)

FB-IMPL-P17-ARCH bump ADR-028 v1.7→v1.8 triggered mandatory POL-29 propagation:
1. BC-2.16.013 v1.9 → v1.10: 6 ADR-028 v1.7→v1.8 cite-pin sweep (frontmatter comment + §Changelog row ADR-028 cite at lines 375-379 + line 403)
2. BC-INDEX v5.31 → v5.32 (BC-2.16.013 row updated)
3. Story v1.9 → v1.10: 8 BC-2.16.013 v1.9→v1.10 cite-pin sweep (frontmatter comment, header version, body BC table version column, AC-004 §Known Gaps, Task 4 Claroty, Task 5 Cyberint ×2, Task 6 Armis, Task 9 BehavioralClone)
4. STORY-INDEX v2.167 → v2.168: BOTH header `**draft** v1.8`→`**draft** v1.10` (F-LP17-HIGH-001) AND embedded `BC-2.16.013(v1.9)`→`BC-2.16.013(v1.10)` (F-LP17-HIGH-002 propagation)

Fixed-point reached in 1 iteration. All 4 greps clean.

---

## Cumulative Metrics

| Metric | Before | After |
|--------|--------|-------|
| Total findings closed | 73 | 77 |
| Fix-bursts completed | 15 | 16 |
| Streak | 0/3 | 0/3 |
| Pass count | 16 | 17 |
| Novel coherence-axis classes | 10 | 12 |

---

## Lessons Codified (S-7.02 Candidates — STRONG)

### Lesson 1 — ADR-028 §D7 Per-File §Changelog Convention Lock

Each ADR's §Changelog convention (ascending vs descending) locks at the time of ADR authoring. Subsequent fix-bursts preserve the established order. POL-26 enforcement targets ROW POSITION within the established convention (newest row at top for descending, bottom for ascending), NOT the convention direction itself. When two ADRs use different conventions, no normalization is required — the per-file lock is authoritative. The convention enumeration table in ADR-028 §D7 documents all current ADRs, their established conventions, and lock dates to prevent sample-bias recurrence.

### Lesson 2 — POL-29 Token-Form Enumeration (4th manifestation)

Fixed-point regex for any artifact MUST enumerate ALL version-bearing token shapes that appear in that artifact. For STORY-INDEX rows: (a) `BC-NNN vX.Y` cite form, (b) `BC-NNN(vX.Y)` compact embedded form, (c) `**draft** vX.Y` row-header status form, (d) `**Version:** vX.Y` body-header form. Missing any shape causes that token to survive sweep passes undetected. This finding manifested in P14/P15/P16/P17 — four consecutive passes with increasing specificity. POL-29 policy text amendment is a priority-1 next policy-add burst item.

### Lesson 3 — TD-VSDD-060 Exhaustive Sibling-Set Enumeration (3rd manifestation)

Convention adjudications (which order, which pattern, which convention?) MUST be resolved via workspace-wide grep across ALL artifact siblings, never a sampled subset. When the earliest-canonical-precedent ADR (ADR-022 with 6 cited POL-26 enforcement decisions) contradicts a 3-ADR sample, ADR-022 wins. The per-file lock rule (Lesson 1) prevents this from requiring re-normalization of all sibling ADRs — each file's lock is respected, but new files cannot claim "ascending" based on a sample that excluded the primary precedent-setter. CLAUDE.md §TD-VSDD-060 must explicitly forbid sampled-set adjudication for any convention determination.

---

## Next Action

Pass-18 fresh-context adversary dispatch. All 4 findings closed in-scope. 3 strong-codification candidates captured for next policy-add burst.
