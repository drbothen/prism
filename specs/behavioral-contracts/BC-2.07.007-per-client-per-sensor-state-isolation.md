---
document_type: behavioral-contract
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-04-14T05:00:00
phase: 1a
origin: greenfield
subsystem: "Cursor State Management"
capability: "CAP-011"
---

# BC-2.07.007: State Is Isolated Per-Client, Per-Sensor, Per-Source

## Preconditions
- Multiple clients are configured, each with one or more sensors and data sources
- The `FileStore` manages state files on disk

## Postconditions
- State files are stored in a directory hierarchy: `{state_dir}/{client_id}/{sensor_id}/{source_id}.json`
- Each state file contains the cursor and fingerprint for exactly one `(client_id, sensor_id, source_id)` tuple
- No state file contains data from multiple clients, sensors, or sources
- State operations (load, save, delete) are always scoped by the full tuple; there is no "load all state" operation that crosses client boundaries
- Directory names are derived from validated `TenantId`, `SensorId`, and `SourceId` values (safe for filesystem paths)

## Invariants
- DI-008: Client data separation -- state files are scoped per client
- DI-001: Cursor forward progress -- applied independently per state file

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| `PrismError::Io` | State directory cannot be created (permissions, disk full) | Fatal error at startup: "Cannot create state directory '{path}': {io_error}" |
| `PrismError::Io` | State file for one client is corrupt but others are fine | Only the affected source fails; other clients/sensors/sources continue normally |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-07-012 | Client is removed from config but state files remain on disk | Orphaned state files are not automatically deleted; they are inert (not loaded because the client is not configured) |
| EC-07-013 | Two clients happen to query the same sensor type (e.g., both have CrowdStrike) | State is fully separate; each has its own `{client_id}/crowdstrike/` directory tree |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-011 |
| L2 Invariants | DI-001, DI-008 |
| Priority | P0 |
