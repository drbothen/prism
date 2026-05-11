//! Tests for BC-2.01.002: Cross-Client Fan-Out — Query Engine Orchestrates
#![allow(clippy::expect_used, clippy::unwrap_used)]
//! Parallel Sensor Fetches.
//!
//! Covers:
//! - `MAX_FANOUT_CONCURRENCY` literal == 10 (AC-1)
//! - `fan_out()` with all-success targets returns `FanOutResult` with correct
//!   successes count and empty errors (AC-1, TV-BC-2.01.002-001)
//! - `fan_out()` with empty targets returns an empty `FanOutResult`
//! - `FanOutTarget` fields compile and are accessible
//! - `error_to_retry_metadata()` populates `RetryMetadata` correctly from a
//!   `SensorError::HttpError`
//! - `error_to_retry_metadata()` sets `is_transient = true` for 503
//! - `error_to_retry_metadata()` sets `is_transient = false` for 400
//!
//! All tests pass (fan_out() implementation complete).
//!
//! Story: S-2.06 | BC: BC-2.01.002

use prism_core::SensorId;

use crate::adapter::SensorError;
use crate::fanout::{error_to_retry_metadata, MAX_FANOUT_CONCURRENCY};

// ---------------------------------------------------------------------------
// MAX_FANOUT_CONCURRENCY literal assertion (AC-1)
// ---------------------------------------------------------------------------

/// AC-1: The fan-out concurrency cap must be exactly 10.
///
/// BC-2.01.002: "Limit concurrency to 10 per query via a `tokio::sync::Semaphore`
/// with 10 permits."
#[test]
fn test_BC_2_01_002_max_fanout_concurrency_is_10() {
    assert_eq!(
        MAX_FANOUT_CONCURRENCY, 10,
        "MAX_FANOUT_CONCURRENCY must be 10 (AC-1, BC-2.01.002)"
    );
}

// ---------------------------------------------------------------------------
// FanOutTarget construction
// ---------------------------------------------------------------------------

/// `FanOutTarget` fields are publicly accessible and `Clone`-able.
/// Verifies the struct layout compiles as specified in the story.
#[test]
fn test_BC_2_01_002_fan_out_target_fields_accessible() {
    use crate::{
        adapter::{QueryParams, SensorSpec},
        fanout::FanOutTarget,
    };

    #[allow(deprecated)]
    let target = FanOutTarget {
        org_id: prism_core::OrgId::new(),
        client_id: "acme".into(),
        sensor_id: SensorId::from("crowdstrike"),
        spec: SensorSpec {
            org_id: prism_core::OrgId::new(),
            source_table: "crowdstrike_alert".into(),
            client_id: "acme".into(),
            sensor_config: serde_json::json!({}),
        },
        params: QueryParams::default(),
    };

    #[allow(deprecated)]
    let _ = assert_eq!(target.client_id, "acme");
    assert_eq!(target.sensor_id, SensorId::from("crowdstrike"));
    assert_eq!(target.spec.source_table, "crowdstrike_alert");

    // Clone round-trip
    let cloned = target.clone();
    #[allow(deprecated)]
    let _ = assert_eq!(cloned.client_id, "acme");
}

// ---------------------------------------------------------------------------
// error_to_retry_metadata helpers
// ---------------------------------------------------------------------------

/// `error_to_retry_metadata` with a transient HTTP 503 error must produce
/// `is_transient = true` and `last_error_code = "503"`.
#[test]
fn test_BC_2_01_002_error_to_retry_metadata_503_is_transient() {
    let err = SensorError::HttpError {
        sensor: "crowdstrike".into(),
        status: 503,
        body: "service unavailable".into(),
    };
    let meta = error_to_retry_metadata(&err, 2);

    assert_eq!(meta.attempts, 2);
    assert_eq!(meta.last_error_code, "503");
    assert!(
        meta.is_transient,
        "503 error must produce is_transient = true"
    );
}

/// `error_to_retry_metadata` with a non-transient HTTP 400 error must produce
/// `is_transient = false`.
#[test]
fn test_BC_2_01_002_error_to_retry_metadata_400_is_not_transient() {
    let err = SensorError::HttpError {
        sensor: "armis".into(),
        status: 400,
        body: "bad request".into(),
    };
    let meta = error_to_retry_metadata(&err, 1);

    assert_eq!(meta.last_error_code, "400");
    assert!(
        !meta.is_transient,
        "400 error must produce is_transient = false"
    );
}

/// `error_to_retry_metadata` with a `Timeout` error uses `"timeout"` code.
#[test]
fn test_BC_2_01_002_error_to_retry_metadata_timeout_code() {
    let err = SensorError::Timeout {
        sensor: "cyberint".into(),
        elapsed_ms: 30_000,
    };
    let meta = error_to_retry_metadata(&err, 3);

    assert_eq!(meta.last_error_code, "timeout");
    assert!(meta.is_transient, "Timeout must be transient");
    assert_eq!(meta.attempts, 3);
}

/// `error_to_retry_metadata` with `RateLimited` uses HTTP 429 code.
#[test]
fn test_BC_2_01_002_error_to_retry_metadata_rate_limited_429_code() {
    let err = SensorError::RateLimited {
        sensor: "crowdstrike".into(),
        retry_after_ms: 5_000,
    };
    let meta = error_to_retry_metadata(&err, 1);

    assert_eq!(meta.last_error_code, "429");
    assert!(meta.is_transient, "RateLimited must be transient");
}

