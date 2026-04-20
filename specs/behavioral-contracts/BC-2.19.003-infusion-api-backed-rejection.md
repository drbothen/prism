---
document_type: behavioral-contract
level: L3
version: "1.1"
status: draft
producer: product-owner
timestamp: 2026-04-16T12:00:00
phase: 2-patch
origin: greenfield
subsystem: "SS-19"
capability: "CAP-031"
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
input-hash: "[pending-recompute]"
traces_to: ["CAP-031"]
extracted_from: ".factory/specs/prd.md"
---

# BC-2.19.003: API-Backed Infusion UDFs Rejected in Detection Rule Filters — E-RULE-012

## Description

Detection rules (`.detect` files or `create_rule` MCP tool) that reference infusion UDFs
backed by WASM plugins (`type = "plugin"`) in their `filter` expressions are rejected
at rule validation time with error `E-RULE-012`. Plugin-backed infusions may make
external HTTP calls, which cannot be allowed in the synchronous DataFusion filter
execution path used by detection rules. Local lookup infusions (`type = "local_lookup"`)
are permitted. This is INV-INFUSE-003.

## Preconditions

- The `InfusionRegistry` has a plugin-backed infusion registered (e.g., `threat_intel`
  with `type = "plugin"`)
- A detection rule is being validated (at load time or via `create_rule` MCP tool)
- The rule's `filter` expression references a UDF from the plugin-backed infusion
  (e.g., `threat_score(device_ip) > 0.8`)

## Postconditions

- `InfusionRegistry::is_api_backed(udf_name)` returns `true` for the referenced UDF
- The detection rule validator (S-4.03, `RuleValidator`) calls `is_api_backed()` for
  each UDF referenced in the filter expression
- The rule is rejected with:
  `E-RULE-012: "Detection rule filter references API-backed infusion UDF '{udf_name}' (from infusion '{infusion_id}', type 'plugin'). API-backed infusions cannot be used in detection rules — use a local_lookup infusion instead."`
- The rejected rule is NOT added to the rule registry
- Other rules in the same file continue loading (isolated rejection)
- A query that uses the same UDF via `SELECT threat_score(device_ip) FROM events`
  (not a detection rule filter) is NOT rejected — the restriction is specific to
  detection rule filters, not all query contexts

## Invariants

- INV-INFUSE-003: API-backed infusions (`type = "plugin"`) are rejected in detection rule filters with `E-RULE-012`
- The enforcement point is S-4.03 (detection rule loader); `prism-spec-engine` provides
  the `is_api_backed()` query interface but does NOT enforce the rejection itself
- Local lookup infusions (`maxmind_mmdb`, `csv`, `json_lookup`) are always permitted
  in detection rule filters

## Error Conditions

| Error | Condition | Behavior |
|-------|-----------|----------|
| `E-RULE-012` | Detection rule filter references plugin-backed infusion UDF | Rule rejected; error message names the UDF and infusion; rule NOT registered |
| — | UDF name not found in infusion registry | `is_api_backed()` returns `false` (unknown UDF is not API-backed); no rejection on this basis |

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-19-010 | Detection rule `SELECT` clause (not filter) references plugin-backed UDF | Allowed; only the `filter:` block is constrained |
| EC-19-011 | Plugin-backed infusion UDF used in a PrismQL query (not a detection rule) | Allowed; the UDF executes via the WASM plugin runtime per BC-2.17.001 |
| EC-19-012 | Plugin-backed infusion hot-reloaded to `type = "local_lookup"` | `is_api_backed()` returns `false` after reload; existing rejected rules can now be re-submitted |
| EC-19-013 | Rule references both a local UDF and a plugin-backed UDF | Rule rejected due to plugin-backed UDF; both UDFs named in `E-RULE-012` |

## Canonical Test Vectors

| ID | Input | Expected Output | Notes |
|----|-------|----------------|-------|
| TV-19-003-happy | Detection rule filter referencing local `geoip_country` UDF | Rule accepted | Baseline |
| TV-19-003-reject | Detection rule filter referencing plugin-backed `threat_score` UDF | `E-RULE-012`; rule not registered | AC-4 |
| TV-19-003-select | Rule `SELECT` clause referencing plugin-backed UDF (no filter) | Rule accepted; only filter is constrained | EC-19-010 |
| TV-19-003-mixed | Rule filter references both local UDF and plugin UDF | Rejected; both UDFs named in error | EC-19-013 |

## Verification Properties

| VP ID | Description | Verification Method |
|-------|-------------|---------------------|
| VP-TBD | Plugin-backed UDF in detection rule filter produces `E-RULE-012` | Integration test (`tests/infusion_tests.rs` AC-4) |
| VP-TBD | `SELECT`-only (non-filter) reference to plugin-backed UDF is allowed | Integration test |

## Related BCs

- BC-2.19.001 — Infusion Spec Loading (`is_api_backed()` implementation based on spec type)
- BC-2.13.001 — Detection Rule Loading (enforcement point where `E-RULE-012` is raised)
- BC-2.17.001 — Plugin Panic Isolation (WASM plugins CAN run in query context, just not detection filters)

## Architecture Anchors

- AD-020: Infusions — API-backed UDF restriction in detection rules
- `specs/architecture/infusions.md` — `is_api_backed()` method, E-RULE-012
- S-1.14 Task 8: `InfusionRegistry::is_api_backed()`
- S-4.03 (Detection Rule Loading): enforcement of E-RULE-012 using `is_api_backed()`

## Story Anchor

S-1.14 — prism-spec-engine: Infusion Spec Loading and UDF Registration (INV-INFUSE-003, AC-4)

## VP Anchors

Integration test: `tests/infusion_tests.rs` — "Verify `type = 'plugin'` infusion UDF rejected in detection rule filter with `E-RULE-012`."

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-031 |
| Story Invariant | INV-INFUSE-003 |
| ADR | AD-020 |
| Story | S-1.14 |
| Priority | P0 |

## Changelog

| Version | Date | Burst | Change |
|---------|------|-------|--------|
| 1.0 | 2026-04-16 | Phase 2 | Initial contract |
| 1.1 | 2026-04-20 | Wave 6 pre-build sweep | Added frontmatter (inputs, input-hash, traces_to, extracted_from, lifecycle fields); renamed Error Cases → Error Conditions; added Canonical Test Vectors, Verification Properties, Changelog |
