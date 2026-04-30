//! Namespace key construction — BC-2.03.004, BC-3.2.002
//!
//! ## Key format (OrgId-based, S-3.1.04 / BC-3.2.002)
//! `"{org_id_uuid}/{sensor}/{name}"` — e.g.
//! `"018e3f71-5c6d-7a8b-9c0d-1e2f3a4b5c6d/crowdstrike/api_key"`
//!
//! The OrgId UUID string is rename-stable (the UUID never changes when the
//! org display name changes) and structurally opaque to AI observers per the
//! AI-opaque credential principle (BC-3.2.002 postcondition 4).
//!
//! The legacy slug-keyed format has been removed from this module (AC-5 /
//! BC-3.2.002 invariant 1). Existing slug-keyed callers in `keyring.rs` and
//! `file.rs` use a local inline helper that is private to those modules.
//!
//! This key is used as the keyring service/account identifier and as the
//! encrypted-file path component. All components are validated before
//! construction.
//!
//! Story: S-1.06 | BC: BC-2.03.004, BC-2.03.008
//! Story: S-3.1.04 | BC: BC-3.2.002 (AC-5 — no slug type in this module)

pub use prism_core::CredentialName;
use prism_core::{OrgId, PrismError};

/// Build the namespaced credential key (OrgId-based format, BC-3.2.002).
///
/// Format: `"{org_id_uuid}/{sensor}/{name}"` — e.g.
/// `"018e3f71-5c6d-7a8b-9c0d-1e2f3a4b5c6d/crowdstrike/api_key"`
///
/// The `org_id_uuid` component is the hyphenated lowercase UUID v7 string
/// produced by `OrgId::to_string()`. This key is rename-stable: it does not
/// change when the org's display slug changes (BC-3.2.002 postcondition 3).
///
/// # Validation
/// Sensor must be non-empty. `OrgId` and `CredentialName` validation is
/// enforced by their respective newtypes before reaching here.
///
/// # Story
/// S-3.1.04 | BC: BC-3.2.002
pub fn namespace_key_by_org_id(org_id: &OrgId, sensor: &str, name: &CredentialName) -> String {
    format!("{}/{}/{}", org_id, sensor, name.as_str())
}

/// Validate that a sensor string is non-empty and ASCII alphanumeric + `_-`.
pub fn validate_sensor(sensor: &str) -> Result<(), PrismError> {
    if sensor.is_empty() {
        return Err(PrismError::CredentialStoreError {
            backend: "namespace".to_owned(),
            reason: "sensor name must not be empty".to_owned(),
        });
    }
    if !sensor
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
    {
        return Err(PrismError::CredentialStoreError {
            backend: "namespace".to_owned(),
            reason: format!(
                "sensor name contains invalid characters (allowed: [a-zA-Z0-9_-]): {sensor:?}"
            ),
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use prism_core::{CredentialName, OrgId};

    /// BC-2.03.004 / BC-3.2.002: namespace_key_by_org_id produces
    /// "{org_id_uuid}/{sensor}/{name}" format (OrgId-keyed, AC-5 compliant).
    #[test]
    fn test_BC_2_03_004_namespace_key_format_basic() {
        let org_id = OrgId::new();
        let name = CredentialName::new_from_validated_storage("api_key");
        let key = namespace_key_by_org_id(&org_id, "crowdstrike", &name);
        let expected = format!("{}/crowdstrike/api_key", org_id);
        assert_eq!(key, expected);
    }
}
