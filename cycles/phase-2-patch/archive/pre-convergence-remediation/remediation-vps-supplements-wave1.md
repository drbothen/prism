---
document_type: remediation-manifest
version: "1.0"
date: 2026-04-20
author: architect
phase: 2-patch
sweep: pre-build-sweep-wave1
---

# Remediation Manifest: VPs and Supplements Wave 1

Pre-build comprehensive template-compliance sweep executed 2026-04-20.
Source of truth for priorities: VP-INDEX.md v1.5.

---

## VP Remediation Summary

**Total VPs edited: 39 / 39**

| Category | Count |
|----------|-------|
| Frontmatter `priority` added | 39 |
| Frontmatter `verification_method` added (alias) | 39 |
| `proof_method` retained (backward compat) | 39 |
| Version bumped 1.0 → 1.1 | 35 |
| Version held at 1.1 (already at version, Changelog row appended only) | 4 |
| `## Changelog` section created (new) | 37 |
| `## Changelog` row appended to existing section | 2 |

### Decision Log: proof_method vs verification_method

The task brief specified a rename approach (`proof_method` → `verification_method`).
However, the `validate-template-compliance.sh` PostToolUse hook explicitly requires
`proof_method` to be present in all VP frontmatter (matched from
`L4-verification-property-template.md`). Removing `proof_method` triggers a blocking
warning: "Missing: proof_method". Therefore the **alias approach was adopted**:
`proof_method` is retained and `verification_method` is added alongside it with the
same value. Both fields are now present in all 39 VPs. This is noted in every
Changelog row as: "verification_method alias added; proof_method retained for backward
compat." The rename approach cannot be executed without template update.

### VP-030 Special Treatment

VP-030 (`schedule-rule-caps`) was already at version 1.1 but had **no `## Changelog`
section** — only a `## Lifecycle` table. A `## Changelog` section was created with
two rows:
1. Backfilled entry for the Burst-41 change (derived from the Lifecycle table's
   `modified` row: BC correction P3P39-A-HIGH-005).
2. Pre-build-sweep row for this remediation pass.

VP-030 Lifecycle table is preserved intact. The changelog section was appended
immediately after the Lifecycle table.

### VPs at 1.1 before this sweep (Changelog row appended, no version change)

| VP | Prior Changelog | Action |
|----|----------------|--------|
| VP-014 | B-52 PrismQL rename | Row appended |
| VP-015 | B-52 PrismQL rename | Row appended |
| VP-021 | B-52 PrismQL rename | Row appended |
| VP-030 | None (Lifecycle only) | Section created; 2 rows added (backfill + sweep) |

### Priority assignments applied (from VP-INDEX v1.5)

| Priority | VPs |
|----------|-----|
| P0 (32 VPs) | VP-001 through VP-024, VP-027, VP-028, VP-031, VP-033, VP-034, VP-036, VP-038, VP-039 |
| P1 (7 VPs) | VP-025, VP-026, VP-029, VP-030, VP-032, VP-035, VP-037 |

No VP-INDEX gaps encountered. All 39 VPs had unambiguous priority assignments.

### No anomalies or VP-INDEX gaps

All 39 VP priorities were clearly specified in VP-INDEX v1.5. No conflicts between
VP-INDEX and VP frontmatter existing fields (there were no `priority` fields prior to
this sweep). All `proof_method` values matched the VP-INDEX `Method` column.

---

## Supplement Remediation Summary

**Total supplements edited: 4 / 4**

| File | Prior Version | New Version | inputs Added | input-hash Added | traces_to Added | Changelog Status |
|------|--------------|-------------|-------------|-----------------|----------------|-----------------|
| error-taxonomy.md | 1.3 | 1.4 | YES | YES | YES | Existing; row appended |
| interface-definitions.md | 2.2 | 2.3 | YES | YES | YES | Existing (## 5. Changelog); row prepended |
| nfr-catalog.md | 1.0 | 1.1 | YES | YES | YES | Created (new section) |
| test-vectors.md | 2.3 | 2.4 | NO (already present) | NO (already null) | NO (already present) | Created (new section) |

### Notes per supplement

**error-taxonomy.md (1.3 → 1.4):**
- Added `inputs: [".factory/specs/prd.md", ".factory/specs/behavioral-contracts/**"]`
- Added `input-hash: "[pending-recompute]"`
- Added `traces_to: [".factory/specs/prd.md"]`
- Appended row to existing `## Changelog` table at file end.

**interface-definitions.md (2.2 → 2.3):**
- Added `inputs: [".factory/specs/prd.md", ".factory/specs/architecture/**"]`
- Added `input-hash: "[pending-recompute]"`
- Added `traces_to: [".factory/specs/prd.md"]`
- Changelog section exists as `## 5. Changelog` — row prepended at top of table
  (file uses reverse-chronological ordering).

**nfr-catalog.md (1.0 → 1.1):**
- Added `inputs: [".factory/specs/prd.md"]`
- Added `input-hash: "[pending-recompute]"`
- Added `traces_to: [".factory/specs/prd.md"]`
- No prior Changelog section; `## Changelog` created and appended.

**test-vectors.md (2.3 → 2.4):**
- `inputs`, `input-hash` (null), `traces_to` already present — no frontmatter change.
- `input-hash` remains `null` (was already null; not changed per scope rules).
- No prior `## Changelog` section (file had inline version history prose); `## Changelog`
  table section created and appended.

---

## Step 4 Reminder: input-hash Recompute

All supplements with `input-hash: "[pending-recompute]"` need hash recomputation:
- `.factory/specs/prd-supplements/error-taxonomy.md`
- `.factory/specs/prd-supplements/interface-definitions.md`
- `.factory/specs/prd-supplements/nfr-catalog.md`

VP files: `input-hash` fields were not touched per scope rules.

Run `compute-input-hash <file> --update` or ask state-manager to compute post-sweep.

---

## Manifest Stats

| Metric | Count |
|--------|-------|
| Total files edited | 43 |
| VP files | 39 |
| Supplement files | 4 |
| VP-INDEX gaps | 0 |
| Anomalies | 1 (proof_method rename blocked by hook → alias approach adopted) |
