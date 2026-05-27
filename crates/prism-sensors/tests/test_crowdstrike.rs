//! CrowdStrike adapter integration tests — superseded by spec-catalog dispatch parity tests.
//!
//! As of PLUGIN-MIGRATION-001-A (AC-006), the `CrowdStrikeAdapter` and `CrowdStrikeAuth`
//! types have been deleted from `prism-sensors`. Sensors now run exclusively via
//! TOML specs + WASM plugins through the spec engine (ADR-023, ADR-028 §D10).
//!
//! The AC-006 deletion gate was satisfied when PLUGIN-MIGRATION-001-E (PR #154,
//! develop@6bf3f659) merged, confirming the spec-driven OAuth2 refresh-on-401
//! WASM plugin achieves behavioral parity.
//!
//! The parity tests in `crates/prism-spec-engine/tests/parity/` exercise equivalent
//! behavior via the spec-driven plugin path (VP-PLUGIN-003 co-merge contract satisfied).
//!
//! Story: PLUGIN-MIGRATION-001-A | BC: BC-2.01.005, BC-2.01.013, BC-2.01.016
