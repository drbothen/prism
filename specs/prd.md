---
document_type: prd
level: L3
version: "1.2"
status: draft
producer: product-owner
timestamp: 2026-04-14T06:00:00
phase: 1a
inputs: [domain-spec/L2-INDEX.md, product-brief.md]
input-hash: "8f9fa70"
traces_to: domain-spec/L2-INDEX.md
supplements: [prd-supplements/interface-definitions.md, prd-supplements/error-taxonomy.md, prd-supplements/nfr-catalog.md, prd-supplements/test-vectors.md]
---

# Product Requirements Document: Prism

## 1. Product Overview

### Problem

MSSP (Managed Security Service Provider) analysts manage dozens of clients, each with multiple security sensors -- CrowdStrike Falcon, Cyberint Argos, Claroty xDome, and Armis Centrix. Today, every client requires separate dashboard logins per sensor. An analyst investigating a multi-sensor incident for a single client must context-switch between 4+ web consoles, manually copy data between browser tabs, and mentally correlate results across incompatible data formats. Cross-client visibility ("which clients have unacknowledged critical alerts?") requires surveying every client dashboard individually. There is no programmatic access for automation, no unified query language, and no way to correlate a CrowdStrike endpoint detection with a Claroty OT event for the same IP address.

### Solution

Prism is a Rust-based MCP server that gives analysts a unified, AI-powered interface to every security sensor across every client from a single Claude Code session. Analysts interact through natural language -- "show me all critical alerts for Acme from the last 24 hours across all sensors" -- while Prism handles authentication, pagination, data normalization, and cross-sensor correlation behind the scenes. All sensor data is normalized to OCSF v1.x via protobuf, making cross-sensor joins possible for the first time. Write operations (host containment, alert acknowledgment) are gated behind a defense-in-depth feature flag system with dry-run defaults and confirmation tokens.

### Competitive Differentiators

1. **AI-Native Interface** -- Consumed exclusively by AI agent harnesses, not dashboards. Every design decision optimizes for LLM consumption.
2. **Cross-Sensor Correlation via OCSF** -- Normalizes all sensor data to a common schema, enabling cross-sensor joins by IP, hostname, and time.
3. **Multi-Client Single Session** -- Explicit `client_id` on every tool call with cross-client query support (`client_id: null`).
4. **Feature-Flagged Write Operations** -- Two-tier gate (compile-time + runtime) with three-tier risk classification and confirmation tokens.
5. **OCSF with Vendor Extension Preservation** -- Normalized view for correlation plus `raw_extensions` for vendor-specific deep dives.
6. **Prompt Injection Defense** -- Four-layer sanitization purpose-built for security data where tool responses contain attacker-controlled content.
7. **Unified Sensor Adapter Architecture** -- Trait-based `DataSource<T>` eliminates the code duplication found in the 4 reference Go pollers.
8. **SOC 2 / ISO 27001 Audit Trail** -- Every MCP invocation logged with compliance-grade structured fields.

### Target Users

| Persona | Role |
|---------|------|
| SOC Analyst | Primary. Daily triage across clients and sensors. |
| Threat Hunter | Primary. Cross-client, cross-sensor indicator searches. |
| Security Engineer | Primary. Programmatic access, automation, client onboarding. |
| Client-Facing Staff | Secondary. Security posture reports generated via AI. |
| Management | Secondary. Cross-client operational visibility. |

### Out of Scope

- Web UI or dashboard (Prism is an MCP server, not a web application)
- SIEM/log storage (Prism queries sensors in real-time; the Vector pipeline handles log aggregation)
- Automated remediation without human direction
- Sensor deployment or agent installation
- ~~Custom detection rule authoring~~ (now in scope: subsystem 13, detection engine with .detect DSL)

---

## 2. Behavioral Contracts Index

200 active behavioral contracts (208 total, 6 removed, 2 retired) organized across 20 subsystems. Each BC specifies a single testable behavior with preconditions, postconditions, invariants, and error cases. Individual BC files are located in `behavioral-contracts/`.

**Phase 3-patch (2026-04-16):** Added 26 BCs total — 22 in Burst 1 closing traceability gaps for AD-019 (WASM Plugin Runtime, subsystem 17), AD-020 (Infusion Enrichment Framework, subsystem 19), AD-021 (Action Delivery Engine, subsystem 18), CAP-022 auto-case-creation, and completing the BC-2.14.012 stub. 4 additional in Burst 2.5: BC-2.08.008/009 (get_diagnostics tool + diagnostic resources, S-5.08), BC-2.05.011 (audit forwarding at-least-once, S-5.10, proposes VP-039), BC-2.13.014 (IOC file loading, S-4.03).

### Subsystem 01: Sensor Adapters (9 BCs)

Capabilities: CAP-001, CAP-002

This subsystem is the sensor adapter layer that the query engine (subsystem 11) calls. Per-sensor MCP read tools have been removed -- all data access goes through the `query` tool (CAP-015). This subsystem provides: per-sensor authentication, pagination, retry/backoff, and partial failure handling as internal adapter behaviors.

| BC ID | Title | Priority |
|-------|-------|----------|
| [BC-2.01.002](behavioral-contracts/BC-2.01.002-cross-client-fan-out.md) | Cross-Client Fan-Out — Query Engine Orchestrates Parallel Sensor Fetches | P0 |
| [BC-2.01.004](behavioral-contracts/BC-2.01.004-offset-based-pagination-claroty.md) | Offset-Based Hybrid Pagination for Claroty Audit Logs | P0 |
| [BC-2.01.005](behavioral-contracts/BC-2.01.005-crowdstrike-oauth2-two-step-fetch.md) | CrowdStrike OAuth2 Authentication and Two-Step Fetch | P0 |
| [BC-2.01.006](behavioral-contracts/BC-2.01.006-cyberint-cookie-auth.md) | Cyberint Cookie-Based Authentication and Multi-Format Timestamp Parsing | P0 |
| [BC-2.01.007](behavioral-contracts/BC-2.01.007-claroty-bearer-polymorphic-ids.md) | Claroty Bearer Token Auth with Polymorphic ID Handling | P0 |
| [BC-2.01.008](behavioral-contracts/BC-2.01.008-armis-bearer-aql.md) | Armis Bearer Token Auth with AQL Query Forwarding and Timestamp Fallback | P0 |
| [BC-2.01.010](behavioral-contracts/BC-2.01.010-partial-failure-handling.md) | Partial Failure Handling for Paginated and Cross-Client Queries | P0 |
| [BC-2.01.013](behavioral-contracts/BC-2.01.013-datasource-trait-adapter-pattern.md) | DataSource Trait Eliminates Per-Sensor Code Duplication | P0 |
| [BC-2.01.014](behavioral-contracts/BC-2.01.014-sensor-api-http-503-mid-pagination.md) | Exponential Backoff and Retry for Transient Sensor API Errors | P0 |

### Subsystem 02: OCSF Normalization (12 BCs)

Capability: CAP-003

| BC ID | Title | Priority |
|-------|-------|----------|
| [BC-2.02.001](behavioral-contracts/BC-2.02.001-ocsf-schema-build-time-loading.md) | OCSF Schema Loading at Build Time via ocsf-proto-gen | P0 |
| [BC-2.02.002](behavioral-contracts/BC-2.02.002-dynamic-message-creation.md) | DynamicMessage Creation from Sensor Records | P0 |
| [BC-2.02.003](behavioral-contracts/BC-2.02.003-crowdstrike-field-mapping.md) | CrowdStrike Alert Field Mapping to OCSF | P0 |
| [BC-2.02.004](behavioral-contracts/BC-2.02.004-cyberint-field-mapping.md) | Cyberint Alert Field Mapping to OCSF | P0 |
| [BC-2.02.005](behavioral-contracts/BC-2.02.005-claroty-field-mapping.md) | Claroty xDome Field Mapping to OCSF (9 Data Sources) | P0 |
| [BC-2.02.006](behavioral-contracts/BC-2.02.006-armis-field-mapping.md) | Armis Centrix Field Mapping to OCSF (7 Data Sources) | P0 |
| [BC-2.02.007](behavioral-contracts/BC-2.02.007-raw-extensions-preservation.md) | Vendor Extension Preservation in raw_extensions | P0 |
| [BC-2.02.008](behavioral-contracts/BC-2.02.008-field-alias-resolution.md) | Four-Tier Field Alias Resolution | P0 |
| [BC-2.02.009](behavioral-contracts/BC-2.02.009-ocsf-version-pinning.md) | OCSF Version Pinning Per Release | P0 |
| [BC-2.02.010](behavioral-contracts/BC-2.02.010-enum-value-map-runtime-lookup.md) | OCSF Enum Value Map for Runtime Display Names | P0 |
| [BC-2.02.011](behavioral-contracts/BC-2.02.011-normalization-error-handling.md) | Graceful Normalization Error Handling (No Silent Data Loss) | P0 |
| [BC-2.02.012](behavioral-contracts/BC-2.02.012-ocsf-event-class-selection.md) | OCSF Event Class Selection Per Sensor Record Type | P0 |

### Subsystem 03: Credential Management (12 BCs)

Capability: CAP-004

| BC ID | Title | Priority |
|-------|-------|----------|
| [BC-2.03.001](behavioral-contracts/BC-2.03.001-credential-store-trait.md) | CredentialStore Trait with Tenant-Scoped Operations | P0 |
| [BC-2.03.002](behavioral-contracts/BC-2.03.002-keyring-backend.md) | OS Keyring Backend via keyring-rs | P0 |
| [BC-2.03.003](behavioral-contracts/BC-2.03.003-encrypted-file-fallback.md) | AES-256-GCM Encrypted File Backend Fallback | P0 |
| [BC-2.03.004](behavioral-contracts/BC-2.03.004-namespace-isolation.md) | Credential Namespace Isolation by (client_id, sensor_id, credential_name) | P0 |
| [BC-2.03.005](behavioral-contracts/BC-2.03.005-credential-crud-operations.md) | Credential CRUD Operations via MCP Tools (Mutations Require Confirmation Token) | P0 |
| [BC-2.03.006](behavioral-contracts/BC-2.03.006-credential-resolution-at-query-time.md) | Credential Resolution at Sensor Query Time | P0 |
| [BC-2.03.007](behavioral-contracts/BC-2.03.007-secret-redaction.md) | Secret Redaction in Logs, Errors, and MCP Responses | P0 |
| [BC-2.03.008](behavioral-contracts/BC-2.03.008-credential-name-sanitization.md) | Credential Name Sanitization Against Path Traversal | P0 |
| [BC-2.03.009](behavioral-contracts/BC-2.03.009-resolve-secret-env-file.md) | resolve_secret() for _FILE Env Var and K8s Secret Mount Compatibility | P0 |
| [BC-2.03.010](behavioral-contracts/BC-2.03.010-credential-access-audit-logging.md) | Credential Access Audit Logging | P0 |
| [BC-2.03.011](behavioral-contracts/BC-2.03.011-keyring-startup-probe.md) | Keyring Startup Probe for Permission Pre-Authorization | P0 |
| [BC-2.03.012](behavioral-contracts/BC-2.03.012-backend-selection-fallback.md) | Credential Backend Selection and Fallback | P0 |

