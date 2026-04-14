# Correction Log: mcp-claroty-xdome

**Date:** 2026-04-13
**Source:** mcp-claroty-xdome-extraction-validation.md
**Scope:** All analysis files in .factory/semport/mcp-claroty-xdome/

---

## Corrections Applied

### 1. Test file count: 36 -> 35

**Root cause:** Pass 0 R2 double-counted `tests/setup.ts` by treating it as separate from the 34 `.ts` files, but it was already included in that count. Actual: 34 `.ts` + 1 `.ts.disabled` = 35 total.

| File | Line/Section | Old Value | New Value |
|------|-------------|-----------|-----------|
| `mcp-claroty-xdome-pass-0-deep-inventory.md` | Test inventory table | `34 active + 1 disabled + 1 setup = 36` | `33 active + 1 disabled + 1 setup = 35` |
| `mcp-claroty-xdome-pass-0-deep-inventory.md` | Undercount note | `undercounts by 2` | `undercounts by 1` |
| `mcp-claroty-xdome-pass-0-deep-inventory.md` | Delta summary | `test count corrected from 34 to 36` | `test count corrected from 34 to 35` |
| `mcp-claroty-xdome-pass-0-deep-inventory-r2.md` | R1 claim audit (test count) | `35 TS files + setup.ts = 36 total` | `34 .ts + 1 .ts.disabled = 35 total` |
| `mcp-claroty-xdome-pass-0-deep-inventory-r2.md` | Delta summary | `36 detailed was correct` | `actual is 35: 34 .ts + 1 .ts.disabled` |
| `mcp-claroty-xdome-coverage-audit.md` | Directory overview table | `36 files` | `35 files` |
| `mcp-claroty-xdome-coverage-audit.md` | Coverage summary table | `36 / 28 / 8` | `35 / 28 / 7` |

### 2. Active .ts test files: 35 -> 34

| File | Line/Section | Old Value | New Value |
|------|-------------|-----------|-----------|
| `mcp-claroty-xdome-pass-5-deep-conventions-r2.md` | File naming audit | `All 35 test files` | `All 34 active .ts test files` |

### 3. Composite GitHub actions: ~18 -> 19

| File | Line/Section | Old Value | New Value |
|------|-------------|-----------|-----------|
| `mcp-claroty-xdome-pass-0-deep-inventory.md` | Delta summary | `18+ composite actions` | `19 composite actions` |
| `mcp-claroty-xdome-pass-1-deep-architecture.md` | Delta summary | `18+ actions` | `19 actions` |
| `mcp-claroty-xdome-pass-1-deep-architecture-r2.md` | R1 claim audit | `~18. The "18+" claim is approximately correct. CONFIRMED.` | `19. CORRECTED from ~18 to 19.` |
| `mcp-claroty-xdome-pass-1-deep-architecture-r2.md` | Delta summary | (no composite action note) | Added `composite actions corrected from ~18 to 19` |

### 4. Total repo files: ~440+ / 570+ -> 897

| File | Line/Section | Old Value | New Value |
|------|-------------|-----------|-----------|
| `mcp-claroty-xdome-pass-0-deep-inventory.md` | Total codebase files | `~440+` | `897` |
| `mcp-claroty-xdome-pass-0-deep-inventory.md` | Novelty assessment | `~440+ files across two languages` | `897 files across two languages` |
| `mcp-claroty-xdome-pass-0-deep-inventory-r2.md` | R1 claim audit (file count) | `~433 files minimum. The "~440+" figure is reasonable...CONFIRMED.` | `897 files total...CORRECTED to 897.` |
| `mcp-claroty-xdome-coverage-audit.md` | Total file count | `570+ files` | `897 files` |

### 5. docs/ files: 200+ / ~100+ -> 535

