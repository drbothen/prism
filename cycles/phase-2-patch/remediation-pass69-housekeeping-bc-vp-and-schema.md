---
document_type: remediation-manifest
version: "1.0"
phase: phase-2-patch
burst: pass-69-housekeeping
timestamp: 2026-04-20T00:00:00
producer: product-owner
---

# Remediation Manifest: Pass-69 Housekeeping — VP-TBD Resolution + Changelog Schema Normalization

## Summary

- **Total BCs targeted for VP-TBD resolution:** 22
- **Total BCs modified for changelog schema normalization:** 22 (all had non-canonical schemas)
- **Total unique BCs touched:** 22 (all modifications in a single pass)

---

## Per-BC Action Log

| BC ID | VP-TBD Action | Schema Source (Before) | Schema Normalized | New Version |
|-------|---------------|------------------------|-------------------|-------------|
| BC-2.08.006 | MARK-NONE | `Version \| Date \| Author \| Notes` (4-col) | Yes | 1.3 |
| BC-2.10.008 | ADD-VP-050 | `Version \| Date \| Author \| Notes` (4-col) | Yes | 1.5 |
| BC-2.17.001 | MARK-NONE | `Version \| Date \| Burst \| Change` (4-col) | Yes | 1.2 |
| BC-2.17.002 | ADD-VP-040 | `Version \| Date \| Burst \| Change` (4-col) | Yes | 1.2 |
| BC-2.17.003 | ADD-VP-041 | `Version \| Date \| Burst \| Change` (4-col) | Yes | 1.2 |
| BC-2.17.004 | MARK-NONE | `Version \| Date \| Burst \| Change` (4-col) | Yes | 1.2 |
| BC-2.17.005 | ADD-VP-042 | `Version \| Date \| Burst \| Change` (4-col) | Yes | 1.3 |
| BC-2.17.006 | ADD-VP-043 | `Version \| Date \| Burst \| Change` (4-col) | Yes | 1.2 |
| BC-2.18.001 | ADD-VP-044 | `Version \| Date \| Burst \| Change` (4-col) | Yes | 1.2 |
| BC-2.18.002 | MARK-NONE | `Version \| Date \| Burst \| Change` (4-col) | Yes | 1.2 |
| BC-2.18.003 | MARK-NONE | `Version \| Date \| Burst \| Change` (4-col) | Yes | 1.2 |
| BC-2.18.004 | ADD-VP-045 | `Version \| Date \| Burst \| Change` (4-col) | Yes | 1.2 |
| BC-2.18.005 | MARK-NONE | `Version \| Date \| Burst \| Change` (4-col) | Yes | 1.2 |
| BC-2.18.006 | MARK-NONE | `Version \| Date \| Burst \| Change` (4-col) | Yes | 1.2 |
| BC-2.18.007 | ADD-VP-046 | `Version \| Date \| Burst \| Change` (4-col) | Yes | 1.2 |
| BC-2.18.008 | MARK-NONE | `Version \| Date \| Burst \| Change` (4-col) | Yes | 1.2 |
| BC-2.18.009 | ADD-VP-047 | `Version \| Date \| Burst \| Change` (4-col) | Yes | 1.2 |
| BC-2.19.001 | ADD-VP-048 | `Version \| Date \| Burst \| Change` (4-col) | Yes | 1.2 |
| BC-2.19.002 | ADD-VP-049 | `Version \| Date \| Burst \| Change` (4-col) | Yes | 1.2 |
| BC-2.19.003 | MARK-NONE | `Version \| Date \| Burst \| Change` (4-col) | Yes | 1.2 |
| BC-2.19.004 | MARK-NONE | `Version \| Date \| Burst \| Change` (4-col) | Yes | 1.2 |
| BC-2.19.005 | MARK-NONE | `Version \| Date \| Burst \| Change` (4-col) | Yes | 1.2 |

---

## Task 1: VP-TBD Resolution Counts

| Decision | Count | BCs |
|----------|-------|-----|
| ADD-VP | 11 | BC-2.10.008 (VP-050), BC-2.17.002 (VP-040), BC-2.17.003 (VP-041), BC-2.17.005 (VP-042), BC-2.17.006 (VP-043), BC-2.18.001 (VP-044), BC-2.18.004 (VP-045), BC-2.18.007 (VP-046), BC-2.18.009 (VP-047), BC-2.19.001 (VP-048), BC-2.19.002 (VP-049) |
| MARK-NONE | 11 | BC-2.08.006, BC-2.17.001, BC-2.17.004, BC-2.18.002, BC-2.18.003, BC-2.18.005, BC-2.18.006, BC-2.18.008, BC-2.19.003, BC-2.19.004, BC-2.19.005 |

Note: The decision matrix summary says "ADD-VP = 10, MARK-NONE = 12" but lists 11 ADD-VP entries (VP-040 through VP-050) and 11 MARK-NONE entries across 22 BCs. The matrix's per-entry analysis is authoritative. VP count is 11 (VP-040 through VP-050 inclusive). Discrepancy in matrix summary row does not affect output.

---

## Task 2: Changelog Schema Normalization Counts

