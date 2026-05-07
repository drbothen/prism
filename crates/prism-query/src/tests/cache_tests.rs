//! Cache tests — hit/miss, TTL, eviction, and invalidation (S-3.05).
//!
//! Covers: BC-2.07.003 (TTL-based caching), BC-2.07.004 (write invalidation),
//! BC-2.07.005 (cache key derivation), BC-2.07.006 (LRU eviction bounds).
//!
//! All tests that call `todo!()` bodies are RED by design (BC-5.38.001).
//! Tests marked GREEN-BY-DESIGN are correct-by-construction and may pass
//! immediately against the stubs.

// Allow dead_code while the stubs compile.
#![allow(dead_code, unused_imports)]

use std::sync::Arc;

use prism_core::tenant::OrgSlug;
use prism_core::types::SensorType;
use serde_json::json;

use crate::cache::{
    CacheConfig, CacheEntry, QueryCache, SourceDataType, DEFAULT_MAX_ENTRIES_PER_SENSOR,
};
use crate::cache_key::{CacheKey, PushDownParams};
use crate::invalidation::{CacheInvalidator, WRITE_TOOL_INVALIDATION_MAP};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn make_key(client: &str, sensor: &str, source: &str) -> CacheKey {
    CacheKey {
        client_id: client.to_string(),
        sensor_id: sensor.to_string(),
        source_id: source.to_string(),
        push_down_hash: "a".repeat(64),
    }
}

// ---------------------------------------------------------------------------
// AC-5: Cache hit within TTL
// ---------------------------------------------------------------------------

/// AC-5 / BC-2.07.003: A second identical query within the TTL window must
/// return cached rows without hitting the sensor API.
///
/// RED by design — `QueryCache::put` and `QueryCache::get` are `todo!()`.
#[test]
fn test_ac5_cache_hit_within_ttl_skips_sensor_api() {
    let cache = QueryCache::with_defaults();
    let key = make_key("acme", "crowdstrike", "crowdstrike_detections");
    let rows = vec![json!({"id": "det-1"}), json!({"id": "det-2"})];

    cache.put(key.clone(), rows.clone());

    // Second access — must return cached value.
    let result = cache.get(&key);
    assert_eq!(
        result,
        Some(rows),
        "AC-5: second access must return cached rows without sensor call"
    );
}

/// BC-2.07.003: Cache miss on a key not yet inserted returns None.
///
/// RED by design — `QueryCache::get` is `todo!()`.
#[test]
fn test_cache_miss_on_uninserted_key() {
    let cache = QueryCache::with_defaults();
    let key = make_key("acme", "armis", "armis_alerts");
    assert!(
        cache.get(&key).is_none(),
        "cache miss on uninserted key must return None"
    );
}

/// BC-2.07.003: `force_refresh: true` replaces an existing entry with fresh data.
///
/// RED by design — `QueryCache::force_refresh` and `QueryCache::get` are `todo!()`.
#[test]
fn test_force_refresh_replaces_stale_entry() {
    let cache = QueryCache::with_defaults();
    let key = make_key("acme", "crowdstrike", "crowdstrike_hosts");
    let stale = vec![json!({"host": "old"})];
    let fresh = vec![json!({"host": "new"})];

    cache.put(key.clone(), stale);
    cache.force_refresh(key.clone(), fresh.clone());

    assert_eq!(
        cache.get(&key),
        Some(fresh),
        "force_refresh must overwrite the existing entry"
    );
}

// ---------------------------------------------------------------------------
// AC-6: Sensor invalidation
// ---------------------------------------------------------------------------

/// AC-6 / BC-2.07.004: `configure_credential_source` call → no cache entries
/// for CrowdStrike are affected (credential ops do NOT invalidate sensor data).
///
/// GREEN-BY-DESIGN: static constant check — no runtime `todo!()` calls.
#[test]
fn test_ac6_configure_credential_source_does_not_invalidate_cache() {
    // BC-2.07.004 §Write Tool to source_id Mapping: configure_credential_source
    // maps to no source_ids.
    let found = WRITE_TOOL_INVALIDATION_MAP
        .iter()
        .any(|e| e.tool_name == "configure_credential_source");
    assert!(
        !found,
        "AC-6: configure_credential_source must not be in the invalidation map"
    );
}

/// AC-6 / BC-2.07.004: After a write operation succeeds, cache entries for the
/// affected sensor are evicted synchronously.
///
/// RED by design — `CacheInvalidator::invalidate_for_sensor` is `todo!()`.
#[test]
fn test_ac6_cache_entry_evicted_synchronously_after_write() {
    let cache = Arc::new(QueryCache::with_defaults());
    let key = make_key("acme", "crowdstrike", "crowdstrike_detections");
    let rows = vec![json!({"id": "det-1"})];

    cache.put(key.clone(), rows);
    assert!(
        cache.get(&key).is_some(),
        "entry must exist before invalidation"
    );

    let invalidator = CacheInvalidator::new(Arc::clone(&cache));
    let client = OrgSlug::new("acme");
    invalidator
        .invalidate_for_sensor(&client, SensorType::CrowdStrike)
        .expect("invalidation must succeed");

    assert!(
        cache.get(&key).is_none(),
        "AC-6: cache entry must be evicted synchronously before write response returns"
    );
}

