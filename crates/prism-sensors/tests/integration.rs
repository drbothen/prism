//! Integration test root for S-2.07 per-sensor auth adapters.
//!
//! Test bodies are provided by the Test Writer agent (next dispatch).
//! This file declares the sub-modules so `cargo check --tests` validates
//! the structure without requiring any test bodies yet.
//!
//! Story: S-2.07 | BC: BC-2.01.004–BC-2.01.008

mod test_armis;
mod test_claroty;
mod test_crowdstrike;
mod test_cyberint;
mod test_pagination;
mod test_timestamp;
