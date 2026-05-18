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
