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

# BC-2.07.003: State Files Use Atomic Write Pattern (temp + fsync + rename)

## Preconditions
- The `FileStore::save()` method is called with new cursor state
- The target state file directory exists and is writable

## Postconditions
- A temporary file is created in the same directory as the target state file
- The cursor state (JSON) is written to the temporary file
- `fsync` is called on the temporary file to ensure data reaches disk
- The temporary file is atomically renamed over the target state file
- If the rename succeeds, the state file contains the new cursor state
- If any step fails, the original state file is untouched (crash-safe)

## Invariants
- DI-013: Atomic state writes -- no state file is ever written in place
- DI-009: Persistence before state update -- the save must complete before in-memory state advances

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| `PrismError::Io` | Temp file creation fails (disk full, permissions) | Error returned; in-memory cursor does not advance; the previous state file is untouched |
| `PrismError::Io` | `fsync` fails | Temp file is cleaned up (best-effort delete); error returned; in-memory cursor does not advance |
| `PrismError::Io` | Rename fails (e.g., cross-filesystem rename) | Temp file is cleaned up; error returned; previous state file preserved |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-07-003 | Prism crashes between fsync and rename | The temp file remains on disk; the state file is the previous version. On next startup, orphaned temp files in the state directory are cleaned up. |
| EC-07-004 | State directory does not exist on first run | `FileStore` creates the directory hierarchy (including parent dirs) before writing |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-011 |
| L2 Invariants | DI-009, DI-013 |
| Priority | P0 |
