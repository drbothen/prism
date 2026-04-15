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
| prism-core | CRITICAL | 6 | 2 | 0 | 95% | VP-001, VP-002, VP-003, VP-004, VP-005, VP-006, VP-011, VP-029 |
| prism-security | CRITICAL | 4 | 1 | 1 | 90% | VP-007, VP-008, VP-009, VP-010, VP-020, VP-024 |
| prism-query | CRITICAL | 3 | 2 | 1 | 90% | VP-012, VP-014, VP-015, VP-021, VP-025, VP-031, VP-013 |
| prism-ocsf | CRITICAL | 0 | 2 | 1 | 90% | VP-016, VP-017, VP-022 |
| prism-operations | HIGH | 2 | 3 | 1 | 85% | VP-018, VP-019, VP-026, VP-027, VP-028, VP-030 |
| prism-spec-engine | HIGH | 0 | 1 | 1 | 85% | VP-023, VP-032 |
| prism-sensors | HIGH | 0 | 0 | 0 | 75% | (integration tests only) |
| prism-credentials | CRITICAL | 0 | 0 | 0 | 80% | (integration tests — I/O-bound) |
| prism-storage | HIGH | 0 | 0 | 0 | 80% | (integration tests — I/O-bound) |
| prism-audit | HIGH | 0 | 0 | 0 | 75% | (integration tests — I/O-bound) |
| prism-mcp | HIGH | 0 | 0 | 0 | 75% | (integration tests — I/O-bound) |
| prism-bin | LOW | 0 | 0 | 0 | 60% | (smoke tests) |

## Totals

| Method | Planned Count | P0 | P1 |
|--------|--------------|----|----|
| Kani proofs | 15 | 13 | 2 |
| Proptest properties | 11 | 9 | 2 |
| Fuzz targets | 5 | 5 | 0 |
| **Total VPs** | **32** | **27** | **5** |

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
| DI-027 (Watchdog) | Integration tests | P0 |
| DI-028 (Schedule/rule caps) | VP-030 | P1 |
| DI-030 (Spec validation) | VP-023 | P0 |
| DI-031 (Reload atomicity) | VP-032 | P1 |
