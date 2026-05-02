//! CR-004 stub test suite — exhaustive `match dtu_type` in `start_clone`.
//!
//! Covers:
//!   BC-3.5.001 postcondition 1 (exhaustive dispatch for all DtuType variants)
//!   BC-3.5.002 postcondition 1 (same dispatch path covers network mode)
//!   AC-003 (sequential if-chains replaced with exhaustive match)
//!   AC-007 (network-mode harness dispatch is also correct)
//!   EC-004 (new DtuType variant triggers compile error — tested via build contract)
//!
//! Every test body is `todo!("AC-NNN: <description>")`.
//! ALL tests MUST fail (Red Gate) before the implementing stub lands.
//!
//! Test naming: `test_BC_3_5_001_CR004_xxx()` per factory convention.
//!
//! NOTE: EC-004 (new DtuType → compile error) is a compile-time guarantee, not a
//! runtime test. The exhaustive match must use no `_ =>` wildcard that silently
//! accepts new variants; any such catch-all is explicitly documented per AC-003.
//! The runtime tests here verify that every currently-known DtuType variant starts
//! a clone without error (postcondition 1).
#![allow(non_snake_case, clippy::expect_used, clippy::unwrap_used)]

use prism_dtu_harness::{DtuType, HarnessBuilder, IsolationMode};

// ===========================================================================
// AC-003: every DtuType variant starts a clone successfully via start_clone
// ===========================================================================

/// BC-3.5.001 postcondition 1 / AC-003:
/// `DtuType::Claroty` dispatches to the Claroty-specific clone path.
/// `HarnessBuilder` start succeeds for a Claroty-only spec.
#[tokio::test]
async fn test_BC_3_5_001_CR004_claroty_clone_starts_via_exhaustive_match() {
    todo!("AC-003: DtuType::Claroty must start without error via the exhaustive match dispatch in start_clone")
}

/// BC-3.5.001 postcondition 1 / AC-003:
/// `DtuType::Armis` dispatches to the Armis-specific clone path.
/// `HarnessBuilder` start succeeds for an Armis-only spec.
#[tokio::test]
async fn test_BC_3_5_001_CR004_armis_clone_starts_via_exhaustive_match() {
    todo!("AC-003: DtuType::Armis must start without error via the exhaustive match dispatch in start_clone")
}

/// BC-3.5.001 postcondition 1 / AC-003:
/// `DtuType::CrowdStrike` dispatches through the generic stub path.
/// `HarnessBuilder` start succeeds for a CrowdStrike-only spec.
#[tokio::test]
async fn test_BC_3_5_001_CR004_crowdstrike_clone_starts_via_exhaustive_match() {
    todo!("AC-003: DtuType::CrowdStrike must start without error via the exhaustive match dispatch in start_clone")
}

/// BC-3.5.001 postcondition 1 / AC-003:
/// `DtuType::Cyberint` dispatches through the generic stub path.
/// `HarnessBuilder` start succeeds for a Cyberint-only spec.
#[tokio::test]
async fn test_BC_3_5_001_CR004_cyberint_clone_starts_via_exhaustive_match() {
    todo!("AC-003: DtuType::Cyberint must start without error via the exhaustive match dispatch in start_clone")
}

/// BC-3.5.001 postcondition 1 / AC-003:
/// MSSP Coordination type `DtuType::Slack` dispatches through the shared-mode path.
/// `HarnessBuilder` start succeeds for a Slack-only spec.
#[tokio::test]
async fn test_BC_3_5_001_CR004_slack_clone_starts_via_exhaustive_match() {
    todo!("AC-003: DtuType::Slack must start without error via the exhaustive match dispatch in start_clone")
}

/// BC-3.5.001 postcondition 1 / AC-003:
/// MSSP Coordination type `DtuType::PagerDuty` dispatches through the shared-mode path.
#[tokio::test]
async fn test_BC_3_5_001_CR004_pagerduty_clone_starts_via_exhaustive_match() {
    todo!("AC-003: DtuType::PagerDuty must start without error via the exhaustive match dispatch in start_clone")
}

/// BC-3.5.001 postcondition 1 / AC-003:
/// MSSP Coordination type `DtuType::Jira` dispatches through the shared-mode path.
#[tokio::test]
async fn test_BC_3_5_001_CR004_jira_clone_starts_via_exhaustive_match() {
    todo!("AC-003: DtuType::Jira must start without error via the exhaustive match dispatch in start_clone")
}

// ===========================================================================
// AC-007: network-mode harness dispatch is also correct (BC-3.5.002)
// ===========================================================================

/// BC-3.5.002 postcondition 1 / AC-007:
/// In `IsolationMode::Network`, `DtuType::Armis` dispatches to the Armis-specific
/// path. Both isolation modes use the same `start_clone` dispatch code path.
#[tokio::test]
async fn test_BC_3_5_002_CR004_armis_network_mode_dispatch_is_correct() {
    todo!(
        "AC-007: DtuType::Armis in IsolationMode::Network must reach the Armis-specific clone path"
    )
}

/// BC-3.5.002 postcondition 1 / AC-007:
/// In `IsolationMode::Network`, `DtuType::CrowdStrike` dispatches through the
/// generic stub path without error.
#[tokio::test]
async fn test_BC_3_5_002_CR004_crowdstrike_network_mode_dispatch_is_correct() {
    todo!("AC-007: DtuType::CrowdStrike in IsolationMode::Network must start without error via exhaustive match")
}
