---
document_type: behavioral-contract
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-04-13T12:00:00
phase: 1a
origin: greenfield
subsystem: "Config-Driven Adapters & Hot Reload"
capability: "CAP-030"
---

# BC-2.16.005: `reload_config` MCP Tool — Re-Read All Config Files, Validate, Atomic Swap, Notify

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
  - If any table schema changed (columns added/removed), `notifications/tools/list_changed` is sent to the MCP client so the AI agent refreshes its tool understanding
  - The tool returns a structured response listing: files reloaded, specs added/removed/unchanged, aliases added/removed/changed, feature flags changed
- If validation fails:
  - The current `ConfigSnapshot` is retained unchanged (DI-031)
  - The tool returns `"status": "validation_failed"` with the full list of validation errors (same multi-error format as BC-2.06.005)
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

## What Is NOT Reloadable
- Client structure changes (add/remove `[clients.{id}]` sections) require restart because they affect RocksDB domain initialization, credential store namespaces, and HTTP client pool allocation
- The tool returns `"warnings": ["Client structure changes detected but not applied — restart required"]` if client sections were added or removed

## Dry Run Mode
- When `dry_run: true`, validation runs fully but no swap occurs
- Returns the same change summary as a real reload, plus validation results
- Useful for the AI agent to preview what would change before committing

## Error Handling
- `E-RELOAD-001`: Config file read error (file not found, permission denied) — includes file path and OS error
- `E-RELOAD-002`: Validation failed for prism.toml or aliases.toml (Tier 1/2 per DI-031) — includes full multi-error list, previous config retained
- `E-RELOAD-003`: Partial reload — some sensor spec files (Tier 3 per DI-031) failed validation, others loaded successfully
- `E-RELOAD-004`: No changes detected (all files match previous hash) — no-op

## Audit
- Every `reload_config` invocation is audit-logged (DI-004) with: dry_run flag, files checked, validation result, changes applied (or not)

## Invariants
- DI-031: Hot reload atomicity — three-tier model: prism.toml all-or-nothing (Tier 1), aliases.toml all-or-nothing (Tier 2), sensor specs per-file independent (Tier 3)
- DI-030: Invalid specs do not prevent valid specs from loading (within a single reload)

## Traces
- CAP-030 (Hot Configuration Reload)
- BC-2.16.006 (arc-swap config access)
- BC-2.16.007 (Sensor spec hot reload)
- BC-2.16.009 (Spec file validation)
