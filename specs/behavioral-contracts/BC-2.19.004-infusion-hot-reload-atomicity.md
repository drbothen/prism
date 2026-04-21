---
document_type: behavioral-contract
level: L3
version: "1.3"
status: draft
producer: product-owner
timestamp: 2026-04-16T12:00:00
phase: 2-patch
origin: greenfield
subsystem: "SS-19"
capability: "CAP-030"
lifecycle_status: active
introduced: cycle-1
modified: 2026-04-20
deprecated: ~
deprecated_by: ~
replacement: ~
retired: ~
removed: ~
removal_reason: ~
inputs:
  - ".factory/specs/prd.md"
  - ".factory/specs/domain-spec/capabilities.md"
input-hash: "365fb25"
traces_to: ["CAP-030", "CAP-031"]
extracted_from: ".factory/specs/prd.md"
---

# BC-2.19.004: Infusion Hot Reload — Failed Validation Retains Previous Registration (CI-002)

## Description

When an `.infusion.toml` spec file changes on disk (detected by the `notify` file
watcher), the updated spec is validated before being applied. If validation fails,
the previous `InfusionRegistry` state is retained unchanged — no UDFs are deregistered
and no partial state is applied. In-flight queries using the old registry complete
without error. This is the CI-002 hot reload invariant applied to infusions (INV-INFUSE-004).

## Preconditions

- `InfusionRegistry` is operational with a valid spec loaded
- The `notify` file watcher detects a change to `{config_dir}/infusions/*.infusion.toml`
  or a data file in `{config_dir}/data/`
- The new spec fails validation (invalid TOML, missing fields, duplicate UDF name, etc.)

## Postconditions

- **Validation failure:**
  - The `InfusionRegistry` arc-swap is NOT executed
  - The previous registry state remains active
  - An `ERROR`-level log: `"Infusion spec '{path}' hot-reload failed: {error}. Previous registration retained."`
  - In-flight queries continue using the old registry and UDF descriptors
- **Validation success:**
  - The new `InfusionRegistry` is swapped in via arc-swap atomically
  - `prism-query` is notified to deregister old UDFs and register new ones
  - In-flight queries using old `Arc<InfusionRegistry>` complete without error (Arc lifetime)
  - Data file changes (MMDB, CSV, JSON): source data reloaded into new reader, arc-swapped;
    old reader stays alive until in-flight queries complete
- The reload is all-or-nothing per spec file: a single invalid field in one spec does not
  partially apply other fields from that spec

## Invariants

- INV-INFUSE-004: Hot reload atomicity — failed spec validation retains the previous registration (CI-002 pattern)
- `InfusionRegistry` MUST use `arc_swap::ArcSwap` for hot reload — NOT `RwLock`
- The previous `Arc<InfusionRegistry>` is retained until all in-flight queries drop their
  Arc references

## Error Conditions

| Error | Condition | Behavior |
|-------|-----------|----------|
| — | Invalid TOML syntax in updated spec file | Validation fails; previous registry retained; `ERROR` log |
| — | Hot-reloaded spec introduces duplicate UDF name (`E-INFUSE-002`) | Validation fails; previous registry retained; `E-INFUSE-002` in log |
| — | MMDB data file changes (valid file) | Data reload succeeds; new reader arc-swapped; old reader alive for in-flight queries |

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-19-013 | Spec hot-reloaded with 2 new fields added | Both new UDFs registered in `prism-query`; old UDFs still present (accumulative); no deregistration of existing UDFs |
| EC-19-014 | Spec hot-reloaded with one field removed | Old UDF must be deregistered from DataFusion `SessionContext`; new sessions after reload do not have the removed UDF |
| EC-19-015 | Spec deleted from disk | All UDFs from that spec are deregistered; `InfusionRegistry` updated; in-flight queries complete |
| EC-19-016 | Valid spec reload while 100 concurrent queries use it | All 100 queries complete using old registry; new queries after swap use new registry |

## Canonical Test Vectors

| ID | Input | Expected Output | Notes |
|----|-------|----------------|-------|
| TV-19-004-happy | Valid spec modification; 0 in-flight queries | New registry active; new UDFs registered | Baseline |
| TV-19-004-fail | Invalid TOML in updated spec | Previous registry retained; ERROR log | CI-002 |
| TV-19-004-inflight | Valid reload; 100 concurrent queries | All 100 complete on old registry; new queries use new registry | EC-19-016 |
| TV-19-004-dupudf | Reload introduces duplicate UDF name | Reload rejected; `E-INFUSE-002`; previous registry retained | Error row 2 |

## Verification Properties

| VP ID | Description | Verification Method |
|-------|-------------|---------------------|
| (none) | CI-002 arc-swap hot reload pattern proven by VP-032; InfusionRegistry uses same arc-swap wrapper; if UDF deregistration logic diverges from VP-032's coverage, revisit during Phase 3 | — |

## Related BCs

- BC-2.19.001 — Infusion Spec Loading (governs what validation checks apply)
- BC-2.16.007 — Sensor Spec Hot Reload (same CI-002 pattern for sensor specs)
- BC-2.17.005 — Plugin Hot Reload (same pattern for WASM plugins)

## Architecture Anchors

- AD-007: arc-swap for hot config reload
- AD-018: Automatic filesystem watching for config reload
- AD-020: Infusions — hot reload
- `specs/architecture/infusions.md` — CI-002 hot reload invariant
- S-1.14 Task 9: `infusion` hot reload participation

## Story Anchor

S-1.14 — prism-spec-engine: Infusion Spec Loading and UDF Registration (INV-INFUSE-004, AC-5)

## VP Anchors

Integration test: `tests/infusion_tests.rs` — "Verify hot reload: modify spec, trigger watch event, verify new UDF registration, old queries still complete."

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-030, CAP-031 (dual anchor) |
| Story Invariant | INV-INFUSE-004 |
| ADR | AD-007, AD-018, AD-020 |
| Story | S-1.14 |
| Priority | P0 |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.3 | pass-69-housekeeping | 2026-04-20 | product-owner | Normalized changelog schema to canonical 5-col Version | Burst | Date | Author | Change form. |
| 1.2 | pass-69-housekeeping | 2026-04-20 | product-owner | Resolved VP-TBD placeholder per decision matrix (MARK-NONE); normalized changelog schema to canonical 5-col form. |
| 1.1 | Wave-6-pre-build-sweep | 2026-04-20 | product-owner | Added frontmatter (inputs, input-hash, traces_to, extracted_from, lifecycle fields); fixed malformed `capability:` frontmatter (was YAML array `[CAP-030, CAP-031]`, normalized to string `"CAP-030"` as primary with secondary noted in traces_to); renamed Error Cases → Error Conditions; added Canonical Test Vectors, Verification Properties, Changelog |
| 1.0 | Phase-2 | 2026-04-16 | product-owner | Initial contract |
