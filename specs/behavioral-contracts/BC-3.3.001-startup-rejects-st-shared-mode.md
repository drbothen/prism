---
document_type: behavioral-contract
level: L3
version: "0.6"
status: PROPOSED
producer: product-owner
timestamp: 2026-04-27T00:00:00
phase: 3.A
inputs:
  - .factory/specs/architecture/decisions/ADR-007-configurable-dtu-mode.md
  - .factory/specs/architecture/decisions/ADR-010-customer-config-schema.md
input-hash: "010087a"
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

If a `[[dtu]]` config block declares a Security Telemetry type (claroty, armis, crowdstrike, cyberint, demo-server) with `mode = "shared"`, the process must refuse to start and must emit `E-CFG-017` — a diagnostic error that names the offending DTU type and the config block location. This guard prevents cross-tenant data leakage that would result from sharing a client-mode DTU instance across organizations. **Wave 3 status: the guard is unconditional — `allow_shared_override` is NOT IMPLEMENTED in Wave 3** (deferred to Wave 4 per ADR-007 §7 OQ-1, locked as DEFERRED). Any `allow_shared_override` field in a `customers/*.toml` file is rejected as an unknown field (`E-CFG-010` from BC-3.3.004 R-CUST-010) because `deny_unknown_fields` is applied by serde at parse time.

## Preconditions

1. The startup pipeline reads and validates `customers/*.toml` before binding the MCP transport.
2. The `DTU_DEFAULT_MODE` registry (ADR-007 §2.3) classifies each DTU type as Security Telemetry or MSSP Coordination.
3. Validation is multi-error: all violations across all config files are reported in one pass before aborting, consistent with CAP-009 multi-error reporting.
4. `allow_shared_override` is NOT implemented in Wave 3; the guard is unconditional. Any `allow_shared_override` field present in a `customers/*.toml` file is rejected by `deny_unknown_fields` serde deserialization as `E-CFG-010` (unknown field). See ADR-007 §7 OQ-1 (DEFERRED to Wave 4).

## Postconditions

1. If any `[[dtu]]` block has a Security Telemetry type with `mode = "shared"`: the process does not start; the MCP stdio transport is never bound; `E-CFG-017` is emitted naming the offending type and config block.
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
| EC-001 | claroty DTU with mode = "shared" | `E-CFG-017`: "DTU type 'claroty' is a Security Telemetry type and must be configured with mode=client" |
| EC-002 | armis DTU with mode = "shared" | `E-CFG-017`: same error pattern as EC-001 for armis |
| EC-003 | slack DTU with mode = "client" | Permitted: MSSP Coordination type with client override is valid; no error or warning |
| EC-004 | slack DTU with mode = "shared" | No error: shared is the default for slack; this is the expected configuration |
| EC-005 | Two config files: one with claroty+shared, one with crowdstrike+shared | Both errors reported in one pass; process does not start |
| EC-006 | Unknown DTU type with any mode | Separate error: "Unknown DTU type 'foo-sensor'"; process does not start |
| EC-007 | claroty DTU with mode = "client" | No error; this is the required configuration for Security Telemetry types |
| EC-008 | demo-server DTU with mode = "shared" in production config | Two errors emitted in one validation pass (multi-error postcondition): `E-CFG-013` (test-only type not permitted in production config) AND `E-CFG-017` (Security Telemetry type with shared mode rejected). Both error codes are reported before the process exits. |

## Canonical Test Vectors

| TV-ID | Inputs | Expected Outputs | Notes |
|-------|--------|-----------------|-------|
| TV-3.3.001-01 | `[[dtu]] type="claroty" mode="shared"` in customer TOML | Startup error naming "claroty" and the file path; process does not start | Core guard — ST + shared |
| TV-3.3.001-02 | `[[dtu]] type="claroty" mode="client"` | No error; process starts normally | Correct ST configuration |
| TV-3.3.001-03 | `[[dtu]] type="slack" mode="client"` | No error; MSSP Coordination type with client override is valid | Permitted override |
| TV-3.3.001-04 | Two config files each with a different ST type in shared mode | Both errors reported; process does not start | Multi-error reporting |
| TV-3.3.001-05 | `[[dtu]] type="demo-server" mode="shared"` | Startup error; demo-server is Security Telemetry | demo-server guard |
| TV-3.3.001-06 | `[[dtu]] type="claroty" mode="client" allow_shared_override=true` in `customers/*.toml` | Exit 1; `E-CFG-010` unknown field rejection; `allow_shared_override` is not a valid Wave 3 field | Wave 4 field rejection |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-095 / VP-3.3.001-01 | Every Security Telemetry type in DTU_DEFAULT_MODE triggers startup error when paired with mode=shared | unit_test (iterate DTU_DEFAULT_MODE Security Telemetry entries; assert each produces startup Err with mode=shared) |
| VP-096 / VP-3.3.001-02 | No MSSP Coordination type triggers startup error when paired with mode=client | unit_test (iterate MSSP Coordination entries; assert each produces startup Ok with mode=client) |
| VP-097 / VP-3.3.001-03 | Error message contains the DTU type string and config file path | unit_test (inspect error message fields) |
| VP-098 / VP-3.3.001-04 | Multi-error: N violations in N config files produce N errors in one pass before abort | unit_test (construct N violating configs; assert N errors reported) |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-009 ("Client Configuration") per capabilities.md §CAP-009 |
| Capability Anchor Justification | CAP-009 ("Client Configuration") per capabilities.md §CAP-009 — this BC is a config validation rule: "Validation is multi-error: all problems reported in one pass. Missing required fields produce actionable error messages specifying the exact config path." BC-3.3.001 extends config validation with a Security Telemetry + shared-mode guard that is structurally the same as other startup validation rules in CAP-009. |
| L2 Domain Invariants | n/a (Wave 3 greenfield) |
| Architecture Module | `prism-config` or startup validation in `prism-spec-engine` (ADR-007 §2.4) |
| ADR Source | ADR-007 §2.1 (DTU type classification), §2.3 (DTU_DEFAULT_MODE registry), §2.4 (config validation rules, rule 3), §3.1 (sensor mode misconfiguration threat) |
| Stories | S-3.3.01 |

