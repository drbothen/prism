//! WGS-W2-002 SecretString tests — superseded by adapter deletion.
//!
//! As of PLUGIN-MIGRATION-001-A (AC-003, AC-006), the `ArmisAdapter`, `ClarotyAdapter`,
//! and `CrowdStrikeAdapter` types tested here have been deleted from `prism-sensors`.
//!
//! The WGS-W2-002 fix (bearer tokens wrapped in `SecretString`) was committed to git
//! history in the wip commit immediately before deletion per ADR-028 §D4/§D6 Action 2.
//! The fix is permanently observable in git history for audit purposes.
//!
//! The `SecretString` bearer-token wrapping pattern is enforced in the WASM plugin
//! path via `SensorAuth::auth_type_name()` contract and plugin SDK credential binding
//! (BC-2.01.016, ADR-026 §D1). New plugin-provided adapters are subject to the same
//! credential-opacity requirement (AD-017).
//!
//! Security fix: WGS-W2-002 | BC: BC-2.01.005 / BC-2.01.008 / BC-2.01.013
//! Story: PLUGIN-MIGRATION-001-A | AC-003, AC-006
