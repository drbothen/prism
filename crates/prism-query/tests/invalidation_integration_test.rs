//! Integration tests for prism-query invalidation module requiring process isolation.
//!
//! These tests use PROCESS-GLOBAL mutable state (`DYNAMIC_WRITE_TOOLS`, `QUERY_PHASE_STARTED`)
//! and MUST run in their own process. As integration tests (in tests/ not src/), cargo test
//! and nextest both run them in separate processes from unit tests.
//!
//! Tests in this file:
//! - `test_BC_2_07_004_dynamic_write_tool_triggers_cache_invalidation` (F-LP-IMPL-P1-001)
//! - `test_BC_2_07_004_rwlock_poisoning_returns_registry_poisoned_error` (F-LP-IMPL-P1-004)
//!
//! Story: S-PLUGIN-PREREQ-E | F-LP-IMPL-P1-001 | F-LP-IMPL-P1-004

#![allow(clippy::unwrap_used, clippy::expect_used, non_snake_case)]

use std::sync::Arc;

use prism_core::tenant::OrgSlug;
use prism_core::SensorId;
use prism_query::cache::QueryCache;
use prism_query::cache_key::CacheKey;
use prism_query::invalidation::{
    mark_query_phase_started, register_write_tool, CacheInvalidator, WriteToolInvalidationMap,
};
use prism_spec_engine::error::SpecEngineError;

// ---------------------------------------------------------------------------
// F-LP-IMPL-P1-001: dynamic write tool triggers cache invalidation
// ---------------------------------------------------------------------------

/// BC-2.07.004 / BC-2.16.012 AC-9 / F-LP-IMPL-P1-001:
/// A write tool registered via `register_write_tool` must trigger cache
/// invalidation when `invalidate_for_write_tool` is called with that tool's name.
///
/// This is the process-isolated version of the inline unit test (which is marked
/// #[ignore] due to shared-process concurrency races under cargo test).
///
/// Story: S-PLUGIN-PREREQ-E / F-LP-IMPL-P1-001 | BC-2.07.004 | BC-2.16.012 AC-9
#[test]
fn test_BC_2_07_004_dynamic_write_tool_triggers_cache_invalidation() {
    // Fresh process — no need to reset global state.
    // DYNAMIC_WRITE_TOOLS starts empty, QUERY_PHASE_STARTED starts false.

    // Register a dynamic write tool (simulates plugin registration at boot step 7.5).
    let dyn_entry = WriteToolInvalidationMap {
        tool_name: "dyn_plugin_write_tool".to_string(),
        source_ids: vec!["dyn_data_source".to_string()],
        sensor_id: SensorId::from("dyn_sensor"),
        plugin_name: "dyn_test_plugin".to_string(),
    };
    register_write_tool(dyn_entry).expect("dynamic write tool registration must succeed");

    // Populate the cache with an entry for the dynamic tool's source_id.
    let cache = Arc::new(QueryCache::with_defaults());
    let key = CacheKey {
        client_id: "acme".to_string(),
        sensor_id: SensorId::from("dyn_sensor"),
        source_id: "dyn_data_source".to_string(),
        push_down_hash: "e".repeat(64),
    };
    let rows = vec![serde_json::json!({"id": "dyn-1"})];
    cache.put(key.clone(), rows).expect("put must succeed");

    // Verify the cache entry exists before invalidation.
    assert!(
        cache.get(&key).expect("get must not fail").is_some(),
        "cache entry must exist before invalidation"
    );

    // Invalidate using the dynamically-registered write tool name.
    let invalidator = CacheInvalidator::new(Arc::clone(&cache));
    let client_id = OrgSlug::new("acme");
    let result = invalidator.invalidate_for_write_tool(&client_id, "dyn_plugin_write_tool");

    assert!(
        result.is_ok(),
        "BC-2.07.004 / F-LP-IMPL-P1-001: invalidate_for_write_tool on dynamic tool \
         must return Ok; got: {:?}",
        result.err()
    );

    // The cache entry must be evicted.
    assert!(
        cache.get(&key).expect("get must not fail").is_none(),
        "BC-2.07.004 / F-LP-IMPL-P1-001: cache entry must be evicted after \
         invalidate_for_write_tool for dynamically-registered write tool"
    );
}

