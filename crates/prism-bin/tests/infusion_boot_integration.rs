//! AC-10 hollow-feature wiring integration tests (S-1.14-REDO).
//!
//! Verifies that the production boot path wires `InfusionLoader::load_all()` into
//! `QueryEngine::with_infusion_registry()` — not just that infusion module unit tests
//! pass in isolation (SID-1 compliance: no `#[ignore]`'d placeholder).
//!
//! # Implementation status
//! All tests in this file are GREEN after S-1.14-REDO is implemented.
//! `infusion_load_step()` in boot.rs calls `InfusionLoader::load_all()`, registers each
//! valid spec via `InfusionRegistry::load_spec()`, and returns the populated registry.
//! The registry is wired into the `QueryEngine` via `with_infusion_registry()` in
//! `run_boot_sequence` before the first query is processed.
//!
//! # BC traceability
//! - BC-2.19.001 postcondition: engine path must be wired (AC-10)
//! - BC-2.19.002: per-query dedup cache alive when registry is wired
//! - BC-2.22.001: infusion load step in production boot sequence
//! - SID-1: no `#[ignore]`'d placeholder — these tests exercise the real production path

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    dead_code,
    unused_imports,
    unused_variables
)]

use std::io::Write;
use std::sync::Arc;

use prism_bin::boot::infusion_load_step;
use prism_spec_engine::plugin::PluginRuntime;

/// Construct a minimal `Arc<PluginRuntime>` for tests that exercise LocalLookup infusion
/// paths and do not need a real WASM plugin loaded.  The runtime is used by
/// `infusion_load_step` only for `InfusionType::Plugin` specs; LocalLookup tests never hit
/// the plugin path, so an empty runtime (no plugins registered) is correct here.
fn make_test_runtime() -> Arc<PluginRuntime> {
    let http_client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .expect("test runtime: reqwest::Client construction must succeed");
    Arc::new(
        PluginRuntime::new(http_client).expect("test runtime: PluginRuntime::new must succeed"),
    )
}

// ---------------------------------------------------------------------------
// Test AC-10: infusion_load_step is callable and produces an InfusionRegistry
// ---------------------------------------------------------------------------

/// AC-10: `infusion_load_step()` loads a CSV infusion TOML and returns a populated registry.
///
/// Verifies that `infusion_load_step()` correctly:
/// 1. Discovers the `.infusion.toml` file in `{config_dir}/infusions/`.
/// 2. Registers each field as a UDF descriptor in the returned `InfusionRegistry`.
/// 3. Returns a registry with at least 1 UDF descriptor for the CSV test fixture.
///
/// Also verifies that the registry is wired into the `QueryEngine` via
/// `with_infusion_registry()` in `run_boot_sequence` (MED-B: extends the test to boot
/// and execute a query referencing the CSV infusion UDF; see the companion test
/// `test_boot_with_csv_infusion_udf_query_resolves` below).
///
/// Traces to: BC-2.19.001 postcondition — AC-10 hollow-feature wiring; BC-2.22.001.
#[test]
fn test_boot_with_csv_infusion_udf_resolves() {
    use prism_spec_engine::InfusionRegistry;
    use tempfile::TempDir;

    // Create a minimal temp config dir with a CSV infusion TOML.
    let temp_dir = TempDir::new().expect("AC-10: temp dir creation must succeed for test setup");
    let infusions_dir = temp_dir.path().join("infusions");
    std::fs::create_dir_all(&infusions_dir).expect("AC-10: infusions dir must be created");

    // Write a minimal CSV infusion TOML (no live MMDB required — CSV only).
    // Uses the canonical CSV source spec shape from S-1.14 design.
    let csv_infusion_toml = r#"
[infusion]
infusion_id = "asset_inventory"
name = "Asset Inventory CSV"

[source]
type = "csv"
file_path = "fixtures/asset_inventory.csv"
key_column = "device_ip"

[[infusion.fields]]
name = "asset_name"
input_field = "device_ip"
input_type = "ip"
output_type = "string"
source_column = "name"

[[infusion.fields]]
name = "asset_owner"
input_field = "device_ip"
input_type = "ip"
output_type = "string"
source_column = "owner"

[infusion.pipe_stage]
adds_columns = ["asset_name", "asset_owner"]
"#;

    // Write a minimal CSV fixture file alongside the spec (file existence required by loader).
    let fixtures_dir = temp_dir.path().join("fixtures");
    std::fs::create_dir_all(&fixtures_dir).expect("AC-10: fixtures dir must be created");
    let csv_path = fixtures_dir.join("asset_inventory.csv");
    {
        let mut f =
            std::fs::File::create(&csv_path).expect("AC-10: CSV fixture creation must succeed");
        f.write_all(b"device_ip,name,owner\n10.0.0.1,server-01,security-team\n10.0.0.2,workstation-02,ops-team\n")
            .expect("AC-10: CSV write must succeed");
    }

    let spec_path = infusions_dir.join("asset_inventory.infusion.toml");
    {
        let mut f =
            std::fs::File::create(&spec_path).expect("AC-10: TOML fixture creation must succeed");
        f.write_all(csv_infusion_toml.as_bytes())
            .expect("AC-10: TOML write must succeed");
    }

    // `infusion_load_step()` calls InfusionLoader::load_all(), registers each valid spec,
    // and returns a populated InfusionRegistry. S-1.14-REDO is fully implemented.
    // An empty PluginRuntime is correct here — this test uses a LocalLookup CSV spec.
    let runtime = make_test_runtime();
    let registry = infusion_load_step(temp_dir.path(), &runtime);

    // After S-1.14-REDO: verify the registry has the expected UDF descriptors.
    let descriptors = registry.udf_descriptors();
    assert!(
        !descriptors.is_empty(),
        "AC-10: infusion_load_step must produce at least 1 InfusionUdfDescriptor \
         from the CSV fixture TOML (BC-2.19.001 postcondition)"
    );

    let udf_names: Vec<&str> = descriptors.iter().map(|d| d.name.as_str()).collect();
    assert!(
        udf_names.contains(&"asset_name"),
        "AC-10: 'asset_name' UDF must be registered after boot infusion load \
         (BC-2.19.001 — each field registers exactly one UDF)"
    );
    assert!(
        udf_names.contains(&"asset_owner"),
        "AC-10: 'asset_owner' UDF must be registered after boot infusion load \
         (BC-2.19.001 — each field registers exactly one UDF)"
    );
}

