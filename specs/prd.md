---
document_type: prd
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-04-14T06:00:00
phase: 1a
inputs: [domain-spec/L2-INDEX.md, product-brief.md]
input-hash: "b668a4b"
traces_to: domain-spec/L2-INDEX.md
supplements: [prd-supplements/interface-definitions.md, prd-supplements/error-taxonomy.md, prd-supplements/nfr-catalog.md]
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
- Custom detection rule authoring

---

## 2. Behavioral Contracts Index

114 behavioral contracts organized across 11 subsystems. Each BC specifies a single testable behavior with preconditions, postconditions, invariants, and error cases. Individual BC files are located in `behavioral-contracts/`.

### Subsystem 01: Sensor Query Pipeline (14 BCs)

Capabilities: CAP-001, CAP-002, CAP-012

| BC ID | Title | Priority |
|-------|-------|----------|
| [BC-2.01.001](behavioral-contracts/BC-2.01.001-single-client-sensor-query.md) | Single-Client Sensor Query Returns Scoped Results | P0 |
| [BC-2.01.002](behavioral-contracts/BC-2.01.002-cross-client-fan-out.md) | Cross-Client Fan-Out Query Aggregates Results with Per-Client Attribution | P0 |
| [BC-2.01.003](behavioral-contracts/BC-2.01.003-cursor-based-pagination.md) | Cursor-Based Forward-Only Pagination | P0 |
| [BC-2.01.004](behavioral-contracts/BC-2.01.004-offset-based-pagination-claroty.md) | Offset-Based Hybrid Pagination for Claroty Audit Logs | P0 |
| [BC-2.01.005](behavioral-contracts/BC-2.01.005-crowdstrike-oauth2-two-step-fetch.md) | CrowdStrike OAuth2 Authentication and Two-Step Fetch | P0 |
| [BC-2.01.006](behavioral-contracts/BC-2.01.006-cyberint-cookie-auth.md) | Cyberint Cookie-Based Authentication and Multi-Format Timestamp Parsing | P0 |
| [BC-2.01.007](behavioral-contracts/BC-2.01.007-claroty-bearer-polymorphic-ids.md) | Claroty Bearer Token Auth with Polymorphic ID Handling | P0 |
| [BC-2.01.008](behavioral-contracts/BC-2.01.008-armis-bearer-aql.md) | Armis Bearer Token Auth with AQL Query Forwarding and Timestamp Fallback | P0 |
| [BC-2.01.009](behavioral-contracts/BC-2.01.009-query-filtering-sorting.md) | Query Filtering and Sorting Parameters | P0 |
| [BC-2.01.010](behavioral-contracts/BC-2.01.010-partial-failure-handling.md) | Partial Failure Handling for Paginated and Cross-Client Queries | P0 |
| [BC-2.01.011](behavioral-contracts/BC-2.01.011-cross-sensor-correlation-ocsf-fields.md) | Cross-Sensor Correlation via OCSF Field Alignment | P1 |
| [BC-2.01.013](behavioral-contracts/BC-2.01.013-datasource-trait-adapter-pattern.md) | DataSource Trait Eliminates Per-Sensor Code Duplication | P0 |
| [BC-2.01.014](behavioral-contracts/BC-2.01.014-sensor-api-http-503-mid-pagination.md) | Exponential Backoff and Retry for Transient Sensor API Errors | P0 |
| [BC-2.01.015](behavioral-contracts/BC-2.01.015-response-envelope-structure.md) | MCP Tool Response Envelope Structure | P0 |

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
| [BC-2.02.008](behavioral-contracts/BC-2.02.008-field-alias-resolution.md) | Three-Tier Field Alias Resolution | P0 |
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

### Subsystem 04: Feature Flag System (14 BCs)

Capabilities: CAP-005, CAP-006

