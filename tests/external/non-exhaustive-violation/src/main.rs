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
//! GREEN: After AC-5 + fix-burst-2 + fix-burst-4 + F-LP22 + post-PREREQ-E +
//!   PLUGIN-MIGRATION-001-E + S-CONFIG-MULTI-TENANT-OVERRIDE-001 + S-5.01-FOLLOWUP-MCP-BOOT +
//!   S-DEMO-001, `#[non_exhaustive]` is applied to all 49 types.
//! `cargo check -p non-exhaustive-violation` exits non-zero with >=49 E0639/E0004 errors.
//!
//! Target types (all 36 — AC-5 original 14 + fix-burst-2 sibling sweep 15 + fix-burst-4
//!   types::SensorSpec + F-LP22 PluginError + post-PREREQ-E WriteToolInvalidationMap +
//!   PLUGIN-MIGRATION-001-E LoadedPlugin + S-CONFIG-MULTI-TENANT-OVERRIDE-001 overlay types):
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
//! fix-burst-4 sibling (F-LP5-LOW-001):
//!   30. types::SensorSpec      — struct, types.rs
//!
//! F-LP22 (D-572):
//!   31. prism_core::PluginError — enum, prism-core/src/error.rs (match without wildcard)
//!
//! post-PREREQ-E cleanup (pr-pass-4 OBS):
//!   32. prism_query::invalidation::WriteToolInvalidationMap — struct, invalidation.rs
//!
//! PLUGIN-MIGRATION-001-E (kv_store field addition):
//!   33. prism_spec_engine::plugin::LoadedPlugin — struct, plugin/loader.rs
//!
//! S-CONFIG-MULTI-TENANT-OVERRIDE-001 (ADR-029 per-org overlay types):
//!   34. prism_spec_engine::overlay::SensorInstanceOverlay — struct, overlay.rs
//!   35. prism_spec_engine::overlay::OverlayProvenance     — struct, overlay.rs
//!   36. prism_spec_engine::overlay::ResolvedSensorSpec    — struct, overlay.rs
//!
//! S-5.01-FOLLOWUP-MCP-BOOT (prism-mcp pub API surface types):
//!   37. prism_mcp::safety_envelope::ResponseMeta          — struct, safety_envelope.rs
//!   38. prism_mcp::safety_envelope::ContentEntry          — struct, safety_envelope.rs
//!   39. prism_mcp::safety_envelope::StructuredContent     — struct, safety_envelope.rs
//!   40. prism_mcp::safety_envelope::ResponseEnvelope      — struct, safety_envelope.rs
//!   41. prism_mcp::safety_envelope::SafetyFlagSchema      — struct, safety_envelope.rs
//!   42. prism_mcp::safety_envelope::MetaEnvelopeSchemaType — struct, safety_envelope.rs
//!   43. prism_mcp::safety_envelope::ResponseEnvelopeSchema — struct, safety_envelope.rs
//!   44. prism_mcp::safety_envelope::DataSource            — enum, safety_envelope.rs (match without wildcard)
//!   45. prism_mcp::tool_registry::ToolRegistration        — struct, tool_registry.rs
//!
//! S-DEMO-001 (prism-bin pub API types — CR-001, CR-006):
//!   48. prism_bin::spec_driven_adapter::AdapterAuthStrategy      — enum (match without wildcard)
//!   49. prism_bin::spec_driven_adapter::BearerStaticAuthProvider — struct
//!   50. prism_bin::spec_driven_adapter::SpecDrivenSensorAdapter  — struct
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
