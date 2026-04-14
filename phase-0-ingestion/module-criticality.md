# Module Criticality Classification -- Prism

**Date:** 2026-04-13
**Phase:** 0 (Pre-Architecture Synthesis)
**Status:** DRAFT -- mutable through Phase 5 gate; frozen after Phase 5 gate passes

---

## Tier Definitions

| Tier | Kill Rate Target | Failure Consequence |
|------|-----------------|---------------------|
| **CRITICAL** | >= 95% | Failure causes data loss, security breach, or total system failure. No workaround exists. |
| **HIGH** | >= 90% | Failure causes degraded service or materially incorrect results delivered to clients or downstream. |
| **MEDIUM** | >= 80% | Failure causes operational inconvenience; service degrades but no data loss or security impact. |
| **LOW** | >= 70% | Failure is cosmetic, logged-only, or has immediate workarounds without service impact. |

---

## 1. Module Inventory

Nine proposed modules derived from the cross-repo synthesis:

| # | Module / Crate | Description |
|---|---------------|-------------|
| 1 | `prism-core` | Domain types, shared traits, newtype wrappers (TenantId, SensorId, RecordType), xMP envelope, error hierarchy, ClientCapabilities (hierarchical feature flag resolution), ConfirmationToken, CapabilityCheckEvent (audit) |
| 2 | `prism-ocsf` | OCSF normalization pipeline: proto-gen build dependency, DynamicMessage wrapping, type mapping, enum value map |
| 3 | `prism-state` | Composite cursor trait + implementations, query fingerprint, atomic file persistence (FileStore), batch receipts |
| 4 | `prism-credentials` | CredentialStore trait, keyring-rs backend, AES-256-GCM encrypted file backend, name index, per-sensor auth injection |
| 5 | `prism-sensors` | Generic DataSource trait + per-sensor adapter impls: CrowdStrike (OAuth2), Cyberint (cookie), Claroty (bearer), Armis (bearer/SDK) |
| 6 | `prism-mcp` | MCP server: rmcp 0.8 tool_router, 24+ tools, prompts, resources, stdio transport, error-to-MCP-code mapping, session lifecycle |
| 7 | `prism-config` | Layered config: env vars (`PRISM_<SENSOR>_*`), `*_FILE` secret resolution, dry-run validation, multi-source error aggregation, TOML per-client capability config (`[clients.{id}.capabilities]`) with hierarchical merge (defaults < client-specific) |
| 8 | `prism-sink` | Batch sink delivery: xMP enrichment, NDJSON/batch HTTP POST with basic auth to Vector, per-record error attribution |
| 9 | `prism` | Binary entry point: signal handling (SIGTERM/SIGINT), component wiring, graceful shutdown, optional health server |

---

## 2. Criticality Classifications

---

### 2.1 prism-core -- CRITICAL

**Rationale:** Every other module depends on this crate. It defines the type language of the system: `TenantId`, `SensorId`, `RecordType`, `XmpMetadata`, `EnrichedPayload<T>`, the error hierarchy root, and the `ClientCapabilities` feature flag system (hierarchical capability resolution, confirmation tokens, audit events). A bug here propagates to all 8 downstream modules simultaneously. The `TenantId` newtype is the primary mechanism preventing accidental client data mixing -- if it is bypassed or incorrectly constructed, Client A's sensor data can be returned for a Client B query (HIGH correctness risk identified in `unified-security-posture.md`, Section 2.2). The `ClientCapabilities` system gates all write operations -- a bug in capability resolution could accidentally enable dangerous write operations (containment, blocking) for clients that should not have them (see ADR-012, P0-SEC-007).

| Attribute | Assessment |
|-----------|-----------|
| Blast radius | All 8 other modules break or silently misbehave |
| Security sensitivity | HIGH -- TenantId is the root of multi-client data correctness; ClientCapabilities gates all write operations |
| Complexity | MEDIUM -- Domain types + trait definitions + capability resolution (~150 lines for ClientCapabilities, ~50 lines for audit events); pure Rust with no I/O. Well-understood pattern from axiathon-core (~612 LOC in reference). |
| Test priority | Property tests for all newtype invariants; unit tests for error display and trait impls; unit tests for hierarchical capability resolution (deny-by-default, parent-path fallback); confirmation token expiry and single-use tests |
| Depended on by | prism-ocsf, prism-state, prism-credentials, prism-sensors, prism-mcp, prism-config, prism-sink, prism |

