//! `prism-dtu-common` — Shared infrastructure for Prism DTU behavioral clone crates.
//!
//! Provides the [`BehavioralClone`] trait, configurable latency and failure injection
//! middleware ([`LatencyLayer`], [`FailureLayer`]), a fixture loader, a generic
//! [`SyslogReceiver`], a generic [`WebhookReceiver`], and shared assertion utilities.
//! All per-surface DTU clone crates build on this foundation (currently 6; target: 13 when
//! S-6.11–S-6.13, S-6.16–S-6.19 land).
//!
//! This crate is gated behind `#[cfg(any(test, feature = "dtu", feature = "fixture-gen"))]`
//! and must NEVER link into a production binary.
//!
//! The `generator` module is additionally gated behind `#[cfg(feature = "fixture-gen")]`
//! (AC-007 / D-056).
#![cfg(any(test, feature = "dtu", feature = "fixture-gen"))]

#[cfg(any(test, feature = "dtu"))]
pub mod clone;
#[cfg(any(test, feature = "dtu"))]
pub mod config;
#[cfg(any(test, feature = "dtu"))]
pub mod fidelity;
#[cfg(any(test, feature = "dtu"))]
pub mod fixture;
#[cfg(feature = "fixture-gen")]
pub mod generator;
#[cfg(any(test, feature = "dtu"))]
pub mod layers;
#[cfg(any(test, feature = "dtu"))]
pub mod seed;
#[cfg(any(test, feature = "dtu"))]
pub mod syslog;
#[cfg(any(test, feature = "dtu"))]
pub mod test_utils;
#[cfg(any(test, feature = "dtu"))]
pub mod webhook;

#[cfg(any(test, feature = "dtu"))]
pub use clone::BehavioralClone;
#[cfg(any(test, feature = "dtu"))]
pub use config::{FailureMode, StubConfig};
#[cfg(any(test, feature = "dtu"))]
pub use fidelity::{FidelityCheck, FidelityFailure, FidelityReport, FidelityValidator};
#[cfg(any(test, feature = "dtu"))]
pub use fixture::{load_fixture, load_fixture_as};
#[cfg(feature = "fixture-gen")]
pub use generator::{
    all_archetypes, apply_overrides, default_page_size, seeded_rng as gen_seeded_rng, Archetype,
    FixtureSet, GenOpts, GenOptsError, OrgId, Provenance,
};
#[cfg(any(test, feature = "dtu"))]
pub use layers::{FailureLayer, FailureLayerShared, LatencyLayer};
#[cfg(any(test, feature = "dtu"))]
pub use seed::seeded_rng;
#[cfg(any(test, feature = "dtu"))]
pub use syslog::SyslogReceiver;
#[cfg(any(test, feature = "dtu"))]
pub use test_utils::{
    assert_field_present, assert_header_present, assert_status, build_test_client,
};
#[cfg(any(test, feature = "dtu"))]
pub use webhook::{CapturedRequest, WebhookReceiver};
