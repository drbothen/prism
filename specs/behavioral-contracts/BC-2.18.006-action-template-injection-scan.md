---
document_type: behavioral-contract
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-04-16T12:00:00
phase: 3-patch
origin: greenfield
subsystem: "Action Delivery Engine"
capability: "CAP-033"
lifecycle_status: active
---

# BC-2.18.006: Action Template Variables from Sensor/Alert Data — Injection-Scanned Before Interpolation

## Description

Before any template variable sourced from sensor or alert data (`${alert.*}`, `${event.*}`,
`${case.*}`) is interpolated into an action template, it is run through the `InjectionScanner`
(BC-2.09.003). Detected patterns are recorded in `_safety_flags` on the audit log entry.
The variable value is still interpolated — data is NEVER stripped (flag, don't strip,
per BC-2.09.004). Variables from trusted internal sources (`${prism_version}`, `${date}`,
`${client_id}`) are exempt from scanning. This is INV-ACTION-006.

## Preconditions

- An action with a template body containing `${alert.*}`, `${event.*}`, or `${case.*}`
  variable references is being rendered
- The `InjectionScanner` (BC-2.09.003) is available in the `prism-security` crate
- Alert or case data has been fetched for template population

## Postconditions

- All `${alert.*}`, `${event.*}`, and `${case.*}` variable values are passed through
  `InjectionScanner::scan()` before interpolation
- **Detection:** If `InjectionScanner` detects a suspicious pattern in a variable value:
  - `_safety_flags` field is set on the audit log entry for this action delivery:
    `{ "field": "alert.description", "pattern": "ignore_previous_instructions", "value_hash": "sha256(...)" }`
  - The original (flagged) value is still interpolated into the template unchanged
- **No detection:** Template rendered normally; `_safety_flags` is empty array in audit entry
- Trusted internal variables (`${prism_version}`, `${date}`, `${client_id}`,
  `${action_id}`, `${rule_id}`) bypass scanning entirely

## Invariants

- INV-ACTION-006: Template variables from sensor/alert data are injection-scanned before interpolation
- Scan MUST run before interpolation (not after) — scanning the already-interpolated template
  is insufficient as the scanner may not recognize interpolated content
- Flagged content is NEVER dropped or modified — the analyst needs forensic completeness
- Injection scanning is synchronous within the template rendering step; it does not block
  on external calls
- The `_safety_flags` audit field is always present (empty array when no flags; never null)

## Error Cases

| Error | Condition | Behavior |
|-------|-----------|----------|
| — | `InjectionScanner` library call panics or throws | Log `ERROR`; treat scan result as "no flags detected"; interpolation proceeds with unscannable value |
| — | Variable value is > 10KB (very large field) | Scanner truncates input at 10KB for performance; logs `WARN "Injection scan input truncated for field '{field}'"` |

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-18-019 | `${alert.description}` contains "ignore previous instructions" | Flagged in `_safety_flags`; value interpolated unchanged; downstream AI consumer sees the warning |
| EC-18-020 | 50 template variables, 3 flagged | All 3 flags recorded in `_safety_flags` array; all 50 variables interpolated |
| EC-18-021 | Template contains only trusted internal variables (`${date}`, `${client_id}`) | No scanning; `_safety_flags: []` in audit entry |
| EC-18-022 | `${case.alert_ids_quoted}` variable (see also BC-2.18.009) | UUID v7 validation runs before injection scan; non-UUID values dropped first; remaining values scanned |

## Related BCs

- BC-2.09.003 — Suspicious Pattern Detection via Regex (the `InjectionScanner` implementation)
- BC-2.09.004 — Safety Flag Parallel Fields — Flag, Don't Strip (policy this BC enforces)
- BC-2.12.012 — RETIRED; this BC (BC-2.18.006) is the normative replacement
- BC-2.18.009 — UUID v7 Validation for `${case.alert_ids_quoted}` (runs before this scan)

## Architecture Anchors

- AD-021: Actions — template injection scanning
- `specs/architecture/actions.md` — template rendering, injection scan
- `specs/architecture/security-architecture.md` — InjectionScanner, flag-don't-strip policy
- S-4.08 Task 5: `action/template.rs` — injection scan integration
- S-4.08 AC-8: "InjectionScanner detects pattern, `_safety_flags` set on audit log, original value STILL interpolated"

## Story Anchor

S-4.08 — prism-operations: Action Delivery Framework (INV-ACTION-006, AC-8)

## VP Anchors

No dedicated VP. Covered by `tests/action_tests.rs` template rendering tests with injection patterns.

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-033 |
| Story Invariant | INV-ACTION-006 |
| ADR | AD-021 |
| Story | S-4.08 |
| Priority | P0 |
