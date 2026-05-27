//! Cyberint adapter integration tests — superseded by spec-catalog dispatch parity tests.
//!
//! As of PLUGIN-MIGRATION-001-A (AC-003, AC-006), the `CyberintAdapter` and `CyberintAuth`
//! types have been deleted from `prism-sensors`. Sensors now run exclusively via
//! TOML specs + WASM plugins through the spec engine (ADR-023, ADR-028 §D10).
//!
//! The parity tests in `crates/prism-spec-engine/tests/parity/` exercise equivalent
//! behavior via the spec-driven plugin path (VP-PLUGIN-003 co-merge contract satisfied).
//!
//! Story: PLUGIN-MIGRATION-001-A | BC: BC-2.01.006, BC-2.01.013, BC-2.01.016
