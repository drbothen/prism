---
document_type: story-index
level: L4
version: "1.1"
status: draft
producer: story-writer
timestamp: 2026-04-16T14:00:00
phase: 3
total_stories: 53
total_bcs_covered: 169
total_vps_assigned: 38
---

# Prism Phase 3 Story Index

## Overview

Phase 3 decomposes the Prism platform into 53 implementation stories spanning 6 parallel
waves. Stories are organized by crate and ordered topologically so that no story begins
before its dependencies are complete.

- **Total stories:** 53 (46 core + 7 osquery-inspired enhancements)
- **Total waves:** 6
- **BCs covered:** 169 (across SS-01 through SS-16, excluding 14 removed; includes 1 STUB: BC-2.14.012)
- **VPs assigned:** 38 (19 Kani proofs, 11 proptests, 6 fuzz targets, 2 integration tests)
- **Note:** The 7 osquery-inspired stories (S-2.08, S-3.08 through S-3.13) have 0 formal BCs at this stage — they are enhancements derived from the osquery synthesis review.

Every story contains: narrative, behavioral contracts table, numbered tasks, acceptance
criteria (Given/When/Then), verification properties, and notes. No story exceeds 3
estimated days. No story's estimated context exceeds 30% of the implementing agent's
context window.

---

## Wave Summary

| Wave | Crates | Stories | BCs | Theme |
|------|--------|---------|-----|-------|
| 1 | prism-core, prism-ocsf, prism-credentials, prism-security, prism-spec-engine | 15 | 58 (+ 5 stories with 0 BCs) | Foundation + Pure Domain |
| 2 | prism-storage, prism-audit, prism-sensors | 8 | 30 | Infrastructure + Adapters |
| 3 | prism-query | 13 | 28 | Query Engine (incl. write ops + osquery enhancements) |
| 4 | prism-operations | 8 | 36 | Operations |
| 5 | prism-mcp (+ SS-06 config) | 6 | 26 | MCP Server + Config |
| 6 | prism-bin | 3 | 0 (infra) | Binary + E2E |

Wave 1 stories have no dependencies outside the wave (except S-1.01 which is the root).
Wave 2 stories depend on Wave 1. Wave 3 depends on Wave 2. Waves 4-6 follow in order.
All dependency chains are acyclic (validated by topological sort below).
Per-wave BC counts are raw story-BC assignments (sum=178); 9 BCs are shared across waves,
9 BCs are shared across stories, so unique BCs = 169 (matching the traceability matrix and header count).