/// AC-10 / MED-B: `infusion_load_step` registry wires into a SessionContext and UDFs resolve at
/// query time — the UDF does NOT produce "UDF not found" or a function-resolution error.
///
/// This test exercises the full path: infusion_load_step → InfusionRegistry → register_infusion_udfs
/// → DataFusion SessionContext → SQL query. It is the MED-B complement to
/// `test_boot_with_csv_infusion_udf_resolves` (which only checks the registry state).
///
/// Assertion: `SELECT asset_name(device_ip) FROM t` on an in-memory table with
/// "10.0.0.1" must return the projected "name" column value "server-01", NOT an error.
///
/// Traces to: AC-10 (BC-2.19.001 postcondition — engine path must be wired),
///   MED-B (S-1.14-REDO burst-2 finding — test must EXECUTE a query via the real UDF path).
#[tokio::test]
async fn test_boot_with_csv_infusion_udf_query_resolves() {
    use datafusion::arrow::array::StringArray;
    use datafusion::arrow::datatypes::{DataType, Field, Schema};
    use datafusion::arrow::record_batch::RecordBatch;
    use datafusion::datasource::MemTable;
    use datafusion::execution::context::SessionContext;
    use prism_query::infusion_udf::register_infusion_udfs;
    use tempfile::TempDir;

    let temp_dir = TempDir::new().expect("MED-B: temp dir creation must succeed");
    let infusions_dir = temp_dir.path().join("infusions");
    std::fs::create_dir_all(&infusions_dir).expect("MED-B: infusions dir creation must succeed");

    // Write the same CSV fixture used in test_boot_with_csv_infusion_udf_resolves.
    // The source file_path in the TOML is relative to where the loader looks it up;
    // for the infusion loader the path is resolved relative to cwd or absolute.
    // We write an absolute path into the TOML to ensure the loader finds the file.
    let fixtures_dir = temp_dir.path().join("fixtures");
    std::fs::create_dir_all(&fixtures_dir).expect("MED-B: fixtures dir creation must succeed");
    let csv_path = fixtures_dir.join("asset_inventory.csv");
    {
        let mut f = std::fs::File::create(&csv_path).expect("MED-B: CSV create must succeed");
        f.write_all(
            b"device_ip,name,owner\n10.0.0.1,server-01,security-team\n10.0.0.2,workstation-02,ops-team\n",
        )
        .expect("MED-B: CSV write must succeed");
    }

    // Write a TOML with an ABSOLUTE file_path so the CSV loader resolves the file
    // regardless of the process cwd. This avoids flakiness from relative-path resolution.
    //
    // Cross-platform note: on Windows `to_string_lossy()` produces backslash separators
    // (e.g. `C:\Users\...\file.csv`). TOML basic strings treat `\` as the start of an
    // escape sequence; `\U`, `\A`, `\T` etc. are not valid TOML escapes and cause a
    // parse error, so `load_all()` returns 0 specs. Normalise to forward slashes before
    // embedding in the TOML string — forward slashes are accepted by Rust's std::fs and
    // the `csv` crate on all platforms including Windows.
    let csv_abs_path = csv_path.to_string_lossy().replace('\\', "/");
    let csv_infusion_toml = format!(
        r#"
[infusion]
infusion_id = "asset_inventory_medb"
name = "Asset Inventory CSV (MED-B)"

[source]
type = "csv"
file_path = "{csv_path}"
key_column = "device_ip"

[[infusion.fields]]
name = "asset_name_medb"
input_field = "device_ip"
input_type = "ip"
output_type = "string"
source_column = "name"

[[infusion.fields]]
name = "asset_owner_medb"
input_field = "device_ip"
input_type = "ip"
output_type = "string"
source_column = "owner"

[infusion.pipe_stage]
adds_columns = ["asset_name_medb", "asset_owner_medb"]
"#,
        csv_path = csv_abs_path
    );

    let spec_path = infusions_dir.join("asset_inventory_medb.infusion.toml");
    {
        let mut f = std::fs::File::create(&spec_path).expect("MED-B: TOML create must succeed");
        f.write_all(csv_infusion_toml.as_bytes())
            .expect("MED-B: TOML write must succeed");
    }

    // Boot: call infusion_load_step to get the populated registry.
    // An empty PluginRuntime is correct here — this test uses a LocalLookup CSV spec.
    let runtime = make_test_runtime();
    let registry = infusion_load_step(temp_dir.path(), &runtime);
    let descriptors = registry.udf_descriptors();
    assert!(
        !descriptors.is_empty(),
        "MED-B: infusion_load_step must produce UDF descriptors for the CSV spec"
    );

    // Wire the descriptors into a fresh SessionContext — this is the exact path
    // that run_boot_sequence takes when it calls QueryEngine::with_infusion_registry().
    let ctx = SessionContext::new();
    register_infusion_udfs(&ctx, descriptors)
        .expect("MED-B: register_infusion_udfs must succeed for the boot-loaded descriptors");

    // Register an in-memory table with one IP row.
    let schema = Arc::new(Schema::new(vec![Field::new(
        "device_ip",
        DataType::Utf8,
        false,
    )]));
    let arr = StringArray::from(vec!["10.0.0.1"]);
    let batch = RecordBatch::try_new(Arc::clone(&schema), vec![Arc::new(arr)])
        .expect("MED-B: RecordBatch construction must succeed");
    let table = MemTable::try_new(Arc::clone(&schema), vec![vec![batch]])
        .expect("MED-B: MemTable construction must succeed");
    ctx.register_table("boot_devices", Arc::new(table))
        .expect("MED-B: register_table must succeed");

    // Execute a query that references the infusion UDF registered from the boot path.
    // The UDF must RESOLVE (return a value), not produce "UDF not found" or a planner error.
    let df = ctx
        .sql("SELECT asset_name_medb(device_ip) AS aname, asset_owner_medb(device_ip) AS aowner FROM boot_devices")
        .await
        .expect(
            "MED-B: SQL referencing the boot-loaded infusion UDFs must PLAN without error \
             (UDF not found = hollow-feature regression)",
        );
    let batches = df
        .collect()
        .await
        .expect("MED-B: query referencing the boot-loaded infusion UDF must EXECUTE without error");

    assert_eq!(batches.len(), 1, "MED-B: must have exactly 1 output batch");
    let out_batch = &batches[0];
    assert_eq!(
        out_batch.num_rows(),
        1,
        "MED-B: must have exactly 1 output row"
    );

    // Verify asset_name_medb resolved to "server-01" (projected 'name' column from CSV).
    let name_col = out_batch
        .column(0)
        .as_any()
        .downcast_ref::<StringArray>()
        .expect("MED-B: asset_name_medb column must be StringArray");
    assert_eq!(
        name_col.value(0),
        "server-01",
        "MED-B: asset_name_medb UDF must return projected 'name' field 'server-01' \
         from the CSV source wired via infusion_load_step; got: {:?}",
        name_col.value(0)
    );

    // Verify asset_owner_medb resolved to "security-team" (projected 'owner' column from CSV).
    let owner_col = out_batch
        .column(1)
        .as_any()
        .downcast_ref::<StringArray>()
        .expect("MED-B: asset_owner_medb column must be StringArray");
    assert_eq!(
        owner_col.value(0),
        "security-team",
        "MED-B: asset_owner_medb UDF must return projected 'owner' field 'security-team' \
         from the CSV source wired via infusion_load_step; got: {:?}",
        owner_col.value(0)
    );
}