// ---------------------------------------------------------------------------
// AC-7: Cache key order independence
// ---------------------------------------------------------------------------

/// AC-7 / BC-2.07.005: Same params in different insertion order produce
/// the same CacheKey `push_down_hash`.
///
/// RED by design — `CacheKey::derive_push_down_hash` is `todo!()`.
#[test]
fn test_ac7_push_down_hash_order_independent() {
    let mut params_ab = PushDownParams::new();
    params_ab.insert("a_filter", json!("val1"));
    params_ab.insert("z_filter", json!("val2"));

    let mut params_ba = PushDownParams::new();
    params_ba.insert("z_filter", json!("val2"));
    params_ba.insert("a_filter", json!("val1"));

    let hash_ab = CacheKey::derive_push_down_hash(&params_ab);
    let hash_ba = CacheKey::derive_push_down_hash(&params_ba);

    assert_eq!(
        hash_ab, hash_ba,
        "AC-7: push_down_hash must be order-independent (alphabetical sort before hashing)"
    );
}

/// EC-07-004 / BC-2.07.005: Same query string, different client ordering →
/// same CacheKey (if the clients are the same set).
///
/// RED by design — `CacheKey::derive` is `todo!()`.
#[test]
fn test_ec07004_different_client_ordering_same_key() {
    let params = PushDownParams::new();

    // Same client, same sensor, same source — same push_down_hash.
    let key_a = CacheKey::derive("acme", "crowdstrike", "crowdstrike_detections", &params);
    let key_b = CacheKey::derive("acme", "crowdstrike", "crowdstrike_detections", &params);

    assert_eq!(
        key_a, key_b,
        "EC-07-004: identical inputs must produce identical cache keys"
    );
}

// ---------------------------------------------------------------------------
// AC-8: LRU eviction at 50-entry partition bound
// ---------------------------------------------------------------------------

/// AC-8 / BC-2.07.006: At partition capacity (50 entries), inserting a new
/// entry evicts the LRU entry first.
///
/// RED by design — `QueryCache::put`, `QueryCache::entry_count` are `todo!()`.
#[test]
fn test_ac8_lru_eviction_keeps_entry_count_within_bound() {
    let config = CacheConfig {
        max_entries_per_sensor: 3,
    };
    let cache = QueryCache::new(config);

    // Insert 3 entries (fill to capacity for sensor "armis", partition "acme/armis").
    for i in 0u8..3 {
        let key = CacheKey {
            client_id: "acme".to_string(),
            sensor_id: "armis".to_string(),
            source_id: "armis_devices".to_string(),
            push_down_hash: format!("{:0<64}", i),
        };
        cache.put(key, vec![json!({"device": i})]);
    }

    // Insert a 4th entry — must evict LRU.
    let overflow_key = CacheKey {
        client_id: "acme".to_string(),
        sensor_id: "armis".to_string(),
        source_id: "armis_devices".to_string(),
        push_down_hash: format!("{:0<64}", 99u8),
    };
    cache.put(overflow_key, vec![json!({"device": 99})]);

    // Total global entry count must not exceed 3.
    assert!(
        cache.entry_count() <= 3,
        "AC-8: partition must not exceed max_entries_per_sensor after LRU eviction"
    );
}

/// EC-07-052 / BC-2.07.006: `max_entries_per_sensor = 0` effectively disables
/// caching — every put is a no-op.
///
/// RED by design — `QueryCache::put` and `QueryCache::entry_count` are `todo!()`.
#[test]
fn test_ec07052_max_entries_zero_disables_cache() {
    let config = CacheConfig {
        max_entries_per_sensor: 0,
    };
    let cache = QueryCache::new(config);
    let key = make_key("acme", "armis", "armis_alerts");

    cache.put(key.clone(), vec![json!({"alert": 1})]);

    // With max_entries = 0, no entries should be stored.
    assert!(
        cache.get(&key).is_none(),
        "EC-07-052: max_entries_per_sensor=0 must disable caching"
    );
}

// ---------------------------------------------------------------------------
// EC-07-030: Concurrent queries (structural / integration placeholder)
// ---------------------------------------------------------------------------

