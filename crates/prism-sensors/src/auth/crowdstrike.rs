//! CrowdStrike Falcon API authentication credentials.
//!
//! Story: S-2.06 | BC: BC-2.01.013

use secrecy::SecretString;

use super::{private::Sealed, SensorAuth};

/// CrowdStrike Falcon API OAuth2 client credentials.
///
/// `Debug` omits the `client_secret` value — credentials MUST NOT transit
/// AI context (AI-opaque credential model).
pub struct CrowdStrikeAuth {
    /// OAuth2 client ID (non-secret; safe to log).
    pub client_id: String,
    /// OAuth2 client secret — MUST NOT appear in any log output.
    pub client_secret: SecretString,
    /// CrowdStrike cloud region (e.g., `"us-1"`, `"eu-1"`).
    pub cloud_region: String,
}

impl std::fmt::Debug for CrowdStrikeAuth {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CrowdStrikeAuth")
            .field("client_id", &self.client_id)
            .field("client_secret", &"Secret(***)")
            .field("cloud_region", &self.cloud_region)
            .finish()
    }
}

impl Sealed for CrowdStrikeAuth {}
impl SensorAuth for CrowdStrikeAuth {}