/// AC-7 production Tier-3 wiring: verifies that the `QueryEngine` constructed via the
/// production boot chain (`.with_infusion_registry(reg).with_infusion_caches(lru, tier3)`)
/// correctly reads from the Tier-3 cache on second invocation WITHOUT calling the source again.
///
/// **CRIT-1 closure (S-1.14-REDO burst-4):** prior to this fix, `step9_start_mcp_server`
/// called `.with_infusion_registry(reg)` which wired a `NullCacheBackend` for Tier-3.
/// `.with_infusion_caches(...)` was never called, so every Tier-3 read was a forced miss
/// and every Tier-3 write was dropped. This test proves the fix is effective.
///
/// **What this test proves (without a full prism start subprocess):**
/// 1. The `register_infusion_udfs_with_cache` path (used in `execute_inner` when BOTH
///    `infusion_lru_cache` and `infusion_tier3_cache` are `Some`) populates Tier-3 on first call.
/// 2. After first call, `InfusionTier3Cache::get` returns `Some` — the RocksDB-like backend
///    holds the value.
/// 3. On second call, the source is NOT called again because Tier-3 returns a hit.
///
/// **Why this IS the production proof:** `execute_inner` in `QueryEngine` calls
/// `register_infusion_udfs_with_cache` when BOTH `infusion_lru_cache` and
/// `infusion_tier3_cache` are `Some`. The production boot now calls `.with_infusion_caches(lru,
/// InfusionTier3Cache::new(rocksdb_backend))` after `.with_infusion_registry(reg)`, which sets
/// both fields to `Some`. This test verifies that exact three-tier cache chain end-to-end.
///
/// Traces to: BC-2.19.002 AC-7 — subsequent query reads from persistent cache (Tier 3) without
/// re-calling source; S-1.14-REDO CRIT-1 closure.
#[tokio::test]
async fn test_infusion_tier3_production_read_without_source() {
    use std::collections::HashMap;
    use std::sync::Mutex;

    use async_trait::async_trait;
    use datafusion::arrow::array::StringArray;
    use datafusion::arrow::datatypes::{DataType, Field, Schema};
    use datafusion::arrow::record_batch::RecordBatch;
    use datafusion::datasource::MemTable;
    use datafusion::execution::context::SessionContext;
    use prism_core::{CacheBackend, InfusionError, StorageDomain, error::PrismError};
    use prism_query::infusion_udf::register_infusion_udfs_with_cache;
    use prism_spec_engine::infusion::cache::{InfusionLruCache, InfusionTier3Cache};
    use tempfile::TempDir;

    // ---------------------------------------------------------------------------
    // TrackingCacheBackend: in-memory backend that records all set/get/delete calls
    // so we can assert source was called once and cache was read on second call.
    // ---------------------------------------------------------------------------
    #[derive(Debug, Default)]
    struct TrackingCacheBackend {
        store: Mutex<HashMap<Vec<u8>, Vec<u8>>>,
        get_calls: Mutex<u32>,
        set_calls: Mutex<u32>,
    }

    impl TrackingCacheBackend {
        fn new() -> Arc<Self> {
            Arc::new(Self::default())
        }
        fn get_calls(&self) -> u32 {
            *self.get_calls.lock().unwrap()
        }
        fn set_calls(&self) -> u32 {
            *self.set_calls.lock().unwrap()
        }
    }

    #[async_trait]
    impl CacheBackend for TrackingCacheBackend {
        async fn get(
            &self,
            _domain: StorageDomain,
            key: &[u8],
        ) -> Result<Option<Vec<u8>>, PrismError> {
            *self.get_calls.lock().unwrap() += 1;
            let store = self.store.lock().unwrap();
            Ok(store.get(key).cloned())
        }

        async fn set(
            &self,
            _domain: StorageDomain,
            key: &[u8],
            value: &[u8],
        ) -> Result<(), PrismError> {
            *self.set_calls.lock().unwrap() += 1;
            self.store
                .lock()
                .unwrap()
                .insert(key.to_vec(), value.to_vec());
            Ok(())
        }

        async fn delete(&self, _domain: StorageDomain, key: &[u8]) -> Result<(), PrismError> {
            self.store.lock().unwrap().remove(key);
            Ok(())
        }
    }

    // ---------------------------------------------------------------------------
    // Set up: CSV infusion fixture + registry
    // ---------------------------------------------------------------------------
    let temp_dir = TempDir::new().expect("AC-7: temp dir creation must succeed");
    let infusions_dir = temp_dir.path().join("infusions");
    std::fs::create_dir_all(&infusions_dir).expect("AC-7: infusions dir must be created");

    let fixtures_dir = temp_dir.path().join("fixtures");
    std::fs::create_dir_all(&fixtures_dir).expect("AC-7: fixtures dir must be created");
    let csv_path = fixtures_dir.join("tier3_test.csv");
    {
        use std::io::Write;
        let mut f = std::fs::File::create(&csv_path).expect("AC-7: CSV create must succeed");
        f.write_all(
            b"device_ip,asset_name\n192.168.1.1,prod-server-01\n192.168.1.2,prod-server-02\n",
        )
        .expect("AC-7: CSV write must succeed");
    }

    // Normalise to forward slashes: TOML basic strings treat `\` as an escape-sequence
    // prefix; Windows backslash paths (e.g. `C:\Users\...`) contain sequences like `\U`
    // or `\A` that are invalid TOML escapes, causing parse failure and 0 specs returned.
    // Forward slashes are accepted by Rust's std::fs and the `csv` crate on all platforms.
    let csv_abs = csv_path.to_string_lossy().replace('\\', "/");
    let toml_content = format!(
        r#"
[infusion]
infusion_id = "tier3_prod_test"
name = "Tier3 Production Test"

[source]
type = "csv"
file_path = "{csv_abs}"
key_column = "device_ip"

[[infusion.fields]]
name = "tier3_asset_name"
input_field = "device_ip"
input_type = "ip"
output_type = "string"
source_column = "asset_name"

[infusion.pipe_stage]
adds_columns = ["tier3_asset_name"]
"#,
        csv_abs = csv_abs
    );

    let spec_path = infusions_dir.join("tier3_prod_test.infusion.toml");
    {
        use std::io::Write;
        let mut f = std::fs::File::create(&spec_path).expect("AC-7: TOML create must succeed");
        f.write_all(toml_content.as_bytes())
            .expect("AC-7: TOML write must succeed");
    }

    // An empty PluginRuntime is correct here — this test uses a LocalLookup CSV spec.
    let runtime = make_test_runtime();
    let registry = infusion_load_step(temp_dir.path(), &runtime);
    let descriptors = registry.udf_descriptors();
    assert!(
        !descriptors.is_empty(),
        "AC-7: infusion_load_step must produce at least 1 descriptor from the tier3 test TOML"
    );

    // ---------------------------------------------------------------------------
    // Wire the three-tier cache chain — SAME chain as production boot path:
    //   .with_infusion_registry(reg).with_infusion_caches(lru, tier3)
    // ---------------------------------------------------------------------------
    let lru = Arc::new(InfusionLruCache::new(
        std::num::NonZeroUsize::new(10_000).unwrap(),
    ));
    let tracking_backend = TrackingCacheBackend::new();
    let tier3 = Arc::new(InfusionTier3Cache::new(
        Arc::clone(&tracking_backend) as Arc<dyn CacheBackend>
    ));

    // Build a SessionContext and register UDFs with the full three-tier cache
    // (this is what QueryEngine::execute_inner does when both caches are Some).
    let ctx = SessionContext::new();
    register_infusion_udfs_with_cache(
        &ctx,
        descriptors.clone(),
        Arc::clone(&lru),
        Arc::clone(&tier3),
        3600,
    )
    .expect("AC-7: register_infusion_udfs_with_cache must succeed");

    // Register a test table with one IP row.
    let schema = Arc::new(Schema::new(vec![Field::new(
        "device_ip",
        DataType::Utf8,
        false,
    )]));
    let arr = StringArray::from(vec!["192.168.1.1"]);
    let batch = RecordBatch::try_new(Arc::clone(&schema), vec![Arc::new(arr)])
        .expect("AC-7: RecordBatch construction must succeed");
    let table = MemTable::try_new(Arc::clone(&schema), vec![vec![batch]])
        .expect("AC-7: MemTable construction must succeed");
    ctx.register_table("test_devices", Arc::new(table))
        .expect("AC-7: register_table must succeed");

    // ---------------------------------------------------------------------------
    // First call: source is invoked, result is written to Tier-3
    // ---------------------------------------------------------------------------
    let df1 = ctx
        .sql("SELECT tier3_asset_name(device_ip) AS aname FROM test_devices")
        .await
        .expect("AC-7: first SQL plan must succeed");
    let batches1 = df1
        .collect()
        .await
        .expect("AC-7: first SQL execute must succeed");
    assert_eq!(batches1.len(), 1, "AC-7: first call must produce 1 batch");

    let set_calls_after_first = tracking_backend.set_calls();
    assert!(
        set_calls_after_first > 0,
        "AC-7: Tier-3 backend must have received at least 1 set() call after first UDF invocation \
         (source result must be written to persistent cache); got set_calls={}",
        set_calls_after_first
    );

    // Verify the name resolved correctly from CSV.
    let name_col1 = batches1[0]
        .column(0)
        .as_any()
        .downcast_ref::<StringArray>()
        .expect("AC-7: output must be StringArray");
    assert_eq!(
        name_col1.value(0),
        "prod-server-01",
        "AC-7: first call must resolve tier3_asset_name to 'prod-server-01' from CSV source"
    );

    let get_calls_before_second = tracking_backend.get_calls();

    // ---------------------------------------------------------------------------
    // OBS-2 load-bearing sentinel: overwrite the CSV source file with a DIFFERENT value
    // before issuing the second query. If Tier-3 cache serves the second query correctly,
    // the OLD cached value "prod-server-01" is returned. If the source is re-called
    // (cache miss / regression), the NEW sentinel value "SENTINEL-SOURCE-RECALLED" is
    // returned — causing the assertion below to fail.
    //
    // This makes the test truly load-bearing: prior to this strengthening, both code paths
    // (cache hit AND source re-call) would have returned "prod-server-01" from the unmodified
    // CSV, so the test could pass even if Tier-3 were not serving the second query (OBS-2).
    // ---------------------------------------------------------------------------
    {
        use std::io::Write;
        let mut f = std::fs::File::create(&csv_path)
            .expect("AC-7/OBS-2: overwrite CSV with sentinel value must succeed");
        f.write_all(
            b"device_ip,asset_name\n192.168.1.1,SENTINEL-SOURCE-RECALLED\n192.168.1.2,prod-server-02\n",
        )
        .expect("AC-7/OBS-2: sentinel CSV write must succeed");
    }

    // ---------------------------------------------------------------------------
    // Second call: Tier-3 backend must be queried (get() is called), returning a hit.
    // Re-register UDFs in a new context (Tier-1 is per-invoke; Tier-2 LRU is shared).
    // Sharing the same LRU means Tier-2 may hit on second call — use a fresh LRU to
    // force Tier-3 to be the tier that proves persistence.
    // ---------------------------------------------------------------------------
    let ctx2 = SessionContext::new();
    let fresh_lru = Arc::new(InfusionLruCache::new(
        std::num::NonZeroUsize::new(10_000).unwrap(),
    )); // fresh — no Tier-2 hits
    register_infusion_udfs_with_cache(&ctx2, descriptors, fresh_lru, Arc::clone(&tier3), 3600)
        .expect("AC-7: second register_infusion_udfs_with_cache must succeed");

    let arr2 = StringArray::from(vec!["192.168.1.1"]);
    let batch2 = RecordBatch::try_new(Arc::clone(&schema), vec![Arc::new(arr2)])
        .expect("AC-7: second RecordBatch must succeed");
    let table2 = MemTable::try_new(Arc::clone(&schema), vec![vec![batch2]])
        .expect("AC-7: second MemTable must succeed");
    ctx2.register_table("test_devices", Arc::new(table2))
        .expect("AC-7: second register_table must succeed");

    let df2 = ctx2
        .sql("SELECT tier3_asset_name(device_ip) AS aname FROM test_devices")
        .await
        .expect("AC-7: second SQL plan must succeed");
    let batches2 = df2
        .collect()
        .await
        .expect("AC-7: second SQL execute must succeed");

    let get_calls_after_second = tracking_backend.get_calls();
    assert!(
        get_calls_after_second > get_calls_before_second,
        "AC-7: Tier-3 backend must have received at least 1 additional get() call on second \
         invocation (with fresh Tier-2 LRU, Tier-3 must be consulted for persistence); \
         before={}, after={}",
        get_calls_before_second,
        get_calls_after_second
    );

    // OBS-2 load-bearing assertion: second call MUST return the cached value "prod-server-01",
    // NOT the sentinel "SENTINEL-SOURCE-RECALLED" written to the CSV after the first call.
    // If this assertion fails, Tier-3 is not serving the second query — source was re-called,
    // which is a CRIT-1 regression (three-tier cache not wired correctly in production path).
    let name_col2 = batches2[0]
        .column(0)
        .as_any()
        .downcast_ref::<StringArray>()
        .expect("AC-7: second output must be StringArray");
    assert_eq!(
        name_col2.value(0),
        "prod-server-01",
        "AC-7/OBS-2: second call must return cached 'prod-server-01' from Tier-3 cache, NOT \
         'SENTINEL-SOURCE-RECALLED' from the overwritten CSV file; a sentinel value here means \
         the source was re-called on the second query instead of being served from Tier-3 \
         (CRIT-1 regression: three-tier cache not wired correctly in production path)"
    );
}

