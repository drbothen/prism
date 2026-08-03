---
document_type: story
story_id: "DEFECT-SENSOR-ERROR-FLATTEN-001"
title: "All SpecEngineError variants flattened to Internal; HTTP 401 misreported as sensor_unreachable"
wave: "C"
epic_id: engine-defects
priority: P1
status: draft
version: "0.1"
severity: HIGH
level: engine
producer: story-writer
timestamp: "2026-08-03"
modified: "2026-08-03"
inputs:
  - .factory/planning/findings-remediation-2026-07-20/triage-capture.md
  - findings/prism-pql-deficiencies.md
origin_finding: "F9 (D-1889 triage 2026-07-20)"
cycle: "v1.0.0-greenfield"
phase: 3
track: "Platform Engineering"
behavioral_contracts:
  - BC-2.08.002
# BC status:
#   BC-2.08.002 (Auth Validity Check): status: active, lifecycle_status: active
# S-7.01: behavioral_contracts non-empty; status may advance to ready after ACs are authored.
verification_properties: []
depends_on: []
blocks: []
points: 0
risk: HIGH
holdout_scenarios: []
assumption_validations: []
risk_mitigations: []
---

# DEFECT-SENSOR-ERROR-FLATTEN-001: All SpecEngineError variants flattened to Internal; HTTP 401 misreported as sensor_unreachable

## Problem

All `SpecEngineError` variants (verified at `crates/prism-core/src/error.rs` and
`crates/prism-bin/src/spec_driven_adapter.rs`) are collapsed to `SpecEngineError::Internal`
at some point in the error propagation path before surfacing to the health check and
query response layers. Concretely, a sensor returning HTTP 401 (authentication failure)
is misreported as `"sensor_unreachable"` (`auth_valid: null`) rather than as an auth
failure (`auth_valid: false`). This violates BC-2.08.002 §Postconditions which requires
auth failure details — including `reason: "auth_failure"` for a 401 — to be correctly
propagated in the health response.

The consequence is that operators cannot distinguish a credential problem from a
network outage by reading the health output, degrading operability.

## Origin — D-1889 Triage (F9)

**Triage date:** 2026-07-20  
**Source findings:** `findings/prism-pql-deficiencies.md`  
**Triage capture:** `.factory/planning/findings-remediation-2026-07-20/triage-capture.md`
§Bucket-B table row F9

The finding reports that all `SpecEngineError` variants are flattened to `Internal`
before reaching the health reporting layer. A live-sensor HTTP 401 comes back as
`"sensor_unreachable"` in the health tool output — the most misleading possible
misclassification because it routes the operator to network diagnostics rather than
credential rotation. `SpecEngineError` is used in `prism-bin/src/spec_driven_adapter.rs`
and `prism-core/src/error.rs`; the specific flattening site is to be confirmed by the
implementer during AC decomposition.

## Authority

| Artifact | Verbatim Status | Relevant Clause |
|----------|-----------------|-----------------|
| BC-2.08.002 (Auth Validity Check) | `status: active` · `lifecycle_status: active` | §Postconditions — `auth_valid: false` + `reason: "auth_failure"` for HTTP 401; §Edge Cases EC-08-005 governs `auth_valid: null` only for genuinely unreachable sensors, not for authentication rejections |

No governing ADR has been identified for this defect. The fix is an error-propagation
correction within existing BC scope; an ADR amendment is not anticipated.

## Routing

Route per triage: **implementer + product-owner**

1. Product-owner confirms or amends BC-2.08.002 to explicitly enumerate the HTTP
   status codes that map to each `auth_valid` / `reason` combination (if not
   already enumerated)
2. Implementer locates the `SpecEngineError` flattening site, threads correct error
   variants through the propagation path, and closes the gap under TDD

No architect adjudication required before dispatch.

## Scope — NOT YET SPECIFIED

Acceptance criteria, Red Gate test enumeration (RG-001..RG-NNN), BC-5.38.001 density
check, `tdd_mode` declaration, task decomposition, and story-point estimate are deferred
to the product-owner (BC amendment if needed) and story-writer (AC decomposition). This
stub registers the defect as a trackable artifact. No implementation guidance is authored
here.

## Changelog

| Version | Date | Author | Change |
|---------|------|--------|--------|
| 0.1 | 2026-08-03 | story-writer | Initial registration stub from D-1889 triage (F9); no ACs or implementation guidance |
