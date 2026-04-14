---
document_type: behavioral-contract
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-04-14T05:00:00
phase: 1a
origin: greenfield
subsystem: "Cursor State Management"
capability: "CAP-011"
---

# BC-2.07.004: Cursor State Is Persisted AFTER Successful Delivery

## Preconditions
- A collection cycle has fetched records and computed a new cursor
- The records are ready to be delivered (returned to MCP caller or forwarded downstream)

## Postconditions
- The ordering is strictly: (1) deliver records, (2) persist cursor, (3) update in-memory cursor
- If delivery fails, the cursor is NOT persisted; the next cycle re-fetches the same records
- If persistence fails after delivery, the in-memory cursor is NOT updated; the records will be re-delivered on the next cycle (at-least-once semantics)
- This ordering prevents the poller-cobra bug where state was saved before delivery, causing data loss on delivery failure

## Invariants
- DI-009: Persistence before state update -- in-memory cursor updated only after successful persistence

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| Delivery failure | MCP response cannot be sent (transport error) | Cursor not persisted; records will be re-fetched on next invocation |
| Persistence failure | `FileStore::save()` returns error after successful delivery | In-memory cursor not updated; warning logged; records may be re-delivered (at-least-once guarantee, not exactly-once) |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-07-005 | Delivery succeeds but Prism crashes before persistence | On restart, the cursor is at the pre-delivery position; records are re-fetched and re-delivered (duplicate delivery); consuming systems must handle idempotency |
| EC-07-006 | Zero records fetched (empty page) | No delivery occurs; cursor is not advanced; no persistence needed |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-011 |
| L2 Invariants | DI-009 |
| Priority | P0 |
