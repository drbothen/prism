---
document_type: architecture-section
level: L3
section: "module-decomposition"
version: "1.0"
status: draft
producer: architect
timestamp: 2026-04-15T12:00:00
phase: 1b
inputs: [domain-spec/L2-INDEX.md, prd.md]
traces_to: ARCH-INDEX.md
---

# Module Decomposition

## Cargo Workspace Structure

Prism is a Cargo workspace with 12 crates organized in 4 layers: binary, application, domain, and infrastructure. Each crate has a single responsibility and explicit public API.

```
prism/
  Cargo.toml          (workspace root)
  prism-bin/           (binary crate — main entry point)
  prism-mcp/           (MCP server, tool registration, routing)
  prism-query/         (PrismQL parser + DataFusion query engine)
  prism-sensors/       (sensor adapter orchestration, auth traits)
  prism-spec-engine/   (TOML spec parser, pipeline executor)
  prism-ocsf/          (OCSF normalization via DynamicMessage)
  prism-operations/    (scheduler, differential, detection, alerts, cases)
  prism-security/      (feature flags, confirmation tokens, prompt injection)
  prism-credentials/   (credential store trait, keyring + file backends)
  prism-storage/       (RocksDB wrapper, StorageDomain, StorageBackend trait)
  prism-audit/         (audit entry construction, buffered forwarding)
  prism-core/          (shared types, errors, TenantId, config, decorators)
```

## Layered Architecture Diagram

```mermaid
graph TB
    subgraph L4["Layer 4: Binary"]
        BIN["prism-bin<br/><i>CLI, signals, startup</i>"]
    end

    subgraph L3["Layer 3: Presentation"]
        MCP["prism-mcp<br/><i>rmcp 1.4, 35+ tools, middleware</i>"]
    end

    subgraph L2["Layer 2: Business Logic"]
        QUERY["prism-query<br/><i>PrismQL parser, DataFusion,<br/>fan-out, UDFs</i>"]
        OPS["prism-operations<br/><i>scheduler, diff engine,<br/>detection, alerts, cases</i>"]
        OCSF["prism-ocsf<br/><i>OCSF normalization,<br/>DynamicMessage</i>"]
        SPEC["prism-spec-engine<br/><i>TOML spec parser,<br/>pipeline executor</i>"]
        SEC["prism-security<br/><i>feature flags, tokens,<br/>injection scanner</i>"]
    end

    subgraph L1["Layer 1: Infrastructure"]
        SENS["prism-sensors<br/><i>HTTP client, auth,<br/>adapter registry</i>"]
        CRED["prism-credentials<br/><i>keyring + AES file</i>"]
        STOR["prism-storage<br/><i>RocksDB wrapper,<br/>StorageBackend trait</i>"]
        AUD["prism-audit<br/><i>SOC 2 trail,<br/>buffered forwarding</i>"]
    end

    subgraph L0["Layer 0: Shared Foundation"]
        CORE["prism-core<br/><i>TenantId, PrismError, ConfigSnapshot,<br/>StorageDomain, entity types</i>"]
    end

    BIN --> MCP
    BIN --> STOR
    MCP --> QUERY
    MCP --> OPS
    MCP --> SEC
    MCP --> AUD
    QUERY --> SENS
    QUERY --> OCSF
    QUERY --> SPEC
    QUERY --> STOR
    OPS --> QUERY
    OPS --> SEC
    OPS --> AUD
    OPS --> STOR
    SENS --> SPEC
    SENS --> CRED
    AUD --> STOR

    OCSF --> CORE
    SEC --> CORE
    SPEC --> CORE
    CRED --> CORE
    STOR --> CORE
    SENS -.-> CORE
    QUERY -.-> CORE
    OPS -.-> CORE
    MCP -.-> CORE
    AUD -.-> CORE
    BIN -.-> CORE

    style L4 fill:#1a1a2e,stroke:#e94560,color:#e0e0e0
    style L3 fill:#16213e,stroke:#533483,color:#e0e0e0
    style L2 fill:#0f3460,stroke:#533483,color:#e0e0e0
    style L1 fill:#1a1a2e,stroke:#0f3460,color:#e0e0e0
    style L0 fill:#2d3436,stroke:#636e72,color:#e0e0e0
    style CORE fill:#e94560,stroke:#ff6b6b,color:#fff
    style QUERY fill:#533483,stroke:#7c3aed,color:#fff
```