**Key invariants that must be proven:**
- `TenantId::new()` rejects empty strings and strings with path traversal characters
- `XmpMetadata` fields are non-empty at construction (site, cluster_name, node_name)
- Error variants are `#[non_exhaustive]` to prevent downstream matching exhaustiveness breaks
- `ClientCapabilities::is_enabled()` returns `false` for any capability not explicitly enabled (deny-by-default)
- `ClientCapabilities::is_enabled()` correctly walks the hierarchy: `sensor.crowdstrike.containment` -> `sensor.crowdstrike.write` -> `sensor.crowdstrike` -> `sensor.write`
- `ConfirmationToken` expires after configured TTL (default 300s) and is single-use (cannot be replayed)

---

### 2.2 prism-ocsf -- CRITICAL

**Rationale:** This module performs the data transformation that is Prism's primary value proposition: converting raw vendor-specific sensor records into normalized OCSF protobuf events. Incorrect normalization silently produces wrong data in the downstream SIEM -- alerts with wrong severity, events attributed to wrong class codes, or fields mapped to wrong OCSF paths. These errors are hard to detect downstream and could cause missed detections in a security context. The DynamicMessage pattern from axiathon is rated HIGH complexity for porting (pass-8: "OCSF Event Modeling: proto generation pipeline, runtime field access, four-tier resolution, Arrow schema derivation"). The build.rs proto generation pipeline (ocsf-proto-gen as build-time library) is a mandatory direct code dependency identified in cross-repo-dependencies.md Section 5.

| Attribute | Assessment |
|-----------|-----------|
| Blast radius | Incorrect normalization silently corrupts all sensor output without error |
| Security sensitivity | MEDIUM -- handles security event content (MEDIUM data classification) but not credentials |
| Complexity | HIGH -- DynamicMessage wrapping, proto build pipeline, 24-type OCSF type mapping table, four-tier field resolution. Axiathon spike (~2977 LOC for OCSF core). |
| Test priority | Golden-file tests per sensor record type; property tests for type mapping roundtrips; integration tests verifying OCSF class code selection per sensor |
| Depended on by | prism-sensors (normalizes records), prism-sink (outputs normalized events) |

**Key invariants that must be proven:**
- Every vendor record type maps to exactly one OCSF class code (no default fallback silently loses classification)
- `timestamp_t` fields are always mapped as `int64` (epoch ms), never as `google.protobuf.Timestamp` (cross-repo-dependencies.md type mapping table)
- `json_t` fields are always stored as serialized JSON string, never as `google.protobuf.Struct`

---

### 2.3 prism-state -- CRITICAL

**Rationale:** This module enforces the forward progress invariant that prevents two categories of production failure observed across all 4 reference pollers: (1) in-memory-only cursor loss on restart causing full historical re-fetch and potential rate limit exhaustion, and (2) state updated before persistence causing cursor advancement past undelivered records (the cobra bug). The forward progress invariant -- cursors can only advance, never regress -- is what prevents duplicate data in the SIEM and infinite loops on cursor regression. Per the unified security posture (Section 2.3 and 2.4), state corruption is rated HIGH risk. poller-cobra and poller-express both have this as a HIGH-severity finding.

| Attribute | Assessment |
|-----------|-----------|
| Blast radius | Incorrect cursor state causes data duplication, data loss, or infinite re-fetch loops across all sensors |
| Security sensitivity | LOW -- cursor data is operational metadata (LOW data classification) |
| Complexity | HIGH -- composite cursor with variable arity (2-tuple through 3-tuple), lexicographic tiebreaking, forward progress invariant, atomic file persistence (temp+fsync+rename), query fingerprint (SHA-256), batch receipts, multi-source state in one document. All pollers rated this HIGH complexity for porting. |
| Test priority | Property tests for forward progress (cursor can never regress); unit tests for atomic write (fsync+rename); failure injection tests for mid-write crash recovery; tests for fingerprint mismatch behavior (fatal) |
| Depended on by | prism-sensors (reads/writes cursor per poll), prism-mcp (exposes cursor status via resources) |

**Key invariants that must be proven:**
- `Cursor::advance()` is irreversible: `new_cursor >= old_cursor` always holds under `PartialOrd`
- State is written to disk BEFORE in-memory state is updated (fixes cobra ordering bug)
- `QueryFingerprint` mismatch on startup is fatal -- no silent continuation with stale state

---

### 2.4 prism-credentials -- CRITICAL