**NOTE on wave vs. topological scheduling:** Wave assignments are grouped by crate boundary
for organizational clarity. The topological sort (below) shows that some stories can start
earlier than their wave number suggests — e.g., S-3.01 (Wave 3) and S-2.01 (Wave 2) are
both in topological Layer 1, meaning they can begin as soon as S-1.01 completes. Teams
pursuing maximum parallelism should schedule by topological layer, not wave number.

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
| S-1.10 | Prompt Injection Defense | prism-security | 8 | VP-024,038 | 2 | S-1.01 |
| S-1.11 | Spec Loading and Pipeline Execution | prism-spec-engine | 5 | VP-023 | 3 | S-1.01 |
| S-1.12 | Hot Reload and Runtime Management | prism-spec-engine | 5 | VP-032 | 2 | S-1.11 |
| S-1.13 | Sensor Spec Write Endpoints | prism-spec-engine | 2 | -- | 2 | S-1.11 |
| S-1.14 | Infusion Spec Loading and UDF Registration | prism-spec-engine | 0 | -- | 3 | S-1.11 |
| S-1.15 | WASM Plugin Runtime | prism-spec-engine | 0 | -- | 3 | S-1.11 |
| S-2.01 | RocksDB Initialization and Domain Operations | prism-storage | 3 | -- | 3 | S-1.01 |
| S-2.02 | Audit Buffer and Watchdog | prism-storage | 5 | -- | 2 | S-2.01 |
| S-2.03 | Decorators and Internal Tables | prism-storage | 3 | -- | 2 | S-2.01,S-1.02 |
| S-2.04 | Audit Entry Construction and Compliance | prism-audit | 6 | VP-033 | 3 | S-2.01,S-2.02 |
| S-2.05 | Specialized Audit Events | prism-audit | 4 | -- | 1 | S-2.04 |
| S-2.06 | DataSource Trait and Auth Patterns | prism-sensors | 4 | -- | 3 | S-1.06,S-1.11 |
| S-2.07 | Per-Sensor Auth and Pagination | prism-sensors | 5 | -- | 3 | S-2.06 |
| S-2.08 | Event Table Abstraction and Local Buffering | prism-sensors | 0 | -- | 3 | S-2.06,S-2.01,S-1.11 |
| S-3.01 | PrismQL Parser (Filter + SQL + Pipe) | prism-query | 4 | VP-014,015,021 | 3 | S-1.01 |
| S-3.02 | Query Tool and Materialization | prism-query | 6 | VP-031 | 3 | S-3.01,S-2.06,S-1.04,S-2.01,S-2.03 |
| S-3.03 | Explain and Query Diagnostics | prism-query | 1 | -- | 1 | S-3.02 |
| S-3.04 | Alias System (P1) | prism-query | 5 | VP-012,013,025,037 | 2 | S-3.02,S-1.08,S-1.09 |
| S-3.05 | Pagination and Caching | prism-query | 6 | -- | 2 | S-3.02 |
| S-3.06 | PrismQL Write Parser Extensions | prism-query | 1 | -- | 2 | S-3.01,S-1.13 |
| S-3.07 | Write Execution Pipeline | prism-query | 5 | -- | 3 | S-3.06,S-3.02,S-1.08,S-1.09,S-2.04 |
| S-3.08 | Hidden Columns | prism-query | 0 | -- | 1 | S-3.02 |
| S-3.09 | Query Performance Profiling | prism-query | 0 | -- | 1 | S-3.02 |
| S-3.10 | Cost Estimation (API Latency-Aware Planner) | prism-query | 0 | -- | 2 | S-3.09,S-3.02 |
| S-3.11 | In-Query Dedup Caching | prism-query | 0 | -- | 1 | S-3.02 |
| S-3.12 | Column Pruning and Field Selection Push-Down | prism-query | 0 | -- | 1 | S-3.02,S-2.06 |
| S-3.13 | Dynamic Table Availability | prism-query | 0 | -- | 1 | S-3.02,S-1.12 |
| S-4.01 | Schedule CRUD and Execution Loop | prism-operations | 5 | VP-026,030 | 3 | S-3.02,S-2.01 |
| S-4.02 | Differential Results and Packs | prism-operations | 5 | VP-019 | 2 | S-4.01 |
| S-4.03 | Detection Rule Loading and Compilation | prism-operations | 7 | VP-018 | 3 | S-3.02,S-1.08,S-2.01 |
| S-4.04 | Detection Evaluation (Single/Correlation/Sequence) | prism-operations | 5 | VP-027,036 | 3 | S-4.03 |
| S-4.05 | Alert Generation | prism-operations | 1 | VP-028 | 1 | S-4.04 |
| S-4.06 | Case Management | prism-operations | 8 | -- | 3 | S-4.05,S-2.01 |
| S-4.07 | Case Metrics and Acknowledge Alert | prism-operations | 3 | -- | 2 | S-4.06 |
| S-4.08 | Action Delivery Framework | prism-operations | 2 | -- | 3 | S-4.05,S-4.06,S-4.01,S-1.15 |
| S-5.01 | Server Bootstrap and Tool Registration | prism-mcp | 5 | -- | 3 | S-1.08,S-3.02,S-4.01 |
| S-5.02 | Tool Routing, Errors, and Client Scoping | prism-mcp | 3 | -- | 2 | S-5.01 |
| S-5.03 | Resources and Prompts | prism-mcp | 4 | -- | 2 | S-5.02 |
| S-5.04 | Sensor Health Subsystem | prism-mcp | 5 | -- | 2 | S-5.03,S-2.07 |
| S-5.05 | Config Loading and Validation | prism-mcp | 9 | -- | 3 | S-5.01,S-1.06 |
| S-5.06 | Action and Infusion MCP Tools | prism-mcp | 0 | -- | 2 | S-5.01,S-4.08,S-1.14 |
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
| BC-2.04.001 | S-1.08, S-3.07 |
| BC-2.04.002 | S-1.08 |
| BC-2.04.003 | S-1.08 |
| BC-2.04.004 | S-1.08 |
| BC-2.04.005 | S-1.08, S-3.07 |
| BC-2.04.006 | S-1.08 |
| BC-2.04.007 | S-1.09, S-3.07 |
| BC-2.04.008 | S-1.09, S-3.07 |
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
| BC-2.05.009 | S-2.05, S-3.07 |
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
| BC-2.11.004 | S-3.01, S-3.06 |
| BC-2.11.005 | S-3.02 |
| BC-2.11.006 | S-3.01, S-3.02 |
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
| BC-2.12.011 | S-4.08 |
| BC-2.12.012 | S-4.08 |
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
| BC-2.16.001 | S-1.11, S-1.13 |
| BC-2.16.002 | S-1.11 |
| BC-2.16.003 | S-1.11 |
| BC-2.16.004 | S-1.11 |
| BC-2.16.005 | S-1.12 |
| BC-2.16.006 | S-1.12 |
| BC-2.16.007 | S-1.12 |
| BC-2.16.008 | S-1.12 |
| BC-2.16.009 | S-1.11, S-1.13 |
| BC-2.16.010 | S-1.12 |

