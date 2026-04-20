---
document_type: behavioral-contract
level: L3
version: "1.2"
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
inputs: [".factory/specs/prd.md", ".factory/specs/domain-spec/capabilities.md"]
input-hash: "[pending-recompute]"
traces_to: ["CAP-009"]
extracted_from: ".factory/specs/prd.md"
---

# BC-2.06.005: Configuration Validation Reports All Errors in One Pass

## Description

The config validator performs a full sweep of all clients, sensors, credentials, and
capabilities before reporting any errors, collecting all problems into a list. Each error
includes the exact TOML path of the invalid field, a human-readable description, and a
suggestion for resolution. If any errors exist, Prism exits with a non-zero code printing
the full list. This multi-error approach lets operators fix all config problems at once
rather than iterating one error at a time.

A post-validation cross-validation pass (DI-029) checks correlation/sequence rule window
durations against the intervals of the schedules that feed them. Mismatches produce WARN
diagnostics (not errors) appended to the validation output; the rule and schedule are both
activated.

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

## Canonical Test Vectors

See `.factory/specs/prd-supplements/test-vectors.md` for canonical test vectors for BC-2.06.005.

| Scenario | Config Errors | Expected Output |
|----------|--------------|----------------|
| Single error | Invalid URL for Client A | 1 error reported; non-zero exit |
| Multi-error | Invalid URL + missing cred ref + bad capability path | 3 errors reported in one pass; non-zero exit |
| DI-029 warning | Correlation rule window 60s < schedule interval 120s | WARN diagnostic appended; Prism starts normally |
| No errors | Valid config | Prism starts; zero error output |

## Verification Properties

No VPs in VP-INDEX v1.5 directly verify multi-error collection behavior. Placeholder for future VP.

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-009 |
| L2 Invariants | DI-029 |
| Priority | P0 |

## Changelog
| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.0 | cycle-1 | 2026-04-14 | product-owner | Initial contract |
| 1.1 | deferred-cleanup-track-1 | 2026-04-19 | product-owner | Added DI-029 cross-validation postcondition (correlation window vs schedule interval WARN); added DI-029 invariant entry |
| 1.2 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added extracted_from/inputs/input-hash/traces_to frontmatter; added ## Description synthesized from body; added ## Canonical Test Vectors scaffolding; added ## Verification Properties cross-ref; appended ## Changelog row. |