**Rationale:** This module holds the master keys to all four external sensor APIs. A path traversal vulnerability (the serveMyAPI CRITICAL finding), plaintext storage, or access control bypass enables an attacker to harvest credentials for CrowdStrike Falcon, Cyberint Argos, Claroty xDome, and Armis Centrix simultaneously -- full MSSP client security visibility. The unified security posture Section 1.5 rates serveMyAPI's reference implementation as having CRITICAL and HIGH vulnerabilities: path traversal (zero sanitization), plaintext storage (`chmod 777`), no access control, and no audit trail. Prism must not replicate any of these. The credential exposure chain (Section 2.1 of security posture) is the highest-impact cross-repo attack vector.

| Attribute | Assessment |
|-----------|-----------|
| Blast radius | Credential exposure enables unauthorized access to all four sensor APIs for all managed MSSP clients |
| Security sensitivity | CRITICAL -- holds API keys, OAuth2 client secrets, bearer tokens (CRITICAL data classification) |
| Complexity | MEDIUM-HIGH -- keyring-rs CRUD + name index (keyring-rs lacks enumeration), AES-256-GCM encrypted file backend with unique salts and external key management, per-sensor auth injection traits (OAuth2, cookie, bearer), input sanitization, access control. serveMyAPI rates credential metadata and file encryption as MEDIUM complexity for porting. |
| Test priority | Adversarial: path traversal attempts on key names; plaintext-absence tests; access control enforcement tests; credential isolation tests (Client A cannot access Client B credentials) |
| Depended on by | prism-sensors (injects auth into outgoing HTTP requests) |

