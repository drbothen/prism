---
document_type: behavioral-contract
level: L3
version: "1.5"
status: draft
producer: product-owner
timestamp: 2026-04-14T05:00:00
phase: 1a
inputs: [".factory/specs/prd.md", ".factory/specs/domain-spec/capabilities.md"]
input-hash: "c36ec87"
traces_to: ["CAP-008"]
extracted_from: ".factory/specs/prd.md"
origin: greenfield
subsystem: "SS-08"
capability: "CAP-008"
lifecycle_status: active
introduced: cycle-1
modified: ["OOD-001-adjudication-2026-06-17", "F-S503-004-adjudication-2026-06-17"]
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.08.005: Health Check MCP Tool

## Description

The `check_sensor_health` tool returns structured health status for one or all sensors for a given client, or a cross-client health matrix when `client_id` is null. Each response includes per-sensor connectivity, auth validity, rate limit state, and last successful query timestamp, plus a `resource_pressure` section with active cursor count and token count. The response uses `structuredContent` for machine-parseable data and `content[].text` prose summary. Trust level is `"internal"` since health data is Prism-generated.

**Two-phase probe behavior (F-S503-004 adjudication):** This BC specifies BOTH S-5.03-scoped and S-5.04 live-probe behavior. An implementation that returns `reachable: true, auth_valid: true` as unverified hardcoded positives violates this contract — it sends a FALSE-POSITIVE health signal to the AI consumer, which may act on it believing sensors are reachable and authenticated when they may not be. The correct two-phase model is:

- **S-5.03 scope (spec-only):** `probe_level: "spec-only"` is set; `reachable: null` and `auth_valid: null` are returned (honest unknown — no live probe has been performed). The prose summary must explicitly state `"spec-only: no live probe performed"` so the AI consumer cannot mistake the response for a live health check. `last_successful_query_at: null` (no query has run).
- **S-5.04 scope (live probe, anchored to S-5.04):** `probe_level: "live"` is set; `reachable` and `auth_valid` are populated from the actual API probe result. S-5.04 depends_on S-5.03 and extends this contract. The live-probe obligation is deferred to S-5.04 — NOT to "a wave" or "later".

## Preconditions
- The `check_sensor_health` MCP tool is registered in `tools/list`
- The tool accepts `client_id: String` (required — always present; the MCP caller must supply the client scope even in per-analyst stdio deployments where a single analyst services multiple clients) and `sensor_id: Option<SensorId>` (optional — null means all sensors for that client)
- Implementation note: the `CheckSensorHealthParams` struct MUST have `pub client_id: String` as a required field. Any stub that omits `client_id` (e.g., `sensor: Option<String>` only) is non-conformant with this contract and MUST be corrected before Task 3 of S-5.03 is implemented (OOD-001 adjudication — SPEC WINS per CLAUDE.md §7). The struct field name MUST be `client_id` to match the MCP tool parameter name; the legacy `sensor` field (absent `client_id`) is incorrect.

## Postconditions
- When `sensor_id` is provided: returns health status for that single sensor
- When `sensor_id` is null: returns health status for all configured sensors for the client
- When `client_id` is null (cross-client): returns health status for all sensors across all configured clients. Each entry includes the `client_id` field so results can be attributed. The `summary` section aggregates counts across all clients. `partial_failures` lists any clients whose health check failed (e.g., credential unavailable) without blocking results from other clients.
- Each sensor health entry contains: `sensor_id`, `client_id` (always present in cross-client responses), `probe_level` (either `"spec-only"` or `"live"`), `reachable` (`true`/`false` for live probe; `null` for spec-only), `auth_valid` (`true`/`false` for live probe; `null` for spec-only), `rate_limit`, `last_successful_query_at` (`null` for spec-only). **`reachable: null` and `auth_valid: null` MUST be used in S-5.03-scoped (spec-only) responses — hardcoded `true` values for unverified fields are forbidden.**
- The response includes a `resource_pressure` section with: `active_cursor_count` (current number of non-expired cursors, out of 200 cap) and `active_token_count` (current number of unexpired, unconsumed confirmation tokens, out of 100 cap). This gives the agent visibility into resource pressure without needing a separate tool.
- Response uses `structuredContent` for machine-parseable health data
- Response includes `content[].text` prose summary. **S-5.03-scoped implementations MUST include the phrase `"spec-only: no live probe performed"` in the prose summary** so the AI consumer cannot mistake the response for a live health check. S-5.04-scoped implementations use `"live probe"` phrasing (e.g., "2 of 3 sensors healthy for client 'acme' (live probe)").
- Response metadata includes `trust_level: "internal"` (health data is Prism-internal, not sensor-sourced)
- Tool annotations: `readOnlyHint: true`, `destructiveHint: false`, `idempotentHint: true`, `openWorldHint: true`
- **S-5.04 live-probe anchor:** `probe_level: "live"` and populated `reachable`/`auth_valid` fields are delivered in S-5.04 (`S-5.04-sensor-health.md`). S-5.04 depends_on S-5.03. The live-probe implementation is NOT deferred to an unspecified wave — it is anchored to S-5.04.