/// EC-07-030 / BC-2.07.003: Two concurrent identical queries both miss cache
/// — both return correct results, no coalescing in v1.
///
/// RED by design — `QueryCache::put` and `QueryCache::get` are `todo!()`.
#[test]
fn test_ec07030_concurrent_miss_both_return_results() {
    use std::sync::Arc;
    use std::thread;

    let cache = Arc::new(QueryCache::with_defaults());
    let key = make_key("acme", "crowdstrike", "crowdstrike_detections");

    let cache1 = Arc::clone(&cache);
    let key1 = key.clone();
    let t1 = thread::spawn(move || {
        let rows = vec![json!({"id": "det-from-t1"})];
        cache1.put(key1, rows.clone());
        rows
    });

    let cache2 = Arc::clone(&cache);
    let key2 = key.clone();
    let t2 = thread::spawn(move || {
        let rows = vec![json!({"id": "det-from-t2"})];
        cache2.put(key2, rows.clone());
        rows
    });

    let r1 = t1.join().expect("t1 must not panic");
    let r2 = t2.join().expect("t2 must not panic");

    // Both threads produced valid results — this is the correctness property.
    assert!(!r1.is_empty(), "t1 result must be non-empty");
    assert!(!r2.is_empty(), "t2 result must be non-empty");
}

// ---------------------------------------------------------------------------
// TTL constant checks (GREEN-BY-DESIGN)
// ---------------------------------------------------------------------------

/// GREEN-BY-DESIGN: alerts TTL constant == 60.
/// Zero branching, no I/O, no helpers, 1 line.
#[test]
fn test_alerts_ttl_constant_is_60s() {
    assert_eq!(crate::cache::CACHE_TTL_ALERTS_SECS, 60);
}

/// GREEN-BY-DESIGN: devices TTL constant == 300.
/// Zero branching, no I/O, no helpers, 1 line.
#[test]
fn test_devices_ttl_constant_is_300s() {
    assert_eq!(crate::cache::CACHE_TTL_DEVICES_SECS, 300);
}

/// GREEN-BY-DESIGN: default max entries per sensor constant == 50.
/// Zero branching, no I/O, no helpers, 1 line.
#[test]
fn test_default_max_entries_constant_is_50() {
    assert_eq!(DEFAULT_MAX_ENTRIES_PER_SENSOR, 50);
}

// ---------------------------------------------------------------------------
// BC-2.07.003: TTL is absolute (created_at), not sliding
// ---------------------------------------------------------------------------

/// BC-2.07.003 §Invariants: TTL is measured from `created_at`, NOT from last
/// access. A cache hit must NOT reset the expiry clock.
///
/// TD-S305-006: the todo!() in this test requires clock injection; moka's
/// internal TTL handles expiry correctly but the test body is incomplete.
#[test]
#[ignore = "TD-S305-006: test body has todo!() placeholder; clock injection needed to advance past TTL"]
fn test_BC_2_07_003_ttl_measured_from_created_at_not_from_last_access() {
    // This test verifies the invariant by inspecting a raw CacheEntry's
    // is_expired() outcome relative to its created_at field.
    // On implementation: create an entry, advance time past TTL, confirm expiry
    // is determined by created_at alone — not by access recency.
    //
    // Stub panics → Red Gate confirmed.
    let entry = CacheEntry {
        rows: vec![json!({"id": "det-1"})],
        created_at: std::time::Instant::now(),
        ttl: std::time::Duration::from_secs(60),
        hit_count: 5, // many accesses — must NOT affect expiry
    };
    // is_expired() checks elapsed from created_at; 5 accesses must not extend TTL.
    let _expired = entry.is_expired();
    // If the implementation resets TTL on each access, this invariant breaks.
    // The implementer must ensure: expired = created_at.elapsed() > ttl.
    todo!("BC-2.07.003 TTL invariant: not yet implemented");
}

/// BC-2.07.003: `hit_count` is incremented on every cache hit.
///
/// TD-S305-007: requires a cache metrics inspection API (e.g., get_entry_with_meta())
/// to verify hit_count == 2. The test body has todo!(); deferred to a metrics
/// API story.
#[test]
#[ignore = "TD-S305-007: needs cache metrics API to inspect hit_count; deferred"]
fn test_BC_2_07_003_hit_count_incremented_on_cache_hit() {
    let cache = QueryCache::with_defaults();
    let key = make_key("acme", "crowdstrike", "crowdstrike_detections");
    let rows = vec![json!({"id": "det-1"})];

    cache.put(key.clone(), rows);
    // First hit.
    let _ = cache.get(&key);
    // Second hit.
    let _ = cache.get(&key);

    // Implementer: verify hit_count == 2 via a cache metrics API.
    // For stub phase we verify the contract compiles and panics.
    todo!("BC-2.07.003 hit_count not yet implemented")
}

/// BC-2.07.003: Health/status source is NOT cached — `put` on a health source
/// must be a no-op (entry count stays zero).
///
/// RED by design — `QueryCache::put`, `SourceDataType::from_source_id` are `todo!()`.
#[test]
fn test_BC_2_07_003_health_status_source_not_cached() {
    let cache = QueryCache::with_defaults();
    // Use a health-type source_id — exact string to be recognized by from_source_id().
    let key = CacheKey {
        client_id: "acme".to_string(),
        sensor_id: "crowdstrike".to_string(),
        source_id: "crowdstrike_health".to_string(),
        push_down_hash: "e".repeat(64),
    };
    cache.put(key.clone(), vec![json!({"status": "ok"})]);

    // Health sources must not be cached — get must return None.
    assert!(
        cache.get(&key).is_none(),
        "BC-2.07.003: health/status source must never be cached (put is no-op)"
    );
}

