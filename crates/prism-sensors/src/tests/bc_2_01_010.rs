//! Tests for BC-2.01.010: Partial Failure Handling for Paginated and
#![allow(clippy::expect_used, clippy::unwrap_used)]
//! Cross-Client Queries.
//!
//! Covers:
//! - EC-001: All targets fail → `Err(SensorError::AllTargetsFailed)`
//! - EC-002 / AC-2: 5 of 6 succeed, 1 returns HTTP 503 →
//!   `successes.len() == 5`, `errors.len() == 1`, `is_transient == true`
//! - Partial failure: at least one success → `Ok(FanOutResult)` not `Err`
//! - `FanOutError` fields accessible: `client_id`, `sensor_type`, `error`,
//!   `retry_metadata`
//! - `SensorError::AllTargetsFailed` carries the full error list
//!
//! All tests pass (fan_out() implemented).
//!
//! Story: S-2.06 | BC: BC-2.01.010

use crate::adapter::SensorError;
use prism_core::types::SensorType;

// ---------------------------------------------------------------------------
// Structural: FanOutError fields accessible
// ---------------------------------------------------------------------------

/// `FanOutError` fields are publicly accessible and `Display`-able.
#[test]
fn test_BC_2_01_010_fan_out_error_fields_accessible() {
    use crate::fanout::{FanOutError, RetryMetadata};

    #[allow(deprecated)]
    let err = FanOutError {
        org_id: prism_core::OrgId::new(),
        client_id: "acme".into(),
        sensor_type: SensorType::CrowdStrike,
        error: SensorError::HttpError {
            sensor: "crowdstrike".into(),
            status: 503,
            body: "unavailable".into(),
        },
        retry_metadata: RetryMetadata {
            attempts: 3,
            last_error_code: "503".into(),
            is_transient: true,
        },
    };

    #[allow(deprecated)]
    let _ = assert_eq!(err.client_id, "acme");
    assert_eq!(err.sensor_type, SensorType::CrowdStrike);
    assert_eq!(err.retry_metadata.attempts, 3);
    assert_eq!(err.retry_metadata.last_error_code, "503");
    assert!(err.retry_metadata.is_transient);

    // Display must not panic
    let display = format!("{err}");
    assert!(
        !display.is_empty(),
        "Display must be non-empty; got: {display:?}"
    );
}

/// `FanOutResult` fields are accessible and `Default`-constructible.
#[test]
fn test_BC_2_01_010_fan_out_result_default_is_empty() {
    use crate::fanout::FanOutResult;

    let result = FanOutResult::default();
    assert!(result.successes.is_empty());
    assert!(result.errors.is_empty());
}

// ---------------------------------------------------------------------------
// SensorError::AllTargetsFailed
// ---------------------------------------------------------------------------

/// `SensorError::AllTargetsFailed` carries the per-target error list and
/// Display contains the count.
#[test]
fn test_BC_2_01_010_all_targets_failed_contains_error_count() {
    use crate::fanout::{FanOutError, RetryMetadata};

    #[allow(deprecated)]
    let errors = vec![
        FanOutError {
            org_id: prism_core::OrgId::new(),
            client_id: "acme".into(),
            sensor_type: SensorType::CrowdStrike,
            error: SensorError::HttpError {
                sensor: "crowdstrike".into(),
                status: 503,
                body: String::new(),
            },
            retry_metadata: RetryMetadata {
                attempts: 3,
                last_error_code: "503".into(),
                is_transient: true,
            },
        },
        FanOutError {
            org_id: prism_core::OrgId::new(),
            client_id: "globex".into(),
            sensor_type: SensorType::Armis,
            error: SensorError::Timeout {
                sensor: "armis".into(),
                elapsed_ms: 30_000,
            },
            retry_metadata: RetryMetadata {
                attempts: 1,
                last_error_code: "timeout".into(),
                is_transient: true,
            },
        },
    ];

    let all_failed = SensorError::AllTargetsFailed {
        count: errors.len(),
        errors,
    };

    let display = format!("{all_failed}");
    assert!(
        display.contains('2') || display.contains("2 errors"),
        "Display must contain the error count; got: {display:?}"
    );
}