### Subsystem 04: Feature Flags (15 BCs)

Capabilities: CAP-005, CAP-006

| BC ID | Title | Priority |
|-------|-------|----------|
| [BC-2.04.001](behavioral-contracts/BC-2.04.001-compile-time-cargo-features.md) | Compile-Time Cargo Features Gate Write Code Families | P0 |
| [BC-2.04.002](behavioral-contracts/BC-2.04.002-runtime-per-client-toml-flags.md) | Runtime Per-Client TOML Feature Flag Configuration | P0 |
| [BC-2.04.003](behavioral-contracts/BC-2.04.003-hierarchical-flag-resolution.md) | Hierarchical Capability Resolution (BTreeMap, Most-Specific-Path Wins, Deny Support) | P0 |
| [BC-2.04.004](behavioral-contracts/BC-2.04.004-two-tier-gate-both-must-pass.md) | Two-Tier Gate -- Both Compile-Time and Runtime Must Permit Operation | P0 |
| [BC-2.04.005](behavioral-contracts/BC-2.04.005-hidden-tools-pattern.md) | Hidden Tools Pattern -- Stateless Tool List Based on Configured Capabilities | P0 |
| [BC-2.04.006](behavioral-contracts/BC-2.04.006-list-capabilities-meta-tool.md) | list_capabilities Meta-Tool for Capability Discovery | P0 |
| [BC-2.04.007](behavioral-contracts/BC-2.04.007-three-tier-risk-classification.md) | Three-Tier Risk Classification for Operations | P1 |
| [BC-2.04.008](behavioral-contracts/BC-2.04.008-dry-run-default-reversible-writes.md) | Dry-Run Default for Reversible Write Operations | P1 |
| [BC-2.04.009](behavioral-contracts/BC-2.04.009-confirmation-token-request.md) | Confirmation Token Generation for Irreversible Write Operations (100-Token Active Cap) | P1 |
| [BC-2.04.010](behavioral-contracts/BC-2.04.010-confirmation-token-consumption.md) | Confirmation Token Consumption via confirm_action | P1 |
| [BC-2.04.011](behavioral-contracts/BC-2.04.011-token-expiry-300s.md) | Token Expiry at 300 Seconds with Structured Error Recovery | P1 |
| [BC-2.04.012](behavioral-contracts/BC-2.04.012-token-content-hash-verification.md) | Token Content Hash Verification Prevents Action Tampering | P1 |
| [BC-2.04.013](behavioral-contracts/BC-2.04.013-capability-check-audit-logging.md) | Feature Flag Evaluation Audit Logging for Write Operations | P0 |
| [BC-2.04.014](behavioral-contracts/BC-2.04.014-tools-list-changed-notification.md) | notifications/tools/list_changed on Config Reload or Server Startup | P0 |
| [BC-2.04.015](behavioral-contracts/BC-2.04.015-write-denied-structured-error.md) | Structured Error When Write Capability Is Denied | P0 |

### Subsystem 05: Audit Trail (11 BCs)

Capability: CAP-007

| BC ID | Title | Priority |
|-------|-------|----------|
| [BC-2.05.001](behavioral-contracts/BC-2.05.001-audit-entry-per-tool-invocation.md) | Every MCP Tool Invocation Produces Exactly One Audit Entry (Fail-Closed for Writes) | P0 |
| [BC-2.05.002](behavioral-contracts/BC-2.05.002-audit-entry-structured-json-format.md) | Audit Entries Use Structured JSON Format with Complete Fields | P0 |
| [BC-2.05.003](behavioral-contracts/BC-2.05.003-secret-redaction-in-audit-entries.md) | Credential Values Are Never Present in Audit Entries | P0 |
| [BC-2.05.004](behavioral-contracts/BC-2.05.004-write-operation-audit-detail.md) | Write Operations Log Capability Check and Execution Outcome | P0 |
| [BC-2.05.005](behavioral-contracts/BC-2.05.005-credential-access-audit.md) | Credential Access Events Are Audit-Logged with Context | P0 |
| [BC-2.05.006](behavioral-contracts/BC-2.05.006-audit-entry-immutability.md) | Audit Entries Are Append-Only and Immutable | P0 |
| [BC-2.05.007](behavioral-contracts/BC-2.05.007-vector-pipeline-compatibility.md) | Audit Entries Are Compatible with the Vector Pipeline | P0 |
| [BC-2.05.008](behavioral-contracts/BC-2.05.008-soc2-iso27001-field-requirements.md) | Audit Entries Satisfy SOC 2 Type II and ISO 27001 Requirements | P0 |
| [BC-2.05.009](behavioral-contracts/BC-2.05.009-feature-flag-evaluation-audit.md) | Feature Flag Evaluations for Write Operations Are Audit-Logged | P0 |
| [BC-2.05.010](behavioral-contracts/BC-2.05.010-confirmation-token-audit.md) | Confirmation Token Lifecycle Events Are Audit-Logged | P0 |
| [BC-2.05.011](behavioral-contracts/BC-2.05.011-audit-forwarding-at-least-once.md) | Audit Forwarding — At-Least-Once Delivery to External Destinations (VP-039 monotonic watermark) | P0 |

### Subsystem 06: Client Configuration (10 BCs)

Capability: CAP-009

| BC ID | Title | Priority |
|-------|-------|----------|
| [BC-2.06.001](behavioral-contracts/BC-2.06.001-toml-config-loading.md) | TOML Configuration Loads and Deserializes at Startup | P0 |
| [BC-2.06.002](behavioral-contracts/BC-2.06.002-per-client-sensor-mapping.md) | Per-Client Sensor Mapping from TOML Configuration | P0 |
| [BC-2.06.003](behavioral-contracts/BC-2.06.003-credential-reference-resolution.md) | Credential References in Config Resolve to Credential Store Entries | P0 |
| [BC-2.06.004](behavioral-contracts/BC-2.06.004-capability-override-resolution.md) | Capability Overrides Merge with Defaults Using More-Specific-Wins | P0 |
| [BC-2.06.005](behavioral-contracts/BC-2.06.005-config-validation-multi-error.md) | Configuration Validation Reports All Errors in One Pass | P0 |
| [BC-2.06.006](behavioral-contracts/BC-2.06.006-dry-run-validation-mode.md) | --dry-run Flag Validates Config and Prints Redacted Summary | P0 |
| [BC-2.06.007](behavioral-contracts/BC-2.06.007-missing-required-field-errors.md) | Missing Required Fields Produce Actionable Error Messages | P0 |
| [BC-2.06.008](behavioral-contracts/BC-2.06.008-default-values-and-env-var-override.md) | Default Values Apply and Environment Variables Override TOML | P0 |
| [BC-2.06.009](behavioral-contracts/BC-2.06.009-tools-list-changed-on-client-switch.md) | Config Reload Triggers notifications/tools/list_changed | P0 |
| [BC-2.06.010](behavioral-contracts/BC-2.06.010-client-id-validation.md) | Client ID Validation Enforces Allowed Character Set | P0 |

### Subsystem 07: Adapter Pagination & Response Cache (6 BCs)

Capabilities: CAP-011, CAP-014

| BC ID | Title | Priority |
|-------|-------|----------|
| [BC-2.07.001](behavioral-contracts/BC-2.07.001-ephemeral-cursor-pagination.md) | Internal Ephemeral Pagination Token Structure | P0 |
| [BC-2.07.002](behavioral-contracts/BC-2.07.002-pagination-token-lifecycle.md) | Internal Pagination Token Lifecycle — Forward Progress, Timeout, and Cleanup | P0 |
| [BC-2.07.003](behavioral-contracts/BC-2.07.003-response-cache-ttl.md) | Query Engine Sensor-Fetch Cache with Configurable TTL | P1 |
| [BC-2.07.004](behavioral-contracts/BC-2.07.004-cache-invalidation-on-writes.md) | Cache Invalidation on Write Operations | P1 |
| [BC-2.07.005](behavioral-contracts/BC-2.07.005-cache-key-derivation.md) | Cache Key Derivation from Push-Down Parameters | P1 |
| [BC-2.07.006](behavioral-contracts/BC-2.07.006-cache-memory-bounds-eviction.md) | Cache Memory Bounds and Eviction Policy | P1 |

### Subsystem 08: Sensor Health (9 BCs)

Capability: CAP-008

| BC ID | Title | Priority |
|-------|-------|----------|
| [BC-2.08.001](behavioral-contracts/BC-2.08.001-on-demand-connectivity-check.md) | On-Demand Connectivity Check Per Sensor Per Client | P1 |
| [BC-2.08.002](behavioral-contracts/BC-2.08.002-auth-validity-check.md) | Auth Validity Check Per Sensor Per Client | P1 |
| [BC-2.08.003](behavioral-contracts/BC-2.08.003-rate-limit-state-detection.md) | Rate Limit State Detection Per Sensor | P1 |
| [BC-2.08.004](behavioral-contracts/BC-2.08.004-last-successful-query-timestamp.md) | Last Successful Query Timestamp Per Sensor Per Client | P1 |
| [BC-2.08.005](behavioral-contracts/BC-2.08.005-health-mcp-tool.md) | Health Check MCP Tool | P1 |
| [BC-2.08.006](behavioral-contracts/BC-2.08.006-health-mcp-resource.md) | Health Status MCP Resource | P1 |
| [BC-2.08.007](behavioral-contracts/BC-2.08.007-partial-health-status.md) | Partial Health Status (Mixed Sensor Availability) | P1 |
| [BC-2.08.008](behavioral-contracts/BC-2.08.008-get-diagnostics-tool.md) | `get_diagnostics` MCP Tool — Subsystem Diagnostic Query with Injection Defense | P1 |
| [BC-2.08.009](behavioral-contracts/BC-2.08.009-diagnostic-resource-templates.md) | Diagnostic Resource Templates — `prism://diagnostics/*` MCP Resources | P1 |

### Subsystem 09: Prompt Injection Defense (8 BCs)

Capability: CAP-010

