---
document_type: behavioral-contract
level: L3
version: "1.1"
status: draft
producer: product-owner
timestamp: 2026-04-14T05:00:00
phase: 1a
origin: greenfield
subsystem: "SS-05"
capability: "CAP-007"
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
input-hash: "abc4070"
traces_to: ["CAP-007"]
extracted_from: ".factory/specs/prd.md"
---

# BC-2.05.008: Audit Entries Satisfy SOC 2 Type II and ISO 27001 Requirements

## Description

Audit entries produced by Prism satisfy both SOC 2 Type II and ISO 27001 evidence
requirements. SOC 2 Type II fields cover who (`user_identity`), what (`tool_name`,
`parameters`), when (`timestamp`), where (`client_id`, `sensor`), outcome
(`result_summary`), and authorization evidence (`capability_checks`). ISO 27001
fields include access control evidence (`capability_checks`), incident response
support (`trace_id`), and credential access records (BC-2.05.005). All fields are
machine-parseable (structured JSON, not free-text prose). Missing `user_identity` is
handled gracefully with a warning rather than blocking the entry.

## Preconditions
- An MCP tool invocation has completed (success or failure)
- The audit middleware is constructing the final `AuditEntry`

## Postconditions
- **SOC 2 Type II** fields are present:
  - **Who**: `user_identity` identifies the analyst
  - **What**: `tool_name` and `parameters` (redacted) describe the action
  - **When**: `timestamp` records the time in ISO 8601 UTC
  - **Where**: `client_id` and `sensor` scope the action to a specific client and sensor
  - **Outcome**: `result_summary` records success, failure, or denial
  - **Authorization**: `capability_checks` records feature flag evaluations for write operations
- **ISO 27001** fields are present:
  - Access control evidence: `capability_checks` demonstrates least-privilege enforcement
  - Incident response support: `trace_id` enables correlation of events across a session
  - Credential access: credential operations include `event_type: "credential_access"` (per BC-2.05.005)
- All fields are machine-parseable (structured JSON, not free-text prose)

## Invariants
- DI-004: Audit completeness
- DI-003: Feature flag deny-by-default -- audit trail proves least-privilege enforcement

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| Missing `user_identity` | MCP session does not provide user identity | `user_identity` is set to `"unknown"` with an `audit_warning` noting the missing identity; the entry is still emitted |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-05-013 | Read-only tool invocation (no capability check needed) | `capability_checks` array is empty (not omitted); this is valid -- read ops do not require authorization evidence |
| EC-05-014 | Tool invocation that triggers multiple capability checks (e.g., a write that falls back through the flag hierarchy) | All evaluated capability paths are recorded in the `capability_checks` array, showing the full resolution chain |

## Canonical Test Vectors

See `.factory/specs/prd-supplements/test-vectors.md` for canonical test vectors for BC-2.05.008.

| Scenario | SOC 2 Check | ISO 27001 Check | Pass? |
|----------|------------|----------------|-------|
| Normal write invocation | All 6 SOC 2 fields present | `capability_checks` non-empty; `trace_id` present | Pass |
| Read invocation | All 6 SOC 2 fields present | `capability_checks: []` (empty, not omitted); `trace_id` present | Pass |
| Missing user identity | `user_identity: "unknown"` with `audit_warning` | Acceptable with warning | Pass with warning |

## Verification Properties

No VPs in VP-INDEX v1.5 directly verify SOC 2 / ISO 27001 field completeness. Placeholder for future VP (proptest: generated audit entries satisfy the compliance field checklist).

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-007 |
| L2 Invariants | DI-003, DI-004 |
| Priority | P0 |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.1 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added extracted_from/inputs/input-hash/traces_to frontmatter; added ## Description synthesized from body; added ## Canonical Test Vectors scaffolding; added ## Verification Properties cross-ref; added ## Changelog. |
