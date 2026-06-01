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
// BearerStaticSensorAuth — concrete SensorAuth for bearer_static sensors
// ---------------------------------------------------------------------------

/// Concrete `SensorAuth` implementation for `bearer_static` sensors (Armis, Claroty).
///
/// Carries the bearer token string for extraction via `as_any().downcast_ref::<BearerStaticSensorAuth>()`
/// in `SpecDrivenSensorAdapter::fetch()` (BearerStatic auth strategy path — S-DEMO-001 §OQ-1).
///
/// The token is NOT held at `SpecDrivenSensorAdapter` construction time — it arrives at
/// fetch call time via this struct passed as `&dyn SensorAuth` (AD-017 credential safety).
///
/// AD-017: `Debug` deliberately omits the token value. Never log `token` at any level.
///
/// Story: S-DEMO-001 | BC-2.01.013 postcondition 4 | OQ-1 Resolution
#[non_exhaustive]
pub struct BearerStaticSensorAuth {
    /// The bearer token string for this fetch call.
    ///
    /// AD-017: value MUST NOT appear in log output at any level.
    pub token: String,
}

impl BearerStaticSensorAuth {
    /// Construct a `BearerStaticSensorAuth` carrying the given bearer token.
    ///
    /// `#[non_exhaustive]` requires callers to use this constructor (not struct literal).
    /// AD-017: token value MUST NOT be logged.
    ///
    /// Story: S-DEMO-001 | OQ-1 Resolution
    pub fn new(token: impl Into<String>) -> Self {
        Self {
            token: token.into(),
        }
    }
}

impl SensorAuth for BearerStaticSensorAuth {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn auth_type_name(&self) -> &'static str {
        "bearer_static"
    }
}

impl std::fmt::Debug for BearerStaticSensorAuth {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // AD-017: never emit the token value.
        f.debug_struct("BearerStaticSensorAuth")
            .field("token", &"<redacted>")
            .finish()
    }
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

    // test_BC_2_01_016_003 (Red Gate for auth_type_name DTU-grounded corrections):
    // Removed post-deletion — corrections proven at commit 36f04029 before auth
    // module deletion. Trait contract coverage provided by test_BC_2_01_016_001.
    // See ADR-028 §D6 Action 2 for git-history observability rationale.

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
