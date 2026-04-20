---
document_type: behavioral-contract
level: L3
version: "1.1"
status: draft
producer: product-owner
timestamp: 2026-04-13T12:00:00
phase: 1a
origin: greenfield
subsystem: "SS-16"
capability: "CAP-030"
lifecycle_status: active
introduced: cycle-1
modified: null
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
inputs:
  - ".factory/specs/prd.md"
  - ".factory/specs/domain-spec/capabilities.md"
input-hash: "[pending-recompute]"
traces_to:
  - "CAP-030"
extracted_from: ".factory/specs/prd.md"
---

# BC-2.16.005: `reload_config` MCP Tool — Re-Read All Config Files, Validate, Atomic Swap, Notify

## Description

The `reload_config` MCP tool enables zero-downtime configuration updates by re-reading
all config files, constructing a new `ConfigSnapshot`, validating it fully, and then
atomically swapping it in via ArcSwap. If validation fails, the current config is
retained unchanged (DI-031 fail-closed). Hash-based change detection makes the reload
a no-op when files are unchanged.

The tool supports dry-run mode for previewing changes before applying them, and emits
a structured response listing exactly what changed. MCP `notifications/tools/list_changed`
is sent when table schemas change so the AI agent can refresh its tool understanding.
Every invocation is audit-logged regardless of outcome.

## Preconditions
- Prism is running with a valid `ConfigSnapshot` loaded at startup
- The analyst (via AI agent) invokes the `reload_config` MCP tool

## Tool Schema
```json
{
  "name": "reload_config",
  "description": "Re-reads prism.toml, sensor spec files, and aliases.toml without restarting the server. Validates all files before applying. Reports what changed.",
  "inputSchema": {
    "type": "object",
    "properties": {
      "dry_run": {
        "type": "boolean",
        "description": "If true, validate and report changes without applying them. Default: false.",
        "default": false
      }
    }
  }
}
```

## Postconditions
- The tool re-reads and parses:
  1. `prism.toml` — main configuration (feature flags, credentials references, defaults)
  2. All `*.sensor.toml` files in the `sensor_specs_dir` — sensor spec files (CAP-029)
  3. `aliases.toml` — query alias definitions (CAP-016)
- A new `ConfigSnapshot` is constructed from the parsed files with a fresh SHA-256 hash
- Hash-based change detection: if the new snapshot's hash matches the current snapshot's hash, the reload is a no-op and the tool returns `"status": "unchanged"`
- If changes are detected, the new snapshot undergoes full validation (BC-2.16.009, BC-2.06.005):
  - All sensor spec files validated
  - All alias definitions validated (cycle detection, depth check)
  - Feature flag consistency checked
  - Credential references validated against credential store
- If validation passes:
  - The new `ConfigSnapshot` atomically replaces the current one via `arc_swap::ArcSwap::store()` (BC-2.16.006)
  - Sensor table registrations are updated: new tables registered, removed tables unregistered, modified tables re-registered
  - If any table schema changed (columns added/removed), `notifications/tools/list_changed` is sent to the MCP client
  - The tool returns a structured response listing: files reloaded, specs added/removed/unchanged, aliases added/removed/changed, feature flags changed
- If validation fails:
  - The current `ConfigSnapshot` is retained unchanged (DI-031)
  - The tool returns `"status": "validation_failed"` with the full list of validation errors
  - No `notifications/tools/list_changed` is sent

## What Is Reloadable
| Config Area | Reloadable | Notes |
|-------------|-----------|-------|
| Feature flags | Yes | Per-client capability overrides take effect immediately |
| Credentials references | Yes | New credential references resolved on next query |
| Sensor spec files | Yes | Tables added/removed/modified (BC-2.16.007) |
| Aliases | Yes | New/modified/deleted aliases available immediately |
| Client structure | **No** | Adding/removing clients requires restart |
| `state_dir` path | **No** | RocksDB path is immutable after startup |
| `watchdog_level` | Yes | New limits apply to next query execution |

## Dry Run Mode
- When `dry_run: true`, validation runs fully but no swap occurs
- Returns the same change summary as a real reload, plus validation results
- Useful for the AI agent to preview what would change before committing

## Invariants
- Validation failure always retains the current config unchanged (DI-031 fail-closed)
- Every invocation is audit-logged regardless of outcome (DI-004)
- Hash-based no-op detection prevents unnecessary table re-registration

## Error Conditions
| Error | Condition | Behavior |
|-------|-----------|----------|
| `E-RELOAD-001` | Config file read error (file not found, permission denied) | Includes file path and OS error |
| `E-RELOAD-002` | Validation failed for prism.toml or aliases.toml (Tier 1/2 per DI-031) | Full multi-error list returned; previous config retained |
| `E-RELOAD-003` | Partial reload — some sensor spec files (Tier 3 per DI-031) failed validation | Failed specs rejected; others loaded successfully |
| `E-RELOAD-004` | No changes detected (all files match previous hash) | No-op; status "unchanged" |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| No changes | files unchanged since last load | `status: unchanged`; no table re-registration |
| Client section added | new `[clients.X]` section in prism.toml | Warning: "Client structure changes detected but not applied — restart required" |
| Schema change in spec | column added to sensor spec | Tables re-registered; `notifications/tools/list_changed` sent |
| Dry run | `dry_run: true` with changes | Change summary returned; no swap occurs |

## Canonical Test Vectors

See `.factory/specs/prd-supplements/test-vectors.md` for full canonical vectors.

| Scenario | Input | Expected Output |
|----------|-------|-----------------|
| Happy path — change detected | spec file modified | Validation passes; atomic swap; change summary returned |
| No-op | files unchanged | `status: unchanged`; no re-registration |
| Validation failure | invalid spec file | `status: validation_failed`; current config retained |
| Dry run | `dry_run: true` | Change summary; no actual swap |
| Partial reload | one spec valid, one invalid | Valid spec loaded; invalid rejected; `E-RELOAD-003` |

## Verification Properties

| VP ID | Description |
|-------|-------------|
| (placeholder) | VP to be assigned — verify fail-closed on validation failure (current config retained) |
| (placeholder) | VP to be assigned — verify MCP notification sent on schema change |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-030 |
| L2 Invariants | DI-004, DI-030, DI-031 |
| L2 Entities | ConfigSnapshot, SensorSpec |
| Priority | P1 |

## Changelog
| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.0 | cycle-1 | 2026-04-13 | product-owner | Initial draft |
| 1.1 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added extracted_from/inputs/input-hash/traces_to frontmatter; added ## Description; added ## Invariants; added ## Canonical Test Vectors; added ## Verification Properties; renamed Error Cases → Error Conditions; added ## Changelog. |
