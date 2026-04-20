---
document_type: remediation-manifest
pass: 61
track: B
issue: MED-001 (BC scope extension), MED-002
date: 2026-04-20
author: product-owner
---

# Pass-61 Track B — BC Changelog Monotonicity Fix

Duplicate version rows introduced by the pre-build-sweep pass (2026-04-20) are renumbered
so no two rows share the same version number. Frontmatter `version:` is updated to the
highest row in each file's changelog.

## Version Trail Per File

### MED-001: Tombstone BCs (6 of 7 with `status: removed`, 1 active)

| File | Old duplicate rows | pre-build-sweep → renumbered | pass-61-fix row | Final version |
|------|--------------------|------------------------------|-----------------|---------------|
| BC-2.01.001-single-client-sensor-query.md | 2.0 (cycle-1) + 2.0 (pre-build-sweep) | 2.0 → 2.1 | 2.2 | **2.2** |
| BC-2.01.002-cross-client-fan-out.md | 2.0 (cycle-1) + 2.0 (pre-build-sweep) | 2.0 → 2.1 | 2.2 | **2.2** |
| BC-2.01.003-cursor-based-pagination.md | 2.0 (cycle-1) + 2.0 (pre-build-sweep) | 2.0 → 2.1 | 2.2 | **2.2** |
| BC-2.01.009-query-filtering-sorting.md | 2.0 (cycle-1) + 2.0 (pre-build-sweep) | 2.0 → 2.1 | 2.2 | **2.2** |
| BC-2.01.011-cross-sensor-correlation-ocsf-fields.md | 2.0 (cycle-1) + 2.0 (pre-build-sweep) | 2.0 → 2.1 | 2.2 | **2.2** |
| BC-2.01.012-query-fingerprint-validation.md | 2.0 (cycle-1) + 2.0 (pre-build-sweep) | 2.0 → 2.1 | 2.2 | **2.2** |
| BC-2.01.015-response-envelope-structure.md | 2.0 (cycle-1) + 2.0 (pre-build-sweep) | 2.0 → 2.1 | 2.2 | **2.2** |

### MED-002: Active BC

| File | Old duplicate rows | pre-build-sweep → renumbered | pass-61-fix row | Final version |
|------|--------------------|------------------------------|-----------------|---------------|
| BC-2.03.005-credential-crud-operations.md | 1.2 (Burst 44) + 1.2 (pre-build-sweep) | 1.2 → 1.3 | 1.4 | **1.4** |

## Change Summary

- 7 tombstone BCs: duplicate `2.0` row renumbered to `2.1`; pass-61-fix row added at `2.2`; frontmatter `version: "2.2"`
- 1 active BC: duplicate `1.2` row renumbered to `1.3`; pass-61-fix row added at `1.4`; frontmatter `version: "1.4"`
- Total files modified: 8
- `input-hash` fields: unchanged (state-manager responsibility)
- No commits created (state-manager handles)
