---
document_type: behavioral-contract
level: L3
version: "1.0"
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
---

# BC-2.05.004: Write Operations Log Capability Check and Execution Outcome

## Preconditions
- An MCP tool invocation targets a write/mutation operation (containment, alert status update, device action)
- Feature flag evaluation has been performed for the operation

## Postconditions
- The `capability_checks` array in the audit entry contains at least one entry with:
  - `capability_path` (the evaluated feature flag path, e.g., `sensor.crowdstrike.containment`)
  - `compile_time_enabled` (boolean)
  - `runtime_enabled` (boolean)
  - `result` (`"permitted"` or `"denied"`)
- If the capability check passes and the write executes, `result_summary` includes the execution outcome (success, dry-run preview, or error)
- If the capability check denies the operation, `result_summary` records `"denied_by_capability_check"` with the specific capability path

## Invariants
- DI-004: Audit completeness -- write operations include capability check details
- DI-003: Feature flag deny-by-default -- audit trail confirms deny was the fallback

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| Capability denied | Feature flag denies the write operation | Audit entry includes the denial reason; tool returns structured error explaining the missing capability |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-05-006 | Write operation uses `dry_run: true` (reversible write default) | Audit entry records `result_summary: "dry_run_preview"` with the preview content; the actual write did not execute |
| EC-05-007 | Irreversible write returns a `ConfirmationToken` (first step) | Audit entry records `result_summary: "confirmation_token_issued"` with the token's `action_summary` but not the token ID |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-007 |
| L2 Invariants | DI-003, DI-004 |
| Priority | P0 |
