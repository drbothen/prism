// VP-055: StorageEngine put_batch Atomicity and Domain Isolation.
//
// Method: proptest over MockStorageEngine.
//
// Prop 1 (atomicity): inject failure at position N < K in a K-entry batch;
//   assert zero entries readable after failure.
//
// Prop 2 (domain isolation): write key K to domain A; assert get(domain_B, K) == None.
//
// Traces to: BC-2.15.002 postconditions.

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use prism_core::StorageDomain;

    use crate::backend::StorageBackend;
    use crate::mock::MockStorageEngine;

    // ── Helpers ───────────────────────────────────────────────────────────────

    fn arb_key() -> impl Strategy<Value = Vec<u8>> {
        proptest::collection::vec(any::<u8>(), 1..=32)
    }

    fn arb_value() -> impl Strategy<Value = Vec<u8>> {
        proptest::collection::vec(any::<u8>(), 1..=64)
    }

    fn arb_kv_entry() -> impl Strategy<Value = (Vec<u8>, Vec<u8>)> {
        (arb_key(), arb_value())
    }

    fn arb_kv_batch(min: usize, max: usize) -> impl Strategy<Value = Vec<(Vec<u8>, Vec<u8>)>> {
        proptest::collection::vec(arb_kv_entry(), min..=max)
    }

    /// Two distinct `StorageDomain` values.
    fn arb_domain_pair() -> impl Strategy<Value = (StorageDomain, StorageDomain)> {
        // Use a fixed pair that is always distinct for determinism.
        // StorageDomain doesn't impl Arbitrary; use a concrete distinct pair.
        prop_oneof![
            Just((StorageDomain::Cases, StorageDomain::Alerts)),
            Just((StorageDomain::Alerts, StorageDomain::Cases)),
            Just((StorageDomain::Cases, StorageDomain::Credentials)),
            Just((StorageDomain::Credentials, StorageDomain::Cases)),
            Just((StorageDomain::Scheduler, StorageDomain::FeatureFlags)),
        ]
    }

    // ── VP-055 Prop 1: put_batch atomicity ────────────────────────────────────

    proptest! {
        #[test]
        fn test_BC_S_02_vp055_put_batch_atomicity_failed_batch_zero_readable(
            entries in arb_kv_batch(1, 20),
            fail_index in 0usize..20,
        ) {
            let fail_at = fail_index % entries.len();
            let mut engine = MockStorageEngine::new_with_failure_at(fail_at);

            let result = engine.put_batch(StorageDomain::Cases, &entries);
            prop_assert!(result.is_err(), "injected failure must produce Err");

            // No entry from the batch must be readable after the failed write.
            for (key, _) in &entries {
                let got = engine.get(StorageDomain::Cases, key);
                prop_assert!(
                    got.is_none(),
                    "no partial batch entries must be readable after failure: key={key:?}"
                );
            }
        }
    }

    // ── VP-055 Prop 2: domain isolation ──────────────────────────────────────

    proptest! {
        #[test]
        fn test_BC_S_02_vp055_domain_isolation_write_a_not_visible_in_b(
            (domain_a, domain_b) in arb_domain_pair(),
            key in arb_key(),
            value in arb_value(),
        ) {
            prop_assume!(domain_a != domain_b);

            let mut engine = MockStorageEngine::new();
            engine.put(domain_a, key.clone(), value).unwrap();

            // Value written to domain A must not be visible in domain B.
            let got = engine.get(domain_b, &key);
            prop_assert!(
                got.is_none(),
                "write to {domain_a:?} must not be visible in {domain_b:?}"
            );
        }
    }

    // ── Concrete unit test: successful put_batch is readable ──────────────────
    // This test also calls unimplemented stubs and MUST FAIL (Red Gate).

    #[test]
    fn test_BC_S_02_vp055_successful_put_batch_entries_are_readable() {
        let mut engine = MockStorageEngine::new();
        let entries = vec![
            (b"key1".to_vec(), b"value1".to_vec()),
            (b"key2".to_vec(), b"value2".to_vec()),
        ];
        engine
            .put_batch(StorageDomain::Cases, &entries)
            .expect("successful batch must not fail");

        assert_eq!(
            engine.get(StorageDomain::Cases, b"key1"),
            Some(b"value1".to_vec())
        );
        assert_eq!(
            engine.get(StorageDomain::Cases, b"key2"),
            Some(b"value2".to_vec())
        );
    }
}
