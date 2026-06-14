---
document_type: behavioral-contract
level: L3
version: "1.5"
status: draft
producer: product-owner
timestamp: 2026-04-14T05:00:00
phase: 1a
inputs: [".factory/specs/prd.md", ".factory/specs/domain-spec/capabilities.md"]
input-hash: "566def3"
traces_to: ["CAP-005"]
extracted_from: ".factory/specs/prd.md"
origin: greenfield
subsystem: "SS-10"
capability: "CAP-005"
lifecycle_status: active
introduced: cycle-1
modified: "2026-06-14"  # v1.5: R2 reconciliation — lock tri-state + capability-path response model; merged-code implementer gap note; supersede story-spec simple-bool shape
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.10.011: list_capabilities Meta-Tool

## Description

The `list_capabilities` tool is always registered (never gated) and returns the full capability matrix for a specified client (or all clients when `client_id` is null), showing each hierarchical capability path with its status: `enabled`, `runtime_disabled`, or `compile_time_disabled`. The response includes the resolution chain showing which hierarchy level determined the result, uses `trust_level: "internal"`, and is annotated `readOnlyHint: true`, `idempotentHint: true`, `openWorldHint: false`. This tool reveals the complete capability state regardless of what is visible in `tools/list`.

## Preconditions
- The `list_capabilities` tool is always registered (not gated by feature flags)
- The tool accepts `client_id: Option<String>` (required)

## Postconditions

### Input shape

`client_id: Option<String>` — optional. Null means "all clients" (cross-client summary mode). Required to be non-null for single-client detailed view. See BC-2.10.004 §Read Management Tools for the scoping model.

### Response model — LOCKED: tri-state + capability-path (supersedes simple bool map)

The story spec's `{effective_capabilities: Map<String, bool>}` and the merged code's `{client_registered, capabilities: Map<String, bool>, not_implemented, note}` are both superseded by this contract. The ratified response model is the **tri-state + hierarchical capability-path** model described below. The implementer must bring the merged code up to this model.

**Rationale for rejecting the simple bool map:** The merged code collapses `compile_time_disabled` and `runtime_disabled` into a single `false` value, losing the critical diagnostic distinction. A SOC analyst viewing `"sensor.crowdstrike.containment": false` cannot determine whether containment is disabled because (a) the TOML spec has no `[[write_endpoints]]` entry (operator must add an endpoint declaration) or (b) the runtime config denies it (operator must set `capabilities.sensor.crowdstrike.containment = "Allow"` in prism.toml). This distinction requires a human configuration change of different kinds. The tri-state model preserves it and is the production-correct choice.

**When `client_id` is provided (single-client mode):**

```json
{
  "client_id": "acme",
  "client_registered": true,
  "capabilities": {
    "sensor.crowdstrike.containment": {
      "status": "enabled",
      "resolution_chain": [
        { "level": "compile_tier", "result": "permit", "source": "WriteEndpointRegistry" },
        { "level": "runtime_tier", "result": "allow", "source": "prism.toml clients.acme.capabilities" }
      ]
    },
    "sensor.claroty.resolve_alert": {
      "status": "runtime_disabled",
      "resolution_chain": [
        { "level": "compile_tier", "result": "permit", "source": "WriteEndpointRegistry" },
        { "level": "runtime_tier", "result": "deny", "source": "prism.toml clients.acme.capabilities (not granted)" }
      ]
    },
    "sensor.armis.device_action": {
      "status": "compile_time_disabled",
      "resolution_chain": [
        { "level": "compile_tier", "result": "deny", "source": "WriteEndpointRegistry (no [[write_endpoints]] entry)" }
      ]
    }
  },
  "not_registered_tools": ["create_schedule", "watchdog_status"]
}
```

**Status values:**
- `"enabled"`: compile tier permits AND runtime TOML grants the capability. The tool will execute.
- `"runtime_disabled"`: compile tier permits (sensor TOML declares `[[write_endpoints]]`) but runtime config for this client does not grant the capability path. The operator must add the capability to `prism.toml clients.{id}.capabilities`.
- `"compile_time_disabled"`: compile tier denies — no `[[write_endpoints]]` entry in the sensor's TOML spec (registry-derived, BC-2.04.001/BC-2.16.012); or for `alias.write`, the `alias-write` cfg feature is not compiled in (BC-2.11.008). Runtime config cannot override this — the sensor spec must declare the endpoint.