## Related BCs

- BC-3.2.005 — composes with (mode is deployment-time only; this BC is the fail-closed guard for the most dangerous misconfiguration)
- BC-3.2.001 — depends on (the guard prevents the state-level isolation violation that would result from a shared client-mode DTU)

## Architecture Anchors

- ADR-007 §2.3 — `DTU_DEFAULT_MODE` static constant (classification source of truth)
- ADR-007 §2.4 — validation rule 3: Security Telemetry + shared mode = startup error
- ADR-007 §3.1 — sensor mode misconfiguration threat model

## Story Anchor

S-3.3.01

## VP Anchors

- VP-095 / VP-3.3.001-01 — every ST type triggers error with shared mode
- VP-096 / VP-3.3.001-02 — no MSSP Coordination type errors with client mode
- VP-097 / VP-3.3.001-03 — error message contains type string and file path
- VP-098 / VP-3.3.001-04 — multi-error reporting across N violations

## Open Questions

None. All open questions resolved.

- `allow_shared_override` in Wave 3: **DEFERRED to Wave 4** (ADR-007 §7 OQ-1, locked). Wave 3 ST guard is unconditional. Any `allow_shared_override = true` in `customers/*.toml` produces `E-CFG-010` (unknown field) due to `deny_unknown_fields`. TV-3.3.001-06 asserts this behavior.
- `demo-server` in `DTU_DEFAULT_MODE`: **Resolved via D-051** — `demo-server` appears in `DTU_DEFAULT_MODE` with `test_only = true` annotation. The production config validator rejects `type = "demo-server"` via an absence-check against the production-allowed set (not an explicit denylist). EC-008 and TV-3.3.001-05 remain valid.

## BC Changelog

| Version | Change |
|---------|--------|
| v0.6 | Pass 6 fixes: m-002: VP table and VP Anchors updated to dual form (VP-095/VP-3.3.001-01 through VP-098/VP-3.3.001-04) matching BC-3.3.004/3.4.004 pattern. m-003: ADR-010 added to inputs list (BC references BC-3.3.004 R-CUST-010/E-CFG-010 which lives in ADR-010 schema). m-005: E-CFG-017 assigned for "Security Telemetry type with shared mode rejected" — EC-008 updated to name E-CFG-017 for the ST+shared guard error; EC-001/EC-002 updated; Description and Postcondition 1 updated with E-CFG-017 code. E-CFG-017 added to error-taxonomy.md v1.10. |
| v0.5 | M-006 fix (2026-04-27): VP proof method labels updated from "unit test (iterate ...)" to "unit_test" — VP-INDEX VP-095..098 are the source of truth (proptest→unit_test per M-006 resolution); BC body now matches. m-002 fix: EC-008 (demo-server+shared) and EC-001..EC-007 already have error codes; no additional citation needed beyond existing text. |
| v0.4 | m-007 fix (2026-04-27): Story Anchor updated from TBD to S-3.3.01 (per STORY-INDEX mapping). |
| v0.3 | C-1/C-2 sync (2026-04-27): Description updated to explicitly state Wave 3 ST guard is unconditional / `allow_shared_override` NOT IMPLEMENTED; Precondition 4 updated with `E-CFG-010` reference; TV-3.3.001-06 added (allow_shared_override field rejected as E-CFG-010); OQs resolved per D-051 (demo-server) and ADR-007 §7 OQ-1 DEFERRED (allow_shared_override); ADR-007 deferred section reference added. |
| v0.2 | Initial authoring from ADR-007. |