| File | Line/Section | Old Value | New Value |
|------|-------------|-----------|-----------|
| `mcp-claroty-xdome-pass-0-deep-inventory.md` | Directory comparison table | `~100+ files` | `535 files` |
| `mcp-claroty-xdome-coverage-audit.md` | Directory overview table | `200+ files` | `535 files` |
| `mcp-claroty-xdome-coverage-audit.md` | Coverage summary table | `200+ / 0 / 5 / 195+ / 3%` | `535 / 0 / 5 / 530 / 1%` |

### 6. .windsurf/ files: ~52 / 60+ -> 68

| File | Line/Section | Old Value | New Value |
|------|-------------|-----------|-----------|
| `mcp-claroty-xdome-pass-0-deep-inventory.md` | Directory comparison table | `~52 files` | `68 files` |
| `mcp-claroty-xdome-pass-0-deep-inventory.md` | .windsurf/ section heading | `52 files` | `68 files` |
| `mcp-claroty-xdome-pass-0-deep-inventory.md` | Delta summary | `52 files` | `68 files` |
| `mcp-claroty-xdome-pass-5-deep-conventions.md` | AI assessment | `52 files` | `68 files` |
| `mcp-claroty-xdome-pass-5-deep-conventions.md` | Novelty assessment | `52 files` | `68 files` |
| `mcp-claroty-xdome-pass-5-deep-conventions-r2.md` | R1 claim correction | `60+ files` | `68 files` |
| `mcp-claroty-xdome-pass-5-deep-conventions-r2.md` | Convention completeness table | `52+ Windsurf files` | `68 Windsurf files` |
| `mcp-claroty-xdome-pass-5-deep-conventions-r2.md` | Delta summary | `52 to 60+` | `52 to 68` |
| `mcp-claroty-xdome-coverage-audit.md` | Directory overview table | `60+ files` | `68 files` |
| `mcp-claroty-xdome-coverage-audit.md` | Coverage summary table | `60+ / 0 / 10 / 50+ / 17%` | `68 / 0 / 10 / 58 / 15%` |

### 7. Test LOC upper bound: 5,000 -> 5,748

| File | Line/Section | Old Value | New Value |
|------|-------------|-----------|-----------|
| `mcp-claroty-xdome-pass-0-deep-inventory-r2.md` | Test LOC estimate | `4,000-5,000 lines` | `5,500-6,000 lines (verified: 5,748 actual)` |

---

## Files Modified

1. `mcp-claroty-xdome-pass-0-deep-inventory.md` (7 edits)
2. `mcp-claroty-xdome-pass-0-deep-inventory-r2.md` (4 edits)
3. `mcp-claroty-xdome-pass-1-deep-architecture.md` (1 edit)
4. `mcp-claroty-xdome-pass-1-deep-architecture-r2.md` (2 edits)
5. `mcp-claroty-xdome-pass-5-deep-conventions.md` (2 edits)
6. `mcp-claroty-xdome-pass-5-deep-conventions-r2.md` (4 edits)
7. `mcp-claroty-xdome-coverage-audit.md` (5 edits)

## Files NOT Modified

- `mcp-claroty-xdome-extraction-validation.md` -- This is the validation report documenting the errors; its references to incorrect values are intentional (documenting what was wrong vs. what is correct).
- `mcp-claroty-xdome-broad-sweep.md` -- The broad sweep's "34 test files" heading is the original pre-correction value. The validation report documents this as a known error that was partially corrected in R1 (overcorrected to 36). The broad sweep value is left as historical context.
- `mcp-claroty-xdome-pass-2-deep-domain-model.md` -- No inaccurate metrics found.
- `mcp-claroty-xdome-pass-2-deep-domain-model-r2.md` -- No inaccurate metrics found.
- `mcp-claroty-xdome-pass-3-deep-behavioral-contracts.md` -- No inaccurate metrics found.
- `mcp-claroty-xdome-pass-3-deep-behavioral-contracts-r2.md` -- No inaccurate metrics found.
- `mcp-claroty-xdome-pass-4-deep-nfr-catalog.md` -- No inaccurate metrics found.
- `mcp-claroty-xdome-pass-4-deep-nfr-catalog-r2.md` -- No inaccurate metrics found.
