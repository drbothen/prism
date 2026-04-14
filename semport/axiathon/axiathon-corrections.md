# Axiathon Extraction Corrections Log

**Date:** 2026-04-13
**Source:** axiathon-extraction-validation.md
**Corrections applied:** 6

---

## Correction 1: Tracing call sites in spike (63/17 -> 66/20)

**Original claim:** 63 tracing calls in 17 files
**Corrected to:** 66 tracing calls in 20 files
**Files edited:**
- `axiathon-pass-1-deep-architecture.md` line 340: updated "17 spike files (63 tracing::info/warn/error/debug calls)" to "20 spike files (66 ...)"
- `axiathon-pass-1-deep-architecture.md` delta summary: updated "63 tracing calls" to "66 tracing calls in 20 files"
- `axiathon-pass-1-deep-architecture-r2.md` line 26: updated verification table from "63 tracing call sites in 17 files" to "66 tracing call sites in 20 files"
- `axiathon-pass-4-deep-nfr-catalog.md` line 120: updated "17 files, 63 call sites" to "20 files, 66 call sites"
- `axiathon-pass-4-deep-nfr-catalog-r2.md` line 34: updated verification table from "63 tracing calls in 17 files" to "66 tracing calls in 20 files"

## Correction 2: Production parser test count (~60 -> 189)

**Original claim:** ~60 tests
**Corrected to:** 189 tests (parser_test.rs); 315 total in tests/
**Files edited:**
- `axiathon-pass-2-deep-domain-model-r4.md` line 378: updated comparison table row "Test count" from "~60 tests | 16 tests" to "189 tests (parser_test.rs); 315 total in tests/ | 20 tests"

## Correction 3: Spike parser test count (16 -> 20)

**Original claim:** 16 tests
**Corrected to:** 20 tests
**Files edited:**
- `axiathon-pass-2-deep-domain-model-r4.md` line 378: updated in same row as Correction 2 (spike column changed from "16 tests" to "20 tests")
**Note:** The "16 tests" in `axiathon-pass-3-deep-behavioral-contracts.md` line 241 refers to aliases_test.rs (a different file), not the spike parser. That value was not changed.

## Correction 4: spike/axiathon-query LOC (~400 -> 1,944)

**Original claim:** ~400
**Corrected to:** ~1,944
**Files edited:**
- `axiathon-broad-sweep.md` line 57: updated from "~400" to "~1,944" and expanded purpose to include query planner and Pest-based parser

## Correction 5: spike/axiathon-core LOC (~1,200 -> 2,977)

**Original claim:** ~1200
**Corrected to:** ~2,977
**Files edited:**
- `axiathon-broad-sweep.md` line 54: updated from "~1200" to "~2,977" and expanded purpose to include proto integration
- `axiathon-pass-0-deep-inventory-r2.md` line 45: updated from "~1200 LOC / PLAUSIBLE" to "~2,977 LOC / VERIFIED"

## Correction 6: Total public type count (174 -> 206)

**Original claim:** 174 public types
**Corrected to:** 206 public types (165 spike + 41 production)
**Files edited:**
- `axiathon-pass-2-deep-domain-model-r4.md` line 393: updated "all 167 public types" to "all 206 public types (165 spike + 41 production)"
- `axiathon-pass-2-deep-domain-model-r4.md` line 397: updated Production crates row from 42 to 41 total types
- `axiathon-pass-2-deep-domain-model-r4.md` line 408: updated total from "174 public types" to "206 public types (165 spike + 41 production)" with correction note; updated remaining from 60 to 92
- `axiathon-pass-2-deep-domain-model-r4.md` line 410: updated "60 uncataloged types" to "92 uncataloged types"

---

## Files Not Modified (no inaccurate values found)

- `axiathon-pass-0-deep-inventory.md` -- no numeric claims matching corrections
- `axiathon-pass-2-deep-domain-model.md` (R1) -- no affected values
- `axiathon-pass-2-deep-domain-model-r2.md` -- no affected values
- `axiathon-pass-2-deep-domain-model-r3.md` -- no affected values
- `axiathon-pass-3-deep-behavioral-contracts.md` -- "16 tests" refers to aliases_test.rs, not spike parser
- `axiathon-pass-3-deep-behavioral-contracts-r2.md` -- no affected values
- `axiathon-pass-5-deep-conventions.md` -- no affected values
- `axiathon-pass-5-deep-conventions-r2.md` -- no affected values
- `axiathon-coverage-audit.md` -- no affected values
- `axiathon-extraction-validation.md` -- this is the validation report itself; not modified
