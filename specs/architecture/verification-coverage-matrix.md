---
document_type: architecture-section
level: L3
section: "verification-coverage-matrix"
version: "1.26"
status: draft
producer: architect
timestamp: 2026-04-20T18:00:00
phase: 1b
inputs: [prd.md, domain-spec/invariants.md]
traces_to: ARCH-INDEX.md
---

# Verification Coverage Matrix

## [Section Content]

See detailed tables below.

## Coverage by Module

| Module | Criticality | Kani Proofs | Proptest | Unit Tests | Fuzz Targets | Integration Tests | Coverage Target | VPs |
|--------|------------|-------------|----------|------------|-------------|-------------------|----------------|-----|
| prism-core | CRITICAL | 13 | 8 | 0 | 0 | 0 | 95% | VP-001, VP-002, VP-003, VP-004, VP-005, VP-006, VP-011, VP-029, VP-051, VP-053, VP-065, VP-070, VP-071 (kani); VP-063, VP-064, VP-069, VP-072, VP-073, VP-074, VP-075, VP-076 (proptest) |
| prism-security | CRITICAL | 5 | 1 | 0 | 1 | 0 | 90% | VP-007, VP-008, VP-009, VP-010, VP-020 (kani); VP-024 (proptest); VP-038 (fuzz) |
| prism-query | CRITICAL | 4 | 2 | 0 | 2 | 0 | 90% | VP-012, VP-014, VP-015, VP-025 (kani); VP-013, VP-031 (proptest); VP-021, VP-037 (fuzz) |
| prism-ocsf | CRITICAL | 0 | 2 | 0 | 1 | 0 | 90% | VP-016, VP-017 (proptest); VP-022 (fuzz) |
| prism-operations | HIGH | 3 | 17 | 0 | 1 | 0 | 85% | VP-026, VP-030, VP-044 (kani); VP-018, VP-019, VP-027, VP-045, VP-046, VP-047, VP-052, VP-054, VP-060, VP-137, VP-138, VP-139, VP-140, VP-141, VP-142, VP-143, VP-145 (proptest); VP-028 (fuzz) |
| prism-spec-engine | HIGH | 2 | 14 | 4 | 1 | 1 | 85% | VP-040, VP-048 (kani); VP-032, VP-041, VP-042, VP-043, VP-049, VP-059, VP-099, VP-100, VP-101, VP-102, VP-103, VP-104, VP-105, VP-106 (proptest); VP-095, VP-096, VP-097, VP-098 (unit_test); VP-023 (fuzz); VP-107 (integration_test) |
| prism-sensors | HIGH | 0 | 10 | 0 | 0 | 2 | 80% | VP-077, VP-078, VP-079, VP-080, VP-087, VP-088, VP-089, VP-091, VP-092, VP-093 (proptest); VP-090, VP-094 (integration_test) |
| prism-credentials | CRITICAL | 0 | 7 | 0 | 0 | 1 | 90% | VP-034, VP-035, VP-081, VP-082, VP-084, VP-085, VP-086 (proptest); VP-083 (integration_test) |
| prism-storage | HIGH | 1 | 2 | 0 | 0 | 0 | 80% | VP-057 (kani); VP-055, VP-058 (proptest) |
| prism-audit | HIGH | 1 | 3 | 0 | 0 | 1 | 80% | VP-039 (kani); VP-056, VP-066, VP-067 (proptest); VP-068 (integration_test) |
| prism-dtu-crowdstrike | HIGH | 0 | 0 | 0 | 0 | 2 | 75% | VP-033, VP-036 (integration_test) |
| prism-mcp | HIGH | 0 | 3 | 0 | 0 | 0 | 75% | VP-050, VP-061, VP-062 (proptest) |
| prism-dtu-common | HIGH | 1 | 11 | 0 | 0 | 2 | 80% | VP-108 (kani); VP-109, VP-110, VP-111, VP-113, VP-114, VP-116, VP-117, VP-118, VP-119, VP-120, VP-121 (proptest); VP-112, VP-115 (integration_test) |
| prism-dtu-harness | HIGH | 0 | 4 | 0 | 0 | 8 | 80% | VP-122, VP-123, VP-125, VP-128 (proptest); VP-124, VP-126, VP-127, VP-129, VP-130, VP-131, VP-132, VP-133 (integration_test) |
| prism-bin | LOW | 0 | 1 | 0 | 0 | 2 | 60% | VP-135 (proptest); VP-134, VP-136 (integration_test) |
| prism-siem-formats | HIGH | 0 | 1 | 0 | 0 | 0 | 80% | VP-144 (proptest) *(new in Wave 4 per ADR-019)* |
| **Totals** | | **30** | **86** | **4** | **6** | **19** | | **145** |

