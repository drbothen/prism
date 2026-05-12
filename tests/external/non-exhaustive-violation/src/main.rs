//! AC-5 (S-PLUGIN-PREREQ-C) — #[non_exhaustive] compile-fail test.
//!
//! Named: `test_BC_2_01_013_non_exhaustive_sensor_spec_no_external_literal`
//!
//! This file attempts struct-literal construction or exhaustive match of all pub
//! TOML-deserialized types in `prism-spec-engine` from OUTSIDE the crate.
//! Once `#[non_exhaustive]` is applied to each type (AC-5 + fix-burst-2), every
//! expression here must fail with:
//!   E0639: cannot create non-exhaustive struct with a struct expression
//!   E0004: non-exhaustive patterns (for enums matched without wildcard)
//!
//! RED GATE: Before AC-5, none of the types carry `#[non_exhaustive]`.
//! Struct-literal construction succeeds and THIS CRATE COMPILES (cargo check exits 0).
//! The Red Gate is: running `cargo check -p non-exhaustive-violation` exits 0 BEFORE
//! AC-5 is implemented, but the expected behaviour is exit non-zero.
//!
//! GREEN: After AC-5 + fix-burst-2, `#[non_exhaustive]` is applied to all 29 types.
//! `cargo check -p non-exhaustive-violation` exits non-zero with >=29 E0639/E0004 errors.
//!
//! Target types (all 29 — AC-5 original 14 + fix-burst-2 sibling sweep 15):
//!
//! Original 14 (fix-burst-1):
//!   1.  CredentialRef               — struct, spec_parser.rs
//!   2.  SensorSpec                  — struct, spec_parser.rs
//!   3.  SensorTableDescriptor       — struct, spec_parser.rs
//!   4.  FetchStep                   — struct, spec_parser.rs
//!   5.  ColumnSpec                  — struct, spec_parser.rs
//!   6.  TableSpec                   — struct, spec_parser.rs
//!   7.  PaginationConfig            — enum, spec_parser.rs (match without wildcard)
//!   8.  AuthType                    — enum, spec_parser.rs (match without wildcard)
//!   9.  RateLimitHints              — struct, spec_parser.rs
//!   10. types::SensorTableDescriptor — struct, types.rs
//!   11. types::CredentialRef        — struct, types.rs
//!   12. infusion::CredentialRef     — struct, infusion/mod.rs
//!   13. prism_core::ColumnType      — enum (match without wildcard)
//!   14. prism_core::ColumnOptions   — enum (match without wildcard)
//!
//! Sibling sweep 15 (fix-burst-2, F-LP2-HIGH-001):
//!   15. BatchMode              — enum, write_endpoint.rs (match without wildcard)
//!   16. WriteStep              — struct, write_endpoint.rs
//!   17. WriteEndpointSpec      — struct, write_endpoint.rs
//!   18. InfusionType           — enum, infusion/mod.rs (match without wildcard)
//!   19. BuiltInSourceType      — enum, infusion/mod.rs (match without wildcard)
//!   20. InfusionSourceConfig   — struct, infusion/mod.rs
//!   21. InfusionField          — struct, infusion/mod.rs
//!   22. PipeStageConfig        — struct, infusion/mod.rs
//!   23. PluginConfig           — struct, infusion/mod.rs
//!   24. InfusionSpec           — struct, infusion/mod.rs
//!   25. types::ColumnType      — enum, types.rs (match without wildcard)
//!   26. types::ColumnDef       — struct, types.rs
//!   27. types::PaginationType  — enum, types.rs (match without wildcard)
//!   28. types::SpecStatus      — enum, types.rs (match without wildcard)
//!   29. types::ClientStatus    — enum, types.rs (match without wildcard)
//!
//! Structure: violations are split across submodules (separate compile units) so that
//! rustc's per-function error budget does not suppress later violations. The CI script
//! counts all E0639/E0004 errors across the entire compilation output.
//!
//! CI run: `cargo check -p non-exhaustive-violation`
//! Expected: FAIL (non-zero) after AC-5 implementation.
//! Currently (Red Gate): PASS (zero) = Red Gate condition met.

mod enum_violations;
mod struct_violations;

fn main() {
    // Compilation will fail before reaching here.
    // main() exists only so this compiles as a binary target.
}
