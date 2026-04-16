---
document_type: story-index
level: L4
version: "1.0"
status: draft
producer: story-writer
timestamp: 2026-04-16T12:00:00
phase: 3
total_stories: 39
total_bcs_covered: 153
total_vps_assigned: 38
---

# Prism Phase 3 Story Index

## Overview

Phase 3 decomposes the Prism platform into 39 implementation stories spanning 6 parallel
waves. Stories are organized by crate and ordered topologically so that no story begins
before its dependencies are complete.

- **Total stories:** 39
- **Total waves:** 6
- **BCs covered:** 153 (across SS-02 through SS-16)
- **VPs assigned:** 38 (Kani proofs, proptests, fuzz targets)

Every story contains: narrative, behavioral contracts table, numbered tasks, acceptance
criteria (Given/When/Then), verification properties, and notes. No story exceeds 13
story points. No story's estimated context exceeds 30% of the implementing agent's
context window.

---

## Wave Summary

| Wave | Crates | Stories | BCs | Theme |
|------|--------|---------|-----|-------|
| 1 | prism-core, prism-ocsf, prism-credentials, prism-security, prism-spec-engine | 12 | 60 + 3 infra | Foundation + Pure Domain |
| 2 | prism-storage, prism-audit, prism-sensors | 7 | 42 | Infrastructure + Adapters |
| 3 | prism-query | 5 | 21 | Query Engine |
| 4 | prism-operations | 7 | 33 | Operations |
| 5 | prism-mcp (+ SS-06 config) | 5 | 26 | MCP Server + Config |
| 6 | prism-bin | 3 | 0 (infra) | Binary + E2E |

Wave 1 stories have no dependencies outside the wave (except S-1.01 which is the root).
Wave 2 stories depend on Wave 1. Wave 3 depends on Wave 2. Waves 4-6 follow in order.
All dependency chains are acyclic (validated by topological sort below).

---

## Full Story List

| Story ID | Title | Crate | BCs | VPs | Days | Depends On |
|----------|-------|-------|-----|-----|------|------------|
| S-1.01 | Foundational Types (TenantId, PrismError, StorageDomain) | prism-core | 0 | VP-001 | 2 | -- |
| S-1.02 | Entity Types and State Machines | prism-core | 0 | VP-005,006,011,029 | 2 | S-1.01 |
| S-1.03 | Capability Resolution Engine | prism-core | 0 | VP-002,003,004 | 2 | S-1.01 |
| S-1.04 | OCSF Schema Loading and DynamicMessage | prism-ocsf | 5 | VP-016,022 | 3 | S-1.01 |
| S-1.05 | OCSF Field Mapping and Normalization | prism-ocsf | 7 | VP-017 | 3 | S-1.04 |
| S-1.06 | Credential Store Trait and Backends | prism-credentials | 7 | VP-034,035 | 3 | S-1.01,S-1.02 |
| S-1.07 | Credential CRUD, Resolution, and Security | prism-credentials | 5 | -- | 2 | S-1.06 |
| S-1.08 | Feature Flags (P0 Core) | prism-security | 8 | VP-020 | 3 | S-1.01,S-1.03 |
| S-1.09 | Confirmation Tokens (P1) | prism-security | 6 | VP-007,008,009,010 | 2 | S-1.08 |
| S-1.10 | Prompt Injection Defense | prism-security | 8 | VP-024 | 2 | S-1.01 |
| S-1.11 | Spec Loading and Pipeline Execution | prism-spec-engine | 5 | VP-023 | 3 | S-1.01 |
| S-1.12 | Hot Reload and Runtime Management | prism-spec-engine | 5 | VP-032 | 2 | S-1.11 |
| S-2.01 | RocksDB Initialization and Domain Operations | prism-storage | 3 | -- | 3 | S-1.01 |
| S-2.02 | Audit Buffer and Watchdog | prism-storage | 5 | -- | 2 | S-2.01 |
| S-2.03 | Decorators and Internal Tables | prism-storage | 3 | -- | 2 | S-2.01 |
| S-2.04 | Audit Entry Construction and Compliance | prism-audit | 6 | VP-033 | 3 | S-2.01,S-2.02 |
| S-2.05 | Specialized Audit Events | prism-audit | 4 | -- | 1 | S-2.04 |
| S-2.06 | DataSource Trait and Auth Patterns | prism-sensors | 4 | -- | 3 | S-1.06,S-1.11 |
| S-2.07 | Per-Sensor Auth and Pagination | prism-sensors | 5 | -- | 3 | S-2.06 |
| S-3.01 | PrismQL Parser (Filter + SQL + Pipe) | prism-query | 3 | VP-014,015,021 | 3 | S-1.01 |
| S-3.02 | Query Tool and Materialization | prism-query | 6 | VP-031 | 3 | S-3.01,S-2.06,S-1.04 |
| S-3.03 | Explain and Query Diagnostics | prism-query | 1 | -- | 1 | S-3.02 |
| S-3.04 | Alias System (P1) | prism-query | 5 | VP-012,013,025,037 | 2 | S-3.02 |
| S-3.05 | Pagination and Caching | prism-query | 6 | -- | 2 | S-3.02 |
| S-4.01 | Schedule CRUD and Execution Loop | prism-operations | 5 | VP-026,030 | 3 | S-3.02,S-2.01 |
| S-4.02 | Differential Results and Packs | prism-operations | 5 | VP-019 | 2 | S-4.01 |
| S-4.03 | Detection Rule Loading and Compilation | prism-operations | 7 | VP-018 | 3 | S-3.02,S-1.08 |
| S-4.04 | Detection Evaluation (Single/Correlation/Sequence) | prism-operations | 5 | VP-027,036 | 3 | S-4.03 |
| S-4.05 | Alert Generation | prism-operations | 1 | VP-028 | 1 | S-4.04 |
| S-4.06 | Case Management | prism-operations | 8 | -- | 3 | S-4.05,S-2.01 |
| S-4.07 | Case Metrics and Acknowledge Alert | prism-operations | 3 | -- | 2 | S-4.06 |
| S-5.01 | Server Bootstrap and Tool Registration | prism-mcp | 5 | -- | 3 | S-1.08,S-3.02,S-4.01 |
| S-5.02 | Tool Routing, Errors, and Client Scoping | prism-mcp | 3 | -- | 2 | S-5.01 |
| S-5.03 | Resources and Prompts | prism-mcp | 4 | -- | 2 | S-5.02 |
| S-5.04 | Sensor Health Subsystem | prism-mcp | 5 | -- | 2 | S-5.03,S-2.07 |
| S-5.05 | Config Loading and Validation | prism-mcp | 9 | -- | 3 | S-5.01,S-1.06 |
| S-6.01 | CLI, Startup, and Initialization | prism-bin | 0 | -- | 2 | S-5.01,S-5.05,S-2.01 |
| S-6.02 | End-to-End Integration Smoke Tests | prism-bin | 0 | -- | 2 | S-6.01 |
| S-6.03 | Installation and Distribution | prism-bin | 0 | -- | 1 | S-6.01 |

