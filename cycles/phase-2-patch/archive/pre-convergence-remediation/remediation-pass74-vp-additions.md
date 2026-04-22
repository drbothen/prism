---
document_type: remediation-manifest
cycle: phase-2-patch
pass: 74
track: CRIT-002
producer: architect
timestamp: 2026-04-20T18:00:00
---

# Remediation Pass 74 — CRIT-002 VP Additions (VP-051 through VP-059)

## Objective

Complete VP-INDEX arithmetic reconciliation and propagate 9 new VP entries (VP-051 through
VP-059) into all downstream architecture documents. Addresses CRIT-002: VP-INDEX count
discrepancy between 50-VP baseline and 59-VP actual state.

## VP-INDEX Final State

**Version:** 1.7 (already correct prior to this pass — all 9 VPs present in the table)

**Arithmetic verified (both directions):**

| Method | Count | P0 | P1 |
|--------|-------|----|----|
| Kani | 26 | 20 | 6 |
| Proptest | 25 | 15 | 10 |
| Fuzz | 6 | 5 | 1 |
| Integration test | 2 | 2 | 0 |
| **Total** | **59** | **42** | **17** |

P0 + P1 = 42 + 17 = 59 ✓
Per-method sum = 26 + 25 + 6 + 2 = 59 ✓

## Per-Module VP Counts (verified)

| Module | Kani | Proptest | Fuzz | Integration | Total |
|--------|------|----------|------|-------------|-------|
| prism-core | 10 | 2 | 0 | 0 | 12 |
| prism-security | 5 | 1 | 1 | 0 | 7 |
| prism-query | 4 | 2 | 2 | 0 | 8 |
| prism-ocsf | 0 | 2 | 1 | 0 | 3 |
| prism-operations | 3 | 6 | 1 | 0 | 10 |
| prism-spec-engine | 2 | 6 | 1 | 0 | 9 |
| prism-credentials | 0 | 2 | 0 | 0 | 2 |
| prism-persistence | 1 | 2 | 0 | 0 | 3 |
| prism-audit | 1 | 1 | 0 | 0 | 2 |
| prism-dtu-crowdstrike | 0 | 0 | 0 | 2 | 2 |
| prism-mcp | 0 | 1 | 0 | 0 | 1 |
| prism-bin | 0 | 0 | 0 | 0 | 0 |
| prism-sensors | 0 | 0 | 0 | 0 | 0 |
| **Totals** | **26** | **25** | **6** | **2** | **59** |

Module sum = 12+7+8+3+10+9+2+3+2+2+1 = 59 ✓
Kani column = 10+5+4+0+3+2+0+1+1+0+0 = 26 ✓
Proptest column = 2+1+2+2+6+6+2+2+1+0+1 = 25 ✓
Fuzz column = 0+1+2+1+1+1+0+0+0+0+0 = 6 ✓
Integration column = 0+0+0+0+0+0+0+0+0+2+0 = 2 ✓

## New VPs (VP-051 through VP-059)

| ID | Property | Module | Method | Priority | Source |
|----|----------|--------|--------|----------|--------|
| VP-051 | Case state machine: exhaustive 5x5 transition table — 12 accept, 13 reject | prism-core | kani | P0 | DI-025 |
| VP-052 | update_case: disposition applied before status transition in single-call update | prism-core | proptest | P0 | BC-4.06.001 |
| VP-053 | Resolved case always has non-null disposition; transition rejects without disposition | prism-core | kani | P0 | BC-4.06.002 |
| VP-054 | TTR uses first resolution timestamp across reopen cycles; null aggregate when no resolved cases | prism-core | proptest | P1 | BC-4.06.003 |
| VP-055 | StorageEngine put_batch atomicity and domain isolation (MockStorageEngine) | prism-persistence | proptest | P1 | DI-033 |
| VP-056 | Audit buffer overflow purge: oldest entries deleted, newest preserved, purge-event produced | prism-audit | proptest | P1 | BC-2.05.010 |
| VP-057 | Crash recovery: denylist triggered at consecutive_crashes >= 3; exact threshold | prism-persistence | kani | P0 | DI-034 |
| VP-058 | Watchdog memory grace period: single check does not terminate; two consecutive checks do | prism-persistence | proptest | P0 | DI-027 |
| VP-059 | Spec validator: all errors collected (no fail-fast); warning-only specs return Ok | prism-spec-engine | proptest | P1 | DI-030 |

P0 additions: VP-051, VP-052, VP-053, VP-057, VP-058 (5 new P0 → 37+5 = 42 ✓)
P1 additions: VP-054, VP-055, VP-056, VP-059 (4 new P1 → 13+4 = 17 ✓)

## Artifacts Updated

### VP-INDEX.md
- **Version:** 1.7 (was already 1.7 with all 59 VPs present; Summary table already correct)
- **Status:** No changes needed — already consistent.

### verification-architecture.md
- **Version bumped:** 1.2 → 1.3
- **Changes:**
  - Added VP-051 through VP-059 to Provable Properties Catalog table (9 rows)
  - P0 list updated: 37 → 42 total (added VP-051, VP-052, VP-053, VP-057, VP-058)
  - P1 list updated: 13 → 17 total (added VP-054, VP-055, VP-056, VP-059)
  - Mermaid diagram "50 Verified Properties" → "59 Verified Properties"
  - Tier 1 label updated to note 26 Kani properties

### verification-coverage-matrix.md
- **Version bumped:** 1.3 → 1.4
- **Changes:**
  - Totals section corrected: was showing stale 50-VP baseline (Kani=23, Proptest=19, Total=50, P0=37, P1=13)
  - Updated to: Kani=26, Proptest=25, Fuzz=6, Integration=2, Total=59, P0=42, P1=17
  - Per-module Coverage table was already correct (showing 59-VP state)
  - Changelog rows added for v1.3 and v1.4

## Invariant Check

Three arithmetic invariants all satisfied:

1. Per-method sum: 26+25+6+2 = 59 ✓
2. Per-priority sum: 42+17 = 59 ✓
3. Per-module sum: 12+7+8+3+10+9+2+3+2+2+1 = 59 ✓

Additionally: Kani column sum across modules (26) = method total Kani (26) ✓
Proptest column sum across modules (25) = method total Proptest (25) ✓
