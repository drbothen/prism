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

/// Subprocess test: BC-2.03.007 secret redaction — `prism credential set` must not echo
/// the credential value to stdout or stderr.
///
/// # F-P10-CRIT-001 compatibility: #[ignore] — spawns real `prism credential set`
///
/// With real platform backends now enabled (F-P10-CRIT-001 fix), the `prism` binary
/// will try to write to the actual OS Keychain (macOS) or libsecret (Linux) when
/// `prism credential set` is invoked as a subprocess. On macOS, this can trigger a
/// user-visible Keychain access prompt — which is UNACCEPTABLE for `just check` and
/// CI runs.
///
/// **SID-1 §4 deferral (S-DEMO-003 / F-P10-CRIT-001):**
///
/// The load-bearing no-echo coverage is provided in-process by:
///   - `test_handle_credential_set_writes_org_id_keyed_namespace` in
///     `bc_2_03_007_credential_set_org_id_keyed.rs` — exercises the full
///     `handle_credential_set_with_store` code path with `InMemoryCredentialStore`
///     (no OS keychain access). The code path is the SAME whether the store is
///     InMemory or Keyring; the "no println!(value)" invariant is structural.
///   - `test_handle_credential_set_writes_org_id_keyed_namespace` calls
///     `handle_credential_set_with_store` which NEVER calls `println!` with the value
///     (AD-017 invariant enforced by code structure, not runtime behavior). This is
///     verifiable by static code inspection: the value is obtained via `read_secret_value_from`
///     and passed directly to `store.set_by_org`; no stdout/stderr write of the value occurs.
///
/// To run this test on a machine with a signed binary AND confirmed non-prompting keychain:
/// ```text
/// cargo test -p prism-bin --test bc_2_03_007_credential_set_no_echo -- --ignored
/// ```
///
/// F-P10-CRIT-001 (zero-keychain-prompts-in-tests) / SID-1 §4 / BC-2.03.007 / AD-017.
#[test]
#[ignore = "spawns real `prism credential set` subprocess → real OS Keychain (macOS) / libsecret (Linux); \
            triggers Keychain access prompt on macOS with real backends (F-P10-CRIT-001). \
            In-process no-echo coverage: test_handle_credential_set_writes_org_id_keyed_namespace \
            in bc_2_03_007_credential_set_org_id_keyed.rs (SID-1 §4 documented deferral; S-DEMO-003)."]
fn test_BC_2_03_007_prism_credential_set_does_not_echo_value_to_stdout() {
    let prism_bin = prism_binary();

    // Verify the binary exists — if not, give an actionable message.
    assert!(
        prism_bin.exists(),
        "prism binary not found at '{}'. Run `cargo build -p prism-bin` first. \
         (S-DEMO-003 Red Gate test requires a compiled binary)",
        prism_bin.display()
    );

    // Use a temporary config directory with a minimal prism.toml fixture.
    // ADR-034 §D3: `handle_credential_set` requires prism.toml to resolve the OrgId
    // for OrgId-keyed write. The UUID-v5 fallback was removed (AD-017 / ADR-034 §D3).
    // Without a prism.toml, the binary exits 2 (config-invalid) before reading stdin
    // — which means this test would assert exit 2, but the actual AD-017 invariant
    // (secret not on stdout/stderr) still holds (the sentinel is never echoed).
    // However, to exercise the FULL keyring write path (and catch any future echo bug),
    // we provide a minimal prism.toml with a "demo-org" org entry.
    let config_dir = tempfile::tempdir().expect("failed to create temp config dir");

    // Write the prism.toml fixture with a demo-org entry (matching --org-slug "demo-org").
    {
        let state_dir = config_dir.path().join("state");
        let spec_dir = config_dir.path().join("specs");
        let plugin_dir = config_dir.path().join("plugins");
        std::fs::create_dir_all(&state_dir).expect("create state dir");
        std::fs::create_dir_all(&spec_dir).expect("create spec dir");
        std::fs::create_dir_all(&plugin_dir).expect("create plugin dir");

        let prism_toml = format!(
            r#"spec_dir = "{spec}"
state_dir = "{state}"
plugin_dir = "{plugin}"

[[orgs]]
org_id = "0196f4b2-3c8d-7e1a-b5f0-2d4c6e8a0b1c"
org_slug = "demo-org"
"#,
            spec = spec_dir.display(),
            state = state_dir.display(),
            plugin = plugin_dir.display(),
        );
        std::fs::write(config_dir.path().join("prism.toml"), &prism_toml)
            .expect("write prism.toml fixture");
    }

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
            // ADR-034 §D3 HIGH-3: --org-slug required when prism.toml is absent.
            // Without --org-slug, the binary now returns exit 2 (no demo-org fallback).
            // The test provides --org-slug directly to isolate the AD-017 no-echo property.
            "--org-slug",
            "demo-org",
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
    // Expected outcomes:
    //   0 — success (keyring available, credential stored).
    //   1 — keyring unavailable (headless CI; stderr contains "Keyring unavailable").
    //       The keyring write failed but the secret was never echoed — AD-017 still holds.
    //   1 — keyring write failed for another reason (stderr contains "keyring write failed").
    // NOT expected:
    //   2 — config error (would mean prism.toml fixture was not loaded correctly).
    //   panic/abort — would mean handle_credential_set contains todo!().
    let exit_code = output.status.code().unwrap_or(-1);
    let keyring_failed = exit_code == 1
        && (stderr_str.contains("Keyring unavailable")
            || stderr_str.contains("keyring write failed")
            || stderr_str.contains("set_password failed")
            || stderr_str.contains("spawn_blocking panicked"));
    assert!(
        exit_code == 0 || keyring_failed,
        "prism credential set must exit 0 on success, or exit 1 on keyring failure \
         (EC-001 of S-DEMO-003). Got exit code {exit_code}.\n\
         stdout: {stdout_str}\n\
         stderr: {stderr_str}\n\
         (If exit 2: prism.toml fixture may not have been written correctly. \
         If panic: implement handle_credential_set — S-DEMO-003)"
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
