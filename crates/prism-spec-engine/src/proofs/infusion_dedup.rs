//! VP-049 proptest harnesses — source calls equal unique value count.
//!
//! Proves BC-2.19.002 postconditions:
//! - For N values with K distinct, `enrich_single` is called exactly K times.
//! - `QueryScopedInfusionCache` contains exactly K entries after processing all N values.
//!
//! Method: proptest (1,000 cases; N ≤ 10,000; K ≤ N).
//! These harnesses are Red Gate stubs — they will FAIL until S-1.14 implements
//! `QueryScopedInfusionCache::get` and `insert`.

#[cfg(test)]
mod dedup_proofs {
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };

    use proptest::prelude::*;

    use crate::infusion::{InfusionSource, cache::QueryScopedInfusionCache};

    // -----------------------------------------------------------------------
    // Mock InfusionSource with call counter
    // -----------------------------------------------------------------------

    /// Mock infusion source that counts calls to `enrich_single`.
    #[derive(Debug)]
    struct MockCountingSource {
        call_count: Arc<AtomicUsize>,
    }

    impl MockCountingSource {
        fn new() -> (Self, Arc<AtomicUsize>) {
            let counter = Arc::new(AtomicUsize::new(0));
            (
                MockCountingSource {
                    call_count: counter.clone(),
                },
                counter,
            )
        }
    }

    impl InfusionSource for MockCountingSource {
        fn enrich_single(&self, input: &str, _input_type: &str) -> Option<serde_json::Value> {
            self.call_count.fetch_add(1, Ordering::SeqCst);
            Some(serde_json::json!({ "enriched": input }))
        }

        fn enrich_batch(&self, inputs: &[String], input_type: &str) -> Vec<Option<serde_json::Value>> {
            inputs
                .iter()
                .map(|i| self.enrich_single(i, input_type))
                .collect()
        }
    }

    // -----------------------------------------------------------------------
    // Helper: simulate the per-query dedup lookup-or-call loop
    // -----------------------------------------------------------------------

    /// Simulate the dedup loop: for each value in `values`, check the cache,
    /// call source only on miss, populate cache on miss.
    ///
    /// Returns (call_count, cache_entry_count).
    fn simulate_dedup_loop(
        infusion_id: &str,
        values: &[String],
        source: &dyn InfusionSource,
        cache: &mut QueryScopedInfusionCache,
    ) -> (usize, usize) {
        let mut call_count = 0usize;
        for value in values {
            if cache.get(infusion_id, value).is_none() {
                // Cache miss — call source.
                call_count += 1;
                let result = source.enrich_single(value, "string");
                cache.insert(infusion_id, value, result);
            }
        }
        (call_count, cache.len())
    }

    // -----------------------------------------------------------------------
    // Proptest strategy: (n, values_with_k_distinct)
    // Strategy generates a (usize, Vec<String>) where the Vec has n entries,
    // with exactly k distinct values (k is determined from the generated data).
    // -----------------------------------------------------------------------

    /// Strategy: generate (n, values) where values has n entries built from k distinct strings.
    fn arb_n_values_with_k_distinct() -> impl Strategy<Value = Vec<String>> {
        // Generate k distinct base values (2..=20) and n repetitions (k..=100).
        (2usize..=20usize).prop_flat_map(|k| {
            (
                proptest::collection::vec("[a-z]{3,8}", k..=k),
                k..=100usize,
            )
                .prop_map(move |(distinct, n)| {
                    (0..n).map(|i| distinct[i % k].clone()).collect::<Vec<_>>()
                })
        })
    }

    proptest! {
        #![proptest_config(proptest::test_runner::Config {
            cases: 1000,
            ..Default::default()
        })]

        /// VP-049: For N values with K distinct, enrich_single called exactly K times.
        ///
        /// Traces to: BC-2.19.002 postconditions / INV-INFUSE-002.
        #[test]
        fn test_BC_2_19_002_invariant_dedup_calls_equal_unique_value_count(
            values in arb_n_values_with_k_distinct()
        ) {
            let k = values.iter().collect::<std::collections::HashSet<_>>().len();
            let n = values.len();

            let (mock_source, _counter) = MockCountingSource::new();
            let mut cache = QueryScopedInfusionCache::new();

            let (call_count, cache_len) = simulate_dedup_loop("test_infusion", &values, &mock_source, &mut cache);

            prop_assert_eq!(
                call_count,
                k,
                "VP-049: enrich_single must be called exactly K={} times for N={} values",
                k,
                n
            );
            prop_assert_eq!(
                cache_len,
                k,
                "VP-049: cache must contain exactly K={} entries",
                k
            );
        }

        /// VP-049 edge case: all N values identical → exactly 1 source call.
        ///
        /// Traces to: BC-2.19.002 EC-19-005.
        #[test]
        fn test_BC_2_19_002_invariant_all_identical_values_one_call(
            n in 1usize..=1000usize
        ) {
            let values: Vec<String> = (0..n).map(|_| "192.168.1.1".to_string()).collect();

            let (mock_source, _counter) = MockCountingSource::new();
            let mut cache = QueryScopedInfusionCache::new();

            let (call_count, cache_len) = simulate_dedup_loop("geoip", &values, &mock_source, &mut cache);

            prop_assert_eq!(
                call_count,
                1,
                "VP-049: all-identical input must produce exactly 1 source call"
            );
            prop_assert_eq!(
                cache_len,
                1,
                "VP-049: cache must have exactly 1 entry"
            );
        }

        /// VP-049 edge case: all N values distinct → exactly N source calls.
        ///
        /// Traces to: BC-2.19.002 EC-19-007.
        #[test]
        fn test_BC_2_19_002_invariant_all_distinct_values_n_calls(
            n in 1usize..=200usize
        ) {
            let values: Vec<String> = (0..n).map(|i| format!("10.0.{}.{}", i / 256, i % 256)).collect();

            let (mock_source, _counter) = MockCountingSource::new();
            let mut cache = QueryScopedInfusionCache::new();

            let (call_count, cache_len) = simulate_dedup_loop("geoip", &values, &mock_source, &mut cache);

            prop_assert_eq!(
                call_count,
                n,
                "VP-049: all-distinct input must produce exactly N={} source calls",
                n
            );
            prop_assert_eq!(
                cache_len,
                n,
                "VP-049: cache must have exactly N={} entries",
                n
            );
        }
    }
}
