---
document_type: architecture-section
level: L3
section: "system-overview"
version: "1.0"
status: draft
producer: architect
timestamp: 2026-04-15T12:00:00
phase: 1b
inputs: [domain-spec/L2-INDEX.md, prd.md, prd-supplements/nfr-catalog.md]
traces_to: ARCH-INDEX.md
---

# System Overview

## Architecture Vision

Prism is an **ephemeral federated query engine** for MSSP security operations, implemented as a single Rust binary exposing an MCP tool interface over stdio transport. It follows the **data-in-flight** model: Query -> Fetch -> Normalize -> Compute -> Return -> Teardown. No data lake, no ETL pipeline, no index maintenance.

Architecturally analogous to Trino/Presto (federated SQL over heterogeneous sources) but purpose-built for the security domain: OCSF as universal schema, per-client multi-sensor fan-out, MCP as the AI-native interface.

## Architecture Pattern

**Modular monolith** via Cargo workspace. 12 crates with enforced dependency boundaries, compiled to a single binary. This matches the deployment model (one process per analyst in Claude Code) while providing module isolation through Rust's crate visibility system.

This is NOT a microservices architecture. There is no inter-service communication, no service discovery, no distributed state. The single-process invariant (DI-017) is a feature, not a limitation.

## Deployment Model

- **Runtime:** Per-analyst MCP server in Claude Code (stdio transport)
- **Process model:** One Prism binary per analyst terminal session
- **Concurrency:** Tokio multi-threaded runtime within the single process
- **State:** RocksDB in a local directory (`--state-dir`, default `./state`)
- **Configuration:** `prism.toml` + `aliases.toml` + `*.sensor.toml` spec files
- **Credentials:** OS keyring (primary) with AES-256-GCM encrypted file fallback

## Design Principles

1. **Data in flight, not at rest.** Sensor data exists only during query execution. The response cache is a performance optimization with TTL, not a data store.
2. **Query engine as universal interface.** All data access (external sensor APIs and internal Prism state) flows through AxiQL and DataFusion. The analyst writes one query language for everything.
3. **Config-driven extensibility.** New sensors are added by dropping a TOML spec file. Built-in sensors use the same spec system (eat our own dog food).
4. **Pure core, effectful shell.** Domain logic (parsing, validation, normalization, state machines) is separated from I/O (HTTP calls, RocksDB, keyring access) for testability and verification.
5. **Defense in depth for writes.** Compile-time cargo features + runtime per-client TOML flags + risk-tiered confirmation tokens. Three independent layers must all permit a write operation.
6. **Client isolation by construction.** `TenantId` newtype threading prevents cross-client data leakage at compile time.
7. **AI-first response design.** Every MCP response is structured for LLM consumption with `outputSchema`, trust annotations, and provenance framing for untrusted sensor data.

## System Boundaries

```
+------------------------------------------------------------------+
|  Claude Code (Analyst's Terminal)                                 |
|  +------------------------------------------------------------+  |
|  |  Prism MCP Server (single binary, stdio transport)         |  |
|  |  +------------------------------------------------------+  |  |
|  |  |  MCP Layer (rmcp 1.4, tool registration, routing)    |  |  |
|  |  +------------------------------------------------------+  |  |
|  |  |  Query Engine (Chumsky parser + DataFusion executor)  |  |  |
|  |  +------------------------------------------------------+  |  |
|  |  |  Sensor Adapters  |  OCSF Normalizer  |  Operations   |  |  |
|  |  |  (spec-driven)    |  (DynamicMessage)  |  (sched/det)  |  |  |
|  |  +------------------------------------------------------+  |  |
|  |  |  Platform (RocksDB, credentials, audit, watchdog)     |  |  |
|  |  +------------------------------------------------------+  |  |
|  +------------------------------------------------------------+  |
+------------------------------------------------------------------+
       |              |              |              |
  CrowdStrike    Cyberint      Claroty         Armis
  Falcon API     Argos API     xDome API       Centrix API
```

## Resource Constraints

| Resource | Budget | Source |
|----------|--------|--------|
| Process RSS | 512 MB | NFR-015, DI-027 |
| Per-query memory | 200 MB (normal), 100 MB (restrictive), 512 MB (permissive) | CAP-024 |
| Max materialized records | 10,000 per query | DI-019 |
| Query timeout | 30 seconds | DI-019 |
| Max concurrent schedules | 16 | DI-032 |
| Max active confirmation tokens | 100 | DI-015 |
| Cache entries | 50 per client per sensor (default) | DI-018 |
| Audit buffer | 100,000 entries max | CAP-025 |
| Max schedules | 500 | DI-028 |
| Max detection rules | 1,000 | DI-028 |