---

## BC Traceability Matrix

Every active BC maps to the story that implements it.

| BC | Story |
|----|-------|
| BC-2.01.002 | S-2.06 |
| BC-2.01.004 | S-2.07 |
| BC-2.01.005 | S-2.07 |
| BC-2.01.006 | S-2.07 |
| BC-2.01.007 | S-2.07 |
| BC-2.01.008 | S-2.07 |
| BC-2.01.010 | S-2.06 |
| BC-2.01.013 | S-2.06 |
| BC-2.01.014 | S-2.06 |
| BC-2.02.001 | S-1.04 |
| BC-2.02.002 | S-1.04 |
| BC-2.02.003 | S-1.05 |
| BC-2.02.004 | S-1.05 |
| BC-2.02.005 | S-1.05 |
| BC-2.02.006 | S-1.05 |
| BC-2.02.007 | S-1.05 |
| BC-2.02.008 | S-1.05 |
| BC-2.02.009 | S-1.04 |
| BC-2.02.010 | S-1.04 |
| BC-2.02.011 | S-1.05 |
| BC-2.02.012 | S-1.04 |
| BC-2.03.001 | S-1.06 |
| BC-2.03.002 | S-1.06 |
| BC-2.03.003 | S-1.06 |
| BC-2.03.004 | S-1.06 |
| BC-2.03.005 | S-1.07 |
| BC-2.03.006 | S-1.07 |
| BC-2.03.007 | S-1.07 |
| BC-2.03.008 | S-1.06 |
| BC-2.03.009 | S-1.07 |
| BC-2.03.010 | S-1.07 |
| BC-2.03.011 | S-1.06 |
| BC-2.03.012 | S-1.06 |
| BC-2.04.001 | S-1.08 |
| BC-2.04.002 | S-1.08 |
| BC-2.04.003 | S-1.08 |
| BC-2.04.004 | S-1.08 |
| BC-2.04.005 | S-1.08 |
| BC-2.04.006 | S-1.08 |
| BC-2.04.007 | S-1.09 |
| BC-2.04.008 | S-1.09 |
| BC-2.04.009 | S-1.09 |
| BC-2.04.010 | S-1.09 |
| BC-2.04.011 | S-1.09 |
| BC-2.04.012 | S-1.09 |
| BC-2.04.013 | S-1.08 |
| BC-2.04.015 | S-1.08 |
| BC-2.05.001 | S-2.04 |
| BC-2.05.002 | S-2.04 |
| BC-2.05.003 | S-2.04 |
| BC-2.05.004 | S-2.04 |
| BC-2.05.005 | S-2.05 |
| BC-2.05.006 | S-2.04 |
| BC-2.05.007 | S-2.05 |
| BC-2.05.008 | S-2.04 |
| BC-2.05.009 | S-2.05 |
| BC-2.05.010 | S-2.05 |
| BC-2.06.001 | S-5.05 |
| BC-2.06.002 | S-5.05 |
| BC-2.06.003 | S-5.05 |
| BC-2.06.004 | S-5.05 |
| BC-2.06.005 | S-5.05 |
| BC-2.06.006 | S-5.05 |
| BC-2.06.007 | S-5.05 |
| BC-2.06.008 | S-5.05 |
| BC-2.06.010 | S-5.05 |
| BC-2.07.001 | S-3.05 |
| BC-2.07.002 | S-3.05 |
| BC-2.07.003 | S-3.05 |
| BC-2.07.004 | S-3.05 |
| BC-2.07.005 | S-3.05 |
| BC-2.07.006 | S-3.05 |
| BC-2.08.001 | S-5.04 |
| BC-2.08.002 | S-5.04 |
| BC-2.08.003 | S-5.04 |
| BC-2.08.004 | S-5.04 |
| BC-2.08.005 | S-5.03 |
| BC-2.08.006 | S-5.03 |
| BC-2.08.007 | S-5.04 |
| BC-2.09.001 | S-1.10 |
| BC-2.09.002 | S-1.10 |
| BC-2.09.003 | S-1.10 |
| BC-2.09.004 | S-1.10 |
| BC-2.09.005 | S-1.10 |
| BC-2.09.006 | S-1.10 |
| BC-2.09.007 | S-1.10 |
| BC-2.09.008 | S-1.10 |
| BC-2.10.001 | S-5.01 |
| BC-2.10.002 | S-5.01 |
| BC-2.10.003 | S-5.01 |
| BC-2.10.004 | S-5.02 |
| BC-2.10.006 | S-5.01 |
| BC-2.10.007 | S-5.02 |
| BC-2.10.008 | S-5.03 |
| BC-2.10.009 | S-5.03 |
| BC-2.10.010 | S-5.01 |
| BC-2.10.011 | S-5.02 |
| BC-2.11.001 | S-3.02 |
| BC-2.11.002 | S-3.01 |
| BC-2.11.003 | S-3.01 |
| BC-2.11.004 | S-3.01 |
| BC-2.11.005 | S-3.02 |
| BC-2.11.006 | S-3.02 |
| BC-2.11.007 | S-3.02 |
| BC-2.11.008 | S-3.04 |
| BC-2.11.009 | S-3.04 |
| BC-2.11.010 | S-3.03 |
| BC-2.11.011 | S-3.02 |
| BC-2.11.012 | S-3.02 |
| BC-2.11.013 | S-3.04 |
| BC-2.11.014 | S-3.04 |
| BC-2.11.015 | S-3.04 |
| BC-2.12.001 | S-4.01 |
| BC-2.12.002 | S-4.01 |
| BC-2.12.003 | S-4.01 |
| BC-2.12.004 | S-4.01 |
| BC-2.12.005 | S-4.02 |
| BC-2.12.006 | S-4.02 |
| BC-2.12.007 | S-4.02 |
| BC-2.12.008 | S-4.02 |
| BC-2.12.009 | S-4.02 |
| BC-2.12.010 | S-4.01 |
| BC-2.13.001 | S-4.03 |
| BC-2.13.002 | S-4.04 |
| BC-2.13.003 | S-4.04 |
| BC-2.13.004 | S-4.04 |
| BC-2.13.005 | S-4.05 |
| BC-2.13.006 | S-4.03 |
| BC-2.13.007 | S-4.03 |
| BC-2.13.008 | S-4.03 |
| BC-2.13.009 | S-4.03 |
| BC-2.13.010 | S-4.03 |
| BC-2.13.011 | S-4.03 |
| BC-2.13.012 | S-4.04 |
| BC-2.13.013 | S-4.04 |
| BC-2.14.001 | S-4.06 |
| BC-2.14.002 | S-4.06 |
| BC-2.14.003 | S-4.06 |
| BC-2.14.004 | S-4.06 |
| BC-2.14.005 | S-4.06 |
| BC-2.14.006 | S-4.06 |
| BC-2.14.007 | S-4.06 |
| BC-2.14.008 | S-4.07 |
| BC-2.14.009 | S-4.06 |
| BC-2.14.010 | S-4.07 |
| BC-2.14.012 | S-4.07 |
| BC-2.15.001 | S-2.01 |
| BC-2.15.002 | S-2.01 |
| BC-2.15.003 | S-2.02 |
| BC-2.15.004 | S-2.02 |
| BC-2.15.005 | S-2.01 |
| BC-2.15.006 | S-2.02 |
| BC-2.15.007 | S-2.02 |
| BC-2.15.008 | S-2.02 |
| BC-2.15.009 | S-2.03 |
| BC-2.15.010 | S-2.03 |
| BC-2.15.011 | S-2.03 |
| BC-2.16.001 | S-1.11 |
| BC-2.16.002 | S-1.11 |
| BC-2.16.003 | S-1.11 |
| BC-2.16.004 | S-1.11 |
| BC-2.16.005 | S-1.12 |
| BC-2.16.006 | S-1.12 |
| BC-2.16.007 | S-1.12 |
| BC-2.16.008 | S-1.12 |
| BC-2.16.009 | S-1.11 |
| BC-2.16.010 | S-1.12 |

