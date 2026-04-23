//! Test suite for prism-ocsf — OCSF schema loading (S-1.04) + field mapping (S-1.05).
//!
//! # BC Coverage
//!
//! S-1.04 tests:
//!   - BC-2.02.001 → bc_2_02_001_pool.rs
//!   - BC-2.02.002 → bc_2_02_002_normalizer.rs
//!   - BC-2.02.009 → bc_2_02_009_version.rs
//!   - BC-2.02.010 → bc_2_02_010_enum_map.rs
//!   - BC-2.02.012 → bc_2_02_012_class_selector.rs
//!   - VP-016     → proptest_normalizer.rs
//!
//! S-1.05 tests:
//!   - BC-2.02.003 → mapper_tests.rs (AC-1, AC-2)
//!   - BC-2.02.004 → mapper_tests.rs (AC-3, AC-4)
//!   - BC-2.02.005 → mapper_tests.rs (AC-5)
//!   - BC-2.02.006 → mapper_tests.rs (AC-6)
//!   - BC-2.02.007 → mapper_tests.rs (AC-7)
//!   - BC-2.02.008 → alias_tests.rs  (AC-8)
//!   - BC-2.02.011 → mapper_tests.rs (AC-9)
//!   - VP-017     → proptest_extensions.rs (AC-10)
#![allow(non_snake_case)]

// S-1.04 test modules
pub mod bc_2_02_001_pool;
pub mod bc_2_02_002_normalizer;
pub mod bc_2_02_009_version;
pub mod bc_2_02_010_enum_map;
pub mod bc_2_02_012_class_selector;
pub mod proptest_normalizer;

// S-1.05 test modules
pub mod alias_tests;
pub mod mapper_tests;
pub mod proptest_extensions;
