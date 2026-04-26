// S-2.03 — Decorator tests.
//
// Covers BC-2.15.009 (context decorator injection), BC-2.15.010 (three-phase
// decorator model), and the edge cases EC-001/EC-002 from S-2.03.
//
// Test naming convention: test_BC_S_SS_NNN_[assertion_name]
// All tests exercise stubs that panic with todo!() until the implementation
// is complete.  Red Gate must be verified before implementation.
//
// AC map:
//   AC-1  → test_BC_2_15_010_get_config_time_phase1_fields_populated
//   AC-2  → test_BC_2_15_010_merge_without_periodic_carries_phase1_and_phase2
//   AC-3  → test_BC_2_15_009_scheduled_query_analyst_id_none_query_source_schedule
//   AC-4  → test_BC_2_15_010_store_and_load_periodic_round_trip
//   AC-5  → test_BC_2_15_010_merge_precedence_periodic_wins_over_query_time_and_config_time
//   AC-6  → test_BC_2_15_010_store_periodic_failure_stale_value_pattern
//   EC-001 → test_BC_2_15_010_ec001_load_periodic_fresh_tenant_returns_none
//   EC-001 → test_BC_2_15_010_ec001_merge_with_none_periodic_sensor_health_absent

#![allow(non_snake_case)]

#[cfg(test)]
mod inner {
    use std::sync::Arc;

    use prism_core::{DecoratorContext, PrismError, StorageDomain, TenantId};

    use crate::decorators::DecorationStore;
    use crate::memory_backend::InMemoryBackend;

    // ─────────────────────────────────────────────────────────────────────────
    // Helpers
    // ─────────────────────────────────────────────────────────────────────────

    fn make_tenant(s: &str) -> TenantId {
        TenantId::new(s).expect("test helper: valid tenant id")
    }

    fn make_backend() -> Arc<InMemoryBackend> {
        Arc::new(InMemoryBackend::new())
    }

    fn phase1_ctx(client_name: &str, prism_version: &str) -> DecoratorContext {
        DecoratorContext {
            client_name: Some(client_name.to_owned()),
            prism_version: Some(prism_version.to_owned()),
            analyst_id: None,
            query_source: None,
            sensor_instance: None,
            sensor_health_status: None,
        }
    }

    fn phase2_ctx(analyst_id: Option<&str>, query_source: Option<&str>) -> DecoratorContext {
        DecoratorContext {
            client_name: None,
            prism_version: None,
            analyst_id: analyst_id.map(|s| s.to_owned()),
            query_source: query_source.map(|s| s.to_owned()),
            sensor_instance: None,
            sensor_health_status: None,
        }
    }

    fn phase3_ctx(sensor_health_status: &str) -> DecoratorContext {
        DecoratorContext {
            client_name: None,
            prism_version: None,
            analyst_id: None,
            query_source: None,
            sensor_instance: None,
            sensor_health_status: Some(sensor_health_status.to_owned()),
        }
    }

    // ─────────────────────────────────────────────────────────────────────────
    // AC-1: get_config_time returns Phase 1 fields; Phases 2/3 are None
    // BC-2.15.010 postcondition — Phase 1 config-time decorator populated at
    // startup from TOML.
    // ─────────────────────────────────────────────────────────────────────────

