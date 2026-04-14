# Prism -- Unified Project Context

**Date:** 2026-04-13
**Phase:** 0 -- Multi-Repo Synthesis, Step 0f
**Status:** Complete
**Purpose:** Single authoritative document summarizing everything known about the Prism project. All downstream phases (brief, PRD, architecture, stories) reference this document as the definitive source of truth.

---

## Table of Contents

1. [Project Overview](#1-project-overview)
2. [Reference Repo Summary](#2-reference-repo-summary)
3. [Unified Architecture Summary](#3-unified-architecture-summary)
4. [Convention Decisions](#4-convention-decisions)
5. [Security Posture Summary](#5-security-posture-summary)
6. [Module Criticality Summary](#6-module-criticality-summary)
7. [Cross-Repo Integration Points](#7-cross-repo-integration-points)
8. [Holdout Scenario Summary](#8-holdout-scenario-summary)
9. [Key Risks and Open Questions](#9-key-risks-and-open-questions)
10. [Recommended Next Steps](#10-recommended-next-steps)

---

## 1. Project Overview

### 1.1 What Prism Is

Prism is a Rust-based MCP (Model Context Protocol) server that unifies multi-client security sensor management for 1898 & Co, a Managed Security Service Provider (MSSP). It replaces four independent Go polling services -- poller-cobra (CrowdStrike), poller-express (Cyberint), poller-bear (Claroty xDome), and poller-coaster (Armis) -- with a single multi-client-aware binary that exposes normalized security data to AI agents via MCP. Prism runs as a per-analyst MCP server in Claude Code (stdio transport) -- each MSS analyst or threat detection engineer runs their own instance. The analyst is a trusted MSSP employee who can work across all clients and sensors in a single session.

Prism is not a SIEM or storage layer. It is a complementary query and collection layer that sits alongside the existing Vector/SIEM pipeline, giving AI agents direct access to security sensor data through standardized MCP tools, resources, and prompts.

### 1.2 Who It Is For

**Primary users:**
- **MSS analysts and threat detection engineers** at 1898 & Co who use Claude Code as their AI-assisted investigation environment. Each analyst runs their own Prism instance to query, correlate, and investigate security alerts across multiple sensor platforms for all MSSP clients.
- **AI agents** (Claude Code, Claude Desktop, and other MCP-compatible clients) that need structured access to security sensor data via MCP tools.

**Primary stakeholders:**
- **1898 & Co engineering** -- responsible for building and maintaining Prism.
- **1898 & Co MSSP clients** -- the end-beneficiaries whose security data flows through Prism. Client data must not be accidentally mixed (data correctness), though the analyst may intentionally query across clients.

### 1.3 What Problem It Solves

**Current state (problems):**

1. **Four separate pollers, four deployments:** Each sensor vendor (CrowdStrike, Cyberint, Claroty, Armis) has its own Go polling service. These share 90%+ identical code for cursor management, health probes, xMP enrichment, backoff, and sink delivery -- but each reimplements it independently. poller-bear duplicates its collection loop 9 times; poller-coaster duplicates it 7 times.

2. **No AI agent access:** The Go pollers produce data for a downstream Vector pipeline, but AI agents have no way to query sensor data directly. There is no MCP interface.

3. **Single-client design with no unified view:** Each poller instance serves one MSSP client. An analyst investigating across clients must switch between N separate tool configurations. No unified cross-client querying exists.

4. **Known production bugs:** poller-cobra hardcodes MemoryStore (all cursor state lost on restart). poller-cobra updates in-memory state before persisting (cursor advances past undelivered records on failure). poller-express also lacks durable persistence.

5. **No OCSF normalization in pollers:** The pollers emit raw vendor-specific JSON. OCSF normalization, when it exists at all, is stubbed (poller-bear has OCSF types but its mapper returns nil).

**Prism's solution:**

1. **Single Rust binary** with pluggable sensor adapters behind traits. One per-analyst process replaces four separate poller deployments.
2. **MCP server** exposing 24+ tools, resources, and prompts to AI agents via rmcp 0.8 over stdio transport.
3. **Multi-client awareness** with explicit `tenant_id` per MCP tool call. The analyst can query any client or all clients (`tenant_id: null`). Client isolation is about data correctness, not adversarial security -- the analyst is trusted.
4. **OCSF normalization** using axiathon's DynamicMessage pattern with ocsf-proto-gen as a build-time library dependency.
5. **Durable cursor state** with atomic file persistence from day one, fixing the MemoryStore and state-ordering bugs.
6. **Encrypted credential management** with per-client isolation, fixing serveMyAPI's critical vulnerabilities.

### 1.4 Key Metrics

| Metric | Value |
|--------|-------|
| Reference repos analyzed | 9 |
| Languages involved | Go (4 repos), TypeScript (2 repos), Rust (3 repos) |
| Target language | Rust |
| Sensor integrations | 4 (CrowdStrike, Cyberint, Claroty xDome, Armis Centrix) |
| Data sources across sensors | 25 (1 + 2 + 9 + 7 + stubs) |
| Crates in workspace | 8 (prism-core, prism-ocsf, prism-state, prism-credentials, prism-sensors, prism-mcp, prism-config, prism) |
| Architecture layers | 8 |
| Architectural decisions (ADRs) | 11 |
| Holdout scenarios | 53 (37 P0, 16 P1) |
| Critical bugs to not inherit | 14 |
| Anti-patterns cataloged | 12 |

---

## 2. Reference Repo Summary

### 2.1 poller-cobra (CrowdStrike Falcon -- Go)

CrowdStrike Falcon API sensor poller written in Go. Implements OAuth2 Client Credentials authentication with multi-region support (us-1, us-2, eu-1, ap-1) and a two-step fetch pattern (QueryV2 for IDs, then PostEntities for details). Prism takes the CrowdStrike API behavioral contract (endpoints, auth flow, 32 alert fields, FQL filter construction), the composite cursor pattern (Timestamp + RecordID), xMP envelope format, health probes, and exponential backoff. Known bugs that Prism must not inherit: hardcoded MemoryStore despite FileStore config, in-memory state updated before persistence, response body not drained on success, health server shutdown never called, zero test coverage on business-critical paths.

### 2.2 poller-express (Cyberint Argos -- Go)

Cyberint Argos threat intelligence poller written in Go. Implements cookie-based authentication via a custom HTTP RoundTripper (injecting `access_token` cookie), CyberintTime multi-format timestamp parsing (4 formats for a single API), and two data sources (alerts with 52 AlertData subtypes, assets). Prism takes the Cyberint API behavioral contract, cookie auth middleware pattern, and CyberintTime parsing logic. Anti-patterns to avoid: MemoryStore only (no persistence), no OS signal handling, string comparison of numeric asset IDs (incorrect ordering), strict JSON decoding (DisallowUnknownFields breaks forward compatibility), and duplicate collector code for alerts and assets.

### 2.3 poller-bear (Claroty xDome -- Go)

Claroty xDome OT/IoT security poller written in Go. The most complex poller with 9 data sources (alerts, activity events, audit logs, device-alert relations, device-vulnerability relations, servers, sites, devices, vulnerabilities). Implements POST-for-read pattern (Claroty accepts POST for read operations), polymorphic JSON handling (IDs returned as string OR number), 3-tuple cursors for some sources, and atomic file persistence (temp + fsync + rename). Prism takes the Claroty API behavioral contract (all 9 endpoints), FileStore atomic write pattern, batch receipts, and the polymorphic JSON handling requirement. The 9x code duplication in its collection loop is the strongest motivator for Prism's generic DataSource trait. Known issues: Helm-config mismatch (4 env vars set by Helm but never read), no rate limiting toward Claroty API.

### 2.4 poller-coaster (Armis Centrix -- Go)

Armis Centrix OT/IoT security poller written in Go. Implements AQL (Armis Query Language) query forwarding via SDK GetSearch method, 7 data sources (alerts, activities, audit logs, risk factors, connections, devices, vulnerabilities), timestamp fallback chains (1-3 fields per source), and ID fallback chains (2-4 fields per source). Has the most mature generic design among the pollers -- its variation table pattern is the closest ancestor to Prism's planned DataSource trait. Prism takes the Armis API behavioral contract, AQL forwarding pattern, timestamp/ID fallback chains, and FileStore persistence. Known issues: inconsistent forward progress error handling (3/7 collectors use sentinel error, 4/7 use plain error), missing limit validation.

### 2.5 tally (Findings Tracker -- Rust MCP Server)

Reference Rust MCP server implementation built on rmcp 0.8. Implements 24 tools, 8 prompts, 14 resources (5 static + 9 templates), stdio transport, and a dual CLI/MCP interface. This is the canonical Rust MCP reference for Prism. Prism takes the `#[tool_router]` macro pattern, `Parameters<T>` with `JsonSchema` for type-safe input validation, `ServerHandler` trait implementation, stdio transport, error handling via `thiserror` with `#[non_exhaustive]`, `#[tracing::instrument(skip_all)]` pattern, `#![forbid(unsafe_code)]`, and `clippy::unwrap_used = deny`. Known issues: all errors mapped to `ErrorCode(-1)` (should be discriminated), O(N) load_all for point lookups, state machine enforcement is caller-level (not enforced by setters).

### 2.6 serveMyAPI (Credential Manager -- TypeScript MCP Server)

MCP server for OS keyring-based API key management written in TypeScript. Implements 4 tools for credential CRUD operations. Prism takes the credential management domain model (store, get, delete, list with name index), OS keyring abstraction pattern (keyring-rs equivalent of keytar), macOS permission pre-auth pattern (probe keyring at startup to consolidate permission prompts), and the Docker fallback concept (encrypted file backend replaces plaintext). CRITICAL vulnerabilities that Prism must NOT inherit: path traversal on key names (zero sanitization), plaintext credential files with chmod 777, no access control on MCP endpoints, no audit trail, `Date.now()` session IDs (collision risk), unstructured string errors, zero test coverage.

### 2.7 axiathon (Security Lake / SIEM -- Rust)

OCSF-based security event normalization and storage platform written in Rust. The most architecturally significant reference for Prism. Prism takes the DynamicMessage OCSF wrapping pattern (runtime field access across all 83 event classes), four-tier field resolution (Prism fields -> proto descriptor -> unmapped JSON -> None), three-tier field alias resolution (analyst shortcut -> canonical -> OCSF path), TenantFilterRule for optimizer-level tenant isolation, two-tier columnar storage concept (hot flat columns + event_data JSON), AES-256-GCM vault concept (properly implemented with unique salts, not hardcoded), CWE-referenced parser security limits, per-tenant file isolation, and `SECURITY(CWE-xxx)` comment convention. CRITICAL vulnerabilities to fix: hardcoded vault passphrase, static Argon2 salt, permissive CORS, unprotected admin endpoints, in-memory stores for alerts/cases, public fields on domain types (78 + 93 call sites).

### 2.8 ocsf-proto-gen (Proto Generator -- Rust)

OCSF JSON schema to protobuf definition generator written in Rust. Prism consumes this as a direct build-time library dependency in `build.rs` (the only actual code-level dependency between reference repos). Prism takes the OCSF type mapping table (24 type conversions including `timestamp_t -> int64 epoch ms`, `json_t -> string`, `float_t -> double`), proto package hierarchy (`ocsf.<version>.<category>`), enum value map generation (`enum-value-map.json` for runtime integer-to-caption lookup), deprecated field skipping, and deterministic output via BTreeMap/BTreeSet. Used with `default-features = false` to prevent network access during builds. The committed OCSF JSON schema files feed the generation pipeline.

### 2.9 mcp-claroty-xdome (Claroty MCP Server -- TypeScript)

TypeScript MCP server wrapping the Claroty xDome sensor API. Implements the ToolHandler -> Service -> ApiClient layering pattern that Prism adapts to Rust trait-based DI. Prism takes the sensor-wrapping architecture pattern, per-service cache isolation, typed error hierarchy mapped to JSON-RPC 2.0 codes (10 distinct codes), Zod schema validation pattern (schemars equivalent in Rust), and the SSE/Streamable HTTP transport patterns. Known issues: unbounded in-memory caches (all 5 service caches grow without limit), no session expiration, CORS wildcard in production, `z.any()` for filter values (untyped), Express body size conflict.

---

## 3. Unified Architecture Summary

### 3.1 Architecture Philosophy

Prism is a single Rust binary (ADR-001) with 8 crates in a Cargo workspace. It follows a layered architecture where each layer has a clear responsibility and well-defined dependency direction. The architecture is driven by three forces:

1. **Behavioral fidelity** -- Prism must replicate the exact polling behavior of the 4 Go pollers (same API contracts, same cursor semantics, same xMP envelope format) while fixing known bugs.
2. **Multi-client data correctness** -- Every layer must correctly scope data by client. The pollers were single-client; Prism is multi-client-aware, with explicit `tenant_id` per tool call. Isolation is a correctness concern (the analyst is trusted), not an adversarial security boundary.
3. **AI agent accessibility** -- The MCP layer makes sensor data queryable by AI agents, a capability that does not exist in the current poller architecture.

### 3.2 The 8-Layer Architecture

| Layer | Crate | Responsibility | Primary Reference |
|-------|-------|---------------|-------------------|
| **L1: MCP Transport** | prism-mcp | JSON-RPC 2.0 protocol, stdio transport | tally (rmcp 0.8) |
| **L2: MCP Tools/Prompts/Resources** | prism-mcp | Tool registration via `tool_router`, `Parameters<T>` input validation, resources, prompts | tally + mcp-claroty-xdome |
| **L3: Sensor Adapters** | prism-sensors | Generic `DataSource<T>` trait + 4 sensor implementations, per-sensor auth | All 4 Go pollers |
| **L4: OCSF Normalization** | prism-ocsf | DynamicMessage wrapping, proto build pipeline, type mapping, xMP enrichment | axiathon + ocsf-proto-gen |
| **L5: State Management** | prism-state | Cursor trait (PartialOrd, forward-only), FileStore (atomic write), query fingerprint, batch receipts | poller-bear + poller-coaster |
| **L6: Credential Management** | prism-credentials | CredentialStore trait, keyring + encrypted file backends, per-tenant isolation | serveMyAPI + axiathon |
| **L7: Configuration** | prism-config | Layered config (CLI > env > file > defaults), `_FILE` secret resolution, dry-run validation | All pollers + tally |
| **L8: Observability** | Cross-cutting | tracing + tracing-subscriber JSON, OpenTelemetry metrics, `skip_all` instrumentation | tally |

### 3.3 The 8-Crate Workspace

```
prism/                          -- workspace root
  Cargo.toml                    -- workspace manifest
  build.rs                      -- OCSF proto generation via ocsf-proto-gen + prost-build

  crates/
    prism-core/                 -- domain types, error types, shared traits (zero I/O)
    prism-config/               -- layered config, env var resolution, dry-run validation
    prism-credentials/          -- CredentialStore trait + keyring and encrypted-file backends
    prism-state/                -- Cursor trait, FileStore, fingerprint, batch receipts
    prism-ocsf/                 -- DynamicMessage wrapper, OCSF normalizer, xMP enrichment
    prism-sensors/              -- SensorAdapter trait + 4 implementations
    prism-mcp/                  -- MCP server, tools, prompts, resources, transport
    prism/                      -- binary entry point (thin: wires dependencies, tokio::main)
```

**Dependency graph (build order):**

```
prism-core            (no Prism dependencies)
    |
    +-- prism-ocsf    (depends on: prism-core + ocsf-proto-gen [build dep])
    |
    +-- prism-state   (depends on: prism-core)
    |
    +-- prism-credentials (depends on: prism-core)
    |
    +-- prism-config  (depends on: prism-core)
    |
    +-- prism-sink    (depends on: prism-core, prism-ocsf)
    |
    +-- prism-sensors (depends on: prism-core, prism-ocsf, prism-state, prism-credentials)
    |
    +-- prism-mcp     (depends on: prism-core, prism-sensors, prism-state, prism-credentials)
    |
    +-- prism (binary)(depends on: all above + prism-config)
```

### 3.4 Key Architectural Decisions (ADRs)

Eleven ADRs were produced during Phase 0 synthesis. The most significant:

| ADR | Decision | Rationale |
|-----|----------|-----------|
| ADR-001 | Single Rust binary, not microservices | Eliminates 4x deployment overhead. Shared infra is DRY. tally proves the pattern. |
| ADR-002 | rmcp 0.8 as MCP library | Only Rust MCP reference available. tally's patterns are production-tested. |
| ADR-003 | DynamicMessage for OCSF (not typed enums) | 83 OCSF event classes; typed enums don't scale. axiathon's pattern enables runtime field access. |
| ADR-004 | ocsf-proto-gen as build-time library | No network at build time. Inherits ocsf-proto-gen's correctness fixes. |
| ADR-005 | Durable cursor persistence from day one | Fixes poller-cobra/express MemoryStore bug. MemoryStore panics in production mode. |
| ADR-006 | Generic DataSource trait | Eliminates 9x/7x/2x duplication. Bug fixes apply to all sensors simultaneously. |
| ADR-007 | Encrypted credential file backend | Fixes serveMyAPI's CRITICAL plaintext storage. AES-256-GCM with external key. |
| ADR-008 | Unified PRISM_ env var prefix | Fixes poller-bear's 5-prefix chaos. Single namespace for all config. |
| ADR-009 | tracing + OpenTelemetry from day one | All 9 repos lack metrics. Observability is a correctness requirement. |
| ADR-010 | Per-sensor auth middleware (sealed trait) | Prevents cross-sensor credential leakage. 4 different auth mechanisms. |
| ADR-011 | LRU-bounded caches with per-tenant isolation | Fixes mcp-claroty-xdome's unbounded caches. Memory usage is bounded. |

### 3.5 Deployment Topology

**Type:** Per-analyst local process, stdio transport.

- **Primary deployment:** Each MSS analyst runs Prism locally as a Claude Code MCP server. One analyst, one process.
- **Transport:** stdio only. No network-facing MCP transport (SSE/HTTP) in initial scope.
- **Configuration:** TOML config file defining all MSSP clients and their sensor connections. Credentials via OS keyring or encrypted file backend.
- **State:** Cursor state persisted to local filesystem (e.g., `~/.local/share/prism/state/`).
- **Health:** Health server available for local diagnostics but not required for K8s orchestration in the per-analyst model.

### 3.6 Data Flow Summary

**Sensor poll to Vector pipeline:**

```
Sensor API -> SensorAdapter -> DataSource.fetch_page() -> validate forward progress
  -> OcsfNormalizer.normalize() -> DynamicMessage -> xMP enrichment -> EnrichedPayload
  -> HTTP POST (batch, basic auth) -> Vector -> SIEM
  -> CursorStore.save() (AFTER successful delivery)
```

**MCP tool call to sensor data:**

```
AI Agent -> MCP Transport (rmcp JSON-RPC) -> tool_router dispatch
  -> Tool Handler (validate Parameters<T>) -> Sensor Service
  -> LRU Cache lookup (per-tenant) -> SensorAdapter.query()
  -> format as CallToolResult -> MCP response -> AI Agent
```

**Credential resolution chain:**

```
PRISM_{SENSOR}_TOKEN_FILE (K8s secret mount)
  -> fallback: PRISM_{SENSOR}_TOKEN (env var)
  -> fallback: keyring-rs (OS keyring)
  -> fallback: ConfigError::MissingSecret
```

---

## 4. Convention Decisions

The 9 reference repos span 3 languages (Go, TypeScript, Rust) with conflicting conventions. All conflicts were resolved in favor of Rust-idiomatic patterns. The key decisions:

### 4.1 Naming Conventions

| Element | Convention | Rationale |
|---------|-----------|-----------|
| Type names | `PascalCase` | Rust convention. Universal across all 9 repos. |
| Functions/methods | `snake_case` | Rust convention. Enforced by compiler warning. |
| Variables | `snake_case` | Rust convention. |
| File names | `snake_case.rs` | Matches axiathon, ocsf-proto-gen, tally. |
| Crate names | `kebab-case` | Rust convention. Matches `axiathon-core`, `tally-ng`. |
| MCP tool names | `snake_case` | Matches tally and mcp-claroty-xdome. AI-agent-friendly. |
| JSON wire format | `snake_case` via `#[serde(rename_all = "snake_case")]` | Matches all Go pollers' wire format. Preserves Vector compatibility. |
| Env var names | `PRISM_{DOMAIN}_{FIELD}` | Single prefix fixes poller-bear's 5-prefix chaos. |
| Test function names | `{subject}_{action}_{expected_outcome}` | Matches tally and axiathon. Descriptive. |
| Error type variants | `PascalCase` enum variants | Rust convention. |
| MCP input DTOs | `{ToolName}Input` suffix | Matches tally: `RecordFindingInput`. |

### 4.2 Error Handling

**Decision:** Use `thiserror` with structured, actionable error variants following tally's pattern.

Key conventions:
- `#[non_exhaustive]` on all error enums (from tally + axiathon) -- prevents breaking downstream match arms.
- Structured fields on variants (from tally's `InvalidTransition { from, to, valid }` pattern) -- enables programmatic handling.
- Actionable messages: "delete state file to reset", "run `prism init`" (from tally) -- guides operators.
- Per-crate `Result<T>` type alias (from axiathon): `pub type Result<T> = std::result::Result<T, PrismError>;`
- Centralized `From<PrismError> for McpError` mapping (improves on tally's distributed `to_mcp_err()`).
- Config validation uses multi-error aggregation: `Result<(), Vec<ConfigError>>` (from Go pollers' `errors.Join()`).

**MCP error code mapping** (adopts mcp-claroty-xdome's code taxonomy, applied to Rust error variants):
- `PrismError::InvalidInput` -> `ErrorCode::INVALID_REQUEST`
- `PrismError::StateNotFound` -> `ErrorCode::INVALID_REQUEST`
- `PrismError::Config` -> `ErrorCode(-32009)`
- Everything else -> `ErrorCode::INTERNAL_ERROR`

### 4.3 Logging and Observability

**Decision:** Use `tracing` + `tracing-subscriber` with JSON output, adopting tally's instrumentation pattern.

Key conventions:
- `skip_all` always (from tally) -- never dump full struct contents into spans.
- Cherry-pick relevant fields: `sensor`, `tenant`, `cursor`, `record_count`, `duration`.
- stderr for all diagnostics (stdout reserved for MCP JSON-RPC).
- JSON format for production, pretty format for development.
- Level from env: `PRISM_LOG_LEVEL=info` (default).
- OpenTelemetry metrics from day one (absent from ALL 9 reference repos):
  - `prism_records_collected_total{sensor, source, tenant}`
  - `prism_records_delivered_total{sensor, source, tenant}`
  - `prism_collection_duration_seconds{sensor, source}`
  - `prism_cursor_lag_seconds{sensor, source}`
  - `prism_retry_count_total{sensor, source}`
  - `prism_cache_hit_ratio{sensor, tenant}`
  - `prism_active_sessions`

### 4.4 Configuration

**Decision:** Layered config with precedence: CLI args > env vars > config file > compiled defaults.

Key conventions:
- `clap::Parser` derive with `#[arg(env = "PRISM_...")]` for unified CLI + env var parsing.
- `_FILE` suffix for all credentials (K8s-native pattern from all 4 Go pollers).
- `resolve_secret(file_env, direct_env) -> Result<String>` utility. File takes precedence.
- Uniform duration parsing via `humantime` crate (`30s`, `5m`, `1h`).
- `--dry-run` flag validates full configuration, prints redacted secrets, exits.
- Multi-error validation so operators see all problems in one pass.

### 4.5 Testing Strategy

**Decision:** Four-tier test strategy (adopt from tally, enhanced with Go patterns).

| Tier | Framework | Location | Purpose |
|------|-----------|----------|---------|
| Unit | `#[test]` | `#[cfg(test)] mod tests` in source file | Single function behavior |
| Property | `proptest` | `tests/property_*.rs` | Invariants over random input |
| Integration | `#[tokio::test]` + wiremock | `tests/integration_*.rs` | API client + state store |
| E2E | assert_cmd or in-process | `tests/e2e_*.rs` | Full MCP tool workflow |

Additional conventions:
- Fakes over mocks (from Go pollers + tally). Use mockall only for complex interfaces.
- Table-driven tests translated to Rust vec-of-tuples pattern.
- Golden file testing with `insta` crate for OCSF mapping validation.
- Benchmark testing with `criterion` for hot paths (cursor comparison, fingerprint computation).
- Test naming: `{subject}_{action}_{expected_outcome}` -- no `test_` prefix.

### 4.6 State Management

**Decision:** Trait-based state store with file-backed default. MemoryStore for tests only.

Key conventions:
- Atomic write: temp file -> fsync -> rename (from poller-bear/poller-coaster).
- State persisted BEFORE in-memory cursor is updated (fixes poller-cobra's ordering bug).
- Forward progress enforced via typed error variant (fixes poller-coaster's inconsistency).
- All persisted data includes `schema_version` field and `#[serde(default)]` on all fields (from tally).
- `QueryFingerprint` mismatch at startup is fatal with actionable message.

### 4.7 Authentication

**Decision:** Per-sensor auth trait with `secrecy::SecretString` for all credential values.

| Sensor | Auth Implementation |
|--------|-------------------|
| CrowdStrike | `OAuth2ClientCredentials` -- auto-refresh via oauth2 crate |
| Cyberint | `CookieAuth` -- static cookie injection via reqwest middleware |
| Claroty | `BearerTokenAuth` -- static bearer header |
| Armis | `BearerTokenAuth` -- static bearer header |
| Vector sink | `BasicAuth` -- username + password |

The `SensorAuth` trait is sealed (not implementable outside prism-sensors) to prevent accidental composition that could leak credentials across sensor types.

### 4.8 API Versioning

**Decision:** MCP tools are NOT versioned in their names. The MCP protocol handles capability negotiation.

- Additive changes (new optional params, new response fields) are non-breaking. Use `Option<T>` and `#[serde(default)]`.
- Removals or semantic changes require a new tool name with deprecation of the old one.
- Sensor API versions are per-adapter config (each adapter owns its API version).
- OCSF version is embedded in proto package path. Single OCSF version per Prism release.
- Proto field numbers are NOT stable across OCSF versions -- never mix proto data across versions.

---

## 5. Security Posture Summary

### 5.1 Vulnerability Inventory

Across 9 reference repos, the unified security analysis identified:

| Severity | Count | Key Examples |
|----------|-------|-------------|
| **CRITICAL** | 6 | serveMyAPI path traversal, serveMyAPI plaintext storage, axiathon hardcoded vault passphrase, axiathon static Argon2 salt |
| **HIGH** | 12 | poller-cobra MemoryStore, poller-cobra state-before-persistence, serveMyAPI Date.now session IDs, mcp-claroty-xdome unbounded caches, axiathon permissive CORS |
| **MEDIUM** | 18 | poller-express no signal handling, poller-bear no rate limiting, tally state machine bypass, multiple repos' unbounded maps |
| **LOW** | 8 | Dead sentinel errors, ocsf-proto-gen path in CLI, partial output cleanup |

### 5.2 Top Security Concerns for Prism

**Concern 1: Credential Exposure Chain (CRITICAL)**

When serveMyAPI's plaintext file storage pattern combines with the pollers' credential loading, the entire credential chain is compromised. An attacker could harvest API keys for CrowdStrike, Cyberint, Claroty, and Armis simultaneously. Prism must use encrypted file storage (AES-256-GCM with external key management), input sanitization on credential names, and per-tenant credential isolation.

**Concern 2: Multi-Client Data Mixing (HIGH)**

The pollers are all single-client. Prism is multi-client-aware, handling data for all MSSP clients in a single per-analyst process. Without proper client scoping, Client A's cursor state could be confused with Client B's, or cached API responses could be returned for the wrong client. This is a data correctness concern, not an adversarial security concern -- the analyst is trusted.

**Concern 3: Resource Exhaustion via Unbounded Data Structures (MEDIUM)**

Multiple repos share unbounded resource patterns: in-memory caches, query sizes. In a per-analyst process handling many clients over long sessions, these can cause memory exhaustion. Blast radius is limited to the single analyst's process.

**Concern 4: State Corruption on Restart (HIGH)**

poller-cobra and poller-express use in-memory-only state. poller-cobra updates state before persistence. These patterns cause full re-fetch, duplicate data, and sensor API quota exhaustion.

### 5.3 Client Data Correctness Model

Simplified from axiathon's 9-layer adversarial isolation model. Since Prism is a per-analyst tool operated by a trusted MSSP employee, the focus is on preventing accidental data mixing, not defending against adversarial tenants:

| Layer | Mechanism | Purpose |
|-------|-----------|---------|
| L1 | **TenantId newtype** | Compile-time prevention of accidental string mixing. Private field, validated constructor. |
| L2 | **Per-client credential store** | Credentials keyed by `(tenant_id, sensor_type)`. Prevents using wrong client's token. |
| L3 | **Per-client cursor state** | Each client's cursors stored independently. Per-client directory isolation. |
| L4 | **Per-client cache isolation** | Cache instances per `(tenant_id, sensor_type)`. Independent TTL and bounds. |
| L5 | **Log context** | Structured logging with `tenant_id` as required span field. Credential values never logged. |
| L6 | **Explicit tenant_id per tool call** | Every MCP tool carries `tenant_id` parameter. `tenant_id: null` means "all clients". No session-level implicit tenant binding. |

### 5.4 Data Classification

| Level | Definition | Examples |
|-------|-----------|---------|
| **CRITICAL** | Credentials granting external system access | API keys, OAuth2 secrets, bearer tokens |
| **HIGH** | Client-identifying metadata | Client IDs, sensor URLs, customer subdomains, tenant IDs |
| **MEDIUM** | Security event data | CrowdStrike alerts, Cyberint threat intel, Claroty OT alerts, Armis device inventory |
| **LOW** | Operational metadata | Cursor positions, fingerprints, health status, cache stats |

### 5.5 Security Priority Matrix

| Priority | Count | Theme |
|----------|-------|-------|
| **P0** | 6 | Credential encryption, tenant isolation, auth type safety, durable state, TLS 1.2+, secret redaction |
| **P1** | 6 | Resource bounds, credential rotation, startup validation, graceful shutdown, error hierarchy, audit trail |
| **P2** | 6 | Container hardening, supply chain, rate limiting, input validation, security annotations, connection pooling |

**P0 items (must have before MVP):**
1. P0-SEC-001: Encrypted credential store with per-tenant isolation
2. P0-SEC-002: TenantId newtype with validated constructor
3. P0-SEC-003: Per-sensor auth middleware with strict type safety
4. P0-SEC-004: Durable state persistence with atomic writes
5. P0-SEC-005: TLS 1.2+ on all HTTP connections
6. P0-SEC-006: Secret redaction in logs and errors

### 5.6 Critical Vulnerabilities That Prism Must NOT Inherit

1. serveMyAPI's plaintext file storage and path traversal (CRITICAL)
2. axiathon's hardcoded vault passphrase and static salt (CRITICAL)
3. poller-cobra's state-before-persistence ordering bug (HIGH)
4. mcp-claroty-xdome's unbounded caches and sessions (HIGH)
5. poller-express's missing signal handling (MEDIUM)
6. All pollers' single-client architecture lacks multi-client data correctness (HIGH)

---

## 6. Module Criticality Summary

### 6.1 Tier Definitions

| Tier | Kill Rate Target | Failure Consequence |
|------|-----------------|---------------------|
| **CRITICAL** | >= 95% | Data loss, security breach, or total system failure. No workaround. |
| **HIGH** | >= 90% | Degraded service or materially incorrect results. |
| **MEDIUM** | >= 80% | Operational inconvenience; no data loss or security impact. |
| **LOW** | >= 70% | Cosmetic, logged-only, or has immediate workarounds. |

### 6.2 Classification Matrix

| Module | Tier | Blast Radius | Security Sensitivity | Complexity | Key Risk |
|--------|------|-------------|---------------------|------------|----------|
| **prism-core** | CRITICAL | All 8 modules | HIGH (TenantId data correctness) | MEDIUM | Bug propagates everywhere; TenantId bypass = data mixing |
| **prism-ocsf** | CRITICAL | Silent data corruption | MEDIUM (event content) | HIGH | Incorrect normalization undetectable downstream |
| **prism-state** | CRITICAL | Data duplication/loss/loops | LOW (metadata) | HIGH | Forward progress invariant is a safety property |
| **prism-credentials** | CRITICAL | Full credential exposure | CRITICAL (API keys) | MEDIUM-HIGH | Holds API keys for all 4 sensor APIs for all clients |
| **prism-sensors** | HIGH | Per-sensor blindness | MEDIUM (event content) | HIGH | 4 sensor APIs, generic DataSource design |
| **prism-mcp** | HIGH | AI agent access loss | LOW (stdio, trusted analyst) | HIGH | 24+ tools, tenant_id routing |
| **prism-config** | MEDIUM | Startup failure | MEDIUM (secret paths) | MEDIUM | 30+ env vars, secret resolution |
| **prism-sink** | MEDIUM | Missed event delivery | LOW (normalized records) | MEDIUM | Batch delivery, xMP format |
| **prism** | LOW | Crash/unclean shutdown | LOW (no domain logic) | LOW | Signal handling, wiring |

### 6.3 Implementation Priority Order

| Priority | Module | Rationale |
|----------|--------|-----------|
| 1 | **prism-core** | Foundation for everything. TenantId is the root safety mechanism. |
| 2 | **prism-state** | Fix cobra/express bugs at design stage. Cursor correctness cannot be retrofitted. |
| 3 | **prism-credentials** | Build before sensors to avoid replicating serveMyAPI's CRITICAL vulns. Security-first. |
| 4 | **prism-ocsf** | Build-time pipeline dependency. Must work before sensors can normalize. HIGH complexity. |
| 5 | **prism-config** | Must be resolved before sensors or MCP can instantiate. |
| 6 | **prism-sensors** | Highest behavioral complexity. Built after all dependencies are proven. |
| 7 | **prism-sink** | Downstream from sensors. Batch delivery improvement. |
| 8 | **prism-mcp** | MCP surface built after sensors provide data to expose. |
| 9 | **prism** | Binary wiring. Built last. |

### 6.4 Cross-Cutting Constraints for CRITICAL Modules

1. `#![forbid(unsafe_code)]` -- no unsafe blocks
2. `clippy::unwrap_used = deny` -- no `.unwrap()` calls
3. Private fields with getter methods -- no `pub` fields on domain types
4. `#[non_exhaustive]` error enums -- all error types
5. Security limits on all parsers -- max length, max nesting, CWE citations
6. No `anyhow::Error` as catch-all variant -- typed error variants only

### 6.5 Anti-Patterns to Explicitly NOT Port

| Anti-Pattern | Source | Prism Constraint |
|-------------|--------|-----------------|
| MemoryStore as production backend | cobra, express | FileStore from day one; MemoryStore for tests only |
| State updated before persistence | cobra | Update in-memory ONLY after successful `store.save()` |
| Per-record HTTP POST to sink | all 4 pollers | Batch delivery with NDJSON |
| Plaintext credential storage | serveMyAPI | AES-256-GCM encryption; no plaintext at rest |
| Path traversal on credential keys | serveMyAPI | Sanitize at service boundary: `[a-zA-Z0-9_.-]+` |
| Hardcoded vault passphrase/static salt | axiathon | External key management; unique random salt per credential |
| `Date.now()` session IDs | serveMyAPI | UUID v4 or CSPRNG-based session IDs |
| Unbounded per-IP rate limiter maps | all pollers, mcp-claroty-xdome | LRU eviction with configurable max entries |
| Public struct fields on domain types | axiathon spike | Private fields with constructor + getters from day one |
| N-way code duplication (7x, 9x) | bear, coaster | Generic `DataSource` trait |
| `(error as Error).message` casts | serveMyAPI | Typed `thiserror` variants throughout |
| Dead/unused sentinel errors | all pollers | Define error variants only when used |

---

## 7. Cross-Repo Integration Points

### 7.1 How Patterns Combine in Prism

Prism is the convergence point for patterns from 9 independent repos. This section maps how those patterns integrate.

#### Sensor Polling Pipeline (from pollers -> prism-sensors + prism-state + prism-ocsf)

The 4 Go pollers each implement a 12-step collection algorithm independently. Prism unifies this into a single generic implementation:

1. **Load cursor** (from prism-state, pattern: poller-bear/coaster FileStore)
2. **Validate fingerprint** (from prism-state, pattern: all pollers SHA-256)
3. **Fetch page** (from prism-sensors, pattern: per-sensor API contract)
4. **Sort records** (from prism-sensors, pattern: per-sensor timestamp ordering)
5. **Filter duplicates** (from prism-sensors, pattern: cursor comparison)
6. **Extract cursor from last record** (from prism-sensors, pattern: per-sensor cursor extraction with fallback chains)
7. **Validate forward progress** (from prism-state, pattern: all pollers, unified error handling)
8. **Normalize to OCSF** (from prism-ocsf, pattern: axiathon DynamicMessage)
9. **Wrap in xMP envelope** (from prism-ocsf, pattern: all pollers identical format)
10. **Deliver batch to sink** (from prism-sink, improved: batch instead of per-record)
11. **Persist cursor** (from prism-state, pattern: atomic write, AFTER delivery)
12. **Append batch receipt** (from prism-state, pattern: poller-bear/coaster)

Each sensor adapter provides the sensor-specific parts (steps 3-6) while the generic infrastructure handles the rest. This eliminates the 9x/7x/2x duplication.

#### MCP Server (from tally + mcp-claroty-xdome -> prism-mcp)

The MCP layer combines patterns from two repos:

| Component | tally Pattern | mcp-claroty-xdome Pattern | Prism Integration |
|-----------|--------------|--------------------------|-------------------|
| Tool registration | `#[tool_router]` macro | Manual per-transport | Use rmcp `tool_router` (tally) |
| Input validation | `Parameters<T>` + `JsonSchema` | Zod schemas | Use `Parameters<T>` + `schemars` (tally) |
| Tool architecture | Flat handler functions | ToolHandler -> Service -> ApiClient | Rust trait-based DI version of mcp-claroty-xdome's layering |
| Error mapping | `to_mcp_err()` -> ErrorCode(-1) | 10 distinct JSON-RPC codes | Centralized `From<PrismError>` with mcp-claroty-xdome's code taxonomy |
| Transport | stdio only | stdio + HTTP/SSE + Streamable HTTP | stdio only (per-analyst model) |
| Session management | None (stdio) | UUID sessions, no expiration | None needed (stdio transport, per-analyst model) |
| Resources | 14 (5 static + 9 templates) | None (TS) / 5 (Python) | URI templates: `prism://sensor/{sensor_id}/...` |
| Prompts | 8 | None | Triage, summarization, per-sensor investigation |

#### Credential Management (from serveMyAPI + axiathon + all pollers -> prism-credentials)

Three distinct credential patterns merge:

1. **serveMyAPI:** OS keyring abstraction + name index (fixes: path traversal, plaintext, no access control)
2. **axiathon:** AES-256-GCM vault concept (fixes: hardcoded passphrase, static salt)
3. **All pollers:** `_FILE` env var pattern for K8s secret mounts (adopted as-is)

Prism's `CredentialStore` trait combines all three with per-tenant isolation that exists in none of the originals.

#### OCSF Normalization (from axiathon + ocsf-proto-gen -> prism-ocsf)

Two repos provide complementary OCSF capabilities:

- **ocsf-proto-gen:** Build-time pipeline (OCSF JSON -> .proto files -> prost-build -> Rust types). Provides the type mapping table, proto package hierarchy, and enum value map.
- **axiathon:** Runtime pattern (DynamicMessage wrapping, four-tier field resolution, table-per-class routing). Provides the runtime architecture for working with OCSF events.

The build pipeline from ocsf-proto-gen feeds into the runtime architecture from axiathon. This is the only direct code dependency between reference repos.

### 7.2 Shared Data Format Dependencies

| Format | Producers | Consumers | Contract |
|--------|-----------|-----------|----------|
| **xMP Envelope** | prism-ocsf (enrichment) | Vector (downstream) | `{"data": ..., "record_type": "<sensor>_<entity>", "xmp": {...}, "ocsf": ...}` |
| **OCSF Proto** | ocsf-proto-gen (definitions) | prism-ocsf (runtime) | Proto3 messages per OCSF class, `ocsf.<version>.<category>` hierarchy |
| **Cursor State JSON** | prism-state (persistence) | prism-state (recovery) | Atomic JSON with composite cursors, fingerprints, receipts |
| **Credential Entries** | prism-credentials (storage) | prism-sensors (auth injection) | Per-tenant, per-sensor credential resolution |

### 7.3 Cross-Repo Inconsistencies Resolved by Prism

| Inconsistency | Repos | Prism Resolution |
|--------------|-------|-----------------|
| Health port (7321 vs 7322) | bear vs cobra/express/coaster | Single configurable `PRISM_HEALTH_ADDR` |
| State persistence (memory vs file) | cobra/express vs bear/coaster | Durable FileStore from day one |
| Record type prefix | cobra: none, coaster: `armis_` | Standardized `<sensor>_<entity>` for all |
| Error handling (sentinel vs plain) | cobra: sentinel, coaster: mixed | Uniform `thiserror` enum |
| Cursor arity (2-tuple vs 3-tuple) | cobra/express vs some bear sources | Generic cursor trait with variable arity |
| Auth mechanism | OAuth2, cookie, bearer, SDK | Per-sensor auth strategy behind sealed trait |
| OCSF support | bear: stub, axiathon: DynamicMessage | DynamicMessage approach; no stubs |
| Log framework | charmbracelet/log vs tracing | tracing + tracing-subscriber uniformly |
| Config prefix (5 different schemes) | poller-bear | Single `PRISM_` prefix |
| Health probes default | all: disabled | Enabled by default |

---

## 8. Holdout Scenario Summary

### 8.1 Overview

53 holdout scenarios across 8 groups were produced to validate that Prism correctly integrates patterns from all 9 reference repos. These scenarios are designed to catch regressions, verify bug fixes, and validate cross-repo pattern integration.

| Metric | Value |
|--------|-------|
| Total scenarios | 53 |
| P0 scenarios (must PASS) | 37 |
| P1 scenarios (must be at least PARTIAL) | 16 |
| Scenario groups | 8 |
| Repos covered | 9/9 |
| Critical bugs verified as fixed | 14 |

### 8.2 Scenario Groups

| Group | Title | Scenarios | Priority | Key Risk |
|-------|-------|-----------|----------|----------|
| **HS-001** | Happy Path | 6 | P0 | Basic MCP-to-sensor-to-OCSF pipeline per sensor |
| **HS-002** | Multi-Sensor | 5 | P0 | Cross-sensor consistency and independence |
| **HS-003** | Multi-Client Data Correctness | 7 | P0 | MSSP client data correctness (no accidental mixing) |
| **HS-004** | Credential Lifecycle | 6 | P0 | Per-tenant credential CRUD and rotation |
| **HS-005** | Failure Scenarios | 7 | P0 | Sensor down, auth expired, rate limited, timeout, malformed response |
| **HS-006** | State Recovery | 6 | P0 | Restart resilience, cursor forward progress, fingerprint detection |
| **HS-007** | Cross-Repo Failure | 8 | P1 | Patterns from one repo failing in unified context |
| **HS-008** | Contract Violation | 8 | P1 | OCSF/proto/API schema mismatches |

### 8.3 P0 Scenario Highlights

**HS-001 (Happy Path):** Validates that each sensor can be queried via MCP, returns OCSF-normalized data, and produces correct xMP envelopes. One scenario per sensor plus cross-sensor scenarios.

**HS-003 (Multi-Client Data Correctness):** The most correctness-critical group. Validates client data separation under normal operation (HS-003-01), explicit tenant_id routing correctness (HS-003-02), cache isolation between clients (HS-003-03), cursor state isolation (HS-003-04), error message client context correctness (HS-003-05), cross-client query aggregation via tenant_id: null (HS-003-06), and log context correctness (HS-003-07). Note: "spoofing prevention" is not applicable since the analyst is trusted -- the focus is on correctness of the tenant_id routing logic.

**HS-006 (State Recovery):** Validates that Prism survives restarts with cursor resume (HS-006-01), recovers from crashes via atomic state files (HS-006-02), detects config changes via query fingerprint (HS-006-03), prevents cursor regression (HS-006-04), preserves batch receipt audit trails (HS-006-05), and recovers all tenants after system-wide restart (HS-006-06).

### 8.4 Critical Bug Verification

These scenarios specifically validate that known bugs from reference repos are NOT inherited:

| Bug | Source | Severity | Verified By |
|-----|--------|----------|-------------|
| MemoryStore hardcoded | poller-cobra | HIGH | HS-007-01 |
| State updated before persistence | poller-cobra | HIGH | HS-007-03, HS-006-02 |
| DisallowUnknownFields breaks forward compat | poller-express | MEDIUM | HS-007-04 |
| No signal handling | poller-express | MEDIUM | HS-006-06 |
| String comparison of numeric IDs | poller-express | MEDIUM | HS-007-05 |
| Inconsistent forward progress errors | poller-coaster | MEDIUM | HS-006-04 |
| Path traversal in credential store | serveMyAPI | CRITICAL | HS-007-06 |
| Plaintext credential storage | serveMyAPI | CRITICAL | HS-004-01 |
| Session ID collision (Date.now) | serveMyAPI | HIGH | HS-005-07 |
| Hardcoded vault passphrase | axiathon | CRITICAL | HS-004-01 |
| Static Argon2 salt | axiathon | HIGH | HS-004-01 |
| Unbounded caches/sessions | mcp-claroty-xdome | HIGH | HS-007-08 |
| ErrorCode(-1) for all errors | tally | MEDIUM | HS-007-07 |
| Health server shutdown never called | poller-cobra | MEDIUM | HS-006-06 |

### 8.5 Repo Coverage

All 9 repos are covered by the holdout scenarios:

| Repo | Groups Covered | Coverage |
|------|---------------|----------|
| poller-cobra | 8/8 | Full |
| poller-bear | 8/8 | Full |
| poller-coaster | 8/8 | Full |
| poller-express | 7/8 | Missing HS-008 |
| tally | 7/8 | Missing HS-006 |
| axiathon | 7/8 | Missing HS-006 |
| mcp-claroty-xdome | 6/8 | Missing HS-004, HS-006 |
| ocsf-proto-gen | 4/8 | Focused on OCSF-specific groups |
| serveMyAPI | 2/8 | Focused on credential-specific groups |

### 8.6 Evaluation Criteria

- **PASS**: Expected outcome fully achieved
- **PARTIAL**: Some expected outcomes achieved, non-critical gaps
- **FAIL**: Expected outcome not achieved or critical regression

Minimum acceptance: All 37 P0 scenarios PASS. All 16 P1 scenarios at least PARTIAL.

---

## 9. Key Risks and Open Questions

### 9.1 Technical Risks

#### Risk 1: rmcp 0.8 is pre-1.0 (HIGH)

**Description:** Prism's MCP layer depends on rmcp 0.8, which is pre-1.0. Breaking changes in rmcp 0.9 or 1.0 could require significant rework of the MCP layer.

**Mitigation:** Isolate rmcp dependency to `prism-mcp` crate only. No rmcp types leak into other crates. Wrap rmcp types in Prism-owned types at the boundary.

**Decision needed:** Accept rmcp 0.8 risk or wait for 1.0. Recommendation: accept -- it is the only Rust MCP library with production usage (tally).

#### Risk 2: CrowdStrike OAuth2 without official Rust SDK (HIGH)

**Description:** poller-cobra uses the official `gofalcon` SDK for CrowdStrike OAuth2 authentication and API access. No equivalent Rust SDK exists. Prism must implement the OAuth2 Client Credentials flow, token caching, and multi-region routing from scratch using the `oauth2` crate and `reqwest`.

**Mitigation:** The OAuth2 Client Credentials flow is well-documented. Token lifecycle management is a known pattern. Contract tests derived from poller-cobra's behavioral specification.

**Decision needed:** None. Implementation proceeds with `oauth2` crate.

#### Risk 3: OCSF DynamicMessage complexity (HIGH)

**Description:** Axiathon's DynamicMessage pattern (runtime field access via prost-reflect) is the most complex piece of the architecture. The axiathon spike was ~2977 LOC for OCSF core. Incorrect field mapping silently corrupts downstream data.

**Mitigation:** Golden-file tests per sensor record type. Property tests for type mapping roundtrips. Integration tests verifying OCSF class code selection per sensor. Build-time validation that all vendor record types map to exactly one OCSF class.

**Decision needed:** None. ADR-003 accepted. DynamicMessage is the right pattern; typed enums do not scale to 83 event classes.

#### Risk 4: Multi-client state isolation under concurrent load (MEDIUM)

**Description:** The pollers are single-client by design. Prism's multi-client model is untested at scale. Concurrent polling for multiple clients using the same sensor type could create subtle timing issues in cursor state management.

**Mitigation:** Per-client FileStore instances with independent file paths. No shared mutable state between clients. Tokio tasks per (tenant, sensor) pair. Holdout scenarios HS-003-04 and HS-006-06 explicitly test this.

**Decision needed:** None. Deployment model is resolved: per-analyst local process, multi-client-aware.

#### Risk 5: ocsf-proto-gen API stability (MEDIUM)

**Description:** ocsf-proto-gen is consumed as a build-time library. If its API changes, Prism's build.rs breaks.

**Mitigation:** Pin ocsf-proto-gen to a specific commit/tag. Use `default-features = false` to minimize surface area. Vendor the OCSF JSON schema to decouple schema updates from library updates.

**Decision needed:** Ownership -- does the Prism team also maintain ocsf-proto-gen? If so, API stability is self-managed. If not, pin and vendor.

### 9.2 Organizational Risks

#### Risk 6: Migration path from pollers to Prism (MEDIUM)

**Description:** Four Go pollers are in production. Migrating to Prism requires: updating Helm charts, provisioning new secrets, validating xMP wire format compatibility, and coordinating with MSSP clients.

**Decision needed:** Migration strategy -- parallel run (both pollers and Prism), canary (one client at a time), or hard cutover? The brief should address this.

#### Risk 7: Sensor API contract drift (LOW-MEDIUM)

**Description:** Sensor vendors (CrowdStrike, Cyberint, Claroty, Armis) may change their APIs. The behavioral contracts recovered from the pollers are point-in-time snapshots.

**Mitigation:** Contract tests against real or mocked API responses. HS-008 scenarios cover API contract changes. `#[serde(default)]` and `#[serde(deny_unknown_fields)]` removed (use lenient parsing from poller-express lesson).

**Decision needed:** API change monitoring strategy -- automated or manual?

### 9.3 Open Questions Requiring Human Decision

| # | Question | Context | Impact |
|---|----------|---------|--------|
| 1 | **~~Deployment model~~ RESOLVED:** Per-analyst local process (stdio transport in Claude Code). Each analyst runs their own Prism instance. Multi-client-aware, not multi-tenant server. | Architecture confirmed. No K8s deployment for MCP layer. |
| 2 | **~~MCP write tools~~ RESOLVED:** Write operations (containment, blocking) excluded from initial scope. Read-only MCP tools only. | Scope confirmed. |
| 3 | **Storage layer: include or defer?** | axiathon has Iceberg/Parquet storage patterns. Prism could store normalized events locally. | Scope, complexity, deployment requirements |
| 4 | **Detection DSL: include or defer?** | axiathon has a detection DSL (.axd files). Prism could run detections on collected data. | Scope, complexity |
| 5 | **Which OCSF version to target?** | ocsf-proto-gen supports any version. axiathon uses 1.7.0. The latest is 1.7.0. | Proto generation, schema compatibility |
| 6 | **Credential rotation without restart: v1 or future?** | All pollers require restart. File watching adds complexity. | Operational impact, implementation effort |
| 7 | **ocsf-proto-gen ownership and maintenance?** | Prism depends on it as a build-time library. Who owns it? | Dependency management, API stability |
| 8 | **Batch sink delivery: NDJSON or JSON array?** | All pollers use per-record POST. Prism improves to batch. Which batch format? | Vector pipeline compatibility |
| 9 | **Circuit breaker pattern: include or defer?** | All pollers lack it. Backoff covers most cases. | Resilience, complexity |
| 10 | **Dead letter queue for failed deliveries: include or defer?** | Failed sink deliveries are retried on next poll cycle. DLQ provides persistence. | Data durability, complexity |

### 9.4 Assumptions Requiring Validation

| Assumption | Basis | Risk if Wrong |
|-----------|-------|---------------|
| xMP envelope format is stable and documented | Recovered from all 4 pollers | Vector pipeline breaks if format differs |
| Sensor APIs will not have breaking changes during development | Current API versions in production | Adapter code needs rework |
| rmcp 0.8 is suitable for production MCP serving | tally uses it in production | MCP layer needs alternative library |
| OCSF 1.7.0 is the correct version target | axiathon and ocsf-proto-gen both use it | Schema mismatch with downstream systems |
| keyring-rs works in headless container environments | serveMyAPI's keytar works on macOS | Falls back to EncryptedFileBackend (acceptable) |
| A single binary can handle the polling load for all sensors and tenants | Go pollers each handle one sensor per client | May need to scale horizontally per tenant |

---

## 10. Recommended Next Steps

### 10.1 Immediate Next Step: Create the Product Brief

The product brief is the first downstream artifact. It should be created using `/vsdd-factory:create-brief` or `/vsdd-factory:guided-brief-creation` with this project context as input.

### 10.2 What the Brief Should Address

The brief must resolve the open questions from Section 9.3 that affect scope and architecture:

**Must address:**
1. **Deployment model** (Question 1) -- RESOLVED: per-analyst local process, stdio transport, multi-client-aware. No open decision needed.
2. **Scope boundary** (Questions 2, 3, 4) -- Question 2 RESOLVED: write operations excluded. Questions 3 and 4 (storage, detection DSL) should be explicitly deferred in the brief.
3. **OCSF version** (Question 5) -- confirm 1.7.0 as the target.
4. **Migration strategy** (Risk 6) -- parallel run, canary, or hard cutover from existing pollers.

**Should address:**
5. **Credential rotation** (Question 6) -- defer or include?
6. **Batch format** (Question 8) -- NDJSON or JSON array?
7. **ocsf-proto-gen ownership** (Question 7) -- internal dependency management.

**Can defer to PRD:**
8. Circuit breaker and DLQ decisions (Questions 9, 10).
9. API monitoring strategy (Risk 7).
10. Horizontal scaling strategy.

### 10.3 What the Brief Should NOT Re-derive

The brief should reference this document for:
- Architecture decisions (ADRs) -- already made, not re-debated in the brief.
- Convention decisions -- already resolved.
- Module decomposition -- already defined.
- Security posture -- already documented.
- Holdout scenarios -- already written.

The brief focuses on **vision, users, constraints, and success criteria** -- not on technical architecture, which is the architecture document's responsibility.

### 10.4 Pipeline After the Brief

```
Product Brief (next)
  -> PRD (from brief + this context)
    -> Architecture (from PRD + this context -- extend recovered-architecture.md)
      -> Stories (decomposed from PRD + architecture)
        -> Wave scheduling (from story dependency graph)
          -> Implementation (TDD per story)
```

The recovered architecture (Section 3) is already substantially complete from Phase 0. The architecture phase should refine and formalize it rather than starting from scratch. The 11 ADRs, 8-layer model, 8-crate decomposition, and deployment topology are all defined.

### 10.5 Key Guidance for PRD Authors

1. **Use the module criticality classification** (Section 6) to prioritize PRD requirements. CRITICAL modules (prism-core, prism-ocsf, prism-state, prism-credentials) must have the most detailed behavioral contracts.

2. **Use the holdout scenarios** (Section 8) as acceptance criteria sources. The 37 P0 scenarios are ready-made acceptance criteria for PRD requirements.

3. **Use the anti-pattern catalog** (Section 6.5) as negative requirements. "The system SHALL NOT store credentials in plaintext" is as important as "The system SHALL encrypt credentials."

4. **Use the sensor API contracts** from cross-repo-dependencies.md as behavioral specifications. These are the ground truth for what each sensor adapter must do.

5. **Use the security priority matrix** (Section 5.5) to ensure all P0 security items are covered by PRD requirements. P1 items should be included as secondary requirements or explicitly deferred.

---

## Appendix A: Reference Manifest

| Repo | URL | Language | Status | Role |
|------|-----|----------|--------|------|
| poller-cobra | github.com/1898andCo/poller-cobra | Go | Analyzed | CrowdStrike sensor behavioral spec |
| poller-express | github.com/1898andCo/poller-express | Go | Analyzed | Cyberint sensor behavioral spec |
| poller-bear | github.com/1898andCo/poller-bear | Go | Analyzed | Claroty sensor behavioral spec |
| poller-coaster | github.com/1898andCo/poller-coaster | Go | Analyzed | Armis sensor behavioral spec |
| tally | github.com/1898andCo/tally | Rust | Analyzed | Primary Rust MCP reference |
| serveMyAPI | github.com/Jktfe/serveMyAPI | TypeScript | Analyzed | Credential management reference |
| axiathon | github.com/1898andCo/axiathon | Rust | Analyzed | OCSF + storage + tenant isolation reference |
| ocsf-proto-gen | github.com/1898andCo/ocsf-proto-gen | Rust | Analyzed | OCSF proto generation (build-time dependency) |
| mcp-claroty-xdome | github.com/1898andCo/mcp-claroty-xdome | TypeScript | Analyzed | Sensor MCP wrapper pattern reference |

## Appendix B: Consumption Mode Per Repo

| Repo | Mode | What Prism Takes |
|------|------|------------------|
| ocsf-proto-gen | **Direct library dependency** (build.rs) | Proto generation, type mapping, enum value map |
| axiathon | **Architectural reference** | DynamicMessage, field resolution, tenant isolation, vault concept |
| tally | **Primary Rust MCP reference** | rmcp patterns, error handling, instrumentation, lint policy |
| poller-cobra | **Behavioral specification** | CrowdStrike API contract, OAuth2 flow, alert field mapping |
| poller-express | **Behavioral specification** | Cyberint API contract, cookie auth, CyberintTime parsing |
| poller-bear | **Behavioral specification** | Claroty API contract (9 endpoints), FileStore, polymorphic JSON |
| poller-coaster | **Behavioral specification** | Armis API contract (7 sources), AQL forwarding, fallback chains |
| mcp-claroty-xdome | **Architectural reference** | Sensor MCP wrapper, cache isolation, error hierarchy |
| serveMyAPI | **Domain reference** | Credential management model, keyring abstraction |

## Appendix C: Key External Dependencies

| Crate | Version | Used By | Purpose |
|-------|---------|---------|---------|
| `rmcp` | 0.8.x | prism-mcp | MCP protocol library |
| `prost` + `prost-reflect` | latest | prism-ocsf | Protobuf runtime + DynamicMessage |
| `prost-build` | latest | prism-ocsf (build) | Proto compilation |
| `ocsf-proto-gen` | pinned commit | prism-ocsf (build) | OCSF schema -> proto generation |
| `keyring` | latest | prism-credentials | OS keyring abstraction |
| `reqwest` | latest (rustls-tls) | prism-sensors | HTTP client with TLS 1.2+ |
| `oauth2` | latest | prism-sensors | CrowdStrike OAuth2 flow |
| `thiserror` | latest | prism-core | Error derivation |
| `serde` + `serde_json` | latest | all crates | Serialization |
| `tracing` + `tracing-subscriber` | latest | all crates | Observability |
| `clap` | latest (derive) | prism-config | CLI + env var parsing |
| `secrecy` | latest | prism-credentials | Secret value protection |
| `proptest` | latest | test deps | Property testing |
| `wiremock` | latest | test deps | HTTP mock server |
| `insta` | latest | test deps | Snapshot testing |
| `criterion` | latest | bench deps | Benchmarking |
| `tokio` | latest (full) | prism | Async runtime |
| `humantime` | latest | prism-config | Duration parsing |

## Appendix D: Phase 0 Artifact Index

All Phase 0 synthesis artifacts that feed into this document:

| Artifact | Path | Content |
|----------|------|---------|
| Cross-repo dependencies | `.factory/phase-0-ingestion/cross-repo-dependencies.md` | API contracts, data flows, shared patterns, dependency graph |
| Convention reconciliation | `.factory/phase-0-ingestion/convention-reconciliation.md` | Naming, error handling, logging, config, testing, state, auth conventions |
| Unified security posture | `.factory/phase-0-ingestion/unified-security-posture.md` | Per-repo audits, attack surface, data classification, security requirements |
| Recovered architecture | `.factory/phase-0-ingestion/recovered-architecture.md` | 8-layer model, 8-crate workspace, ADRs, deployment topology |
| Module criticality | `.factory/phase-0-ingestion/module-criticality.md` | 4-tier classification, implementation order, anti-patterns |
| Holdout scenarios | `.factory/holdout-scenarios/HOLDOUT-INDEX.md` + 8 group files | 53 scenarios, 37 P0, repo coverage matrix |
| Project manifest | `.factory/project.yaml` | Project metadata and factory configuration |
| Reference manifest | `.factory/reference-manifest.yaml` | Git URLs, commits, ingestion dates for all 9 repos |

---

## State Checkpoint

```yaml
document: project-context
phase: 0
step: 0f
status: complete
sections: 10 + 4 appendices
input_files_consumed: 8
  - cross-repo-dependencies.md
  - convention-reconciliation.md
  - unified-security-posture.md
  - recovered-architecture.md
  - module-criticality.md
  - HOLDOUT-INDEX.md
  - project.yaml
  - reference-manifest.yaml
downstream_consumers:
  - product-brief (next)
  - PRD
  - architecture (extension of recovered-architecture.md)
  - stories
open_questions: 10
risks_identified: 7
assumptions_requiring_validation: 6
timestamp: 2026-04-13T00:00:00Z
```
