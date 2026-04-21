---
document_type: architecture-section
level: L3
section: "verification-coverage-matrix"
version: "1.8"
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

| Module | Criticality | Kani Proofs | Proptest | Fuzz Targets | Integration Tests | Coverage Target | VPs |
|--------|------------|-------------|----------|-------------|-------------------|----------------|-----|
| prism-core | CRITICAL | 10 | 2 | 0 | 0 | 95% | VP-001, VP-002, VP-003, VP-004, VP-005, VP-006, VP-011, VP-029, VP-051, VP-053 (Kani); VP-052, VP-054 (proptest) |
| prism-security | CRITICAL | 5 | 1 | 1 | 0 | 90% | VP-007, VP-008, VP-009, VP-010, VP-020 (Kani); VP-024 (proptest); VP-038 (fuzz — injection scanner) |
| prism-query | CRITICAL | 4 | 2 | 2 | 0 | 90% | VP-012, VP-014, VP-015, VP-025 (Kani); VP-013, VP-031 (proptest); VP-021 (fuzz), VP-037 (fuzz — alias expansion) |
| prism-ocsf | CRITICAL | 0 | 2 | 1 | 0 | 90% | VP-016, VP-017, VP-022 |
| prism-operations | HIGH | 3 | 7 | 1 | 0 | 85% | VP-026, VP-030, VP-044 (Kani); VP-018, VP-019, VP-027, VP-045, VP-046, VP-047, VP-060 (proptest); VP-028 (fuzz) |
| prism-spec-engine | HIGH | 2 | 6 | 1 | 0 | 85% | VP-023 (fuzz); VP-032, VP-041, VP-042, VP-043, VP-049, VP-059 (proptest); VP-040, VP-048 (kani) |
| prism-sensors | HIGH | 0 | 0 | 0 | 0 | 75% | (integration tests only — no formal VP) |
| prism-credentials | CRITICAL | 0 | 2 | 0 | 0 | 90% | VP-034 (encryption round-trip), VP-035 (key derivation). Integration tests per platform for I/O. Coverage raised to 90% to match CRITICAL classification (SOC 2 compliance). |
| prism-persistence | HIGH | 1 | 2 | 0 | 0 | 80% | VP-057 (Kani — crash recovery denylist threshold); VP-055 (proptest — batch atomicity and domain isolation), VP-058 (proptest — watchdog memory two-check policy) |
| prism-audit | HIGH | 1 | 1 | 0 | 0 | 75% | VP-039 (Kani — audit forward watermark monotonicity); VP-056 (proptest — audit buffer overflow purge preserves newest) |
| prism-dtu-crowdstrike | HIGH | 0 | 0 | 0 | 2 | 75% | VP-033 (integration test — audit buffer RocksDB-write-before-delivery ordering), VP-036 (integration test — SessionContext drop on error/panic). |
| prism-mcp | HIGH | 0 | 3 | 0 | 0 | 75% | VP-050 (proptest — sensor resource redacts credentials); VP-061 (proptest — log forwarder min-level filter determinism); VP-062 (proptest — log forwarder queue cap bounded at 10×batch_size) |
| prism-bin | LOW | 0 | 0 | 0 | 0 | 60% | (smoke tests) |
| **Totals** | | **26** | **28** | **6** | **2** | | **62** |

## Totals

| Method | Planned Count | P0 | P1 |
|--------|--------------|----|----|
| Kani proofs | 26 | 20 | 6 |
| Proptest properties | 28 | 16 | 12 |
| Fuzz targets | 6 | 5 | 1 |
| Integration test VPs | 2 | 2 | 0 |
| **Total VPs** | **62** | **43** | **19** |

## Coverage Gaps and Mitigations

| Gap | Reason | Mitigation |
|-----|--------|-----------|
| prism-sensors: no formal verification | Effectful shell (HTTP I/O) | Integration tests with mock HTTP server, contract tests against sensor API recordings |
| prism-credentials: no formal verification | OS keyring I/O + encryption | Integration tests per platform, encryption round-trip tests |
| prism-storage: no formal verification | RocksDB I/O | StorageBackend trait tested via InMemoryBackend; RocksDB integration tests |
| prism-mcp: no formal verification | MCP transport I/O | End-to-end integration tests with mock MCP client |
| Detection correlation/sequence state: complex state machines | State transitions with time windows | Proptest with time-travel-capable test harness |

## Invariant-to-VP Traceability

| Invariant | Verified By | Status |
|-----------|------------|--------|
| DI-001 (Cursor validity) | VP-029 | P1 |
| DI-002 (Credential isolation) | VP-001 (TenantId) + integration tests | P0 |
| DI-003 (Deny-by-default) | VP-002, VP-003, VP-004, VP-020 | P0 |
| DI-005 (OCSF validity) | VP-016, VP-017, VP-022 | P0 |
| DI-006 (Prompt injection) | VP-024 | P0 |
| DI-007 (Token expiry) | VP-007, VP-008, VP-009 | P0 |
| DI-008 (Client separation) | VP-001 + integration tests | P0 |
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