| BC ID | Title | Priority |
|-------|-------|----------|
| [BC-2.09.001](behavioral-contracts/BC-2.09.001-structural-separation.md) | Structural Separation of Untrusted Data | P0 |
| [BC-2.09.002](behavioral-contracts/BC-2.09.002-provenance-framing.md) | Provenance Framing in Tool Descriptions | P0 |
| [BC-2.09.003](behavioral-contracts/BC-2.09.003-suspicious-pattern-detection.md) | Suspicious Pattern Detection via Regex with NFKC Normalization | P0 |
| [BC-2.09.004](behavioral-contracts/BC-2.09.004-safety-flag-parallel-fields.md) | Safety Flags via _meta.safety_flags Array (Centralized, Not Per-Field) | P0 |
| [BC-2.09.005](behavioral-contracts/BC-2.09.005-trust-level-metadata.md) | Trust-Level Metadata Per Response | P0 |
| [BC-2.09.006](behavioral-contracts/BC-2.09.006-tool-description-security-warnings.md) | Tool Description Security Warnings | P0 |
| [BC-2.09.007](behavioral-contracts/BC-2.09.007-output-schema-type-safety.md) | OutputSchema for Type-Safe LLM Reasoning | P0 |
| [BC-2.09.008](behavioral-contracts/BC-2.09.008-response-envelope-trust-annotations.md) | Response Envelope with Trust Annotations | P0 |

### Subsystem 10: MCP Interface (11 BCs)

Capabilities: CAP-034, CAP-005, CAP-009

| BC ID | Title | Priority |
|-------|-------|----------|
| [BC-2.10.001](behavioral-contracts/BC-2.10.001-server-handler-implementation.md) | rmcp ServerHandler Implementation | P0 |
| [BC-2.10.002](behavioral-contracts/BC-2.10.002-tool-registration-via-tool-router.md) | Tool Registration via #[tool_router] | P0 |
| [BC-2.10.003](behavioral-contracts/BC-2.10.003-conditional-tool-registration.md) | Conditional Tool Registration (Feature-Flag Gated) | P0 |
| [BC-2.10.004](behavioral-contracts/BC-2.10.004-client-id-parameter-requirement.md) | Client Scoping on Every Tool (Stateless Model) | P0 |
| [BC-2.10.005](behavioral-contracts/BC-2.10.005-notifications-tools-list-changed.md) | notifications/tools/list_changed on Config Reload | P0 |
| [BC-2.10.006](behavioral-contracts/BC-2.10.006-stdio-transport.md) | Stdio Transport | P0 |
| [BC-2.10.007](behavioral-contracts/BC-2.10.007-structured-error-responses.md) | Structured Error Responses | P0 |
| [BC-2.10.008](behavioral-contracts/BC-2.10.008-mcp-resources.md) | MCP Resources for Client List and Sensor Inventory | P0 |
| [BC-2.10.009](behavioral-contracts/BC-2.10.009-mcp-prompts.md) | MCP Prompts for Common Workflows | P1 |
| [BC-2.10.010](behavioral-contracts/BC-2.10.010-graceful-shutdown.md) | Graceful Shutdown on SIGTERM/SIGINT | P0 |
| [BC-2.10.011](behavioral-contracts/BC-2.10.011-list-capabilities-meta-tool.md) | list_capabilities Meta-Tool | P0 |

### Subsystem 11: Query Execution (15 BCs)

Capabilities: CAP-015, CAP-016

| BC ID | Title | Priority |
|-------|-------|----------|
| [BC-2.11.001](behavioral-contracts/BC-2.11.001-query-mcp-tool.md) | `query` MCP Tool Accepts Scoping + PrismQL Query String | P0 |
| [BC-2.11.002](behavioral-contracts/BC-2.11.002-prismql-filter-mode.md) | PrismQL Filter Mode Parsing | P0 |
| [BC-2.11.003](behavioral-contracts/BC-2.11.003-prismql-sql-mode.md) | PrismQL SQL Mode Parsing | P0 |
| [BC-2.11.004](behavioral-contracts/BC-2.11.004-prismql-pipe-mode.md) | PrismQL Pipe Mode Parsing | P0 |
| [BC-2.11.005](behavioral-contracts/BC-2.11.005-ephemeral-materialization.md) | Ephemeral Materialization — Fan-Out, Normalize, Arrow RecordBatch, DataFusion MemTable | P0 |
| [BC-2.11.006](behavioral-contracts/BC-2.11.006-query-security-limits.md) | Query Security Limits Enforcement | P0 |
| [BC-2.11.007](behavioral-contracts/BC-2.11.007-sensor-filter-push-down.md) | Sensor Filter Push-Down | P0 |
| [BC-2.11.008](behavioral-contracts/BC-2.11.008-create-alias-tool.md) | `create_alias` MCP Tool | P1 |
| [BC-2.11.009](behavioral-contracts/BC-2.11.009-alias-resolution.md) | Alias Resolution — Pre-Parse Expansion, Composition, Cycle Detection | P1 |
| [BC-2.11.010](behavioral-contracts/BC-2.11.010-explain-query-tool.md) | `explain_query` MCP Tool | P0 |
| [BC-2.11.011](behavioral-contracts/BC-2.11.011-cross-client-query-scoping.md) | Cross-Client Query Scoping | P0 |
| [BC-2.11.012](behavioral-contracts/BC-2.11.012-virtual-fields.md) | Virtual Fields in Queries — `_sensor`, `_client`, `_source_table` | P0 |
| [BC-2.11.013](behavioral-contracts/BC-2.11.013-list-aliases-tool.md) | `list_aliases` MCP Tool | P1 |
| [BC-2.11.014](behavioral-contracts/BC-2.11.014-delete-alias-tool.md) | `delete_alias` MCP Tool | P1 |
| [BC-2.11.015](behavioral-contracts/BC-2.11.015-explain-alias-tool.md) | `explain_alias` MCP Tool | P1 |

### Subsystem 12: Scheduler (10 BCs)

Capabilities: CAP-017, CAP-018, CAP-023

Scheduled federated queries with differential result computation (what changed since last check?), tick-based execution with client_id-hash splay to avoid thundering herds, and query packs for curated MSSP workflow bundles. Differential results are computed by hashing previous results and returning only added/removed records. State persisted to RocksDB.

| BC ID | Title | Priority |
|-------|-------|----------|
| [BC-2.12.001](behavioral-contracts/BC-2.12.001-create-schedule-tool.md) | `create_schedule` MCP Tool — Create a Scheduled Query | P0 |
| [BC-2.12.002](behavioral-contracts/BC-2.12.002-list-schedules-tool.md) | `list_schedules` MCP Tool — List Active Schedules with Next Run Times | P0 |
| [BC-2.12.003](behavioral-contracts/BC-2.12.003-delete-schedule-tool.md) | `delete_schedule` MCP Tool — Remove a Schedule (Confirmation Required) | P0 |
| [BC-2.12.004](behavioral-contracts/BC-2.12.004-schedule-execution-loop.md) | Schedule Execution Loop — Tick-Based with Splay and In-Flight Skip | P0 |
| [BC-2.12.005](behavioral-contracts/BC-2.12.005-differential-result-computation.md) | Differential Result Computation — Hash Previous Results, Return Added/Removed | P0 |
| [BC-2.12.006](behavioral-contracts/BC-2.12.006-epoch-counter-tracking.md) | Epoch/Counter Tracking — Exactly-Once Semantics, Persist to Storage After Each Run | P0 |
| [BC-2.12.007](behavioral-contracts/BC-2.12.007-get-diff-results-tool.md) | `get_diff_results` MCP Tool — Retrieve Differential Results for a Scheduled Query | P0 |
| [BC-2.12.008](behavioral-contracts/BC-2.12.008-pack-loading-discovery.md) | Pack Loading and Discovery — Load Packs from Config, Run Discovery Queries, Conditional Execution | P0 |
| [BC-2.12.009](behavioral-contracts/BC-2.12.009-pack-crud-tools.md) | Pack CRUD MCP Tools — `create_pack`, `list_packs`, `delete_pack` | P0 |
| [BC-2.12.010](behavioral-contracts/BC-2.12.010-schedule-state-persistence.md) | Schedule State Persistence — RocksDB Domain for Scheduling Metadata | P0 |

### Subsystem 13: Detection Engine (14 BCs)

Capabilities: CAP-020, CAP-021, CAP-027

Three-tier detection: single-event (stateless per-record), correlation (threshold over sliding window with group-by, reset-after-fire), and sequence (ordered multi-step pattern matching within time window). Rules defined in .detect DSL, compiled to DataFusion SQL for push-down optimization. Three-scope rule resolution (global baseline + per-client overrides + analyst ad-hoc). Security UDFs (subnet_contains, ioc_match, time_window) registered with DataFusion. Alert generation with template interpolation and MCP notification broadcast. IOC pattern files provide locally-cached indicator matching for detection rules.

| BC ID | Title | Priority |
|-------|-------|----------|
| [BC-2.13.001](behavioral-contracts/BC-2.13.001-detection-rule-loading.md) | Detection Rule Loading — Parse PrismQL Predicate, Validate at Load Time, Reject Invalid Rules | P0 |
| [BC-2.13.002](behavioral-contracts/BC-2.13.002-single-event-detection.md) | Single-Event Detection — Evaluate Rule Predicate Against Each Differential Record | P0 |
| [BC-2.13.003](behavioral-contracts/BC-2.13.003-correlation-detection.md) | Correlation Detection — Threshold Over Sliding Window with Group-By, Reset-After-Fire | P0 |
| [BC-2.13.004](behavioral-contracts/BC-2.13.004-sequence-detection.md) | Sequence Detection — Ordered Multi-Event Pattern Matching Within Time Window | P0 |
| [BC-2.13.005](behavioral-contracts/BC-2.13.005-alert-generation.md) | Alert Generation — Interpolate Template, Persist Alert, Broadcast via MCP Notification | P0 |
| [BC-2.13.006](behavioral-contracts/BC-2.13.006-create-rule-tool.md) | `create_rule` MCP Tool — Create Detection Rule with Scope | P0 |
| [BC-2.13.007](behavioral-contracts/BC-2.13.007-list-rules-tool.md) | `list_rules` MCP Tool — List Active Rules by Scope | P0 |
| [BC-2.13.008](behavioral-contracts/BC-2.13.008-delete-rule-tool.md) | `delete_rule` MCP Tool — Remove Rule (Confirmation for Global Rules) | P0 |
| [BC-2.13.009](behavioral-contracts/BC-2.13.009-rule-to-sql-compilation.md) | Rule-to-SQL Compilation — Translate Detection Predicates to DataFusion WHERE Clauses | P0 |
| [BC-2.13.010](behavioral-contracts/BC-2.13.010-security-udf-registration.md) | Security UDF Registration — Register Domain-Specific Functions with DataFusion | P0 |
| [BC-2.13.011](behavioral-contracts/BC-2.13.011-three-scope-rule-resolution.md) | Three-Scope Rule Resolution — Global Baseline + Per-Client Overrides + Analyst Ad-Hoc | P0 |
| [BC-2.13.012](behavioral-contracts/BC-2.13.012-detection-state-persistence.md) | Detection State Persistence — RocksDB Domain for Correlation Windows, Sequence State, Alert History | P0 |
| [BC-2.13.013](behavioral-contracts/BC-2.13.013-alert-deduplication.md) | Alert Deduplication — Per-Match-Mode Dedup Keys Prevent Duplicate Alerts | P0 |
| [BC-2.13.014](behavioral-contracts/BC-2.13.014-ioc-file-loading-pattern-store.md) | IOC File Loading and Pattern Store — At-Startup Load with Hot Reload and Bounded Memory | P0 |

