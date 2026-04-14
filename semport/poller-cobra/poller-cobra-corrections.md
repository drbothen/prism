# Extraction Corrections -- poller-cobra

> Applied 2026-04-13 based on extraction validation findings.

---

## Correction 1: Production LOC (2,245 -> 2,259)

**Cause:** Original `wc -l` count was off by 14 lines across production files.

| File | Change |
|------|--------|
| `poller-cobra-pass-0-deep-inventory.md` | Production total row: 2,245 -> 2,259; Grand total: 2,926 -> 2,943 |
| `poller-cobra-pass-0-deep-inventory.md` | Novelty assessment paragraph: 2,245 -> 2,259, 681 -> 684, 2,926 -> 2,943 |
| `poller-cobra-pass-0-deep-inventory-r2.md` | LOC verification section: replaced "verified correct" with correction note |
| `poller-cobra-coverage-audit.md` | Cross-reference table: 2,245 -> 2,259 |
| `poller-cobra-pass-5-deep-conventions.md` | Test ratio line: 681/2,245 -> 684/2,259 |
| `poller-cobra-pass-5-deep-conventions-r2.md` | Test coverage percentage section: added corrected LOC note |

## Correction 2: Test LOC (681 -> 684)

**Cause:** Original `wc -l` count was off by 3 lines across test files.

| File | Change |
|------|--------|
| `poller-cobra-pass-0-deep-inventory.md` | Test total row: 681 -> 684 |
| `poller-cobra-coverage-audit.md` | Cross-reference table: 681 -> 684 |
| `poller-cobra-pass-5-deep-conventions.md` | Test ratio line: updated numerator |
| `poller-cobra-pass-5-deep-conventions-r2.md` | Added corrected LOC note inline |

## Correction 3: source.go Line Count (183 -> 184)

**Cause:** Off-by-one in original line count for `internal/crowdstrike/source.go`.

| File | Change |
|------|--------|
| `poller-cobra-pass-0-deep-inventory.md` | File manifest row: 183 -> 184 |
| `poller-cobra-pass-1-deep-architecture.md` | Novelty assessment: "(3 components, 183 lines)" -> "(3 components, 184 lines)" |
| `poller-cobra-pass-1-deep-architecture-r2.md` | Dead code verification: 183 -> 184 |
| `poller-cobra-pass-5-deep-conventions.md` | AP-001 location: "(183 lines)" -> "(184 lines)" |

## Correction 4: Pass 3 R2 Test Function Counts (10/10/8 -> 4/12/9)

**Cause:** Original Pass 3 R2 conflated subtests with top-level test functions. Coverage Audit and Pass 5 R2 had already established the correct counts (4 + 12 + 9 = 25 top-level).

| File | Change |
|------|--------|
| `poller-cobra-pass-3-deep-behavioral-contracts-r2.md` | Test coverage analysis section: corrected all three counts and added correction note explaining the subtest vs top-level distinction |
| `poller-cobra-pass-0-deep-inventory.md` | File manifest test rows: corrected function counts and descriptions to match authoritative values |

---

## Unchanged Values (Verified Correct)

- Individual file line counts for api_test.go (184), server_test.go (286), pprof_test.go (211) were not changed -- only the production total, test total, source.go, and test function counts were wrong.
- Test ratio percentage (30.3%) remains the same after correction (684/2259 = 30.28%).
- All other cross-reference consistency checks in the coverage audit remain valid.