| BC ID | Title | Priority |
|-------|-------|----------|
| [BC-2.04.001](behavioral-contracts/BC-2.04.001-compile-time-cargo-features.md) | Compile-Time Cargo Features Gate Write Code Families | P0 |
| [BC-2.04.002](behavioral-contracts/BC-2.04.002-runtime-per-client-toml-flags.md) | Runtime Per-Client TOML Feature Flag Configuration | P0 |
| [BC-2.04.003](behavioral-contracts/BC-2.04.003-hierarchical-flag-resolution.md) | Hierarchical Capability Resolution (BTreeMap, Most-Specific-Path Wins, Deny Support) | P0 |
| [BC-2.04.004](behavioral-contracts/BC-2.04.004-two-tier-gate-both-must-pass.md) | Two-Tier Gate -- Both Compile-Time and Runtime Must Permit Operation | P0 |
| [BC-2.04.005](behavioral-contracts/BC-2.04.005-hidden-tools-pattern.md) | Hidden Tools Pattern -- Disabled Write Tools Omitted from tools/list | P0 |
| [BC-2.04.006](behavioral-contracts/BC-2.04.006-list-capabilities-meta-tool.md) | list_capabilities Meta-Tool for Capability Discovery | P0 |
| [BC-2.04.007](behavioral-contracts/BC-2.04.007-three-tier-risk-classification.md) | Three-Tier Risk Classification for Operations | P1 |
| [BC-2.04.008](behavioral-contracts/BC-2.04.008-dry-run-default-reversible-writes.md) | Dry-Run Default for Reversible Write Operations | P1 |
| [BC-2.04.009](behavioral-contracts/BC-2.04.009-confirmation-token-request.md) | Confirmation Token Generation with 100-Token Active Cap | P1 |
| [BC-2.04.010](behavioral-contracts/BC-2.04.010-confirmation-token-consumption.md) | Confirmation Token Consumption via confirm_action | P1 |
| [BC-2.04.011](behavioral-contracts/BC-2.04.011-token-expiry-300s.md) | Token Expiry at 300 Seconds with Structured Error Recovery | P1 |
| [BC-2.04.012](behavioral-contracts/BC-2.04.012-token-content-hash-verification.md) | Token Content Hash Verification Prevents Action Tampering | P1 |
| [BC-2.04.013](behavioral-contracts/BC-2.04.013-capability-check-audit-logging.md) | Feature Flag Evaluation Audit Logging for Write Operations | P0 |
| [BC-2.04.015](behavioral-contracts/BC-2.04.015-write-denied-structured-error.md) | Structured Error When Write Capability Is Denied | P0 |

### Subsystem 05: Audit & Compliance (10 BCs)

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

### Subsystem 06: Client Configuration (9 BCs)

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
| [BC-2.06.010](behavioral-contracts/BC-2.06.010-client-id-validation.md) | Client ID Validation Enforces Allowed Character Set | P0 |

### Subsystem 07: Pagination & Caching (6 BCs)

Capabilities: CAP-011, CAP-014

| BC ID | Title | Priority |
|-------|-------|----------|
| [BC-2.07.001](behavioral-contracts/BC-2.07.001-ephemeral-cursor-pagination.md) | Ephemeral Cursor-Based Pagination (No Persistent State) | P0 |
| [BC-2.07.002](behavioral-contracts/BC-2.07.002-pagination-token-lifecycle.md) | Pagination Token Lifecycle — Forward Progress, Expiry, and Cleanup | P0 |
| [BC-2.07.003](behavioral-contracts/BC-2.07.003-response-cache-ttl.md) | Response Cache with Configurable TTL | P1 |
| [BC-2.07.004](behavioral-contracts/BC-2.07.004-cache-invalidation-on-writes.md) | Cache Invalidation on Write Operations | P1 |
| [BC-2.07.005](behavioral-contracts/BC-2.07.005-cache-key-derivation.md) | Cache Key Derivation from Query Parameters | P1 |
| [BC-2.07.006](behavioral-contracts/BC-2.07.006-cache-memory-bounds-eviction.md) | Cache Memory Bounds and Eviction Policy | P1 |

### Subsystem 08: Sensor Health (7 BCs)

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

### Subsystem 09: Prompt Injection Defense (8 BCs)

Capability: CAP-010