### Subsystem 14: Alert & Case Management (12 BCs)

Capabilities: CAP-021, CAP-022

Investigation case lifecycle with a 5-state machine (New, Acknowledged, Investigating, Resolved, Closed) and 12 valid transitions. Disposition assignment (TruePositive/FalsePositive/Benign/Inconclusive) required on resolution. Timeline annotations for audit trail (5 types including OT-specific impact). MTTD/MTTR auto-computed from alert and case timestamps. Alert acknowledgment (idempotent). Auto-case-creation from high-severity detection rules.

| BC ID | Title | Priority |
|-------|-------|----------|
| [BC-2.14.001](behavioral-contracts/BC-2.14.001-create-case-tool.md) | `create_case` MCP Tool — Create Case from One or More Alerts | P0 |
| [BC-2.14.002](behavioral-contracts/BC-2.14.002-case-state-transitions.md) | Case State Transitions — 5-State Machine with 12 Valid Transitions | P0 |
| [BC-2.14.003](behavioral-contracts/BC-2.14.003-update-case-tool.md) | `update_case` MCP Tool — Transition State, Set Disposition, Add Annotation | P0 |
| [BC-2.14.004](behavioral-contracts/BC-2.14.004-list-cases-tool.md) | `list_cases` MCP Tool — Filter by Status, Client, Severity, Assignee | P0 |
| [BC-2.14.005](behavioral-contracts/BC-2.14.005-get-case-tool.md) | `get_case` MCP Tool — Full Case Detail with Timeline and Linked Alerts | P0 |
| [BC-2.14.006](behavioral-contracts/BC-2.14.006-disposition-assignment.md) | Disposition Assignment — Required on Resolved Transition | P0 |
| [BC-2.14.007](behavioral-contracts/BC-2.14.007-timeline-annotations.md) | Timeline Annotations — 5 Types: Note, StatusChange, AlertLink, EvidenceLink, OtImpact | P0 |
| [BC-2.14.008](behavioral-contracts/BC-2.14.008-mttd-mttr-computation.md) | TTD/TTI/TTR Per-Case and Aggregate MTTD/MTTI/MTTR Computation — From Event Timestamps to Case State Transitions | P0 |
| [BC-2.14.009](behavioral-contracts/BC-2.14.009-case-persistence.md) | Case Persistence — RocksDB Domain for Case State, Timeline, Disposition, Metrics | P0 |
| [BC-2.14.010](behavioral-contracts/BC-2.14.010-case-metrics-tool.md) | `case_metrics` MCP Tool — Aggregate MTTD/MTTR and Case Status Counts | P0 |
| [BC-2.14.012](behavioral-contracts/BC-2.14.012-acknowledge-alert.md) | `acknowledge_alert` MCP Tool — Mark Alert as Acknowledged (Idempotent) | P0 |
| [BC-2.14.013](behavioral-contracts/BC-2.14.013-auto-case-creation.md) | Auto-Case-Creation from High-Severity Detection Rules | P1 |

### Subsystem 15: Storage Layer (11 BCs)

Capabilities: CAP-019, CAP-024, CAP-025, CAP-026, CAP-028

Cross-cutting platform services: RocksDB storage engine with domain-based column families, buffered audit log persistence with exponential backoff forwarding, crash recovery via dirty bits, resource watchdog with graduated limit levels (normal/restrictive/permissive) and query denylisting, context decorator injection (auto-inject client_id, sensor, analyst_id into all results) with a three-phase model (config-time, query-time, periodic), and unified query surface registration of internal RocksDB-backed tables as DataFusion tables queryable via PrismQL.

| BC ID | Title | Priority |
|-------|-------|----------|
| [BC-2.15.001](behavioral-contracts/BC-2.15.001-rocksdb-initialization.md) | RocksDB Initialization — Create/Open Database, Initialize Column Families for All Domains | P0 |
| [BC-2.15.002](behavioral-contracts/BC-2.15.002-domain-kv-operations.md) | Domain-Based Key-Value Operations — get/put/putBatch/remove/removeRange/scan per Domain | P0 |
| [BC-2.15.003](behavioral-contracts/BC-2.15.003-buffered-audit-log-persistence.md) | Buffered Audit Log Persistence — Write to RocksDB Before stderr/Vector, Exponential Backoff on Forward Failure | P0 |
| [BC-2.15.004](behavioral-contracts/BC-2.15.004-audit-buffer-overflow.md) | Audit Buffer Overflow — Purge Oldest Entries When Exceeding 100K, Log Warning | P0 |
| [BC-2.15.005](behavioral-contracts/BC-2.15.005-crash-recovery-dirty-bits.md) | Crash Recovery Dirty Bits — Set Before Operation, Clear After, Detect on Restart | P0 |
| [BC-2.15.006](behavioral-contracts/BC-2.15.006-resource-watchdog-initialization.md) | Resource Watchdog Initialization — Set Memory/CPU/Timeout Limits Based on Graduated Level | P0 |
| [BC-2.15.007](behavioral-contracts/BC-2.15.007-watchdog-query-termination.md) | Watchdog Query Termination — Kill Query Exceeding Limits, Return Structured Error | P0 |
| [BC-2.15.008](behavioral-contracts/BC-2.15.008-query-denylisting.md) | Query Denylisting — After N Consecutive Failures, Denylist with Manual Override | P0 |
| [BC-2.15.009](behavioral-contracts/BC-2.15.009-context-decorator-injection.md) | Context Decorator Injection — Auto-Inject Metadata into All Results | P0 |
| [BC-2.15.010](behavioral-contracts/BC-2.15.010-decorator-three-phase-model.md) | Decorator Three-Phase Model — Config-Time, Query-Time, Periodic | P0 |
| [BC-2.15.011](behavioral-contracts/BC-2.15.011-internal-table-registration.md) | Internal Table Registration — RocksDB Domains as DataFusion Tables | P0 |

### Subsystem 16: Spec Engine (10 BCs)

Capabilities: CAP-029, CAP-030

Config-driven sensor adapters defined in TOML spec files (not Rust code) with multi-step fetch pipelines, variable interpolation between steps, column-to-OCSF field mapping, and pagination config. Runtime-interpreted by the `prism-spec-engine` crate. Hot configuration reload via `reload_config` MCP tool using arc-swap for lock-free reads. Approximately 80% of REST API sensors fully config-driven; approximately 20% use Rust escape hatches for exotic auth or binary protocols. Adding a new sensor = drop a TOML spec file + add client config + set credentials.

| BC ID | Title | Priority |
|-------|-------|----------|
| [BC-2.16.001](behavioral-contracts/BC-2.16.001-sensor-spec-file-loading.md) | Sensor Spec File Loading — Parse TOML, Validate Schema, Register Tables | P0 |
| [BC-2.16.002](behavioral-contracts/BC-2.16.002-multi-step-fetch-pipeline.md) | Multi-Step Fetch Pipeline Execution — Sequential Steps with Variable Interpolation | P0 |
| [BC-2.16.003](behavioral-contracts/BC-2.16.003-column-to-ocsf-mapping.md) | Column-to-OCSF Mapping at Query Time — Map Sensor Columns to OCSF Fields Per Spec | P0 |
| [BC-2.16.004](behavioral-contracts/BC-2.16.004-rust-escape-hatch.md) | Rust Escape Hatch for Custom Adapters — Trait-Based Override When Config Is Insufficient | P0 |
| [BC-2.16.005](behavioral-contracts/BC-2.16.005-reload-config-tool.md) | `reload_config` MCP Tool — Re-Read All Config Files, Validate, Atomic Swap, Notify | P1 |
| [BC-2.16.006](behavioral-contracts/BC-2.16.006-arc-swap-config-access.md) | Arc-Swap Config Access on Hot Path — Lock-Free Reads for Query-Time Config Access | P1 |
| [BC-2.16.007](behavioral-contracts/BC-2.16.007-sensor-spec-hot-reload.md) | Sensor Spec Hot Reload — Add/Remove/Update Sensor Tables Without Restart | P1 |
| [BC-2.16.008](behavioral-contracts/BC-2.16.008-add-sensor-spec-tool.md) | `add_sensor_spec` MCP Tool — Upload a New Sensor Spec at Runtime | P0 |
| [BC-2.16.009](behavioral-contracts/BC-2.16.009-spec-file-validation.md) | Spec File Validation — Schema Validation, Variable Reference Resolution, OCSF Field Validation | P0 |
| [BC-2.16.010](behavioral-contracts/BC-2.16.010-list-sensor-specs-tool.md) | `list_sensor_specs` MCP Tool — List Loaded Sensor Specs with Table Schemas and Status | P0 |

### Subsystem 17: WASM Plugin Runtime (6 BCs)

Capabilities: CAP-032, CAP-030

