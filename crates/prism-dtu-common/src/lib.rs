//! `prism-dtu-common` — Shared infrastructure for Prism DTU behavioral clone crates.
//!
//! Provides the [`BehavioralClone`] trait, configurable latency and failure injection
//! middleware ([`LatencyLayer`], [`FailureLayer`]), a fixture loader, a generic
//! [`SyslogReceiver`], a generic [`WebhookReceiver`], and shared assertion utilities.
//! All per-surface DTU clone crates build on this foundation (currently 6; target: 13 when
//! S-6.11–S-6.13, S-6.16–S-6.19 land).
//!
//! This crate is gated behind `#[cfg(any(test, feature = "dtu"))]` and must NEVER
//! link into a production binary.
#![cfg(any(test, feature = "dtu"))]

pub mod clone;
pub mod config;
pub mod fidelity;
pub mod fixture;
pub mod layers;
pub mod seed;
pub mod syslog;
pub mod test_utils;
pub mod webhook;

pub use clone::BehavioralClone;
pub use config::{FailureMode, StubConfig};
pub use fidelity::{FidelityCheck, FidelityFailure, FidelityReport, FidelityValidator};
pub use fixture::{load_fixture, load_fixture_as};
pub use layers::{FailureLayer, FailureLayerShared, LatencyLayer};
pub use seed::seeded_rng;
pub use syslog::SyslogReceiver;
pub use test_utils::{
    assert_field_present, assert_header_present, assert_status, build_test_client,
};
pub use webhook::{CapturedRequest, WebhookReceiver};
