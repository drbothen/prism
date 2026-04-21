---
document_type: verification-property
level: L4
version: "1.3"
status: draft
producer: architect
timestamp: 2026-04-19T00:00:00
phase: 1c
inputs: [VP-INDEX.md, S-4.02-diff-results-packs.md]
input-hash: "ff9068b"
traces_to: architecture/verification-architecture.md
source_bc: BC-2.12.005
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

# VP-019: Diff Computation — Deterministic

## Property Statement

For every pair of result sets `(a, b)`, `diff(a, b)` returns the same `DiffResult`
across repeated invocations and across any permutation of the input rows. The
function is pure, order-independent over the rows of each set, and produces no
nondeterministic output (no timestamps, no HashMap iteration order leakage).

## Source Contract

- **Anchor Story:** `S-4.02`
- **Source BC:** BC-2.12.005 — Differential Result Computation — Hash Previous Results, Return Added/Removed
- **Module:** prism-operations
- **Category:** Correctness

## Proof Method

| Method | Tool | Bounded? | Coverage |
|--------|------|----------|----------|
| proptest | proptest (latest) | No — random result sets + permutations | Order independence and repeatability |

## Proof Harness Skeleton

```rust
// [TODO: harness skeleton — author during Phase 5 formal-verify]
// Method: proptest
// Target: prism_operations::diff::diff
//
// Sketch: generate (a, b); compute d1 = diff(a,b); permute a and b -> a', b';
// compute d2 = diff(a', b'); assert d1 == d2.
```

## Feasibility Assessment

| Factor | Assessment | Notes |
|--------|------------|-------|
| Bounded inputs? | No | Size-bounded result sets |
| Tool support? | Full | proptest with shuffle strategy |
| Execution time budget | <60 seconds | Lightweight row ops |
| Assumptions required | Rows have total order for canonicalization | Required for determinism |

## Lifecycle

| Event | Date | Actor |
|-------|------|-------|
| introduced | 2026-04-14 | architect |

## Changelog

| Version | Burst | Date | Author | Notes |
|---------|-------|------|--------|-------|
| 1.3 | pass-88-remediation | 2026-04-21 | architect | F88-012: Anchor Story normalized from slug form (S-4.02-diff-results-packs.md) to pure ID (S-4.02). |
| 1.2 | pass-87-remediation | 2026-04-21 | architect | F87-006: Source BC label corrected "Differential Result Computation" → "Differential Result Computation — Hash Previous Results, Return Added/Removed" (matches BC-2.12.005 H1). |
| 1.1 | pre-build-sweep | 2026-04-20 | architect | Template-compliance sweep: added priority frontmatter (from VP-INDEX v1.5); added verification_method alias (proof_method retained for backward compat). |
