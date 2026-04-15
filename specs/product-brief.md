---
document_type: product-brief
level: L1
version: "1.0"
status: draft
producer: "human + orchestrator"
timestamp: 2026-04-14T03:00:00
phase: 1a
inputs:
  - phase-0-ingestion/project-context.md
  - phase-0-ingestion/recovered-architecture.md
  - phase-0-ingestion/unified-security-posture.md
  - phase-0-ingestion/convention-reconciliation.md
  - phase-0-ingestion/module-criticality.md
  - phase-0-ingestion/feature-flag-research.md
  - phase-0-ingestion/ai-consumable-design-research.md
  - phase-0-ingestion/mssp-workflow-research.md
input-hash: "phase-0-complete-9c6f540"
traces_to: ""
---

# Product Brief: Prism

## What Is This?

Prism is an ephemeral federated query engine that lets MSS analysts query live security sensor data across all clients and sensors using a unified query language (AxiQL), with results normalized to OCSF. There is no database -- the data lake is conjured on demand from live API calls and exists only for the duration of the query.

Where a traditional SIEM follows an **Ingest -> Store -> Index -> Query** pipeline (data at rest), Prism inverts the model: **Query -> Fetch -> Normalize -> Compute -> Return** (data in flight). The analyst writes what looks like a database query, but underneath Prism orchestrates live API calls to CrowdStrike, Claroty, Cyberint, and Armis, normalizes every response to OCSF via the DynamicMessage protobuf pattern, materializes an ephemeral Arrow virtual table, executes the query through DataFusion, returns results, and tears down the table. No ETL pipeline, no index maintenance, no stale data.

Architecturally, Prism is analogous to Trino/Presto (federated SQL over heterogeneous data sources), but purpose-built for the security domain: OCSF as universal schema, per-client multi-sensor fan-out, and MCP as the AI-native interface consumed by Claude Code.

Prism is implemented as a Rust-based MCP server that gives MSS analysts a unified, AI-powered interface to every security sensor across every client -- from a single Claude Code session. Instead of logging into sensor dashboards separately for each client, an analyst asks their AI agent to pull alerts, correlate data, and take action across all clients and sensors through natural language. OCSF normalization via protobuf makes cross-sensor correlation possible for the first time.

## Who Is It For?

| Persona | Pain Point | Current Workaround |
|---------|-----------|-------------------|
| **SOC Analyst** | Context-switches between 4+ sensor dashboards per client, multiplied by dozens of clients. Cannot correlate CrowdStrike endpoint alerts with Claroty OT events for the same client. | Logs into each sensor's web console individually. Copies data between browser tabs. Maintains mental model of which client has which sensors. |
| **Threat Hunter** | Cannot search for indicators of compromise across all clients and sensors simultaneously. Each sensor has its own query language and search interface. | Runs the same search in each sensor console for each client sequentially. Manually correlates results in spreadsheets or notes. |
| **Security Engineer** | No programmatic access to sensor APIs for automation or integration. Client onboarding requires manual credential setup per sensor. | Writes one-off scripts per sensor. Manages credentials in shared vaults or documentation. |
| **Client-Facing Staff** (secondary) | Needs to generate security posture reports for clients but data is scattered across sensor dashboards. | Manually screenshots dashboards, copies data into slide decks. |
| **Management** (secondary) | Needs cross-client operational visibility — which clients have open critical alerts, which sensors are healthy. | Asks analysts to compile status manually. No real-time cross-client view. |

## Scope

### In Scope

1. **Unified sensor access** — Full API coverage for CrowdStrike Falcon, Cyberint Argos, Claroty xDome, and Armis. All data access is through a single `query` tool using the AxiQL query language -- there are no per-sensor read tools. Write operations (containment, blocking, alert status updates) are per-sensor MCP tools gated behind a two-tier feature flag system (compile-time cargo features + runtime per-client TOML config).

2. **Client-aware multi-sensor management (stateless model)** — Per-client configuration mapping clients to their sensors and credentials. Read tools use a `clients` array parameter on the `query` tool; write tools use scalar `client_id`. No session-level "active client". Cross-client queries via `clients: null` ("show me all clients with critical CrowdStrike alerts").

3. **OCSF data normalization** — All sensor data normalized to OCSF v1.x via protobuf using the DynamicMessage pattern. Enables cross-sensor correlation ("show me the CrowdStrike alert and the Claroty event for the same IP on the same day for client Acme").

