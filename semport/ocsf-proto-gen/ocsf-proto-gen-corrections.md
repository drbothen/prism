# Correction Log: ocsf-proto-gen

**Date:** 2026-04-13
**Trigger:** Extraction validation report (`ocsf-proto-gen-extraction-validation.md`)
**Scope:** All analysis files in `/Users/jmagady/Dev/prism/.factory/semport/ocsf-proto-gen/`

---

## Corrections Applied

### 1. Systematic Off-by-One in File Line Counts

**Root cause:** The Read tool displays lines 1-N where N equals the `wc -l` count. The analyst recorded N+1 (interpreting "up to line N" as "N+1 lines").

**Fix:** Decremented every file line count by 1 across all analysis files.

| File | Old | New | Files Edited |
|------|-----|-----|-------------|
| `src/main.rs` | 165 | 164 | broad-sweep, pass-0-R1, coverage-audit |
| `src/lib.rs` | 36 | 35 | broad-sweep, pass-0-R1, coverage-audit |
| `src/codegen.rs` | 640 | 639 | broad-sweep, pass-0-R1, coverage-audit |
| `src/schema.rs` | 389 | 388 | broad-sweep, pass-0-R1, coverage-audit |
| `src/type_map.rs` | 231 | 230 | broad-sweep, pass-0-R1, coverage-audit |
| `src/error.rs` | 46 | 45 | broad-sweep, pass-0-R1, coverage-audit |
| `tests/integration.rs` | 603 | 602 | broad-sweep, pass-0-R1, pass-0-R2, coverage-audit |
| `README.md` | 158 | 157 | broad-sweep, pass-0-R1, pass-0-R2, coverage-audit |
| `CLAUDE.md` | ~87 | 86 | broad-sweep, pass-0-R1, pass-0-R2, coverage-audit |
| `CHANGELOG.md` | 30 | 29 | broad-sweep, pass-0-R1, pass-0-R2, coverage-audit |
| `CONTRIBUTING.md` | 40 | 39 | pass-0-R1, pass-0-R2, coverage-audit |
| `INGESTION.md` | 613 | 612 | pass-0-R1, pass-0-R2, coverage-audit |
| `LICENSE` | 21 | 21 | (no change -- see correction #6 below) |
| `Cargo.toml` | 38 | 37 | broad-sweep (was already 37 in R1) |
| `.github/workflows/ci.yml` | 67 | 66 | pass-0-R1, coverage-audit |
| `.github/workflows/release.yml` | 50 | 49 | pass-0-R1, pass-0-R2, coverage-audit |
| `.github/workflows/validate-codeowners.yml` | 29 | 28 | pass-0-R1, pass-0-R2, coverage-audit |

### 2. Source LOC Subtotal

**Old:** 1,507
**New:** 1,501 (164+35+639+388+230+45)
**Files edited:** pass-0-R1, pass-0-R2

### 3. Test Count: type_map.rs

**Old:** 10 unit tests
**New:** 12 unit tests (`screaming_snake_conversion` and `sanitize_object_name_strips_prefix` were missed)
**Files edited:** pass-0-R1, pass-0-R2, pass-5-R1

### 4. Test Count: integration.rs

**Old:** 8 integration tests
**New:** 9 integration tests (`empty_object_type_emits_string` at line 551 was missed)
**Files edited:** pass-0-R1, pass-0-R2, broad-sweep

### 5. Total Test Count

**Old:** 22 (21 runnable + 1 compile-check)
**New:** 25 (24 runnable + 1 compile-check)
**Files edited:** pass-0-R1, pass-0-R2, broad-sweep, pass-4-R1, pass-5-R2, coverage-audit

### 6. LICENSE Line Count (Revert R2 Overcorrection)

**R1 value:** 21 (correct)
**R2 "correction":** 22 (wrong -- R2 misinterpreted the Read tool's "lines 1-22" display as 22 lines when `wc -l` = 21)
**Fix:** Reverted to 21. Updated pass-0-R2 to mark as VERIFIED instead of corrected. Updated coverage-audit.

### 7. BC-T.02.001: Test Fixture Attribute Count

**Old:** "1 class with 8 attributes"
**New:** "1 class with 9 attributes" (the `time` field with `timestamp_t` type was omitted from the count)
**Files edited:** pass-3-R2, pass-2-R1, coverage-audit

### 8. Diagnostic Message Count (Revert R2 Overcorrection)

**R1 value:** 14 `eprintln!` calls (correct: main.rs=10, schema.rs=2, codegen.rs=2)
**R2 "correction":** 13 (wrong -- R2 miscounted main.rs as having 9 calls instead of 10)
**Fix:** Reverted to 14. Updated pass-4-R2 to mark as VERIFIED instead of corrected.

### 9. Pass 5 Test Name Count

**Old:** "23 test names cataloged"
**New:** "24 test names cataloged" (the list in Pass 5 R1 actually enumerates 24 names)
**Files edited:** pass-5-R1

### 10. Grand Total Line Counts

Updated the Pass 0 R1 grand total table to reflect corrected per-file counts:
- CI/CD: 143 (66+49+28)
- Documentation: 944 (157+86+29+39+612+21)
- Total: ~3,242

---

## Files Modified

| File | Corrections Applied |
|------|-------------------|
| `ocsf-proto-gen-broad-sweep.md` | Line counts (#1), test counts (#3, #4, #5) |
| `ocsf-proto-gen-pass-0-deep-inventory.md` | Line counts (#1, #2, #10), test counts (#3, #4, #5) |
| `ocsf-proto-gen-pass-0-deep-inventory-r2.md` | Line counts (#1), LICENSE revert (#6), test counts (#3, #4, #5), subtotal (#2) |
| `ocsf-proto-gen-pass-2-deep-domain-model.md` | Attribute count (#7) |
| `ocsf-proto-gen-pass-3-deep-behavioral-contracts-r2.md` | Attribute count (#7) |
| `ocsf-proto-gen-pass-4-deep-nfr-catalog.md` | Test count (#5) |
| `ocsf-proto-gen-pass-4-deep-nfr-catalog-r2.md` | Diagnostic count revert (#8) |
| `ocsf-proto-gen-pass-5-deep-conventions.md` | Test count (#3), test name count (#9) |
| `ocsf-proto-gen-pass-5-deep-conventions-r2.md` | Test count (#5) |
| `ocsf-proto-gen-coverage-audit.md` | Line counts (#1, #6), test count (#5), attribute count (#7) |

## Files NOT Modified

| File | Reason |
|------|--------|
| `ocsf-proto-gen-extraction-validation.md` | This is the validation report itself; it correctly documents the original "Claimed" vs "Recounted" values |
| `ocsf-proto-gen-pass-1-deep-architecture.md` | No line count or test count claims requiring correction |
| `ocsf-proto-gen-pass-1-deep-architecture-r2.md` | No line count or test count claims requiring correction |
| `ocsf-proto-gen-pass-2-deep-domain-model-r2.md` | No incorrect claims found |
| `ocsf-proto-gen-pass-3-deep-behavioral-contracts.md` | No incorrect claims found (attribute count claim is in R2, not R1) |
