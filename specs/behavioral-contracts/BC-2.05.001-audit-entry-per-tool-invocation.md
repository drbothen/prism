---
document_type: behavioral-contract
level: L3
version: "1.4"
status: draft
producer: product-owner
timestamp: 2026-04-14T05:00:00
phase: 1a
origin: greenfield
subsystem: "SS-05"
capability: "CAP-007"
lifecycle_status: active
introduced: cycle-1
modified: 2026-06-11
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
inputs: [".factory/specs/prd.md", ".factory/specs/domain-spec/capabilities.md"]
input-hash: "c36ec87"
traces_to: ["CAP-007"]
extracted_from: ".factory/specs/prd.md"
---

# BC-2.05.001: Every MCP Tool Invocation Produces Exactly One Audit Entry (Fail-Closed for Writes)

## Description

The audit middleware ensures that every MCP tool invocation — whether it succeeds or fails —
produces exactly one `AuditEntry` as a structured JSON log event. Write operations are
fail-closed with respect to audit: if audit emission fails for any write operation (including
confirmation token generation, credential mutation, confirmed action execution, or live
config-snapshot swap), the write is aborted and `E-AUDIT-001` is returned. The write is never
executed without a successful audit record. Read operations are fail-open: a tracing failure
during read audit produces a `_meta.audit_warning` in the response but does not block the
operation.

**Write operations enumeration (v1.4):** The following MCP tools are classified as
`WriteTool` (fail-closed) per `tool_classification_registry()` in `prism-mcp/src/server.rs`:
`confirm_action`, `add_sensor_spec`, `create_alias`, `delete_alias`, `reload_config`
(dry_run=false path). The `reload_config` tool calls `ConfigManager::store(candidate)` on the
live-swap path (non-dry-run) — the same ArcSwap mutation that `add_sensor_spec` lands on —
and is therefore mutation-equivalent to `add_sensor_spec`. Classification: `WriteTool` per
mutation-equivalence precedent (PRL-P4-01 adjudication, 2026-06-11).

**`reload_plugin` non-classification note:** `reload_plugin` currently returns
`not_yet_available_msg("plugin management")` before any mutation; it is a non-mutating stub
and is NOT classified `WriteTool` at this time. When `reload_plugin` is wired to actually
load or swap a plugin, it MUST be added to `tool_classification_registry()` as `WriteTool`
before that story merges (see BC body note below).

## Preconditions
- An MCP tool invocation is dispatched through the tool dispatch middleware
- The `tracing` subscriber is initialized

## Postconditions
- Exactly one `AuditEntry` is emitted as a structured JSON log event via `tracing::info!`
- The entry is emitted regardless of whether the tool invocation succeeds or fails
- Successful invocations include `result_summary` with outcome details
- Failed invocations include `result_summary` with the error category and message
- **Write operations fail-closed on audit failure**: if audit emission fails for a write operation (including confirmation token generation, credential mutation, confirmed action execution, or live config-snapshot swap via `reload_config` dry_run=false), the write operation is aborted and a structured error `E-AUDIT-001` is returned. The write is never executed without a successful audit record. Write-classified tools: `confirm_action`, `add_sensor_spec`, `create_alias`, `delete_alias`, `reload_config` (five tools as of v1.4).
- **Read operations proceed on audit failure**: if audit emission fails for a read-only operation, the operation still proceeds. A warning `_meta.audit_warning: "audit emission failed"` is included in the response.