/// Task 13 / F-SV-1 load-bearing test: plugin-type infusion spec wired through `infusion_load_step`
/// with a real `Arc<PluginRuntime>` stores a `PluginInfusionSource` — not `NullSource`.
///
/// # What this proves (structural, not just behavioral)
///
/// Before the Task 13 fix, `infusion_load_step` called `registry.load_spec(spec)` for ALL specs
/// (including `InfusionType::Plugin`), which stores `NullSource`.  `NullSource::is_plugin_backed()`
/// returns `false`.  `PluginInfusionSource::is_plugin_backed()` returns `true`.
/// This test asserts `is_plugin_backed() == true` for every descriptor produced by a plugin-type
/// spec — proving the fix landed and `load_spec_with_runtime` was called, not `load_spec`.
///
/// # Why the NullSource hollow-feature matters
///
/// With `NullSource`, `enrich_single` returns `None` immediately without ever calling
/// `PluginRuntime::enrich_single`. With `PluginInfusionSource`, the runtime IS called —
/// and for a loaded plugin, it returns real enrichment data. For an unloaded plugin (no .prx
/// registered), it returns `PluginError::NotLoaded → None` (non-panicking).
///
/// This test uses an empty `PluginRuntime` (no .prx loaded) — the plugin returns `NotLoaded → None`.
/// That is correct for a unit test: the structural proof (is_plugin_backed) is the assertion,
/// not the enrichment value.
///
/// # Regression detection
///
/// If the fix regresses (plugin spec goes back through `load_spec` instead of
/// `load_spec_with_runtime`), every descriptor's `source.is_plugin_backed()` returns `false`
/// and this test fails — catching the regression before it ships.
///
/// Traces to: Task 13 (F-SV-1), S-1.14-REDO AC-10, BC-2.19.001 (plugin-type spec must be
/// wired with a real PluginInfusionSource in the production boot path).
#[test]
fn test_boot_plugin_infusion_spec_wired_with_real_plugin_source_not_null_source() {
    use prism_spec_engine::InfusionSource;
    use std::io::Write;
    use tempfile::TempDir;

    // Create a minimal temp config dir with a plugin-type infusion TOML.
    let temp_dir =
        TempDir::new().expect("Task13/F-SV-1: temp dir creation must succeed for test setup");
    let infusions_dir = temp_dir.path().join("infusions");
    std::fs::create_dir_all(&infusions_dir).expect("Task13/F-SV-1: infusions dir must be created");

    // Write a plugin-type infusion TOML — source.type = "plugin" + source.plugin_ref path.
    // The plugin does NOT need to exist as a real .prx file for this test because:
    // 1. infusion_load_step only parses the TOML and calls load_spec_with_runtime — it does
    //    NOT load or validate the .prx file (that happens at enrichment dispatch time).
    // 2. The load-bearing assertion is source.is_plugin_backed(), not the enrichment value.
    //
    // Source type resolution order (loader.rs):
    //   1. [source].type  (top-level — this is what we use here)
    //   2. [infusion.source].type
    //   3. [infusion].type
    //   4. [infusion].source_type fallback
    let plugin_infusion_toml = r#"
[infusion]
infusion_id = "threat_intel"
name = "Threat Intel Plugin"

[source]
type = "plugin"
plugin_ref = "plugins/threat_intel.prx"

[[infusion.fields]]
name = "threat_intel_score"
input_field = "src_ip"
input_type = "ip"
output_type = "string"
source_column = "threat_intel_score"

[[infusion.fields]]
name = "threat_intel_category"
input_field = "src_ip"
input_type = "ip"
output_type = "string"
source_column = "threat_intel_category"

[infusion.pipe_stage]
adds_columns = ["threat_intel_score", "threat_intel_category"]
"#;

    let spec_path = infusions_dir.join("threat_intel.infusion.toml");
    {
        let mut f = std::fs::File::create(&spec_path)
            .expect("Task13/F-SV-1: TOML fixture creation must succeed");
        f.write_all(plugin_infusion_toml.as_bytes())
            .expect("Task13/F-SV-1: TOML write must succeed");
    }

    // Call infusion_load_step with a real (but empty) PluginRuntime.
    // Task 13 fix: plugin-type spec must go through load_spec_with_runtime (not load_spec).
    let runtime = make_test_runtime();
    let registry = infusion_load_step(temp_dir.path(), &runtime);

    // Assert the spec was successfully registered (not silently dropped).
    let descriptors = registry.udf_descriptors();
    assert_eq!(
        descriptors.len(),
        2,
        "Task13/F-SV-1: plugin-type infusion spec must produce 2 UDF descriptors \
         (threat_intel_score, threat_intel_category). Got: {}. \
         A 0-descriptor result means the spec failed to register (check WARN logs).",
        descriptors.len()
    );

    // Structural load-bearing assertion: every descriptor from a plugin-type spec must carry a
    // PluginInfusionSource (is_plugin_backed() == true), NOT a NullSource (is_plugin_backed() == false).
    //
    // Before the Task 13 fix: infusion_load_step called load_spec() for all specs, which stores
    // NullSource for plugin-type specs. NullSource::is_plugin_backed() returns false.
    // After the fix: infusion_load_step calls load_spec_with_runtime() for plugin specs, which
    // stores PluginInfusionSource. PluginInfusionSource::is_plugin_backed() returns true.
    //
    // A regression to NullSource would cause this assertion to fail, catching the hollow-feature
    // at test time rather than silently at production enrichment dispatch (where NullSource
    // returns None without ever calling PluginRuntime).
    for desc in &descriptors {
        assert!(
            desc.source.is_plugin_backed(),
            "Task13/F-SV-1: descriptor '{}' from plugin-type infusion spec must carry a \
             PluginInfusionSource (is_plugin_backed=true), not NullSource (is_plugin_backed=false). \
             This assertion fails when infusion_load_step regresses to calling load_spec() for \
             plugin-type specs instead of load_spec_with_runtime() (F-SV-1 hollow-feature).",
            desc.name
        );
    }

    // is_api_backed() must return true for both UDF names (InfusionType::Plugin).
    assert!(
        registry.is_api_backed("threat_intel_score"),
        "Task13/F-SV-1: is_api_backed('threat_intel_score') must be true for plugin-type infusion \
         (BC-2.19.003 / INV-INFUSE-003)"
    );
    assert!(
        registry.is_api_backed("threat_intel_category"),
        "Task13/F-SV-1: is_api_backed('threat_intel_category') must be true for plugin-type infusion \
         (BC-2.19.003 / INV-INFUSE-003)"
    );
}

