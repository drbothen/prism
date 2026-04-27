---
document_type: behavioral-contract
level: L3
version: "0.1"
status: PROPOSED
producer: product-owner
timestamp: 2026-04-27T00:00:00
phase: 3.A
inputs: [.factory/specs/architecture/decisions/ADR-007-configurable-dtu-mode.md]
input-hash: ""
traces_to: .factory/specs/architecture/decisions/ADR-007-configurable-dtu-mode.md
origin: greenfield
extracted_from: null
subsystem: SS-06
capability: CAP-009
lifecycle_status: active
introduced: v3.0.0
modified: []
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
bc_id: BC-3.3.001
title: Startup rejects Security Telemetry DTU type declared with shared mode
wave: 3
related_decisions: [D-042, D-045]
related_adrs: [ADR-007]
inherits_from: null
superseded_by: null
---

# BC-3.3.001: Startup rejects Security Telemetry DTU type declared with shared mode

## Description

If a `[[dtu]]` config block declares a Security Telemetry type (claroty, armis, crowdstrike, cyberint, demo-server) with `mode = "shared"`, the process must refuse to start and must emit a diagnostic error that names the offending DTU type and the config block location. This guard prevents cross-tenant data leakage that would result from sharing a client-mode DTU instance across organizations. The `allow_shared_override` escape hatch is deferred to Wave 4 and is not implemented by this contract — the guard is unconditional in Wave 3.

## Preconditions

1. The startup pipeline reads and validates `customers/*.toml` before binding the MCP transport.
2. The `DTU_DEFAULT_MODE` registry (ADR-007 §2.3) classifies each DTU type as Security Telemetry or MSSP Coordination.
3. Validation is multi-error: all violations across all config files are reported in one pass before aborting, consistent with CAP-009 multi-error reporting.
4. `allow_shared_override` is NOT implemented in Wave 3; the guard is unconditional.

## Postconditions

1. If any `[[dtu]]` block has a Security Telemetry type with `mode = "shared"`: the process does not start; the MCP stdio transport is never bound; a diagnostic error is emitted naming the offending type and config block.
2. The diagnostic error message includes: the DTU type string, the config file path, and an actionable suggestion to set `mode = "client"`.
3. All other valid `[[dtu]]` blocks are validated in the same pass; their errors (if any) are also reported before the process exits.
4. If no Security Telemetry type has `mode = "shared"`: startup proceeds normally; this validation rule is a no-op.
5. An MSSP Coordination type (slack, pagerduty, jira, nvd, threatintel) with `mode = "client"` passes this validation rule without error (overriding the shared default is permitted for MSSP Coordination types).

## Invariants

1. The validation runs before any adapter is instantiated — no state is created for a DTU block that fails this check.
2. The classification source of truth is `DTU_DEFAULT_MODE` (a compile-time constant); it cannot be overridden by config at runtime.
3. An unknown DTU type (not in `DTU_DEFAULT_MODE`) is a separate startup error (ADR-007 §2.4 rule 1) and is also caught in the same validation pass.
4. The guard is fail-closed: when in doubt (unknown type, malformed mode value), the process refuses to start.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | claroty DTU with mode = "shared" | Startup error: "DTU type 'claroty' is a Security Telemetry type and must be configured with mode=client" |
| EC-002 | armis DTU with mode = "shared" | Same error pattern as EC-001 for armis |
| EC-003 | slack DTU with mode = "client" | Permitted: MSSP Coordination type with client override is valid; no error or warning |
| EC-004 | slack DTU with mode = "shared" | No error: shared is the default for slack; this is the expected configuration |
| EC-005 | Two config files: one with claroty+shared, one with crowdstrike+shared | Both errors reported in one pass; process does not start |
| EC-006 | Unknown DTU type with any mode | Separate error: "Unknown DTU type 'foo-sensor'"; process does not start |
| EC-007 | claroty DTU with mode = "client" | No error; this is the required configuration for Security Telemetry types |
| EC-008 | demo-server DTU with mode = "shared" | Startup error: demo-server is classified as Security Telemetry (ADR-007 §2.1); shared mode rejected |