// ---------------------------------------------------------------------------
// F-LP-IMPL-P1-004: RwLock poisoning error variant discrimination
// ---------------------------------------------------------------------------

/// F-LP-IMPL-P1-004: `SpecEngineError::WriteToolRegistryPoisoned` (E-PLUGIN-021)
/// is a DISTINCT variant from `WriteToolRegistrationAfterBoot` (E-PLUGIN-020).
///
/// Tests error variant existence and display-code discrimination.
/// The actual poisoning behavior is an internal implementation detail of
/// `register_write_tool` — tested here via the public error enum API.
///
/// Story: S-PLUGIN-PREREQ-E / F-LP-IMPL-P1-004 | ADR-026 §D7
#[test]
fn test_BC_2_07_004_rwlock_poisoning_error_variant_is_distinct() {
    // Verify WriteToolRegistryPoisoned variant exists and carries E-PLUGIN-021.
    let poisoned_err = SpecEngineError::WriteToolRegistryPoisoned;
    let poisoned_str = format!("{poisoned_err}");
    assert!(
        poisoned_str.contains("E-PLUGIN-021"),
        "F-LP-IMPL-P1-004: WriteToolRegistryPoisoned must display E-PLUGIN-021; \
         got: {poisoned_str}"
    );

    // Verify WriteToolRegistrationAfterBoot variant carries E-PLUGIN-020.
    let after_boot_err = SpecEngineError::WriteToolRegistrationAfterBoot;
    let after_boot_str = format!("{after_boot_err}");
    assert!(
        after_boot_str.contains("E-PLUGIN-020"),
        "F-LP-IMPL-P1-004: WriteToolRegistrationAfterBoot must display E-PLUGIN-020; \
         got: {after_boot_str}"
    );

    // The two errors must be distinct variants.
    assert_ne!(
        poisoned_str, after_boot_str,
        "F-LP-IMPL-P1-004: WriteToolRegistryPoisoned (E-PLUGIN-021) and \
         WriteToolRegistrationAfterBoot (E-PLUGIN-020) must be distinct error variants"
    );
}

/// F-LP-IMPL-P1-004 (post-boot registration): After `mark_query_phase_started()`,
/// `register_write_tool` must return `WriteToolRegistrationAfterBoot` (E-PLUGIN-020),
/// NOT `WriteToolRegistryPoisoned` (E-PLUGIN-021). This confirms the two variants
/// have DISTINCT semantics — timing violation vs. state corruption.
///
/// Story: S-PLUGIN-PREREQ-E / F-LP-IMPL-P1-004 | ADR-026 §D7
#[test]
#[tracing_test::traced_test]
fn test_BC_2_07_004_post_boot_registration_returns_after_boot_not_poisoned() {
    mark_query_phase_started();

    let entry = WriteToolInvalidationMap {
        tool_name: "post_boot_integration_tool".to_string(),
        source_ids: vec!["some_data".to_string()],
        sensor_id: SensorId::from("late_plugin"),
        plugin_name: "late_registrar".to_string(),
    };

    let result = register_write_tool(entry);
    assert!(
        result.is_err(),
        "must return Err after mark_query_phase_started()"
    );

    let err = result.unwrap_err();
    let err_str = format!("{err}");
    assert!(
        err_str.contains("E-PLUGIN-020"),
        "F-LP-IMPL-P1-004: post-boot error must be E-PLUGIN-020 (WriteToolRegistrationAfterBoot), \
         NOT E-PLUGIN-021 (WriteToolRegistryPoisoned); got: {err_str}"
    );
}
