# TD-S303-AUDIT-MCP004-001: E-MCP-004 audit path has no regression test

**Story:** S-3.03
**Status:** open
**Severity:** tech_debt

## Description

The `explain()` function in `crates/prism-query/src/explain.rs` emits an
`AuditEvent { outcome_summary: "E-MCP-004" }` when `resolve_clients()` returns
an error (line ~887). This path is never exercised by the current test suite —
all tests use either a valid `ClientRegistry` or `None` (which produces an empty
registry and succeeds).

O-LOCAL-NEW-4: The E-MCP-004 audit emission exists in production code but has
no corresponding regression test, meaning a future refactor could silently remove
the audit emission without any test failing.

## Current Behavior

The E-MCP-004 path is reachable only when `resolve_clients()` returns `Err`.
Looking at `scoping.rs`, `resolve_clients()` currently only returns `Err` in
specific conditions (e.g., registry poisoning or invalid org slug). These
conditions are not easily triggerable through the public `ExplainOptions` API
without mock injection.

## Required Fix

Add a test that:
1. Provides a `ClientRegistry` or `clients` list that causes `resolve_clients()`
   to return `Err`.
2. Asserts that the captured audit event has `outcome_summary: "E-MCP-004"`.
3. Asserts that the audit event is emitted even on the error path.

This requires either making `resolve_clients()` fallible in a test-configurable way
or adding a mock/stub path to `ExplainOptions` for injecting the error.

## Deferred In

S-3.03 adversarial local pass-2 (O-LOCAL-NEW-4). The code path exists and is
correct; the gap is test coverage only.

## References

- `crates/prism-query/src/explain.rs` — Step 7, `resolve_clients()` error path (~line 887)
- `crates/prism-query/src/scoping.rs` — `resolve_clients()` error conditions
- S-3.03 adversary local pass-2 finding O-LOCAL-NEW-4