## Crate Criticality & Purity

```mermaid
quadrantChart
    title Crate Classification (Criticality vs Purity)
    x-axis Pure Core --> Effectful Shell
    y-axis Low Criticality --> Critical
    quadrant-1 "Critical + Effectful"
    quadrant-2 "Critical + Pure"
    quadrant-3 "Lower + Pure"
    quadrant-4 "Lower + Effectful"
    prism-core: [0.1, 0.95]
    prism-ocsf: [0.15, 0.9]
    prism-security: [0.4, 0.9]
    prism-credentials: [0.85, 0.9]
    prism-query: [0.45, 0.85]
    prism-operations: [0.5, 0.7]
    prism-sensors: [0.9, 0.7]
    prism-spec-engine: [0.45, 0.65]
    prism-storage: [0.9, 0.65]
    prism-audit: [0.85, 0.6]
    prism-mcp: [0.9, 0.55]
    prism-bin: [0.95, 0.15]
```

## Component Map (Machine-Readable)

```yaml
components:
  - id: COMP-001
    name: "prism-bin"
    layer: "infrastructure"
    purity: "effectful-shell"
    criticality: "LOW"
    dependencies: [COMP-002, COMP-010, COMP-012]
    interfaces_provided: ["main() entry point", "CLI argument parsing", "signal handling"]
    interfaces_consumed: ["PrismServer from prism-mcp", "Storage from prism-storage", "Config from prism-core"]

  - id: COMP-002
    name: "prism-mcp"
    layer: "presentation"
    purity: "effectful-shell"
    criticality: "HIGH"
    dependencies: [COMP-003, COMP-008, COMP-009, COMP-011, COMP-012, COMP-007]
    interfaces_provided: ["PrismServer (rmcp ServerHandler)", "Tool registration", "MCP resources/prompts"]
    interfaces_consumed: ["QueryEngine", "FeatureFlagEvaluator", "CredentialStore", "AuditEmitter", "ConfigSnapshot"]

  - id: COMP-003
    name: "prism-query"
    layer: "business-logic"
    purity: "mixed"
    criticality: "CRITICAL"
    dependencies: [COMP-004, COMP-005, COMP-006, COMP-010, COMP-012]
    interfaces_provided: ["QueryEngine::execute()", "QueryEngine::execute_scheduled() -> (results, SessionContext)", "QueryEngine::explain()", "PrismQL parser", "UDF registry", "Infusion UDF registration"]
    interfaces_consumed: ["SensorAdapter", "SpecEngine", "OcsfNormalizer", "StorageBackend", "ConfigSnapshot", "InfusionRegistry"]

  - id: COMP-004
    name: "prism-sensors"
    layer: "infrastructure"
    purity: "effectful-shell"
    criticality: "HIGH"
    dependencies: [COMP-005, COMP-009, COMP-012]
    interfaces_provided: ["SensorAdapter trait", "SensorAuth sealed trait", "AdapterRegistry"]
    interfaces_consumed: ["SpecEngine", "CredentialStore", "ConfigSnapshot"]

  - id: COMP-005
    name: "prism-spec-engine"
    layer: "business-logic"
    purity: "mixed"
    criticality: "HIGH"
    dependencies: [COMP-012]
    interfaces_provided: ["SpecParser", "PipelineExecutor", "ConfigManager (arc-swap)", "PluginRuntime (wasmtime)", "InfusionRegistry", "InfusionPluginExecutor"]
    interfaces_consumed: ["ConfigSnapshot"]
    notes: "Owns WASM plugin runtime (AD-019), infusion spec loading + plugin execution (AD-020), sensor spec loading. Infusion UDFs are registered into prism-query's DataFusion SessionContext via InfusionRegistry."

  - id: COMP-006
    name: "prism-ocsf"
    layer: "business-logic"
    purity: "pure-core"
    criticality: "CRITICAL"
    dependencies: [COMP-012]
    interfaces_provided: ["OcsfNormalizer", "DynamicMessage", "FieldResolver", "EventClassSelector"]
    interfaces_consumed: ["OcsfSchema (compiled protobuf descriptors)"]

  - id: COMP-007
    name: "prism-operations"
    layer: "business-logic"
    purity: "mixed"
    criticality: "HIGH"
    dependencies: [COMP-003, COMP-005, COMP-008, COMP-010, COMP-011, COMP-012]
    interfaces_provided: ["Scheduler", "DiffEngine", "DetectionEngine", "AlertStore", "CaseManager", "ActionEngine"]
    interfaces_consumed: ["QueryEngine", "StorageBackend", "ConfigSnapshot", "InjectionScanner", "AuditEmitter", "PluginRuntime"]
    notes: "Owns action delivery (AD-021) — ActionEngine evaluates action specs against alerts/cases/schedules, renders templates, delivers via built-in types or WASM plugins. Action report queries execute through QueryEngine."

  - id: COMP-008
    name: "prism-security"
    layer: "business-logic"
    purity: "mixed"
    criticality: "CRITICAL"
    dependencies: [COMP-012]
    interfaces_provided: ["FeatureFlagEvaluator", "ConfirmationTokenStore", "PromptInjectionScanner", "SafetyFlagAggregator"]
    interfaces_consumed: ["ConfigSnapshot"]

  - id: COMP-009
    name: "prism-credentials"
    layer: "infrastructure"
    purity: "effectful-shell"
    criticality: "CRITICAL"
    dependencies: [COMP-012]
    interfaces_provided: ["CredentialStore trait", "KeyringBackend", "EncryptedFileBackend"]
    interfaces_consumed: ["TenantId", "error types"]

  - id: COMP-010
    name: "prism-storage"
    layer: "infrastructure"
    purity: "effectful-shell"
    criticality: "HIGH"
    dependencies: [COMP-012]
    interfaces_provided: ["StorageBackend trait", "RocksDbBackend", "InMemoryBackend (tests)"]
    interfaces_consumed: ["error types"]

  - id: COMP-011
    name: "prism-audit"
    layer: "infrastructure"
    purity: "effectful-shell"
    criticality: "HIGH"
    dependencies: [COMP-010, COMP-012]
    interfaces_provided: ["AuditEmitter", "BufferedForwarder", "AuditEntry construction"]
    interfaces_consumed: ["StorageBackend", "tracing subscriber"]

  - id: COMP-012
    name: "prism-core"
    layer: "shared"
    purity: "pure-core"
    criticality: "CRITICAL"
    dependencies: []
    interfaces_provided: ["TenantId", "PrismError", "ConfigSnapshot", "StorageDomain enum", "ColumnOptions", "entity types", "decorator types"]
    interfaces_consumed: []
```

