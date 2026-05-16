---
document_type: adversarial-review
story: S-PLUGIN-PREREQ-E
pass: 4
scope: spec
verdict: BLOCKED
total_findings: 9
severity_breakdown:
  critical: 0
  high: 4
  medium: 3
  low: 2
  observation: 0
in_scope_findings: 9
observations_queued: 0
produced_by: adversary
reviewed_at: 2026-05-15
fix_burst: fix-burst-4
fix_burst_closed_at: D-578
streak_after_fix: "0/3"
---

# S-PLUGIN-PREREQ-E Adversarial Spec Review — Pass 4

**Verdict: BLOCKED — 9 findings (0C + 4H + 3M + 2L + 0OBS)**

Fix-burst-4 closed all 9 in-scope findings. Trajectory: 14 → 9 → 8 → 9 (FLAT — not converging).
3 of 4 HIGH findings were systemic VP-156 anchor-back gaps (fix-burst-1 residue): VP-156 was
authored in FB1 but its anchor-back chain was never fully swept across the story
`verification_properties:` array, `anchor_vps:` frontmatter, story §References, and
ADR-026 §Verification Property Anchors. 1 fresh-context HIGH finding (F-LP4-HIGH-003): BC-2.16.004
row title in the story's BC table was paraphrased, not POL-7 verbatim H1 — this was undetected
by passes 1, 2, and 3. 2 MEDIUM findings were VP changelog version-collision repairs (VP-153/154/155/156
version monotonicity broken by FB1 backfill). 1 MEDIUM was error-taxonomy `modified:` date drift.
2 LOW findings were ADR-027 D5 scope and VP `modified:` ISO field normalization.

CASCADE LENGTH WARNING: 4 passes deep with a flat novelty curve (14→9→8→9). PREREQ-D took 17+
passes. The production-grade default cascade is finding genuine quality issues each pass. User
may want to assess cascade strategy. NEXT: adversary pass-5 fresh-context.

---

## Finding Inventory

### F-LP4-HIGH-001 — VP-156 absent from story `verification_properties:` frontmatter array

**Severity:** HIGH
**Type:** VP-156 anchor-back gap (FB1 residue)
**Closed by:** product-owner (D-578 fix-burst-4)

VP-156 (`vp-156-write-tool-registration-uniqueness.md`) was authored in fix-burst-1 and cited in
ADR-026 D7 narrative and BC-2.16.012. However, the story frontmatter `verification_properties:`
array did not include VP-156. The `anchor_vps:` array also lacked VP-156.

**Fix:** VP-156 added to `verification_properties:` array (after VP-155, before VP-PLUGIN-001) and
to `anchor_vps:` array. Story v1.4 → v1.5.

---

### F-LP4-HIGH-002 — VP-156 absent from story §References section

**Severity:** HIGH
**Type:** VP-156 anchor-back gap (FB1 residue)
**Closed by:** product-owner (D-578 fix-burst-4)

The story §References "Architecture Compliance" subsection listed VP-153, VP-154, VP-155 with
markdown links but omitted VP-156 entirely. VP-156's existence as a proptest for
`register_write_tool` uniqueness semantics was established in FB1 — all three prior passes
missed this §References gap.

**Fix:** VP-156 markdown link + "uniqueness-only" description appended to §References
Architecture Compliance list. Story v1.4 → v1.5.

---

### F-LP4-HIGH-003 — BC-2.16.004 row title paraphrased, not POL-7 verbatim H1

**Severity:** HIGH
**Type:** POL-7 verbatim H1 violation (fresh-context catch — undetected by passes 1–3)
**Closed by:** product-owner (D-578 fix-burst-4)

The story's BC table row for BC-2.16.004 used a paraphrased title: "Rust Escape Hatch
(CustomAdapter trait — deprecated in PREREQ-F, removed by this story)". The canonical H1 of
BC-2.16.004 is "Rust Escape Hatch for Custom Adapters — Trait-Based Override When Config Is
Insufficient". POL-7 requires verbatim H1 in BC table Title cells.

This finding was introduced in the original story draft and survived 3 prior adversary passes
without detection — consistent with fresh-context cognitive diversity value (BC-5.39.001 rationale:
each pass uses a fully independent context load).

**Fix:** BC-2.16.004 Title cell updated to verbatim H1. Lifecycle annotation "(deprecated →
removed)" moved inline to Role column (appropriate per POL-7: annotations belong in Role, not
Title). Story v1.4 → v1.5.

---

### F-LP4-HIGH-004 — ADR-026 §Verification Property Anchors missing VP-156 entry

**Severity:** HIGH
**Type:** VP-156 anchor-back gap (FB1 residue)
**Closed by:** architect (D-578 fix-burst-4)

ADR-026 v1.5 added D7 narrative citing VP-156 (line 293) and D7 rationale referencing VP-156
(line 270), and the v1.5 changelog described VP-156 as a D7 deliverable. However, the
§Verification Property Anchors section (which carries structured entries for VP-153/154/155) did
not contain a VP-156 entry.

**Fix:** VP-156 entry added to §Verification Property Anchors matching format of prior VP
entries: ID + title + verification method (proptest/P1) + status (draft) + BC anchor
(BC-2.16.012 EC-016-012-004 / INV-INVALIDATION-EXT-001). ADR-026 v1.5 → v1.6.

---

### F-LP4-MED-001 — VP-153 changelog version-collision (v0.3 row absent; v0.4 without predecessor)