## Canonical Test Vectors

| TV-ID | Inputs | Expected Outputs | Notes |
|-------|--------|-----------------|-------|
| TV-3.3.001-01 | `[[dtu]] type="claroty" mode="shared"` in customer TOML | Startup error naming "claroty" and the file path; process does not start | Core guard — ST + shared |
| TV-3.3.001-02 | `[[dtu]] type="claroty" mode="client"` | No error; process starts normally | Correct ST configuration |
| TV-3.3.001-03 | `[[dtu]] type="slack" mode="client"` | No error; MSSP Coordination type with client override is valid | Permitted override |
| TV-3.3.001-04 | Two config files each with a different ST type in shared mode | Both errors reported; process does not start | Multi-error reporting |
| TV-3.3.001-05 | `[[dtu]] type="demo-server" mode="shared"` | Startup error; demo-server is Security Telemetry | demo-server guard |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-3.3.001-01 | Every Security Telemetry type in DTU_DEFAULT_MODE triggers startup error when paired with mode=shared | unit test (iterate DTU_DEFAULT_MODE Security Telemetry entries; assert each produces startup Err with mode=shared) |
| VP-3.3.001-02 | No MSSP Coordination type triggers startup error when paired with mode=client | unit test (iterate MSSP Coordination entries; assert each produces startup Ok with mode=client) |
| VP-3.3.001-03 | Error message contains the DTU type string and config file path | unit test (inspect error message fields) |
| VP-3.3.001-04 | Multi-error: N violations in N config files produce N errors in one pass before abort | unit test (construct N violating configs; assert N errors reported) |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-009 ("Client Configuration") per capabilities.md §CAP-009 |
| Capability Anchor Justification | CAP-009 ("Client Configuration") per capabilities.md §CAP-009 — this BC is a config validation rule: "Validation is multi-error: all problems reported in one pass. Missing required fields produce actionable error messages specifying the exact config path." BC-3.3.001 extends config validation with a Security Telemetry + shared-mode guard that is structurally the same as other startup validation rules in CAP-009. |
| L2 Domain Invariants | n/a (Wave 3 greenfield) |
| Architecture Module | `prism-config` or startup validation in `prism-spec-engine` (ADR-007 §2.4) |
| ADR Source | ADR-007 §2.1 (DTU type classification), §2.3 (DTU_DEFAULT_MODE registry), §2.4 (config validation rules, rule 3), §3.1 (sensor mode misconfiguration threat) |
| Stories | TBD (filled by story-writer) |

## Related BCs

- BC-3.2.005 — composes with (mode is deployment-time only; this BC is the fail-closed guard for the most dangerous misconfiguration)
- BC-3.2.001 — depends on (the guard prevents the state-level isolation violation that would result from a shared client-mode DTU)

## Architecture Anchors

- ADR-007 §2.3 — `DTU_DEFAULT_MODE` static constant (classification source of truth)
- ADR-007 §2.4 — validation rule 3: Security Telemetry + shared mode = startup error
- ADR-007 §3.1 — sensor mode misconfiguration threat model

## Story Anchor

TBD — implementing story to be assigned by story-writer (Epic E-3.1, config validation sub-task)

## VP Anchors

- VP-3.3.001-01 — every ST type triggers error with shared mode
- VP-3.3.001-02 — no MSSP Coordination type errors with client mode
- VP-3.3.001-03 — error message contains type string and file path
- VP-3.3.001-04 — multi-error reporting across N violations

## Open Questions

- ADR-007 §8 Q1: should `allow_shared_override` be implemented in Wave 3 at all? This BC treats it as deferred to Wave 4. If a Wave 3 story requires it, this BC must be revised and the guard made conditional on `allow_shared_override = true` plus an audit-log warning at startup.
- ADR-007 §8 Q3: should `demo-server` appear in `DTU_DEFAULT_MODE` as Security Telemetry, or only in test configuration? EC-008 assumes it does appear; if that decision changes, EC-008 and TV-3.3.001-05 are no longer valid.
