---
document_type: behavioral-contract
level: L3
version: "0.5"
status: PROPOSED
producer: product-owner
timestamp: 2026-04-27T00:00:00
phase: 3.A
inputs: [.factory/specs/architecture/decisions/ADR-006-multi-tenant-dtu-topology.md, .factory/specs/architecture/decisions/ADR-007-configurable-dtu-mode.md]
input-hash: ""
traces_to: .factory/specs/architecture/decisions/ADR-007-configurable-dtu-mode.md
origin: greenfield
extracted_from: null
subsystem: SS-06
capability: CAP-040
lifecycle_status: active
introduced: v3.0.0
modified: []
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
bc_id: BC-3.2.005
title: DTU mode is deployment-time config — no runtime API to change it
wave: 3
related_decisions: [D-042, D-045]
related_adrs: [ADR-006, ADR-007]
inherits_from: null
superseded_by: null
---

# BC-3.2.005: DTU mode is deployment-time config — no runtime API to change it

## Description

The `mode` field (`"shared"` or `"client"`) in each `[[dtu]]` block of a customer TOML config is a deployment-time decision. It is parsed once at startup, stored immutably in the in-memory sensor spec registry as an enum (`DtuMode::Shared` or `DtuMode::Client`), and never changed by any MCP tool, HTTP endpoint, or administrative API for the lifetime of the process. The only path to change a DTU's mode is to edit the config file and restart the process. Config validation rejects any value other than `"shared"` or `"client"`. The `allow_shared_override` escape hatch is deferred to Wave 4 and is NOT part of this contract.

## Preconditions

1. The customer TOML config contains a `[[dtu]]` block with an explicit `mode` field.
2. `mode` must be one of the string literals `"shared"` or `"client"` (case-sensitive). Any other value is a startup validation error.
3. The `DtuMode` type is an enum (`Shared` | `Client`), not a `String`, so serde validates the value space at parse time.
4. The mode is stored in the sensor spec registration struct (e.g., `SensorSpec.mode: DtuMode`) as a non-mutable field.

## Postconditions

1. A config with `mode = "shared"` results in the DTU instance being registered and dispatched as a shared-mode adapter for the lifetime of the process.
2. A config with `mode = "client"` results in the DTU instance being registered and dispatched as a client-mode adapter for the lifetime of the process.
3. A config with `mode = "Hybrid"` (or any value other than `"shared"` or `"client"`) causes a startup validation error with a message identifying the offending `[[dtu]]` block; the process does not start.
4. No MCP tool, HTTP endpoint, or in-process API exists that changes a sensor's mode at runtime. A runtime mode-change attempt is impossible at the API level — there is no method on the registry or adapter that accepts a new `DtuMode` value after startup.
5. A config file edit followed by process restart applies the new mode. In-flight mode changes without restart are never observed.

## Invariants

1. `DtuMode` is `#[derive(Debug, Clone, Copy, PartialEq, Eq)]` — it is a value type with no interior mutability.
2. The `mode` field in `SensorSpec` (or equivalent registration struct) is set exactly once, at startup parse time, and has no setter method.
3. The `BehavioralClone` trait and DTU HTTP API layer (per ADR-001, ADR-003) expose no mode-change endpoint. No `POST /dtu/configure` or `PUT /dtu/mode` endpoint exists.
4. Hot config reload (`reload_config` MCP tool, CAP-030) explicitly excludes mode as a reloadable field: mode changes in a reloaded config file are detected and reported as warnings but not applied.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Config has `mode = "shared"` for a `claroty` DTU (Security Telemetry type) | Startup error: "DTU type 'claroty' is a Security Telemetry type and must be mode=client" (ADR-007 §2.4 rule 3). Process does not start. |
| EC-002 | Config has `mode = "client"` for a `slack` DTU (MSSP Coordination type) | Loads successfully; no warning. Operator override of shared default is permitted for MSSP Coordination types. |
| EC-003 | Config has `mode = "Hybrid"` | Startup error: serde deserialization fails; error identifies the offending key and value; process does not start. |
| EC-004 | Analyst sends MCP tool call attempting to change mode at runtime | No such tool exists; the tool call returns an MCP error indicating the tool is not found. |
| EC-005 | Config file edited to change mode from "client" to "shared"; process is NOT restarted | Running process continues with original mode for the full session; the file change has no effect until restart. |
| EC-006 | `reload_config` is called after editing mode in TOML | Reload detects the mode change, emits a warning that mode changes require restart, but does not apply the change; running mode is preserved. |

## Canonical Test Vectors