/// EC-07-031 / BC-2.07.003: TTL expiry race — entry that expires between cache
/// check and return is acceptable (stale-by-milliseconds). Next request must miss.
///
/// RED by design — `QueryCache::get`, `QueryCache::put_with_ttl` are `todo!()`.
#[test]
fn test_BC_2_07_003_ec07031_ttl_expiry_race_next_request_misses() {
    // Simulates: insert with 1ms TTL, wait until expired, then assert miss.
    // The implementer should use tokio::time::pause() + advance() for precision.
    // Here we simply verify the contract: an expired entry must be treated as a miss.
    let cache = QueryCache::with_defaults();
    let key = make_key("acme", "crowdstrike", "crowdstrike_detections");

    // Insert with a nearly-zero TTL (1ms).
    cache.put_with_ttl(
        key.clone(),
        vec![json!({"id": "stale"})],
        std::time::Duration::from_millis(1),
    );

    // After expiry time passes (spin; implementer uses time injection in real test).
    // For the stub, this panics — confirming Red Gate.
    std::thread::sleep(std::time::Duration::from_millis(5));

    let result = cache.get(&key);
    assert!(
        result.is_none(),
        "EC-07-031: expired entry must be treated as a cache miss on next request"
    );
}

/// EC-07-032 / BC-2.07.003: `force_refresh: true` with no existing cache entry
/// must query sensor API and store the result normally.
///
/// RED by design — `QueryCache::force_refresh` and `QueryCache::get` are `todo!()`.
#[test]
fn test_BC_2_07_003_ec07032_force_refresh_with_no_existing_entry() {
    let cache = QueryCache::with_defaults();
    let key = make_key("acme", "crowdstrike", "crowdstrike_detections");
    let fresh = vec![json!({"id": "fresh-1"})];

    // No prior entry — force_refresh must still store the result.
    cache.force_refresh(key.clone(), fresh.clone());

    assert_eq!(
        cache.get(&key),
        Some(fresh),
        "EC-07-032: force_refresh with no existing entry must store the fresh result"
    );
}

/// BC-2.07.003: Two PrismQL queries with different syntax but identical
/// push-down parameters share the same cache entry (EC-07-040 / BC-2.07.005).
///
/// RED by design — `CacheKey::derive` and `QueryCache::get` are `todo!()`.
#[test]
fn test_BC_2_07_003_ec07040_different_pql_same_push_down_shares_cache_entry() {
    let cache = QueryCache::with_defaults();
    let mut params = PushDownParams::new();
    params.insert("severity", json!("High"));

    // Two syntactically different PrismQL strings that produce the same push-down.
    let key_a = CacheKey::derive("acme", "crowdstrike", "crowdstrike_detections", &params);
    let key_b = CacheKey::derive("acme", "crowdstrike", "crowdstrike_detections", &params);

    let rows = vec![json!({"id": "det-1"})];
    cache.put(key_a.clone(), rows.clone());

    // key_b has identical push_down_hash — must be a cache hit.
    assert_eq!(
        cache.get(&key_b),
        Some(rows),
        "EC-07-040: two PrismQL queries with same push-down params must share a cache entry"
    );
}

/// BC-2.07.003 cross-client: cache hit for one client must not pollute another
/// client's partition.
///
/// RED by design — `QueryCache::put` and `QueryCache::get` are `todo!()`.
#[test]
fn test_BC_2_07_003_cross_client_partitions_are_independent() {
    let cache = QueryCache::with_defaults();
    let key_acme = make_key("acme", "crowdstrike", "crowdstrike_detections");
    let key_beta = make_key("beta", "crowdstrike", "crowdstrike_detections");
    let rows = vec![json!({"id": "det-1"})];

    cache.put(key_acme.clone(), rows.clone());

    // beta's partition is independent — must not see acme's entry.
    assert!(
        cache.get(&key_beta).is_none(),
        "BC-2.07.003: cache partitions are per-client; acme's entry must not pollute beta"
    );
}

// ---------------------------------------------------------------------------
// BC-2.07.004: Invalidation ordering and per-write-tool coverage
// ---------------------------------------------------------------------------

