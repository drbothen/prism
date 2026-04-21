---
document_type: verification-property
level: L4
version: "1.3"
status: draft
producer: architect
timestamp: 2026-04-19T00:00:00
phase: 1c
inputs: [VP-INDEX.md, S-1.02-entity-types.md]
input-hash: "220d361"
traces_to: architecture/verification-architecture.md
source_bc: "BC-2.07.002"
module: prism-core
priority: P1
proof_method: kani
verification_method: kani
feasibility: medium
verification_lock: false
proof_completed_date: null
proof_file_hash: null
lifecycle_status: active
introduced: cycle-1
modified: null
deprecated: null
deprecated_by: null
replacement: null
retired: null
withdrawn: null
withdrawal_reason: null
removed: null
removal_reason: null
---

# VP-029: Cursor Cap — Rejects at 200 Active

## Property Statement

For every cursor registry, if the count of active (non-expired, non-closed) cursors
is at or above 200, any subsequent `allocate_cursor()` call returns
`Err(CursorError::CapacityExhausted)` without registering a new cursor. The active
cursor count is strictly bounded above by 200.

## Source Contract

- **Anchor Story:** `S-1.02`
- **Source BC:** BC-2.07.002 — Internal Pagination Token Lifecycle — Forward Progress, Timeout, and Cleanup
- **Module:** prism-core
- **Category:** Security

## Proof Method

| Method | Tool | Bounded? | Coverage |
|--------|------|----------|----------|
| kani | Kani (latest) | Yes — scaled cap (e.g. 3) proves invariant | Boundary around scaled cap |

## Proof Harness Skeleton

```rust
// [TODO: harness skeleton — author during Phase 5 formal-verify]
// Method: kani
// Target: prism_core::cursor::CursorRegistry::allocate_cursor
//
// Sketch: with CAP scaled down, fill registry to CAP then assert next
// allocate returns Err(CapacityExhausted).
```

## Feasibility Assessment

| Factor | Assessment | Notes |
|--------|------------|-------|
| Bounded inputs? | Yes | Scaled cap via const generic |
| Tool support? | Full | Kani handles bounded collections |
| Execution time budget | <3 minutes | Small cap |
| Assumptions required | Scaling generalizes to 200 by linearity | Documented |

## Lifecycle

| Event | Date | Actor |
|-------|------|-------|
| introduced | 2026-04-14 | architect |

## Changelog

| Version | Burst | Date | Author | Notes |
|---------|-------|------|--------|-------|
| 1.3 | pass-88-remediation | 2026-04-21 | architect | F88-012: Anchor Story normalized from slug form (S-1.02-entity-types.md) to pure ID (S-1.02). |
| 1.2 | pass-87-remediation | 2026-04-21 | architect | F87-006: Source BC label corrected "Internal Pagination Token Lifecycle (200 concurrent fetch cap invariant)" → "Internal Pagination Token Lifecycle — Forward Progress, Timeout, and Cleanup" (matches BC-2.07.002 H1); removed backtick wrapping around BC ID. |
| 1.1 | pre-build-sweep | 2026-04-20 | architect | Template-compliance sweep: added priority frontmatter (from VP-INDEX v1.5); added verification_method alias (proof_method retained for backward compat). |
