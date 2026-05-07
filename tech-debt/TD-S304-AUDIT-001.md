# TD-S304-AUDIT-001: Replace tracing::info audit placeholders with prism_audit::emit_audit

**Story:** S-3.04
**Status:** open
**Severity:** tech_debt
**Filing:** pass-1 review CR-023

## Description

All four alias tool handlers (`create_alias`, `list_aliases`, `delete_alias`, `explain_alias`)
emit DI-004 audit entries via `tracing::info!` placeholders. These must be replaced with
actual `prism_audit::emit_audit` calls once the audit crate is available in the workspace.

## Current Behavior

`tracing::info!` spans record operation type, alias name, scope, and outcome. These appear
in structured logs but are not forwarded to the formal audit trail.

## Required Fix

1. Add `prism-audit` crate dependency to `prism-query`.
2. Replace each `tracing::info!` in `alias_tools.rs` with `prism_audit::emit_audit(...)`.
3. Ensure the audit record includes: alias_name, scope, operation, success/error_code,
   and timestamp (DI-004 invariant).

## Tests Deferred

- `test_BC_2_11_008_create_alias_emits_audit_entry` — requires audit crate integration.
- `test_BC_2_11_014_delete_alias_emits_audit_entry` — same.

## References

- DI-004 (audit trail invariant)
- CR-023 (pass-1 review finding)
- BC-2.11.008, BC-2.11.014
