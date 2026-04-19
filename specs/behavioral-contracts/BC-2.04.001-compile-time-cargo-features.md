---
document_type: behavioral-contract
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-04-14T05:00:00
phase: 1a
origin: greenfield
subsystem: "SS-04"
capability: "CAP-005"
lifecycle_status: active
introduced: cycle-1
modified: null
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.04.001: Compile-Time Cargo Features Gate Write Code Families

## Preconditions
- `Cargo.toml` defines feature flags: `crowdstrike-write`, `cyberint-write`, `claroty-write`, `armis-write`, `all-write`, with `read-all` as default
- Build is invoked with or without write feature flags

## Postconditions
- When `crowdstrike-write` is not enabled, all CrowdStrike write operation code (containment, alert status updates) is excluded from the binary via `#[cfg(feature = "crowdstrike-write")]`
- A binary built without write features physically cannot execute write operations -- the code does not exist
- Read operations for all sensors are always available (part of `read-all` default feature)
- `all-write` enables all sensor write features simultaneously for convenience

## Invariants
- DI-003: Deny-by-default -- write code absent from binary unless explicitly opted in at compile time

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| Compile error | Code references a write tool without the corresponding feature gate | Conditional compilation error at build time |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-04-001 | Binary built with `crowdstrike-write` but not `claroty-write` | CrowdStrike write tools available (subject to runtime flags); Claroty write tools do not exist in the binary |
| EC-04-002 | Binary built with no features (bare `cargo build --no-default-features`) | No sensor support at all; only core infrastructure compiles; not useful but not a build error |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-005 |
| L2 Invariants | DI-003 |
| Priority | P0 |