## Crate Responsibilities

| Crate | Subsystems | BC Count | Key Exports |
|-------|-----------|----------|-------------|
| prism-core | (shared) | — | TenantId, PrismError, ConfigSnapshot, entity types, decorator types |
| prism-mcp | SS-10 | 10 | PrismServer, tool dispatch, resource/prompt handlers |
| prism-query | SS-11, SS-07 (partial) | 21 | QueryEngine, PrismQlParser, AliasResolver, UdfRegistry |
| prism-sensors | SS-01 | 9 | SensorAdapter, SensorAuth, AdapterRegistry |
| prism-spec-engine | SS-16 | 10 | SpecParser, PipelineExecutor, ConfigManager |
| prism-ocsf | SS-02 | 12 | OcsfNormalizer, DynamicMessage, FieldResolver |
| prism-operations | SS-12, SS-13, SS-14 | 33 | Scheduler, DiffEngine, DetectionEngine, AlertStore, CaseManager |
| prism-security | SS-04, SS-09 | 22 | FeatureFlagEvaluator, TokenStore, InjectionScanner |
| prism-credentials | SS-03 | 12 | CredentialStore, KeyringBackend, FileBackend |
| prism-storage | SS-15 (partial) | 11 | StorageBackend, RocksDbBackend, InMemoryBackend |
| prism-audit | SS-05 | 10 | AuditEmitter, BufferedForwarder |
| prism-bin | — | — | main(), CLI, signal handling, startup orchestration |