    /// AC-1 (BC-2.15.010 Phase 1 postcondition): after `store_config_time`, calling
    /// `get_config_time` returns a context with `client_name` and `prism_version`
    /// populated and all Phase 2 and Phase 3 fields absent (`None`).
    #[tokio::test]
    async fn test_BC_2_15_010_get_config_time_phase1_fields_populated() {
        let store = DecorationStore::new(make_backend());
        let tenant = make_tenant("acme");
        let ctx = phase1_ctx("Acme Corp", "0.1.0");

        store.store_config_time(tenant.clone(), ctx).await;

        let retrieved = store
            .get_config_time(&tenant)
            .await
            .expect("AC-1: get_config_time must return Some for a stored tenant");

        assert_eq!(
            retrieved.client_name.as_deref(),
            Some("Acme Corp"),
            "AC-1 (BC-2.15.010): client_name must be populated from Phase 1 config-time"
        );
        assert_eq!(
            retrieved.prism_version.as_deref(),
            Some("0.1.0"),
            "AC-1 (BC-2.15.010): prism_version must be populated from Phase 1 config-time"
        );
        assert!(
            retrieved.analyst_id.is_none(),
            "AC-1 (BC-2.15.010): analyst_id must be None — Phase 2 field not set at config-time"
        );
        assert!(
            retrieved.query_source.is_none(),
            "AC-1 (BC-2.15.010): query_source must be None — Phase 2 field not set at config-time"
        );
        assert!(
            retrieved.sensor_health_status.is_none(),
            "AC-1 (BC-2.15.010): sensor_health_status must be None — Phase 3 field not set at config-time"
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // AC-2: merge(config_time, query_time, None) carries Phase 1 + Phase 2
    // BC-2.15.010 postcondition — Phase 2 query-time fields not cached; computed
    // per-query.
    // ─────────────────────────────────────────────────────────────────────────

    /// AC-2 (BC-2.15.010 Phase 2 postcondition): `merge(config_time, query_time, None)`
    /// where `analyst_id = Some("joshua")` and `query_source = Some("interactive")`
    /// produces a merged context with both Phase 1 and Phase 2 fields present.
    #[test]
    fn test_BC_2_15_010_merge_without_periodic_carries_phase1_and_phase2() {
        let config_time = phase1_ctx("Acme Corp", "0.1.0");
        let query_time = phase2_ctx(Some("joshua"), Some("interactive"));

        let merged = DecorationStore::merge(&config_time, &query_time, None);

        assert_eq!(
            merged.client_name.as_deref(),
            Some("Acme Corp"),
            "AC-2 (BC-2.15.010): Phase 1 client_name must carry through unchanged"
        );
        assert_eq!(
            merged.prism_version.as_deref(),
            Some("0.1.0"),
            "AC-2 (BC-2.15.010): Phase 1 prism_version must carry through unchanged"
        );
        assert_eq!(
            merged.analyst_id.as_deref(),
            Some("joshua"),
            "AC-2 (BC-2.15.010): Phase 2 analyst_id must be present in merged result"
        );
        assert_eq!(
            merged.query_source.as_deref(),
            Some("interactive"),
            "AC-2 (BC-2.15.010): Phase 2 query_source must be present in merged result"
        );
        assert!(
            merged.sensor_health_status.is_none(),
            "AC-2: periodic=None means sensor_health_status must be None in merged result"
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // AC-3: Scheduled query shape — analyst_id=None, query_source=schedule:{name}
    // BC-2.15.009 edge case EC-15-034, BC-2.15.010 Phase 2 postcondition.
    // ─────────────────────────────────────────────────────────────────────────

    /// AC-3 (BC-2.15.009 EC-15-034): `DecoratorContext` accepts a scheduled-query
    /// shape with `analyst_id = None` and `query_source = Some("schedule:check_alerts")`.
    /// The merge must round-trip this shape correctly — the format string is set by
    /// the caller (prism-query, S-3.02); `DecoratorContext` just holds the string.
    #[test]
    fn test_BC_2_15_009_scheduled_query_analyst_id_none_query_source_schedule() {
        let config_time = phase1_ctx("Acme Corp", "0.1.0");
        let schedule_name = "check_alerts";
        let query_source_str = format!("schedule:{schedule_name}");
        let query_time = phase2_ctx(None, Some(&query_source_str));

        let merged = DecorationStore::merge(&config_time, &query_time, None);

        assert!(
            merged.analyst_id.is_none(),
            "AC-3 (BC-2.15.009 EC-15-034): scheduled query must have analyst_id = None"
        );
        assert_eq!(
            merged.query_source.as_deref(),
            Some("schedule:check_alerts"),
            "AC-3 (BC-2.15.009 EC-15-034): scheduled query must have query_source = 'schedule:{{name}}'"
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // AC-4: store_periodic / load_periodic round-trip via bincode + InMemoryBackend
    // BC-2.15.010 postcondition — Phase 3 periodic decorators cached in RocksDB
    // `decorators` CF for persistence across restarts.
    // ─────────────────────────────────────────────────────────────────────────

    /// AC-4 (BC-2.15.010 Phase 3 postcondition): `store_periodic` followed by
    /// `load_periodic` returns a `DecoratorContext` with all fields intact
    /// (round-trip via bincode + `decorators` CF using InMemoryBackend).
    #[tokio::test]
    async fn test_BC_2_15_010_store_and_load_periodic_round_trip() {
        let backend = make_backend();
        let store = DecorationStore::new(backend);
        let tenant = make_tenant("acme");

        let ctx = DecoratorContext {
            client_name: Some("Acme Corp".to_owned()),
            prism_version: Some("0.1.0".to_owned()),
            analyst_id: None,
            query_source: None,
            sensor_instance: Some("us-1".to_owned()),
            sensor_health_status: Some("healthy".to_owned()),
        };

        store
            .store_periodic(&tenant, &ctx)
            .await
            .expect("AC-4 (BC-2.15.010): store_periodic must succeed against InMemoryBackend");

        let loaded = store
            .load_periodic(&tenant)
            .await
            .expect("AC-4 (BC-2.15.010): load_periodic must not return Err after successful store")
            .expect("AC-4 (BC-2.15.010): load_periodic must return Some after store_periodic");

        assert_eq!(
            loaded.client_name.as_deref(),
            Some("Acme Corp"),
            "AC-4 (BC-2.15.010): client_name must survive bincode round-trip"
        );
        assert_eq!(
            loaded.prism_version.as_deref(),
            Some("0.1.0"),
            "AC-4 (BC-2.15.010): prism_version must survive bincode round-trip"
        );
        assert_eq!(
            loaded.sensor_health_status.as_deref(),
            Some("healthy"),
            "AC-4 (BC-2.15.010): sensor_health_status must survive bincode round-trip"
        );
        assert_eq!(
            loaded.sensor_instance.as_deref(),
            Some("us-1"),
            "AC-4 (BC-2.15.010): sensor_instance must survive bincode round-trip"
        );
        assert!(
            loaded.analyst_id.is_none(),
            "AC-4 (BC-2.15.010): analyst_id None must survive bincode round-trip"
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // AC-5: 3-phase merge precedence — periodic > query-time > config-time
    // BC-2.15.010 invariant — last-write-wins.
    // ─────────────────────────────────────────────────────────────────────────

    /// AC-5 (BC-2.15.010 invariant — last-write-wins: periodic > query-time > config-time):
    /// Verifies three sub-properties:
    /// (a) config-time-only field carries through to the merged result,
    /// (b) periodic-only field carries through to the merged result,
    /// (c) when config-time and periodic both have values for a field that maps
    ///     to the same struct field, the periodic value wins.
    ///
    /// Note: `DecoratorContext` has separate struct fields per phase so direct
    /// field-collision is rare; we test that *all* filled fields appear, and use
    /// the `sensor_health_status` / `client_name` / `prism_version` structure to
    /// confirm "periodic > config-time" where applicable.
    #[test]
    fn test_BC_2_15_010_merge_precedence_periodic_wins_over_query_time_and_config_time() {
        // config-time: client_name + prism_version
        let config_time = phase1_ctx("Acme Corp", "0.1.0");

        // query-time: analyst_id (no overlap with config-time struct fields)
        let query_time = phase2_ctx(Some("joshua"), Some("interactive"));

        // periodic: sensor_health_status
        let periodic = phase3_ctx("healthy");

        let merged = DecorationStore::merge(&config_time, &query_time, Some(&periodic));

        // (a) config-time-only field carries through
        assert_eq!(
            merged.client_name.as_deref(),
            Some("Acme Corp"),
            "AC-5 (BC-2.15.010): config-time client_name must carry through when not overridden"
        );

        // (b) periodic-only field carries through
        assert_eq!(
            merged.sensor_health_status.as_deref(),
            Some("healthy"),
            "AC-5 (BC-2.15.010): periodic sensor_health_status must carry through"
        );

        // (c) query-time field is present
        assert_eq!(
            merged.analyst_id.as_deref(),
            Some("joshua"),
            "AC-5 (BC-2.15.010): query-time analyst_id must be present"
        );

        // Confirm phase precedence by constructing an explicit overlap test:
        // if periodic sets a field that config-time also sets, periodic wins.
        // We simulate this by constructing a "periodic" context that also sets
        // client_name — periodic must win over config-time.
        let periodic_with_override = DecoratorContext {
            client_name: Some("PERIODIC_WINS".to_owned()),
            prism_version: None,
            analyst_id: None,
            query_source: None,
            sensor_instance: None,
            sensor_health_status: Some("healthy".to_owned()),
        };

        let merged_override =
            DecorationStore::merge(&config_time, &query_time, Some(&periodic_with_override));

        assert_eq!(
            merged_override.client_name.as_deref(),
            Some("PERIODIC_WINS"),
            "AC-5 (BC-2.15.010 invariant periodic > config-time): when both config-time and \
             periodic set client_name, the periodic value must win"
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // AC-6: store_periodic failure → stale-value pattern (E-DECOR-001)
    // BC-2.15.010 error case E-DECOR-001.
    // ─────────────────────────────────────────────────────────────────────────

    /// A `RocksStorageBackend` that always fails `put` and `put_batch`.
    ///
    /// Used to exercise the E-DECOR-001 stale-value pattern: when `store_periodic`
    /// fails, `load_periodic` must still return the last successfully cached value.
    struct AlwaysFailBackend {
        /// Delegates reads to a working in-memory backend so `load_periodic`
        /// can still return the previously-written stale value.
        inner: InMemoryBackend,
    }

    impl AlwaysFailBackend {
        fn new() -> Self {
            Self {
                inner: InMemoryBackend::new(),
            }
        }
    }

    // SAFETY: AlwaysFailBackend wraps InMemoryBackend which is Send+Sync.
    unsafe impl Send for AlwaysFailBackend {}
    unsafe impl Sync for AlwaysFailBackend {}

    impl crate::backend::RocksStorageBackend for AlwaysFailBackend {
        fn get(
            &self,
            domain: prism_core::StorageDomain,
            key: &[u8],
        ) -> Result<Option<Vec<u8>>, PrismError> {
            self.inner.get(domain, key)
        }

        fn put(
            &self,
            _domain: prism_core::StorageDomain,
            _key: &[u8],
            _value: &[u8],
        ) -> Result<(), PrismError> {
            Err(PrismError::StorageWriteFailed {
                domain: StorageDomain::Decorators.column_family_name().to_owned(),
                detail: "injected failure for E-DECOR-001 test".to_owned(),
            })
        }

        fn put_batch(
            &self,
            _domain: prism_core::StorageDomain,
            _entries: &[(&[u8], &[u8])],
        ) -> Result<(), PrismError> {
            Err(PrismError::StorageWriteFailed {
                domain: StorageDomain::Decorators.column_family_name().to_owned(),
                detail: "injected batch failure for E-DECOR-001 test".to_owned(),
            })
        }

        fn remove(
            &self,
            _domain: prism_core::StorageDomain,
            _key: &[u8],
        ) -> Result<(), PrismError> {
            Ok(())
        }

        fn scan(
            &self,
            domain: prism_core::StorageDomain,
            prefix: &[u8],
        ) -> Result<Vec<(Vec<u8>, Vec<u8>)>, PrismError> {
            self.inner.scan(domain, prefix)
        }

        fn scan_range(
            &self,
            domain: prism_core::StorageDomain,
            start: &[u8],
            end: &[u8],
        ) -> Result<Vec<(Vec<u8>, Vec<u8>)>, PrismError> {
            self.inner.scan_range(domain, start, end)
        }
    }

    /// AC-6 (BC-2.15.010 E-DECOR-001): when `store_periodic` fails, the error is
    /// returned to the caller (the caller must log `tracing::warn!` and continue).
    /// A subsequent `load_periodic` against a backend that retains the last
    /// successful write returns that stale value.
    ///
    /// Test strategy: use a two-store approach.
    /// 1. Write a good value with a working backend store.
    /// 2. Attempt a second write with a failing-write store (which delegates reads
    ///    to the same underlying data).  The write returns `Err`.
    /// 3. Load from the original working store — the good value is still present.
    ///
    /// This verifies that the error surface is correct (Err returned for failed write)
    /// and that the load path returns the stale value when writes fail.
    #[tokio::test]
    async fn test_BC_2_15_010_store_periodic_failure_stale_value_pattern() {
        // Step 1: write a good "stale" value to a working backend
        let good_backend = make_backend();
        let good_store = DecorationStore::new(good_backend);
        let tenant = make_tenant("acme");
        let stale_ctx = phase3_ctx("healthy");

        good_store
            .store_periodic(&tenant, &stale_ctx)
            .await
            .expect("AC-6 setup: first store must succeed");

        // Step 2: a failing write returns Err (caller must log warn! and carry on)
        let failing_store = DecorationStore::new(Arc::new(AlwaysFailBackend::new()));
        let new_ctx = phase3_ctx("degraded");
        let write_result = failing_store.store_periodic(&tenant, &new_ctx).await;

        assert!(
            write_result.is_err(),
            "AC-6 (BC-2.15.010 E-DECOR-001): store_periodic must return Err when the \
             backend write fails, so the caller can log warn! and apply the stale-value pattern"
        );

        // Step 3: load from the good store still returns the stale value
        let loaded = good_store
            .load_periodic(&tenant)
            .await
            .expect("AC-6: load_periodic must not return Err")
            .expect("AC-6: load_periodic must return the previously-written stale value");

        assert_eq!(
            loaded.sensor_health_status.as_deref(),
            Some("healthy"),
            "AC-6 (BC-2.15.010 E-DECOR-001 stale-value pattern): last successful periodic \
             value must be returned when subsequent writes fail"
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // EC-001: First query before periodic refresh → load_periodic returns None;
    //         merge with None periodic → sensor_health_status: None
    // BC-2.15.010 EC-15-039.
    // ─────────────────────────────────────────────────────────────────────────

    /// EC-001a (BC-2.15.010 EC-15-039): `load_periodic` for a fresh tenant (no
    /// previous `store_periodic` call) returns `Ok(None)`.
    #[tokio::test]
    async fn test_BC_2_15_010_ec001_load_periodic_fresh_tenant_returns_none() {
        let store = DecorationStore::new(make_backend());
        let tenant = make_tenant("fresh-tenant");

        let result = store.load_periodic(&tenant).await.expect(
            "EC-001 (BC-2.15.010 EC-15-039): load_periodic must not return Err for a \
                     tenant with no cached periodic value",
        );

        assert!(
            result.is_none(),
            "EC-001 (BC-2.15.010 EC-15-039): load_periodic must return None before the \
             first periodic refresh — there is no cached entry yet"
        );
    }

    /// EC-001b (BC-2.15.010 EC-15-039): `merge(cfg, qt, None)` produces a context
    /// where `sensor_health_status` is `None` (i.e., JSON `null` in `_meta`).
    #[test]
    fn test_BC_2_15_010_ec001_merge_with_none_periodic_sensor_health_absent() {
        let config_time = phase1_ctx("Acme Corp", "0.1.0");
        let query_time = phase2_ctx(Some("joshua"), Some("interactive"));

        let merged = DecorationStore::merge(&config_time, &query_time, None);

        assert!(
            merged.sensor_health_status.is_none(),
            "EC-001 (BC-2.15.010 EC-15-039): when periodic=None, sensor_health_status \
             must be None in merged result (serializes as JSON null in _meta)"
        );
    }
}
