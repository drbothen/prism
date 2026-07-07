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
//! S-1.05 tests (post-PLUGIN-MIGRATION-001-C):
//!   - BC-2.02.003–006, BC-2.02.007, BC-2.02.011 → covered by SpecDrivenMapper
//!     integration tests in spec_driven_mapper_fixtures.rs (external test crate)
//!     and by proptest_extensions.rs (VP-017, AC-10)
//!   - BC-2.02.008 → alias_tests.rs  (AC-8)
//!   - VP-017     → proptest_extensions.rs (AC-10)
//!
//! Note: mapper_tests.rs (per-sensor hardcoded mapper unit tests) was removed in
//! PLUGIN-MIGRATION-001-C when the four per-sensor modules were deleted and replaced
//! by SpecDrivenMapper.
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
pub mod proptest_extensions;

// S-PRISMQL-CASE-INSENSITIVE-001: Red Gate tests RG-019, RG-020, RG-021 —
// OcsfEnumMap::normalize_label case-insensitive adapter boundary normalization.
pub mod test_adapter_normalization;