WASM Component Model plugin runtime per AD-019. Enables polyglot plugins (Rust, Go, Python, JS, C#) as `.prx` files extending sensor adapters, infusions, and actions. Enforces four sandbox dimensions: panic isolation (INV-PLUGIN-001), no direct filesystem/network access (INV-PLUGIN-002), per-instance memory limit (INV-PLUGIN-003, default 64MB), and CPU time limit via epoch interruption (INV-PLUGIN-004, default 5s). Hot reload via atomic arc-swap (INV-PLUGIN-005). WIT interface validation before registration (INV-PLUGIN-006).

| BC ID | Title | Priority |
|-------|-------|----------|
| [BC-2.17.001](behavioral-contracts/BC-2.17.001-plugin-panic-isolation.md) | Plugin Panic Isolation — Crashed Plugin Does Not Terminate Host Process | P0 |
| [BC-2.17.002](behavioral-contracts/BC-2.17.002-plugin-sandbox-filesystem.md) | Plugin Sandbox — No Direct Filesystem or Network Access | P0 |
| [BC-2.17.003](behavioral-contracts/BC-2.17.003-plugin-memory-limit.md) | Plugin Sandbox — Memory Limit Enforced Per Plugin Instance (default 64MB) | P0 |
| [BC-2.17.004](behavioral-contracts/BC-2.17.004-plugin-cpu-time-limit.md) | Plugin Sandbox — CPU Time Limit Enforced via Epoch Interruption (default 5s) | P0 |
| [BC-2.17.005](behavioral-contracts/BC-2.17.005-plugin-hot-reload-atomic-swap.md) | Plugin Hot Reload — Atomic Module Swap, In-Flight Calls Complete Against Old Version | P0 |
| [BC-2.17.006](behavioral-contracts/BC-2.17.006-plugin-wit-validation.md) | WIT Interface Validation Before Plugin Registration | P0 |

### Subsystem 18: Action Delivery Engine (9 BCs)

Capability: CAP-033

Config-driven alert delivery and scheduled reporting per AD-021. Three trigger modes with different delivery guarantees: alert/case triggers (at-least-once with exponential backoff retry, INV-ACTION-001), schedule triggers (best-effort, INV-ACTION-002), manual triggers (fire-and-forget, INV-ACTION-003). Scheduled report queries acquire the shared 16-permit semaphore via try_acquire() (INV-ACTION-004) and tolerate per-section failures (INV-ACTION-005). Template variables injection-scanned before interpolation (INV-ACTION-006). Credentials use AI-opaque reference model (INV-ACTION-007). All outcomes audit-logged (INV-ACTION-008). UUID v7 validation for `${case.alert_ids_quoted}` (INV-ACTION-009).

| BC ID | Title | Priority |
|-------|-------|----------|
| [BC-2.18.001](behavioral-contracts/BC-2.18.001-action-at-least-once-delivery.md) | Alert and Case Action Triggers — At-Least-Once Delivery with Exponential Backoff Retry | P0 |
| [BC-2.18.002](behavioral-contracts/BC-2.18.002-action-schedule-best-effort.md) | Schedule Action Triggers — Best-Effort, Retry on Next Cron Tick | P0 |
| [BC-2.18.003](behavioral-contracts/BC-2.18.003-action-manual-fire-and-forget.md) | Manual Action Triggers — Fire-and-Forget, Result Returned Immediately to AI Caller | P0 |
| [BC-2.18.004](behavioral-contracts/BC-2.18.004-action-schedule-semaphore.md) | Scheduled Report Queries — try_acquire() on 16-Permit Semaphore, Skip If Unavailable | P0 |
| [BC-2.18.005](behavioral-contracts/BC-2.18.005-action-partial-report-failure.md) | Partial Report Failure — Failed Sections Include Error Note, Others Delivered | P0 |
| [BC-2.18.006](behavioral-contracts/BC-2.18.006-action-template-injection-scan.md) | Action Template Variables from Sensor/Alert Data — Injection-Scanned Before Interpolation | P0 |
| [BC-2.18.007](behavioral-contracts/BC-2.18.007-action-credential-opaque-reference.md) | Action Credentials Must Use AI-Opaque Reference Model — No Inline Values (E-ACTION-001) | P0 |
| [BC-2.18.008](behavioral-contracts/BC-2.18.008-action-delivery-audit-logging.md) | All Action Executions Are Audit-Logged — Success, Failure, and Suppression | P0 |
| [BC-2.18.009](behavioral-contracts/BC-2.18.009-action-uuid-v7-validation.md) | `${case.alert_ids_quoted}` Values Validated as UUID v7 Before Interpolation | P0 |

### Subsystem 19: Infusion Enrichment Framework (5 BCs)

Capability: CAP-031

Composable enrichment framework per AD-020 enabling GeoIP, threat intel, asset inventory, and CVSS enrichment via TOML specs and `.prx` WASM plugins. Each `[[infusion.fields]]` entry registers exactly one DataFusion scalar UDF (INV-INFUSE-001). Per-query dedup eliminates redundant source calls for repeated input values (INV-INFUSE-002). Plugin-backed infusions rejected in detection rule filters to prevent blocking async calls (INV-INFUSE-003). Hot reload uses CI-002 pattern (INV-INFUSE-004). Credential values never logged (INV-INFUSE-005).

| BC ID | Title | Priority |
|-------|-------|----------|
| [BC-2.19.001](behavioral-contracts/BC-2.19.001-infusion-spec-loading.md) | Infusion Spec Loading — Each Field Registers Exactly One DataFusion Scalar UDF | P0 |
| [BC-2.19.002](behavioral-contracts/BC-2.19.002-infusion-per-query-dedup.md) | Per-Query Dedup Cache — Unique Input Values Only, Not Per-Row | P0 |
| [BC-2.19.003](behavioral-contracts/BC-2.19.003-infusion-api-backed-rejection.md) | API-Backed Infusion UDFs Rejected in Detection Rule Filters — E-RULE-012 | P0 |
| [BC-2.19.004](behavioral-contracts/BC-2.19.004-infusion-hot-reload-atomicity.md) | Infusion Hot Reload — Failed Validation Retains Previous Registration (CI-002) | P0 |
| [BC-2.19.005](behavioral-contracts/BC-2.19.005-infusion-credential-redaction.md) | Infusion Credentials Are Never Logged or Included in Error Messages | P0 |

### Subsystem 20: Observability / Log Forwarding (5 BCs)

Capability: CAP-035 (Diagnostic Log Forwarding)

External log forwarding from Prism's observability pipeline to destinations
(Datadog, Splunk HEC, Elasticsearch, OTLP, syslog, generic webhook). Implemented
in `prism-mcp` per the architecture (see `architecture/observability.md`).

**Stories:** S-5.09 (External Log Forwarding Subsystem), S-6.16–S-6.19 (per-destination
DTU stories).

| BC ID | Title | Priority |
|-------|-------|----------|
| [BC-2.20.001](behavioral-contracts/BC-2.20.001-log-forwarder-recursive-prevention.md) | Log Forwarder Recursive Prevention — Plugin host.log() Writes to Local Sink Only | P0 |
| [BC-2.20.002](behavioral-contracts/BC-2.20.002-log-forwarder-min-level-filter.md) | Log Forwarder Min-Level Filter — Per-Destination min_level Applied Before Enqueue | P0 |
| [BC-2.20.003](behavioral-contracts/BC-2.20.003-log-forwarder-queue-cap.md) | Log Forwarder Queue Cap — Drop-Oldest on Overflow with Metric Emission | P0 |
| [BC-2.20.004](behavioral-contracts/BC-2.20.004-log-forwarder-credential-resolution.md) | Log Forwarder Credential Resolution — AD-017 Opaque Reference Model at Forward Time | P0 |
| [BC-2.20.005](behavioral-contracts/BC-2.20.005-log-forwarder-destination-isolation.md) | Log Forwarder Destination Isolation — Single Failed Destination Must Not Block Others | P0 |

### BC Distribution Summary

| Subsystem | BC Count | P0 | P1 |
|-----------|----------|----|----|
| 01 - Sensor Adapters | 9 | 9 | 0 |
| 02 - OCSF Normalization | 12 | 12 | 0 |
| 03 - Credential Management | 12 | 12 | 0 |
| 04 - Feature Flags | 15 | 9 | 6 |
| 05 - Audit Trail | 11 | 11 | 0 |
| 06 - Client Configuration | 10 | 10 | 0 |
| 07 - Adapter Pagination & Response Cache | 6 | 2 | 4 |
| 08 - Sensor Health | 9 | 0 | 9 |
| 09 - Prompt Injection Defense | 8 | 8 | 0 |
| 10 - MCP Interface | 11 | 10 | 1 |
| 11 - Query Execution | 15 | 10 | 5 |
| 12 - Scheduler | 10 | 10 | 0 |
| 13 - Detection Engine | 14 | 14 | 0 |
| 14 - Alert & Case Management | 12 | 11 | 1 |
| 15 - Storage Layer | 11 | 11 | 0 |
| 16 - Spec Engine | 10 | 7 | 3 |
| 17 - WASM Plugin Runtime | 6 | 6 | 0 |
| 18 - Action Delivery Engine | 9 | 9 | 0 |
| 19 - Infusion Enrichment Framework | 5 | 5 | 0 |
| 20 - Observability / Log Forwarding | 5 | 5 | 0 |
| **Total** | **200** | **171** | **29** |

---

## 3. Interface Definitions

Prism exposes functionality exclusively through the Model Context Protocol (MCP) over stdio transport. The interface definition supplement specifies:

- **MCP Tool Schemas** -- Parameter types and `outputSchema` for sensor query, health check, credential management, capability listing, and confirmation tools
- **Error Response Schema** -- Structured error envelope with `category`, `retryable`, `retry_after_seconds`, `suggestion`, and `original_params_valid` fields enabling LLM self-correction
- **TOML Configuration Schema** -- Required and optional fields for client, sensor, credential reference, and feature flag configuration
- **CLI Flags** -- `--config`, `--dry-run`, `--log-level`, and other startup parameters
- **Exit Codes** -- Numeric codes for clean shutdown, config error, fatal state error, etc.

Full specification: [prd-supplements/interface-definitions.md](prd-supplements/interface-definitions.md)

---

## 4. Non-Functional Requirements

23 non-functional requirements covering five quality dimensions:

- **Performance** -- Query latency budgets, OCSF normalization overhead limits, memory ceiling
- **Security** -- AES-256-GCM credential encryption at rest, prompt injection defense for attacker-controlled content, secret redaction in all outputs
- **Reliability** -- Ephemeral pagination with automatic cleanup, response caching with TTL-based eviction, graceful shutdown within 5 seconds
- **Observability** -- Structured JSON logging via `tracing`, Vector pipeline compatibility, audit trail completeness
- **Compatibility** -- Cross-platform (Linux, macOS, Windows), OCSF v1.x version pinning, rmcp 1.4 SDK compatibility (upgraded from 0.8 during architecture phase; rmcp reached 1.x stability)

Full specification: [prd-supplements/nfr-catalog.md](prd-supplements/nfr-catalog.md)

---

## 5. Error Taxonomy

33 error categories with 100+ error codes, each specifying severity, retryability, and structured message format:

| Category | Code Prefix | Description |
|----------|-------------|-------------|
| AUTH | AUTH-* | Sensor authentication failures (expired tokens, invalid credentials, cookie rejection) |
| SENSOR | SENSOR-* | Sensor API errors (HTTP 5xx, timeouts, rate limiting, unexpected response formats) |
| OCSF | OCSF-* | Normalization failures (unmappable fields, schema version mismatch, invalid enum values) |
| CRED | CRED-* | Credential store errors (keyring locked, decryption failure, missing credentials) |
| FLAG | FLAG-* | Feature flag errors (capability denied, token expired, token hash mismatch, cross-client protection) |
| STATE | STATE-* | Pagination state errors (expired or invalid ephemeral cursor, RocksDB scan failure) |
| CACHE | CACHE-* | Response cache errors (cache invalidation failure during write operations) |
| CFG | CFG-* | Configuration errors (invalid TOML, missing required fields, validation failures) |
| MCP | MCP-* | Protocol errors (invalid parameters, unknown tool, transport failures, diagnostics truncation) |
| AUDIT | AUDIT-* | Audit emission errors (write operation blocked when audit subscriber fails, buffer overflow) |
| QUERY | QUERY-* | Query engine errors (parse errors, type errors, security limits, timeout, materialization limits, denylist) |
| ALIAS | ALIAS-* | Alias errors (not found, cycles, depth exceeded, parameter validation, reserved name conflicts) |
| IO | IO-* | Filesystem I/O errors (atomic file operations) |
| SAFETY | SAFETY-* | Prompt injection defense triggers (suspicious patterns detected, trust level violations) |
| SCHED | SCHED-* | Scheduled query errors (not found, invalid interval, name conflicts, concurrency limits) |
| PACK | PACK-* | Query pack errors (parse errors, validation, name conflicts) |
| DIFF | DIFF-* | Differential result errors (no previous results, record limit exceeded) |
| RULE | RULE-* | Detection rule errors (parse errors, validation failures, scope conflicts, infusion rejection) |
| DETECT | DETECT-* | Detection evaluation errors (field type mismatch, sequence state failures, dedup failures) |
| ALERT | ALERT-* | Alert errors (not found, already acknowledged) |
| CASE | CASE-* | Case management errors (not found, invalid transitions, disposition required, annotation validation) |
| STORE | STORE-* | Storage errors (RocksDB initialization, write/read failures, corruption, disk space, dirty bit failure) |
| IOC | IOC-* | Indicator of compromise file errors (invalid regex patterns, size/count limits, file count cap) |
| UDF | UDF-* | User-defined function errors (unknown pattern set, invalid duration, malformed CIDR) |
| WATCH | WATCH-* | Watchdog configuration errors (invalid level, unsafe override values) |
| WATCHDOG | WATCHDOG-* | Watchdog enforcement errors (memory limit exceeded, concurrent query pressure) |
| DECOR | DECOR-* | Context decorator errors (refresh failures, missing config fields) |
| SPEC | SPEC-* | Sensor spec errors (parse errors, invalid column types, circular step dependencies, duplicate IDs) |
| INFUSE | INFUSE-* | Infusion enrichment errors (unknown infusion name) |
| METRICS | METRICS-* | Case metrics errors (date range exceeding aggregation limit) |
| ACTION | ACTION-* | Action delivery errors (inline credential detected in action spec) |
| RELOAD | RELOAD-* | Configuration reload errors (file read errors, validation failures, partial spec load, no-op detection) |
| PLUGIN | PLUGIN-* | WASM plugin errors (execution failure, incompatible WIT interface, resource limit exceeded) |

Table regenerated from `prd-supplements/error-taxonomy.md` section headers. Keep in sync.

Full specification: [prd-supplements/error-taxonomy.md](prd-supplements/error-taxonomy.md)

---

## 5b. Test Vectors

Canonical input/output pairs for behavioral contracts whose correctness is verifiable
via deterministic example inputs are cataloged in
[prd-supplements/test-vectors.md](prd-supplements/test-vectors.md).

The supplement provides TV-NNN identifiers that Phase 3 test-writer agents consume as
golden fixtures. When a BC's postconditions or error taxonomy changes, its canonical
test vector MUST be updated in the same commit (Policy 7: source-of-truth integrity).

**Coverage (v1.0):** 10 vectors spanning 8 subsystems (audit, authn/feature-flags, query,
detection, case, stdio, spec-engine). Expansion is expected during Phase 3 as test-writers
identify additional canonical cases.

**Reference convention:** BCs in this PRD that have a canonical test vector cite it via
`**Canonical test vector:** [TV-NNN](prd-supplements/test-vectors.md#tv-nnn-*)` in the
BC's body. Missing citation is a drift signal.

---

## 6. Competitive Differentiator Traceability

Mapping each competitive differentiator to the behavioral contracts that implement it.

### 6.1 AI-Native Interface (Not a Dashboard)

Prism is consumed by AI agent harnesses, not humans. All responses are structured for LLM reasoning.

| BC ID | Contribution |
|-------|-------------|
| BC-2.11.001 | `query` tool response format with structured events and query context |
| BC-2.09.007 | OutputSchema for type-safe LLM reasoning |
| BC-2.09.008 | Response envelope with trust annotations |
| BC-2.10.001 | rmcp ServerHandler implementation |
| BC-2.10.007 | Structured error responses with retryability and suggestions |
| BC-2.05.001 | Audit entry per invocation (observability for AI-driven workflows) |

### 6.2 Cross-Sensor Correlation via OCSF

All sensor data normalized to a common schema, enabling cross-sensor joins via the query engine.

| BC ID | Contribution |
|-------|-------------|
| BC-2.11.005 | Ephemeral materialization -- unified Arrow table from multiple sensors |
| BC-2.11.012 | Virtual fields (`_sensor`, `_client`, `_source_table`) for cross-sensor filtering |
| BC-2.02.001 | OCSF schema loading at build time |
| BC-2.02.002 | DynamicMessage creation from sensor records |
| BC-2.02.003 | CrowdStrike field mapping to OCSF |
| BC-2.02.004 | Cyberint field mapping to OCSF |
| BC-2.02.005 | Claroty field mapping to OCSF (9 data sources) |
| BC-2.02.006 | Armis field mapping to OCSF (7 data sources) |
| BC-2.02.008 | Four-tier field alias resolution |
| BC-2.02.012 | Event class selection per record type |

### 6.3 Multi-Client Single Session (Stateless Model)

Client scoping on every tool call with cross-client query support. Read tools use `clients` array via `query`; write tools use scalar `client_id`. No session-level "active client".

| BC ID | Contribution |
|-------|-------------|
| BC-2.11.011 | Cross-client query scoping via `clients` array |
| BC-2.01.002 | Cross-client fan-out with per-client attribution |
| BC-2.06.001 | TOML config with per-client structure |
| BC-2.06.002 | Per-client sensor mapping |
| BC-2.06.010 | Client ID validation |
| BC-2.10.004 | Client scoping on every tool (stateless model) |
| BC-2.10.008 | MCP resources for client list and sensor inventory |

### 6.4 Feature-Flagged Write Operations

Two-tier gate with three-tier risk classification, hierarchical override (BTreeMap, most-specific-path wins, deny support), and confirmation tokens (100-token active cap).

| BC ID | Contribution |
|-------|-------------|
| BC-2.04.001 | Compile-time cargo features gate write code |
| BC-2.04.002 | Runtime per-client TOML flags |
| BC-2.04.003 | Hierarchical capability resolution (BTreeMap, most-specific-path wins, deny) |
| BC-2.04.004 | Two-tier gate (both must pass) |
| BC-2.04.005 | Hidden tools pattern |
| BC-2.04.007 | Three-tier risk classification |
| BC-2.04.008 | Dry-run default for reversible writes |
| BC-2.04.009 | Confirmation token generation with 100-token active cap |
| BC-2.04.010 | Confirmation token consumption |
| BC-2.04.011 | Token expiry at 300 seconds |
| BC-2.04.012 | Token content hash verification |
| BC-2.10.003 | Conditional tool registration (feature-flag gated) |

### 6.5 OCSF with Vendor Extension Preservation

Normalized view for correlation plus raw_extensions for vendor-specific deep dives.

| BC ID | Contribution |
|-------|-------------|
| BC-2.02.007 | Vendor extension preservation in raw_extensions |
| BC-2.02.009 | OCSF version pinning per release |
| BC-2.02.010 | Enum value map for runtime display names |
| BC-2.02.011 | Graceful normalization error handling (no silent data loss) |

### 6.6 Prompt Injection Defense for Security Data

Four-layer sanitization for attacker-controlled content in LLM context.

| BC ID | Contribution |
|-------|-------------|
| BC-2.09.001 | Structural separation of untrusted data |
| BC-2.09.002 | Provenance framing in tool descriptions |
| BC-2.09.003 | Suspicious pattern detection via regex with NFKC normalization |
| BC-2.09.004 | Safety flags via _meta.safety_flags array (centralized, not per-field) |
| BC-2.09.005 | Trust-level metadata per response |
| BC-2.09.006 | Tool description security warnings |
| BC-2.09.007 | OutputSchema for type-safe LLM reasoning |
| BC-2.09.008 | Response envelope with trust annotations |
| BC-2.10.007 | Structured error responses (untrusted data in errors) |
| BC-2.10.009 | MCP prompts with security framing |

### 6.7 Unified Sensor Adapter Architecture

Trait-based DataSource<T> eliminates per-sensor code duplication.

| BC ID | Contribution |
|-------|-------------|
| BC-2.01.013 | DataSource trait adapter pattern |
| BC-2.01.005 | CrowdStrike OAuth2 adapter |
| BC-2.01.006 | Cyberint cookie auth adapter |
| BC-2.01.007 | Claroty bearer token adapter |
| BC-2.01.008 | Armis bearer token adapter |
| BC-2.01.014 | Exponential backoff (shared across adapters) |
| BC-2.08.001 | On-demand connectivity check (per adapter) |
| BC-2.08.002 | Auth validity check (per adapter) |

### 6.8 SOC 2 / ISO 27001 Audit Trail

Every MCP invocation logged with compliance-grade structured fields.

| BC ID | Contribution |
|-------|-------------|
| BC-2.05.001 | Audit entry per tool invocation |
| BC-2.05.002 | Structured JSON format with complete fields |
| BC-2.05.003 | Secret redaction in audit entries |
| BC-2.05.004 | Write operation audit detail |
| BC-2.05.005 | Credential access audit |
| BC-2.05.006 | Append-only immutability |
| BC-2.05.007 | Vector pipeline compatibility |
| BC-2.05.008 | SOC 2 Type II and ISO 27001 field requirements |
| BC-2.05.009 | Feature flag evaluation audit |
| BC-2.05.010 | Confirmation token lifecycle audit |
| BC-2.03.007 | Secret redaction in logs, errors, and MCP responses |
| BC-2.03.010 | Credential access audit logging |
| BC-2.04.013 | Feature flag evaluation audit logging |

---

## 7. Requirements Traceability Matrix

Complete mapping of all 200 active behavioral contracts (208 total, 6 removed, 2 retired) to source capabilities, subsystems, and priorities.

| BC ID | Source CAP | Subsystem | Priority |
|-------|-----------|-----------|----------|
| BC-2.01.002 | CAP-002 | 01 - Sensor Adapters | P0 |
| BC-2.01.004 | CAP-001 | 01 - Sensor Adapters | P0 |
| BC-2.01.005 | CAP-001 | 01 - Sensor Adapters | P0 |
| BC-2.01.006 | CAP-001 | 01 - Sensor Adapters | P0 |
| BC-2.01.007 | CAP-001 | 01 - Sensor Adapters | P0 |
| BC-2.01.008 | CAP-001 | 01 - Sensor Adapters | P0 |
| BC-2.01.010 | CAP-001, CAP-002 | 01 - Sensor Adapters | P0 |
| BC-2.01.013 | CAP-001 | 01 - Sensor Adapters | P0 |
| BC-2.01.014 | CAP-001 | 01 - Sensor Adapters | P0 |
| BC-2.02.001 | CAP-003 | 02 - OCSF Normalization | P0 |
| BC-2.02.002 | CAP-003 | 02 - OCSF Normalization | P0 |
| BC-2.02.003 | CAP-003 | 02 - OCSF Normalization | P0 |
| BC-2.02.004 | CAP-003 | 02 - OCSF Normalization | P0 |
| BC-2.02.005 | CAP-003 | 02 - OCSF Normalization | P0 |
| BC-2.02.006 | CAP-003 | 02 - OCSF Normalization | P0 |
| BC-2.02.007 | CAP-003 | 02 - OCSF Normalization | P0 |
| BC-2.02.008 | CAP-003 | 02 - OCSF Normalization | P0 |
| BC-2.02.009 | CAP-003 | 02 - OCSF Normalization | P0 |
| BC-2.02.010 | CAP-003 | 02 - OCSF Normalization | P0 |
| BC-2.02.011 | CAP-003 | 02 - OCSF Normalization | P0 |
| BC-2.02.012 | CAP-003 | 02 - OCSF Normalization | P0 |
| BC-2.03.001 | CAP-004 | 03 - Credential Management | P0 |
| BC-2.03.002 | CAP-004 | 03 - Credential Management | P0 |
| BC-2.03.003 | CAP-004 | 03 - Credential Management | P0 |
| BC-2.03.004 | CAP-004 | 03 - Credential Management | P0 |
| BC-2.03.005 | CAP-004 | 03 - Credential Management | P0 |
| BC-2.03.006 | CAP-004 | 03 - Credential Management | P0 |
| BC-2.03.007 | CAP-004 | 03 - Credential Management | P0 |
| BC-2.03.008 | CAP-004 | 03 - Credential Management | P0 |
| BC-2.03.009 | CAP-004 | 03 - Credential Management | P0 |
| BC-2.03.010 | CAP-004 | 03 - Credential Management | P0 |
| BC-2.03.011 | CAP-004 | 03 - Credential Management | P0 |
| BC-2.03.012 | CAP-004 | 03 - Credential Management | P0 |
| BC-2.04.001 | CAP-005 | 04 - Feature Flags | P0 |
| BC-2.04.002 | CAP-005 | 04 - Feature Flags | P0 |
| BC-2.04.003 | CAP-005 | 04 - Feature Flags | P0 |
| BC-2.04.004 | CAP-005 | 04 - Feature Flags | P0 |
| BC-2.04.005 | CAP-005 | 04 - Feature Flags | P0 |
| BC-2.04.006 | CAP-005 | 04 - Feature Flags | P0 |
| BC-2.04.007 | CAP-006 | 04 - Feature Flags | P1 |
| BC-2.04.008 | CAP-006 | 04 - Feature Flags | P1 |
| BC-2.04.009 | CAP-006 | 04 - Feature Flags | P1 |
| BC-2.04.010 | CAP-006 | 04 - Feature Flags | P1 |
| BC-2.04.011 | CAP-006 | 04 - Feature Flags | P1 |
| BC-2.04.012 | CAP-006 | 04 - Feature Flags | P1 |
| BC-2.04.013 | CAP-005 | 04 - Feature Flags | P0 |
| BC-2.04.014 | CAP-005 | 04 - Feature Flags | P0 |
| BC-2.04.015 | CAP-005 | 04 - Feature Flags | P0 |
| BC-2.05.001 | CAP-007 | 05 - Audit Trail | P0 |
| BC-2.05.002 | CAP-007 | 05 - Audit Trail | P0 |
| BC-2.05.003 | CAP-007 | 05 - Audit Trail | P0 |
| BC-2.05.004 | CAP-007 | 05 - Audit Trail | P0 |
| BC-2.05.005 | CAP-007 | 05 - Audit Trail | P0 |
| BC-2.05.006 | CAP-007 | 05 - Audit Trail | P0 |
| BC-2.05.007 | CAP-007 | 05 - Audit Trail | P0 |
| BC-2.05.008 | CAP-007 | 05 - Audit Trail | P0 |
| BC-2.05.009 | CAP-007 | 05 - Audit Trail | P0 |
| BC-2.05.010 | CAP-007 | 05 - Audit Trail | P0 |
| BC-2.05.011 | CAP-007 | 05 - Audit Trail | P0 |
| BC-2.06.001 | CAP-009 | 06 - Client Configuration | P0 |
| BC-2.06.002 | CAP-009 | 06 - Client Configuration | P0 |
| BC-2.06.003 | CAP-009 | 06 - Client Configuration | P0 |
| BC-2.06.004 | CAP-009 | 06 - Client Configuration | P0 |
| BC-2.06.005 | CAP-009 | 06 - Client Configuration | P0 |
| BC-2.06.006 | CAP-009 | 06 - Client Configuration | P0 |
| BC-2.06.007 | CAP-009 | 06 - Client Configuration | P0 |
| BC-2.06.008 | CAP-009 | 06 - Client Configuration | P0 |
| BC-2.06.009 | CAP-009 | 06 - Client Configuration | P0 |
| BC-2.06.010 | CAP-009 | 06 - Client Configuration | P0 |
| BC-2.07.001 | CAP-011 | 07 - Adapter Pagination & Response Cache | P0 |
| BC-2.07.002 | CAP-011 | 07 - Adapter Pagination & Response Cache | P0 |
| BC-2.07.003 | CAP-014 | 07 - Adapter Pagination & Response Cache | P1 |
| BC-2.07.004 | CAP-014 | 07 - Adapter Pagination & Response Cache | P1 |
| BC-2.07.005 | CAP-014 | 07 - Adapter Pagination & Response Cache | P1 |
| BC-2.07.006 | CAP-014 | 07 - Adapter Pagination & Response Cache | P1 |
| BC-2.08.001 | CAP-008 | 08 - Sensor Health | P1 |
| BC-2.08.002 | CAP-008 | 08 - Sensor Health | P1 |
| BC-2.08.003 | CAP-008 | 08 - Sensor Health | P1 |
| BC-2.08.004 | CAP-008 | 08 - Sensor Health | P1 |
| BC-2.08.005 | CAP-008 | 08 - Sensor Health | P1 |
| BC-2.08.006 | CAP-008 | 08 - Sensor Health | P1 |
| BC-2.08.007 | CAP-008 | 08 - Sensor Health | P1 |
| BC-2.08.008 | CAP-008 | 08 - Sensor Health | P1 |
| BC-2.08.009 | CAP-008 | 08 - Sensor Health | P1 |
| BC-2.09.001 | CAP-010 | 09 - Prompt Injection Defense | P0 |
| BC-2.09.002 | CAP-010 | 09 - Prompt Injection Defense | P0 |
| BC-2.09.003 | CAP-010 | 09 - Prompt Injection Defense | P0 |
| BC-2.09.004 | CAP-010 | 09 - Prompt Injection Defense | P0 |
| BC-2.09.005 | CAP-010 | 09 - Prompt Injection Defense | P0 |
| BC-2.09.006 | CAP-010 | 09 - Prompt Injection Defense | P0 |
| BC-2.09.007 | CAP-010 | 09 - Prompt Injection Defense | P0 |
| BC-2.09.008 | CAP-010 | 09 - Prompt Injection Defense | P0 |
| BC-2.10.001 | CAP-034 | 10 - MCP Interface | P0 |
| BC-2.10.002 | CAP-005, CAP-015 | 10 - MCP Interface | P0 |
| BC-2.10.003 | CAP-005 | 10 - MCP Interface | P0 |
| BC-2.10.004 | CAP-009 | 10 - MCP Interface | P0 |
| BC-2.10.005 | CAP-005, CAP-009 | 10 - MCP Interface | P0 |
| BC-2.10.006 | CAP-034 | 10 - MCP Interface | P0 |
| BC-2.10.007 | CAP-034 | 10 - MCP Interface | P0 |
| BC-2.10.008 | CAP-008, CAP-009 | 10 - MCP Interface | P0 |
| BC-2.10.009 | CAP-034 | 10 - MCP Interface | P1 |
| BC-2.10.010 | CAP-034 | 10 - MCP Interface | P0 |
| BC-2.10.011 | CAP-005 | 10 - MCP Interface | P0 |
| BC-2.11.001 | CAP-015 | 11 - Query Execution | P0 |
| BC-2.11.002 | CAP-015 | 11 - Query Execution | P0 |
| BC-2.11.003 | CAP-015 | 11 - Query Execution | P0 |
| BC-2.11.004 | CAP-015 | 11 - Query Execution | P0 |
| BC-2.11.005 | CAP-015 | 11 - Query Execution | P0 |
| BC-2.11.006 | CAP-015 | 11 - Query Execution | P0 |
| BC-2.11.007 | CAP-015 | 11 - Query Execution | P0 |
| BC-2.11.008 | CAP-016 | 11 - Query Execution | P1 |
| BC-2.11.009 | CAP-016 | 11 - Query Execution | P1 |
| BC-2.11.010 | CAP-015 | 11 - Query Execution | P0 |
| BC-2.11.011 | CAP-015 | 11 - Query Execution | P0 |
| BC-2.11.012 | CAP-015 | 11 - Query Execution | P0 |
| BC-2.11.013 | CAP-016 | 11 - Query Execution | P1 |
| BC-2.11.014 | CAP-016 | 11 - Query Execution | P1 |
| BC-2.11.015 | CAP-016 | 11 - Query Execution | P1 |
| BC-2.12.001 | CAP-017 | 12 - Scheduler | P0 |
| BC-2.12.002 | CAP-017 | 12 - Scheduler | P0 |
| BC-2.12.003 | CAP-017 | 12 - Scheduler | P0 |
| BC-2.12.004 | CAP-017 | 12 - Scheduler | P0 |
| BC-2.12.005 | CAP-018 | 12 - Scheduler | P0 |
| BC-2.12.006 | CAP-018 | 12 - Scheduler | P0 |
| BC-2.12.007 | CAP-018 | 12 - Scheduler | P0 |
| BC-2.12.008 | CAP-023 | 12 - Scheduler | P0 |
| BC-2.12.009 | CAP-023 | 12 - Scheduler | P0 |
| BC-2.12.010 | CAP-017 | 12 - Scheduler | P0 |
| BC-2.13.001 | CAP-020 | 13 - Detection Engine | P0 |
| BC-2.13.002 | CAP-020 | 13 - Detection Engine | P0 |
| BC-2.13.003 | CAP-020 | 13 - Detection Engine | P0 |
| BC-2.13.004 | CAP-020 | 13 - Detection Engine | P0 |
| BC-2.13.005 | CAP-020 | 13 - Detection Engine | P0 |
| BC-2.13.006 | CAP-020 | 13 - Detection Engine | P0 |
| BC-2.13.007 | CAP-020 | 13 - Detection Engine | P0 |
| BC-2.13.008 | CAP-020 | 13 - Detection Engine | P0 |
| BC-2.13.009 | CAP-027 | 13 - Detection Engine | P0 |
| BC-2.13.010 | CAP-027 | 13 - Detection Engine | P0 |
| BC-2.13.011 | CAP-020 | 13 - Detection Engine | P0 |
| BC-2.13.012 | CAP-020 | 13 - Detection Engine | P0 |
| BC-2.13.013 | CAP-021 | 13 - Detection Engine | P0 |
| BC-2.13.014 | CAP-020 | 13 - Detection Engine | P0 |
| BC-2.14.001 | CAP-022 | 14 - Alert & Case Management | P0 |
| BC-2.14.002 | CAP-022 | 14 - Alert & Case Management | P0 |
| BC-2.14.003 | CAP-022 | 14 - Alert & Case Management | P0 |
| BC-2.14.004 | CAP-022 | 14 - Alert & Case Management | P0 |
| BC-2.14.005 | CAP-022 | 14 - Alert & Case Management | P0 |
| BC-2.14.006 | CAP-022 | 14 - Alert & Case Management | P0 |
| BC-2.14.007 | CAP-022 | 14 - Alert & Case Management | P0 |
| BC-2.14.008 | CAP-022 | 14 - Alert & Case Management | P0 |
| BC-2.14.009 | CAP-022 | 14 - Alert & Case Management | P0 |
| BC-2.14.010 | CAP-022 | 14 - Alert & Case Management | P0 |
| BC-2.14.012 | CAP-022 | 14 - Alert & Case Management | P0 |
| BC-2.14.013 | CAP-022 | 14 - Alert & Case Management | P1 |
| BC-2.15.001 | CAP-019 | 15 - Storage Layer | P0 |
| BC-2.15.002 | CAP-019 | 15 - Storage Layer | P0 |
| BC-2.15.003 | CAP-025 | 15 - Storage Layer | P0 |
| BC-2.15.004 | CAP-025 | 15 - Storage Layer | P0 |
| BC-2.15.005 | CAP-024 | 15 - Storage Layer | P0 |
| BC-2.15.006 | CAP-024 | 15 - Storage Layer | P0 |
| BC-2.15.007 | CAP-024 | 15 - Storage Layer | P0 |
| BC-2.15.008 | CAP-024 | 15 - Storage Layer | P0 |
| BC-2.15.009 | CAP-026 | 15 - Storage Layer | P0 |
| BC-2.15.010 | CAP-026 | 15 - Storage Layer | P0 |
| BC-2.15.011 | CAP-028 | 15 - Storage Layer | P0 |
| BC-2.16.001 | CAP-029 | 16 - Spec Engine | P0 |
| BC-2.16.002 | CAP-029 | 16 - Spec Engine | P0 |
| BC-2.16.003 | CAP-029 | 16 - Spec Engine | P0 |
| BC-2.16.004 | CAP-029 | 16 - Spec Engine | P0 |
| BC-2.16.005 | CAP-030 | 16 - Spec Engine | P1 |
| BC-2.16.006 | CAP-030 | 16 - Spec Engine | P1 |
| BC-2.16.007 | CAP-030 | 16 - Spec Engine | P1 |
| BC-2.16.008 | CAP-029, CAP-030 | 16 - Spec Engine | P0 |
| BC-2.16.009 | CAP-029 | 16 - Spec Engine | P0 |
| BC-2.16.010 | CAP-029 | 16 - Spec Engine | P0 |
| BC-2.17.001 | CAP-032 | 17 - WASM Plugin Runtime | P0 |
| BC-2.17.002 | CAP-032 | 17 - WASM Plugin Runtime | P0 |
| BC-2.17.003 | CAP-032 | 17 - WASM Plugin Runtime | P0 |
| BC-2.17.004 | CAP-032 | 17 - WASM Plugin Runtime | P0 |
| BC-2.17.005 | CAP-030 | 17 - WASM Plugin Runtime | P0 |
| BC-2.17.006 | CAP-032 | 17 - WASM Plugin Runtime | P0 |
| BC-2.18.001 | CAP-033 | 18 - Action Delivery Engine | P0 |
| BC-2.18.002 | CAP-033 | 18 - Action Delivery Engine | P0 |
| BC-2.18.003 | CAP-033 | 18 - Action Delivery Engine | P0 |
| BC-2.18.004 | CAP-033 | 18 - Action Delivery Engine | P0 |
| BC-2.18.005 | CAP-033 | 18 - Action Delivery Engine | P0 |
| BC-2.18.006 | CAP-033 | 18 - Action Delivery Engine | P0 |
| BC-2.18.007 | CAP-033 | 18 - Action Delivery Engine | P0 |
| BC-2.18.008 | CAP-033 | 18 - Action Delivery Engine | P0 |
| BC-2.18.009 | CAP-033 | 18 - Action Delivery Engine | P0 |
| BC-2.19.001 | CAP-031 | 19 - Infusion Enrichment Framework | P0 |
| BC-2.19.002 | CAP-031 | 19 - Infusion Enrichment Framework | P0 |
| BC-2.19.003 | CAP-031 | 19 - Infusion Enrichment Framework | P0 |
| BC-2.19.004 | CAP-030, CAP-031 | 19 - Infusion Enrichment Framework | P0 |
| BC-2.19.005 | CAP-031 | 19 - Infusion Enrichment Framework | P0 |
| BC-2.20.001 | CAP-035 | 20 - Observability / Log Forwarding | P0 |
| BC-2.20.002 | CAP-035 | 20 - Observability / Log Forwarding | P0 |
| BC-2.20.003 | CAP-035 | 20 - Observability / Log Forwarding | P0 |
| BC-2.20.004 | CAP-035 | 20 - Observability / Log Forwarding | P0 |
| BC-2.20.005 | CAP-035 | 20 - Observability / Log Forwarding | P0 |

### Capability Coverage Summary

Regenerated from BC file `capability:` frontmatter fields (Burst 13 Part B, updated Burst 21 Task A, 2026-04-17; CAP-035 re-anchor pass-80 follow-on). CAP titles are canonical from `domain-spec/capabilities.md`. Grand total column sum = 206 (200 active BCs + 6 active dual-anchor extras). Dual-anchor BCs (6 active = BC-2.01.010, BC-2.10.002, BC-2.10.008, BC-2.16.008, BC-2.19.004, BC-2.10.005) are each counted once under each anchored CAP in the summary.

| CAP ID | Capability | BC Count | BC References |
|--------|-----------|----------|---------------|
| CAP-001 | Sensor Adapter Layer (Internal) | 8 | BC-2.01.004/005/006/007/008/010/013/014 |
| CAP-002 | Cross-Client Fan-Out (Internal) | 2 | BC-2.01.002, BC-2.01.010 |
| CAP-003 | OCSF Normalization | 12 | BC-2.02.001–012 |
| CAP-004 | Credential Management | 12 | BC-2.03.001–012 |
| CAP-005 | Feature Flag Evaluation | 13 | BC-2.04.001/002/003/004/005/006/013/014/015, BC-2.10.002/003/005/011 |
| CAP-006 | Write Operation Gating | 6 | BC-2.04.007/008/009/010/011/012 |
| CAP-007 | Audit Logging | 11 | BC-2.05.001/002/003/004/005/006/007/008/009/010/011 |
| CAP-008 | Sensor Health Monitoring | 10 | BC-2.08.001/002/003/004/005/006/007/008/009, BC-2.10.008 |
| CAP-009 | Client Configuration | 13 | BC-2.06.001/002/003/004/005/006/007/008/009/010, BC-2.10.004/005/008 |
| CAP-010 | Prompt Injection Defense | 8 | BC-2.09.001/002/003/004/005/006/007/008 |
| CAP-011 | Adapter Pagination (Internal) | 2 | BC-2.07.001/002 |
| CAP-014 | Response Caching | 4 | BC-2.07.003/004/005/006 |
| CAP-015 | Ephemeral OCSF Query Engine | 11 | BC-2.11.001/002/003/004/005/006/007/010/011/012, BC-2.10.002 |
| CAP-016 | Query Aliases | 5 | BC-2.11.008/009/013/014/015 |
| CAP-017 | Scheduled Queries | 5 | BC-2.12.001/002/003/004/010 |
| CAP-018 | Differential Results | 3 | BC-2.12.005/006/007 |
| CAP-019 | Persistent Storage | 2 | BC-2.15.001/002 |
| CAP-020 | Detection Rules | 11 | BC-2.13.001/002/003/004/005/006/007/008/011/012/014 |
| CAP-021 | Alert Generation | 1 | BC-2.13.013 |
| CAP-022 | Case Management | 12 | BC-2.14.001/002/003/004/005/006/007/008/009/010/012/013 |
| CAP-023 | Query Packs | 2 | BC-2.12.008/009 |
| CAP-024 | Resource Watchdog | 4 | BC-2.15.005/006/007/008 |
| CAP-025 | Buffered Audit Logging | 2 | BC-2.15.003/004 |
| CAP-026 | Context Decorators | 2 | BC-2.15.009/010 |
| CAP-027 | Security Domain UDFs | 2 | BC-2.13.009/010 |
| CAP-028 | Unified Query Surface | 1 | BC-2.15.011 |
| CAP-029 | Config-Driven Sensor Adapters | 7 | BC-2.16.001/002/003/004/008/009/010 |
| CAP-030 | Hot Configuration Reload | 6 | BC-2.16.005/006/007/008, BC-2.17.005, BC-2.19.004 |
| CAP-031 | Infusion Enrichment | 5 | BC-2.19.001/002/003/004/005 |
| CAP-032 | WASM Plugin Runtime | 5 | BC-2.17.001/002/003/004/006 |
| CAP-033 | Action Delivery Engine | 9 | BC-2.18.001/002/003/004/005/006/007/008/009 |
| CAP-034 | MCP Server & Transport | 5 | BC-2.10.001/006/007/009/010 |
| CAP-035 | Diagnostic Log Forwarding | 5 | BC-2.20.001/002/003/004/005 |

---

## Change Log

- 2026-04-21 (pass-81-remediation F81-005): §4 NFR count synced 18 → 23 (NFR-001..023 in nfr-catalog.md). Version bumped 1.1→1.2.
- 2026-04-21 (pass-80 follow-on F80-002): Added SS-20 subsystem block to §2 with CAP-035 anchor and BC-2.20.001–005 table (5 P0 BCs). Updated §2 distribution summary SS-20 row 0→5 BCs and grand total 195→200. Version bumped 1.0→1.1.
- 2026-04-21 (pass-80 follow-on): Re-anchored SS-20 BCs CAP-025 → CAP-035 (Diagnostic Log Forwarding). §2 and §7 counts updated 195→200 active / 203→208 total. Added BC-2.20.001–005 rows to §7 traceability matrix. Added CAP-035 row to Capability Coverage Summary. Updated grand total column sum 201→206.
- 2026-04-19 (Burst 34): Updated §4 NFR count from 16 to 18 per pass-33 M-002. Canonical count lives in prd-supplements/nfr-catalog.md which defines NFR-001 through NFR-018 (18 entries). NFR-017 (Cache Bounds per DI-018) and NFR-018 (Token Store Cap per DI-015) were added during Phase 2 patch refinement. PRD body text had not been synced.