| Source Schema | Count | BCs |
|---------------|-------|-----|
| `Version \| Date \| Author \| Notes` (4-col) | 2 | BC-2.08.006, BC-2.10.008 |
| `Version \| Date \| Burst \| Change` (4-col, reordered) | 20 | BC-2.17.001 through BC-2.19.005 |
| Already canonical `Version \| Burst \| Date \| Author \| Changes` (5-col) | 0 | — |

**Canonical schema adopted:** `Version | Burst | Date | Author | Changes` (5-col)

**Conversion strategy applied:**
- For `Version | Date | Author | Notes`: reordered Date↔Burst (Date moved to col 3, new Burst col 2), renamed Notes→Changes.
- For `Version | Date | Burst | Change`: reordered (Burst promoted to col 2, Date to col 3), added Author column (inferred from file producer/context), renamed Change→Changes.
- All historical rows preserved with lossless content migration.

---

## Anomalies

1. **BC-2.10.008 version gap:** The file had changelog row for v1.3 but version frontmatter was 1.4 (an architect updated to 1.4 in burst-49 without adding a changelog row). The new v1.5 row is added correctly; the missing v1.4 row for burst-49 is present in the existing changelog content (not a new gap).

2. **Decision matrix row count discrepancy:** The matrix summary row says "ADD-VP = 10, MARK-NONE = 12" but the detailed per-BC analysis lists 11 ADD-VP entries (VP-040 through VP-050) and 11 MARK-NONE entries. The corrected totals section in the matrix (final table) shows 49 total VPs (+10 net additions from 11 proposed due to a Kani/Proptest tally correction). This manifest applied 11 ADD-VP actions matching the detailed per-BC analysis.

3. **BC-2.17.002 second VP-TBD** (HTTP proxy integration test): The matrix specifies VP-040 for the WASI linker invariant only. The second VP-TBD entry ("HTTP proxy routes through host reqwest client") is explicitly MARK-NONE per the matrix analysis. Both VP-TBD rows were resolved in a single Verification Properties table update — the WASI property becomes VP-040 and the HTTP proxy becomes (none).

---

## Changelog Row Added to Each BC

All 22 BCs received the following row at the top of their normalized changelog table:

```
| {new_version} | pass-69-housekeeping | 2026-04-20 | product-owner | Resolved VP-TBD placeholder per decision matrix; normalized changelog schema to canonical 5-col form. |
```

---

## VP Citation Changes

VP citations changed in the following BCs:
- BC-2.10.008: VP-TBD → VP-050
- BC-2.17.002: VP-TBD → VP-040 (first), (none) (second)
- BC-2.17.003: VP-TBD → VP-041 (first), (none) (second)
- BC-2.17.005: VP-TBD → VP-042 (first), (none) (second)
- BC-2.17.006: VP-TBD → VP-043
- BC-2.18.001: VP-TBD → VP-044 (first), (none) (second and third)
- BC-2.18.004: VP-TBD → VP-045
- BC-2.18.007: VP-TBD → VP-046
- BC-2.18.009: VP-TBD → VP-047
- BC-2.19.001: VP-TBD → VP-048
- BC-2.19.002: VP-TBD → VP-049 (first), (none) (second)
- BC-2.08.006, BC-2.17.001, BC-2.17.004, BC-2.18.002, BC-2.18.003, BC-2.18.005, BC-2.18.006, BC-2.18.008, BC-2.19.003, BC-2.19.004, BC-2.19.005: VP-TBD → (none)

**Architect must propagate to VP-INDEX, verification-architecture.md, and verification-coverage-matrix.md under `vp_index_is_vp_catalog_source_of_truth` policy.**

---

## Files Modified

All files are under `/Users/jmagady/Dev/prism/.factory/specs/behavioral-contracts/`:

1. BC-2.08.006-health-mcp-resource.md
2. BC-2.10.008-mcp-resources.md
3. BC-2.17.001-plugin-panic-isolation.md
4. BC-2.17.002-plugin-sandbox-filesystem.md
5. BC-2.17.003-plugin-memory-limit.md
6. BC-2.17.004-plugin-cpu-time-limit.md
7. BC-2.17.005-plugin-hot-reload-atomic-swap.md
8. BC-2.17.006-plugin-wit-validation.md
9. BC-2.18.001-action-at-least-once-delivery.md
10. BC-2.18.002-action-schedule-best-effort.md
11. BC-2.18.003-action-manual-fire-and-forget.md
12. BC-2.18.004-action-schedule-semaphore.md
13. BC-2.18.005-action-partial-report-failure.md
14. BC-2.18.006-action-template-injection-scan.md
15. BC-2.18.007-action-credential-opaque-reference.md
16. BC-2.18.008-action-delivery-audit-logging.md
17. BC-2.18.009-action-uuid-v7-validation.md
18. BC-2.19.001-infusion-spec-loading.md
19. BC-2.19.002-infusion-per-query-dedup.md
20. BC-2.19.003-infusion-api-backed-rejection.md
21. BC-2.19.004-infusion-hot-reload-atomicity.md
22. BC-2.19.005-infusion-credential-redaction.md
