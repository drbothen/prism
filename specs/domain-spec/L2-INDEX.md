---
document_type: domain-spec-index
level: L2
version: "1.3"
status: draft
producer: business-analyst
timestamp: 2026-04-14T04:00:00
phase: 1a
inputs: [product-brief.md, capabilities.md]
input-hash: "52f2391"
traces_to: L2-INDEX.md
sections:
  - architecture-concept.md
  - assumptions.md
  - capabilities.md
  - differentiators.md
  - edge-cases.md
  - entities.md
  - events.md
  - failure-modes.md
  - invariants.md
  - prismql-grammar.md
  - risks.md
  - scheduled-detection-concept.md
---

# L2 Domain Specification Index -- Prism

## Domain Summary

Prism is a Rust MCP server that unifies multi-client security sensor management for MSSP analysts. The query engine (CAP-015) is the sole data access interface -- all sensor data is accessed via PrismQL queries, not per-sensor read tools. The query engine orchestrates live API calls through an internal sensor adapter layer (CAP-001), normalizes all responses to OCSF, materializes ephemeral Arrow tables, and executes queries via DataFusion. Per-sensor MCP tools exist only for write operations (containment, acknowledgment, device actions), gated behind a two-tier feature flag system. The domain model encompasses the query engine with filter/SQL/pipe modes (Chumsky + DataFusion), an internal sensor adapter layer with per-client credential isolation and ephemeral cursor-based pagination (never exposed to MCP clients), response caching with configurable TTL, bounded confirmation tokens, a four-layer sanitization pipeline for attacker-controlled content, a composable query alias system, scheduled queries with differential results, PrismQL-based detection rules with alert generation and case management, query packs, RocksDB-backed persistent storage, a resource watchdog, buffered audit logging, context decorators, security domain UDFs, config-driven sensor adapters (TOML spec files with multi-step fetch pipelines, runtime-interpreted by the prism-spec-engine crate), and hot configuration reload (arc-swap lock-free config access with atomic swap). Phase 3 capabilities extend the platform with: a composable infusion enrichment framework (CAP-031, GeoIP/CSV/WASM-backed query-time enrichment), a sandboxed WASM plugin runtime (CAP-032, wasmtime-based .prx plugins for custom adapters and enrichment), a config-driven action delivery engine (CAP-033, alert-triggered and scheduled outbound notifications to Slack/PagerDuty/Jira/webhook), the MCP server and transport layer (CAP-034, stdio-transport rmcp server binding all capabilities to the Model Context Protocol), and diagnostic log forwarding (CAP-035, best-effort batched forwarding of Prism's own observability stream to Datadog/Splunk HEC/Elasticsearch/OTLP/syslog/webhook/plugin destinations). The MCP tool surface is approximately 33 tools.

## Document Map

| Section | File | Est. Tokens | Primary Consumer | Purpose |
|---------|------|-------------|-----------------|---------|
| Architecture Concept | architecture-concept.md | ~2500 | All consumers, New contributors | Explains the core architectural concept (ephemeral federated query engine), query flow, and comparisons with SIEM/Trino/direct API access |
| Scheduled Detection Concept | scheduled-detection-concept.md | ~3000 | All consumers, New contributors | Explains the unified data pipeline: scheduled queries as the detection engine's data source, three detection modes (single-event, correlation, sequence), pack bundling, RocksDB persistence domains, and cross-sensor detection via OCSF |
| Capabilities | capabilities.md | ~5800 | PRD Author, Architect | Enumerates all domain capabilities (CAP-001 through CAP-035, CAP-013 removed); CAP-001/002/011/012 are internal capabilities consumed by the query engine, not MCP-facing; CAP-017 through CAP-027 add scheduled queries, differential results, persistent storage, detection rules, alert generation, case management, query packs, resource watchdog, buffered audit logging, context decorators, and security domain UDFs; CAP-028 adds unified query surface; CAP-029 adds config-driven sensor adapters; CAP-030 adds hot configuration reload; CAP-031 adds infusion enrichment; CAP-032 adds WASM plugin runtime; CAP-033 adds action delivery engine; CAP-034 adds MCP server and transport; CAP-035 adds diagnostic log forwarding (SS-20, P1) |
| Entities | entities.md | ~5500 | Architect, Implementer | Defines 30 domain entities (QueryFingerprint removed; CacheEntry, QueryPlan, MaterializedTable, Alias, Schedule, DiffState, DetectionRule, Alert, Case, Pack, StorageDomain, InternalTable, SensorSpec, TableSpec, ColumnSpec, FetchStep, ConfigSnapshot added) with key attributes and invariants |
| Invariants | invariants.md | ~3500 | Architect, Test Writer | Specifies 25 domain rules (DI-001 through DI-031; DI-009, DI-010, DI-011, DI-013 removed) that must always hold with violation behavior |
| Events | events.md | ~1100 | Architect, Implementer | Documents 10 processing stages from tool invocation through audit emission |
| Edge Cases | edge-cases.md | ~4200 | Test Writer, Implementer | Specifies expected behavior for 37 boundary scenarios (DEC-001 through DEC-039; DEC-012 removed) |
| Assumptions | assumptions.md | ~1000 | Product Owner, Architect | Lists 10 assumptions (ASM-001 through ASM-010) requiring validation with impact analysis |
| Risks | risks.md | ~1100 | Product Owner, Architect | Risk register with 12 entries (R-001 through R-012) including mitigations |
| Failure Modes | failure-modes.md | ~1100 | Implementer, SRE | Documents 12 runtime failure modes (FM-001 through FM-012) with detection and recovery |
| Differentiators | differentiators.md | ~1000 | Product Owner, Stakeholders | Maps 8 competitive differentiators to supporting capabilities |
| Index | L2-INDEX.md | ~500 | All consumers | Navigation, cross-references, and ID registry |


## Cross-References

| Source ID | Target IDs | Relationship |
|-----------|-----------|-------------|
| CAP-001 | DI-001, DI-008, DI-021, DEC-001, DEC-010, DEC-013, DEC-027, FM-001, FM-006, R-003 | Sensor adapter layer (internal) constrained by cursor invariant, client separation, required column enforcement; edge cases for failures and future streaming API support; risks from API changes. Consumed by query engine (CAP-015), not exposed as MCP tools. |
| CAP-002 | DEC-003, DEC-005, DEC-020, DI-008, R-007 | Cross-client fan-out (internal to query engine) edge cases for partial failures, missing sensors, internal cursor cap pressure; data mixing risk |
| CAP-003 | DI-005, DEC-007, DEC-015, ASM-002, ASM-005, ASM-010, R-004, FM-005 | OCSF normalization constrained by schema validity; edge cases for unmappable fields; risks from schema instability |
| CAP-004 | DI-002, DI-014, DEC-011, ASM-003, R-006, FM-004 | Credential management constrained by isolation and sanitization invariants; keyring availability edge case |
| CAP-005 | DI-003, DEC-006, ASM-001, R-001 | Feature flags constrained by deny-by-default with deny override; edge case for config change during session |
| CAP-006 | DI-007, DI-015, DEC-009, DEC-016, DEC-017, DEC-019, DEC-021, R-012, FM-007 | Write gating constrained by token expiry and token cap; edge cases for expired/lost/capped tokens, concurrency, and combined cursor+token pressure; replay risk |
| CAP-007 | DI-004, DI-016, DEC-014, R-005 | Audit logging constrained by completeness invariant (split: reads proceed, writes fail-closed); edge case for logging failure |
| CAP-010 | DI-006, DEC-008, R-005 | Prompt injection defense constrained by sanitization invariant; edge case for hostile hostnames |
| CAP-008 | DEC-001, FM-001, FM-002, FM-010, R-003, R-010 | Sensor health monitoring detects unreachable sensors, expired auth, rate limiting; risks from API changes |
| CAP-009 | DI-002, DI-003, DI-008, DEC-004, DEC-006, FM-010, R-007 | Client configuration constrained by credential isolation, deny-by-default flags, client separation; edge cases for zero-sensor and config changes |
| CAP-011 | DI-001, DI-017, DEC-020, DEC-021, R-008 | Internal adapter pagination constrained by pagination validity and single-process invariant; edge cases for internal cursor cap pressure and combined cursor+token pressure. Cursors never exposed to MCP clients. |
| CAP-012 | CAP-003, CAP-015, DI-005, DEC-003, DEC-005, ASM-002 | Cross-sensor correlation is a natural consequence of the query engine's unified OCSF table; depends on OCSF normalization; edge cases for cross-client partial failures and mixed sensor availability |
| ~~CAP-013~~ | — | **REMOVED** — xMP backward compatibility not required |
| CAP-014 | DI-018, DEC-018, DEC-019 | Response caching constrained by cache bounds; edge cases for stale data after writes and concurrent access |
| CAP-015 | DI-019, DI-021, DEC-022, DEC-023, DEC-026, CAP-003, CAP-014 | Ephemeral OCSF query engine constrained by security limits and required column enforcement; edge cases for empty results, scope too broad, and timeout; depends on OCSF normalization and response cache for sensor fetch layer |
| CAP-016 | DI-020, DEC-024, DEC-025, CAP-015, BC-2.11.008, BC-2.11.009, BC-2.11.013, BC-2.11.014, BC-2.11.015 | Query aliases constrained by composition depth and cycle detection; edge cases for undefined alias references and cross-client alias gaps; aliases feed into the query engine; MCP tools for create, list, delete, and explain alias operations |
| CAP-017 | DI-022, DI-023, DEC-028, CAP-015, CAP-018, CAP-019, CAP-023 | Scheduled queries constrained by splay distribution and exactly-once semantics; edge case for overlapping executions; depends on query engine, differential results, persistent storage; consumed by packs |
| CAP-018 | DI-023, DEC-029, CAP-017, CAP-019, CAP-020 | Differential results constrained by exactly-once semantics; edge case for large diffs; depends on scheduled queries and persistent storage; feeds detection rules |
| CAP-019 | DI-026, DEC-032, CAP-017, CAP-018, CAP-020, CAP-021, CAP-022, CAP-024, CAP-025 | Persistent storage (RocksDB) constrained by audit buffer durability; edge case for write failures; consumed by all stateful capabilities |
| CAP-020 | DI-024, DEC-030, CAP-015, CAP-018, CAP-019, CAP-021, CAP-023 | Detection rules constrained by load-time validation; edge case for partial correlation matches; depends on query engine, differential results, persistent storage; generates alerts; consumed by packs |
| CAP-021 | CAP-019, CAP-020, CAP-022 | Alert generation depends on detection rules and persistent storage; alerts feed into case management |
| CAP-022 | DI-025, DEC-031, CAP-019, CAP-021 | Case management constrained by state transition validity; edge case for post-resolution disposition; depends on persistent storage and alerts |
| CAP-023 | DEC-034, CAP-016, CAP-017, CAP-020 | Query packs edge case for discovery query failure; bundles aliases, scheduled queries, and detection rules |
| CAP-024 | DI-027, DEC-033, CAP-015, CAP-019 | Resource watchdog constrained by enforcement invariant; edge case for mid-execution kill; depends on query engine and persistent storage (crash recovery, denylist) |
| CAP-025 | DI-026, CAP-007, CAP-019 | Buffered audit logging constrained by durability invariant; extends base audit logging; depends on persistent storage |
| CAP-026 | CAP-003, CAP-015, CAP-017 | Context decorators depend on OCSF normalization, query engine, and scheduled queries for injection phases |
| CAP-027 | CAP-015 | Security domain UDFs registered in query engine's DataFusion SessionContext |
| CAP-028 | CAP-015, CAP-019, DI-004, DI-008 | Unified query surface registers both external sensor tables (API-backed) and internal Prism tables (RocksDB-backed) as DataFusion tables; depends on query engine and persistent storage; constrained by audit completeness and client data separation |
| CAP-029 | DI-030, DEC-036, DEC-038, CAP-001, CAP-003, CAP-004, CAP-015, R-003 | Config-driven sensor adapters: TOML spec files define sensor tables, columns with OCSF mappings, multi-step fetch pipelines; constrained by spec validation invariant; edge cases for missing credentials and undefined variables; depends on sensor adapter layer, OCSF normalization, credential management, and query engine |
| CAP-030 | DI-031, DEC-037, DEC-039, CAP-029, CAP-009, CAP-016 | Hot configuration reload: arc-swap lock-free config access, hash-based change detection, atomic swap; constrained by reload atomicity invariant; edge cases for in-flight queries and credential changes during execution; depends on config-driven adapters, client configuration, and query aliases |
| CAP-031 | CAP-015, CAP-029, CAP-032, CAP-004 | Infusion enrichment framework: TOML-defined infusion specs register DataFusion scalar UDFs backed by MaxMind MMDB, CSV/JSON lookup tables, or WASM plugin sources; per-query dedup cache; plugin-backed infusions prohibited in detection rule filter expressions; AI-opaque credential references; MCP tools: list_infusions, infusion_status, reload_infusion |
| CAP-032 | CAP-029, CAP-031, CAP-033, CAP-019 | WASM plugin runtime: wasmtime-based sandboxed execution of .prx plugin files; capability-gated host function model; WIT interface validation at load time; fuel-metered CPU and memory-bounded linear memory per invocation; plugin panics isolated from host process; hot reload via atomic swap; MCP tools: list_plugins, plugin_status, reload_plugin |
| CAP-033 | CAP-015, CAP-020, CAP-021, CAP-004, CAP-010, CAP-019, CAP-030, CAP-032 | Action delivery engine: TOML-defined action specs dispatch outbound notifications (Slack, PagerDuty, Jira, email, syslog, webhook, plugin) on alert trigger, schedule, or manual invocation; at-least-once delivery with RocksDB-backed retry; rate limiting and quarantine per destination; injection-scanned template interpolation; AI-opaque credential references; MCP tools: fire_action, list_actions, action_status, create_action, delete_action, test_action |
| CAP-034 | CAP-005, CAP-007, CAP-009 | MCP server and transport layer: stdio-transport rmcp 1.4 server binding all Prism capabilities to the Model Context Protocol; single-process stdio session per analyst; tool registration via #[tool_router] macro with conditional write-tool hiding; structured error responses; MCP prompts for analyst workflows; subscribable MCP resources; graceful shutdown with in-flight drain |
| CAP-035 | CAP-007, CAP-025, CAP-032, BC-2.20.001, BC-2.20.002, BC-2.20.003, BC-2.20.004, BC-2.20.005 | Diagnostic log forwarding (SS-20): best-effort batched forwarding of Prism's own observability log stream to external sinks (Datadog, Splunk HEC, Elasticsearch, OTLP, syslog, webhook, WASM plugin). Distinct from CAP-025 (audit log, at-least-once, RocksDB-durable) — diagnostic forwarding is best-effort and may drop entries on overflow. Constrained by no-recursive-forwarding rule (BC-2.20.001): log entries emitted during forward-batch() route to local sink only. WASM plugin-backed destinations use CAP-032 sandbox. |

## ID Registry Summary

| ID Format | Range | Count | Section |
|-----------|-------|-------|---------|
| CAP-NNN | CAP-001 to CAP-035 (CAP-013 removed) | 34 | capabilities.md |
| DI-NNN | DI-001 to DI-031 (DI-009, DI-010, DI-011, DI-013 removed) | 25 | invariants.md |
| DEC-NNN | DEC-001 to DEC-039 (DEC-012 removed) | 37 | edge-cases.md |
| ASM-NNN | ASM-001 to ASM-010 | 10 | assumptions.md |
| R-NNN | R-001 to R-012 | 12 | risks.md |
| FM-NNN | FM-001 to FM-012 | 12 | failure-modes.md |
| **Total** | | **130** | |

## Priority Distribution

| Priority | Capabilities | Description |
|----------|-------------|-------------|
| P0 | CAP-001, CAP-002, CAP-003, CAP-004, CAP-005, CAP-007, CAP-009, CAP-010, CAP-011, CAP-015, CAP-017, CAP-018, CAP-019, CAP-020, CAP-021, CAP-022, CAP-023, CAP-024, CAP-025, CAP-026, CAP-027, CAP-028, CAP-029, CAP-031, CAP-034 | Query engine (sole data access), internal sensor adapter layer, internal adapter pagination, OCSF normalization, credential management, feature flags, audit, config, prompt injection defense, scheduled queries, differential results, persistent storage, detection rules, alert generation, case management, query packs, resource watchdog, buffered audit logging, context decorators, security domain UDFs, unified query surface (external + internal tables), config-driven sensor adapters, infusion enrichment, MCP server and transport -- required for MVP |
| P1 | CAP-006, CAP-008, CAP-012, CAP-014, CAP-016, CAP-030, CAP-035 | Write operation gating, sensor health, cross-sensor correlation (via query engine), response caching, query aliases, hot configuration reload, diagnostic log forwarding -- required for full launch |
| P2 | CAP-032, CAP-033 | WASM plugin runtime, action delivery engine (Phase 3, post-launch platform extension capabilities) |

**P0 count:** 25 capabilities (74%)
**P1 count:** 7 capabilities (21%)
**P2 count:** 2 capabilities (6%)

## Changelog

| Version | Date | Change |
|---------|------|--------|
| 1.3 | 2026-04-21 | Pass-80 F80-002 follow-on: registered CAP-035 (Diagnostic Log Forwarding, SS-20, P1). ID Registry updated CAP-001..034 → CAP-001..035, count 33 → 34, total IDs 129 → 130. Cross-References row added for CAP-035. Priority Distribution: CAP-035 added to P1 bucket; P1 count 6 → 7. Domain Summary narrative extended. |
| 1.2 | 2026-04-21 | Remediation for pass-80 F80-001: added CAP-031/032/033/034 to Cross-References table, ID Registry (CAP-001 to CAP-034, 33 total), and Priority Distribution (CAP-031/034 to P0; CAP-032/033 to P2). Extended Domain Summary narrative to name Phase 3 capabilities. Updated inputs frontmatter to include capabilities.md. |
| 1.1 | 2026-04-14 | Initial index with CAP-001 through CAP-030 (CAP-013 removed). |