// ---------------------------------------------------------------------------
// fan_out() partial failure tests
// ---------------------------------------------------------------------------

/// EC-001: When ALL fan-out targets fail, `fan_out()` must return
/// `Err(SensorError::AllTargetsFailed { errors })`.
///
/// BC-2.01.010: "If ALL targets fail, return `Err(SensorError::AllTargetsFailed)`."
///
#[tokio::test]
async fn test_BC_2_01_010_fan_out_all_targets_fail_returns_all_targets_failed() {
    use arrow::record_batch::RecordBatch;
    use async_trait::async_trait;
    use std::sync::Arc;

    use crate::{
        adapter::{QueryParams, SensorAdapter, SensorError, SensorSpec},
        auth::SensorAuth,
        fanout::{fan_out, CredentialResolver, FanOutTarget},
        registry::AdapterRegistry,
    };

    struct AlwaysFailsAdapter;

    #[async_trait]
    impl SensorAdapter for AlwaysFailsAdapter {
        fn sensor_type(&self) -> SensorType {
            SensorType::CrowdStrike
        }
        fn sensor_name(&self) -> &'static str {
            "always-fail"
        }
        async fn fetch(
            &self,
            _spec: &SensorSpec,
            _params: &QueryParams,
            _auth: &dyn SensorAuth,
        ) -> Result<Vec<RecordBatch>, SensorError> {
            Err(SensorError::HttpError {
                sensor: "crowdstrike".into(),
                status: 503,
                body: "down".into(),
            })
        }
    }

    struct StubCreds;
    impl CredentialResolver for StubCreds {
        fn resolve(
            &self,
            _client_id: &str,
            _sensor_type: SensorType,
        ) -> Result<Box<dyn SensorAuth>, SensorError> {
            use secrecy::SecretString;
            Ok(Box::new(crate::auth::CrowdStrikeAuth {
                client_id: "stub".into(),
                client_secret: SecretString::new("s".into()),
                cloud_region: "us-1".into(),
            }))
        }
    }

    // Single shared OrgId — test validates all-fail behavior, not per-org isolation.
    let shared_org_id = prism_core::OrgId::from_uuid(uuid::Uuid::from_bytes([
        0x01, 0x8e, 0x3f, 0x71, 0x5c, 0x6d, 0x7a, 0x8b, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x01,
    ]));

    let mut registry = AdapterRegistry::new();
    registry.register(shared_org_id, Arc::new(AlwaysFailsAdapter));

    #[allow(deprecated)]
    let targets = vec![
        FanOutTarget {
            org_id: shared_org_id,
            client_id: "acme".into(),
            sensor_type: SensorType::CrowdStrike,
            spec: SensorSpec {
                org_id: shared_org_id,
                source_table: "crowdstrike_alert".into(),
                client_id: "acme".into(),
                sensor_config: serde_json::json!({}),
            },
            params: QueryParams::default(),
        },
        FanOutTarget {
            org_id: shared_org_id,
            client_id: "globex".into(),
            sensor_type: SensorType::CrowdStrike,
            spec: SensorSpec {
                org_id: shared_org_id,
                source_table: "crowdstrike_alert".into(),
                client_id: "globex".into(),
                sensor_config: serde_json::json!({}),
            },
            params: QueryParams::default(),
        },
    ];

    let result = fan_out(targets, Arc::new(registry), Arc::new(StubCreds)).await;

    assert!(
        result.is_err(),
        "all targets failed — must return Err(SensorError::AllTargetsFailed)"
    );
    assert!(
        matches!(result.unwrap_err(), SensorError::AllTargetsFailed { count, .. } if count == 2),
        "error must be AllTargetsFailed with count == 2"
    );
}