---

## VP Assignment Matrix

| VP | Story | Method | Property (from verification-architecture.md) |
|----|-------|--------|----------------------------------------------|
| VP-001 | S-1.01 | kani | TenantId rejects invalid characters |
| VP-002 | S-1.03 | kani | Capability resolution: deny-by-default |
| VP-003 | S-1.03 | kani | Capability resolution: most-specific-path wins |
| VP-004 | S-1.03 | kani | Capability resolution: deny overrides allow at same specificity |
| VP-005 | S-1.02 | kani | Case state machine: exactly 12 valid transitions |
| VP-006 | S-1.02 | kani | Case state machine: no self-transitions |
| VP-007 | S-1.09 | kani | Confirmation token expiry: expired at boundary (inclusive) |
| VP-008 | S-1.09 | kani | Confirmation token: single-use (consumed rejects second use) |
| VP-009 | S-1.09 | kani | Confirmation token: content hash mismatch rejects |
| VP-010 | S-1.09 | kani | Token cap: store rejects at 100 active tokens |
| VP-011 | S-1.02 | kani | Credential name sanitization: rejects path traversal |
| VP-012 | S-3.04 | kani | Alias depth: rejects composition beyond depth 3 |
| VP-013 | S-3.04 | proptest | Alias cycles: detects and rejects cyclic references |
| VP-014 | S-3.01 | kani | Query security limits: rejects oversized queries |
| VP-015 | S-3.01 | kani | Query security limits: rejects excessive nesting depth |
| VP-016 | S-1.04 | proptest | OCSF normalization: output is valid protobuf |
| VP-017 | S-1.05 | proptest | OCSF normalization: unmapped fields preserved in raw_extensions |
| VP-018 | S-4.03 | proptest | Detection rule validation: rejects invalid rules |
| VP-019 | S-4.02 | proptest | Diff computation: deterministic (same inputs → same output) |
| VP-020 | S-1.08 | kani | Feature flag: compile-time AND runtime must both permit |
| VP-021 | S-3.01 | fuzz | PrismQL parser: never panics on arbitrary input |
| VP-022 | S-1.04 | fuzz | OCSF normalizer: never panics on arbitrary sensor response |
| VP-023 | S-1.11 | fuzz | Sensor spec parser: never panics on arbitrary TOML |
| VP-024 | S-1.10 | proptest | Injection scanner: detects known injection patterns |
| VP-025 | S-3.04 | kani | Cache key derivation: deterministic for same parameters |
| VP-026 | S-4.01 | kani | Splay computation: deterministic per (query, client) |
| VP-027 | S-4.04 | proptest | Alert dedup key: correct per match mode |
| VP-028 | S-4.05 | fuzz | Template interpolation: never panics, handles missing vars |
| VP-029 | S-1.02 | kani | Cursor cap: rejects at 200 active cursors |
| VP-030 | S-4.01 | kani | Schedule/rule count caps: rejects beyond limits |
| VP-031 | S-3.02 | proptest | Required column enforcement: rejects unconstrained queries |
| VP-032 | S-1.12 | proptest | Hot reload atomicity: failed validation retains old config |
| VP-033 | S-2.04 | integration_test | Audit buffer: RocksDB write completes before delivery attempt |
| VP-034 | S-1.06 | proptest | Encryption round-trip: encrypt then decrypt returns plaintext |
| VP-035 | S-1.06 | proptest | Key derivation: same inputs produce same key |
| VP-036 | S-4.04 | integration_test | SessionContext dropped before error propagation and on panic |
| VP-037 | S-3.04 | fuzz | Alias expansion: never panics on arbitrary alias graphs |
| VP-038 | S-1.10 | fuzz | Injection scanner: never panics on arbitrary input strings |