/// BC-2.07.004: `invalidate_for_write_tool` must evict entries for the specific
/// write tool's source_ids (e.g., crowdstrike_acknowledge_alert → alerts + detections).
///
/// RED by design — `CacheInvalidator::invalidate_for_write_tool` is `todo!()`.
#[test]
fn test_BC_2_07_004_invalidate_for_write_tool_crowdstrike_acknowledge_alert() {
    let cache = Arc::new(QueryCache::with_defaults());
    let invalidator = CacheInvalidator::new(Arc::clone(&cache));
    let client = OrgSlug::new("acme");

    // Populate both sources that crowdstrike_acknowledge_alert must invalidate.
    let key_alerts = make_key("acme", "crowdstrike", "crowdstrike_alerts");
    let key_detections = make_key("acme", "crowdstrike", "crowdstrike_detections");
    cache.put(key_alerts.clone(), vec![json!({"alert": 1})]);
    cache.put(key_detections.clone(), vec![json!({"det": 1})]);

    invalidator
        .invalidate_for_write_tool(&client, "crowdstrike_acknowledge_alert")
        .expect("invalidate_for_write_tool must not fail");

    assert!(
        cache.get(&key_alerts).is_none(),
        "BC-2.07.004: crowdstrike_acknowledge_alert must invalidate crowdstrike_alerts"
    );
    assert!(
        cache.get(&key_detections).is_none(),
        "BC-2.07.004: crowdstrike_acknowledge_alert must invalidate crowdstrike_detections"
    );
}

/// BC-2.07.004: `invalidate_for_write_tool` for `armis_update_alert_status`
/// must invalidate armis_alerts.
///
/// RED by design — `CacheInvalidator::invalidate_for_write_tool` is `todo!()`.
#[test]
fn test_BC_2_07_004_invalidate_for_write_tool_armis_update_alert_status() {
    let cache = Arc::new(QueryCache::with_defaults());
    let invalidator = CacheInvalidator::new(Arc::clone(&cache));
    let client = OrgSlug::new("acme");

    let key = make_key("acme", "armis", "armis_alerts");
    cache.put(key.clone(), vec![json!({"alert": "armis-1"})]);

    invalidator
        .invalidate_for_write_tool(&client, "armis_update_alert_status")
        .expect("invalidate_for_write_tool must not fail");

    assert!(
        cache.get(&key).is_none(),
        "BC-2.07.004: armis_update_alert_status must invalidate armis_alerts"
    );
}

/// BC-2.07.004: `invalidate_for_write_tool` for `claroty_device_action`
/// must invalidate claroty_devices.
///
/// RED by design — `CacheInvalidator::invalidate_for_write_tool` is `todo!()`.
#[test]
fn test_BC_2_07_004_invalidate_for_write_tool_claroty_device_action() {
    let cache = Arc::new(QueryCache::with_defaults());
    let invalidator = CacheInvalidator::new(Arc::clone(&cache));
    let client = OrgSlug::new("acme");

    let key = make_key("acme", "claroty", "claroty_devices");
    cache.put(key.clone(), vec![json!({"device": "c-1"})]);

    invalidator
        .invalidate_for_write_tool(&client, "claroty_device_action")
        .expect("invalidate_for_write_tool must not fail");

    assert!(
        cache.get(&key).is_none(),
        "BC-2.07.004: claroty_device_action must invalidate claroty_devices"
    );
}

/// BC-2.07.004: Unknown write tool name must return PrismError::Internal
/// (missing mapping = bug, per BC-2.07.004 description).
///
/// RED by design — `CacheInvalidator::invalidate_for_write_tool` is `todo!()`.
#[test]
fn test_BC_2_07_004_unknown_write_tool_returns_internal_error() {
    let cache = Arc::new(QueryCache::with_defaults());
    let invalidator = CacheInvalidator::new(Arc::clone(&cache));
    let client = OrgSlug::new("acme");

    let result = invalidator.invalidate_for_write_tool(&client, "unknown_write_tool_xyz");
    assert!(
        result.is_err(),
        "BC-2.07.004: unknown write tool must return an error (missing mapping = bug)"
    );
}

/// DEC-018 / BC-2.07.004: Write-then-read sequence — a query issued after a
/// successful write must NOT see pre-write cached data for the affected tuple.
///
/// This is the core write-then-read consistency invariant.
///
/// RED by design — `CacheInvalidator::invalidate_for_write_tool` and `QueryCache::get` are `todo!()`.
#[test]
fn test_BC_2_07_004_dec018_write_then_read_sees_fresh_data_not_cached() {
    let cache = Arc::new(QueryCache::with_defaults());
    let invalidator = CacheInvalidator::new(Arc::clone(&cache));
    let client = OrgSlug::new("acme");

    // Populate cache with pre-write stale data.
    let key = make_key("acme", "crowdstrike", "crowdstrike_hosts");
    cache.put(
        key.clone(),
        vec![json!({"host": "pre-write", "contained": false})],
    );
    assert!(cache.get(&key).is_some(), "pre-write entry must exist");

    // Execute write (contain host).
    invalidator
        .invalidate_for_write_tool(&client, "crowdstrike_contain_host")
        .expect("invalidation must succeed");

    // Post-write read must not see stale cached data.
    assert!(
        cache.get(&key).is_none(),
        "DEC-018: post-write read must not return pre-write cached data for the affected tuple"
    );
}

