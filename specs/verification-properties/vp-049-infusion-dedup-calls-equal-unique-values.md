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
  - specs/behavioral-contracts/BC-2.19.002
input-hash: "b4904e222c060f7e59d7290aff52de2e"
traces_to: architecture/verification-architecture.md
source_bc: BC-2.19.002
module: prism-spec-engine
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

# VP-049: Infusion Per-Query Dedup — Source Calls Equal Unique Value Count

## Property Statement

For any input sequence of N values containing K distinct values (1 <= K <= N <= 10000),
the per-query dedup cache results in exactly K calls to `InfusionSource::enrich_single`.
After processing all N values, the dedup cache contains exactly K entries. Duplicate
values (second and subsequent occurrences) reuse the cached result without invoking the
source again.

## Source Contract

- **Anchor Story:** `S-1.14`
- **Source BC:** BC-2.19.002 — Per-Query Dedup Cache
- **Module:** prism-spec-engine
- **Category:** Correctness / Performance

## Proof Method

| Method | Tool | Bounded? | Coverage |
|--------|------|----------|----------|
| proptest | proptest (latest) | No — parameterized over (N, K) with K <= N <= 10000 | All (N, K) combinations including K=1 (all same), K=N (all distinct), and random K |

## Proof Harness Skeleton

```rust
// [TODO: harness skeleton — author during Phase 5 formal-verify]
// Method: proptest
// Target: prism_spec_engine::infusion::DedupCache (wrapping InfusionSource)
//
// proptest!(|(values in arb_values_with_duplicates(1..=10000usize, 1..=100usize))| {
//     // values is a Vec<String> with K distinct values, N total
//     let k = values.iter().collect::<std::collections::HashSet<_>>().len();
//     let n = values.len();
//
//     let call_counter = Arc::new(AtomicUsize::new(0));
//     let counter_clone = call_counter.clone();
//
//     let source = MockInfusionSource::new(move |_value| {
//         counter_clone.fetch_add(1, Ordering::SeqCst);
//         Ok(EnrichedValue::default())
//     });
//
//     let dedup_cache = DedupCache::new(source);
//
//     for value in &values {
//         dedup_cache.enrich(value).unwrap();
//     }
//
//     let calls = call_counter.load(Ordering::SeqCst);
//     prop_assert_eq!(calls, k,
//         "source must be called exactly K={} times for N={} values", k, n);
//     prop_assert_eq!(dedup_cache.len(), k,
//         "cache must contain exactly K={} entries", k);
// });
```

## Feasibility Assessment

| Factor | Assessment | Notes |
|--------|------------|-------|
| Bounded inputs? | No — but N capped at 10000 per spec | proptest generates varied (N, K) pairs and shrinks to minimal failing case |
| Tool support? | Full | proptest + mock InfusionSource with call counter; entirely independent of DataFusion |
| Execution time budget | <120 seconds for 1000 cases | In-memory HashMap operations; no I/O |
| Assumptions required | DedupCache is testable with an injected MockInfusionSource; per-query cache is scoped to a single DedupCache instance | Standard test-injection pattern |

## Lifecycle

| Event | Date | Actor |
|-------|------|-------|
| introduced | 2026-04-20 | architect |

## Changelog

| Version | Burst | Date | Author | Notes |
|---------|-------|------|--------|-------|
| 1.0 | pass-69-housekeeping | 2026-04-20 | architect | Initial draft. Resolves VP-TBD in BC-2.19.002. Tests all (N, K) regimes including degenerate cases K=1 (all identical) and K=N (all distinct). |
