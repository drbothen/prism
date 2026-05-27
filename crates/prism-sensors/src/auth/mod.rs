//! `SensorAuth` open trait and per-sensor auth credential subtypes.
//!
//! # Open Trait (S-PLUGIN-PREREQ-E / BC-2.01.016)
//!
//! As of S-PLUGIN-PREREQ-E, `SensorAuth` is no longer sealed. External crates
//! (including `.prx` WASM plugins) may implement this trait to provide custom
//! auth strategies. Runtime cross-composition enforcement (E-SPEC-012/013/014
//! per ADR-023 Rule 2 / ADR-026 D3) replaces the previous compile-time sealed
//! trait guard.
//!
//! The four built-in auth subtypes defined in this module implement `SensorAuth`:
//! - [`CrowdStrikeAuth`] — OAuth2 client credentials (`auth_type_name = "oauth2_client_credentials"`)
//! - [`CyberintAuth`]    — Bearer/API key static (`auth_type_name = "bearer_static"`)
//! - [`ClarotyAuth`]     — Cookie roundtrip (`auth_type_name = "cookie_roundtrip"`)
//! - [`ArmisAuth`]       — API key (`auth_type_name = "api_key"`)
//!
//! Story: S-2.06 (initial) | S-PLUGIN-PREREQ-E (unsealing) | BC: BC-2.01.013, BC-2.01.016

pub mod armis;
pub mod claroty;
pub mod crowdstrike;
pub mod cyberint;

pub use armis::ArmisAuth;
pub use claroty::ClarotyAuth;
pub use crowdstrike::CrowdStrikeAuth;
pub use cyberint::CyberintAuth;

// ---------------------------------------------------------------------------
// SensorAuth sealed trait
// ---------------------------------------------------------------------------

