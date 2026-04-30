//! Namespace key construction — BC-2.03.004, BC-3.2.002
//!
//! ## Legacy key format (OrgSlug-based, deprecated by BC-3.2.002)
//! `"{slug}/{sensor}/{name}"` — e.g. `"acme/crowdstrike/api_key"`
//!
//! ## New key format (OrgId-based, S-3.1.04 / BC-3.2.002)
//! `"{org_id_uuid}/{sensor}/{name}"` — e.g.
//! `"018e3f71-5c6d-7a8b-9c0d-1e2f3a4b5c6d/crowdstrike/api_key"`
//!
//! The OrgId UUID string is rename-stable (the UUID never changes when the
//! OrgSlug changes) and structurally opaque to AI observers per the
//! AI-opaque credential principle (BC-3.2.002 postcondition 4).
//!
//! This key is used as the keyring service/account identifier and as the
//! encrypted-file path component. All components are validated before
//! construction.
//!
//! Story: S-1.06 | BC: BC-2.03.004, BC-2.03.008
//! Story: S-3.1.04 | BC: BC-3.2.002

pub use prism_core::CredentialName;
use prism_core::{OrgId, OrgSlug, PrismError};

/// Build the namespaced credential key (legacy OrgSlug-based format).
///
/// Format: `"{slug}/{sensor}/{name}"` — e.g. `"acme/crowdstrike/api_key"`
///
/// **Deprecated by S-3.1.04 / BC-3.2.002.** New code MUST use
/// [`namespace_key_by_org_id`] instead. This shim remains temporarily to
/// allow call sites to migrate incrementally without a compilation break.
///
/// Validation: sensor must be non-empty. OrgSlug and CredentialName
/// validation is enforced by their respective newtypes before reaching here.
pub fn namespace_key(tenant: &OrgSlug, sensor: &str, name: &CredentialName) -> String {
    format!("{}/{}/{}", tenant.as_str(), sensor, name.as_str())
}

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
    use prism_core::{CredentialName, OrgSlug};

    #[test]
    fn test_BC_2_03_004_namespace_key_format_basic() {
        let tenant = OrgSlug::new_unchecked("acme");
        let name = CredentialName::new_from_validated_storage("api_key");
        let key = namespace_key(&tenant, "crowdstrike", &name);
        assert_eq!(key, "acme/crowdstrike/api_key");
    }
}