---

## VP Assignment Matrix

| VP | Story | Type |
|----|-------|------|
| VP-001 | S-1.01 | Kani proof: TenantId validation |
| VP-002 | S-1.03 | Kani proof: deny-by-default |
| VP-003 | S-1.03 | Kani proof: most-specific path wins |
| VP-004 | S-1.03 | Kani proof: Deny overrides Allow at same level |
| VP-005 | S-1.02 | Kani proof: exactly 12 CaseStatus transitions |
| VP-006 | S-1.02 | Kani proof: no self-transitions |
| VP-007 | S-1.09 | Kani proof: confirmation token expiry |
| VP-008 | S-1.09 | Kani proof: token single-use |
| VP-009 | S-1.09 | Kani proof: token entropy minimum |
| VP-010 | S-1.09 | Kani proof: token scope isolation |
| VP-011 | S-1.02 | Kani proof: CredentialName path traversal rejection |
| VP-012 | S-3.04 | Proptest: alias resolution idempotent |
| VP-013 | S-3.04 | Proptest: alias chains terminate |
| VP-014 | S-3.01 | Kani proof: PrismQL parser accepts all valid syntax |
| VP-015 | S-3.01 | Fuzz target: parser never panics on arbitrary input |
| VP-016 | S-1.04 | Proptest: normalize() output is always valid protobuf |
| VP-017 | S-1.05 | Proptest: unmapped fields always in raw_extensions |
| VP-018 | S-4.03 | Kani proof: detection rule compilation is deterministic |
| VP-019 | S-4.02 | Proptest: differential result ordering is stable |
| VP-020 | S-1.08 | Kani proof: feature flag deny-by-default |
| VP-021 | S-3.01 | Proptest: parse then unparse is round-trip stable |
| VP-022 | S-1.04 | Fuzz target: normalize() never panics |
| VP-023 | S-1.11 | Proptest: spec pipeline output is deterministic |
| VP-024 | S-1.10 | Kani proof: injection scanner rejects all OWASP LLM Top 10 patterns |
| VP-025 | S-3.04 | Kani proof: alias max depth enforced |
| VP-026 | S-4.01 | Kani proof: schedule next-fire never regresses |
| VP-027 | S-4.04 | Kani proof: detection evaluation is monotonic |
| VP-028 | S-4.05 | Kani proof: alert severity maps correctly from detection |
| VP-029 | S-1.02 | Kani proof: CursorId enforces 200-cap |
| VP-030 | S-4.01 | Proptest: schedule execution loop makes progress |
| VP-031 | S-3.02 | Proptest: query materialization produces valid OCSF |
| VP-032 | S-1.12 | Proptest: hot reload preserves in-flight queries |
| VP-033 | S-2.04 | Kani proof: audit entry tamper detection |
| VP-034 | S-1.06 | Proptest: encrypt then decrypt returns original value |
| VP-035 | S-1.06 | Proptest: key derivation is deterministic |
| VP-036 | S-4.04 | Proptest: sequence detection window boundary correctness |
| VP-037 | S-3.04 | Proptest: alias expansion does not leak cross-tenant |

---

## Topological Order (Dependency Validation)

Topological sort confirms the dependency graph is acyclic. Execution order:

```
Layer 0 (no deps):   S-1.01
Layer 1:             S-1.02, S-1.03, S-1.04, S-1.10, S-1.11, S-3.01, S-2.01
Layer 2:             S-1.05, S-1.06, S-1.08, S-1.12, S-2.02, S-2.03
Layer 3:             S-1.07, S-1.09, S-2.04, S-2.06
Layer 4:             S-2.05, S-2.07, S-3.02
Layer 5:             S-3.03, S-3.04, S-3.05, S-4.01, S-4.03
Layer 6:             S-4.02, S-4.04, S-5.01
Layer 7:             S-4.05, S-5.02
Layer 8:             S-4.06, S-5.03
Layer 9:             S-4.07, S-5.04, S-5.05
Layer 10:            S-6.01
Layer 11:            S-6.02, S-6.03
```

No cycles detected. Wave assignments follow these layers grouped by crate boundary.
