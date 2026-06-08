//! Test modules for prism-credentials (S-1.06).
//!
//! All tests pass (implementation complete. Was Red Gate before
//! implementation). They are either:
//!   - Calling unimplemented!() stubs (panics with "unimplemented")
//!   - asserting postconditions that the stubs do not yet satisfy

// F-P10-CRIT-001 keyring mock override — installs the in-memory keyring mock for
// all tests in this binary so no test touches the real OS Keychain.
//
// keyring 3.x uses a global default credential builder; `set_default_credential_builder`
// overrides it for the entire process. With this override, `probe_keyring`,
// `BackendSelector::select_backend`, and any other in-process `KeyringBackend`
// operation uses the mock store (in-memory HashMap, no OS access, no prompts).
//
// `std::sync::OnceLock` ensures the initialization runs exactly once even when
// multiple tests call `install_keyring_mock` concurrently (cargo nextest runs
// each test binary in a single process; tests within a binary may run in parallel
// threads, but OnceLock::get_or_init is thread-safe).
//
// IMPORTANT: This override applies ONLY to in-process `keyring::Entry` calls.
// Subprocess tests (spawning `prism credential set`) spawn a new process that does NOT
// inherit this override → those tests must be #[ignore]'d separately (SID-1 §4).
//
// Coverage: store_tests::test_BC_2_03_011_probe_*, test_BC_2_03_012_*
// (These tests previously reached the real macOS Keychain via probe_keyring + spawn_blocking;
//  the mock makes them fast and prompt-free.)
static _KEYRING_MOCK_INSTALLED: std::sync::OnceLock<()> = std::sync::OnceLock::new();

/// Install the keyring mock builder for this test binary.
///
/// Call this at the start of any test that directly or indirectly calls `KeyringBackend`
/// methods or `probe_keyring`. Idempotent: repeated calls are safe (OnceLock).
///
/// F-P10-CRIT-001 / S-DEMO-003.
pub fn install_keyring_mock() {
    _KEYRING_MOCK_INSTALLED.get_or_init(|| {
        keyring::set_default_credential_builder(keyring::mock::default_credential_builder());
    });
}

pub mod proptest_crypto;
pub mod store_tests;
