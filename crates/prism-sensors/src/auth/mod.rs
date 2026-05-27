//! `SensorAuth` open trait — plugin-implementable auth contract.
//!
//! # Open Trait (S-PLUGIN-PREREQ-E / BC-2.01.016)
//!
//! As of S-PLUGIN-PREREQ-E, `SensorAuth` is no longer sealed. External crates
//! (including `.prx` WASM plugins) may implement this trait to provide custom
//! auth strategies. Runtime cross-composition enforcement (E-SPEC-012/013/014
//! per ADR-023 Rule 2 / ADR-026 D3) replaces the previous compile-time sealed
//! trait guard.
//!
//! All four built-in per-sensor auth subtypes (CrowdStrike, Cyberint, Claroty, Armis)
//! were deleted in PLUGIN-MIGRATION-001-A after VP-PLUGIN-003 parity tests confirmed
//! the spec-driven plugin path is behaviorally equivalent (ADR-028 §D10 co-merge
//! contract satisfied). Sensors now run exclusively via TOML specs + WASM plugins.
//!
//! Story: S-2.06 (initial) | S-PLUGIN-PREREQ-E (unsealing) | PLUGIN-MIGRATION-001-A (deletion)
//! BC: BC-2.01.013, BC-2.01.016

// ---------------------------------------------------------------------------
// SensorAuth open trait
// ---------------------------------------------------------------------------

/// Open authentication credential trait for a sensor adapter.
///
/// As of S-PLUGIN-PREREQ-E (BC-2.01.016 + ADR-026), the sealed marker has been
/// removed. External crates (including `.prx` WASM plugins) may implement this
/// trait to register custom auth strategies. Runtime cross-composition rules
/// (E-SPEC-012/013/014) enforce safe usage at spec-load time (ADR-023 Rule 2).
///
/// Credentials MUST NOT appear in `Debug` output or log output at any level
/// (AI-opaque credential model, AD-017).
///
/// Story: S-2.06 (initial) | S-PLUGIN-PREREQ-E (unsealing) | BC: BC-2.01.013, BC-2.01.016
pub trait SensorAuth: Send + Sync + 'static {
    /// Returns `self` as `&dyn std::any::Any` to enable downcasting in adapters.
    ///
    /// Adapters receive `&dyn SensorAuth` but need access to concrete credential
    /// fields (e.g., `client_id`, `api_key`). `as_any()` allows safe downcasting
    /// to the concrete type using `downcast_ref::<ConcreteAuthType>()`.
    fn as_any(&self) -> &dyn std::any::Any;

    /// Returns the canonical auth-type discriminator string for this implementation.
    ///
    /// Must return one of the closed enumeration values defined in ADR-026 §D3:
    /// `"oauth2_client_credentials"`, `"bearer_static"`, `"cookie_roundtrip"`,
    /// `"api_key"`, `"custom_via_plugin"`.
    ///
    /// Used by the runtime E-SPEC-012 validator (BC-2.01.016 Rule 2 / ADR-023 Rule 2, Rule A)
    /// to enforce that declared `auth_type` matches the resolved credential's structural shape.
    ///
    /// Story: S-PLUGIN-PREREQ-E AC-1/AC-2 | BC: BC-2.01.016 | ADR-026 §D1/D2 Path B
    fn auth_type_name(&self) -> &'static str;
}