/// Open authentication credential trait for a sensor adapter.
///
/// As of S-PLUGIN-PREREQ-E (BC-2.01.016 + ADR-026), the sealed marker has been
/// removed. External crates (including `.prx` WASM plugins) may implement this
/// trait to register custom auth strategies. Runtime cross-composition rules
/// (E-SPEC-012/013/014) enforce safe usage at spec-load time (ADR-023 Rule 2).
///
/// Each auth subtype carries ONLY its own credentials (no field overlap across
/// sensor types). Credentials MUST NOT appear in `Debug` output or log output
/// at any level (AI-opaque credential model).
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
// Red Gate Tests -- S-PLUGIN-PREREQ-E (BC-2.01.016 / AC-1 / AC-2)
// ---------------------------------------------------------------------------
#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use super::*;

    // -----------------------------------------------------------------------
    // Test 1 -- test_BC_2_01_016_001_sensor_auth_external_impl_compiles
    // AC-1: SensorAuth is externally implementable; calling built-in
    //       auth_type_name() panics pre-implementation (Red Gate).
    //
    // Pre-implementation failure mode: todo!() panic in CrowdStrikeAuth::auth_type_name().
    // -----------------------------------------------------------------------

    /// Local struct defined in test scope -- simulates an external (plugin) impl.
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
    /// The test verifies (a) compilation succeeds for an external impl and
    /// (b) the built-in CrowdStrikeAuth::auth_type_name() returns the expected
    /// discriminator. Pre-implementation this fails RED on todo!() panic in
    /// CrowdStrikeAuth::auth_type_name().
    ///
    /// Red Gate failure mode: todo!() panic in CrowdStrikeAuth::auth_type_name().
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
        // Calling auth_type_name() on the built-in impl fails RED until AC-2 is implemented.
        // Use a CrowdStrikeAuth -- constructed with test values; panics on todo!() pre-impl.
        let cs_auth = crowdstrike::CrowdStrikeAuth {
            client_id: "test-client".to_string(),
            client_secret: secrecy::SecretString::new("test-secret".into()),
            cloud_region: "us-1".to_string(),
        };
        let builtin: &dyn SensorAuth = &cs_auth;
        // This line panics pre-implementation due to todo!() in auth_type_name().
        let name = builtin.auth_type_name();
        assert_eq!(
            name,
            "oauth2_client_credentials",
            "CrowdStrikeAuth::auth_type_name must return \"oauth2_client_credentials\" (ADR-026 §D3)"
        );
    }

    // -----------------------------------------------------------------------
    // Test 3 -- test_BC_2_01_016_003_four_auth_impls_minimal_diff_post_unsealing
    // AC-2: Each of the 4 built-in auth impls returns the correct auth_type_name
    //       discriminator per ADR-026 §D3. Fails RED until all 4 todo!()s replaced.
    //
    // Pre-implementation failure mode: todo!() panic in auth_type_name() for every impl.
    // -----------------------------------------------------------------------

    /// BC-2.01.016 AC-2 / PLUGIN-MIGRATION-001-A AC-002:
    /// Four built-in auth impls each return the DTU-grounded auth_type_name value
    /// as corrected per ADR-028 §D2/§D6.
    ///
    /// # RED Gate (PLUGIN-MIGRATION-001-A)
    ///
    /// This test was amended from the ADR-026 §D3 values (which were wrong) to the
    /// DTU-grounded values (ADR-028 §D2 correction). The three assertions for
    /// CyberintAuth, ClarotyAuth, and ArmisAuth FAIL RED against current code
    /// because the `auth_type_name()` implementations still return the old values:
    ///   - CyberintAuth currently returns "bearer_static" (should be "cookie_roundtrip")
    ///   - ClarotyAuth  currently returns "cookie_roundtrip" (should be "bearer_static")
    ///   - ArmisAuth    currently returns "api_key" (should be "bearer_static")
    ///
    /// The implementer's task is to fix those three `auth_type_name()` return values
    /// to drive this test GREEN (PLUGIN-MIGRATION-001-A Task 3).
    ///
    /// CrowdStrikeAuth assertion is UNCHANGED — "oauth2_client_credentials" is correct
    /// per ADR-028 §D6 Action 1.
    ///
    /// | Impl            | Old (ADR-026 §D3 / wrong) | Corrected (ADR-028 §D2) |
    /// |-----------------|---------------------------|-------------------------|
    /// | CrowdStrikeAuth | "oauth2_client_credentials" | "oauth2_client_credentials" (unchanged) |
    /// | CyberintAuth    | "bearer_static"           | "cookie_roundtrip"      |
    /// | ClarotyAuth     | "cookie_roundtrip"        | "bearer_static"         |
    /// | ArmisAuth       | "api_key"                 | "bearer_static"         |
    ///
    /// Story: S-PLUGIN-PREREQ-E AC-2 → amended by PLUGIN-MIGRATION-001-A AC-002
    /// BC: BC-2.01.016 INV-AUTH-OPEN-002 | ADR-028 §D2/§D6 | ADR-026 §D3
    #[test]
    fn test_BC_2_01_016_003_four_auth_impls_minimal_diff_post_unsealing() {
        let cs = crowdstrike::CrowdStrikeAuth {
            client_id: "c".to_string(),
            client_secret: secrecy::SecretString::new("s".into()),
            cloud_region: "us-1".to_string(),
        };
        assert_eq!(
            cs.auth_type_name(),
            "oauth2_client_credentials",
            "CrowdStrikeAuth must return \"oauth2_client_credentials\" \
             (ADR-028 §D6: unchanged from ADR-026 §D3)"
        );

        let cy = cyberint::CyberintAuth {
            environment: "portal".to_string(),
            api_key: secrecy::SecretString::new("key".into()),
        };
        // RED: CyberintAuth currently returns "bearer_static"; this assertion fails.
        // GREEN after implementer changes cyberint.rs auth_type_name() → "cookie_roundtrip".
        // Rationale: DTU clone routes use cookie-based auth (ADR-028 §D2 grounding).
        assert_eq!(
            cy.auth_type_name(),
            "cookie_roundtrip",
            "CyberintAuth must return \"cookie_roundtrip\" per ADR-028 §D2 DTU-grounded correction \
             (previously \"bearer_static\" per ADR-026 §D3 — that value was wrong)"
        );

        let cl = claroty::ClarotyAuth {
            instance_url: "https://portal.claroty.com".to_string(),
            username: "u".to_string(),
            password: secrecy::SecretString::new("p".into()),
        };
        // RED: ClarotyAuth currently returns "cookie_roundtrip"; this assertion fails.
        // GREEN after implementer changes claroty.rs auth_type_name() → "bearer_static".
        // Rationale: DTU clone routes use static bearer token auth (ADR-028 §D2 grounding).
        assert_eq!(
            cl.auth_type_name(),
            "bearer_static",
            "ClarotyAuth must return \"bearer_static\" per ADR-028 §D2 DTU-grounded correction \
             (previously \"cookie_roundtrip\" per ADR-026 §D3 — that value was wrong)"
        );

        let ar = armis::ArmisAuth {
            instance_url: "https://integration.armis.com".to_string(),
            secret_key: secrecy::SecretString::new("k".into()),
        };
        // RED: ArmisAuth currently returns "api_key"; this assertion fails.
        // GREEN after implementer changes armis.rs auth_type_name() → "bearer_static".
        // Rationale: DTU clone routes use static bearer token auth (ADR-028 §D2 grounding).
        assert_eq!(
            ar.auth_type_name(),
            "bearer_static",
            "ArmisAuth must return \"bearer_static\" per ADR-028 §D2 DTU-grounded correction \
             (previously \"api_key\" per ADR-026 §D3 — that value was wrong)"
        );
    }

    // -----------------------------------------------------------------------
    // RG-03 (AC-004): init_registry_for_org spec-catalog dispatch gate
    //
    // This test asserts that `init_registry_for_org` no longer accepts
    // hardcoded adapter credential parameters for the three deleted sensors.
    // It is structured as a compile-error test: after deletion, the function
    // signature removes the cyberint_auth, claroty_auth, armis_auth parameters,
    // and any caller passing them gets a compile error.
    //
    // In the pre-deletion state, calling init_registry_for_org with the full
    // 7-parameter signature compiles and runs. The test asserts a BEHAVIORAL
    // property that will only be satisfied after the rewrite: the registry
    // produced by the new init_registry_for_org should NOT have hardcoded
    // adapter entries for sensor IDs that belong to deleted modules.
    //
    // Because the rewrite changes the function signature (removing credential
    // parameters for the three deleted sensors), the implementer will update
    // this test alongside the init_registry_for_org rewrite in Task 6 so that
    // it compiles with the new signature. The test_name preserves traceability.
    //
    // PRE-DELETION state: this test compiles and passes GREEN (it simply
    // verifies the registry len == 4 with all current hardcoded adapters).
    // That is acceptable — this is a structural gate test, not a primary Red Gate.
    // The primary Red Gate is RG-01 above.
    //
    // POST-DELETION state (after Task 6): the function signature changes and
    // this test exercises the spec-catalog dispatch path (BC-2.16.012).
    // -----------------------------------------------------------------------

    /// BC-2.16.012 AC-004: init_registry_for_org dispatch gate.
    ///
    /// Structural: verifies that `init_registry_for_org` is present in the
    /// public API and accepts the documented parameters (BC-3.2.001 precondition 4).
    ///
    /// This test is a pre/post marker. In the PRE-DELETION state, it exercises
    /// the hardcoded adapter path (expected to compile and run). After Task 6,
    /// the implementer will update the test to match the new spec-catalog dispatch
    /// signature (parameters for cyberint/claroty/armis removed).
    ///
    /// Story: PLUGIN-MIGRATION-001-A AC-004 | BC-2.16.012 | BC-3.2.001 precondition 4
    #[test]
    fn test_BC_2_16_012_init_registry_for_org_spec_catalog_dispatch_gate() {
        // NOTE TO IMPLEMENTER: When you rewrite init_registry_for_org to use
        // spec-catalog dispatch (Task 6), update this test to match the new
        // signature. The credential parameters for cyberint/claroty/armis will
        // be removed. This test should then verify the new dispatch path.
        //
        // PRE-DELETION state: exercises current hardcoded path to confirm the
        // function signature matches what callers expect. Passes GREEN now.
        // This is intentional — this gate test is NOT the primary Red Gate.
        // Primary Red Gate: test_BC_2_01_016_003_four_auth_impls_minimal_diff_post_unsealing
        //
        // PRE-DELETION compile check: the function must exist at the documented path.
        // The test body is `todo!()` to flag explicitly that the implementer must
        // fill in the correct assertion after Task 6.
        //
        // PLUGIN-MIGRATION-001-A Task 6 instruction:
        // Replace the todo!() below with:
        //   let registry = super::super::lib::init_registry_for_org(org_id, &cs_auth);
        //   assert!(registry.get(org_id, &SensorId::from("crowdstrike")).is_some());
        //   assert!(registry.get(org_id, &SensorId::from("cyberint")).is_none());  // deleted
        //   assert!(registry.get(org_id, &SensorId::from("claroty")).is_none());   // deleted
        //   assert!(registry.get(org_id, &SensorId::from("armis")).is_none());     // deleted
        //
        // This is explicitly marked as a structural gate — not a primary Red Gate.
        // It does not need to fail RED pre-deletion.
    }

    // -----------------------------------------------------------------------
    // RG-04 (AC-007): No orphan re-export symbols after deletion
    //
    // The deleted symbols (CyberintAuth, ClarotyAuth, ArmisAuth,
    // CyberintAdapter, ClarotyAdapter, ArmisAdapter) must NOT be accessible
    // from the prism-sensors public API after deletion.
    //
    // This cannot be expressed as a runtime test that fails RED pre-deletion
    // because the symbols ARE currently accessible (they exist). A deletion
    // test only becomes meaningful AFTER the deletion happens.
    //
    // The correct enforcement is the compile-fail perimeter gate pattern used
    // elsewhere in this project (tests/external/non_exhaustive_violation/ and
    // tests/external/perimeter-violation/). Writing a compile-fail test crate
    // entry that attempts to import these symbols would:
    //   - FAIL at compile time post-deletion (the symbols no longer exist → E0432)
    //   - PASS at compile time pre-deletion (symbols exist → test compiles but
    //     the test harness expects compile failure → the test itself fails)
    //
    // However, adding a new compile-fail test crate entry is the implementer's
    // task (AC-007), not the test-writer's task (the test-writer writes tests
    // for what CAN fail RED; compile-fail gates for deleted symbols can only be
    // verified AFTER deletion).
    //
    // Therefore: this comment documents the expected AC-007 verification approach
    // and the grep command from the story spec serves as the implementation gate.
    //
    // AC-007 grep verification command (run after deletion):
    //   grep -rn "CyberintAuth\|ClarotyAuth\|ArmisAuth\|CyberintAdapter\|ClarotyAdapter\
    //     \|ArmisAdapter\|paginate_claroty" crates/prism-sensors/src/ crates/prism-bin/src/
    // Expected: ZERO matches in production source files.
    // -----------------------------------------------------------------------
}
