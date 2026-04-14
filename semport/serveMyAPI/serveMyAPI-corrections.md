---
document_type: correction-log
project: serveMyAPI
source: serveMyAPI-extraction-validation.md
timestamp: 2026-04-13
---

# Correction Log: serveMyAPI

All corrections derived from the extraction validation report at
`/Users/jmagady/Dev/prism/.factory/semport/serveMyAPI/serveMyAPI-extraction-validation.md`.

---

## Systematic Off-by-One File Line Counts

**Root cause:** The analyzer used a counting method that overcounted by 1 line per file (likely treating the file as 1-indexed or counting a trailing line that `wc -l` does not). Six files were affected.

| File | Old Value | Corrected Value | Files Edited |
|------|-----------|-----------------|--------------|
| `src/index.ts` | 158 | 157 | broad-sweep, pass-0-deep-inventory (Broad Sweep column), pass-0-deep-inventory-r2, coverage-audit |
| `src/server.ts` | 230 | 229 | broad-sweep, pass-0-deep-inventory (Broad Sweep column), pass-0-deep-inventory-r2, coverage-audit |
| `src/services/keychain.ts` | 198 | 197 | broad-sweep, pass-0-deep-inventory (Broad Sweep column), pass-0-deep-inventory-r2, coverage-audit |
| `Dockerfile` | 35 | 34 | broad-sweep, pass-0-deep-inventory, pass-0-deep-inventory-r2, coverage-audit |
| `build_dmg.sh` | 109 | 108 | broad-sweep, pass-0-deep-inventory, pass-0-deep-inventory-r2, coverage-audit |
| `.gitignore` | 40 | 39 | pass-0-deep-inventory, pass-0-deep-inventory-r2 |

**Note on code-location line references:** References like `keychain.ts:198` (singleton instantiation) and `server.ts:230` (`export { server }`) refer to actual source lines visible in `cat -n` output, not LOC counts. These references are correct and were NOT changed. The `wc -l` tool reports fewer lines when the final line lacks a trailing newline, but the content at those line numbers exists.

---

## Aggregate LOC Total

| Metric | Old Value | Corrected Value | Files Edited |
|--------|-----------|-----------------|--------------|
| Total source LOC (7 files) | 933 | 915 | pass-0-deep-inventory, pass-0-deep-inventory-r2 |
| TypeScript-only LOC (4 files) | 703 | 700 | pass-0-deep-inventory (text + delta summary) |

**Corrected sum:** 157 + 229 + 197 + 117 + 163 + 37 + 15 = 915

---

## Line Citation Corrections

| Citation | Old Value | Corrected Value | File Edited |
|----------|-----------|-----------------|-------------|
| OBS-3.05 server.ts source range | `server.ts:226-228` | `server.ts:226-230` | pass-3-deep-behavioral-contracts-r2 |

The original citation `226-228` only covered the `app.listen()` block. The observation describes both the listen call and the `export { server }` at line 230, so the range was extended to include it.

---

## Files Modified

1. `serveMyAPI-broad-sweep.md` -- 5 line count corrections (index.ts, server.ts, keychain.ts, Dockerfile, build_dmg.sh)
2. `serveMyAPI-pass-0-deep-inventory.md` -- 7 corrections (6 line counts + 2 aggregate LOC values + delta summary)
3. `serveMyAPI-pass-0-deep-inventory-r2.md` -- 7 corrections (6 line counts + aggregate LOC header)
4. `serveMyAPI-coverage-audit.md` -- 5 line count corrections (index.ts, server.ts, keychain.ts, Dockerfile, build_dmg.sh)
5. `serveMyAPI-pass-3-deep-behavioral-contracts-r2.md` -- 1 source citation correction (OBS-3.05)

## Files NOT Modified (and why)

- `serveMyAPI-extraction-validation.md` -- This is the validation report itself; it correctly documents the errors and their corrections. It is the source of truth, not a target for correction.
- `serveMyAPI-pass-3-deep-behavioral-contracts.md` -- Code-location references (e.g., `index.ts:152-158`, `server.ts:152-230`) point to lines that exist in `cat -n` output and are accurate.
- `serveMyAPI-pass-5-deep-conventions.md`, `serveMyAPI-pass-5-deep-conventions-r2.md` -- References to `keychain.ts:198` are code-location references to the singleton instantiation line, which exists at that line number.
- `serveMyAPI-pass-3-deep-behavioral-contracts-r2.md` (keychain.ts:198 references) -- Same reasoning; code-location, not LOC count.
- `serveMyAPI-pass-2-deep-domain-model.md`, `serveMyAPI-pass-2-deep-domain-model-r2.md` -- No LOC claims or affected line counts found.
- `serveMyAPI-pass-1-deep-architecture.md`, `serveMyAPI-pass-1-deep-architecture-r2.md` -- No LOC claims or affected line counts found.
- `serveMyAPI-pass-4-deep-nfr-catalog.md`, `serveMyAPI-pass-4-deep-nfr-catalog-r2.md` -- No LOC claims or affected line counts found.