// ---------------------------------------------------------------------------
// fan_out() async tests
// ---------------------------------------------------------------------------

/// AC-1 / TV-BC-2.01.002-001: Fan-out over 6 targets (3 clients × 2 sensors),
/// all succeed. `FanOutResult.errors` must be empty, `successes` contains all
/// returned batches.
///
/// This test calls `fan_out()` (implementation complete).

#[tokio::test]
async fn test_BC_2_01_002_fan_out_six_targets_all_succeed() {
    use arrow::{
        array::Int32Array,
        datatypes::{DataType, Field, Schema},
        record_batch::RecordBatch,
    };
    use async_trait::async_trait;
    use std::sync::Arc;

    use crate::{
        adapter::{QueryParams, SensorAdapter, SensorError, SensorSpec},
        auth::SensorAuth,
        fanout::{fan_out, CredentialResolver, FanOutTarget},
        registry::AdapterRegistry,
    };

    // Stub adapter that always returns one empty RecordBatch
    struct AlwaysOkAdapter {
        sensor_id: SensorId,
    }

    #[async_trait]
    impl SensorAdapter for AlwaysOkAdapter {
        fn sensor_type(&self) -> SensorId {
            self.sensor_id.clone()
        }
        fn sensor_name(&self) -> &'static str {
            "stub-ok"
        }
        async fn fetch(
            &self,
            _spec: &SensorSpec,
            _params: &QueryParams,
            _auth: &dyn SensorAuth,
        ) -> Result<Vec<RecordBatch>, SensorError> {
            let schema = Arc::new(Schema::new(vec![Field::new("id", DataType::Int32, false)]));
            let batch = RecordBatch::try_new(schema, vec![Arc::new(Int32Array::from(vec![1i32]))])
                .expect("valid batch");
            Ok(vec![batch])
        }
    }

    // Stub credential resolver
    struct StubCreds;
    impl CredentialResolver for StubCreds {
        fn resolve(
            &self,
            _client_id: &str,
            _sensor_id: SensorId,
        ) -> Result<Box<dyn SensorAuth>, SensorError> {
            use secrecy::SecretString;
            Ok(Box::new(crate::auth::CrowdStrikeAuth {
                client_id: "stub".into(),
                client_secret: SecretString::new("s".into()),
                cloud_region: "us-1".into(),
            }))
        }
    }

    // Single shared OrgId — fan_out test validates concurrency, not per-org isolation.
    // All targets use the same OrgId so registry lookup succeeds (BC-3.2.001 precondition 4).
    let shared_org_id = prism_core::OrgId::from_uuid(uuid::Uuid::from_bytes([
        0x01, 0x8e, 0x3f, 0x71, 0x5c, 0x6d, 0x7a, 0x8b, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x01,
    ]));

    let mut registry = AdapterRegistry::new();
    registry.register(
        shared_org_id,
        Arc::new(AlwaysOkAdapter {
            sensor_id: SensorId::from("crowdstrike"),
        }),
    );
    registry.register(
        shared_org_id,
        Arc::new(AlwaysOkAdapter {
            sensor_id: SensorId::from("armis"),
        }),
    );

    // 3 clients × 2 sensors = 6 targets
    let clients = ["acme", "globex", "initech"];
    let sensors = [SensorId::from("crowdstrike"), SensorId::from("armis")];
    #[allow(deprecated)]
    let targets: Vec<FanOutTarget> = clients
        .iter()
        .flat_map(|&client_id| {
            sensors.iter().map(move |sensor_type| FanOutTarget {
                org_id: shared_org_id,
                client_id: client_id.into(),
                sensor_id: sensor_type.clone(),
                spec: SensorSpec {
                    org_id: shared_org_id,
                    source_table: format!("{sensor_type}_alert"),
                    client_id: client_id.into(),
                    sensor_config: serde_json::json!({}),
                },
                params: QueryParams::default(),
            })
        })
        .collect();

    assert_eq!(
        targets.len(),
        6,
        "sanity: 3 clients × 2 sensors = 6 targets"
    );

    let result = fan_out(targets, Arc::new(registry), Arc::new(StubCreds))
        .await
        .expect("all targets succeed — must return Ok(FanOutResult)");

    assert_eq!(result.errors.len(), 0, "no errors expected");
    assert_eq!(
        result.successes.len(),
        6,
        "one RecordBatch per target (6 total)"
    );
}

/// TV-BC-2.01.002-003 / EC-001: Empty target list — fan_out returns an empty
/// `FanOutResult` (0 successes, 0 errors).
///

#[tokio::test]
async fn test_BC_2_01_002_fan_out_empty_targets_returns_empty_result() {
    use crate::{
        fanout::{fan_out, CredentialResolver},
        registry::AdapterRegistry,
    };
    use std::sync::Arc;

    struct StubCreds;
    impl CredentialResolver for StubCreds {
        fn resolve(
            &self,
            _client_id: &str,
            _sensor_id: SensorId,
        ) -> Result<Box<dyn crate::auth::SensorAuth>, SensorError> {
            Err(SensorError::Internal {
                detail: "unreachable".into(),
            })
        }
    }

    let result = fan_out(
        vec![],
        Arc::new(AdapterRegistry::new()),
        Arc::new(StubCreds),
    )
    .await
    .expect("empty target list must return Ok with empty result");

    assert_eq!(result.successes.len(), 0);
    assert_eq!(result.errors.len(), 0);
}