**Severity:** MEDIUM
**Type:** Changelog monotonicity violation (state-manager catch — renumber required)
**Closed by:** state-manager (D-578 fix-burst-4)

VP-153 was authored at v0.3 in the spec draft package (D-574), but fix-burst-1 assigned it the
same v0.3 version label for its FB1 changes, producing a duplicate row conflict. Monotonic
version sequence broken: no v0.4 row existed despite the file having accumulated 4 distinct
change events.

**Fix:** state-manager renumbered VP-153 changelog: original authored v0.3 row retained as
canonical v0.3; FB1 changes promoted to new v0.4 row with 4-row monotonic sequence restored.
VP-153 frontmatter version: v0.3 → v0.4.

---

### F-LP4-MED-002 — VP-154/155/156 changelog version-collisions (duplicate rows)

**Severity:** MEDIUM
**Type:** Changelog monotonicity violation (state-manager catch — renumber required)
**Closed by:** state-manager (D-578 fix-burst-4)

Same class of defect as F-LP4-MED-001 but across VP-154, VP-155, and VP-156. Each VP had a
duplicate `modified:` field introduced during fix-burst-1 backfill, producing duplicate
version rows in their respective changelogs.

**Fix:** state-manager repaired all three:
- VP-154: v0.4 duplicate row resolved → v0.5 canonical
- VP-155: v0.1 duplicate row resolved → v0.2 canonical
- VP-156: v0.2 duplicate row resolved → v0.3 canonical

---

### F-LP4-MED-003 — error-taxonomy `modified:` date stale (not synced to 2026-05-15)

**Severity:** MEDIUM
**Type:** Modified-date drift (POL-27 class)
**Closed by:** product-owner (D-578 fix-burst-4)

The error-taxonomy `modified:` field had not been updated to 2026-05-15 after the
E-PLUGIN-012/E-PLUGIN-020 allocations in fix-burst-2 (D-576). POL-27 requires ISO date sync
on any `modified:` field when a file is touched. error-taxonomy v1.27 frontmatter `modified:`
synced; no version bump required (date-only field correction).

---

### F-LP4-LOW-001 — ADR-027 D5 scope under-specified (audit + migrate not enumerated)

**Severity:** LOW
**Type:** Scope completeness gap
**Closed by:** architect (D-578 fix-burst-4)

ADR-027 D5 described "remove hardcoded sensor strings" as a vague goal without enumerating the
two required actions: (1) clean-pass to remove CustomAdapter trait references and (2) audit of
hardcoded sensor strings per BC-2.16.012 INV-SPEC-PARSER-OPEN-001. The implementer needs both
enumerated to correctly scope the migration work.

**Fix:** ADR-027 D5 scope expanded to enumerate both actions explicitly (CustomAdapter clean-pass
+ hardcoded-sensor-string audit). ADR-027 v1.2 → v1.3.

---

### F-LP4-LOW-002 — 4 VP `modified:` fields non-ISO scalar (string-quoted inconsistency)

**Severity:** LOW
**Type:** POL-27 format normalization (ISO scalar vs quoted string)
**Closed by:** architect / state-manager (D-578 fix-burst-4)

VP-153/154/155/156 `modified:` fields used quoted string format (`"2026-05-15"`) instead of
bare ISO scalar (`2026-05-15`) required by the VP template and POL-27. This inconsistency was
introduced during the original spec draft (D-574) and carried through fix-bursts 1–3 without
detection.

**Fix:** All 4 VP `modified:` fields normalized to bare ISO scalar. No version bump beyond the
version increments applied for F-LP4-MED-001/002 repairs.

---

## Cascade Length Warning

Pass-4 trajectory: **14 → 9 → 8 → 9 (FLAT)**.

The PREREQ-D cascade ran to 17+ passes. The production-grade default cascade (BC-5.39.001,
Standing Rule 3 §1) is finding genuine quality issues each pass — the flat trajectory is evidence
the spec package still has exploitable inconsistencies, not evidence of converging noise.

Codification candidates from this pass (queued for cycle-close):
- **(a)** Extend POL-25 to enumerate VP→story anchor-back sweep targets explicitly (VP-156 chain
  gap class: `verification_properties:` array + `anchor_vps:` array + §References + ADR
  §Verification Property Anchors)
- **(b)** Extend POL-27 scope from BC files to VPs and PRD-supplements (VP `modified:` drift
  same class as BC modified-field drift)

---

## Artifact Versions After Fix-Burst-4

| Artifact | After FB3 | After FB4 |
|----------|-----------|-----------|
| ADR-026 | v1.5 | v1.6 |
| ADR-027 | v1.2 | v1.3 |
| ARCH-INDEX | v2.46 | v2.47 |
| BC-2.16.012 | v1.4 | v1.5 |
| S-PLUGIN-PREREQ-E story | v1.4 | v1.5 |
| VP-153 | v0.3 | v0.4 |
| VP-154 | v0.4 | v0.5 |
| VP-155 | v0.1 | v0.2 |
| VP-156 | v0.2 | v0.3 |
| error-taxonomy | v1.27 | v1.27 (modified: date sync only) |
| STATE + HANDOFF | v7.282 | v7.283 |

## Next Step

Adversary pass-5 dispatch (fresh-context). BC-5.39.001 3-CLEAN protocol — streak 0/3.
Pass-4 report: `cycles/wave-4-operations/adversarial-reviews/S-PLUGIN-PREREQ-E-spec-pass-4.md`