// ---------------------------------------------------------------------------
// Tests -- BC-2.01.016 (post-deletion state: all 4 built-in impls deleted)
// ---------------------------------------------------------------------------
#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use super::*;

    // -----------------------------------------------------------------------
    // Test 1 -- test_BC_2_01_016_001_sensor_auth_external_impl_compiles
    // AC-1: SensorAuth is externally implementable (no sealed supertrait).
    // -----------------------------------------------------------------------

    /// Local struct defined in test scope — simulates an external (plugin) impl.
    /// If `SensorAuth` still carried a sealed supertrait bound, this would fail
    /// to compile. After PREREQ-E Task 1, it must compile cleanly.
    #[allow(dead_code)]
    struct TestExternalAuth {
        name: String,
    }

    impl SensorAuth for TestExternalAuth {
        fn as_any(&self) -> &dyn std::any::Any {
            self
        }

        fn auth_type_name(&self) -> &'static str {
            "custom_via_plugin"
        }
    }

    /// BC-2.01.016 AC-1: An externally-defined struct can implement `SensorAuth`.
    ///
    /// After PLUGIN-MIGRATION-001-A all four built-in auth impls are deleted.
    /// This test verifies that the trait remains externally implementable and
    /// that `custom_via_plugin` dispatches correctly via `dyn SensorAuth`.
    ///
    /// Story: S-PLUGIN-PREREQ-E AC-1 / AC-2 | BC: BC-2.01.016 | ADR-026 §D1 Path B
    #[test]
    fn test_BC_2_01_016_001_sensor_auth_external_impl_compiles() {
        // External impl must compile and dispatch correctly via dyn SensorAuth.
        let external: Box<dyn SensorAuth> = Box::new(TestExternalAuth {
            name: "plugin_sensor".to_string(),
        });
        assert_eq!(
            external.auth_type_name(),
            "custom_via_plugin",
            "external SensorAuth impl must return its declared auth_type_name"
        );
    }

    // -----------------------------------------------------------------------
    // Test 3 -- test_BC_2_01_016_003_auth_type_name_valid_discriminator_post_deletion
    //
    // The four built-in auth impls (CrowdStrike, Cyberint, Claroty, Armis) were
    // deleted in PLUGIN-MIGRATION-001-A (AC-003, AC-006). Their corrected
    // auth_type_name() values (ADR-028 §D2/§D6) were committed in the wip commit
    // immediately before deletion — the correction is permanently observable in
    // git history per ADR-028 §D4/§D6 Action 2.
    //
    // Corrected values before deletion:
    //   - CrowdStrikeAuth: "oauth2_client_credentials" (unchanged, ADR-028 §D6)
    //   - CyberintAuth:    "cookie_roundtrip" (corrected from "bearer_static")
    //   - ClarotyAuth:     "bearer_static"    (corrected from "cookie_roundtrip")
    //   - ArmisAuth:       "bearer_static"    (corrected from "api_key")
    //
    // Post-deletion this test verifies only that the SensorAuth trait itself
    // remains consistent with its contract (external impls can return valid values).
    //
    // Story: PLUGIN-MIGRATION-001-A AC-002/AC-003/AC-006 | BC: BC-2.01.016 INV-AUTH-OPEN-002
    // -----------------------------------------------------------------------

    /// BC-2.01.016 AC-2 (post-deletion): auth_type_name correctness.
    ///
    /// The four built-in auth impls were deleted in PLUGIN-MIGRATION-001-A after
    /// their auth_type_name() values were corrected to DTU-grounded values per
    /// ADR-028 §D2/§D6 (observable in the wip commit before deletion).
    ///
    /// This test verifies the trait contract remains consistent for external impls.
    ///
    /// Story: PLUGIN-MIGRATION-001-A AC-002 | BC: BC-2.01.016 | ADR-028 §D2/§D6
    #[test]
    fn test_BC_2_01_016_003_auth_type_name_valid_discriminator_post_deletion() {
        // All four built-in impls deleted in PLUGIN-MIGRATION-001-A (AC-003, AC-006).
        // Their corrected auth_type_name() values are recorded in the wip commit
        // before deletion:
        //   - CrowdStrikeAuth: "oauth2_client_credentials" (ADR-028 §D6: unchanged)
        //   - CyberintAuth:    "cookie_roundtrip"           (ADR-028 §D2 correction)
        //   - ClarotyAuth:     "bearer_static"              (ADR-028 §D2 correction)
        //   - ArmisAuth:       "bearer_static"              (ADR-028 §D2 correction)
        //
        // Verify the open trait itself continues to work for plugin-provided impls.
        let plugin_auth: Box<dyn SensorAuth> = Box::new(TestExternalAuth {
            name: "crowdstrike".to_string(),
        });
        // Plugin impls may return any of the valid discriminator values.
        let type_name = plugin_auth.auth_type_name();
        assert!(
            matches!(
                type_name,
                "oauth2_client_credentials"
                    | "bearer_static"
                    | "cookie_roundtrip"
                    | "api_key"
                    | "custom_via_plugin"
            ),
            "auth_type_name must return a valid discriminator value; got: {type_name:?}"
        );
    }

    // -----------------------------------------------------------------------
    // AC-007: No orphan re-export symbols after deletion
    //
    // The deleted symbols (CyberintAuth, ClarotyAuth, ArmisAuth, CrowdStrikeAuth,
    // CyberintAdapter, ClarotyAdapter, ArmisAdapter, CrowdStrikeAdapter) must NOT
    // be accessible from the prism-sensors public API after deletion.
    //
    // AC-007 grep verification command:
    //   grep -rn "CyberintAuth\|ClarotyAuth\|ArmisAuth\|CrowdStrikeAuth\
    //     \|CyberintAdapter\|ClarotyAdapter\|ArmisAdapter\|CrowdStrikeAdapter\
    //     \|paginate_claroty" crates/prism-sensors/src/ crates/prism-bin/src/
    // Expected: ZERO matches in production source files.
    // -----------------------------------------------------------------------
}
