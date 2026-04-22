# Pass-72 PO Remediation Track

**Date:** 2026-04-20
**Agent:** product-owner
**Pass:** 72

---

## CRIT-001: Non-Monotonic Changelog Row Order — Class-Based Audit

### Audit Scope

Scanned all 207 BC files in `.factory/specs/behavioral-contracts/` using Python version-tuple comparison to detect any changelog sequence where version at row N < version at row N+1 (ascending = defect).

### Audit Findings

**18 files found with non-monotonic changelog row order** (7 known + 11 newly discovered).

Known files (per pass-72 issue list):
1. BC-2.01.001-single-client-sensor-query.md — had 2.4, 2.0, 2.1, 2.2, 2.3
2. BC-2.01.002-cross-client-fan-out.md — had 2.3, 2.0, 2.1, 2.2
3. BC-2.01.003-cursor-based-pagination.md — had 2.4, 2.0, 2.1, 2.2, 2.3
4. BC-2.01.009-query-filtering-sorting.md — had 2.4, 2.0, 2.1, 2.2, 2.3
5. BC-2.01.011-cross-sensor-correlation-ocsf-fields.md — had 2.4, 2.0, 2.1, 2.2, 2.3
6. BC-2.01.012-query-fingerprint-validation.md — had 2.3, 2.0, 2.1, 2.2
7. BC-2.01.015-response-envelope-structure.md — had 2.4, 2.0, 2.1, 2.2, 2.3

Newly discovered (class expansion):
8. BC-2.01.010-partial-failure-handling.md — had 1.3, 1.0, 1.1, 1.2
9. BC-2.04.005-hidden-tools-pattern.md — had 1.1, 1.2, 1.3 (ascending throughout)
10. BC-2.04.007-three-tier-risk-classification.md — had 1.0, 1.1, 1.2 (ascending throughout)
11. BC-2.04.009-confirmation-token-request.md — had 1.1, 1.2, 1.3 (ascending throughout)
12. BC-2.06.005-config-validation-multi-error.md — had 1.0, 1.1, 1.2 (ascending throughout)
13. BC-2.07.004-cache-invalidation-on-writes.md — had 3.3, 3.0, 3.1, 3.2
14. BC-2.09.003-suspicious-pattern-detection.md — had 1.3, 1.0, 1.1, 1.2
15. BC-2.09.004-safety-flag-parallel-fields.md — had 1.3, 1.0, 1.1, 1.2
16. BC-2.10.005-notifications-tools-list-changed.md — had 1.4, 1.0, 1.1, 1.2, 1.3
17. BC-2.12.001-create-schedule-tool.md — had 1.3, 1.0, 1.1, 1.2
18. BC-2.13.006-create-rule-tool.md — had 1.4, 1.0, 1.1, 1.2, 1.3

Note: BC-2.04.005, BC-2.04.007, BC-2.04.009, BC-2.06.005 had ascending-only sequences (no high-version row at top to create the "partial descending" pattern). These were still caught as non-monotonic because rows are not descending throughout.

### Fix Applied Per File

All 18 files received:
- Changelog rows reordered to fully descending (highest version at top)
- Version bumped by minor increment
- New top row added:
  `| {new_version} | pass-72-fix | 2026-04-20 | product-owner | Reordered changelog rows to fully descending (CRIT-001 class scope expansion from pass-71 MED-002 fix). |`

Version bumps:
| File | Pre-fix top version | Post-fix version |
|------|--------------------|--------------------|
| BC-2.01.001 | 2.4 | 2.5 |
| BC-2.01.002 | 2.3 | 2.4 |
| BC-2.01.003 | 2.4 | 2.5 |
| BC-2.01.009 | 2.4 | 2.5 |
| BC-2.01.010 | 1.3 | 1.4 |
| BC-2.01.011 | 2.4 | 2.5 |
| BC-2.01.012 | 2.3 | 2.4 |
| BC-2.01.015 | 2.4 | 2.5 |
| BC-2.04.005 | 1.3 | 1.4 |
| BC-2.04.007 | 1.2 | 1.3 |
| BC-2.04.009 | 1.3 | 1.4 |
| BC-2.06.005 | 1.2 | 1.3 |
| BC-2.07.004 | 3.3 | 3.4 |
| BC-2.09.003 | 1.3 | 1.4 |
| BC-2.09.004 | 1.3 | 1.4 |
| BC-2.10.005 | 1.4 | 1.5 |
| BC-2.12.001 | 1.3 | 1.4 |
| BC-2.13.006 | 1.4 | 1.5 |

---

## HIGH-001: Supplement Changelog Header `Notes` vs `Change`

### Pre-fix State

| File | Header before fix |
|------|------------------|
| test-vectors.md | `Notes` |
| nfr-catalog.md | `Notes` |
| error-taxonomy.md | `Change` (already correct — pass-71 fixed) |
| interface-definitions.md | `Change` (already correct — pass-71 fixed) |

### Fix Applied

Both supplements:
- Renamed `Notes` column header → `Change`
- Separator row updated from `-------|` to `--------|` (5-col canonical alignment)
- New pass-72-fix row added at top of changelog
- Frontmatter `version:` bumped

| File | Pre-fix version | Post-fix version |
|------|----------------|------------------|
| test-vectors.md | 2.4 | 2.5 |
| nfr-catalog.md | 1.1 | 1.2 |

---

## Cross-Corpus Check Results

### Supplements (prd-supplements/)
- `error-taxonomy.md`: uses `Change` — correct
- `interface-definitions.md`: uses `Change` — correct
- `test-vectors.md`: fixed this pass (Notes → Change)
- `nfr-catalog.md`: fixed this pass (Notes → Change)

### Stories (.factory/stories/)
- All stories checked use `Changes` (plural)
- Dominant convention in story corpus: `Changes`
- Assessment: separate corpus from supplements; story convention is independently established. No action taken.

### Verification Properties (.factory/specs/verification-properties/)
- All VP files sampled use `Notes`
- VPs are a separate corpus with their own convention
- Not in scope for this pass; flag for future VP-specific normalization sweep if desired

---

## Summary

- Total BC files scanned: 207
- BC files with non-monotonic changelog: 18 (7 known + 11 newly discovered)
- All 18 BCs fixed
- 2 supplements fixed (Notes → Change header)
- 2 supplements confirmed already correct from pass-71
- Stories: `Changes` (plural) — dominant convention, no change
- VPs: `Notes` — separate corpus, not in scope

**No commit. State-manager closer handles.**
