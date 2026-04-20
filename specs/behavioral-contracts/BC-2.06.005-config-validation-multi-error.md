---
document_type: behavioral-contract
level: L3
version: "1.1"
status: draft
producer: product-owner
timestamp: 2026-04-14T05:00:00
phase: 1a
origin: greenfield
subsystem: "SS-06"
capability: "CAP-009"
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

# BC-2.06.005: Configuration Validation Reports All Errors in One Pass

## Preconditions
- TOML configuration has been parsed into an unvalidated structure
- The config validator is checking all clients, sensors, credentials, and capabilities

## Postconditions
- Validation collects ALL errors before reporting, rather than failing on the first error
- The error output is a list of individual validation errors, each including:
  - The exact TOML path of the invalid field (e.g., `clients.acme.sensors.crowdstrike.api_base`)
  - A human-readable description of the problem
  - A suggestion for resolution
- If any validation errors exist, Prism exits with a non-zero exit code and prints the full error list
- If no validation errors exist, Prism proceeds to start normally
- **DI-029 cross-validation (WARN):** After validating individual fields, the config validator performs a cross-validation pass comparing each correlation and sequence rule's window duration against the interval of the schedule(s) that feed it. For each pair where `window < interval`, a WARN-level diagnostic is appended to the validation output: `"Correlation rule '<rule_name>' has a <window>s window but is fed by schedule '<schedule_name>' with <interval>s interval — detections may be missed between runs. Recommended: set window >= <interval>s or interval <= <window>s."` This warning does not prevent startup; the rule and schedule are both activated.

## Invariants
- None specific -- this is a usability contract for configuration experience
- DI-029: Correlation window vs schedule interval cross-validation is performed at config load time; mismatches produce WARN diagnostics (not errors)

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| Multiple errors | Client A has an invalid URL AND Client B has a missing credential ref AND defaults have an invalid capability path | All three errors are reported in a single output; the operator can fix all at once |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-06-007 | Config has 50+ validation errors (e.g., a template was copy-pasted without customization) | All 50+ errors are reported; no truncation. The first few errors should be the most actionable. |
| EC-06-008 | A single field has multiple problems (e.g., `api_base` is both empty and not a valid URL) | Each distinct problem is reported as a separate error entry for that field |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-009 |
| L2 Invariants | DI-029 |
| Priority | P0 |

## Changelog
| Version | Date | Burst | Change |
|---------|------|-------|--------|
| 1.0 | 2026-04-14 | cycle-1 | Initial contract |
| 1.1 | 2026-04-19 | deferred-cleanup-track-1 | Added DI-029 cross-validation postcondition (correlation window vs schedule interval WARN); added DI-029 invariant entry |
