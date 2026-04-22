---
document_type: remediation-manifest
pass: 65
track: B
finding: P3P65-A-LOW-001
burst: pass-65-fix
date: 2026-04-20
producer: product-owner
---

# Pass-65 Track B Remediation Manifest

**Finding:** P3P65-A-LOW-001 — 5 removed BCs have `replacement: null` in frontmatter but body declares multiple replacement BCs. Schema inconsistency between frontmatter scalar null and body YAML list.

**Fix applied:** Changed `replacement:` from `null` to YAML block-list form for all 5 files. Version bumped 2.2 → 2.3 on each file. Changelog row added to each.

## Files Modified

| BC File | Old replacement | New replacement (YAML list) | Version |
|---------|----------------|-----------------------------|---------|
| BC-2.01.001-single-client-sensor-query.md | null | [BC-2.11.001, BC-2.11.011] | 2.2 → 2.3 |
| BC-2.01.003-cursor-based-pagination.md | null | [BC-2.11.001, BC-2.07.001, BC-2.07.002] | 2.2 → 2.3 |
| BC-2.01.009-query-filtering-sorting.md | null | [BC-2.11.002, BC-2.11.003, BC-2.11.004, BC-2.11.007] | 2.2 → 2.3 |
| BC-2.01.011-cross-sensor-correlation-ocsf-fields.md | null | [BC-2.11.001, BC-2.11.005, BC-2.11.012] | 2.2 → 2.3 |
| BC-2.01.015-response-envelope-structure.md | null | [BC-2.11.001, BC-2.09.008] | 2.2 → 2.3 |

## Replacement Lists (Canonical)

```yaml
# BC-2.01.001
replacement:
  - BC-2.11.001
  - BC-2.11.011

# BC-2.01.003
replacement:
  - BC-2.11.001
  - BC-2.07.001
  - BC-2.07.002

# BC-2.01.009
replacement:
  - BC-2.11.002
  - BC-2.11.003
  - BC-2.11.004
  - BC-2.11.007

# BC-2.01.011
replacement:
  - BC-2.11.001
  - BC-2.11.005
  - BC-2.11.012

# BC-2.01.015
replacement:
  - BC-2.11.001
  - BC-2.09.008
```

## Schema Note

Single-BC `replacement:` fields (scalar string form, e.g. `replacement: "BC-2.18.xxx"`) are left unchanged per instruction. The YAML block-list form is used only for multi-BC cases where `null` was previously masking the list.

## Constraints Honored

- No commit performed
- No input-hash recomputed
- One Write per file (5 writes total)
- All other file content preserved verbatim