## Invariants
- DI-004: Audit completeness -- exactly one AuditEntry emitted per tool invocation
- DI-008: Client data separation -- only the specified client's sensors are checked

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| `PrismError::InvalidInput` | Invalid `client_id` format | Structured error with validation details |
| `PrismError::Config` | `client_id` not found in config | Structured error with suggestion to check config |
| `PrismError::InvalidInput` | Invalid `sensor_id` value | Structured error listing valid sensor IDs from loaded spec files |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| DEC-004 | Client has zero sensors configured | Returns empty health array with message "Client '{id}' has no sensors configured" |
| EC-08-010 | One sensor healthy, another unreachable | Returns partial health results; does not fail the entire tool call |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `check_sensor_health("acme", sensor_id: null)` — S-5.03 scope (spec-only) | `structuredContent` with all sensors `probe_level: "spec-only"`, `reachable: null`, `auth_valid: null`, `last_successful_query_at: null`; prose includes "spec-only: no live probe performed" | happy-path (S-5.03) |
| `check_sensor_health("acme", sensor_id: null)` — S-5.04 scope (live probe) | `structuredContent` with all sensors `probe_level: "live"`, `reachable: true`, `auth_valid: true`; prose "3 of 3 sensors healthy for client 'acme' (live probe)" | happy-path (S-5.04) |
| `check_sensor_health(null)` — cross-client, S-5.03 scope | Health matrix across all clients; each entry includes `client_id`, `probe_level: "spec-only"`, `reachable: null`, `auth_valid: null` | happy-path (S-5.03) |
| One sensor healthy, one unreachable — S-5.04 scope | Partial results; healthy sensor `reachable: true`; unreachable sensor `reachable: false` | edge-case (S-5.04) |
| Client with zero sensors configured | Empty array; message "Client 'x' has no sensors configured" | edge-case |

See `.factory/specs/prd-supplements/test-vectors.md` for canonical test vector tables.

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| (no matching VP) | Exactly one AuditEntry emitted per tool invocation | integration test |
| (no matching VP) | `trust_level: "internal"` always set on health responses | integration test |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-008 |
| L2 Invariants | DI-004, DI-008 |
| Priority | P1 |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.5 | F-S503-004-adjudication-2026-06-17 | 2026-06-17 | product-owner | **F-S503-004 adjudication — honest-unknown semantics for S-5.03 scope; live-probe anchored to S-5.04.** Ruling: S-5.03 MUST NOT return `reachable: true, auth_valid: true` as unverified hardcoded positives — this sends a false-positive health signal to the AI consumer and violates the production-grade default (CLAUDE.md Canonical Principle Rule 1). Correct S-5.03-scoped behavior: `probe_level: "spec-only"`, `reachable: null`, `auth_valid: null`, `last_successful_query_at: null`. Prose summary MUST include "spec-only: no live probe performed". S-5.04 delivers `probe_level: "live"` with real probe results. Changes: (1) Description section expanded with Two-phase probe behavior block. (2) Postconditions updated: `probe_level` field added to health entry definition; `reachable`/`auth_valid` spec'd as `null` for spec-only; hardcoded-true prohibition made explicit; prose-summary S-5.03 requirement added; S-5.04 anchor added. (3) Canonical Test Vectors updated to show both scope variants. (4) Live-probe deferral anchored to S-5.04 (real story ID, not "a wave"). **Story-writer propagation required for S-5.03:** update AC-4 to assert `probe_level: "spec-only"`, `reachable: null`, `auth_valid: null` (not `true`) and prose contains "spec-only: no live probe performed". Story-writer propagation required for S-5.04: AC must assert `probe_level: "live"` and live `reachable`/`auth_valid` values. **Bumped v1.4→v1.5.** |
| 1.4 | OOD-001-adjudication-2026-06-17 | 2026-06-17 | product-owner | OOD-001 adjudication — SPEC WINS. `client_id: String` is unambiguously required; expanded Preconditions to make this explicit and document the S-5.03 implementer obligation (add `client_id` field to `CheckSensorHealthParams`; `sensor: Option<String>`-only struct is non-conformant). No semantic contract change — the multi-client architecture has always required `client_id`; this version makes the implementer-visible contract text unambiguous. |
| 1.3 | pass-73-fix | 2026-04-20 | state-manager | Deterministic changelog reorder: sorted all rows to descending version order (pass-73 bash script). |
| 1.2 | pass-69-housekeeping | 2026-04-20 | product-owner | Normalized changelog schema to canonical 5-col schema. |
| 1.1 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added extracted_from/inputs/input-hash/traces_to frontmatter; added ## Description synthesized from body; added ## Canonical Test Vectors scaffolding; added ## Verification Properties cross-ref; added ## Changelog. |
| 1.0 | cycle-1 | 2026-04-14 | product-owner | Initial draft |