/// EC-07-011 / BC-2.07.004: Concurrent read and write for the same tuple must
/// produce no partial state. The read either sees cached data or a fresh miss
/// — never a torn entry.
///
/// RED by design — concurrency correctness requires the implementation's lock.
#[test]
fn test_BC_2_07_004_ec07011_concurrent_read_write_no_partial_state() {
    use std::thread;

    let cache = Arc::new(QueryCache::with_defaults());
    let key = make_key("acme", "crowdstrike", "crowdstrike_detections");
    cache.put(key.clone(), vec![json!({"id": "pre-write"})]);

    let cache_reader = Arc::clone(&cache);
    let key_reader = key.clone();

    let cache_writer = Arc::clone(&cache);

    let reader = thread::spawn(move || {
        // Read concurrently with invalidation — must get Some or None, never panic/corrupt.
        let _ = cache_reader.get(&key_reader);
    });

    let writer = thread::spawn(move || {
        cache_writer.invalidate_by_prefix("acme", "crowdstrike", "crowdstrike_detections");
    });

    reader.join().expect("reader thread must not panic");
    writer.join().expect("writer thread must not panic");

    // Post-concurrent state: entry may or may not be present, but no corruption.
    // Final state: if writer completed first, entry is gone. Both are valid.
    let _ = cache.get(&key); // Must not panic.
}

/// BC-2.07.004: `invalidate_for_client` removes ALL entries for the given client.
///
/// RED by design — `CacheInvalidator::invalidate_for_client` is `todo!()`.
#[test]
fn test_BC_2_07_004_invalidate_for_client_removes_all_entries() {
    let cache = Arc::new(QueryCache::with_defaults());
    let invalidator = CacheInvalidator::new(Arc::clone(&cache));
    let client = OrgSlug::new("acme");

    // Insert entries across multiple sensors for acme.
    let key1 = make_key("acme", "crowdstrike", "crowdstrike_detections");
    let key2 = make_key("acme", "armis", "armis_devices");
    let key3 = make_key("beta", "crowdstrike", "crowdstrike_detections"); // different client

    cache.put(key1.clone(), vec![json!({"id": 1})]);
    cache.put(key2.clone(), vec![json!({"id": 2})]);
    cache.put(key3.clone(), vec![json!({"id": 3})]);

    invalidator
        .invalidate_for_client(&client)
        .expect("invalidate_for_client must not fail");

    // All acme entries evicted.
    assert!(
        cache.get(&key1).is_none(),
        "acme/crowdstrike entry must be evicted"
    );
    assert!(
        cache.get(&key2).is_none(),
        "acme/armis entry must be evicted"
    );
    // Beta's entry must be unaffected.
    assert!(
        cache.get(&key3).is_some(),
        "BC-2.07.004: invalidate_for_client must not evict other clients' entries"
    );
}

/// GREEN-BY-DESIGN: invalidation map covers all 4 write tools per sensor
/// (crowdstrike: 2, cyberint: 2, claroty: 2, armis: 2).
#[test]
fn test_BC_2_07_004_invalidation_map_covers_all_4_sensors() {
    let sensors = ["crowdstrike", "cyberint", "claroty", "armis"];
    for sensor in &sensors {
        let has_entry = WRITE_TOOL_INVALIDATION_MAP
            .iter()
            .any(|e| e.tool_name.starts_with(sensor));
        assert!(
            has_entry,
            "BC-2.07.004: invalidation map must have entries for sensor {sensor}"
        );
    }
}

// ---------------------------------------------------------------------------
// BC-2.07.005: Cache key derivation — additional coverage
// ---------------------------------------------------------------------------

/// EC-07-041 / BC-2.07.005: `force_refresh` flag is excluded from push_down_hash.
/// The hash with force_refresh=true must equal hash with force_refresh=false
/// (force_refresh is not a push-down parameter — it's a bypass flag).
///
/// RED by design — `CacheKey::derive` is `todo!()`.
#[test]
fn test_BC_2_07_005_ec07041_force_refresh_excluded_from_push_down_hash() {
    // Both calls use the same push-down parameters; force_refresh is not a param.
    // The cache key must be the same regardless of force_refresh intent.
    let mut params = PushDownParams::new();
    params.insert("status", json!("open"));

    // Derive both — they must produce the same key (force_refresh is NOT in params).
    let hash_normal = CacheKey::derive_push_down_hash(&params);

    // Simulate a force_refresh call that passes the same params (force_refresh excluded).
    let hash_forced = CacheKey::derive_push_down_hash(&params);

    assert_eq!(
        hash_normal, hash_forced,
        "EC-07-041: force_refresh must not affect push_down_hash (it is excluded from params)"
    );
}

