---
document_type: remediation-manifest
pass: pass-63
track: B
producer: product-owner
timestamp: 2026-04-20
findings_addressed:
  - P3P63-A-MED-001
  - P3P63-A-OBS-001
---

# Pass-63 Track B — BC-Owned Findings Remediation

## Canonical BC Changelog Schema Decision

**Chosen schema: 4-column `| Version | Burst | Finding | Change |`**

Corpus survey (two reference BCs read):
- BC-2.03.005: uses 4-column `| Version | Date | Burst | Change |` — Date/Burst order
- BC-2.01.001: uses 5-column `| Version | Burst | Date | Author | Changes |`
- BC-2.10.004: uses 4-column `| Version | Date | Burst | Change |` (Date/Burst order)
- BC-2.12.011: uses 4-column `| Version | Burst | Finding | Change |` (Burst/Finding order, no Date col)

**Observation:** There is genuine schema drift across the corpus. Two dominant variants exist:
1. `Version | Date | Burst | Change` — used by BCs with a date-anchored row convention (BC-2.03.005, BC-2.10.004)
2. `Version | Burst | Finding | Change` — used by BCs with a finding-reference convention (BC-2.12.011)

BC-2.12.011 is a retired tombstone. Its existing header `| Version | Burst | Finding | Change |` is its local convention. The fix aligns its rows to that header rather than migrating to a different schema. BC-2.10.004 uses the `| Version | Date | Burst | Change |` variant; its row 2.2 was malformed (5 values) and is corrected in-place.

---

## Finding P3P63-A-MED-001 — BC-2.12.011 Changelog Table Structure Malformed

**File:** `.factory/specs/behavioral-contracts/BC-2.12.011-action-at-least-once-delivery.md`

**Root cause:**
- Row 1.2: Burst and Date columns semantically swapped (date `2026-04-20` in Burst column; label `pre-build-sweep` in Finding column). Stale "version bump 1.0 → 1.1" text present (already superseded by pass-62 renumbering).
- Row 1.3: 5 values (`| 1.3 | pass-62-fix | 2026-04-20 | product-owner | ...`) inserted into a 4-column table.

**Fix applied:**
- Row 1.2 rewritten: `| 1.2 | pre-build-sweep | — | Template-compliance sweep: standardized inputs/input-hash/traces_to/extracted_from frontmatter to Wave 4 convention. |` (stale "version bump 1.0→1.1" text removed; columns aligned to header).
- Row 1.3 rewritten: `| 1.3 | pass-62-fix | P3P62-A-MED-001 | Renumbered duplicate 1.0 Changelog rows for monotonicity... |` (collapsed from 5-value to 4-column; author column removed; Finding citation added).
- Row 1.4 prepended at top of table (pass-63-fix entry).
- Changelog rows ordered newest-first (1.4 → 1.0).
- Frontmatter `version` bumped: `"1.3"` → `"1.4"`.

**Version trail:** 1.0 → 1.1 → 1.2 → 1.3 → 1.4

---

## Finding P3P63-A-OBS-001 — BC-2.10.004 Unquoted capability Frontmatter Value

**File:** `.factory/specs/behavioral-contracts/BC-2.10.004-client-id-parameter-requirement.md`

**Root cause:**
- Line 15: `capability: CAP-009` — unquoted YAML value, inconsistent with corpus convention (e.g., BC-2.03.005 uses `capability: "CAP-004"`).
- Secondary issue found during review: Changelog row 2.2 had 5 values in the 4-column `| Version | Date | Burst | Change |` table (extra `product-owner` column spill). Corrected in same edit.

**Fix applied:**
- `capability: CAP-009` → `capability: "CAP-009"`
- Changelog row 2.2 collapsed from 5-value to 4-column; `product-owner` author spill removed.
- Row 2.3 prepended at top of changelog (pass-63-fix entry).
- Changelog rows ordered newest-first (2.3 → 2.0).
- Frontmatter `version` bumped: `"2.2"` → `"2.3"`.

**Version trail:** 2.0 → 2.1 → 2.2 → 2.3

---

## Schema Drift Observation

The corpus has two 4-column changelog schemas in active use:
1. `Version | Date | Burst | Change` (BC-2.03.005, BC-2.10.004)
2. `Version | Burst | Finding | Change` (BC-2.12.011)

And one 5-column schema (BC-2.01.001, tombstone). A future normalization pass could unify to a single canonical header. However, since this pass only addresses the malformed rows (not schema normalization), existing headers are preserved as-is. The fixes ensure all rows are internally consistent with their file's own header.

---

## Files Modified

| File | Change | Version Before | Version After |
|------|--------|---------------|---------------|
| `.factory/specs/behavioral-contracts/BC-2.12.011-action-at-least-once-delivery.md` | Aligned rows 1.2/1.3 to 4-col schema; added 1.4 pass-63 row | 1.3 | 1.4 |
| `.factory/specs/behavioral-contracts/BC-2.10.004-client-id-parameter-requirement.md` | Quoted capability; fixed malformed row 2.2; added 2.3 pass-63 row | 2.2 | 2.3 |

## Constraints Honored

- No commit performed.
- No input-hash recomputed.
- Single Write per file (Edit tool used for targeted changes).
- All other content preserved.
- Absolute paths used throughout.
