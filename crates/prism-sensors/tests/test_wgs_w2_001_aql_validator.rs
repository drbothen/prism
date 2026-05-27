//! WGS-W2-001 AQL validator tests — superseded by adapter deletion.
//!
//! As of PLUGIN-MIGRATION-001-A (AC-003, AC-006), the `ArmisAdapter`, `ArmisAuth`,
//! `validate_aql()`, and `AqlValidationError` types tested here have been deleted
//! from `prism-sensors`.
//!
//! The WGS-W2-001 AQL injection mitigation (`validate_aql()` + `build_aql()` pre-wire
//! rejection per ADR-005) was committed to git history in the wip commit immediately
//! before deletion per ADR-028 §D4/§D6 Action 2.
//!
//! AQL validation in the WASM plugin path is enforced at spec-load time via
//! E-SPEC-012/013/014 validators (ADR-023 Rule 2, ADR-026 §D3). Plugin-provided
//! adapters that accept user-supplied query parameters are subject to the same
//! injection-prevention requirements.
//!
//! Security fix: WGS-W2-001 | ADR: ADR-005 | BC: BC-2.01.008 (TV-006)
//! Story: PLUGIN-MIGRATION-001-A | AC-003, AC-006
