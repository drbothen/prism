---
document_type: correction-log
project: poller-express
generated: 2026-04-13T00:00:00Z
source: poller-express-extraction-validation.md
---

# Correction Log: poller-express

All corrections derived from the extraction validation report. Each entry identifies the inaccuracy, the files edited, and the exact change made.

---

## Correction 1: Sentinel error total count (14 -> 15)

**Inaccuracy:** Declared sentinel error count was stated as 14; actual count is 15.

**Files edited:**
- `poller-express-pass-0-deep-inventory.md` -- File manifest row for `apperrors/errors.go`: changed "14 sentinel errors" to "15 sentinel errors (10 active, 5 unused)". Delta summary: changed "14 sentinel errors" to "15 sentinel errors". Novelty assessment: changed "total 14 vs. 10" to "total 15 vs. 10".
- `poller-express-pass-0-deep-inventory-r2.md` -- Sentinel Error Count section: changed "14 `var` declarations. Correct." to "**15** `var` declarations (not 14). Round 1 undercounted by 1."
- `poller-express-pass-1-deep-architecture.md` -- Error Handling Architecture diagram: changed "apperrors (14 sentinel errors)" to "apperrors (15 sentinel errors: 10 active, 5 unused)".

---

## Correction 2: Active sentinel error count (9 -> 10)

**Inaccuracy:** Active sentinel error count was stated as 9; actual count is 10. `ErrSinkRequestBuild` was omitted from the active count summary despite being listed in the narrative.

**Files edited:**
- `poller-express-pass-0-deep-inventory-r2.md` -- Changed "active sentinel error count is **9**" to "**10**". Changed section heading "Active Sentinel Errors (9)" to "(10)". Updated delta summary from "corrected to 9" to "corrected to 10" and from "9 active + 5 unused" to "15 total = 10 active + 5 unused".

---

## Correction 3: Hand-written Go LOC (~1,500 -> 3,754)

**Inaccuracy:** Hand-written Go LOC was estimated as "~1,500"; actual count is 3,754 (2.5x underestimate).

**Files edited:**
- `poller-express-broad-sweep.md` -- Executive summary: changed "~1,500 LOC of hand-written Go" to "~3,700 LOC of hand-written Go".
- `poller-express-pass-0-deep-inventory.md` -- LOC Claims Verification section: replaced the "approximate" hedge with the corrected counts (3,754 hand-written, 35,864 generated).

---

## Correction 4: Generated Go LOC (~10,000+ -> 35,864)

**Inaccuracy:** Generated client LOC was estimated as "~10,000+"; actual count is 35,864 (3.6x underestimate).

**Files edited:**
- `poller-express-broad-sweep.md` -- Module boundaries table: changed "~10,000+" to "~36,000". Executive summary: changed "~100+ model files" to "124 model files, ~36,000 LOC".
- `poller-express-pass-0-deep-inventory-r2.md` -- Generated Client File Count section: changed from "reasonable but cannot be precisely verified" to noting the actual 124 files and 35,864 LOC.

---

## Correction 5: Indirect dependencies (13 -> 21)

**Inaccuracy:** Indirect dependency count in go.mod was stated as 13; actual count is 21.

**Files edited:**
- `poller-express-pass-0-deep-inventory.md` -- Non-Go File Manifest row for `go.mod`: changed "13 indirect" to "21 indirect".

---

## Correction 6: Makefile targets (11 -> 13)

**Inaccuracy:** Makefile target count was stated as 11; actual count is 13. The `all` and `generate` targets were omitted from the list.

**Files edited:**
- `poller-express-pass-0-deep-inventory.md` -- Non-Go File Manifest row for `Makefile`: changed "11 targets: help, build, test, fmt, lint, vuln, clean, deps, get, run, vector" to "13 targets: help, all, build, test, fmt, lint, vuln, clean, deps, get, run, vector, generate".
- `poller-express-coverage-audit.md` -- BS-005 section: changed "11 targets" to "13 targets" and "12 targets" to "13 targets" (the listed items already numbered 13 but the count said 12).

---

## Correction 7: BC-6.003 precondition description

**Inaccuracy:** BC-6.003 stated the precondition as "Both CYBERINT_API_URL and CYBERINT_API_KEY empty", implying a combined check. The code checks them sequentially and independently -- either missing triggers its own distinct error, with API key checked before URL.

**Files edited:**
- `poller-express-pass-3-deep-behavioral-contracts.md` -- BC-6.003: Changed precondition from "Both CYBERINT_API_URL and CYBERINT_API_KEY empty" to "CYBERINT_API_KEY empty, or CYBERINT_API_URL empty (checked sequentially -- key first, then URL)". Updated postconditions to describe the sequential independent behavior. Added `config.go:168-173` to evidence.

---

## Additional correction: Distroless user UID presentation

**Inaccuracy (from validation report, not in known corrections list):** Multiple files stated "nonroot user 65532" as if the numeric UID were directly coded. The Dockerfile uses `USER nonroot:nonroot` symbolically; 65532 is the conventional UID for the distroless nonroot image but is not stated in source.

**Files edited:**
- `poller-express-broad-sweep.md` -- Deployment topology: changed "nonroot user 65532" to "nonroot user, conventionally UID 65532".
- `poller-express-pass-0-deep-inventory.md` -- Dockerfile manifest row: changed "nonroot user 65532" to "nonroot user, conventionally UID 65532".
- `poller-express-pass-1-deep-architecture.md` -- Security posture table: changed "User 65532 (nonroot)" to "User nonroot (conventionally UID 65532)".
- `poller-express-pass-4-deep-nfr-catalog.md` -- Container hardening table: changed "65532 (nonroot)" to "nonroot (conventionally UID 65532)".