| BC ID | Title | Priority |
|-------|-------|----------|
| [BC-2.09.001](behavioral-contracts/BC-2.09.001-structural-separation.md) | Structural Separation of Untrusted Data | P0 |
| [BC-2.09.002](behavioral-contracts/BC-2.09.002-provenance-framing.md) | Provenance Framing in Tool Descriptions | P0 |
| [BC-2.09.003](behavioral-contracts/BC-2.09.003-suspicious-pattern-detection.md) | Suspicious Pattern Detection via Regex | P0 |
| [BC-2.09.004](behavioral-contracts/BC-2.09.004-safety-flag-parallel-fields.md) | Safety Flag Parallel Fields (Flag, Don't Strip) | P0 |
| [BC-2.09.005](behavioral-contracts/BC-2.09.005-trust-level-metadata.md) | Trust-Level Metadata Per Response | P0 |
| [BC-2.09.006](behavioral-contracts/BC-2.09.006-tool-description-security-warnings.md) | Tool Description Security Warnings | P0 |
| [BC-2.09.007](behavioral-contracts/BC-2.09.007-output-schema-type-safety.md) | OutputSchema for Type-Safe LLM Reasoning | P0 |
| [BC-2.09.008](behavioral-contracts/BC-2.09.008-response-envelope-trust-annotations.md) | Response Envelope with Trust Annotations | P0 |

### Subsystem 10: MCP Server & Transport (10 BCs)

Cross-cutting capabilities: CAP-005, CAP-007, CAP-009, CAP-010

| BC ID | Title | Priority |
|-------|-------|----------|
| [BC-2.10.001](behavioral-contracts/BC-2.10.001-server-handler-implementation.md) | rmcp ServerHandler Implementation | P0 |
| [BC-2.10.002](behavioral-contracts/BC-2.10.002-tool-registration-via-tool-router.md) | Tool Registration via #[tool_router] | P0 |
| [BC-2.10.003](behavioral-contracts/BC-2.10.003-conditional-tool-registration.md) | Conditional Tool Registration (Feature-Flag Gated) | P0 |
| [BC-2.10.004](behavioral-contracts/BC-2.10.004-client-id-parameter-requirement.md) | client_id Parameter on Every Tool (Stateless Model) | P0 |
| [BC-2.10.006](behavioral-contracts/BC-2.10.006-stdio-transport.md) | Stdio Transport | P0 |
| [BC-2.10.007](behavioral-contracts/BC-2.10.007-structured-error-responses.md) | Structured Error Responses | P0 |
| [BC-2.10.008](behavioral-contracts/BC-2.10.008-mcp-resources.md) | MCP Resources for Client List and Sensor Inventory | P0 |
| [BC-2.10.009](behavioral-contracts/BC-2.10.009-mcp-prompts.md) | MCP Prompts for Common Workflows | P1 |
| [BC-2.10.010](behavioral-contracts/BC-2.10.010-graceful-shutdown.md) | Graceful Shutdown on SIGTERM/SIGINT | P0 |
| [BC-2.10.011](behavioral-contracts/BC-2.10.011-list-capabilities-meta-tool.md) | list_capabilities Meta-Tool | P0 |

### Subsystem 11: Query Engine & Aliases (12 BCs)

Capabilities: CAP-015, CAP-016

| BC ID | Title | Priority |
|-------|-------|----------|
| [BC-2.11.001](behavioral-contracts/BC-2.11.001-query-mcp-tool.md) | `query` MCP Tool Accepts Scoping + AxiQL Query String | P0 |
| [BC-2.11.002](behavioral-contracts/BC-2.11.002-axiql-filter-mode.md) | AxiQL Filter Mode Parsing | P0 |
| [BC-2.11.003](behavioral-contracts/BC-2.11.003-axiql-sql-mode.md) | AxiQL SQL Mode Parsing | P0 |
| [BC-2.11.004](behavioral-contracts/BC-2.11.004-axiql-pipe-mode.md) | AxiQL Pipe Mode Parsing | P0 |
| [BC-2.11.005](behavioral-contracts/BC-2.11.005-ephemeral-materialization.md) | Ephemeral Materialization — Fan-Out, Normalize, Arrow RecordBatch, DataFusion MemTable | P0 |
| [BC-2.11.006](behavioral-contracts/BC-2.11.006-query-security-limits.md) | Query Security Limits Enforcement | P0 |
| [BC-2.11.007](behavioral-contracts/BC-2.11.007-sensor-filter-push-down.md) | Sensor Filter Push-Down | P0 |
| [BC-2.11.008](behavioral-contracts/BC-2.11.008-create-alias-tool.md) | `create_alias` MCP Tool | P1 |
| [BC-2.11.009](behavioral-contracts/BC-2.11.009-alias-resolution.md) | Alias Resolution — Pre-Parse Expansion, Composition, Cycle Detection | P1 |
| [BC-2.11.010](behavioral-contracts/BC-2.11.010-explain-query-tool.md) | `explain_query` MCP Tool | P0 |
| [BC-2.11.011](behavioral-contracts/BC-2.11.011-cross-client-query-scoping.md) | Cross-Client Query Scoping | P0 |
| [BC-2.11.012](behavioral-contracts/BC-2.11.012-virtual-fields.md) | Virtual Fields in Queries — `sensor`, `client_id`, `source` | P0 |

### BC Distribution Summary

| Subsystem | BC Count | P0 | P1 |
|-----------|----------|----|----|
| 01 - Sensor Query Pipeline | 14 | 13 | 1 |
| 02 - OCSF Normalization | 12 | 12 | 0 |
| 03 - Credential Management | 12 | 12 | 0 |
| 04 - Feature Flag System | 14 | 8 | 6 |
| 05 - Audit & Compliance | 10 | 10 | 0 |
| 06 - Client Configuration | 9 | 9 | 0 |
| 07 - Pagination & Caching | 6 | 2 | 4 |
| 08 - Sensor Health | 7 | 0 | 7 |
| 09 - Prompt Injection Defense | 8 | 8 | 0 |
| 10 - MCP Server & Transport | 10 | 9 | 1 |
| 11 - Query Engine & Aliases | 12 | 10 | 2 |
| **Total** | **114** | **93** | **21** |

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

16 non-functional requirements covering five quality dimensions:

- **Performance** -- Query latency budgets, OCSF normalization overhead limits, memory ceiling
- **Security** -- AES-256-GCM credential encryption at rest, prompt injection defense for attacker-controlled content, secret redaction in all outputs
- **Reliability** -- Ephemeral pagination with automatic cleanup, response caching with TTL-based eviction, graceful shutdown within 5 seconds
- **Observability** -- Structured JSON logging via `tracing`, Vector pipeline compatibility, audit trail completeness
- **Compatibility** -- Cross-platform (Linux, macOS, Windows), OCSF v1.x version pinning, rmcp 0.8 SDK compatibility

Full specification: [prd-supplements/nfr-catalog.md](prd-supplements/nfr-catalog.md)

---

## 5. Error Taxonomy

11 error categories with 37 error codes, each specifying severity, retryability, and structured message format:

| Category | Code Prefix | Description |
|----------|-------------|-------------|
| AUTH | AUTH-* | Sensor authentication failures (expired tokens, invalid credentials, cookie rejection) |
| SENSOR | SENSOR-* | Sensor API errors (HTTP 5xx, timeouts, rate limiting, unexpected response formats) |
| OCSF | OCSF-* | Normalization failures (unmappable fields, schema version mismatch, invalid enum values) |
| CRED | CRED-* | Credential store errors (keyring locked, decryption failure, missing credentials) |
| FLAG | FLAG-* | Feature flag errors (capability denied, token expired, token hash mismatch) |
| STATE | STATE-* | Pagination state errors (expired or invalid ephemeral cursor) |
| CACHE | CACHE-* | Response cache errors (cache invalidation failure during write operations) |
| CFG | CFG-* | Configuration errors (invalid TOML, missing required fields, validation failures) |
| MCP | MCP-* | Protocol errors (invalid parameters, unknown tool, transport failures) |
| AUDIT | AUDIT-* | Audit emission errors (write operation blocked when audit subscriber fails) |
| SAFETY | SAFETY-* | Prompt injection defense triggers (suspicious patterns detected, trust level violations) |

Full specification: [prd-supplements/error-taxonomy.md](prd-supplements/error-taxonomy.md)

---

## 6. Competitive Differentiator Traceability

Mapping each competitive differentiator to the behavioral contracts that implement it.

### 6.1 AI-Native Interface (Not a Dashboard)

Prism is consumed by AI agent harnesses, not humans. All responses are structured for LLM reasoning.

| BC ID | Contribution |
|-------|-------------|
| BC-2.01.015 | MCP tool response envelope with structuredContent |
| BC-2.09.007 | OutputSchema for type-safe LLM reasoning |
| BC-2.09.008 | Response envelope with trust annotations |
| BC-2.10.001 | rmcp ServerHandler implementation |
| BC-2.10.007 | Structured error responses with retryability and suggestions |
| BC-2.05.001 | Audit entry per invocation (observability for AI-driven workflows) |

### 6.2 Cross-Sensor Correlation via OCSF

All sensor data normalized to a common schema, enabling cross-sensor joins.

| BC ID | Contribution |
|-------|-------------|
| BC-2.01.011 | Cross-sensor correlation via OCSF field alignment |
| BC-2.02.001 | OCSF schema loading at build time |
| BC-2.02.002 | DynamicMessage creation from sensor records |
| BC-2.02.003 | CrowdStrike field mapping to OCSF |
| BC-2.02.004 | Cyberint field mapping to OCSF |
| BC-2.02.005 | Claroty field mapping to OCSF (9 data sources) |
| BC-2.02.006 | Armis field mapping to OCSF (7 data sources) |
| BC-2.02.008 | Three-tier field alias resolution |
| BC-2.02.012 | Event class selection per record type |

### 6.3 Multi-Client Single Session (Stateless Model)

Explicit client_id on every tool call with cross-client query support. No session-level "active client" -- every tool call carries client_id.

| BC ID | Contribution |
|-------|-------------|
| BC-2.01.001 | Single-client scoped results |
| BC-2.01.002 | Cross-client fan-out with per-client attribution |
| BC-2.06.001 | TOML config with per-client structure |
| BC-2.06.002 | Per-client sensor mapping |
| BC-2.06.010 | Client ID validation |
| BC-2.10.004 | client_id parameter on every tool (stateless model) |
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
| BC-2.09.003 | Suspicious pattern detection via regex |
| BC-2.09.004 | Safety flag parallel fields (flag, don't strip) |
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

Complete mapping of all 102 behavioral contracts to source capabilities, subsystems, and priorities.

| BC ID | Source CAP | Subsystem | Priority |
|-------|-----------|-----------|----------|
| BC-2.01.001 | CAP-001 | 01 - Sensor Query Pipeline | P0 |
| BC-2.01.002 | CAP-002 | 01 - Sensor Query Pipeline | P0 |
| BC-2.01.003 | CAP-001 | 01 - Sensor Query Pipeline | P0 |
| BC-2.01.004 | CAP-001 | 01 - Sensor Query Pipeline | P0 |
| BC-2.01.005 | CAP-001 | 01 - Sensor Query Pipeline | P0 |
| BC-2.01.006 | CAP-001 | 01 - Sensor Query Pipeline | P0 |
| BC-2.01.007 | CAP-001 | 01 - Sensor Query Pipeline | P0 |
| BC-2.01.008 | CAP-001 | 01 - Sensor Query Pipeline | P0 |
| BC-2.01.009 | CAP-001 | 01 - Sensor Query Pipeline | P0 |
| BC-2.01.010 | CAP-001, CAP-002 | 01 - Sensor Query Pipeline | P0 |
| BC-2.01.011 | CAP-012 | 01 - Sensor Query Pipeline | P1 |
| BC-2.01.013 | CAP-001 | 01 - Sensor Query Pipeline | P0 |
| BC-2.01.014 | CAP-001 | 01 - Sensor Query Pipeline | P0 |
| BC-2.01.015 | CAP-001 | 01 - Sensor Query Pipeline | P0 |
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
| BC-2.04.001 | CAP-005 | 04 - Feature Flag System | P0 |
| BC-2.04.002 | CAP-005 | 04 - Feature Flag System | P0 |
| BC-2.04.003 | CAP-005 | 04 - Feature Flag System | P0 |
| BC-2.04.004 | CAP-005 | 04 - Feature Flag System | P0 |
| BC-2.04.005 | CAP-005 | 04 - Feature Flag System | P0 |
| BC-2.04.006 | CAP-005 | 04 - Feature Flag System | P0 |
| BC-2.04.007 | CAP-006 | 04 - Feature Flag System | P1 |
| BC-2.04.008 | CAP-006 | 04 - Feature Flag System | P1 |
| BC-2.04.009 | CAP-006 | 04 - Feature Flag System | P1 |
| BC-2.04.010 | CAP-006 | 04 - Feature Flag System | P1 |
| BC-2.04.011 | CAP-006 | 04 - Feature Flag System | P1 |
| BC-2.04.012 | CAP-006 | 04 - Feature Flag System | P1 |
| BC-2.04.013 | CAP-005 | 04 - Feature Flag System | P0 |
| BC-2.04.015 | CAP-005 | 04 - Feature Flag System | P0 |
| BC-2.05.001 | CAP-007 | 05 - Audit & Compliance | P0 |
| BC-2.05.002 | CAP-007 | 05 - Audit & Compliance | P0 |
| BC-2.05.003 | CAP-007 | 05 - Audit & Compliance | P0 |
| BC-2.05.004 | CAP-007 | 05 - Audit & Compliance | P0 |
| BC-2.05.005 | CAP-007 | 05 - Audit & Compliance | P0 |
| BC-2.05.006 | CAP-007 | 05 - Audit & Compliance | P0 |
| BC-2.05.007 | CAP-007 | 05 - Audit & Compliance | P0 |
| BC-2.05.008 | CAP-007 | 05 - Audit & Compliance | P0 |
| BC-2.05.009 | CAP-007 | 05 - Audit & Compliance | P0 |
| BC-2.05.010 | CAP-007 | 05 - Audit & Compliance | P0 |
| BC-2.06.001 | CAP-009 | 06 - Client Configuration | P0 |
| BC-2.06.002 | CAP-009 | 06 - Client Configuration | P0 |
| BC-2.06.003 | CAP-009 | 06 - Client Configuration | P0 |
| BC-2.06.004 | CAP-009 | 06 - Client Configuration | P0 |
| BC-2.06.005 | CAP-009 | 06 - Client Configuration | P0 |
| BC-2.06.006 | CAP-009 | 06 - Client Configuration | P0 |
| BC-2.06.007 | CAP-009 | 06 - Client Configuration | P0 |
| BC-2.06.008 | CAP-009 | 06 - Client Configuration | P0 |
| BC-2.06.010 | CAP-009 | 06 - Client Configuration | P0 |
| BC-2.07.001 | CAP-011 | 07 - Pagination & Caching | P0 |
| BC-2.07.002 | CAP-011 | 07 - Pagination & Caching | P0 |
| BC-2.07.003 | CAP-014 | 07 - Pagination & Caching | P0 |
| BC-2.07.004 | CAP-014 | 07 - Pagination & Caching | P0 |
| BC-2.07.005 | CAP-014 | 07 - Pagination & Caching | P0 |
| BC-2.07.006 | CAP-014 | 07 - Pagination & Caching | P0 |
| BC-2.08.001 | CAP-008 | 08 - Sensor Health | P1 |
| BC-2.08.002 | CAP-008 | 08 - Sensor Health | P1 |
| BC-2.08.003 | CAP-008 | 08 - Sensor Health | P1 |
| BC-2.08.004 | CAP-008 | 08 - Sensor Health | P1 |
| BC-2.08.005 | CAP-008 | 08 - Sensor Health | P1 |
| BC-2.08.006 | CAP-008 | 08 - Sensor Health | P1 |
| BC-2.08.007 | CAP-008 | 08 - Sensor Health | P1 |
| BC-2.09.001 | CAP-010 | 09 - Prompt Injection Defense | P0 |
| BC-2.09.002 | CAP-010 | 09 - Prompt Injection Defense | P0 |
| BC-2.09.003 | CAP-010 | 09 - Prompt Injection Defense | P0 |
| BC-2.09.004 | CAP-010 | 09 - Prompt Injection Defense | P0 |
| BC-2.09.005 | CAP-010 | 09 - Prompt Injection Defense | P0 |
| BC-2.09.006 | CAP-010 | 09 - Prompt Injection Defense | P0 |
| BC-2.09.007 | CAP-010 | 09 - Prompt Injection Defense | P0 |
| BC-2.09.008 | CAP-010 | 09 - Prompt Injection Defense | P0 |
| BC-2.10.001 | CAP-005 | 10 - MCP Server & Transport | P0 |
| BC-2.10.002 | CAP-005 | 10 - MCP Server & Transport | P0 |
| BC-2.10.003 | CAP-005 | 10 - MCP Server & Transport | P0 |
| BC-2.10.004 | CAP-009 | 10 - MCP Server & Transport | P0 |
| BC-2.10.006 | -- | 10 - MCP Server & Transport | P0 |
| BC-2.10.007 | CAP-007 | 10 - MCP Server & Transport | P0 |
| BC-2.10.008 | CAP-009 | 10 - MCP Server & Transport | P0 |
| BC-2.10.009 | CAP-010 | 10 - MCP Server & Transport | P1 |
| BC-2.10.010 | -- | 10 - MCP Server & Transport | P0 |
| BC-2.10.011 | CAP-005 | 10 - MCP Server & Transport | P0 |

### Capability Coverage Summary

| CAP ID | Capability | BC Count |
|--------|-----------|----------|
| CAP-001 | Sensor Query (Single-Client) | 13 |
| CAP-002 | Cross-Client Query | 2 |
| CAP-003 | OCSF Normalization | 12 |
| CAP-004 | Credential Management | 12 |
| CAP-005 | Feature Flag Evaluation | 12 |
| CAP-006 | Write Operation Gating | 6 |
| CAP-007 | Audit Logging | 11 |
| CAP-008 | Sensor Health Monitoring | 7 |
| CAP-009 | Client Configuration | 11 |
| CAP-010 | Prompt Injection Defense | 9 |
| CAP-011 | Ephemeral Pagination | 2 |
| CAP-012 | Cross-Sensor Correlation | 1 |
| CAP-014 | Response Caching | 4 |
| -- | Infrastructure (no CAP) | 2 |