/// AC-2 / EC-002: 5 of 6 targets succeed; 1 returns HTTP 503.
///
/// Expected:
/// - `FanOutResult.successes.len() == 5`
/// - `FanOutResult.errors.len() == 1`
/// - `errors[0].retry_metadata.is_transient == true`
///
#[tokio::test]
async fn test_BC_2_01_010_fan_out_five_succeed_one_503_returns_partial_result() {
    use arrow::{
        array::Int32Array,
        datatypes::{DataType, Field, Schema},
        record_batch::RecordBatch,
    };
    use async_trait::async_trait;
    use std::sync::{atomic::Ordering, Arc};

    use crate::{
        adapter::{QueryParams, SensorAdapter, SensorError, SensorSpec},
        auth::SensorAuth,
        fanout::{fan_out, CredentialResolver, FanOutTarget},
        registry::AdapterRegistry,
    };

    // Counter to track how many times the adapter is invoked; fails on the 6th call.
    static CALL_COUNT: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

    struct PartialFailAdapter;

    #[async_trait]
    impl SensorAdapter for PartialFailAdapter {
        fn sensor_type(&self) -> SensorType {
            SensorType::CrowdStrike
        }
        fn sensor_name(&self) -> &'static str {
            "partial-fail"
        }
        async fn fetch(
            &self,
            _spec: &SensorSpec,
            _params: &QueryParams,
            _auth: &dyn SensorAuth,
        ) -> Result<Vec<RecordBatch>, SensorError> {
            let n = CALL_COUNT.fetch_add(1, Ordering::SeqCst);
            if n == 5 {
                // 6th call (0-indexed) fails with 503
                return Err(SensorError::HttpError {
                    sensor: "crowdstrike".into(),
                    status: 503,
                    body: "unavailable".into(),
                });
            }
            let schema = Arc::new(Schema::new(vec![Field::new("id", DataType::Int32, false)]));
            let batch = RecordBatch::try_new(schema, vec![Arc::new(Int32Array::from(vec![1i32]))])
                .expect("valid batch");
            Ok(vec![batch])
        }
    }

    struct StubCreds;
    impl CredentialResolver for StubCreds {
        fn resolve(
            &self,
            _client_id: &str,
            _sensor_type: SensorType,
        ) -> Result<Box<dyn SensorAuth>, SensorError> {
            use secrecy::SecretString;
            Ok(Box::new(crate::auth::CrowdStrikeAuth {
                client_id: "stub".into(),
                client_secret: SecretString::new("s".into()),
                cloud_region: "us-1".into(),
            }))
        }
    }

    // Single shared OrgId — test validates partial-fail behavior, not per-org isolation.
    let shared_org_id = prism_core::OrgId::from_uuid(uuid::Uuid::from_bytes([
        0x01, 0x8e, 0x3f, 0x71, 0x5c, 0x6d, 0x7a, 0x8b, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x01,
    ]));

    let mut registry = AdapterRegistry::new();
    registry.register(shared_org_id, Arc::new(PartialFailAdapter));

    // Reset counter for this test run
    CALL_COUNT.store(0, Ordering::SeqCst);

    let clients = ["a", "b", "c", "d", "e", "f"];
    #[allow(deprecated)]
    let targets: Vec<FanOutTarget> = clients
        .iter()
        .map(|&client_id| FanOutTarget {
            org_id: shared_org_id,
            client_id: client_id.into(),
            sensor_type: SensorType::CrowdStrike,
            spec: SensorSpec {
                org_id: shared_org_id,
                source_table: "crowdstrike_alert".into(),
                client_id: client_id.into(),
                sensor_config: serde_json::json!({}),
            },
            params: QueryParams::default(),
        })
        .collect();

    let result = fan_out(targets, Arc::new(registry), Arc::new(StubCreds))
        .await
        .expect("partial failure must return Ok(FanOutResult), not Err");

    assert_eq!(
        result.successes.len(),
        5,
        "AC-2: 5 successes expected; got {}",
        result.successes.len()
    );
    assert_eq!(
        result.errors.len(),
        1,
        "AC-2: 1 error expected; got {}",
        result.errors.len()
    );
    assert!(
        result.errors[0].retry_metadata.is_transient,
        "AC-2: the 503 error must be classified as transient"
    );
}
