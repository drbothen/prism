//! Armis Centrix API authentication credentials.
//!
//! Story: S-2.06 | BC: BC-2.01.013

use secrecy::SecretString;

use super::{private::Sealed, SensorAuth};

/// Armis Centrix REST API key credentials.
///
/// `Debug` omits the `secret_key` value — credentials MUST NOT transit AI context.
pub struct ArmisAuth {
    /// Armis tenant base URL (e.g., `"https://acme.armis.com"`).
    pub instance_url: String,
    /// Armis API secret key — MUST NOT appear in any log output.
    pub secret_key: SecretString,
}

impl std::fmt::Debug for ArmisAuth {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ArmisAuth")
            .field("instance_url", &self.instance_url)
            .field("secret_key", &"Secret(***)")
            .finish()
    }
}

impl Sealed for ArmisAuth {}
impl SensorAuth for ArmisAuth {}
