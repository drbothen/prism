---
document_type: remediation-manifest
pass: 61
track: C
finding: MED-003
date: 2026-04-20
author: architect
status: complete
---

# Remediation Pass-61 Track C — MED-003: Duplicate Changelog Version Rows

## Problem

4 VP files each contained two `| 1.1 |` Changelog rows, violating version monotonicity:
- Row 1 (1.1): B-52 / Burst-41 — substantive rename or BC correction
- Row 2 (1.1): pre-build-sweep — template-compliance additions

## Fix Applied

For each file:
1. Renumbered pre-build-sweep row from `1.1` to `1.2`
2. Added `1.3 | pass-61-fix` row at top of Changelog
3. Updated frontmatter `version:` to `"1.3"`

## Files Modified

| VP | File | Version Trail |
|----|------|---------------|
| VP-014 | verification-properties/vp-014-query-oversized-rejection.md | 1.0 → 1.1 (B-52) → 1.2 (pre-build-sweep) → 1.3 (pass-61-fix) |
| VP-015 | verification-properties/vp-015-query-nesting-depth.md | 1.0 → 1.1 (B-52) → 1.2 (pre-build-sweep) → 1.3 (pass-61-fix) |
| VP-021 | verification-properties/vp-021-prismql-parser-no-panic.md | 1.0 → 1.1 (B-52) → 1.2 (pre-build-sweep) → 1.3 (pass-61-fix) |
| VP-030 | verification-properties/vp-030-schedule-rule-caps.md | 1.0 → 1.1 (Burst-41) → 1.2 (pre-build-sweep) → 1.3 (pass-61-fix) |

## Constraints Observed

- No commit made
- input-hash fields untouched (state-manager responsibility)
- All body content preserved verbatim
- Single Write per file
- Absolute paths used throughout