**`resolution_chain`:** Array of resolution steps, each with:
- `level`: `"compile_tier"` or `"runtime_tier"`
- `result`: `"permit"`, `"allow"`, or `"deny"`
- `source`: human-readable string identifying what produced this result (e.g., `"WriteEndpointRegistry"`, `"prism.toml clients.acme.capabilities"`, `"prism.toml clients.acme.capabilities (not granted)"`)

**`client_registered`:** `true` if `client_id` exists in the runtime capability registry (same signal as the merged code's `client_registered`; preserved).

**`not_registered_tools`:** Array of tool names that are registered in the MCP catalog but return `-32003 NOT_IMPLEMENTED` because their underlying module (e.g., prism-operations) is not yet wired. These are distinct from capability-gated tools — they are unavailable regardless of feature flags. (Replaces merged code's `not_implemented` field; renamed for clarity.)

**When `client_id` is null (cross-client summary mode):**

Returns a per-client summary:

```json
{
  "client_id": null,
  "clients": {
    "acme": { "client_registered": true, "enabled_count": 3, "runtime_disabled_count": 1, "compile_time_disabled_count": 2 },
    "globex": { "client_registered": true, "enabled_count": 1, "runtime_disabled_count": 4, "compile_time_disabled_count": 2 }
  },
  "not_registered_tools": ["create_schedule", "watchdog_status"]
}
```

**Response envelope:** The response is wrapped in `SafetyEnvelopeBuilder` (consistent with all other Prism MCP tools) with `trust_level: "internal"` in `_meta`.

**Tool annotations (unchanged):** `readOnlyHint: true`, `destructiveHint: false`, `idempotentHint: true`, `openWorldHint: false`.

### Implementer gap — merged code delta

The merged `list_capabilities` handler in `crates/prism-mcp/src/server.rs` (lines 3108–3197) must be updated:
1. Replace `capabilities: Map<String, bool>` (LIVE_TOOLS bool) with `capabilities: Map<String, CapabilityEntry>` where `CapabilityEntry` has `status` (tri-state enum) + `resolution_chain` (Vec of steps). This requires calling the capability resolver per capability path for the given client.
2. Replace `not_implemented: NOT_YET_AVAILABLE_TOOLS` with `not_registered_tools` (renamed field; same constant).
3. For null `client_id`: return the cross-client summary shape instead of `"<all>"` placeholder.
4. `client_registered` is already correctly driven by `FeatureFlagEvaluator::client_exists()` — keep this behavior.
5. Remove the `note` field from the merged code; the response shape is now self-documenting via `resolution_chain`.
6. The capability resolver used here MUST be the same resolution logic used by the write pipeline two-tier check (BC-2.04.004), not a separate implementation. If the capability resolver is not yet accessible from `PrismServer`, the implementer should expose it from `WriteExecutor` or `FeatureFlagEvaluator`.

**Migration note:** `ListCapabilitiesParams.client_id` is already `Option<String>` in the merged code — this is correct and matches the BC's `client_id: Option<String>` precondition. No change needed to the params struct.

## Invariants
- DI-003: Feature flag deny-by-default -- the capability matrix reflects deny-by-default semantics

## Error Cases
| Error | Condition | PrismError Variant | MCP Error Code | Behavior |
|-------|-----------|-------------------|---------------|---------|
| Invalid `client_id` format | `client_id` fails `[a-zA-Z0-9_-]{1,64}` validation | (validate_client_ids pre-dispatch) | `E-MCP-001` | BC-2.10.007 structured error; `original_params_valid: false` |
| `client_id` not found in config | Passes format validation; not in runtime registry | `PrismError::ClientNotFound` | `E-CFG-100` | BC-2.10.007 structured error; `original_params_valid: true` |
| Invalid capability path | Capability path string is malformed | `PrismError::InvalidCapabilityPath` | `E-CFG-106` | BC-2.10.007 structured error |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-10-021 | Client with zero capabilities enabled | Returns full matrix with all capabilities showing `runtime_disabled` or `compile_time_disabled` |
| EC-10-022 | All sensor TOML specs declare `[[write_endpoints]]` but all capabilities runtime-disabled | Matrix shows all write capabilities as `runtime_disabled` with TOML paths for enabling |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `list_capabilities("acme")` with containment enabled in runtime config AND `[[write_endpoints]]` declared | `sensor.crowdstrike.containment: {status: "enabled", resolution_chain: [...]}` | happy-path |
| `list_capabilities("acme")` with no `[[write_endpoints]]` declarations loaded (empty `WriteEndpointRegistry`) | All sensor write capabilities `{status: "compile_time_disabled", resolution_chain: [{level: "compile_tier", result: "deny", source: "WriteEndpointRegistry (no [[write_endpoints]] entry)"}]}` | edge-case |
| `list_capabilities("acme")` with `[[write_endpoints]]` declared but runtime config does not grant the capability | `{status: "runtime_disabled", resolution_chain: [{level: "compile_tier", result: "permit", ...}, {level: "runtime_tier", result: "deny", ...}]}` | edge-case |
| `list_capabilities(null)` | `{clients: {"acme": {client_registered: true, enabled_count: N, ...}}, not_registered_tools: [...]}` | happy-path |
| Invalid `client_id` format (e.g., `"acme/../../etc"`) | `E-MCP-001` structured validation error; `original_params_valid: false` | error |
| `client_id: ""` (empty string) | `E-MCP-001` structured validation error; `original_params_valid: false` | error |
| Unknown `client_id` (valid format, not in config) | `E-CFG-100` structured error; `original_params_valid: true` | error |

See `.factory/specs/prd-supplements/test-vectors.md` for canonical test vector tables.

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-002 | Capability resolution: deny-by-default | kani |
| VP-003 | Capability resolution: most-specific-path wins | kani |
| VP-004 | Capability resolution: deny overrides allow at same specificity | kani |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-005 |
| L2 Invariants | DI-003 |
| Priority | P0 |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.5 | S-5.02-pre-TDD-reconciliation | 2026-06-14 | product-owner | R2 reconciliation: Locked response model as tri-state + hierarchical capability-path (supersedes story-spec `{effective_capabilities: Map<String,bool>}` and merged-code `{client_registered, capabilities: Map<String,bool>, not_implemented, note}`). Added complete JSON response schema for single-client and cross-client summary modes. Added `resolution_chain` spec (level/result/source per step). Added implementer gap note: merged code must replace bool map with CapabilityEntry{status+resolution_chain}, rename `not_implemented` → `not_registered_tools`, and use the shared capability resolver. Updated error cases table with PrismError variants and MCP error codes. Updated canonical test vectors for new response shape. Added input-shape note (`client_id: Option<String>` with null=all-clients). `not_registered_tools` field replaces `not_implemented`. Status labels, VP citations, trust_level, and tool annotations unchanged. |
| 1.4 | MCP cascade pass-1 P1-02 BC sibling sweep (2026-06-10 review-cycle PO micro-burst) | 2026-06-10 | product-owner | Stale cargo-feature framing rewritten to registry-derived compile-tier semantics, aligned with error-taxonomy v1.67 E-FLAG-002 row and BC-2.04.001 v1.2: `compile_time_disabled` status redefined as registry-derived compile-tier denial (no `[[write_endpoints]]` entry in the sensor's TOML spec, BC-2.16.012; alias-write cfg feature for `alias.write`); `runtime_disabled` definition reworded "compile-time feature present" → "compile tier permits"; EC-10-022 and the compile-tier test vector restated to declaration-based conditions (EC ID preserved). Status labels (`enabled`/`runtime_disabled`/`compile_time_disabled`), trust_level, annotations, and VP-002/003/004 citations unchanged. |
| 1.3 | pass-73-fix | 2026-04-20 | state-manager | Deterministic changelog reorder: sorted all rows to descending version order (pass-73 bash script). |
| 1.2 | pass-69-housekeeping | 2026-04-20 | product-owner | Normalized changelog schema to canonical 5-col schema. |
| 1.1 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added extracted_from/inputs/input-hash/traces_to frontmatter; added ## Description synthesized from body; added ## Canonical Test Vectors scaffolding; added ## Verification Properties cross-ref; added ## Changelog. |
| 1.0 | cycle-1 | 2026-04-14 | product-owner | Initial draft |