### BC-level Invariant Properties Cited by VPs

<!-- BC-level invariants defined within BCs (not domain-spec/invariants.md DI-NNN) are listed here, separate from the DI-NNN table above. -->

<!-- 24 BC-anchored VPs tracked in this table. -->

| BC | BC-level Invariant | Verified By | Priority |
|----|--------------------|-------------|----------|
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
| BC-2.14.003 (Case update disposition ordering) | Disposition applied before status transition in single-call update | VP-052 (module: prism-core, proptest) | P0 |
| BC-2.14.006 (Resolved case disposition required) | Resolved case always has non-null disposition; transition rejects without | VP-053 (module: prism-core, kani) | P0 |
| BC-2.14.008 (TTR first-resolution timestamp) | TTR uses first resolution timestamp across reopen cycles; null aggregate when none | VP-054 (module: prism-core, proptest) | P1 |
| BC-2.15.002 (StorageEngine put_batch atomicity) | put_batch atomicity and domain isolation (MockStorageEngine) | VP-055 (module: prism-persistence, proptest) | P1 |
| BC-2.15.004 (Audit buffer overflow purge) | Oldest entries deleted, newest preserved, purge-event produced | VP-056 (module: prism-audit, proptest) | P1 |
| BC-2.15.005 (Crash recovery denylist threshold) | Denylist triggered at consecutive_crashes >= 3; exact threshold | VP-057 (module: prism-persistence, kani) | P0 |
| BC-2.14.013 (Dedup link-or-create decision) | Link(c.id) iff existing case within window; Create otherwise | VP-060 (module: prism-operations, proptest) | P0 |
| BC-2.20.002 (Log forwarder min-level filter) | Per-destination enqueue/discard matches level-rank ordering for all 5×5 level pairs | VP-061 (module: prism-mcp, proptest) | P1 |
| BC-2.20.003 (Log forwarder queue cap) | queue.len() never exceeds 10 × batch_size; drop_count +1 per overflow enqueue | VP-062 (module: prism-mcp, proptest) | P1 |

## Changelog

| Version | Author | Date | Description |
|---------|--------|------|-------------|
| 1.8 | architect | 2026-04-21 | pass-86 F86-005: DI-025 row updated to include VP-051 (VP-005, VP-006, VP-051). BC-level table: added BC-2.14.002 row for VP-051. BC-anchored VP count 23 → 24. |
| 1.7 | architect | 2026-04-21 | pass-84 F84-002: DI-027 row updated to include VP-058 (proptest, watchdog memory grace period). BC-level Invariant Properties table expanded to cover all 23 BC-anchored VPs — added VP-027, VP-028, VP-040 through VP-050, VP-052 through VP-057, VP-060, VP-061, VP-062 with their BC anchors. |
| 1.6 | architect | 2026-04-21 | F81-009: added VP-061 and VP-062 (proptest) to prism-mcp. Proptest 26→28; Total VPs 60→62; P1 17→19. |
| 1.5 | architect | 2026-04-20 | Added VP-060 (dedup-decision-link-or-create) to prism-operations Proptest column. Total VPs 59→60; P0 42→43; Proptest 25→26. Closes BC-2.14.013 DEFER. |
| 1.4 | architect | 2026-04-20 | Pass-74 CRIT-002 remediation: fixed stale Totals section (was showing 50-VP baseline). Updated to Kani=26, Proptest=25, Fuzz=6, Integration=2, Total=59, P0=42, P1=17. Verified per-module column sums equal 26+25+6+2=59. |
| 1.3 | architect | 2026-04-20 | Pass-74 CRIT-002: added 9 VPs (VP-051 through VP-059) to per-module Coverage table. prism-core +4 (VP-051 kani, VP-052 proptest, VP-053 kani, VP-054 proptest); prism-persistence +3 (VP-055 proptest, VP-057 kani, VP-058 proptest); prism-audit +1 (VP-056 proptest); prism-spec-engine +1 (VP-059 proptest). Grand total 50→59. |
| 1.2 | architect | 2026-04-20 | Burst 2A: updated Totals table to reflect 11 new VPs (VP-040 through VP-050). Kani 20→23 (+VP-040, VP-044, VP-048); Proptest 11→19 (+VP-041, VP-042, VP-043, VP-045, VP-046, VP-047, VP-049, VP-050); grand total 39→50. P0 32→37, P1 7→13. Per-module Coverage table already correct from prior burst. |
| 1.1 | architect | 2026-04-20 | Fixed LOW-002: added Integration Tests column so per-module VP counts sum to 39 (previously summed to 37, missing the 2 DTU integration VPs). prism-dtu-crowdstrike Integration=2 (VP-033 + VP-036). Totals row added to coverage table. |
| 1.0 | architect | 2026-04-15 | Initial version |