## Totals

| Method | Planned Count | P0 | P1 |
|--------|--------------|----|----|
| Kani proofs | 30 | 23 | 7 |
| Proptest properties | 86 | 65 | 21 |
| Unit test VPs | 4 | 4 | 0 |
| Fuzz targets | 6 | 5 | 1 |
| Integration test VPs | 19 | 17 | 2 |
| **Total VPs** | **145** | **114** | **31** |

<!-- P0/P1 per-method breakdown from VP-INDEX v1.22 recount (Wave 4 Phase 3 ADR burst):
     Kani: 23 P0 / 7 P1; Proptest: 64 P0 / 21 P1; Unit test: 4 P0 / 0 P1;
     Fuzz: 5 P0 / 1 P1; Integration: 17 P0 / 2 P1; Total: 113 P0 / 31 P1 / 144 VPs.
     Unit test VPs = VP-095..VP-098 (BC-3.3.001 bounded DTU type enumeration).
     VP-137 + VP-138 added Wave 4 Phase 1 ADR burst (2026-05-02): proptest P1 13→15.
     VP-139..VP-142 added Wave 4 Phase 2 ADR burst (2026-05-02): proptest P1 15→19.
     VP-143 + VP-144 added Wave 4 Phase 3 ADR burst (2026-05-02): proptest P1 19→21.
     VP-138 elevated P1→P0 (W4-Phase4A-Pass4 2026-05-03): proptest P0 64→65, P1 21→20; Total P0 113→114, P1 31→30. -->


## Coverage Gaps and Mitigations

| Gap | Reason | Mitigation |
|-----|--------|-----------|
| prism-sensors: proptest coverage added (Wave 3) | Effectful shell (HTTP I/O); formal proof still not applicable | VP-077..VP-080, VP-087..VP-089, VP-091..VP-093 (proptest); VP-090, VP-094 (integration_test); remaining gap: stateful HTTP adapter paths |
| prism-credentials: proptest coverage added (Wave 3) | OS keyring I/O + encryption | VP-081, VP-082, VP-084..VP-086 (proptest); VP-083 (integration_test); remaining gap: OS keyring live I/O |
| prism-storage: no formal verification | RocksDB I/O | StorageBackend trait tested via InMemoryBackend; RocksDB integration tests |
| prism-mcp: no formal verification | MCP transport I/O | End-to-end integration tests with mock MCP client |
| Detection correlation/sequence state: complex state machines | State transitions with time windows | Proptest with time-travel-capable test harness |

## Invariant-to-VP Traceability

