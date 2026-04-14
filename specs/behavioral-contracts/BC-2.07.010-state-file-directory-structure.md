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

# BC-2.07.010: State File Directory Follows {client}/{sensor}/{source}.json

## Preconditions
- A `FileStore` instance is configured with a root state directory
- A state operation is requested for a `(client_id, sensor_id, source_id)` tuple

## Postconditions
- The state file path is deterministically computed as: `{state_dir}/{client_id}/{sensor_id}/{source_id}.json`
- Directory components use the validated string form of `TenantId`, `SensorId`, and `SourceId`
- The JSON file contains:
  - `cursor`: the serialized composite cursor
  - `fingerprint`: the hex-encoded SHA-256 query fingerprint
  - `updated_at`: ISO 8601 timestamp of the last save
- Parent directories are created automatically if they do not exist
- File and directory names contain only characters from the validated ID patterns (no filesystem-unsafe characters)

## Invariants
- DI-008: Client data separation -- directory hierarchy enforces per-client isolation
- DI-014: Credential name sanitization pattern applied to all ID components (prevents path traversal)

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| `PrismError::Io` | Cannot create directory hierarchy | Error with the full intended path and the I/O error |
| `PrismError::InvalidInput` | ID component contains filesystem-unsafe characters | Prevented at the `TenantId`/`SensorId`/`SourceId` validation layer; never reaches `FileStore` |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-07-018 | State directory path is very long (approaching OS path length limits) | `PrismError::Io` with the full path; the operator must configure a shorter state directory |
| EC-07-019 | `source_id` contains hyphens (e.g., `crowdstrike-alerts`) | Valid per the `SourceId` pattern; the file name is `crowdstrike-alerts.json` |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-011 |
| L2 Invariants | DI-008, DI-014 |
| Priority | P0 |
