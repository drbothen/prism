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

    /// BC-2.01.016 AC-2: Four built-in auth impls each return exactly one new
    /// method body (`auth_type_name`) per ADR-026 D1/D2 Path B.
    ///
    /// | Impl            | Expected auth_type_name            |
    /// |-----------------|------------------------------------|
    /// | CrowdStrikeAuth | "oauth2_client_credentials"        |
    /// | CyberintAuth    | "bearer_static"                    |
    /// | ClarotyAuth     | "cookie_roundtrip"                 |
    /// | ArmisAuth       | "api_key"                          |
    ///
    /// Red Gate failure mode: todo!() panic in auth_type_name() for each impl.
    ///
    /// Story: S-PLUGIN-PREREQ-E AC-2 | BC: BC-2.01.016 INV-AUTH-OPEN-002 | ADR-026 §D3
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
            "CrowdStrikeAuth must return \"oauth2_client_credentials\" (ADR-026 §D3)"
        );

        let cy = cyberint::CyberintAuth {
            environment: "portal".to_string(),
            api_key: secrecy::SecretString::new("key".into()),
        };
        assert_eq!(
            cy.auth_type_name(),
            "bearer_static",
            "CyberintAuth must return \"bearer_static\" (ADR-026 §D3)"
        );

        let cl = claroty::ClarotyAuth {
            instance_url: "https://portal.claroty.com".to_string(),
            username: "u".to_string(),
            password: secrecy::SecretString::new("p".into()),
        };
        assert_eq!(
            cl.auth_type_name(),
            "cookie_roundtrip",
            "ClarotyAuth must return \"cookie_roundtrip\" (ADR-026 §D3)"
        );

        let ar = armis::ArmisAuth {
            instance_url: "https://integration.armis.com".to_string(),
            secret_key: secrecy::SecretString::new("k".into()),
        };
        assert_eq!(
            ar.auth_type_name(),
            "api_key",
            "ArmisAuth must return \"api_key\" (ADR-026 §D3)"
        );
    }
}