| Invariant | Verified By | Status |
|-----------|------------|--------|
| DI-001 (Cursor validity) | VP-029 | P1 |
| DI-002 (Credential isolation) | integration tests | P0 |
| DI-003 (Deny-by-default) | VP-002, VP-003, VP-004, VP-020 | P0 |
| DI-005 (OCSF validity) | VP-016, VP-017, VP-022 | P0 |
| DI-006 (Prompt injection) | VP-024 | P0 |
| DI-007 (Token expiry) | VP-007, VP-008, VP-009 | P0 |
| DI-008 (Org separation) | integration tests | P0 |
| DI-014 (Credential name) | VP-011 | P0 |
| DI-015 (Token cap) | VP-010 | P0 |
| DI-019 (Query limits) | VP-014, VP-015, VP-021 | P0 |
| DI-020 (Alias depth/cycles) | VP-012, VP-013 | P0 |
| DI-021 (Required columns) | VP-031 | P0 |
| DI-022 (Splay distribution) | VP-026 | P1 |
| DI-023 (Diff exactly-once) | VP-019 | P0 |
| DI-024 (Rule validation) | VP-018 | P0 |
| DI-025 (Case transitions) | VP-005, VP-006, VP-051 | P0 |
| DI-012 (Sealed auth trait) | Compile-time enforcement by type system | P0 (no runtime VP needed) |
| DI-017 (Single-process LOCK) | Integration test: verify RocksDB LOCK prevents concurrent open | P1 |
| DI-026 (Audit buffer durability) | VP-033 (module: prism-dtu-crowdstrike) | P0 |
| DI-027 (Watchdog) | VP-058 (proptest, watchdog memory grace period) + Integration tests | P0 |
| DI-028 (Schedule/rule caps) | VP-030 | P1 |
| DI-029 (Correlation window >= interval) | Config validation integration test (warning path) | P1 |
| DI-030 (Spec validation) | VP-023 | P0 |
| DI-031 (Reload atomicity) | VP-032 | P1 |
| DI-032 (Concurrent schedule cap) | Integration test: verify semaphore enforcement | P0 |
| DI-033 (OrgRegistry Bijectivity) | VP-063, VP-064, VP-065 (BC-3.1.001 depends-on), VP-069, VP-070, VP-071, VP-072, VP-073, VP-074, VP-075, VP-076 | P0 |

### BC-level Invariant Properties Cited by VPs

<!-- BC-level invariants defined within BCs (not domain-spec/invariants.md DI-NNN) are listed here, separate from the DI-NNN table above. -->

<!-- Phase 1-2 BC-anchored VPs tracked in this table (24 entries) PLUS one Wave 3 exception (BC-3.1.001/VP-001 — VP-001 is a legacy numbered VP from Phase 1, not a Wave 3 VP-063+; included here for back-compat). Other Wave 3 BC-anchored VPs (VP-063..VP-136) are tracked separately in VP-INDEX via [BC-3.X.Y] reference tags. -->

