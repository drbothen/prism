---
document_type: behavioral-contract
level: L3
version: "1.1"
status: draft
producer: product-owner
timestamp: 2026-04-14T05:00:00
phase: 1a
origin: greenfield
subsystem: "SS-05"
capability: "CAP-007"
lifecycle_status: active
introduced: cycle-1
modified: null
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
inputs: [".factory/specs/prd.md", ".factory/specs/domain-spec/capabilities.md"]
input-hash: "e5de7f9"
traces_to: ["CAP-007"]
extracted_from: ".factory/specs/prd.md"
---

# BC-2.05.006: Audit Entries Are Append-Only and Immutable

## Description

Audit entries are written sequentially via the `tracing` structured logging framework; once
emitted, no code path exists to modify, overwrite, or delete them from Prism's side. The
append-only property is structural: the `tracing` crate emits events to a subscriber that
writes to an output stream with no update mechanism. The `trace_id` field on each entry
provides a unique identifier enabling external systems to verify completeness. If a correction
is needed, a new entry is emitted referencing the original `trace_id` — the original entry
is never altered.

This structural immutability supports SOC 2 Type II audit trail requirements.

## Preconditions
- Audit entries are being emitted via the `tracing` structured logging framework
- The tracing subscriber writes to stderr (or a file/pipe consumed by Vector)

## Postconditions
- Once an audit entry is emitted, it cannot be modified, overwritten, or deleted by Prism
- Audit entries are written sequentially; no API or code path exists to update a previously emitted entry
- The `trace_id` field provides a unique identifier for each entry, enabling external systems to verify completeness
- If a correction is needed (e.g., a tool result was initially reported as success but later determined to be a partial failure), a new audit entry is emitted referencing the original `trace_id`

## Invariants
- DI-004: Audit completeness -- entries are append-only, supporting SOC 2 audit trail requirements

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| N/A | No code path exists to mutate or delete audit entries | Immutability is structural -- the `tracing` crate emits events to a subscriber that writes to an output stream with no update mechanism |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-05-010 | External process truncates the log file while Prism is running | Prism is unaware; it continues appending. Log completeness is the responsibility of the log infrastructure (Vector pipeline). Prism's contract is limited to emitting entries. |

## Canonical Test Vectors

See `.factory/specs/prd-supplements/test-vectors.md` for canonical test vectors for BC-2.05.006.

| Scenario | Action | Expected Behavior |
|----------|--------|-------------------|
| Tool invocation | Any tool completes | Single audit entry appended; `trace_id` unique |
| Correction needed | Result updated after initial emission | New audit entry emitted with `trace_id` referencing original; original not modified |
| Concurrent invocations | Multiple simultaneous tools | Separate entries with unique `trace_ids`; ordering sequential per subscriber |

## Verification Properties

No VPs in VP-INDEX v1.5 directly verify audit entry immutability. Placeholder for future VP.

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-007 |
| L2 Invariants | DI-004 |
| Priority | P0 |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.1 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added extracted_from/inputs/input-hash/traces_to frontmatter; added ## Description synthesized from body; added ## Canonical Test Vectors scaffolding; added ## Verification Properties cross-ref; added ## Changelog. |
