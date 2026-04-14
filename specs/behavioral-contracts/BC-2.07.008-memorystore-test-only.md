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

# BC-2.07.008: MemoryStore Is Test-Only and Panics in Production

## Preconditions
- The `CursorStore` trait has two implementations: `FileStore` (production) and `MemoryStore` (test)

## Postconditions
- `MemoryStore` is gated behind `#[cfg(test)]` -- it is only available in test builds
- Any attempt to use `MemoryStore` in a release binary is a compile error
- In test builds, `MemoryStore` provides an in-memory `HashMap`-based implementation of `CursorStore` for fast, isolated unit tests
- `FileStore` is the default and only production implementation

## Invariants
- DI-011: MemoryStore production ban -- compile-time enforcement via `#[cfg(test)]`

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| Compile error | Code outside `#[cfg(test)]` references `MemoryStore` | Rust compiler error: "cannot find type MemoryStore in this scope" |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-07-014 | Integration tests need a store that behaves like production | Integration tests should use `FileStore` with a temporary directory (`tempdir` crate) to test real file I/O behavior |
| EC-07-015 | `MemoryStore` used in a benchmark build (`#[cfg(bench)]`) | `MemoryStore` is NOT available in bench builds; benchmarks use `FileStore` with temp dirs |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-011 |
| L2 Invariants | DI-011 |
| Priority | P0 |