| BC | BC-level Invariant | Verified By | Priority |
|----|--------------------|-------------|----------|
| BC-3.1.001 (OrgRegistry resolution) | OrgSlug rejects invalid characters (precondition for resolution correctness) | VP-001 (module: prism-core, kani) | P0 |
| BC-2.05.011 (Audit forward watermark monotonicity) | INV-AUDIT-FWD-001 | VP-039 (module: prism-audit) | P0 |
| BC-2.13.013 (Alert deduplication key correctness) | Alert dedup key correct per match mode | VP-027 (module: prism-operations, proptest) | P0 |
| BC-2.13.005 (Template interpolation safety) | Template interpolation never panics; handles missing vars | VP-028 (module: prism-operations, fuzz) | P0 |
| BC-2.17.002 (Plugin WASI namespace exclusion) | Plugin linker excludes all WASI namespace imports | VP-040 (module: prism-spec-engine, kani) | P1 |
| BC-2.17.003 (Plugin memory limit boundary) | At-limit succeeds, over-limit traps | VP-041 (module: prism-spec-engine, proptest) | P1 |
| BC-2.17.005 (Plugin hot reload retention) | Failed compile retains old InstancePre | VP-042 (module: prism-spec-engine, proptest) | P1 |
| BC-2.17.006 (WIT required exports validation) | WIT validation rejects component missing required exports | VP-043 (module: prism-spec-engine, proptest) | P1 |
| BC-2.18.001 (Action retry state machine) | Bounded by 5 attempts, dead-letter terminal | VP-044 (module: prism-operations, kani) | P0 |
| BC-2.18.004 (Schedule semaphore non-blocking) | try_acquire used (non-blocking), never acquire | VP-045 (module: prism-operations, proptest) | P0 |
| BC-2.18.007 (Action inline credential rejection) | Inline credential rejected at load time; not in error message | VP-046 (module: prism-operations, proptest) | P0 |
| BC-2.18.009 (UUID v7 validation) | Non-v7 always rejected, v7 always accepted, order preserved | VP-047 (module: prism-operations, proptest) | P0 |
| BC-2.19.001 (Infusion spec field mapping) | N fields produces exactly N UDF descriptors; duplicates error | VP-048 (module: prism-spec-engine, kani) | P1 |
| BC-2.19.002 (Infusion per-query dedup) | Source calls = unique value count | VP-049 (module: prism-spec-engine, proptest) | P1 |
| BC-2.10.008 (MCP sensor resource credential redaction) | Response redacts credentials and full API URLs | VP-050 (module: prism-mcp, proptest) | P0 |
| BC-2.14.002 (Case state machine exhaustive transitions) | 5×5 transition table: exactly 12 Ok, 13 Err; self-transitions always Err(E-CASE-005) | VP-051 (module: prism-core, kani) | P0 |
| BC-2.14.003 (Case update disposition ordering) | Disposition applied before status transition in single-call update | VP-052 (module: prism-operations, proptest) | P0 |
| BC-2.14.006 (Resolved case disposition required) | Resolved case always has non-null disposition; transition rejects without | VP-053 (module: prism-core, kani) | P0 |
| BC-2.14.008 (TTR first-resolution timestamp) | TTR uses first resolution timestamp across reopen cycles; null aggregate when none | VP-054 (module: prism-operations, proptest) | P1 |
| BC-2.15.002 (StorageEngine put_batch atomicity) | put_batch atomicity and domain isolation (MockStorageEngine) | VP-055 (module: prism-storage, proptest) | P1 |
| BC-2.15.004 (Audit buffer overflow purge) | Oldest entries deleted, newest preserved, purge-event produced | VP-056 (module: prism-audit, proptest) | P1 |
| BC-2.15.005 (Crash recovery denylist threshold) | Denylist triggered at consecutive_crashes >= 3; exact threshold | VP-057 (module: prism-storage, kani) | P0 |
| BC-2.14.013 (Dedup link-or-create decision) | Link(c.id) iff existing case within window; Create otherwise | VP-060 (module: prism-operations, proptest) | P0 |
| BC-2.20.002 (Log forwarder min-level filter) | Per-destination enqueue/discard matches level-rank ordering for all 5×5 level pairs | VP-061 (module: prism-mcp, proptest) | P1 |
| BC-2.20.003 (Log forwarder queue cap) | queue.len() never exceeds 10 × batch_size; drop_count +1 per overflow enqueue | VP-062 (module: prism-mcp, proptest) | P1 |

## Changelog

