//! `prism-dtu-threatintel` — L2-stateful behavioral clone of the Threat Intel Aggregator.
//!
//! Exposes unified IP/domain/hash lookup endpoints returning aggregated threat scores.
//! Fixture registry maps known lookups to canned responses covering malicious, benign,
//! and unknown scenarios. Gated behind `feature = "dtu"`.
#![cfg(any(test, feature = "dtu"))]
#![allow(unused_variables, dead_code)]

pub mod clone;
pub mod routes;
pub mod state;
pub mod types;

pub use clone::ThreatIntelClone;
