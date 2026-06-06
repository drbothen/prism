//! Red Gate test for S-DEMO-003 AC-005 — BC-2.03.007 secret redaction contract.
//!
//! # Contract under test
//!
//! BC-2.03.007 postcondition: "Secret Redaction in Logs, Errors, and MCP Responses".
//! For the `prism credential set` subcommand (AC-005 of S-DEMO-003): the credential
//! VALUE passed via stdin MUST NOT appear on stdout or stderr.
//!
//! AC-005 of S-DEMO-003 (traces to BC-2.03.007):
//!   "prompts 'Enter value: ' on stderr (no terminal echo); reads the value from stdin;
//!    [...] the value is NOT logged, NOT printed to stdout"
//!
//! # Test approach
//!
//! This test spawns the `prism` binary as a subprocess with:
//! - Subcommand: `credential set --sensor crowdstrike --name client_id`
//! - Stdin: a known sentinel value ("PRISM_DEMO_SECRET_SENTINEL_12345")
//! - Capture: both stdout and stderr
//!
//! The test asserts that the sentinel value does NOT appear in either stdout or stderr.
//!
//! The test is designed to fail at Red Gate because `handle_credential_set()` panics
//! with `todo!()` — the subprocess exits non-zero (panic exit code 1) AND the panic
//! message appears in stderr, causing the "secret not in stderr" assertion to fail
//! if the sentinel were in the message, but more directly: the subprocess will panic,
//! which means stdout will be empty (no "Credential stored successfully."), confirming
//! the test catches the unimplemented state.
//!
//! # Why this FAILS at Red Gate
//!
//! Two failure modes during Red Gate phase:
//! 1. The binary panics in `handle_credential_set()` (`todo!()`) — subprocess exits 1.
//! 2. The assertion `assert_eq!(exit_code, 0)` fails because the subprocess panicked.
//!
//! # What the implementer must do to make this PASS
//!
//! 1. Implement `handle_credential_set()` in `credential_cli.rs`.
//! 2. The implementation must:
//!    a. Prompt on **stderr** (not stdout): "Enter value for prism/{sensor}/{name}: "
//!    b. Read value from stdin using `rpassword::prompt_password()` (echo disabled).
//!    c. Write to keyring via `KeyringBackend::set()`.
//!    d. On success: print "Credential stored successfully." to **stdout** only.
//!    e. The sentinel value must NEVER appear on stdout or stderr at any point.
//! 3. Add `rpassword = "7.*"` to `[dependencies]` in `crates/prism-bin/Cargo.toml`.
//!
//! # AD-017 invariant tested
//!
//! AD-017 (AI-opaque credential model): credential values must never transit a
//! visible channel. This test is the behavioral enforcement of that invariant for
//! the `prism credential set` subcommand.
//!
//! # Test infrastructure note
//!
//! This test spawns a subprocess. On CI, the keyring backend may not be available
//! (headless environment). The test handles this by checking for exit code 1 with
//! "Keyring unavailable" on stderr — this is an acceptable failure mode per EC-001
//! of S-DEMO-003 (the secret redaction invariant still holds: the sentinel is not
//! echoed even in the error case).
//!
//! Story: S-DEMO-003 | BC: BC-2.03.007
//! Test vector: TV-BC-2.03.007-001 — secret value never appears on stdout after stdin input.

#![allow(clippy::unwrap_used)]
#![allow(non_snake_case)]

use std::{
    io::Write,
    path::PathBuf,
    process::{Command, Stdio},
};

/// Locate the compiled `prism` binary for subprocess testing.
///
/// Uses `CARGO_BIN_EXE_prism` set by cargo-nextest / cargo test when the [[bin]]
/// target is in the same workspace. Falls back to searching `target/debug/prism`
/// relative to the workspace root.
fn prism_binary() -> PathBuf {
    // cargo-nextest sets CARGO_BIN_EXE_prism automatically for [[bin]] targets.
    if let Ok(path) = std::env::var("CARGO_BIN_EXE_prism") {
        return PathBuf::from(path);
    }
    // Fallback: construct from CARGO_MANIFEST_DIR (crates/prism-bin) → workspace root.
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent() // crates/
        .and_then(|p| p.parent()) // workspace root
        .expect("could not locate workspace root from CARGO_MANIFEST_DIR");
    workspace_root.join("target").join("debug").join("prism")
}

/// The sentinel credential value used as the test oracle.
///
/// This value is chosen to be:
/// 1. Distinctive enough not to appear in prism's ordinary output.
/// 2. Not a real credential.
///
/// If this string appears in stdout or stderr output from `prism credential set`,
/// it means the subcommand echoed the credential value — a BC-2.03.007 / AD-017 violation.
const SENTINEL_SECRET: &str = "PRISM_DEMO_SECRET_SENTINEL_12345";

