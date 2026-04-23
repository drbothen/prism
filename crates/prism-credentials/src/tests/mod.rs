//! Test modules for prism-credentials (S-1.06).
//!
//! All tests in this module are expected to FAIL at Red Gate (before
//! implementation). They are either:
//!   - Calling unimplemented!() stubs (panics with "unimplemented")
//!   - asserting postconditions that the stubs do not yet satisfy

pub mod proptest_crypto;
pub mod store_tests;
