//! `SensorAuth` sealed trait and per-sensor auth credential subtypes.
//!
//! # Sealed Trait Pattern
//!
//! `SensorAuth` is sealed via a private `Sealed` marker trait defined in
//! `private`. External crates cannot implement `private::Sealed`, so they
//! cannot implement `SensorAuth`. This prevents cross-sensor auth composition
//! at compile time (BC-2.01.013, DI-012).
//!
//! Only the four subtypes defined in this module implement `SensorAuth`:
//! - [`CrowdStrikeAuth`] — CrowdStrike Falcon API credentials
//! - [`CyberintAuth`]    — Cyberint API credentials
//! - [`ClarotyAuth`]     — Claroty xDome API credentials
//! - [`ArmisAuth`]       — Armis Centrix API credentials
//!
//! Story: S-2.06 | BC: BC-2.01.013

pub mod armis;
pub mod claroty;
pub mod crowdstrike;
pub mod cyberint;

pub use armis::ArmisAuth;
pub use claroty::ClarotyAuth;
pub use crowdstrike::CrowdStrikeAuth;
pub use cyberint::CyberintAuth;

// ---------------------------------------------------------------------------
// Sealed trait machinery (private module — not re-exported)
// ---------------------------------------------------------------------------

mod private {
    /// Marker trait that seals `SensorAuth`.
    ///
    /// Because this module is private, external crates cannot name `Sealed`
    /// and therefore cannot implement it. Any type that does not implement
    /// `Sealed` cannot implement `SensorAuth`.
    pub trait Sealed {}
}

// ---------------------------------------------------------------------------
// SensorAuth sealed trait
// ---------------------------------------------------------------------------

/// Sealed authentication credential for a sensor adapter.
///
/// Implement this trait only within `prism-sensors`. External crates cannot
/// add new auth types — `private::Sealed` is unreachable outside this crate.
///
/// Each auth subtype carries ONLY its own credentials (no field overlap across
/// sensor types). Credentials MUST NOT appear in `Debug` output or log output
/// at any level (AI-opaque credential model).
///
/// Story: S-2.06 | BC: BC-2.01.013
pub trait SensorAuth: private::Sealed + Send + Sync + 'static {
    /// Returns `self` as `&dyn std::any::Any` to enable downcasting in adapters.
    ///
    /// Adapters receive `&dyn SensorAuth` but need access to concrete credential
    /// fields (e.g., `client_id`, `api_key`). `as_any()` allows safe downcasting
    /// to the concrete type using `downcast_ref::<ConcreteAuthType>()`.
    fn as_any(&self) -> &dyn std::any::Any;
}
