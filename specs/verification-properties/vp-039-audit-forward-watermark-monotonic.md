---
document_type: verification-property
level: L4
version: "1.3"
status: draft
producer: architect
timestamp: 2026-04-19T00:00:00
phase: 1c
inputs: [VP-INDEX.md, S-5.10-audit-trail-forwarding.md]
input-hash: "6c47ec1"
traces_to: architecture/verification-architecture.md
source_bc: BC-2.05.011
module: prism-audit
priority: P0
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

# VP-039: Audit Forward Watermark — Monotonically Non-Decreasing Per Destination

## Property Statement

For every destination `d`, across every legal sequence of events (ACK, transient
network failure, permanent destination failure, and process restart with RocksDB
watermark recovery), the persisted forward watermark `W[d]` is monotonically
non-decreasing. For any two observations `W_t1[d]` and `W_t2[d]` with `t1 <= t2`,
`W_t1[d] <= W_t2[d]` holds; the watermark never rewinds even after crash-recovery.

## Source Contract

- **Anchor Story:** `S-5.10`
- **Source BC:** BC-2.05.011 — Audit Forwarding — At-Least-Once Delivery to External Destinations (VP-039 monotonic watermark)
- **Module:** prism-audit
- **Category:** Correctness

## Proof Method

| Method | Tool | Bounded? | Coverage |
|--------|------|----------|----------|
| kani | Kani (latest) | Yes — bounded event sequences (<=6 events, <=3 destinations) | All ACK/failure/restart orderings within bound |

## Proof Harness Skeleton

```rust
// [TODO: harness skeleton — author during Phase 5 formal-verify]
// Method: kani
// Target: prism_audit::forwarder::Watermark
//
// Sketch: model destination state machine with symbolic event sequence;
// after each state transition assert current_watermark >= previous_watermark.
// Include simulated restart path that reloads watermark from RocksDB mock.
```

## Feasibility Assessment

| Factor | Assessment | Notes |
|--------|------------|-------|
| Bounded inputs? | Yes | 6 events x 3 destinations |
| Tool support? | Full | Kani handles small state machines well |
| Execution time budget | <15 minutes | State-machine exploration |
| Assumptions required | RocksDB watermark read/write modeled as pure mapping | Documented abstraction |

## Lifecycle

| Event | Date | Actor |
|-------|------|-------|
| introduced | 2026-04-14 | architect |

## Changelog

| Version | Burst | Date | Author | Notes |
|---------|-------|------|--------|-------|
| 1.3 | pass-88-remediation | 2026-04-21 | architect | F88-012: Anchor Story normalized from slug form (S-5.10-audit-trail-forwarding.md) to pure ID (S-5.10). |
| 1.2 | pass-87-remediation | 2026-04-21 | architect | F87-006: Source BC label corrected "Audit Forwarding At-Least-Once" → "Audit Forwarding — At-Least-Once Delivery to External Destinations (VP-039 monotonic watermark)" (matches BC-2.05.011 H1). |
| 1.1 | pre-build-sweep | 2026-04-20 | architect | Template-compliance sweep: added priority frontmatter (from VP-INDEX v1.5); added verification_method alias (proof_method retained for backward compat). |
