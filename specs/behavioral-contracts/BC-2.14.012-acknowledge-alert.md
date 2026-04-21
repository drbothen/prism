---
document_type: behavioral-contract
level: L3
version: "1.3"
status: draft
producer: product-owner
timestamp: 2026-04-16T12:00:00
phase: 2-patch
origin: greenfield
subsystem: "SS-14"
capability: "CAP-022"
lifecycle_status: active
introduced: cycle-1
modified: null
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
inputs:
  - ".factory/specs/prd.md"
  - ".factory/specs/domain-spec/capabilities.md"
input-hash: "47125c0"
traces_to:
  - "CAP-022"
extracted_from: ".factory/specs/prd.md"
---

# BC-2.14.012: `acknowledge_alert` MCP Tool — Mark Alert as Acknowledged (Idempotent)

## Description

Implements **CAP-022** (case management) — the human-initiated acknowledgment of an alert, recording which analyst acked and when.

The `acknowledge_alert` MCP tool marks an alert as seen/acknowledged by an analyst,
recording the acknowledging analyst's identity, an ISO 8601 timestamp, and an optional
free-text note. The operation is idempotent: re-acknowledging an already-acknowledged
alert returns the existing acknowledgment record without mutation. The acknowledgment is
persisted to the `alerts` RocksDB column family and an audit event is emitted.
Client-scoping is enforced: the analyst must supply the `client_id` that owns the alert.

## Preconditions

- The `alerts` RocksDB column family is initialized (BC-2.15.001)
- The alert identified by `alert_id` exists in the `alerts` CF under the given `client_id`
- The calling tool invocation includes a valid `client_id` matching the alert's `client_id`
- The feature flag `alert.acknowledge` is enabled for the target client (defaults to
  enabled for all clients unless explicitly denied via the capability system, BC-2.04.003)
- The request includes either an analyst identity from the MCP session context or an
  explicit `analyst_id` parameter

## Postconditions

- **New acknowledgment:** The alert record is updated with:
  - `acknowledged: true`
  - `acknowledged_at`: ISO 8601 UTC timestamp of the acknowledgment
  - `acknowledged_by`: analyst identifier (from session context or explicit parameter)
  - `acknowledgment_note`: optional free-text note (null if not provided)
  - Updated record written atomically to `alerts:{client_id}:{alert_id}` key in RocksDB
- **Re-acknowledgment (idempotent):** The existing acknowledgment record is returned
  unchanged; no mutation occurs; `acknowledged_at` is NOT updated to the current time
- The MCP response includes the full acknowledgment record:
  ```json
  {
    "_meta": { "tool": "acknowledge_alert", "trust_level": "internal" },
    "alert": {
      "alert_id": "<uuid>",
      "status": "acknowledged",
      "acknowledged_at": "<ISO 8601>",
      "acknowledged_by": "<analyst_id>",
      "acknowledgment_note": "<note or null>",
      "idempotent": false
    }
  }
  ```
  (Field `idempotent: true` when the alert was already acknowledged and no mutation occurred)
- An audit event is emitted with event type `alert_acknowledged` regardless of whether
  the operation mutated state, including: `alert_id`, `client_id`, `acknowledged_by`,
  `idempotent` (bool), and `acknowledgment_note` (redacted if null, otherwise included)
- Alert indexes in RocksDB are NOT updated (acknowledgment is a field on the alert
  record, not a separate index dimension)

## Invariants

- DI-008 (Client Data Separation): Alert access is scoped to `client_id`; an analyst cannot
  acknowledge an alert from a different client than the one specified
- DI-004 (Audit completeness — write fail-closed): `acknowledge_alert` is a write
  operation (it mutates alert state). If audit emission fails before the RocksDB write,
  the acknowledgment is NOT persisted — the operation is aborted with `PrismError::AuditRequired`.
  The audit event is written before the write, per the write fail-closed behavior in DI-004/DI-016.
- DI-008 (Client data separation): The RocksDB key `alerts:{client_id}:{alert_id}` is
  prefix-scoped by `client_id` — no cross-client access is possible
- Acknowledgment is idempotent: the alert's `acknowledged_at` timestamp reflects the
  FIRST acknowledgment, not the most recent invocation
- The operation is non-destructive: alert fields other than acknowledgment status are
  not modified by this tool
- Audit emit-before-return: the audit event is written before the MCP response is sent

## Error Conditions

