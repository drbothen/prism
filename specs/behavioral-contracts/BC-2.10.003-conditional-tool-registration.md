---
document_type: behavioral-contract
level: L3
version: "1.3"
status: draft
producer: product-owner
timestamp: 2026-04-14T05:00:00
phase: 1a
inputs: [".factory/specs/prd.md", ".factory/specs/domain-spec/capabilities.md"]
input-hash: "5b48b9c"
traces_to: ["CAP-005"]
extracted_from: ".factory/specs/prd.md"
origin: greenfield
subsystem: "SS-10"
capability: "CAP-005"
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

# BC-2.10.003: Conditional Tool Registration (Feature-Flag Gated)

## Description

Write tools are gated at two tiers: compile-time cargo features (absent = tool not in binary) and runtime per-client TOML config (denied for all clients = tool hidden from `tools/list`). Hidden means completely omitted, not listed as unavailable. `confirm_action` is hidden only when no write tools are registered. `list_capabilities` reveals the full capability matrix regardless of current registration state. A `notifications/tools/list_changed` notification is sent at startup and on any config/spec reload that changes the visible tool set.

## Preconditions
- Two-tier feature flag system is active: compile-time cargo features + runtime per-client TOML config
- Write tools (containment, alert acknowledgment, device actions) are defined behind `#[cfg(feature = "...")]` gates

## Postconditions
- If a cargo feature is absent at compile time, the corresponding write tools do not exist in the binary at all
- If a cargo feature is present but the runtime TOML config denies the capability for ALL configured clients, the tool is excluded from `tools/list`
- The hidden tools pattern: write tools disabled for all clients are completely omitted from `tools/list`, not listed as unavailable. Write tools enabled for at least one client are shown; per-call `client_id` determines authorization at invocation time.
- The server is stateless with respect to client context. There is no "client context switch". `notifications/tools/list_changed` is sent at server startup (when the initial tool set is computed from config) and on any config/spec reload that changes the available tool set (BC-2.16.005, BC-2.16.007). Specifically: if reload_config changes feature flags affecting tool visibility, or if sensor spec changes add/remove tables (changing the query engine's available sources), a `notifications/tools/list_changed` is sent so the AI agent refreshes its tool understanding.
- `confirm_action` is hidden from `tools/list` if and only if no write tools are registered (all write capabilities disabled for all clients). If at least one write tool is registered, `confirm_action` is visible.
- The `list_capabilities` meta-tool (BC-2.10.011) reveals the full capability matrix regardless of what is currently registered

## Invariants
- DI-003: Feature flag deny-by-default -- both tiers must permit the operation

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| Structured error | Agent calls a write tool that is runtime-disabled for the current client | Tool not found in `tools/list`; if agent somehow invokes it directly, returns `PrismError::Permission` with `suggestion: "Write capability '{path}' is not enabled for client '{id}'. Enable in TOML config at clients.{id}.capabilities.{path}"` |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| DEC-006 | Config changed after startup to enable a capability | Running session still uses startup-time capabilities unless config reload is triggered |
| EC-10-005 | All write features disabled at compile time | Zero write tools in binary; `list_capabilities` shows all write capabilities as "compile-time disabled" |
| EC-10-006 | Client A has containment enabled, Client B does not | Containment tool appears in `tools/list` (enabled for at least one client). Invocation with `client_id: "a"` succeeds. Invocation with `client_id: "b"` returns `E-FLAG-001`. |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| All write capabilities disabled for all clients | Only always-visible tools + no `confirm_action` in `tools/list` | happy-path |
| One client has containment enabled, one does not | Containment tool visible in `tools/list`; `client_id: "b"` invocation returns E-FLAG-001 | edge-case |
| Config reload enables a new capability | `notifications/tools/list_changed` sent; agent re-fetches `tools/list` | edge-case |

See `.factory/specs/prd-supplements/test-vectors.md` for canonical test vector tables.

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-020 | Feature flag: compile AND runtime must both permit | kani |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-005 |
| L2 Invariants | DI-003 |
| L2 Edge Cases | DEC-006 |
| Priority | P0 |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.3 | pass-73-fix | 2026-04-20 | state-manager | Deterministic changelog reorder: sorted all rows to descending version order (pass-73 bash script). |
| 1.2 | pass-69-housekeeping | 2026-04-20 | product-owner | Normalized changelog schema to canonical 5-col schema. |
| 1.1 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added extracted_from/inputs/input-hash/traces_to frontmatter; added ## Description synthesized from body; added ## Canonical Test Vectors scaffolding; added ## Verification Properties cross-ref; added ## Changelog. |
| 1.0 | cycle-1 | 2026-04-14 | product-owner | Initial draft |
