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

```mermaid
graph TD
    BIN["prism-bin"]
    MCP["prism-mcp"]
    QUERY["prism-query"]
    OPS["prism-operations"]
    SENS["prism-sensors"]
    SPEC["prism-spec-engine"]
    OCSF["prism-ocsf"]
    SEC["prism-security"]
    CRED["prism-credentials"]
    STOR["prism-storage"]
    AUD["prism-audit"]
    CORE["prism-core"]

    BIN --> MCP
    BIN --> STOR
    BIN --> CORE

    MCP --> QUERY
    MCP --> OPS
    MCP --> SEC
    MCP --> AUD

    QUERY --> SENS
    QUERY --> OCSF
    QUERY --> SPEC
    QUERY --> STOR

    OPS --> QUERY
    OPS --> SPEC
    OPS --> SEC
    OPS --> AUD
    OPS --> STOR

    SENS --> SPEC
    SENS --> CRED

    AUD --> STOR

    SPEC --> CORE
    OCSF --> CORE
    SEC --> CORE
    CRED --> CORE
    STOR --> CORE

    style CORE fill:#e94560,stroke:#ff6b6b,color:#fff,font-weight:bold
    style QUERY fill:#533483,stroke:#7c3aed,color:#fff,font-weight:bold
    style BIN fill:#636e72,stroke:#b2bec3,color:#fff
    style MCP fill:#0f3460,stroke:#533483,color:#e0e0e0
    style OPS fill:#0f3460,stroke:#533483,color:#e0e0e0
    style SENS fill:#1a1a2e,stroke:#0f3460,color:#e0e0e0
    style SPEC fill:#1a1a2e,stroke:#0f3460,color:#e0e0e0
    style OCSF fill:#0f3460,stroke:#533483,color:#e0e0e0
    style SEC fill:#0f3460,stroke:#533483,color:#e0e0e0
    style CRED fill:#1a1a2e,stroke:#0f3460,color:#e0e0e0
    style STOR fill:#1a1a2e,stroke:#0f3460,color:#e0e0e0
    style AUD fill:#1a1a2e,stroke:#0f3460,color:#e0e0e0
```

## Build Order Visualization

```mermaid
gantt
    title Topological Build Order (parallel within levels)
    dateFormat X
    axisFormat %s

    section Level 0
    prism-core           :core, 0, 1

    section Level 1
    prism-ocsf           :ocsf, 1, 2
    prism-credentials    :cred, 1, 2
    prism-security       :sec, 1, 2
    prism-spec-engine    :spec, 1, 2

    section Level 2
    prism-storage        :stor, 2, 3

    section Level 3
    prism-audit          :aud, 3, 4
    prism-sensors        :sens, 3, 4

    section Level 4
    prism-query          :query, 4, 5

    section Level 5
    prism-operations     :ops, 5, 6

    section Level 6
    prism-mcp            :mcp, 6, 7

    section Level 7
    prism-bin            :bin, 7, 8
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
| 5 | prism-operations | prism-query, prism-spec-engine, prism-security, prism-audit, prism-storage, prism-core |
| 6 | prism-mcp | prism-query, prism-operations, prism-security, prism-audit, prism-core |
| 7 | prism-bin | prism-mcp, prism-storage, prism-core |

## Dependency Rules

1. **prism-core depends on nothing.** It is the leaf crate. All shared types, errors, and config structures live here.
2. **No upward dependencies.** Lower-layer crates never depend on higher-layer crates. prism-storage never imports from prism-mcp.
3. **No peer dependencies between infrastructure crates.** prism-credentials does not depend on prism-storage; prism-audit does not depend on prism-credentials. They communicate through traits defined in prism-core.
4. **prism-query depends on prism-sensors but not vice versa.** The query engine orchestrates sensor adapters, not the other way around. Sensor adapters do not know about DataFusion or PrismQL.
5. **prism-operations depends on prism-query.** The scheduler and detection engine use the query engine to execute queries. They do not directly call sensor adapters.
6. **Feature-gated dependencies.** Write-operation code paths in prism-mcp are behind Cargo feature gates (e.g., `crowdstrike-write`). If the feature is not enabled, the dependency on write-specific sensor adapter code is not compiled.

## External Dependency Summary

| External Crate | Used By | Purpose | Version |
|----------------|---------|---------|---------|
| rmcp | prism-mcp | MCP SDK (server, tools, transport) | 1.4 |
| datafusion | prism-query | SQL execution engine | 53 |
| arrow | prism-query, prism-ocsf | Columnar in-memory format | 53 |
| chumsky | prism-query | PrismQL parser combinator | 0.12 |
| rust-rocksdb | prism-storage | Persistent key-value storage | 0.24 |
| prost | prism-ocsf | Protobuf message encoding | 0.13 (pin exact in Cargo.toml — proto field stability per ASM-005) |
| prost-reflect | prism-ocsf | DynamicMessage runtime reflection | 0.14 (pin exact — DynamicMessage API stability critical) |
| keyring | prism-credentials | OS keyring access | 3.x (verify cross-platform per ASM-003) |
| vaultrs | prism-credentials | HashiCorp Vault client (feature: `vault`) | 0.8 |
| aws-sdk-secretsmanager | prism-credentials | AWS Secrets Manager (feature: `aws-sm`) | latest |
| azure_security_keyvault_secrets | prism-credentials | Azure Key Vault (feature: `azure-kv`) | latest |
| google-cloud-secretmanager-v1 | prism-credentials | GCP Secret Manager (feature: `gcp-sm`) | latest |
| reqwest | prism-sensors | HTTP client for sensor APIs | 0.12 |
| tokio | all crates | Async runtime | 1.x |
| serde / serde_json | all crates | Serialization | 1.x |
| arc-swap | prism-spec-engine, prism-core | Lock-free config access | 1.x |
| notify | prism-spec-engine | Cross-platform filesystem watcher (inotify/FSEvents/ReadDirectoryChangesW) | 7.x |
| git2 | prism-spec-engine | Git repo operations for config source sync (libgit2 bindings) | latest |
| wasmtime | prism-spec-engine | WASM Component Model runtime for sensor plugin execution | latest stable |
| wit-bindgen | (plugin authors) | WIT interface code generation for plugin development | latest stable |
| bincode | prism-storage | Binary serialization for RocksDB values | 2.x |
| uuid | prism-core | UUID v7 generation for alerts/cases | 1.x |
| tracing | all crates | Structured logging | 0.1 |
| ipnet | prism-operations | subnet_contains() UDF for detection rules | latest |
| regex | prism-security, prism-query | Pattern matching (injection detection, IOC match) | latest |
| scopeguard | prism-operations | RAII guard for SessionContext drop on error/panic (VP-036) | 1.x |
| cron | prism-operations | Cron expression parsing for action scheduled triggers (AD-021) | 0.12 |
| blake3 | prism-operations | Row hashing for differential result computation | 1.x |
| toml | prism-spec-engine, prism-operations | TOML parsing for sensor specs, detection rules, packs | 0.8 |
| ariadne | prism-query | Error formatting with source spans for PrismQL parse errors | 0.4 |
| maxminddb | prism-spec-engine | MaxMind MMDB reader for GeoIP infusion | latest |
| lru | prism-spec-engine | In-memory LRU cache for infusion Tier 2 caching | 0.12 |
| lettre | prism-operations | SMTP email delivery for action framework | 0.11 |
| sha2 | prism-operations | SHA-256 hashing for action deduplication keys | 0.10 |