| Error | Condition | Behavior |
|-------|-----------|----------|
| `E-CASE-010` | `alert_id` not found in `alerts:{client_id}:*` | Structured error: "Alert '{alert_id}' not found for client '{client_id}'. Verify alert_id and client_id." |
| `E-CASE-011` | `client_id` does not match the alert's stored `client_id` | Structured error: "Alert '{alert_id}' does not belong to client '{client_id}'." |
| `E-GATE-001` | Feature flag `alert.acknowledge` is denied for the client | Structured error per BC-2.04.015: capability denied with explanation |
| `E-STORE-002` | RocksDB write failure during acknowledgment persistence | Non-retryable error: "Acknowledgment could not be persisted. Alert state unchanged." Audit log records failure. |
| `E-AUTH-001` | `analyst_id` is missing and cannot be derived from session context | Structured error: "analyst_id is required to acknowledge an alert. Provide analyst_id or configure session identity." |

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-14-040 | Alert is already acknowledged; tool called again with same analyst | Returns existing ack record with `idempotent: true`; no write to RocksDB; audit event emitted with `idempotent: true` |
| EC-14-041 | Alert is already acknowledged; tool called again with different analyst | Returns existing ack record with `idempotent: true`; original `acknowledged_by` is preserved; no mutation |
| EC-14-042 | Alert exists but has never been linked to a case | Acknowledgment succeeds; alert and case are independent entities; no case is created |
| EC-14-043 | `acknowledgment_note` exceeds 2048 characters | Rejected with `E-CASE-012: "acknowledgment_note must be 2048 characters or fewer ({actual} provided)."` |
| EC-14-044 | `alert_id` is a valid UUID format but references a deleted/purged alert | Treated identically to not-found: `E-CASE-010` |
| EC-14-045 | 100 concurrent `acknowledge_alert` calls for the same alert | First writer wins and persists; subsequent concurrent calls (racing) return the already-acknowledged state with `idempotent: true` due to RocksDB's atomic compare-and-swap write semantics |

## Canonical Test Vectors

See `.factory/specs/prd-supplements/test-vectors.md` for full canonical vectors.

| Scenario | Input | Expected Output |
|----------|-------|-----------------|
| Happy path — first ack | `alert_id=X, client_id=acme, analyst_id=alice` | `acknowledged=true`, `idempotent=false`, audit emitted |
| Idempotent re-ack (same analyst) | acknowledge already-acked alert | Existing record returned; `idempotent=true`; no RocksDB write |
| Idempotent re-ack (different analyst) | second analyst acks same alert | Original `acknowledged_by` preserved; `idempotent=true` |
| Alert not found | `alert_id` does not exist | `E-CASE-010` |
| Note too long | `acknowledgment_note` with 2049 chars | `E-CASE-012` |

## Verification Properties

| VP ID | Description |
|-------|-------------|
| (none) | Idempotent no-write is behavioral (conditional RocksDB write omission); audit-before-write covered transitively by VP-033 and DI-016 architectural invariant; confirmed by BC's own VP Anchors section. |

## Related BCs

- BC-2.13.005 — Alert Generation (creates the alert record this tool acknowledges)
- BC-2.13.013 — Alert Deduplication (dedup logic does not interact with acknowledgment status)
- BC-2.14.001 — `create_case` (acknowledgment is independent of case creation)
- BC-2.04.003 — Hierarchical Flag Resolution (governs `alert.acknowledge` capability evaluation)
- BC-2.05.001 — Audit Entry per Tool Invocation (covers this tool's audit requirement)
- BC-2.14.013 — Auto-Case-Creation (high-severity alerts that trigger this flow are still independently acknowledgeable)

## Architecture Anchors

- AD-004: RocksDB with column families — `alerts` CF stores acknowledgment state
- AD-017: AI-opaque credential management — analyst identity is derived from session context, not inline parameters where possible
- AD-016: Write-audit ordering — audit event written before response returned
- `specs/architecture/api-surface.md` section 1.24b — tool schema for `acknowledge_alert`

## Story Anchor

S-4.07 (Case Metrics / acknowledge_alert) — acknowledgment is part of the alert lifecycle and operational metrics handled by prism-operations.

## VP Anchors

No dedicated VPs currently. Covered by integration tests in S-4.07 test suite (alert acknowledgment lifecycle, idempotency, RocksDB write atomicity).

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-022 |
| L2 Invariants | DI-004, DI-008 |
| ADR | AD-004, AD-016, AD-017 |
| Story | S-4.07 |
| Priority | P0 |
| Interface | api-surface.md §1.24b |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.3 | pass-74-fix | 2026-04-20 | product-owner | Resolved (placeholder) row in ## Verification Properties per pass-74 VP-TBD decision matrix extension. |
| 1.2 | pass-73-fix | 2026-04-20 | state-manager | Deterministic changelog reorder: sorted all rows to descending version order (pass-73 bash script). |
| 1.1 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added extracted_from/inputs/input-hash/traces_to frontmatter; added ## Canonical Test Vectors scaffolding; added ## Verification Properties cross-ref; renamed Error Cases → Error Conditions; added ## Changelog. |
| 1.0 | cycle-1 | 2026-04-16 | product-owner | Initial phase-2-patch BC |
