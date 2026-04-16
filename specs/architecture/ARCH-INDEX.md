---
document_type: architecture-index
level: L3
version: "1.0"
status: draft
producer: architect
timestamp: 2026-04-15T12:00:00
phase: 1b
inputs: [domain-spec/L2-INDEX.md, prd.md, prd-supplements/interface-definitions.md, prd-supplements/nfr-catalog.md, prd-supplements/error-taxonomy.md]
traces_to: prd.md
deployment_topology: single-service
---

# Architecture Index: Prism

> **Context Engineering:** This is a lightweight index (~400 tokens). Agents load
> ONLY the section files they need, not the full architecture. See the Document Map
> for per-section consumer guidance.

## Document Map

| Section | File | Tokens | Primary Consumer | Purpose |
|---------|------|--------|-----------------|---------|
| System Overview | system-overview.md | ~1,000 | orchestrator, all agents | Architecture vision, principles, deployment model, constraints |
| Module Decomposition | module-decomposition.md | ~1,200 | story-writer, implementer | Crate catalog with boundaries, responsibilities, public APIs |
| Dependency Graph | dependency-graph.md | ~800 | story-writer, consistency-validator | Inter-crate dependencies, topological build order |
| API Surface | api-surface.md | ~1,000 | test-writer, implementer | MCP tool registry, error contract, resource/prompt surface |
| Data Layer | data-layer.md | ~1,000 | implementer, test-writer | RocksDB domains, Arrow materialization, caching strategy |
| Query Engine | query-engine.md | ~1,200 | implementer, test-writer | PrismQL parser, DataFusion integration, fan-out pipeline |
| Sensor Adapters | sensor-adapters.md | ~1,000 | implementer, test-writer | Config-driven TOML specs, CustomAdapter escape hatch |
| Security Architecture | security-architecture.md | ~1,000 | security-reviewer, implementer | Credentials, feature flags, audit, prompt injection defense |
| Operational Pipeline | operational-pipeline.md | ~1,000 | implementer, test-writer | Scheduler, differential results, detection, alerts, cases |
| Concurrency Architecture | concurrency-architecture.md | ~800 | implementer, formal-verifier | Tokio runtime, arc-swap, shared state protection |
| Purity Boundary Map | purity-boundary-map.md | ~800 | implementer, formal-verifier | Pure core / effectful shell classification per crate |
| Verification Architecture | verification-architecture.md | ~1,000 | formal-verifier, architect | Provable Properties Catalog, proof strategy |
| Tooling Selection | tooling-selection.md | ~400 | formal-verifier, devops-engineer | Kani, proptest, fuzz tool versions and config |
| Detection Rule Format | detection-rule-format.md | ~1,200 | implementer, test-writer | .detect rule structure, condition modes, rule-to-SQL compilation |
| Infusions | infusions.md | ~1,500 | implementer, test-writer | Enrichment framework — GeoIP, threat intel, asset inventory, CVSS. TOML specs + .prx plugins. |
| Actions | actions.md | ~1,500 | implementer, test-writer | Alert delivery + scheduled reports — Slack, PagerDuty, Jira, email, syslog. TOML specs + .prx plugins. |
| Installation | installation.md | ~1,500 | devops-engineer, implementer | Distribution channels, CLI commands, secops-factory integration, first-run UX |
| Config Schema | config-schema.md | ~1,500 | implementer, devops-engineer | Full prism.toml schema, aliases.toml, env var overrides, validation tiers, config diff tool |
| Observability | observability.md | ~2,000 | implementer, devops-engineer | 18 diagnostic log targets, per-subsystem levels, trace IDs, `prism logs` CLI, `get_diagnostics` tool, external log forwarding (Datadog/Splunk/Elastic/OTLP/plugin) |
| Verification Coverage | verification-coverage-matrix.md | ~600 | consistency-validator | VP-to-module coverage mapping |

## Cross-References

