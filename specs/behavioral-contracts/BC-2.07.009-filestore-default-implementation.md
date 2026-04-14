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

# BC-2.07.009: FileStore Is the Default and Only Production CursorStore

## Preconditions
- Prism is built as a release binary
- A state directory path is configured (CLI arg, env var, or default)

## Postconditions
- `FileStore` is automatically used as the `CursorStore` implementation
- The state directory defaults to a platform-appropriate location (e.g., `$XDG_DATA_HOME/prism/state/` on Linux, `~/Library/Application Support/prism/state/` on macOS) if not explicitly configured
- `FileStore` creates the state directory hierarchy on first use if it does not exist
- All `CursorStore` trait operations (`load`, `save`, `delete`, `exists`) are implemented using the atomic write pattern (BC-2.07.003)
- The `load` operation deserializes the state file JSON into the cursor and fingerprint structures
- The `save` operation serializes the cursor and fingerprint to JSON and writes atomically

## Invariants
- DI-011: MemoryStore production ban -- `FileStore` is the only option in release builds
- DI-013: Atomic state writes

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| `PrismError::Io` | State directory path is on a read-only filesystem | Fatal error at startup with the path and I/O error |
| `PrismError::Io` | State file JSON deserialization fails (corrupt or incompatible format) | Error with the path and suggestion to delete the file to reset |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-07-016 | State directory is on a network filesystem (NFS) | Atomic rename may not be supported; `FileStore` should log a warning if it detects a non-local filesystem and document the limitation |
| EC-07-017 | Concurrent Prism instances share the same state directory | Not supported; each analyst runs their own instance with a separate state directory. No file locking is implemented. |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-011 |
| L2 Invariants | DI-011, DI-013 |
| Priority | P0 |