---

## Topological Order (Dependency Validation)

Topological sort confirms the dependency graph is acyclic. Execution order:

```
Layer 0 (no deps):   S-1.01
Layer 1:             S-1.02, S-1.03, S-1.04, S-1.10, S-1.11, S-3.01, S-2.01
Layer 2:             S-1.05, S-1.06, S-1.08, S-1.12, S-1.13, S-1.14, S-1.15, S-2.02, S-2.03
Layer 3:             S-1.07, S-1.09, S-2.04, S-2.06, S-3.06
Layer 4:             S-2.05, S-2.07, S-2.08, S-3.02
Layer 5:             S-3.03, S-3.04, S-3.05, S-3.07, S-3.08, S-3.11, S-3.12, S-3.13, S-4.01, S-4.03
Layer 6:             S-3.09, S-4.02, S-4.04, S-5.01
Layer 7:             S-3.10, S-4.05, S-5.02, S-5.05
Layer 8:             S-4.06, S-5.03, S-6.01
Layer 9:             S-4.07, S-4.08, S-5.04, S-6.02, S-6.03
Layer 10:            S-5.06
```

Notes on story placement:
- S-1.13 (write endpoint specs) lands in Layer 2 — depends only on S-1.11 (Layer 1)
- S-1.14 (infusion specs) lands in Layer 2 — depends only on S-1.11 (Layer 1)
- S-1.15 (WASM plugin runtime) lands in Layer 2 — depends only on S-1.11 (Layer 1)
- S-3.06 (write parser) lands in Layer 3 — depends on S-3.01 (Layer 1) and S-1.13 (Layer 2)
- S-2.08 (event tables) lands in Layer 4 — depends on S-2.06 (Layer 3), S-2.01 (Layer 1),
  and S-1.11 (Layer 1). Gated by S-2.06 as the longest dep chain.
- S-3.07 (write execution) lands in Layer 5 — depends on S-3.06 (Layer 3), S-3.02 (Layer 4),
  S-1.08 (Layer 2), S-1.09 (Layer 3), and S-2.04 (Layer 3). Gated by S-3.02.
- S-3.08 (hidden columns) lands in Layer 5 — depends only on S-3.02 (Layer 4)
- S-3.11 (in-query caching) lands in Layer 5 — depends only on S-3.02 (Layer 4)
- S-3.12 (column pruning) lands in Layer 5 — depends on S-3.02 (Layer 4) and S-2.06 (Layer 3).
  Gated by S-3.02.
- S-3.13 (dynamic table availability) lands in Layer 5 — depends on S-3.02 (Layer 4) and
  S-1.12 (Layer 2). Gated by S-3.02.
- S-3.09 (query profiling) lands in Layer 6 — depends only on S-3.02 (Layer 4) but logically
  positioned here to allow S-3.08/S-3.11/S-3.12/S-3.13 to be wired into it.
- S-3.10 (cost estimation) lands in Layer 7 — depends on S-3.09 (Layer 6) and S-3.02 (Layer 4).
  Gated by S-3.09.
- S-4.08 (action delivery) lands in Layer 9 — depends on S-4.05 (Layer 7), S-4.06 (Layer 8),
  S-4.01 (Layer 5), and S-1.15 (Layer 2). Gated by S-4.06 (Layer 8) as the longest dep chain.
- S-5.06 (action/infusion tools) lands in Layer 10 — depends on S-5.01 (Layer 6), S-4.08
  (Layer 9), and S-1.14 (Layer 2). Gated by S-4.08 as the longest dep chain.

No cycles detected. Wave assignments follow these layers grouped by crate boundary.