| If you need... | Read these together |
|----------------|-------------------|
| Implementation plan for a module | module-decomposition.md + dependency-graph.md + api-surface.md |
| Verification plan for a module | verification-architecture.md + purity-boundary-map.md + tooling-selection.md |
| Full module picture | module-decomposition.md + purity-boundary-map.md + verification-coverage-matrix.md |
| Story decomposition input | module-decomposition.md + dependency-graph.md |
| Query pipeline understanding | query-engine.md + sensor-adapters.md + data-layer.md |
| Security review | security-architecture.md + purity-boundary-map.md |
| Operational features | operational-pipeline.md + data-layer.md |

## Architecture Decisions

| ID | Decision | Rationale |
|----|----------|-----------|
| AD-001 | Modular monolith via Cargo workspace with 12 crates | Single binary deployment matches per-analyst stdio model; crate boundaries enforce module isolation without network overhead |
| AD-002 | DataFusion as SQL execution engine | Provides Arrow-native SQL with UDF extensibility; ephemeral SessionContext per query aligns with data-in-flight model |
| AD-003 | Chumsky 0.12 for PrismQL parsing | Zero-copy parser combinators with error recovery; axiathon reference proves pattern viability |
| AD-004 | RocksDB with 12 column families | Domain-isolated persistence for operational state; osquery-proven pattern; single-process LOCK fits stdio model |
| AD-005 | rmcp 1.4 as MCP SDK | Official Anthropic SDK; #[tool_router] macro for 35+ tool registration; native tokio async |
| AD-006 | Config-driven sensor adapters via TOML spec files | 80% of sensors need zero Rust code; eat-our-own-dog-food principle for built-in sensors |
| AD-007 | arc-swap for hot config reload | Lock-free reads on query hot path; atomic snapshot swap; in-flight queries unaffected |
| AD-008 | Pure core / effectful shell separation | Maximizes formal verification surface; domain logic testable without I/O mocking |
| AD-009 | Sealed trait pattern for SensorAuth | Prevents cross-sensor auth composition at compile time; type-level safety |
| AD-010 | TenantId newtype for client isolation | Compile-time enforcement of client data separation; prevents accidental cross-client leakage |
| AD-011 | Two-tier feature flag system (compile-time + runtime) | Compile-time gates remove code from binary; runtime gates enable per-client control; defense in depth |
| AD-012 | Bincode for RocksDB value serialization | Compact binary encoding; schema evolution via versioned keys; faster than JSON for structured data |
| AD-013 | tokio multi-threaded runtime | Required for concurrent sensor fan-out; DataFusion uses tokio internally; rmcp requires tokio |
| AD-014 | Process-level RSS watchdog with self-SIGTERM | Last-resort memory protection; graceful shutdown path preserves state integrity |
| AD-015 | DynamicMessage protobuf for OCSF normalization | Runtime-flexible field mapping without per-class codegen; axiathon-proven pattern |
| AD-016 | Write-audit ordering (intent-log pattern) | Durable audit trace for every write operation, even across crashes |
| AD-017 | AI-opaque credential management | Credential values never transit through AI context; reference-based model |
| AD-018 | Automatic filesystem watching for config reload | `notify` crate monitors config/spec/IOC/plugin directories; debounced 500ms; same validation as manual reload |
| AD-019 | WASM plugins for custom sensor adapters and infusions | Polyglot (Rust/Go/Python/JS/C#), sandboxed, hot-reloadable via `.prx` files. `wasmtime` runtime with WIT interface. Augments TOML specs, doesn't replace them. |
| AD-020 | Infusions — composable enrichment framework | GeoIP, threat intel, asset inventory, CVSS as TOML specs + `.prx` plugins. Register as DataFusion UDFs and `enrich` pipe stages. Same two-tier pattern as sensors. |
| AD-021 | Actions — config-driven alert delivery and reporting | Slack, PagerDuty, Jira, email, syslog, custom webhooks as TOML specs + `.prx` plugins. Three triggers: alert, schedule, manual. At-least-once delivery with retry. |
