---
document_type: verification-property
level: L4
version: "1.1"
status: draft
producer: architect
timestamp: 2026-04-20T00:00:00Z
phase: 2-patch
inputs:
  - specs/prd.md
  - specs/verification-properties/VP-INDEX.md
  - specs/behavioral-contracts/BC-2.15.004
input-hash: "572c2a9"
traces_to: architecture/verification-architecture.md
source_bc: BC-2.15.004
module: prism-audit
priority: P1
proof_method: proptest
verification_method: proptest
feasibility: feasible
verification_lock: false
proof_completed_date: null
proof_file_hash: null
lifecycle_status: active
introduced: cycle-2-patch
modified: []
deprecated: null
deprecated_by: null
replacement: null
retired: null
withdrawn: null
withdrawal_reason: null
removed: null
removal_reason: null
---

# VP-056: Audit Buffer Overflow Purge Preserves Newest Entries

## Property Statement

For any audit buffer of N entries (N > threshold), `compute_purge_targets(entries, threshold)`
returns exactly the oldest `(N - floor(threshold * 0.9))` entries by timestamp key. The newest
`floor(threshold * 0.9)` entries are never included in the purge target set. A purge-event
record is always included in the output as a separate audit entry documenting the purge.

The function `compute_purge_targets(entries: &[(key, entry)], threshold: usize) -> PurgeResult`
is deterministic and pure — given the same inputs, it always produces the same purge target list.

## Source Contract

- **Anchor Story:** `S-5.10`
- **Source BC:** BC-2.15.004 — Audit Buffer Overflow — Purge Oldest Entries When Exceeding 100K, Log Warning
- **Module:** prism-audit
- **Category:** Data Integrity / Buffer Management

## Proof Method

| Method | Tool | Bounded? | Coverage |
|--------|------|----------|----------|
| proptest | proptest (latest) | Yes — buffer sizes N up to 10,000; threshold values 1..=5,000 | All valid (N > threshold) configurations |

**Feasibility:** The purge selection function is a pure sorting + slicing operation over a
timestamp-keyed buffer. No I/O or RocksDB dependency in the selection logic. Proptest can
exhaustively verify the oldest-N deletion and newest-K retention invariants across all buffer
sizes and threshold configurations.

## Proof Harness Skeleton

```rust
// [TODO: harness skeleton — author during Phase 5 formal-verify]
// Method: proptest
// Target: prism_audit::buffer::compute_purge_targets
//
// proptest! {
//     #[test]
//     fn verify_purge_preserves_newest(
//         n_entries in 2usize..=10_000,
//         threshold_fraction in 0.5f64..=0.99,
//     ) {
//         let threshold = (n_entries as f64 * threshold_fraction) as usize;
//         prop_assume!(n_entries > threshold && threshold >= 1);
//
//         // Generate N entries with distinct timestamps (ascending)
//         let entries: Vec<(AuditKey, AuditEntry)> = (0..n_entries)
//             .map(|i| (AuditKey::from_timestamp(i as u64), AuditEntry::default()))
//             .collect();
//
//         let result = compute_purge_targets(&entries, threshold);
//
//         let retain_count = (threshold as f64 * 0.9).floor() as usize;
//         let purge_count = n_entries - retain_count;
//
//         // Exact purge count
//         prop_assert_eq!(result.targets.len(), purge_count,
//             "purge target count must equal N - floor(threshold * 0.9)");
//
//         // Purge targets must be the OLDEST entries (smallest timestamp keys)
//         let oldest_keys: HashSet<_> = entries[..purge_count].iter().map(|(k, _)| k).collect();
//         for target_key in &result.targets {
//             prop_assert!(oldest_keys.contains(target_key),
//                 "purge target must be an oldest entry");
//         }
//
//         // Newest entries must NOT be in purge targets
//         let newest_keys: HashSet<_> = entries[purge_count..].iter().map(|(k, _)| k).collect();
//         for target_key in &result.targets {
//             prop_assert!(!newest_keys.contains(target_key),
//                 "newest entries must never be purge targets");
//         }
//
//         // Purge-event record always present
//         prop_assert!(result.purge_event.is_some(),
//             "purge-event audit entry must always be produced");
//     }
// }
```

## Feasibility Assessment

| Factor | Assessment | Notes |
|--------|------------|-------|
| Bounded inputs? | Yes | Buffer size and threshold are numeric parameters; fully bounded |
| Tool support? | Full | Pure sorting and slicing function; no side effects |
| Execution time budget | <3 minutes | Sorting N entries; proptest with bounded N ≤ 10,000 is fast |
| Assumptions required | `compute_purge_targets` is extracted as a pure function separate from RocksDB delete operations | Phase 3 story author must ensure this extraction |

## Lifecycle

| Event | Date | Actor |
|-------|------|-------|
| introduced | 2026-04-20 | architect |

## Changelog

| Version | Burst | Date | Author | Notes |
|---------|-------|------|--------|-------|
| 1.1 | pass-87-remediation | 2026-04-21 | architect | F87-006: Source BC label corrected "Audit Buffer Overflow" → "Audit Buffer Overflow — Purge Oldest Entries When Exceeding 100K, Log Warning" (matches BC-2.15.004 H1). |
| 1.0 | pass-74-vp-additions | 2026-04-20 | architect | Initial draft. Resolves VP-TBD in BC-2.15.004. Proves oldest-first purge with newest-preservation and purge-event emission. Method: Proptest. P1. |
