---
document_type: remediation-manifest
pass: pass-64
track: A
date: 2026-04-20
author: story-writer
findings_addressed: [P3P64-A-HIGH-001, P3P64-A-MED-001]
---

# Remediation Manifest — Pass 64 Track A

## P3P64-A-HIGH-001: Wave-1/2 Stories with Unfilled Template Scaffolding

### Per-File TODO Marker Counts (body sections only; changelog text excluded)

| Story | File | Before | After | Sections Filled |
|-------|------|--------|-------|-----------------|
| S-1.07 | S-1.07-credential-crud.md | 18 markers across 6 sections | 0 | Narrative, Token Budget, Previous Story Intelligence, Architecture Compliance Rules, Library & Framework Requirements, File Structure Requirements |
| S-1.08 | S-1.08-feature-flags.md | 18 markers across 6 sections | 0 | Narrative, Token Budget, Previous Story Intelligence, Architecture Compliance Rules, Library & Framework Requirements, File Structure Requirements |
| S-1.09 | S-1.09-confirmation-tokens.md | 18 markers across 6 sections | 0 | Narrative, Token Budget, Previous Story Intelligence, Architecture Compliance Rules, Library & Framework Requirements, File Structure Requirements |
| S-1.10 | S-1.10-prompt-injection-defense.md | 18 markers across 6 sections | 0 | Narrative, Token Budget, Previous Story Intelligence, Architecture Compliance Rules, Library & Framework Requirements, File Structure Requirements |
| S-1.11 | S-1.11-spec-loading.md | 18 markers across 6 sections | 0 | Narrative, Token Budget, Previous Story Intelligence, Architecture Compliance Rules, Library & Framework Requirements, File Structure Requirements |
| S-1.12 | S-1.12-hot-reload.md | 18 markers across 6 sections | 0 | Narrative, Token Budget, Previous Story Intelligence, Architecture Compliance Rules, Library & Framework Requirements, File Structure Requirements |
| S-1.13 | S-1.13-sensor-write-specs.md | 6 markers across 3 sections | 0 | Narrative, Token Budget, Previous Story Intelligence (Architecture Compliance Rules, Library & Framework Requirements, File Structure Requirements were pre-filled in prior sweep) |

**Note:** The only `[TODO` strings remaining in these files appear inside Changelog version row descriptions as literal text recording what was filled — not as unfilled template markers. All body-section placeholders are replaced.

### Content Authorship Approach

All content was derived faithfully from:
- Story `behavioral_contracts:` frontmatter and BC titles
- Existing Tasks and Acceptance Criteria (BC-faithful)
- S-1.06 exemplar (filled Wave-1 story in same epic)
- S-4.08 exemplar (filled Wave-4 story with full token budget table)
- `architecture/module-decomposition.md` module boundaries (from prior session reads)

No content was invented without BC grounding.

---

## P3P64-A-MED-001: S-4.08 Policy 8 Violation — BC-2.09.004 Missing from Frontmatter

**File:** `S-4.08-action-delivery.md`

| Item | Before | After |
|------|--------|-------|
| `behavioral_contracts:` frontmatter | 9 BCs (BC-2.18.001–009) | 10 BCs (added BC-2.09.004) |
| `anchor_bcs:` frontmatter | 9 BCs | 10 BCs (added BC-2.09.004) |
| `inputs:` frontmatter | 9 BC paths | 10 BC paths (added BC-2.09.004 path) |
| Body BC table | 9 rows | 10 rows (added BC-2.09.004 with cross-story note) |

**Cross-story note added:** BC-2.09.004 row in body table explicitly notes it is consumed via S-1.10 InjectionScanner. AC-8 already correctly traced to BC-2.09.004 and BC-2.18.006; the frontmatter was the only violation.

---

## Wave 3–8 Corpus Audit (Bonus)

**Method:** `grep -l "\[TODO" .factory/stories/*.md`

**Result:** Zero additional stories found with unfilled `[TODO:` markers beyond the 7 addressed above.

The grep returned exactly the 7 files listed in P3P64-A-HIGH-001. After fixes, remaining `[TODO` strings in those 7 files are confined to Changelog description text only (the remediation log entry itself). No other stories in waves 3–8 have unfilled template scaffolding.

---

## Summary

| Category | Count |
|----------|-------|
| Files touched (wave-1/2 stories filled) | 7 |
| Files touched (S-4.08 Policy 8 fix) | 1 |
| Additional corpus TODO markers found and fixed | 0 |
| **Total files touched** | **8** |

## Judgment Calls

1. **S-1.13 partial fill:** Architecture Compliance Rules, Library & Framework Requirements, and File Structure Requirements were already populated in the `pre-build-sweep` pass. Only Narrative, Token Budget, and Previous Story Intelligence were unfilled. The changelog row reflects this accurately.

2. **Token budget table format:** Used artifact-per-file row format (as in S-1.06 and S-4.08 exemplars) rather than the generic 4-row Source/Tokens format, as it provides more actionable context for the implementer agent.

3. **BC-2.09.004 body table note:** Added explicit "(consumed via S-1.10 cross-story reference)" note to the BC row in S-4.08's body table to satisfy the cross-story traceability requirement without implying S-4.08 re-implements the scanner.

4. **Changelog `[TODO` text:** The Changelog rows in all 7 remediated stories contain the literal text `[TODO: template scaffolding` as part of the change description. This is intentional (describes what was fixed) and is not an unfilled placeholder. The grep-based audit correctly identifies these as non-actionable after inspection.
