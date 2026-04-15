---
document_type: architecture-section
level: L3
section: "dependency-graph"
version: "1.0"
status: draft
producer: architect
timestamp: 2026-04-15T12:00:00
phase: 1b
inputs: [domain-spec/L2-INDEX.md, prd.md]
traces_to: ARCH-INDEX.md
---

# Dependency Graph

## Inter-Crate Dependencies

All arrows point downward (toward prism-core). The graph is strictly acyclic — no circular dependencies.

```
prism-bin
  |
  +---> prism-mcp
  |       |
  |       +---> prism-query
  |       |       |
  |       |       +---> prism-sensors
  |       |       |       |
  |       |       |       +---> prism-spec-engine ---> prism-core
  |       |       |       |
  |       |       |       +---> prism-credentials ---> prism-core
  |       |       |
  |       |       +---> prism-ocsf ---> prism-core
  |       |       |
  |       |       +---> prism-storage ---> prism-core
  |       |
  |       +---> prism-operations
  |       |       |
  |       |       +---> prism-query (re-use, not circular)
  |       |       |
  |       |       +---> prism-storage ---> prism-core
  |       |
  |       +---> prism-security ---> prism-core
  |       |
  |       +---> prism-audit
  |               |
  |               +---> prism-storage ---> prism-core
  |
  +---> prism-storage ---> prism-core
  |
  +---> prism-core (direct, for startup config)
```

## Topological Build Order

Build order from leaves to root (each level can build in parallel):

| Level | Crates | Dependencies Satisfied |
|-------|--------|----------------------|
| 0 | prism-core | (none — leaf crate) |
| 1 | prism-ocsf, prism-credentials, prism-security, prism-spec-engine | prism-core |
| 2 | prism-storage | prism-core |
| 3 | prism-audit, prism-sensors | prism-storage, prism-credentials, prism-spec-engine, prism-core |
| 4 | prism-query | prism-sensors, prism-ocsf, prism-storage, prism-spec-engine, prism-core |
| 5 | prism-operations | prism-query, prism-storage, prism-core |
| 6 | prism-mcp | prism-query, prism-operations, prism-security, prism-audit, prism-core |
| 7 | prism-bin | prism-mcp, prism-storage, prism-core |

## Dependency Rules

1. **prism-core depends on nothing.** It is the leaf crate. All shared types, errors, and config structures live here.
2. **No upward dependencies.** Lower-layer crates never depend on higher-layer crates. prism-storage never imports from prism-mcp.
3. **No peer dependencies between infrastructure crates.** prism-credentials does not depend on prism-storage; prism-audit does not depend on prism-credentials. They communicate through traits defined in prism-core.
4. **prism-query depends on prism-sensors but not vice versa.** The query engine orchestrates sensor adapters, not the other way around. Sensor adapters do not know about DataFusion or AxiQL.
5. **prism-operations depends on prism-query.** The scheduler and detection engine use the query engine to execute queries. They do not directly call sensor adapters.
6. **Feature-gated dependencies.** Write-operation code paths in prism-mcp are behind Cargo feature gates (e.g., `crowdstrike-write`). If the feature is not enabled, the dependency on write-specific sensor adapter code is not compiled.

## External Dependency Summary

| External Crate | Used By | Purpose | Version |
|----------------|---------|---------|---------|
| rmcp | prism-mcp | MCP SDK (server, tools, transport) | 1.4 |
| datafusion | prism-query | SQL execution engine | 53 |
| arrow | prism-query, prism-ocsf | Columnar in-memory format | 53 |
| chumsky | prism-query | AxiQL parser combinator | 0.12 |
| rust-rocksdb | prism-storage | Persistent key-value storage | 0.24 |
| prost | prism-ocsf | Protobuf message encoding | latest |
| prost-reflect | prism-ocsf | DynamicMessage runtime reflection | latest |
| keyring | prism-credentials | OS keyring access | latest |
| reqwest | prism-sensors | HTTP client for sensor APIs | latest |
| tokio | all crates | Async runtime | 1.x |
| serde / serde_json | all crates | Serialization | 1.x |
| arc-swap | prism-spec-engine, prism-core | Lock-free config access | latest |
| bincode | prism-storage | Binary serialization for RocksDB values | 2.x |
| uuid | prism-core | UUID v7 generation for alerts/cases | 1.x |
| tracing | all crates | Structured logging | 0.1 |
| ipnet | prism-query | subnet_contains() UDF | latest |
| regex | prism-security, prism-query | Pattern matching (injection detection, IOC match) | latest |