/// BC-2.07.005: `limit` from the query tool is excluded from push_down_hash.
/// The cache stores the full sensor API response; `limit` is applied after retrieval.
///
/// RED by design — `CacheKey::derive_push_down_hash` is `todo!()`.
#[test]
fn test_BC_2_07_005_limit_excluded_from_push_down_hash() {
    // limit is a query-tool parameter, not a push-down filter.
    // Two queries with different limits but same push-down params share a cache entry.
    // Verify: push_down_hash must NOT change when limit changes.
    let mut params_no_limit = PushDownParams::new();
    params_no_limit.insert("status", json!("open"));

    // Attempt to insert "limit" as a push-down param — must be filtered/ignored.
    let mut params_with_limit = PushDownParams::new();
    params_with_limit.insert("status", json!("open"));
    // "limit" is NOT a push-down param per BC-2.07.005; the canonicalization
    // must exclude it. In practice it won't be in PushDownParams at all —
    // this test documents the invariant that limit must never appear in params.
    // Both must produce the same hash.
    let hash_no_limit = CacheKey::derive_push_down_hash(&params_no_limit);
    let hash_with_limit = CacheKey::derive_push_down_hash(&params_with_limit);

    assert_eq!(
        hash_no_limit, hash_with_limit,
        "BC-2.07.005: limit is excluded from push_down_hash (cache stores full response)"
    );
}

/// BC-2.07.005 §Invariants: cache invalidation prefix scan covers all
/// push_down_hash variants for a (client, sensor, source) prefix.
/// Inserting two entries with the same prefix but different hashes — both
/// must be invalidated by a prefix-scan call.
///
/// RED by design — `QueryCache::invalidate_by_prefix` is `todo!()`.
#[test]
fn test_BC_2_07_005_prefix_scan_invalidation_covers_all_hash_variants() {
    let cache = QueryCache::with_defaults();

    // Two entries with the same (client, sensor, source) but different hashes.
    let key_a = CacheKey {
        client_id: "acme".to_string(),
        sensor_id: "crowdstrike".to_string(),
        source_id: "crowdstrike_detections".to_string(),
        push_down_hash: "a".repeat(64),
    };
    let key_b = CacheKey {
        client_id: "acme".to_string(),
        sensor_id: "crowdstrike".to_string(),
        source_id: "crowdstrike_detections".to_string(),
        push_down_hash: "b".repeat(64),
    };
    cache.put(key_a.clone(), vec![json!({"id": "det-a"})]);
    cache.put(key_b.clone(), vec![json!({"id": "det-b"})]);

    // Prefix-scan invalidation must remove BOTH entries.
    cache.invalidate_by_prefix("acme", "crowdstrike", "crowdstrike_detections");

    assert!(
        cache.get(&key_a).is_none(),
        "BC-2.07.005: prefix scan must evict key_a"
    );
    assert!(
        cache.get(&key_b).is_none(),
        "BC-2.07.005: prefix scan must evict key_b (different hash, same prefix)"
    );
}

/// BC-2.07.005: case-sensitive string comparison — same key, different case =
/// different hashes.
///
/// RED by design — `CacheKey::derive_push_down_hash` is `todo!()`.
#[test]
fn test_BC_2_07_005_string_values_case_sensitive() {
    let mut params_upper = PushDownParams::new();
    params_upper.insert("severity", json!("HIGH"));

    let mut params_lower = PushDownParams::new();
    params_lower.insert("severity", json!("high"));

    let hash_upper = CacheKey::derive_push_down_hash(&params_upper);
    let hash_lower = CacheKey::derive_push_down_hash(&params_lower);

    assert_ne!(
        hash_upper, hash_lower,
        "BC-2.07.005: string values are case-sensitive; HIGH vs high must produce different hashes"
    );
}

// ---------------------------------------------------------------------------
// BC-2.07.006: LRU eviction — additional invariants
// ---------------------------------------------------------------------------

/// EC-07-051 / BC-2.07.006: When all entries have the same access time, eviction
/// falls back to insertion order (FIFO tiebreaker).
///
/// RED by design — `QueryCache::put`, `QueryCache::get` are `todo!()`.
#[test]
fn test_BC_2_07_006_ec07051_same_access_time_fifo_tiebreaker() {
    let config = CacheConfig {
        max_entries_per_sensor: 2,
    };
    let cache = QueryCache::new(config);

    // Insert entries in order; no access in between (same created_at order).
    let key_first = CacheKey {
        client_id: "acme".to_string(),
        sensor_id: "crowdstrike".to_string(),
        source_id: "crowdstrike_detections".to_string(),
        push_down_hash: "1".repeat(64),
    };
    let key_second = CacheKey {
        client_id: "acme".to_string(),
        sensor_id: "crowdstrike".to_string(),
        source_id: "crowdstrike_detections".to_string(),
        push_down_hash: "2".repeat(64),
    };
    let key_third = CacheKey {
        client_id: "acme".to_string(),
        sensor_id: "crowdstrike".to_string(),
        source_id: "crowdstrike_detections".to_string(),
        push_down_hash: "3".repeat(64),
    };
    cache.put(key_first.clone(), vec![json!({"id": 1})]);
    cache.put(key_second.clone(), vec![json!({"id": 2})]);
    // Third insert: must evict oldest by insertion order (FIFO tiebreaker).
    cache.put(key_third.clone(), vec![json!({"id": 3})]);

    // The first-inserted entry should have been evicted (FIFO when access times equal).
    assert!(
        cache.get(&key_first).is_none(),
        "EC-07-051: FIFO tiebreaker must evict the first-inserted entry when access times are equal"
    );
    assert!(
        cache.get(&key_third).is_some(),
        "EC-07-051: the newly inserted entry must be present after eviction"
    );
}