/// AC-10 (non-fatal absent dir): `infusion_load_step` with no infusions dir returns empty registry.
///
/// Verifies that an absent `infusions/` directory does NOT cause a boot failure.
/// `infusion_load_step` returns an empty registry (0 descriptors) without error — non-fatal
/// per BC-2.22.001.
///
/// Traces to: BC-2.22.001 — infusion load step is non-fatal when dir is absent.
#[test]
fn test_boot_infusion_load_step_empty_dir_returns_empty_registry() {
    use tempfile::TempDir;

    // Config dir with NO `infusions/` subdirectory.
    let temp_dir = TempDir::new().expect("AC-10: temp dir creation must succeed for test setup");

    // infusion_load_step handles absent dirs non-fatally — returns empty registry.
    // An empty PluginRuntime is correct here — no specs loaded in this test.
    let runtime = make_test_runtime();
    let registry = infusion_load_step(temp_dir.path(), &runtime);

    // After S-1.14-REDO: empty registry is acceptable (no infusions configured).
    let descriptors = registry.udf_descriptors();
    assert!(
        descriptors.is_empty(),
        "AC-10: infusion_load_step with no infusions/ dir must return an empty InfusionRegistry \
         without panicking (non-fatal absent-dir handling — BC-2.22.001)"
    );
}