**Key invariants that must be proven:**
- Key name sanitization: `CredentialStore::store()` rejects names containing `/`, `..`, `\`, or null bytes
- Credential isolation: a credential stored under `(tenant_a, sensor)` is inaccessible via `(tenant_b, sensor)` lookup
- Encrypted file backend never writes plaintext to disk at any point (including temp files)

---

### 2.5 prism-sensors -- HIGH

**Rationale:** This module implements the 4 sensor adapters -- the gene-transfusion targets from the 4 Go pollers. Failure in a specific adapter causes loss of visibility for that sensor across all managed clients. Failure in the generic `DataSource` trait causes total collection failure. The primary complexity drivers are: CrowdStrike's OAuth2 two-step fetch with no official Rust SDK (rated HIGH complexity in poller-cobra pass-8), Claroty's polymorphic JSON (IDs returned as string OR number), Armis' AQL query forwarding, and Cyberint's 4-format timestamp parsing. Per-sensor failures are isolated (HIGH not CRITICAL) because other sensors continue operating.

| Attribute | Assessment |
|-----------|-----------|
| Blast radius | Per-adapter failure: one sensor goes dark for all clients. Generic DataSource failure: total collection outage. |
| Security sensitivity | MEDIUM -- processes security event content; per-sensor auth is delegated to prism-credentials |
| Complexity | HIGH aggregate -- CrowdStrike OAuth2+two-step-fetch: HIGH; Claroty polymorphic JSON+9-endpoint: HIGH; Armis AQL+7-source: HIGH; Cyberint cookie-auth+multi-format timestamp: MEDIUM. Generic DataSource trait (eliminating 9x/7x/2x duplication): HIGH design complexity. |
| Test priority | Per-adapter: contract tests against behavioral specs from pass-8 syntheses; forward progress tests; OAuth2 token refresh lifecycle; polymorphic JSON handling; timestamp fallback chains |
| Depended on by | prism-mcp (surfaces sensor query tools), prism-sink (receives normalized records for delivery) |

**Sub-module breakdown:**

| Sub-module | Complexity | Notes |
|-----------|-----------|-------|
| `DataSource` trait (generic) | HIGH | Must model cursor extraction variation, timestamp fallbacks, ID fallbacks across 4 sensors + 9+7+2+1 sources |
| CrowdStrike adapter | HIGH | No official Rust SDK; OAuth2 Client Credentials; two-step fetch; 32 alert fields; region routing |
| Claroty adapter | HIGH | 9 endpoints; POST-for-read; polymorphic JSON (IDs as string or number); 3-tuple cursors; dual pagination strategies |
| Armis adapter | HIGH | AQL query forwarding; 7 sources; timestamp fallback chains (1-3 fields); ID fallback chains (2-4 fields) |
| Cyberint adapter | MEDIUM | Cookie auth middleware; 4-format CyberintTime parsing; 2 sources (alerts + assets); asset ID string comparison bug to avoid |

---

### 2.6 prism-mcp -- HIGH

**Rationale:** This module is the primary interface between Prism and AI agents. Failure causes complete loss of AI agent access to all sensor data. Incorrect tool schemas or error mappings cause silent incorrect behavior in AI agents consuming the tools. Incorrect feature flag evaluation in tool registration could expose write operations to clients that should not have them, or fail to expose write operations to clients that should. The rmcp 0.8 patterns from tally are directly applicable (rated the canonical Rust MCP reference in cross-repo-dependencies.md Section 5). The MCP server itself is VERY HIGH complexity in tally's pass-8 (24 tools, 8 prompts, 14 resources, error mapping, server lifecycle in ~3300 LOC). Prism adds conditional write tool registration via two-tier feature flags (ADR-012), `list_capabilities` meta-tool, dry-run defaults for reversible writes, and confirmation token pattern for irreversible writes. However, Prism's MCP layer is primarily a translation layer over prism-sensors -- domain logic lives elsewhere -- reducing the blast radius of MCP-layer bugs.

| Attribute | Assessment |
|-----------|-----------|
| Blast radius | Total AI agent access loss; incorrect tool results delivered to AI agents |
| Security sensitivity | MEDIUM -- stdio transport with no network exposure; the analyst is trusted. Data correctness (returning correct client's data) is one concern. Additionally, incorrect feature flag evaluation could expose write operations to unauthorized clients (P0-SEC-007). |
| Complexity | HIGH -- tool_router macro patterns, Parameters<T> + JsonSchema, 24+ tools (read + write), conditional write tool registration via two-tier feature flags, `list_capabilities` meta-tool, dry-run default for reversible writes, confirmation token pattern for irreversible writes, `notifications/tools/list_changed` on client context switch, prompts, resources, stdio transport, error-to-MCP-code mapping, explicit tenant_id parameter handling |
| Test priority | Tool schema contract tests; error code mapping tests; tenant_id routing correctness tests; feature flag evaluation tests (write tool hidden when flag disabled, visible when enabled); dry-run default enforcement tests; confirmation token lifecycle tests; `list_capabilities` completeness tests; stdio transport roundtrip tests |
| Depended on by | `prism` binary (mounts and runs the MCP server) |

---

### 2.7 prism-config -- MEDIUM

**Rationale:** Misconfiguration causes operational failure but is immediately visible at startup (dry-run validation, fatal mismatch errors). Config errors do not cause silent data corruption or security breaches in isolation. The `*_FILE` secret resolution logic is security-adjacent (file takes priority over direct env var -- cross-repo-dependencies.md Section 2.3) but the logic itself is simple. The primary risk is the Helm-config mismatch anti-pattern observed in poller-bear (4 env vars set by Helm but never read -- MEDIUM finding). All 4 Go pollers had 30+ env vars and complex validation; the Rust implementation using the `config` crate or a custom env-var reader is straightforward.

| Attribute | Assessment |
|-----------|-----------|
| Blast radius | Startup failure; misconfigured sensor queries; incorrect sensor endpoints or credentials loaded |
| Security sensitivity | MEDIUM -- handles secret file paths and resolves `*_FILE` env vars; must not log secret values |
| Complexity | MEDIUM -- 30+ env vars per sensor config, secret file precedence, duration parsing, multi-error validation, dry-run mode, redacted config printing |
| Test priority | Validation tests for each required field; secret file precedence tests; redacted log output tests (no secret values in logs) |
| Depended on by | `prism` binary (top-level config resolution at startup) |

---

### 2.8 prism-sink -- MEDIUM

**Rationale:** The sink delivers normalized records to the downstream Vector pipeline. The reference implementations all use per-record HTTP POST (identified as an anti-pattern in all 4 poller pass-8 syntheses). Prism improves this to batch delivery with NDJSON or array POST, preserving per-record error attribution. Sink failure causes data loss for the current batch but the cursor has not yet advanced, so records will be retried on the next poll cycle (assuming prism-state correctly implements post-persistence state update). The xMP envelope format must match the existing Vector pipeline wire format exactly (cross-repo-dependencies.md Tier 1 adoption). Failure is operational (missed events) not a security breach.

| Attribute | Assessment |
|-----------|-----------|
| Blast radius | Missed event delivery to Vector/SIEM; no data loss if cursor state is correct (records will be retried) |
| Security sensitivity | LOW -- delivers already-normalized records; basic auth credentials for Vector are not exposed |
| Complexity | MEDIUM -- xMP enrichment, batch delivery, per-record error attribution, basic auth, error body limiting, NDJSON format. Pollers rated HTTPSender as MEDIUM complexity. |
| Test priority | xMP envelope wire format contract tests; batch delivery retry behavior; per-record error attribution in partial-batch failure |
| Depended on by | Downstream Vector pipeline (external) |

---

### 2.9 prism -- LOW

**Rationale:** The binary entry point is pure wiring: component instantiation, tokio signal handling, graceful shutdown with drain windows. The pattern is identical across all 4 Go pollers and tally. All complexity lives in the modules being wired together. A bug here is immediately visible (crash at startup or unclean shutdown), not silent.

| Attribute | Assessment |
|-----------|-----------|
| Blast radius | Process crash or unclean shutdown; all sensors affected simultaneously but immediately visible |
| Security sensitivity | LOW -- no domain logic, no credentials, no data processing |
| Complexity | LOW -- signal handling via `tokio::signal`, component wiring, 5s graceful drain. Pollers rated runner/main as LOW complexity. |
| Test priority | Integration smoke test (binary starts, responds to SIGTERM, exits cleanly) |
| Depended on by | Nothing (top of the dependency tree) |

---

## 3. Summary Matrix

| Module | Tier | Blast Radius | Security Sensitivity | Implementation Complexity | Test Priority |
|--------|------|-------------|---------------------|--------------------------|---------------|
| prism-core | **CRITICAL** | All 8 modules | HIGH (TenantId data correctness) | MEDIUM (pure types) | Property tests + unit tests |
| prism-ocsf | **CRITICAL** | Silent data corruption | MEDIUM (event content) | HIGH (DynamicMessage, proto pipeline) | Golden files + property tests |
| prism-state | **CRITICAL** | Data duplication / loss / infinite loops | LOW (operational metadata) | HIGH (atomic persistence, cursor invariants) | Property + failure injection |
| prism-credentials | **CRITICAL** | Full credential compromise | CRITICAL (API keys, secrets) | MEDIUM-HIGH (encryption, access control) | Adversarial + isolation tests |
| prism-sensors | **HIGH** | Per-sensor blindness | MEDIUM (event content) | HIGH (4 sensor APIs, generic DataSource) | Contract + behavioral tests |
| prism-mcp | **HIGH** | AI agent access loss | LOW (stdio, trusted analyst) | HIGH (24+ tools, rmcp patterns) | Schema + routing correctness tests |
| prism-config | **MEDIUM** | Startup failure / misconfiguration | MEDIUM (secret file paths) | MEDIUM (30+ env vars) | Validation + redaction tests |
| prism-sink | **MEDIUM** | Missed event delivery | LOW (normalized records) | MEDIUM (batch delivery) | Wire format + retry tests |
| prism | **LOW** | Crash / unclean shutdown | LOW (no domain logic) | LOW (signal + wiring) | Integration smoke test |

---

## 4. Dependency Graph (Build Order)

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

**External library dependencies of note:**
- `ocsf-proto-gen` -- build-time library dependency in `prism-ocsf`'s `build.rs` (direct code dependency per cross-repo-dependencies.md Section 5.1)
- `keyring-rs` -- runtime dependency in `prism-credentials`
- `rmcp 0.8` -- runtime dependency in `prism-mcp`
- `prost` + `prost-reflect` -- runtime dependencies in `prism-ocsf`

---

## 5. Implementation Priority Order

Priority is determined by: (1) criticality tier, (2) position in the dependency graph (foundations first), and (3) risk -- modules where reference anti-patterns are most likely to be replicated if built hastily.

| Priority | Module | Rationale |
|----------|--------|-----------|
| 1 | **prism-core** | Foundation for everything; TenantId is the root multi-client data correctness mechanism; must be correct before any other module is built |
| 2 | **prism-state** | Must fix poller-cobra's ordering bug and poller-express's MemoryStore bug at the design stage; cursor correctness is a safety property that cannot be retrofitted |
| 3 | **prism-credentials** | Must be built before prism-sensors to avoid replicate serveMyAPI's CRITICAL vulnerabilities; security-first construction |
| 4 | **prism-ocsf** | Build-time pipeline dependency; must be working before sensor adapters can produce normalized output; HIGH complexity that benefits from early iteration |
| 5 | **prism-config** | Config must be resolved before sensors or MCP can be instantiated; relatively straightforward but must be correct |
| 6 | **prism-sensors** | Core collection logic; highest behavioral complexity across the adapters; built after all its dependencies are proven |
| 7 | **prism-sink** | Downstream from sensors; batch delivery improvement over per-record POST anti-pattern |
| 8 | **prism-mcp** | MCP server surface; built after sensors provide data to expose |
| 9 | **prism** | Binary wiring; built last as it instantiates all the above |

---

## 6. Cross-Cutting Concerns by Tier

### CRITICAL modules -- enforced constraints

The following constraints apply to all CRITICAL modules and must be enforced by code review, clippy configuration, and test gates:

1. **`#![forbid(unsafe_code)]`** -- no unsafe blocks (tally and axiathon-production enforce this)
2. **`clippy::unwrap_used = deny`** -- no `.unwrap()` calls (tally enforces this)
3. **Private fields with getter methods** -- no `pub` fields on domain types (axiathon anti-pattern AP-1 and AP-2: 78+93 call sites requiring migration)
4. **`#[non_exhaustive]` error enums** -- all error types must be `#[non_exhaustive]` (tally pattern)
5. **Security limits on all parsers** -- max length, max nesting depth, CWE citations in comments (axiathon BC-003 pattern)
6. **No `anyhow::Error` as a catch-all variant** -- typed error variants only (axiathon anti-pattern AP-8)

