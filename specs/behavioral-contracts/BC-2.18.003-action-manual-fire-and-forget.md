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

# BC-2.18.003: Manual Action Triggers — Fire-and-Forget, Result Returned Immediately

## Description

Actions triggered via the `fire_action` MCP tool (`trigger = "manual"`) are fire-and-forget:
the delivery is attempted once, the result (success or error) is returned immediately to
the AI caller, and no retry occurs. Manual triggers are designed for interactive workflows
where the analyst wants immediate feedback. This is INV-ACTION-003.

## Preconditions

- `ActionEngine` has a registered `ActionSpec` with `trigger = "manual"` (or the
  `fire_action` MCP tool specifies an action_id for a manual-trigger action)
- The `fire_action` MCP tool is called by the AI agent with a valid `action_id` and
  optional template variables

## Postconditions

- The action destination is called once (no retry on failure)
- The result is returned synchronously to the `fire_action` MCP tool caller:
  ```json
  {
    "_meta": { "tool": "fire_action", "trust_level": "internal" },
    "action_id": "<id>",
    "status": "delivered" | "failed",
    "error": "<error message or null>",
    "delivery_latency_ms": 1234
  }
  ```
- An audit event is emitted: `action_manual_fired` with `action_id`, `status`, `latency_ms`
- On failure: `status: "failed"` with the error message; no retry state written; no dead-letter

## Invariants

- INV-ACTION-003: Manual triggers are fire-and-forget — result returned immediately, no retry
- The `fire_action` MCP tool call MUST NOT block for longer than the destination timeout
  (e.g., 10s for webhook). If the delivery takes longer, the tool times out and returns `status: "failed"`.
- No retry state is persisted to `action_state` CF for manual triggers
- Template injection scanning (BC-2.18.006) still applies to manually-provided template variables

## Error Cases

| Error | Condition | Behavior |
|-------|-----------|----------|
| `E-ACTION-005` | Action destination returns an error | `status: "failed"` returned immediately; no retry |
| `E-ACTION-006` | `action_id` not found in registry | Structured error: "Action '{action_id}' is not registered." |
| `E-ACTION-007` | `fire_action` called for a non-manual trigger action | Structured error: "Action '{action_id}' is not a manual-trigger action (trigger: '{actual_trigger}')." |

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-18-009 | Manual action destination (webhook) times out after 10s | Tool returns `status: "failed"` with timeout error after 10s |
| EC-18-010 | Template variable contains injection-scanned content | `InjectionScanner` flags the content; `_safety_flags` set in audit log; value still interpolated |
| EC-18-011 | Manual action called 100 times/second | Each call is independent; rate limiting (BC-2.18.005) may suppress some deliveries |

## Related BCs

- BC-2.18.001 — Alert/Case At-Least-Once Delivery (stricter guarantee)
- BC-2.18.002 — Schedule Best-Effort (intermediate guarantee)
- BC-2.18.006 — Template Injection Scanning (applies to all triggers including manual)
- BC-2.18.005 — Rate Limiting and Deduplication (applies to manual triggers)

## Architecture Anchors

- AD-021: Actions — manual trigger fire-and-forget
- `specs/architecture/actions.md` — ManualTrigger, fire_action MCP tool
- S-4.08 Task 3: `action/trigger.rs` — ManualTrigger

## Story Anchor

S-4.08 — prism-operations: Action Delivery Framework (INV-ACTION-003)

## VP Anchors

No dedicated test fixture. Covered by `fire_action` MCP tool integration tests in `tests/action_tests.rs`.

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-033 |
| Story Invariant | INV-ACTION-003 |
| ADR | AD-021 |
| Story | S-4.08 |
| Priority | P0 |