## Invariants
- DI-004: Audit completeness -- every MCP tool invocation produces exactly one AuditEntry
- Write operations are fail-closed with respect to audit: no unaudited writes
- **Write tool set invariant (v1.4):** The complete set of write-classified tools is `{confirm_action, add_sensor_spec, create_alias, delete_alias, reload_config}`. Any new MCP tool that calls `ConfigManager::store`, mutates the alias store, mutates confirmation tokens, or executes a confirmed write action MUST be added to `tool_classification_registry()` as `WriteTool` in the same commit as the implementation. `reload_plugin` is currently non-mutating (stub); it MUST be reclassified `WriteTool` when wired to actual plugin mutation.

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| `PrismError::Audit` | Tracing subscriber failure during write operation audit | Write operation aborted; structured error: `code: "E-AUDIT-001"`, `message: "Audit emission failed; write operation blocked"`, `category: "transient"`, `retryable: true`, `suggestion: "Retry the operation. If the error persists, check tracing subscriber health."` |
| Warning | Tracing subscriber failure during read operation audit | Read operation proceeds; `_meta.audit_warning: "audit emission failed"` set on response |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| DEC-014 | Tracing subscriber encounters an error during audit emission for a write operation | Write operation is aborted; `E-AUDIT-001` error returned; the write is never executed |
| EC-05-001 | Tool invocation panics before audit middleware completes | Panic is caught by the MCP transport layer; an audit entry is still emitted with `result_summary: "panic"` if the middleware uses a catch-unwind guard |
| EC-05-002 | Audit emission fails for a read-only query | Query proceeds and returns results; response includes `_meta.audit_warning` |
| EC-05-003 | `reload_config` called dry_run=false; audit emission fails | Config swap is aborted (no `ConfigManager::store` call); `E-AUDIT-001` returned; the ArcSwap state is unchanged. Mutation-equivalence: same fail-closed behavior as `add_sensor_spec` (both call `ConfigManager::store`). |

## Canonical Test Vectors

See `.factory/specs/prd-supplements/test-vectors.md` for canonical test vectors for BC-2.05.001.

| Scenario | Tool Type | Audit Failure? | Expected Behavior |
|----------|-----------|---------------|-------------------|
| Normal write | `crowdstrike_contain_host` | No | Audit entry emitted; token or execution proceeds |
| Write with audit failure | Any write tool (`confirm_action`, `add_sensor_spec`, `create_alias`, `delete_alias`, `reload_config` dry_run=false) | Yes | Write aborted; `E-AUDIT-001` returned; no write executed |
| Normal read | `query_crowdstrike_alerts` | No | Audit entry emitted; results returned |
| Read with audit failure | Any read tool | Yes | Results returned; `_meta.audit_warning` set |
| `reload_config` audit failure | `reload_config` (dry_run=false) | Yes | `E-AUDIT-001` returned; `ConfigManager::store` never called; config snapshot unchanged |

## Verification Properties

- **VP-033** (Audit buffer: RocksDB write completes before delivery attempt) — verifies ordering of audit persistence relative to execution for the DTU CrowdStrike clone.

No VP in VP-INDEX v1.5 directly covers the general fail-closed write behavior. Placeholder for future VP.

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-007 |
| L2 Invariants | DI-004 |
| Addresses | ADV-2-009 |
| Priority | P0 |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.4 | PRL-P4-01 adjudication burst | 2026-06-11 | product-owner | **`reload_config` reclassified as WriteTool (PRL-P4-01 MEDIUM finding closure).** `reload_config` (dry_run=false) calls `ConfigManager::store(candidate)` — the same ArcSwap mutation path as `add_sensor_spec`. Under BC-2.05.001 invariant "no unaudited writes," this is a write operation. Description updated to enumerate five write tools; Postconditions §write-fail-closed updated to name all five tools; Invariants §write-tool-set-invariant added with `reload_plugin` future-wiring note; EC-05-003 added for reload_config audit-failure edge case; Canonical Test Vectors updated to add reload_config audit-failure row. Sibling: BC-2.16.002 v1.77 (mcp.tool.called row 145 WriteTool enumeration updated from four to five tools). |
| 1.3 | pass-73-fix | 2026-04-20 | state-manager | Deterministic changelog reorder: sorted all rows to descending version order (pass-73 bash script). |
| 1.2 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added extracted_from/inputs/input-hash/traces_to frontmatter; added ## Description synthesized from body; added ## Canonical Test Vectors scaffolding; added ## Verification Properties cross-ref; appended ## Changelog row. |
| 1.1 | Phase 1 | 2026-04-14 | product-owner | Previous version |
