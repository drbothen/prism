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

# BC-2.18.009: `${case.alert_ids_quoted}` Values Validated as UUID v7 Before Interpolation

## Description

When an action template references `${case.alert_ids_quoted}`, each alert ID in the
list is validated against UUID v7 format BEFORE interpolation into the template string.
Non-UUID values are dropped with a `WARN` log. If all values are dropped, an empty
string is produced (not an error). This validation protects the PrismQL query string
(and other downstream systems) from injection via alert IDs that contain malicious content.
This is INV-ACTION-009.

## Preconditions

- An action template contains the `${case.alert_ids_quoted}` variable
- The template is being rendered for a case-triggered action
- The case's `source_alert_ids` list contains one or more values

## Postconditions

- Each value in the alert ID list is validated against UUID v7 format using the `uuid`
  crate's `Uuid::parse_str()` and checking the version byte (version = 7)
- **Valid UUID v7:** Included in the interpolated output, quoted (e.g., `"'01905a7b-...'"`)
- **Non-UUID value:** Dropped from the output with:
  `WARN "Dropping non-UUID v7 value from alert_ids_quoted in action '{action_id}': '{value}'"`
  (Note: the value itself is included in the WARN log for debugging; it is NOT included
  in any audit entry or MCP response)
- **All values valid:** Interpolated output: `'uuid1', 'uuid2', 'uuid3'` (comma-separated,
  single-quoted, suitable for PrismQL IN clause)
- **All values invalid:** Interpolated output: `''` (empty string); WARN log per dropped value
- UUID v7 validation runs BEFORE injection scanning (BC-2.18.006) for this variable

## Invariants

- INV-ACTION-009: `${case.alert_ids_quoted}` values are validated as UUID v7 before interpolation; non-UUID values are dropped with WARN log
- Validation occurs BEFORE interpolation (not after string construction)
- The order of remaining valid UUIDs in the output matches their order in the case's alert_ids list
- UUID v4, v1, or other UUID versions are NOT accepted (v7 only, as all Prism alert IDs are UUID v7)

## Error Cases

| Error | Condition | Behavior |
|-------|-----------|----------|
| — | One non-UUID value in list of 5 | 1 value dropped with WARN; 4 values interpolated |
| — | All values are non-UUID | Empty string interpolated; WARN per dropped value; action still fires (empty IN clause may produce no results in PrismQL) |
| — | Alert ID list is empty (case has no linked alerts) | `${case.alert_ids_quoted}` interpolates to `''` (empty string); no WARN |

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-18-030 | Alert ID list: `["01905a7b-...", "not-a-uuid"]` | One valid UUID interpolated; "not-a-uuid" dropped with WARN; output: `'01905a7b-...'` (matches AC-10) |
| EC-18-031 | Alert ID list contains a UUID v4 (valid UUID format but wrong version) | Dropped: UUID v4 is not UUID v7. WARN logged. |
| EC-18-032 | `${case.alert_ids_quoted}` variable in a webhook template body | Validated identically; the webhook body may contain an empty string if all IDs invalid |
| EC-18-033 | Alert ID contains SQL injection payload (`'; DROP TABLE alerts; --`) | Not a valid UUID v7; dropped with WARN. The protection is UUID format validation, not SQL escaping. |
| EC-18-034 | 1000 alert IDs in the list | All validated sequentially; valid ones interpolated; performance bound by `O(n)` uuid parse |

## Related BCs

- BC-2.18.006 — Template Injection Scanning (runs after UUID validation for this variable)
- BC-2.13.005 — Alert Generation (alerts are created with UUID v7 IDs; this BC enforces that downstream)
- BC-2.14.001 — `create_case` (case links alerts by ID; valid cases only contain UUID v7 alert IDs in production)

## Architecture Anchors

- AD-021: Actions — `${case.alert_ids_quoted}` UUID v7 validation
- `specs/architecture/actions.md` — UUID v7 validation, injection protection
- S-4.08 Architecture Compliance: "`${case.alert_ids_quoted}` UUID v7 validation MUST occur before interpolation, not after"
- S-4.08 Task 5: `action/template.rs` — UUID v7 validation

## Story Anchor

S-4.08 — prism-operations: Action Delivery Framework (INV-ACTION-009, AC-10)

## VP Anchors

Integration test: `tests/action_tests.rs` — "`${case.alert_ids_quoted}` with `['01905a7b-...', 'not-a-uuid']` → only valid UUID v7 interpolated, 'not-a-uuid' dropped with WARN."

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-033 |
| Story Invariant | INV-ACTION-009 |
| ADR | AD-021 |
| Story | S-4.08 |
| Priority | P0 |