| Version | Author | Date | Description |
|---------|--------|------|-------------|
| 1.26 | state-manager | 2026-05-02 | P5 architecture aggregate sync: Totals row Proptest 85→86, Total VPs 144→145; method-totals table Proptest 85→86, Total VPs 144→145 (VP-145 INV-CASE-006 proptest). |
| 1.25 | state-manager | 2026-05-02 | W4-Phase4A-Pass4: VP-138 elevated P1→P0 (INV-CASE-003 cross-org case isolation safety-critical). Proptest P0 64→65, P1 21→20. Total P0 113→114, P1 31→30. |
| 1.24 | state-manager | 2026-05-02 | W4-ADR-Phase3-burst: VP-143 added to prism-operations proptest list. VP-144 added as new prism-siem-formats row (new crate per ADR-019). Total VPs 142→144 (Proptest 83→85). P1 enumeration 29→31. |
| 1.23 | state-manager | 2026-05-02 | W4-ADR-Phase2-burst: VP-139..142 added to prism-operations proptest list. Total VPs 138→142 (Proptest 79→83). P1 enumeration 25→29. |
| 1.22 | product-owner | 2026-04-27 | m-31-003 (Pass 31): BC-level comment updated — clarifies BC-3.1.001/VP-001 is a Wave 3 exception (legacy Phase 1 VP included for back-compat); other Wave 3 VPs (VP-063..VP-136) remain in VP-INDEX only. |
| 1.21 | product-owner | 2026-04-27 | m-30-002 (pass-30-remediation): VP-001 re-anchored from DI-033 to BC-3.1.001. DI-033 row removes VP-001 (VP-001 tests slug character validity, not bijection). BC-3.1.001 row added to BC-level table: VP-001 verifies OrgSlug character rejection precondition for resolution. |
| 1.20 | product-owner | 2026-04-27 | m-17-002 (pass-17-remediation): DI-033 row updated — added VP-063, VP-064, VP-065 to coverage set per option (a) adoption: BC-3.1.001 depends-on DI-033 (resolution correctness assumes bijectivity), so its VPs (VP-063/064/065) are included as touching this invariant. Full DI-033 VP set: VP-001, VP-063, VP-064, VP-065, VP-069, VP-070, VP-071, VP-072, VP-073, VP-074, VP-075, VP-076. |
| 1.19 | product-owner | 2026-04-27 | m-15-002 follow-on: VP-001 re-anchored from DI-002/DI-008 rows to DI-033 row (OrgSlug char validation belongs to OrgRegistry Bijectivity invariant, not credential isolation or org separation). DI-002 and DI-008 rows now cite integration tests only. DI-033 row includes VP-001 alongside VP-069..VP-076. |
| 1.18 | product-owner | 2026-04-27 | Pass 15 sweep: DI-033 (OrgRegistry Bijectivity) row added to Invariant-to-VP Traceability table, verified by VP-069..VP-076 (P0). |
| 1.17 | product-owner | 2026-04-27 | M-14-002 (pass-14-remediation): VP-001 description updated "TenantId" → "OrgSlug" in Invariant-to-VP Traceability table (DI-002 and DI-008 rows) to reflect Wave-3 OrgSlug rename. |
| 1.16 | product-owner | 2026-04-27 | M-003 (pass-8-remediation): Added "Unit Tests" column to Coverage by Module table (between Proptest and Fuzz). prism-spec-engine Unit Tests = 4 (VP-095..VP-098); all other modules = 0. Totals row updated: Unit Tests = 4. The "(+4 unit_test)" parenthetical footnote removed from Totals row. Column header and per-row counts updated throughout. |
| 1.15 | product-owner | 2026-04-27 | M-001 (pass-6-remediation): VP-135 module mis-attribution corrected. Moved VP-135 from prism-dtu-harness proptest (5→4) to prism-bin proptest (0→1). prism-dtu-harness row total 13→12; prism-bin row total 2→3. Column totals unchanged (Proptest=77, Total=136). |
| 1.14 | product-owner | 2026-04-27 | M-001 (pass-5-remediation): Filled Kani P0/P1 (23/7), Integration P0/P1 (17/2), and Total P0/P1 (113/23) from VP-INDEX v1.14 recount values stored in HTML comment (lines 52-55). Replaced all four TODO cells with confirmed values. |
| 1.13 | product-owner | 2026-04-27 | m-003 (pass-4-remediation): HTML comment updated — scope clarified to Phase 1-2 BC-anchored VPs (24 entries); Wave 3 VPs tracked in VP-INDEX reference tags. Coverage Gaps updated — prism-sensors and prism-credentials no longer have zero formal verification (Wave 3 added proptest VPs). Totals sub-table Proptest P0/P1 updated 77/TODO/TODO → 77/64/13 from VP-INDEX v1.14 recount. |
| 1.12 | product-owner | 2026-04-27 | M-002: VP-083 removed from prism-sensors row (VP-083 belongs to prism-credentials per VP-INDEX). prism-sensors integration count 3→2. Per-module sum and Totals row both remain 136 (VP-083 still counted under prism-credentials). |
| 1.11 | product-owner | 2026-04-27 | C-002: Totals sub-table updated from stale 62-VP baseline to 136 VP totals matching VP-INDEX v1.12: Kani=30, Proptest=81, Fuzz=6, Integration=19, Total=136. P0/P1 per-method split marked TODO pending per-VP enumeration sweep (Wave 3 VPs VP-063..VP-136 not individually enumerated in prior P0/P1 tallies). |
| 1.10 | architect | 2026-04-21 | F90-004: VP-052 and VP-054 moved from prism-core to prism-operations in Coverage by Module table and BC-level Invariant Properties table. prism-core proptest 2→0; prism-operations proptest 7→9. Totals unchanged (62 VPs). |
| 1.9 | architect | 2026-04-21 | F87-004: prism-persistence → prism-storage in Coverage by Module table row and BC-level Invariant Properties table VP-055/VP-057 module annotations. |
| 1.8 | architect | 2026-04-21 | pass-86 F86-005: DI-025 row updated to include VP-051 (VP-005, VP-006, VP-051). BC-level table: added BC-2.14.002 row for VP-051. BC-anchored VP count 23 → 24. |
| 1.7 | architect | 2026-04-21 | pass-84 F84-002: DI-027 row updated to include VP-058 (proptest, watchdog memory grace period). BC-level Invariant Properties table expanded to cover all 23 BC-anchored VPs — added VP-027, VP-028, VP-040 through VP-050, VP-052 through VP-057, VP-060, VP-061, VP-062 with their BC anchors. |
| 1.6 | architect | 2026-04-21 | F81-009: added VP-061 and VP-062 (proptest) to prism-mcp. Proptest 26→28; Total VPs 60→62; P1 17→19. |
| 1.5 | architect | 2026-04-20 | Added VP-060 (dedup-decision-link-or-create) to prism-operations Proptest column. Total VPs 59→60; P0 42→43; Proptest 25→26. Closes BC-2.14.013 DEFER. |
| 1.4 | architect | 2026-04-20 | Pass-74 CRIT-002 remediation: fixed stale Totals section (was showing 50-VP baseline). Updated to Kani=26, Proptest=25, Fuzz=6, Integration=2, Total=59, P0=42, P1=17. Verified per-module column sums equal 26+25+6+2=59. |
| 1.3 | architect | 2026-04-20 | Pass-74 CRIT-002: added 9 VPs (VP-051 through VP-059) to per-module Coverage table. prism-core +4 (VP-051 kani, VP-052 proptest, VP-053 kani, VP-054 proptest); prism-persistence +3 (VP-055 proptest, VP-057 kani, VP-058 proptest); prism-audit +1 (VP-056 proptest); prism-spec-engine +1 (VP-059 proptest). Grand total 50→59. |
| 1.2 | architect | 2026-04-20 | Burst 2A: updated Totals table to reflect 11 new VPs (VP-040 through VP-050). Kani 20→23 (+VP-040, VP-044, VP-048); Proptest 11→19 (+VP-041, VP-042, VP-043, VP-045, VP-046, VP-047, VP-049, VP-050); grand total 39→50. P0 32→37, P1 7→13. Per-module Coverage table already correct from prior burst. |
| 1.1 | architect | 2026-04-20 | Fixed LOW-002: added Integration Tests column so per-module VP counts sum to 39 (previously summed to 37, missing the 2 DTU integration VPs). prism-dtu-crowdstrike Integration=2 (VP-033 + VP-036). Totals row added to coverage table. |
| 1.0 | architect | 2026-04-15 | Initial version |