| TV-ID | Inputs | Expected Outputs | Notes |
|-------|--------|-----------------|-------|
| TV-3.2.005-01 | TOML with `mode = "shared"` for a slack DTU | Process starts; slack DTU registered as DtuMode::Shared | Happy path shared |
| TV-3.2.005-02 | TOML with `mode = "client"` for a claroty DTU | Process starts; claroty DTU registered as DtuMode::Client | Happy path client |
| TV-3.2.005-03 | TOML with `mode = "Hybrid"` for any DTU | Startup error naming the offending block; process does not start | Invalid mode string |
| TV-3.2.005-04 | TOML with `mode = "shared"` for claroty (Security Telemetry) | Startup error: Security Telemetry type must be client; process does not start | ST+shared guard |
| TV-3.2.005-05 | Config file edited to change mode; process NOT restarted; check effective mode | DtuMode unchanged from startup value | Mode immutability |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-091 / VP-3.2.005-01 | DtuMode enum has no setter: no public method on SensorSpec or registry accepts DtuMode after startup | proptest (static analysis — grep for setter patterns on DtuMode field) |
| VP-092 / VP-3.2.005-02 | Startup rejects unknown mode values: serde deserialization of any non-"shared"/non-"client" string returns Err | proptest (serde roundtrip with adversarial inputs) |
| VP-093 / VP-3.2.005-03 | Security Telemetry type with mode=shared causes startup error | proptest (attempt to load such a config; assert process-start Err) |
| VP-094 / VP-3.2.005-04 | reload_config does not apply mode changes | integration test (reload after mode edit; verify DtuMode unchanged) |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-040 ("Multi-Tenant Adapter Dispatch Mode") per capabilities.md §CAP-040 |
| Capability Anchor Justification | CAP-040 ("Multi-Tenant Adapter Dispatch Mode") per capabilities.md §CAP-040 — this BC specifies that "Mode is a deployment-time-only declaration read from `[[dtu]]` TOML blocks, stored immutably as `enum DtuMode { Client, Shared }`, and never changed by any runtime API," which is the exact mode-immutability guarantee CAP-040 defines. |
| L2 Domain Invariants | n/a (Wave 3 greenfield) |
| Architecture Module | `prism-config` or startup pipeline in `prism-spec-engine` (ADR-007 §2.4) |
| ADR Source | ADR-006 §2.4 (configurable shared/client mode), ADR-007 §2.3 (default mode registry), §2.4 (config schema and validation rules), §2.5 (mode change semantics and enforcement) |
| Stories | S-3.0.02, S-3.2.05, S-3.2.06, S-3.2.07, S-3.3.06 |

## Related BCs

- BC-3.2.004 — depends on (shared-mode payload tagging only applies when mode=shared; this BC defines that mode is fixed at startup)
- BC-3.2.001 — depends on (per-org state keying only applies to client-mode DTUs; mode determines which crates are in scope)

## Architecture Anchors

- ADR-007 §2.3 — `DTU_DEFAULT_MODE` static registry constant
- ADR-007 §2.4 — config schema validation rules 1-5
- ADR-007 §2.5 — mode change semantics and enforcement chain
- `crates/prism-dtu-common/src/config.rs` — `StubConfig`; mode field to be added to sensor registration struct

## Story Anchor

S-3.0.02, S-3.2.05, S-3.2.06, S-3.2.07, S-3.3.06

## VP Anchors

- VP-091 (VP-3.2.005-01) — proptest: no DtuMode setter exists post-startup
- VP-092 (VP-3.2.005-02) — proptest: unknown mode values rejected at startup
- VP-093 (VP-3.2.005-03) — proptest: Security Telemetry + shared mode causes startup error
- VP-094 (VP-3.2.005-04) — integration_test: reload_config does not apply mode changes

## Open Questions

None. All open questions resolved.

- `allow_shared_override` escape hatch: **DEFERRED to Wave 4** (ADR-007 §7 OQ-1, locked). Wave 3 ST guard is unconditional. Any `allow_shared_override` field in `customers/*.toml` produces `E-CFG-010` (unknown field via `deny_unknown_fields`). This BC does not need revision for Wave 3.
- `mode: DtuMode` placement in `SensorSpec` vs `DtuInstanceSpec`: still open as an implementation-level decision for the implementing story. Both approaches satisfy this BC's postconditions (mode is immutable post-startup regardless of which struct holds it).

## BC Changelog

| Version | Change |
|---------|--------|
| v0.5 | m-002 (Pass 4): Verification Properties table and VP Anchors section updated to include flat VP-NNN IDs alongside dotted forms (VP-091/VP-3.2.005-01 through VP-094/VP-3.2.005-04). Proof methods aligned with VP-INDEX method column (proptest for VP-091..093, integration_test for VP-094). |
| v0.4 | M-003 (Pass 3): Stories field and Story Anchor resolved from TBD to S-3.0.02, S-3.2.05, S-3.2.06, S-3.2.07, S-3.3.06 per STORY-INDEX BC Traceability Matrix. |
| v0.3 | C-2 sync (2026-04-27): Open Questions updated — allow_shared_override explicitly DEFERRED to Wave 4 per ADR-007 §7 OQ-1; added reference to ADR-007 deferred section. |
| v0.2 | Initial authoring from ADR-006, ADR-007. |
