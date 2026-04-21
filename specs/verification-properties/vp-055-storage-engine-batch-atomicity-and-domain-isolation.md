---
document_type: verification-property
level: L4
version: "1.0"
status: draft
producer: architect
timestamp: 2026-04-20T00:00:00Z
phase: 2-patch
inputs:
  - specs/prd.md
  - specs/verification-properties/VP-INDEX.md
  - specs/behavioral-contracts/BC-2.15.002
input-hash: "3ff257e"
traces_to: architecture/verification-architecture.md
source_bc: BC-2.15.002
module: prism-persistence
priority: P1
proof_method: proptest
verification_method: proptest
feasibility: conditional
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

# VP-055: StorageEngine put_batch Atomicity and Domain Isolation

## Property Statement

Two distinct invariants verified over a `MockStorageEngine`:

1. **Batch atomicity:** For any `put_batch` where the underlying write fails partway through
   (failure injected at position N < K in a K-entry batch), zero entries from that batch are
   readable via `get` after the failure. All-or-nothing semantics hold for every batch.

2. **Domain isolation:** For any pair of distinct storage domains (A, B), a write to domain A
   at key K produces `get(domain_B, K) == None`. Writes in one domain never appear in another
   domain regardless of key overlap.

**Feasibility caveat:** These proofs target a `MockStorageEngine` that faithfully implements
the `StorageBackend` trait. If the mock cannot be made behaviorally equivalent to RocksDB's
all-or-nothing WriteBatch semantics, downgrade to integration test and MARK-NONE.

## Source Contract

- **Anchor Story:** `S-1.02`
- **Source BC:** BC-2.15.002 — Domain KV Operations
- **Module:** prism-persistence
- **Category:** Data Integrity / Storage Abstraction

## Proof Method

| Method | Tool | Bounded? | Coverage |
|--------|------|----------|----------|
| proptest | proptest (latest) | Yes — batch sizes 1..=100, domain pairs from finite set | All batch-size and domain-pair combinations |

**Feasibility caveat:** The mock must implement failure injection at position N. If
`MockStorageEngine` cannot simulate mid-batch failure without coupling to RocksDB internals,
the atomicity proof degrades to an integration test. The domain isolation proof is fully
feasible as a pure mock property regardless.

## Proof Harness Skeleton

```rust
// [TODO: harness skeleton — author during Phase 5 formal-verify]
// Method: proptest
// Target: MockStorageEngine implementing StorageBackend trait
//
// proptest! {
//     #[test]
//     fn verify_put_batch_atomicity(
//         entries in vec(arb_kv_entry(), 1..=100),
//         fail_at in 0usize..100,
//     ) {
//         let fail_at = fail_at % entries.len();  // inject failure within batch
//         let mut engine = MockStorageEngine::new_with_failure_at(fail_at);
//
//         let result = engine.put_batch(StorageDomain::Cases, &entries);
//         prop_assert!(result.is_err(), "injected failure must produce Err");
//
//         // No entry from the batch must be readable
//         for (key, _) in &entries {
//             let got = engine.get(StorageDomain::Cases, key);
//             prop_assert!(got.is_none(), "no partial batch entries must be readable after failure");
//         }
//     }
//
//     #[test]
//     fn verify_domain_isolation(
//         domain_a in arb_storage_domain(),
//         domain_b in arb_storage_domain(),
//         key in arb_storage_key(),
//         value in arb_storage_value(),
//     ) {
//         prop_assume!(domain_a != domain_b);
//
//         let mut engine = MockStorageEngine::new();
//         engine.put(domain_a, key.clone(), value).unwrap();
//
//         // Value must not be visible in domain B
//         let got = engine.get(domain_b, &key);
//         prop_assert!(got.is_none(), "write to domain A must not appear in domain B");
//     }
// }
```

## Feasibility Assessment

| Factor | Assessment | Notes |
|--------|------------|-------|
| Bounded inputs? | Yes | Batch sizes 1..=100; StorageDomain is a finite enum |
| Tool support? | Conditional | Domain isolation: fully feasible. Atomicity: requires faithful MockStorageEngine with failure injection |
| Execution time budget | <3 minutes | In-memory mock operations; fast proptest exploration |
| Assumptions required | MockStorageEngine faithfully implements all-or-nothing batch semantics | Phase 3 story author must implement and document mock fidelity constraints |

## Lifecycle

| Event | Date | Actor |
|-------|------|-------|
| introduced | 2026-04-20 | architect |

## Changelog

| Version | Burst | Date | Author | Notes |
|---------|-------|------|--------|-------|
| 1.0 | pass-74-vp-additions | 2026-04-20 | architect | Initial draft. Resolves VP-TBD in BC-2.15.002. Two invariants: batch atomicity (conditional on mock fidelity) and domain isolation (fully feasible). Method: Proptest. P1. |