### HIGH modules -- strongly recommended constraints

1. All CRITICAL constraints above apply
2. **Client routing correctness tests** -- prism-mcp must have tests proving that a tool call with `tenant_id: A` never returns Client B's data, and `tenant_id: null` correctly aggregates across all clients
3. **No unbounded maps or caches** -- LRU eviction on all in-memory structures (mcp-claroty-xdome finding: unbounded caches rated HIGH)

---

## 7. Anti-Patterns to Explicitly Not Port

The following patterns from reference repos are identified as anti-patterns and must NOT appear in any Prism module:

| Anti-Pattern | Source | Risk | Prism Constraint |
|-------------|--------|------|-----------------|
| MemoryStore as production cursor backend | poller-cobra, poller-express | HIGH: state lost on restart, re-fetch all data | FileStore (or DB-backed) from day one; MemoryStore for tests only |
| In-memory state updated BEFORE persistence | poller-cobra | HIGH: cursor advances past undelivered records on failure | Update in-memory state ONLY after successful `store.save()` |
| Per-record HTTP POST to sink | all 4 pollers | MEDIUM: 100 records = 100 HTTP requests | Batch delivery with NDJSON; per-record error attribution preserved |
| Plaintext credential file storage | serveMyAPI | CRITICAL: CWE-73, chmod 777 | AES-256-GCM encryption with unique salt per credential; no plaintext at rest |
| Path traversal on credential key names | serveMyAPI | CRITICAL: CWE-22 | Sanitize key names at service layer boundary |
| Hardcoded vault passphrase / static salt | axiathon spike | CRITICAL: CWE-798, CWE-760 | External key management; unique random salt per credential |
| `Date.now()` session ID generation | serveMyAPI | HIGH: ID collision, message routing to wrong client | Not applicable to Prism (stdio transport, no session management needed). Documented as anti-pattern for reference. |
| Unbounded per-IP rate limiter maps | all pollers, mcp-claroty-xdome | MEDIUM: memory leak under high-cardinality IPs | LRU eviction with configurable max entries |
| Dead / unused sentinel errors | all pollers | LOW: misleads developers | Define error variants only when used; deny dead_code |
| Public struct fields on domain types | axiathon spike (78-93 call sites) | MEDIUM: prevents encapsulation migration | Private fields with constructor + getters from day one |
| N-way code duplication (7x, 9x) | poller-bear, poller-coaster | MEDIUM: maintenance cost | Generic `DataSource` trait eliminates all duplication |
| `(error as Error).message` untyped casts | serveMyAPI | MEDIUM: swallows structure | Typed `thiserror` variants throughout |

---

## State Checkpoint

```yaml
phase: 0
step: 5
artifact: module-criticality.md
status: complete
modules_classified: 9
critical_modules: 4 (prism-core, prism-ocsf, prism-state, prism-credentials)
high_modules: 2 (prism-sensors, prism-mcp)
medium_modules: 2 (prism-config, prism-sink)
low_modules: 1 (prism)
anti_patterns_cataloged: 12
implementation_priority_order: defined
freeze_condition: after Phase 5 gate passes
timestamp: 2026-04-13T00:00:00Z
```
