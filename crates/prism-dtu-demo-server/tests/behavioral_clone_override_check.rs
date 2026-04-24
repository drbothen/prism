//! Compile-time BehavioralClone override check (P3WV1-A-L-002).
//!
//! Instantiates each of the 6 clone types and verifies that:
//! - `.start_on(addr, None).await` succeeds without panic.
//! - `.stop().await` succeeds without panic.
//!
//! This catches any clone author who forgets to override `start_on` or `stop`,
//! which would fall through to the default `unimplemented!()` in the trait.
//!
//! If any clone fails this test, the developer MUST implement `start_on` and `stop`
//! in that clone crate before merging.

#![allow(clippy::unwrap_used, clippy::expect_used)]
use prism_dtu_common::BehavioralClone;

/// Verify CrowdStrike clone overrides start_on + stop (both do not panic).
#[tokio::test]
async fn clone_override_crowdstrike_start_stop() {
    let mut clone = prism_dtu_crowdstrike::CrowdstrikeClone::new();
    let addr = clone
        .start_on("127.0.0.1:0".parse().unwrap(), None, None)
        .await
        .expect("CrowdstrikeClone::start_on must not panic or error");
    assert!(
        addr.port() > 0,
        "CrowdstrikeClone::start_on must bind to a non-zero port"
    );
    clone
        .stop()
        .await
        .expect("CrowdstrikeClone::stop must not panic or error");
}

/// Verify Claroty clone overrides start_on + stop (both do not panic).
#[tokio::test]
async fn clone_override_claroty_start_stop() {
    let mut clone = prism_dtu_claroty::ClarotyClone::new();
    let addr = clone
        .start_on("127.0.0.1:0".parse().unwrap(), None, None)
        .await
        .expect("ClarotyClone::start_on must not panic or error");
    assert!(
        addr.port() > 0,
        "ClarotyClone::start_on must bind to a non-zero port"
    );
    clone
        .stop()
        .await
        .expect("ClarotyClone::stop must not panic or error");
}

/// Verify Cyberint clone overrides start_on + stop (both do not panic).
#[tokio::test]
async fn clone_override_cyberint_start_stop() {
    let mut clone =
        prism_dtu_cyberint::CyberintClone::new().expect("CyberintClone::new must succeed");
    let addr = clone
        .start_on("127.0.0.1:0".parse().unwrap(), None, None)
        .await
        .expect("CyberintClone::start_on must not panic or error");
    assert!(
        addr.port() > 0,
        "CyberintClone::start_on must bind to a non-zero port"
    );
    clone
        .stop()
        .await
        .expect("CyberintClone::stop must not panic or error");
}

/// Verify Armis clone overrides start_on + stop (both do not panic).
#[tokio::test]
async fn clone_override_armis_start_stop() {
    let mut clone = prism_dtu_armis::ArmisClone::new().expect("ArmisClone::new must succeed");
    let addr = clone
        .start_on("127.0.0.1:0".parse().unwrap(), None, None)
        .await
        .expect("ArmisClone::start_on must not panic or error");
    assert!(
        addr.port() > 0,
        "ArmisClone::start_on must bind to a non-zero port"
    );
    clone
        .stop()
        .await
        .expect("ArmisClone::stop must not panic or error");
}

/// Verify ThreatIntel clone overrides start_on + stop (both do not panic).
#[tokio::test]
async fn clone_override_threatintel_start_stop() {
    let mut clone = prism_dtu_threatintel::ThreatIntelClone::new();
    let addr = clone
        .start_on("127.0.0.1:0".parse().unwrap(), None, None)
        .await
        .expect("ThreatIntelClone::start_on must not panic or error");
    assert!(
        addr.port() > 0,
        "ThreatIntelClone::start_on must bind to a non-zero port"
    );
    clone
        .stop()
        .await
        .expect("ThreatIntelClone::stop must not panic or error");
}

/// Verify NVD clone overrides start_on + stop (both do not panic).
#[tokio::test]
async fn clone_override_nvd_start_stop() {
    let mut clone = prism_dtu_nvd::NvdClone::new().expect("NvdClone::new must succeed");
    let addr = clone
        .start_on("127.0.0.1:0".parse().unwrap(), None, None)
        .await
        .expect("NvdClone::start_on must not panic or error");
    assert!(
        addr.port() > 0,
        "NvdClone::start_on must bind to a non-zero port"
    );
    clone
        .stop()
        .await
        .expect("NvdClone::stop must not panic or error");
}
