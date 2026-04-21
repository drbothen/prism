---
document_type: verification-property
level: L4
version: "1.3"
status: draft
producer: architect
timestamp: 2026-04-19T00:00:00
phase: 1c
inputs: [VP-INDEX.md, S-4.04-detection-evaluation.md]
input-hash: "1386cde"
traces_to: architecture/verification-architecture.md
source_bc: BC-2.13.013
module: prism-operations
priority: P0
proof_method: proptest
verification_method: proptest
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

# VP-027: Alert Dedup Key — Correct Per Match Mode

## Property Statement

For every detection match `m` produced by mode `M` (single-event, correlation, or
sequence), `dedup_key(m)` includes exactly the fields specified for `M` in the
dedup contract. Two matches with equal key-contributing fields produce equal dedup
keys; matches that differ in any key-contributing field produce distinct keys.

## Source Contract

- **Anchor Story:** `S-4.04`
- **Source BC:** BC-2.13.013 — Alert Deduplication — Per-Match-Mode Dedup Keys Prevent Duplicate Alerts
- **Module:** prism-operations
- **Category:** Correctness

## Proof Method

| Method | Tool | Bounded? | Coverage |
|--------|------|----------|----------|
| proptest | proptest (latest) | No — random matches per mode | All three modes and boundary field-combinations |

## Proof Harness Skeleton

```rust
// [TODO: harness skeleton — author during Phase 5 formal-verify]
// Method: proptest
// Target: prism_operations::detections::dedup_key
//
// Sketch: for each mode, generate pairs of matches differing only in non-key
// fields; assert equal dedup_key. Generate pairs differing in key fields and
// assert unequal dedup_key.
```

## Feasibility Assessment

| Factor | Assessment | Notes |
|--------|------------|-------|
| Bounded inputs? | No | Size-bounded match generator |
| Tool support? | Full | proptest |
| Execution time budget | <60 seconds | Fast key comparisons |
| Assumptions required | Dedup spec per-mode stable during test | Fixture-co-located |

## Lifecycle

| Event | Date | Actor |
|-------|------|-------|
| introduced | 2026-04-14 | architect |

## Changelog

| Version | Burst | Date | Author | Notes |
|---------|-------|------|--------|-------|
| 1.3 | pass-88-remediation | 2026-04-21 | architect | F88-012: Anchor Story normalized from slug form (S-4.04-detection-evaluation.md) to pure ID (S-4.04). |
| 1.2 | pass-85 F85-001 | 2026-04-21 | architect | Fixed source_bc mis-anchor: BC-2.13.003 → BC-2.13.013; updated body Source BC label to canonical BC-2.13.013 title. |
| 1.1 | pre-build-sweep | 2026-04-20 | architect | Template-compliance sweep: added priority frontmatter (from VP-INDEX v1.5); added verification_method alias (proof_method retained for backward compat). |
