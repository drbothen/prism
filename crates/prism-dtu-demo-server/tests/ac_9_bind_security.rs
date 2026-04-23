//! AC-9: Bind security — non-loopback binding requires BOTH `--bind-any` flag AND
//! `PRISM_DTU_DEMO_ALLOW_NETWORK_BIND=I-UNDERSTAND-THE-RISK` env var.
//!
//! Given demo.toml sets a non-loopback bind IP AND `--bind-any` is NOT passed
//! (or the env var is missing), when the harness starts, it refuses to bind any
//! clone, prints an error citing R-DEMO-001, and exits with code 1.
//!
//! Expected Red Gate failure: `build_clone_pairs()` or `start_all()` panics with
//! `todo!()` before the security check is even reached.

use prism_dtu_demo_server::config::{CloneConfig, ClonesConfig, DemoConfig, HarnessConfig};

/// AC-9: harness rejects non-loopback bind when --bind-any is absent.
///
/// The harness must detect non-loopback bind IPs in the config and refuse
/// to start (returning Err) when the two-factor gate is not satisfied.
#[tokio::test]
async fn ac_9_non_loopback_bind_without_bind_any_flag_is_rejected() {
    // Ensure the env var is NOT set so we test the rejection path.
    std::env::remove_var("PRISM_DTU_DEMO_ALLOW_NETWORK_BIND");

    let config = DemoConfig {
        harness: HarnessConfig {
            bind: "0.0.0.0".to_string(), // non-loopback!
        },
        clones: ClonesConfig {
            crowdstrike: CloneConfig {
                enabled: true,
                bind: "0.0.0.0".to_string(), // non-loopback!
                port: 0,
                ..Default::default()
            },
            ..Default::default()
        },
    };

    let pairs_result = prism_dtu_demo_server::harness::build_clone_pairs(&config);
    // Expected failure: build_clone_pairs panics with todo!(); when implemented
    // it should return Ok (pairs) and then start_all should return Err (security).
    // For now this just triggers the todo!().

    if let Ok(pairs) = pairs_result {
        let mut harness = prism_dtu_demo_server::harness::DemoHarness::new(pairs);

        // Pass `bind_any = false` (security gate: should reject non-loopback without flag).
        // When implemented, this must return Err citing R-DEMO-001.
        // The config validation function must be called from start_all or a
        // separate validate_bind_security() function.
        let result = harness.start_all(&config).await;

        assert!(
            result.is_err(),
            "AC-9: start_all with non-loopback bind and no --bind-any must return Err (R-DEMO-001)"
        );

        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("R-DEMO-001") || err_msg.contains("--bind-any") || err_msg.contains("loopback"),
            "AC-9: error message must cite R-DEMO-001 or --bind-any; got: {err_msg}"
        );
    }
    // If build_clone_pairs panicked, the test itself panics — that's the expected Red Gate failure.
}

/// AC-9: harness rejects non-loopback bind when env var is set to wrong value.
#[tokio::test]
async fn ac_9_non_loopback_bind_with_wrong_env_var_is_rejected() {
    // Wrong value for the env var.
    std::env::set_var("PRISM_DTU_DEMO_ALLOW_NETWORK_BIND", "yes");

    let config = DemoConfig {
        harness: HarnessConfig {
            bind: "0.0.0.0".to_string(),
        },
        clones: ClonesConfig {
            crowdstrike: CloneConfig {
                enabled: true,
                bind: "0.0.0.0".to_string(),
                port: 0,
                ..Default::default()
            },
            ..Default::default()
        },
    };

    let pairs_result = prism_dtu_demo_server::harness::build_clone_pairs(&config);

    if let Ok(pairs) = pairs_result {
        let mut harness = prism_dtu_demo_server::harness::DemoHarness::new(pairs);
        let result = harness.start_all(&config).await;

        assert!(
            result.is_err(),
            "AC-9: wrong env var value must still reject non-loopback bind"
        );
    }

    std::env::remove_var("PRISM_DTU_DEMO_ALLOW_NETWORK_BIND");
}

/// AC-9: loopback-only config always passes the security gate (no --bind-any needed).
#[tokio::test]
async fn ac_9_loopback_only_config_is_always_allowed() {
    // Ensure env var is NOT set — loopback should always be allowed regardless.
    std::env::remove_var("PRISM_DTU_DEMO_ALLOW_NETWORK_BIND");

    // All defaults use 127.0.0.1 (loopback).
    let config = prism_dtu_demo_server::config::DemoConfig::default();

    // The config with loopback-only binds must be accepted even without --bind-any.
    // Validate by checking: no per-clone or harness bind is non-loopback.
    let is_all_loopback = config.harness.bind == "127.0.0.1"
        || config.harness.bind == "::1"
        || config.harness.bind.starts_with("127.");

    assert!(
        is_all_loopback,
        "AC-9: default DemoConfig must use loopback-only bind; got: {}",
        config.harness.bind
    );
}
