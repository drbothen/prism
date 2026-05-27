//! Integration test root for prism-sensors.
//!
//! As of PLUGIN-MIGRATION-001-A, the four per-sensor adapter test modules
//! (test_armis, test_claroty, test_crowdstrike, test_cyberint) have been
//! superseded by spec-catalog dispatch parity tests in `prism-spec-engine/tests/parity/`.
//! They are no longer declared here (AC-003, AC-006 of PLUGIN-MIGRATION-001-A).
//!
//! Retained sub-modules cover infrastructure that remains in `prism-sensors`:
//! - `test_pagination`: `paginate_claroty()` stream and `OffsetCursor` (S-2.07)
//! - `test_timestamp`: multi-format timestamp parsing (S-2.07)
//!
//! Story: PLUGIN-MIGRATION-001-A | BC: BC-2.01.013, BC-2.01.016

mod test_pagination;
mod test_timestamp;
