---
document_type: architecture-section
level: L3
section: "security-architecture"
version: "1.0"
status: draft
producer: architect
timestamp: 2026-04-15T12:00:00
phase: 1b
inputs: [prd.md, domain-spec/invariants.md, prd-supplements/nfr-catalog.md]
traces_to: ARCH-INDEX.md
---

# Security Architecture

## Threat Model

Prism operates in a **trusted analyst, untrusted sensor data** model. The analyst is an MSSP employee — client isolation is about data correctness, not security isolation between adversarial tenants. However, sensor data (hostnames, file paths, process names) is attacker-controlled content that flows through the LLM context.

### Primary Threats

| Threat | Vector | Mitigation |
|--------|--------|-----------|
| Prompt injection via sensor data | Attacker-controlled hostnames/filenames in sensor responses | 4-layer sanitization (SS-09) |
| Cross-client data leakage | Bug mixes client data in responses/cache/logs | TenantId newtype (DI-008, AD-010) |
| Credential exposure | Secrets in logs, errors, MCP responses | Secret redaction at every boundary (DI-002) |
| Unauthorized write operations | AI agent attempts disabled operations | Two-tier feature flags + hidden tools (AD-011) |
| Path traversal in credentials | Malicious credential names | Name sanitization regex (DI-014) |
| Token replay | Reuse of confirmation tokens | Single-use + content hash verification (DI-007) |

## Credential Management (prism-credentials)

### Decision: OS Keyring Primary, Encrypted File Fallback (NFR-005)

**Status:** accepted
**Context:** Credentials must be encrypted at rest with hardware-backed encryption where available.
**Decision:** OS keyring (macOS Keychain, Windows Credential Vault, Linux libsecret) as primary backend. AES-256-GCM encrypted file as fallback when keyring is unavailable.
**Rationale:** Reference: serveMyAPI demonstrated OS keyring integration for MCP server credential management, proving the pattern works for per-analyst deployment.

**Credential namespace:** `(client_id, sensor_id, credential_name)` — three-component key. No "get all credentials" method that crosses client boundaries (DI-002).

**Encrypted file backend:**
- AES-256-GCM with HKDF-SHA256 key derivation
- Per-credential 32-byte random salt
- Key material from environment variable or K8s secret mount
- File permissions: 0600 (files), 0700 (directories)
- Stored in `{state_dir}/{client_id}/` directory structure

**Startup probe:** Keyring availability check at startup (pre-authorize macOS Keychain permission prompt) before any credential access.

## Feature Flag System (prism-security)

### Decision: Two-Tier Feature Flags (AD-011)

**Status:** accepted
**Context:** Write operations need defense-in-depth gating. Must be possible to compile a binary with no write code at all.
**Decision:** Tier 1: Cargo compile-time features (`--features crowdstrike-write`). Tier 2: Runtime per-client TOML config (`BTreeMap<String, Effect>`).
**Rationale:** Compile-time gates ensure write operation code is not present in the binary when not needed. Runtime gates enable per-client control without recompilation.

**Resolution algorithm:**
1. Check compile-time feature. If absent → deny (tool code not compiled).
2. Walk capability path from most-specific to least-specific (BTreeMap iteration).
3. First matching rule determines effect (Allow or Deny).
4. At same specificity, Deny beats Allow.
5. No match → implicit Deny.

**Hidden tools pattern:** Tools whose capability resolves to Deny are omitted from `tools/list`. The AI agent never sees disabled operations.

## Confirmation Token System

For irreversible write operations (host containment, credential deletion, schedule deletion):

1. First call returns `ConfirmationToken` with action summary and 300s expiry
2. Agent presents summary to analyst (human-in-the-loop)
3. `confirm_action(token_id)` executes the operation
4. Token content hash prevents action parameter tampering between steps

Caps: 100 active tokens max (DI-015). In-memory only — lost on restart.

## Prompt Injection Defense (prism-security)

Four-layer defense operating on all sensor data in MCP responses:

| Layer | Mechanism | Component |
|-------|-----------|-----------|
| 1. Structural separation | Sensor data in `structuredContent` JSON, never interpolated into prose | Response construction in prism-mcp |
| 2. Provenance framing | Tool descriptions warn about untrusted data; `trust_level: "untrusted_external"` | Tool registration in prism-mcp |
| 3. Pattern detection | Regex scanner for injection patterns ("ignore previous", "system:", etc.) | `InjectionScanner` in prism-security |
| 4. Trust metadata | Per-response `_meta.safety_flags` array aggregating flagged fields | Response envelope in prism-mcp |

Safety flags **flag** suspicious content — they never strip it. The analyst needs full forensic data.

## Audit Trail (prism-audit)

Every MCP tool invocation, scheduled query execution, and detection evaluation produces exactly one `AuditEntry`. Structured JSON via `tracing` crate.

**Fail-closed for writes (DI-004/DI-016):** If audit emission fails, write operations are aborted. Read operations proceed with warning. Scheduled/detection operations proceed with warning.

**Buffered forwarding (CAP-025):** Audit entries are written to RocksDB (WAL-synced) before external delivery attempt. Exponential backoff on delivery failure (2s base, 60s max). 100K entry buffer with oldest-first purge.

## Client Isolation (TenantId Newtype)

### Decision: TenantId Newtype for Client Isolation (AD-010)

**Status:** accepted
**Context:** Cross-client data leakage is the highest-impact correctness bug in an MSSP system.
**Decision:** `TenantId` newtype wrapping a validated `String`. Every function that handles client-scoped data takes `&TenantId` as a parameter.
**Rationale:** Compile-time enforcement via the type system. You cannot accidentally pass a raw string where a TenantId is expected. Validation occurs once at the boundary (MCP parameter parsing), then the validated type propagates through the call chain.
**Consequences:** All storage keys, cache keys, log spans, and error messages include `TenantId`. Code review can verify client isolation by checking TenantId threading.
