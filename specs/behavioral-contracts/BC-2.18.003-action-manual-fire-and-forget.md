---
document_type: behavioral-contract
level: L3
version: "1.3"
status: draft
producer: product-owner
timestamp: 2026-04-16T12:00:00
phase: 2-patch
origin: greenfield
subsystem: "SS-18"
capability: "CAP-033"
lifecycle_status: active
introduced: cycle-1
modified: 2026-04-20
deprecated: ~
deprecated_by: ~
replacement: ~
retired: ~
removed: ~
removal_reason: ~
inputs:
  - ".factory/specs/prd.md"
  - ".factory/specs/domain-spec/capabilities.md"
input-hash: "365fb25"
traces_to: ["CAP-033"]
extracted_from: ".factory/specs/prd.md"
---

# BC-2.18.003: Manual Action Triggers — Fire-and-Forget, Result Returned Immediately to AI Caller

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

## Error Conditions

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

## Canonical Test Vectors

| ID | Input | Expected Output | Notes |
|----|-------|----------------|-------|
| TV-18-003-happy | Valid `action_id`; webhook returns 200 | `{"status": "delivered", "delivery_latency_ms": N}` | Baseline |
| TV-18-003-fail | Valid `action_id`; webhook returns 500 | `{"status": "failed", "error": "..."}` immediately; no retry | Error row 1 |
| TV-18-003-notfound | Unknown `action_id` | Structured error: "Action '{action_id}' is not registered." | EC-18-006 |
| TV-18-003-wrongtype | `action_id` for schedule-trigger action | Structured error referencing actual trigger type | Error row 3 |

## Verification Properties

| VP ID | Description | Verification Method |
|-------|-------------|---------------------|
| (none) | Fire-and-forget and E-ACTION-007 trigger-type check are integration behaviors; no pure-function invariant warrants a formal VP; verified by integration tests in tests/action_tests.rs | — |

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

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.3 | pass-69-housekeeping | 2026-04-20 | product-owner | Normalized changelog schema to canonical 5-col schema. |
| 1.2 | pass-69-housekeeping | 2026-04-20 | product-owner | Resolved VP-TBD placeholder per decision matrix (MARK-NONE); normalized changelog schema to canonical 5-col form. |
| 1.1 | Wave-6-pre-build-sweep | 2026-04-20 | product-owner | Added frontmatter (inputs, input-hash, traces_to, extracted_from, lifecycle fields); renamed Error Cases → Error Conditions; added Canonical Test Vectors, Verification Properties, Changelog |
| 1.0 | Phase-2 | 2026-04-16 | product-owner | Initial contract |
