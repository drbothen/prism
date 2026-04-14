# Correction Log: poller-coaster

**Date:** 2026-04-13
**Basis:** poller-coaster-extraction-validation.md

---

## Corrections Applied

### 1. COLLECTOR_HEALTH_ADDR -> HEALTH_ADDR

The env var name was incorrectly written as `COLLECTOR_HEALTH_ADDR` in analysis files. The actual env var is `HEALTH_ADDR` (config.go:41 defines `collectorHealthAddrEnv = "HEALTH_ADDR"`). The `COLLECTOR_` prefix does not exist. This is operator-facing and would cause misconfiguration.

**Files corrected:**
- `poller-coaster-pass-1-deep-architecture-r2.md` -- Helm env var table (line 54), health address section (line 65), both corrected to `HEALTH_ADDR`

### 2. ARMIS_xxx_FIELDS env vars do not exist

Field lists (AlertFields, DeviceFields, etc.) are compile-time defaults in `DefaultConfig()`. There are zero env var constants or `os.Getenv` calls for them. They are NOT runtime-configurable. All references claiming these are env vars have been removed or corrected.

**Files corrected:**
- `poller-coaster-pass-0-deep-inventory-r2.md` -- Removed `ARMIS_xxx_FIELDS` from env var list (line 81), added clarifying note that fields are compile-time only
- `poller-coaster-pass-1-deep-architecture-r2.md` -- Mermaid diagram changed from `AQLFields["ARMIS_*_FIELDS (7x)"]` to `FieldDefaults["Field lists (compile-time defaults only, no env var)"]`; removed "(5 more AQL/LIMIT/FIELDS pairs)" reference to FIELDS; corrected "16+ env vars" to "9+ env vars" in impact statement and delta summary

### 3. Sentinel error count: 14 -> 15

`ErrConfigLoad` at `apperrors/errors.go:52-53` was omitted from all passes. It is defined but unused, making it the 5th "defined but unused" sentinel error (the analysis identified 4 but missed this one).

**Files corrected:**
- `poller-coaster-pass-0-deep-inventory.md` (R1) -- Table entry corrected from 14 to 15; corrections section updated to show 15
- `poller-coaster-pass-0-deep-inventory-r2.md` (R2) -- Sentinel count heading corrected from 14 to 15; ErrConfigLoad added as row 7 in table; unused error note updated from 4 to 5; delta summary updated
- `poller-coaster-pass-1-deep-architecture.md` (R1) -- Component catalog corrected from 14 to 15; error handling section corrected
- `poller-coaster-pass-4-deep-nfr-catalog-r2.md` -- Unused sentinel count corrected from 4 to 5; ErrConfigLoad added to list

### 4. Total Go files: 32 -> 33 (reverting R2's incorrect correction)

R2 "corrected" R1's count from 33 to 32, but R1 was right. `tools/tools.go` is the 22nd source file; R2 missed it in manual enumeration. Correct count: 33 total (22 source + 11 test).

**Files corrected:**
- `poller-coaster-pass-0-deep-inventory-r2.md` -- Correction note revised to acknowledge R1 was correct; file count table corrected to 33 (22+11); convergence declaration updated
- `poller-coaster-pass-2-deep-domain-model.md` -- Basis line corrected from "32 Go files" to "33 Go files"
- `poller-coaster-pass-3-deep-behavioral-contracts-r2.md` -- Checkpoint files_scanned corrected from 32 to 33
- `poller-coaster-coverage-audit.md` -- Cross-reference check and final assessment corrected from "32 Go files" to "33 Go files"
- `poller-coaster-broad-sweep.md` -- Key numbers corrected from "~32 Go source files" to "33 Go files (22 source + 11 test)"

### 5. Source Go files: 21 -> 22

`tools/tools.go` was excluded from R2's manual enumeration. It is a legitimate source file (Go build tag `tools`, pins golangci-lint + govulncheck). Correct count: 22 source files.

**Corrected as part of item 4 above** -- all "21 source" references updated to "22 source" where they appeared alongside the total count fixes.

---

## Files NOT modified

- `poller-coaster-extraction-validation.md` -- This is the validation report that documents the original inaccuracies. Its references to incorrect values are intentional (describing what was wrong vs. what is correct). No changes needed.
- `poller-coaster-pass-2-deep-domain-model-r2.md` -- No inaccurate references found.
- `poller-coaster-pass-3-deep-behavioral-contracts.md` -- No inaccurate references found.
- `poller-coaster-pass-5-deep-conventions.md` -- No inaccurate references found.
- `poller-coaster-pass-5-deep-conventions-r2.md` -- No inaccurate references found.
- `poller-coaster-pass-4-deep-nfr-catalog.md` -- No inaccurate references found.
