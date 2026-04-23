//! Namespace key construction — BC-2.03.004
//!
//! Format: `"{tenant}/{sensor}/{name}"`  e.g. `"acme/crowdstrike/api_key"`
//!
//! This key is used as the keyring service/account identifier and as the
//! encrypted-file entry key. All three components are validated before
//! construction.
//!
//! Story: S-1.06 | BC: BC-2.03.004, BC-2.03.008

pub use prism_core::CredentialName;
use prism_core::{PrismError, TenantId};

/// Build the namespaced credential key.
///
/// Format: `"{tenant}/{sensor}/{name}"` — e.g. `"acme/crowdstrike/api_key"`
///
/// Validation: sensor must be non-empty. TenantId and CredentialName
/// validation is enforced by their respective newtypes before reaching here.
pub fn namespace_key(tenant: &TenantId, sensor: &str, name: &CredentialName) -> String {
    format!("{}/{}/{}", tenant.as_str(), sensor, name.as_str())
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
    use prism_core::{CredentialName, TenantId};

    #[test]
    fn test_BC_2_03_004_namespace_key_format_basic() {
        let tenant = TenantId::new_unchecked("acme");
        let name = CredentialName::new_from_validated_storage("api_key");
        let key = namespace_key(&tenant, "crowdstrike", &name);
        assert_eq!(key, "acme/crowdstrike/api_key");
    }
}
