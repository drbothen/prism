//! Test suite for prism-ocsf S-1.05 — field mapping and normalization.
//!
//! All tests MUST FAIL before S-1.05 implementation begins (Red Gate).
//!
//! BC coverage:
//!   - BC-2.02.003 → mapper_tests.rs (AC-1, AC-2)
//!   - BC-2.02.004 → mapper_tests.rs (AC-3, AC-4)
//!   - BC-2.02.005 → mapper_tests.rs (AC-5)
//!   - BC-2.02.006 → mapper_tests.rs (AC-6)
//!   - BC-2.02.007 → mapper_tests.rs (AC-7)
//!   - BC-2.02.008 → alias_tests.rs  (AC-8)
//!   - BC-2.02.011 → mapper_tests.rs (AC-9)
//! VP coverage:
//!   - VP-017 → proptest_extensions.rs (AC-10)

pub mod alias_tests;
pub mod mapper_tests;
pub mod proptest_extensions;