/// Red Gate test: BC-2.03.007 secret redaction — `prism credential set` must not echo
/// the credential value to stdout or stderr.
///
/// # Red Gate failure mechanism
///
/// The subprocess panics inside `handle_credential_set()` (`todo!()`). The panic hook
/// writes to stderr. `SENTINEL_SECRET` does NOT appear in the panic message (it was
/// the stdin input, not embedded in the source), so the secret-not-on-stderr assertion
/// would pass. HOWEVER, the `assert_eq!(exit_status.code(), Some(0))` assertion fails
/// because the subprocess exits 1 (panic exit code). This is the primary Red Gate failure.
///
/// After implementation, the test passes when:
/// - The subprocess exits 0 (success) OR exits 1 with "Keyring unavailable" (CI headless).
/// - Neither stdout nor stderr contains `SENTINEL_SECRET`.
#[test]
fn test_BC_2_03_007_prism_credential_set_does_not_echo_value_to_stdout() {
    let prism_bin = prism_binary();

    // Verify the binary exists — if not, give an actionable message.
    assert!(
        prism_bin.exists(),
        "prism binary not found at '{}'. Run `cargo build -p prism-bin` first. \
         (S-DEMO-003 Red Gate test requires a compiled binary)",
        prism_bin.display()
    );

    // Use a temporary config directory so the subprocess does not require
    // a real prism.toml to exist. The `credential set` subcommand loads config
    // to resolve the org slug — the stub implementation panics before reaching that
    // point, so this tempdir is sufficient for the Red Gate assertion.
    let config_dir = tempfile::tempdir().expect("failed to create temp config dir");

    // Spawn the `prism credential set` subprocess.
    // Key design choices:
    // - stdin: piped so we can write SENTINEL_SECRET as if a user typed it.
    // - stdout: piped so we can capture and assert the secret is absent.
    // - stderr: piped so we can capture and assert the secret is absent.
    // - No TTY: subprocess runs without a terminal, so rpassword must read from piped stdin.
    //
    // Note: rpassword::prompt_password detects non-TTY stdin and reads from stdin directly
    // (not /dev/tty). This is the correct behavior for subprocess testing.
    let mut child = Command::new(&prism_bin)
        .args([
            "--config-dir",
            config_dir.path().to_str().unwrap(),
            "credential",
            "set",
            "--sensor",
            "crowdstrike",
            "--name",
            "client_id",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap_or_else(|e| {
            panic!(
                "failed to spawn prism binary '{}': {e}",
                prism_bin.display()
            )
        });

    // Write the sentinel secret to the subprocess stdin as if a user typed it.
    // The newline terminates the rpassword read (simulates pressing Enter).
    {
        let stdin = child.stdin.as_mut().expect("subprocess stdin not piped");
        writeln!(stdin, "{SENTINEL_SECRET}").expect("failed to write sentinel to subprocess stdin");
        // stdin drops here, closing the pipe — signals EOF to the subprocess.
    }

    // Wait for the subprocess to exit and collect output.
    let output = child
        .wait_with_output()
        .expect("failed to wait for subprocess exit");

    let stdout_str = String::from_utf8_lossy(&output.stdout);
    let stderr_str = String::from_utf8_lossy(&output.stderr);

    // --- Primary Red Gate assertion ---
    // The subprocess must have exited (panicked at todo!() → exit 1 is expected at Red Gate).
    // The test is asserting exit 0 OR exit 1 with "Keyring unavailable" (headless CI).
    // At Red Gate, the binary panics: exit code is 1 (panic hook calls process::exit(1)).
    // This assertion FAILS at Red Gate, establishing the gate.
    let exit_code = output.status.code().unwrap_or(-1);
    assert!(
        exit_code == 0 || (exit_code == 1 && stderr_str.contains("Keyring unavailable")),
        "prism credential set must exit 0 on success, or exit 1 with 'Keyring unavailable' \
         on headless CI (EC-001 of S-DEMO-003). Got exit code {exit_code}.\n\
         stdout: {stdout_str}\n\
         stderr: {stderr_str}\n\
         (If the binary panicked with todo!(), implement handle_credential_set — S-DEMO-003)"
    );

    // --- BC-2.03.007 postcondition: secret MUST NOT appear on stdout ---
    // This is the core AD-017 / BC-2.03.007 assertion.
    // Even if the keyring is unavailable (exit 1 + "Keyring unavailable"),
    // the sentinel must not be echoed.
    assert!(
        !stdout_str.contains(SENTINEL_SECRET),
        "BC-2.03.007 VIOLATION: credential value appeared on stdout.\n\
         The sentinel value '{SENTINEL_SECRET}' must NEVER be printed to stdout \
         (AD-017 AI-opaque credential model; BC-2.03.007 postcondition: \
         'Secret Redaction in Logs, Errors, and MCP Responses').\n\
         Full stdout: {stdout_str}"
    );

    // --- BC-2.03.007 postcondition: secret MUST NOT appear on stderr ---
    // stderr carries the prompt ("Enter value for...") and error messages.
    // The credential VALUE must never transit stderr — only the prompt + error messages.
    assert!(
        !stderr_str.contains(SENTINEL_SECRET),
        "BC-2.03.007 VIOLATION: credential value appeared on stderr.\n\
         The sentinel value '{SENTINEL_SECRET}' must NEVER be printed to stderr \
         (AD-017 AI-opaque credential model; BC-2.03.007 postcondition).\n\
         Full stderr: {stderr_str}"
    );
}
