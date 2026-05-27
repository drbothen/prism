//! Claroty adapter integration tests — superseded by spec-catalog dispatch parity tests.
//!
//! As of PLUGIN-MIGRATION-001-A (AC-003, AC-006), the `ClarotyAdapter`, `ClarotyAuth`,
//! and `ClarotyId` types have been deleted from `prism-sensors`. Sensors now run
//! exclusively via TOML specs + WASM plugins through the spec engine (ADR-023, ADR-028 §D10).
//!
//! The parity tests in `crates/prism-spec-engine/tests/parity/` exercise equivalent
//! behavior via the spec-driven plugin path (VP-PLUGIN-003 co-merge contract satisfied).
//!
//! `ClarotyId` deserialization tests are re-exercised via the plugin normalization layer.
//! `paginate_claroty()` tests remain in `test_pagination.rs` (the function is retained
//! in `prism-sensors::pagination` for potential WASM plugin use).
//!
//! Story: PLUGIN-MIGRATION-001-A | BC: BC-2.01.004, BC-2.01.007, BC-2.01.013, BC-2.01.016
