---
document_type: behavioral-contract
level: L3
version: "1.1"
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
inputs: [".factory/specs/prd.md", ".factory/specs/domain-spec/capabilities.md"]
input-hash: "e5de7f9"
traces_to: ["CAP-005"]
extracted_from: ".factory/specs/prd.md"
---

# BC-2.04.001: Compile-Time Cargo Features Gate Write Code Families

## Description

Compile-time Cargo feature flags (`crowdstrike-write`, `cyberint-write`, `claroty-write`,
`armis-write`, `all-write`) control whether sensor-specific write operation code is included
in the binary. When a write feature is not enabled at build time, the corresponding write
operation code is entirely absent from the compiled artifact — it cannot be invoked, not
even through indirect paths. Read operations (the `read-all` default feature) are always
available regardless of which write features are selected.

This is the first tier of Prism's two-tier write gate (see BC-2.04.004): compile-time
exclusion provides a physical guarantee that write code does not exist in production binaries
unless explicitly opted in at build time.

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

## Canonical Test Vectors

See `.factory/specs/prd-supplements/test-vectors.md` for canonical test vectors for BC-2.04.001.

| Scenario | Input | Expected Output |
|----------|-------|----------------|
| Write feature absent | Binary built without `crowdstrike-write` | `crowdstrike_contain_host` tool absent from binary; `tools/list` does not include it |
| Write feature present | Binary built with `crowdstrike-write` | `crowdstrike_contain_host` tool present; subject to runtime flag check |
| All-write feature | Binary built with `all-write` | All sensor write tools present in binary |

## Verification Properties

- **VP-020** (Feature flag: compile AND runtime must both permit) — verifies that the two-tier gate requires both compile-time feature presence and runtime flag enablement.

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-005 |
| L2 Invariants | DI-003 |
| Priority | P0 |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.1 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added extracted_from/inputs/input-hash/traces_to frontmatter; added ## Description synthesized from body; added ## Canonical Test Vectors scaffolding; added ## Verification Properties cross-ref; added ## Changelog. |