4. **Credential management** — Per-client, per-sensor credential storage using OS keyring (macOS Keychain, Windows Credential Vault, Linux libsecret) with encrypted file fallback. Namespaced as `(client_id, sensor_id, credential_name)`.

5. **AI-consumable response design** — All MCP tool responses designed for LLM consumption: structured JSON with `outputSchema`, provenance framing for untrusted sensor data, prompt injection defense (attacker-controlled content in hostnames/file paths/process names flows through the LLM context), and structured error messages that guide the AI toward resolution.

6. **Feature-flagged write operations** — Three-tier risk classification: read operations (no gate), reversible writes (dry-run default with preview), irreversible writes (confirmation token with expiry and 100-token active cap). Per-client capability configuration with hierarchical override (BTreeMap, most-specific-path wins, deny support). Hidden tools pattern — disabled write tools omitted from `tools/list`.

7. **Comprehensive audit trail** — Every MCP tool invocation logged with: client_id, sensor, tool name, parameters, user identity, timestamp, feature flags evaluated, and result summary. Write operations fail-closed if audit logging fails. Supports SOC 2 and ISO 27001 audit requirements.

8. **Extensible sensor adapter architecture** — Trait-based `DataSource` pattern making it straightforward to add new sensors in the future without modifying core infrastructure.

9. **Ephemeral pagination and response caching** — Pagination cursors are ephemeral per-query with automatic expiry (no persistent cursor state, no FileStore, no fingerprints). Response caching with configurable TTL, write-through invalidation, memory bounds, and LRU eviction.

10. **Credential mutation gating** — Credential updates and deletions require confirmation tokens; initial creates proceed immediately. This aligns with the interface-definitions.md `set_credential` tool schema, which returns `"created"` for new credentials and `"confirmation_required"` for updates to existing credentials.

11. **Ephemeral OCSF Query Engine** — AxiQL-style query language (filter/SQL/pipe modes) operating over an on-demand OCSF data lake materialized from live sensor APIs. Cross-sensor correlation via unified OCSF fields. Powered by Chumsky parser + DataFusion execution. Three modes: filter (`field = value AND field >= value`), SQL (`SELECT ... FROM events WHERE ... GROUP BY ... ORDER BY ... LIMIT`), pipe (`FROM source | where ... | sort ... | head N`). Auto-detects mode from first token. Security limits: 64KB query, 64 nesting depth, 32 pipe stages, 10K max materialized records, 30s timeout. Includes `explain_query` tool for dry-run inspection.

12. **Query Aliases** — Reusable, parameterized query shortcuts with global and per-client scopes. Composable (depth 3 max). Stored in TOML config alongside client definitions. MCP tools for CRUD and explanation. Per-client aliases override global aliases of the same name. Cycle detection at config load time.

### Out of Scope

- **Web UI or dashboard** — Prism is an MCP server consumed by AI agent harnesses, not a web application. Management dashboards and client reports are generated by the AI agent using Prism's tools, not by Prism itself.
- **SIEM/log storage** — Prism queries sensors in real-time; it does not store or index historical data. The existing Vector pipeline continues to handle log aggregation.
- **Automated remediation without human direction** — Prism enables write operations but only when directed by a human through the AI agent. No autonomous containment or blocking.
- **Sensor deployment or agent installation** — Prism manages data from sensors; it does not install, configure, or update the sensors themselves.
- **Custom detection rule authoring** — Prism reads detection results; it does not create or manage detection rules within sensors.

## Success Criteria

| Outcome | Metric | Target |
|---------|--------|--------|
| **Analyst adoption** | Percentage of SOC analysts using Prism daily | 100% within 3 months of deployment |
| **Efficiency gain** | Reduction in time-to-triage for multi-sensor incidents | 50%+ reduction (measured via audit trail timestamps) |
| **Client coverage** | Active clients onboarded to Prism | All active clients within 3 months |
| **Sensor coverage** | Sensor APIs with full read coverage | 4/4 sensors at launch |
| **Cross-sensor correlation** | Analysts can correlate data across sensors for the same client in a single query | Functional from day one |

## Constraints & Integration Points

