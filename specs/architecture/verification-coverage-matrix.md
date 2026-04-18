---
document_type: architecture-section
level: L3
section: "verification-coverage-matrix"
version: "1.0"
status: draft
producer: architect
timestamp: 2026-04-15T12:00:00
phase: 1b
inputs: [prd.md, domain-spec/invariants.md]
traces_to: ARCH-INDEX.md
---

# Verification Coverage Matrix

## VP-to-Module Coverage

| Module | Criticality | Kani Proofs | Proptest | Fuzz Targets | Coverage Target | VPs |
|--------|------------|-------------|----------|-------------|----------------|-----|
| prism-core | CRITICAL | 8 | 0 | 0 | 95% | VP-001, VP-002, VP-003, VP-004, VP-005, VP-006, VP-011, VP-029 |
| prism-security | CRITICAL | 5 | 1 | 1 | 90% | VP-007, VP-008, VP-009, VP-010, VP-020 (Kani); VP-024 (proptest); VP-038 (fuzz — injection scanner) |
| prism-query | CRITICAL | 4 | 2 | 2 | 90% | VP-012, VP-014, VP-015, VP-025 (Kani); VP-013, VP-031 (proptest); VP-021 (fuzz), VP-037 (fuzz — alias expansion) |
| prism-ocsf | CRITICAL | 0 | 2 | 1 | 90% | VP-016, VP-017, VP-022 |
| prism-operations | HIGH | 2 | 3 | 1 | 85% | VP-018, VP-019, VP-026, VP-027, VP-028, VP-030 |
| prism-spec-engine | HIGH | 0 | 1 | 1 | 85% | VP-023, VP-032 |
| prism-sensors | HIGH | 0 | 0 | 0 | 75% | (integration tests only) |
| prism-credentials | CRITICAL | 0 | 2 | 0 | 90% | VP-034 (encryption round-trip), VP-035 (key derivation). Integration tests per platform for I/O. Coverage raised to 90% to match CRITICAL classification (SOC 2 compliance). |
| prism-storage | HIGH | 0 | 0 | 0 | 80% | (integration tests — I/O-bound) |
| prism-audit | HIGH | 1 | 0 | 0 | 75% | VP-039 (Kani — audit forward watermark monotonicity). Other coverage via integration tests (I/O-bound). |
| prism-dtu-crowdstrike | HIGH | 0 | 0 | 0 | 75% | VP-033 (integration test — audit buffer RocksDB-write-before-delivery ordering), VP-036 (integration test — SessionContext drop on error/panic). |
| prism-mcp | HIGH | 0 | 0 | 0 | 75% | (integration tests — I/O-bound) |
| prism-bin | LOW | 0 | 0 | 0 | 60% | (smoke tests) |

## Totals

| Method | Planned Count | P0 | P1 |
|--------|--------------|----|----|
| Kani proofs | 20 | 16 | 4 |
| Proptest properties | 11 | 9 | 2 |
| Integration test VPs | 2 | 2 | 0 |
| Fuzz targets | 6 | 5 | 1 |
| **Total VPs** | **39** | **32** | **7** |

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
| DI-025 (Case transitions) | VP-005, VP-006 | P0 |
| DI-012 (Sealed auth trait) | Compile-time enforcement by type system | P0 (no runtime VP needed) |
| DI-017 (Single-process LOCK) | Integration test: verify RocksDB LOCK prevents concurrent open | P1 |
| DI-026 (Audit buffer durability) | VP-033 (module: prism-dtu-crowdstrike) | P0 |
| DI-027 (Watchdog) | Integration tests | P0 |
| DI-028 (Schedule/rule caps) | VP-030 | P1 |
| DI-029 (Correlation window >= interval) | Config validation integration test (warning path) | P1 |
| DI-030 (Spec validation) | VP-023 | P0 |
| DI-031 (Reload atomicity) | VP-032 | P1 |
| DI-032 (Concurrent schedule cap) | Integration test: verify semaphore enforcement | P0 |

### BC-level Invariant Properties Cited by VPs

<!-- BC-level invariants defined within BCs (not domain-spec/invariants.md DI-NNN) are listed here, separate from the DI-NNN table above. -->

| BC | BC-level Invariant | Verified By | Priority |
|----|--------------------|-------------|----------|
| BC-2.05.011 (Audit forward watermark monotonicity) | INV-AUDIT-FWD-001 | VP-039 (module: prism-audit) | P0 |
