//! Test modules for `prism-ocsf`.
//!
//! Each submodule maps to one or more behavioral contracts.
//!
//! # Naming Convention
//!
//! Tests follow the VSDD `test_BC_S_SS_NNN_xxx()` naming pattern for traceability.
//! The `non_snake_case` lint is suppressed in this module because the BC-derived
//! uppercase identifiers are intentional and required by the factory protocol.
#![allow(non_snake_case)]

pub mod bc_2_02_001_pool;
pub mod bc_2_02_002_normalizer;
pub mod bc_2_02_009_version;
pub mod bc_2_02_010_enum_map;
pub mod bc_2_02_012_class_selector;
pub mod proptest_normalizer;