- **Language:** Rust (Edition 2024, MSRV 1.85+)
- **MCP SDK:** rmcp 0.8 (pre-1.0; pin to specific version, monitor for breaking changes)
- **Transport:** stdio (Claude Code MCP integration). Single session per analyst.
- **Platforms:** Linux, macOS, Windows (cross-platform binary)
- **OCSF version:** v1.x via ocsf-proto-gen build-time library
- **Protobuf:** prost + prost-reflect for DynamicMessage pattern
- **Credential storage:** keyring-rs + AES-256-GCM encrypted file fallback
- **Configuration:** TOML files for client/sensor/credential/feature-flag config
- **Downstream integration:** Audit logs exportable via structured JSON; no xMP backward compatibility required
- **Compliance:** SOC 2 Type II and ISO 27001 alignment — comprehensive audit logging of all actions, credential encryption at rest, principle of least privilege for sensor API access
- **Security:** Prompt injection defense for attacker-controlled content in sensor data flowing through LLM context. Four-layer sanitization: structural separation, provenance framing, suspicious pattern flagging, trust-level metadata.

## Overflow Context (Reference Only)

### Architecture Foundation (from Phase 0)

Phase 0 brownfield ingest of 9 reference repos produced 203 actionable lessons (49 P0 correctness gaps, 54 P1 high-ROI patterns, 49 P2 improvements, 49 P3 intentional divergences). The recovered architecture defines an 8-crate Cargo workspace with 12 ADRs. Key architectural decisions:

- **prism-core**: Domain types, `ClientId` newtype, `ClientCapabilities`, trait definitions
- **prism-cache**: Ephemeral pagination tokens with automatic expiry, response cache with configurable TTL, LRU eviction, and write-through invalidation
- **prism-credentials**: `CredentialStore` trait with keyring + encrypted file backends, `(client_id, sensor_id, credential_name)` namespace
- **prism-ocsf**: DynamicMessage pattern from axiathon, ocsf-proto-gen as build.rs dependency, two-tier storage (flat hot fields + JSON blob)
- **prism-config**: Layered config (CLI args > env vars > TOML defaults), `_FILE` suffix for K8s secret mounts, per-client feature flag resolution with hierarchical override (BTreeMap, most-specific-path wins, deny support)
- **prism-sensors**: Generic `DataSource<T>` trait eliminating the 9x/7x code duplication found in reference pollers, per-sensor adapter implementations
- **prism-mcp**: rmcp 0.8 `#[tool_router]` + `Parameters<T>` + `JsonSchema`, conditional tool registration based on feature flags, stateless client model (client_id on every tool call, no session-level active client)
- **prism**: Binary entry point, component wiring

Implementation priority order: core → cache → credentials → ocsf → config → sensors → mcp → prism

### Sensor API Coverage

| Sensor | Auth | Read Sources | Write Operations |
|--------|------|-------------|-----------------|
| CrowdStrike Falcon | OAuth2 Client Credentials | Alerts, Detections, Hosts | Host containment, Alert status updates |
| Cyberint Argos | Cookie-based (access_token) | Alerts, Digital Assets | Alert acknowledgment, closure |
| Claroty xDome | Bearer token | Alerts, Events, Audit Logs, Devices, Vulnerabilities, Servers, Sites, Relations | Alert resolution, Device actions |
| Armis | Bearer token (static API key) | Alerts, Activities, Audit Logs, Risk Factors, Connections, Devices, Vulnerabilities | Alert status updates, Device actions |

### Reference Repos

| Repo | Role | Key Contribution to Prism |
|------|------|--------------------------|
| poller-cobra | CrowdStrike behavioral spec | OAuth2 flow, two-step fetch, cursor+fingerprint |
| poller-express | Cyberint behavioral spec | Cookie auth, alert/asset dual collection |
| poller-bear | Claroty behavioral spec | 9 data sources, dual pagination, polymorphic IDs |
| poller-coaster | Armis behavioral spec | AQL queries, 7 collectors, atomic state writes |
| tally | Rust MCP reference | rmcp 0.8 patterns, tool registration, dual CLI/MCP |
| axiathon | OCSF+protobuf reference | DynamicMessage, field aliases, query language, tenant isolation |
| ocsf-proto-gen | Build pipeline | OCSF schema → protobuf generation, type mapping |
| serveMyAPI | Credential management reference | OS keyring patterns, MCP credential CRUD |
| mcp-claroty-xdome | MCP sensor wrapping reference | Tool design for security APIs, caching, session management |

### Compliance Requirements

- **SOC 2 Type II**: All sensor API calls logged with timestamp, user, client, action, and result. Credential access logged. Feature flag evaluations logged. Audit trail immutable and exportable.
- **ISO 27001**: Credentials encrypted at rest (AES-256-GCM). Principle of least privilege enforced via feature flags. Access control via per-client capability configuration. Incident response supported via comprehensive audit trail.
- **Audit log format**: Structured JSON via `tracing` crate, compatible with existing Vector pipeline for centralized log aggregation.
