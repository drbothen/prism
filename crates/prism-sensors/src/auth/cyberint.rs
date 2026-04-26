//! Cyberint API authentication credentials.
//!
//! Story: S-2.06 | BC: BC-2.01.013

use secrecy::SecretString;

use super::{private::Sealed, SensorAuth};

/// Cyberint portal API key credentials.
///
/// `Debug` omits the `api_key` value — credentials MUST NOT transit AI context.
pub struct CyberintAuth {
    /// Cyberint portal environment (e.g., `"portal"`, `"portal.eu"`).
    pub environment: String,
    /// Cyberint API key — MUST NOT appear in any log output.
    pub api_key: SecretString,
}

impl std::fmt::Debug for CyberintAuth {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CyberintAuth")
            .field("environment", &self.environment)
            .field("api_key", &"Secret(***)")
            .finish()
    }
}

impl Sealed for CyberintAuth {}
impl SensorAuth for CyberintAuth {}
