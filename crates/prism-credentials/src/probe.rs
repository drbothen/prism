//! Startup keyring availability probe — BC-2.03.011
//!
//! Performs a test read/write/delete against the OS keyring at startup to:
//! 1. Trigger any OS permission prompts (macOS Keychain authorization dialog)
//!    once, at process start — before normal operation begins.
//! 2. Confirm the keyring is available and functional.
//!
//! On failure: logs WARN and returns `KeyringStatus::Unavailable(reason)`.
//! On success: logs INFO and returns `KeyringStatus::Available`.
//!
//! NEVER panics — unavailability is handled gracefully via `BackendSelector`.
//!
//! Story: S-1.06 | BC: BC-2.03.011

use tracing::{info, warn};

/// Result of the startup keyring probe.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeyringStatus {
    /// Keyring is accessible and functional.
    Available,
    /// Keyring is unavailable; contains a human-readable reason string.
    Unavailable(String),
}

/// Probe the OS keyring for availability.
///
/// Algorithm (BC-2.03.011):
/// 1. Create a test entry `Entry::new(app_name, "prism_probe")`.
/// 2. Set password to `"prism_probe_ok"`.
/// 3. Read password back — verify it matches.
/// 4. Delete the entry.
/// 5. Return `Available` or `Unavailable(reason)`.
///
/// All keyring-rs calls are wrapped in `spawn_blocking`.
pub async fn probe_keyring(app_name: &str) -> KeyringStatus {
    let app_name = app_name.to_owned();

    let result = tokio::task::spawn_blocking(move || -> Result<(), String> {
        let entry = keyring::Entry::new(&app_name, "prism_probe")
            .map_err(|e| format!("failed to create keyring entry: {e}"))?;

        // Write probe value.
        entry
            .set_password("prism_probe_ok")
            .map_err(|e| format!("failed to write probe value: {e}"))?;

        // Read probe value back and verify.
        let read_back = entry
            .get_password()
            .map_err(|e| format!("failed to read probe value: {e}"))?;

        if read_back != "prism_probe_ok" {
            return Err(format!(
                "probe value mismatch: expected 'prism_probe_ok', got {read_back:?}"
            ));
        }

        // Delete probe entry (cleanup).
        // Ignore delete errors — cleanup is best-effort.
        let _ = entry.delete_credential();

        Ok(())
    })
    .await;

    match result {
        Ok(Ok(())) => {
            info!(
                app_name = %"prism",
                "keyring probe: OS keyring is available and functional"
            );
            KeyringStatus::Available
        }
        Ok(Err(reason)) => {
            warn!(
                reason = %reason,
                "keyring probe: OS keyring unavailable — will fall back to encrypted file backend"
            );
            KeyringStatus::Unavailable(reason)
        }
        Err(join_err) => {
            let reason = format!("spawn_blocking task panicked: {join_err}");
            warn!(
                reason = %reason,
                "keyring probe: task join error — treating keyring as unavailable"
            );
            KeyringStatus::Unavailable(reason)
        }
    }
}
