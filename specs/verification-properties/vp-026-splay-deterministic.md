---
document_type: verification-property
level: L4
version: "1.3"
status: draft
producer: architect
timestamp: 2026-04-19T00:00:00
phase: 1c
inputs: [VP-INDEX.md, S-4.01-schedule-crud.md]
input-hash: "219aeab"
traces_to: architecture/verification-architecture.md
source_bc: BC-2.12.004
module: prism-operations
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

# VP-026: Splay Computation — Deterministic Per (Query, Client)

## Property Statement

For every `(query_id, client_id)` pair and every configured splay window, the
function `compute_splay(query_id, client_id, window)` returns the same offset for
repeated calls with the same inputs. Splay is a pure function of its inputs and
does not consult wall-clock time or RNG.

## Source Contract

- **Anchor Story:** `S-4.01`
- **Source BC:** BC-2.12.004 — Schedule Execution Loop — Tick-Based with Splay and In-Flight Skip
- **Module:** prism-operations
- **Category:** Correctness

## Proof Method

| Method | Tool | Bounded? | Coverage |
|--------|------|----------|----------|
| kani | Kani (latest) | Yes — bounded id lengths and window | Determinism over input domain |

## Proof Harness Skeleton

```rust
// [TODO: harness skeleton — author during Phase 5 formal-verify]
// Method: kani
// Target: prism_operations::splay::compute_splay
//
// Sketch: symbolic (qid, cid, win); assert compute_splay(qid, cid, win) ==
// compute_splay(qid, cid, win).
```

## Feasibility Assessment

| Factor | Assessment | Notes |
|--------|------------|-------|
| Bounded inputs? | Yes | id lengths <= 16, window bounded |
| Tool support? | Partial | Hash as UF |
| Execution time budget | <5 minutes | Small symbolic domain |
| Assumptions required | Hash is UF; no global state | Enforced by impl review |

## Lifecycle

| Event | Date | Actor |
|-------|------|-------|
| introduced | 2026-04-14 | architect |

## Changelog

| Version | Burst | Date | Author | Notes |
|---------|-------|------|--------|-------|
| 1.3 | pass-88-remediation | 2026-04-21 | architect | F88-012: Anchor Story normalized from slug form (S-4.01-schedule-crud.md) to pure ID (S-4.01). |
| 1.2 | pass-86-remediation | 2026-04-21 | architect | F86-008: updated body Source BC label to canonical BC-2.12.004 title (Schedule Execution Loop — Tick-Based with Splay and In-Flight Skip). |
| 1.1 | pre-build-sweep | 2026-04-20 | architect | Template-compliance sweep: added priority frontmatter (from VP-INDEX v1.5); added verification_method alias (proof_method retained for backward compat). |
