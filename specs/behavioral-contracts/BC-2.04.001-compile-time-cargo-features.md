---
document_type: behavioral-contract
level: L3
version: "1.2"
status: draft
producer: product-owner
timestamp: 2026-04-14T05:00:00
phase: 1a
origin: greenfield
subsystem: "SS-04"
capability: "CAP-005"
lifecycle_status: active
introduced: cycle-1
modified: "2026-06-10"
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
inputs: [".factory/specs/prd.md", ".factory/specs/domain-spec/capabilities.md"]
input-hash: "566def3"
traces_to: ["CAP-005"]
extracted_from: ".factory/specs/prd.md"
---

# BC-2.04.001: Compile-Time Write Capability Tier Is Registry-Derived from Write-Endpoint Declarations

## Description

The compile-time tier of Prism's two-tier write gate (see BC-2.04.004) is registry-derived:
a sensor has write capability at the compile tier if and only if its TOML spec declares a
matching `[[write_endpoints]]` section loaded into the `WriteEndpointRegistry` at boot
(registry-driven dispatch per BC-2.16.012, post-PLUGIN-MIGRATION-001-B). `write_pipeline.rs`
derives the `CompileFeatureGate` from registry presence; a missing declaration yields
`CapabilityCheckResult::DeniedCompileTime` → `PrismError::CapabilityDenied` (E-FLAG-002)
regardless of runtime capability configuration.

The `{sensor}-write` Cargo features (`crowdstrike-write`, `cyberint-write`, `claroty-write`,
`armis-write`, `all-write`) are empty test-gating declarations in `prism-query` — they
preserve `#[cfg]`-gated test coverage under `--all-features` and are NOT the production
gate; sensor write code is present in the binary regardless of feature selection. Read
operations are always available (never gated by the compile tier). The one genuinely
cfg-gated write path is the alias-write tier (`alias.write`, BC-2.11.008):
`alias_write_compile_gate()` derives the compile gate from `#[cfg(feature = "alias-write")]`
(a runtime-advisory gate, not a compile-time exclusion — BC-2.11.006) and produces the same
`DeniedCompileTime` denial class.

## Preconditions
- A sensor's TOML spec does or does not declare `[[write_endpoints]]` sections; declared sections are loaded into the `WriteEndpointRegistry` at boot
- A write operation is planned or invoked targeting that sensor

## Postconditions
- A sensor has compile-tier write capability if and only if its TOML spec declares a matching `[[write_endpoints]]` entry loaded into the `WriteEndpointRegistry` at boot (BC-2.16.012)
- When no matching declaration exists, the write is denied at the compile tier (`CapabilityCheckResult::DeniedCompileTime` → `PrismError::CapabilityDenied`, E-FLAG-002) regardless of runtime capability configuration — deny-by-default; the runtime tier cannot override
- The `{sensor}-write` Cargo features are empty test-gating declarations in `prism-query`; enabling or omitting them does not change production write gating, and sensor write code remains in the binary either way
- Read operations for all sensors are always available (never gated by the compile tier)
- Alias-write exception: the `alias.write` compile gate is derived from `#[cfg(feature = "alias-write")]` (BC-2.11.008, BC-2.11.006) and produces the same `DeniedCompileTime` denial class

## Invariants
- DI-003: Deny-by-default -- no compile-tier write capability unless explicitly declared via a `[[write_endpoints]]` entry in the sensor's TOML spec

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| E-FLAG-002 (`PrismError::CapabilityDenied`) | A write targets a sensor/table with no matching `[[write_endpoints]]` declaration in the `WriteEndpointRegistry` | Structured denial: "Write capability '{path}' denied: no write-endpoint declaration (no [[write_endpoints]] entry in the sensor's TOML spec)"; runtime config cannot override |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-04-001 | CrowdStrike sensor TOML spec declares `[[write_endpoints]]`; Claroty sensor TOML spec does not | CrowdStrike writes pass the compile tier (subject to runtime flags); Claroty writes are denied `DeniedCompileTime` (E-FLAG-002) |
| EC-04-002 | Binary built with default features vs `--all-features` | Identical production write gating in both builds — the registry, not Cargo features, is the gate; `--all-features` only un-gates the `#[cfg]`-gated test suites |

## Canonical Test Vectors

See `.factory/specs/prd-supplements/test-vectors.md` for canonical test vectors for BC-2.04.001.

| Scenario | Input | Expected Output |
|----------|-------|----------------|
| Declaration absent | CrowdStrike sensor TOML spec has no `[[write_endpoints]]` entry for the target table | Write denied `DeniedCompileTime` → `CAPABILITY_DENIED` (E-FLAG-002); runtime config cannot override |
| Declaration present | CrowdStrike sensor TOML spec declares a matching `[[write_endpoints]]` entry; loaded at boot | Compile tier passes; outcome then subject to the runtime capability check (BC-2.04.002) |
| Test-gating features | Build with `--all-features` vs default features | Identical production write gating; only `#[cfg]`-gated tests differ |

## Verification Properties

- **VP-020** (Feature flag: compile AND runtime must both permit) — verifies that the two-tier gate requires both compile-tier permission (registry-derived) and runtime flag enablement.

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-005 |
| L2 Invariants | DI-003 |
| Priority | P0 |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.2 | MCP cascade pass-1 P1-02 BC sibling sweep (2026-06-10 review-cycle PO micro-burst) | 2026-06-10 | product-owner | Contract synced to registry-derived compile-tier reality, aligned with error-taxonomy v1.67 E-FLAG-002 row and `write_pipeline.rs` (which cites this BC as the compile-tier anchor). H1 retitled "Compile-Time Cargo Features Gate Write Code Families" → "Compile-Time Write Capability Tier Is Registry-Derived from Write-Endpoint Declarations" (BC-INDEX title column synced in same burst; filename slug immutable per append_only_numbering). Postconditions rewritten: compile tier = `WriteEndpointRegistry` presence (`[[write_endpoints]]` in sensor TOML, BC-2.16.012), denial = `DeniedCompileTime` → E-FLAG-002; `{sensor}-write` Cargo features documented as empty test-gating declarations (per prism-query Cargo.toml, post-PLUGIN-MIGRATION-001-A; de-gating tracked for PLUGIN-MIGRATION-001-F); cfg-exclusion "code physically absent" claims removed (false post-migration); alias-write (BC-2.11.008/BC-2.11.006) documented as the surviving genuinely cfg-derived case. Error case "conditional compilation error" replaced with E-FLAG-002 structured denial. EC-04-001/EC-04-002 and test vectors restated to declaration-present/absent semantics (EC IDs preserved). DI-003 invariant wording updated to declaration-based deny-by-default. |
| 1.1 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added extracted_from/inputs/input-hash/traces_to frontmatter; added ## Description synthesized from body; added ## Canonical Test Vectors scaffolding; added ## Verification Properties cross-ref; added ## Changelog. |
