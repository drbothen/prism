//! OrgRegistry boot orchestrator — wires `load_and_validate` into startup.
//!
//! Implements BC-3.3.004 Invariant 1 (validate-before-register ordering):
//! `load_and_validate` is called for ALL files BEFORE `OrgRegistry::register`
//! is called for ANY file (ADR-010 §2.5).
//!
//! # Error contract
//!
//! Returns `Err(BootError::ValidationFailed(errors))` if any config file fails
//! validation. Returns `Err(BootError::RegistrationFailed(e))` if
//! `OrgRegistry::register` rejects a pair after validation passed (defense-in-depth
//! guard, BC-3.3.04 Task 6).
//!
//! Returns `Ok(n)` where `n` is the number of orgs registered.
//!
//! # Caller responsibility
//!
//! The caller (startup entry point, typically `prism-mcp/src/main.rs`) is
//! responsible for writing errors to stderr and calling `std::process::exit(1)`.
//! This function performs no I/O beyond filesystem reads inside
//! `load_and_validate`; it does NOT call `process::exit`.
//!
//! Traces to: BC-3.3.004, BC-3.1.003, BC-3.1.004 (S-3.3.02)

use std::path::Path;

use prism_core::org_registry::{OrgRegistry, RegistrationError};

use crate::error::ConfigError;

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

/// Errors returned by [`boot_org_registry`].
///
/// Traces to BC-3.3.004 postconditions and BC-3.1.004.
#[derive(Debug)]
pub enum BootError {
    /// One or more customer config files failed validation.
    ///
    /// Contains ALL errors from ALL files (multi-error).
    /// BC-3.3.004 postcondition "On any validation failure" clauses 1–4.
    ValidationFailed(Vec<ConfigError>),

    /// `OrgRegistry::register` rejected a (slug, id) pair despite successful
    /// validation — defense-in-depth guard (BC-3.3.04 Task 6 / EC-003).
    RegistrationFailed(RegistrationError),
}

impl std::fmt::Display for BootError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BootError::ValidationFailed(errors) => {
                write!(f, "{} customer config validation error(s)", errors.len())
            }
            BootError::RegistrationFailed(e) => {
                write!(f, "OrgRegistry registration conflict: {e}")
            }
        }
    }
}

impl std::error::Error for BootError {}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Boot `OrgRegistry` from all `*.toml` files in `customers_dir`.
///
/// # Algorithm (ADR-010 §2.5)
///
/// 1. Call `load_and_validate(customers_dir)` — validates ALL files before any
///    registration occurs (BC-3.3.004 Invariant 1).
/// 2. On `Err(errors)` — return `Err(BootError::ValidationFailed(errors))`.
///    Zero entries are added to `registry` (BC-3.3.004 postcondition 4).
/// 3. On `Ok(configs)` — call `registry.register(slug, id)` for each config
///    in lexicographic file order (BC-3.3.004 postcondition 1).
/// 4. If `register` returns `Err(e)` — return `Err(BootError::RegistrationFailed(e))`.
///    This is a defense-in-depth guard; duplicates should have been caught in step 1
///    via E-CFG-011/E-CFG-012.
/// 5. Return `Ok(n)` where `n` is the number of orgs registered.
///
/// # Notes
///
/// - `OrgSlug::new` is called with `config.org_slug`. Validation guarantees the
///   slug matches the filename stem and the pattern `^[a-zA-Z0-9_-]{1,64}$`
///   (R-CUST-002). A panic here would indicate a validator bug.
/// - `OrgId::from_uuid` wraps the `Uuid` from `CustomerConfig.org_id`. UUID v7
///   version is enforced by R-CUST-003 in the validator; `from_uuid` does not
///   re-check (use `from_uuid_v7` for stricter enforcement if needed).
///
/// Traces to: BC-3.3.004 postconditions, BC-3.1.003 postcondition 1,
///            BC-3.1.004 postconditions 2–4.
pub fn boot_org_registry(customers_dir: &Path, registry: &OrgRegistry) -> Result<usize, BootError> {
    // S-3.3.02 stub — implementation pending (Red Gate prep).
    //
    // IMPLEMENTER: replace this todo!() with the algorithm described above.
    // All tests in tests/startup_boot_test.rs must pass after implementation.
    // Do NOT add OrgRegistry entries before load_and_validate returns Ok (ADR-010 §2.5).
    let _ = (customers_dir, registry);
    todo!("S-3.3.02: boot_org_registry not yet implemented — Red Gate stub")
}
