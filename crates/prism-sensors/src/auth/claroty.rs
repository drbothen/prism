//! Claroty xDome API authentication credentials.
//!
//! Story: S-2.06 | BC: BC-2.01.013

use secrecy::SecretString;

use super::{private::Sealed, SensorAuth};

/// Claroty xDome REST API credentials (username + password).
///
/// `Debug` omits the `password` value — credentials MUST NOT transit AI context.
pub struct ClarotyAuth {
    /// xDome instance base URL (e.g., `"https://acme.claroty.com"`).
    pub instance_url: String,
    /// xDome API username (non-secret; safe to log).
    pub username: String,
    /// xDome API password — MUST NOT appear in any log output.
    pub password: SecretString,
}

impl std::fmt::Debug for ClarotyAuth {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ClarotyAuth")
            .field("instance_url", &self.instance_url)
            .field("username", &self.username)
            .field("password", &"Secret(***)")
            .finish()
    }
}

impl Sealed for ClarotyAuth {}
impl SensorAuth for ClarotyAuth {}