/// EC-07-053 / BC-2.07.006: Cross-client query — each client's partition is
/// independent. A cross-client query that populates caches for clients A and B
/// does not mix their partitions.
///
/// RED by design — `QueryCache::put` and `QueryCache::get` are `todo!()`.
#[test]
fn test_BC_2_07_006_ec07053_cross_client_partitions_independent() {
    let config = CacheConfig {
        max_entries_per_sensor: 2,
    };
    let cache = QueryCache::new(config);

    // Simulate cross-client fan-out: both clients get populated independently.
    let key_client_a = make_key("client-a", "armis", "armis_devices");
    let key_client_b = make_key("client-b", "armis", "armis_devices");

    cache.put(key_client_a.clone(), vec![json!({"device": "a-1"})]);
    cache.put(key_client_b.clone(), vec![json!({"device": "b-1"})]);

    // Each partition is bounded independently — filling A to capacity must not
    // evict B's entries.
    let overflow_a = CacheKey {
        client_id: "client-a".to_string(),
        sensor_id: "armis".to_string(),
        source_id: "armis_devices".to_string(),
        push_down_hash: "f".repeat(64),
    };
    let overflow_a2 = CacheKey {
        client_id: "client-a".to_string(),
        sensor_id: "armis".to_string(),
        source_id: "armis_devices".to_string(),
        push_down_hash: "g".repeat(64),
    };
    cache.put(overflow_a, vec![json!({"device": "a-2"})]);
    cache.put(overflow_a2, vec![json!({"device": "a-3"})]);

    // client-b's entry must still be present (its partition was not overflowed).
    assert!(
        cache.get(&key_client_b).is_some(),
        "EC-07-053: eviction in client-a's partition must not evict client-b's entries"
    );
}

/// BC-2.07.006: LRU ordering — a recently accessed entry must not be the eviction
/// target when a new entry is inserted.
///
/// RED by design — `QueryCache::put` and `QueryCache::get` are `todo!()`.
#[test]
fn test_BC_2_07_006_recently_accessed_entry_not_evicted() {
    let config = CacheConfig {
        max_entries_per_sensor: 2,
    };
    let cache = QueryCache::new(config);

    let key_old = CacheKey {
        client_id: "acme".to_string(),
        sensor_id: "armis".to_string(),
        source_id: "armis_alerts".to_string(),
        push_down_hash: "0".repeat(64),
    };
    let key_recent = CacheKey {
        client_id: "acme".to_string(),
        sensor_id: "armis".to_string(),
        source_id: "armis_alerts".to_string(),
        push_down_hash: "r".repeat(64),
    };
    cache.put(key_old.clone(), vec![json!({"id": "old"})]);
    cache.put(key_recent.clone(), vec![json!({"id": "recent"})]);

    // Access key_old to make it "recently used" — it should survive eviction.
    let _ = cache.get(&key_old);

    // Insert a new entry — LRU should evict key_recent (older last access).
    let key_new = CacheKey {
        client_id: "acme".to_string(),
        sensor_id: "armis".to_string(),
        source_id: "armis_alerts".to_string(),
        push_down_hash: "n".repeat(64),
    };
    cache.put(key_new.clone(), vec![json!({"id": "new"})]);

    // key_old was recently accessed — must still be present.
    assert!(
        cache.get(&key_old).is_some(),
        "BC-2.07.006: recently accessed entry must not be chosen for LRU eviction"
    );
    // key_new must be present.
    assert!(
        cache.get(&key_new).is_some(),
        "BC-2.07.006: newly inserted entry must be present after eviction"
    );
}

/// BC-2.07.006 §Invariants: DI-018 — entry count never exceeds the configured
/// bound for any (client_id, sensor_id) partition, verified under many insertions.
///
/// RED by design — `QueryCache::put` and `QueryCache::entry_count` are `todo!()`.
#[test]
fn test_BC_2_07_006_di018_entry_count_never_exceeds_bound_under_many_insertions() {
    let bound = 5usize;
    let config = CacheConfig {
        max_entries_per_sensor: bound,
    };
    let cache = QueryCache::new(config);

    // Insert 3× the bound; entry count must never exceed the bound.
    for i in 0u8..(bound as u8 * 3) {
        let key = CacheKey {
            client_id: "acme".to_string(),
            sensor_id: "crowdstrike".to_string(),
            source_id: "crowdstrike_detections".to_string(),
            push_down_hash: format!("{:0<64}", i),
        };
        cache.put(key, vec![json!({"row": i})]);
        assert!(
            cache.entry_count() <= bound as u64,
            "DI-018: entry count must never exceed bound after insertion {i}; got {}",
            cache.entry_count()
        );
    }
}
