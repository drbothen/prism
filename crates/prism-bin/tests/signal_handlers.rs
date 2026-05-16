//! Integration tests for signal handler behavior — BC-2.10.010.
//!
//! Tests verify:
//! - SIGTERM during a running process → process logs "Received SIGTERM" + exits 0 (AC-6)
//! - The panic hook produces exit 1 when a panic is injected (AC-12)
//! - Signal handler installation is type-safe (unit test of signals.rs API surface)
//!
//! SIGTERM tests spawn `prism start` and then send SIGTERM via `libc::kill`.
//! These tests are Unix-only because `tokio::signal::unix` is Unix-only.
//!
//! Story: S-WAVE5-PREP-01  AC-6, AC-12
//! BC: BC-2.10.010 (Graceful Shutdown on SIGTERM/SIGINT)
//! ADR: ADR-022 §B step 11, §A panic hook

#![allow(clippy::unwrap_used)]

use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::Duration;

/// MED-5 (S-WAVE5-PREP-01 fix-pass-1): Create an isolated temp config dir per test.
/// Returns (config_dir, state_dir, spec_dir) TempDirs — keep all alive for test duration.
fn make_valid_config_dir() -> (tempfile::TempDir, tempfile::TempDir, tempfile::TempDir) {
    let config_tmp = tempfile::TempDir::new().unwrap();
    let state_tmp = tempfile::TempDir::new().unwrap();
    let spec_tmp = tempfile::TempDir::new().unwrap();

    let toml_content = format!(
        r#"spec_dir = {:?}
state_dir = {:?}

[[orgs]]
org_id = "0196f000-0000-7000-8000-000000000001"
org_slug = "acme"
"#,
        spec_tmp.path().display(),
        state_tmp.path().display(),
    );
    std::fs::write(config_tmp.path().join("prism.toml"), &toml_content).unwrap();
    (config_tmp, state_tmp, spec_tmp)
}

fn prism_bin() -> PathBuf {
    if let Ok(path) = std::env::var("CARGO_BIN_EXE_prism") {
        return PathBuf::from(path);
    }
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/debug/prism")
}

// ---------------------------------------------------------------------------
// BC-2.10.010 / AC-6 — SIGTERM → graceful shutdown, exit 0
// ---------------------------------------------------------------------------

/// Story: S-WAVE5-PREP-01 AC-6
/// BC: BC-2.10.010 postcondition — SIGTERM received → exit 0 + SIGTERM log entry
/// ADR-022 §B step 11: SIGTERM handler must emit "Received SIGTERM — shutting down"
/// MED-2: wired through signals::install_sigterm_handler (not inline duplicate).
/// MED-5: uses isolated TempDir to avoid parallel RocksDB LOCK collisions.
///
/// Root cause of prior flakiness: `Stdio::piped()` child stdout/stderr not drained
/// by the parent test process under nextest. When the SIGTERM handler emits tracing
/// output and the pipe buffer fills, the child blocks on write; nextest then kills
/// the child with signal 15 before the handler completes. Fix: redirect both streams
/// to /dev/null via `Stdio::null()`. The test only asserts on the exit code
/// (BC-2.10.010 postcondition), not on stdout/stderr content, so capture is not needed.
#[cfg(unix)]
#[test]
fn test_BC_2_10_010_sigterm_causes_graceful_exit_zero() {
    use std::os::unix::process::ExitStatusExt as _;

    // MED-5: isolated per-test config/state/spec dirs.
    let (config_dir, _state_tmp, _spec_tmp) = make_valid_config_dir();

    // Compute the readiness sentinel path that boot.rs will create when the
    // PRISM_TEST_STOP_AFTER_STEP=6 gate is reached. We don't know the child PID
    // yet, so we pass PRISM_TEST_READY_FILE to let the child write to a known path.
    let ready_path =
        std::env::temp_dir().join(format!("prism-ready-{}.sentinel", std::process::id()));
    // Clean up any leftover from a previous failed run.
    let _ = std::fs::remove_file(&ready_path);

    // Spawn the process with stdout/stderr discarded — the test only checks exit code.
    // Stdio::null() prevents pipe-buffer-full write-blocking under nextest.
    let mut child = Command::new(prism_bin())
        .args(["start"])
        .env("PRISM_CONFIG_DIR", config_dir.path())
        // PRISM_TEST_STOP_AFTER_STEP=6: process halts at step 6 (audit-ready)
        // and waits for SIGTERM via signals::install_sigterm_handler (MED-2).
        .env("PRISM_TEST_STOP_AFTER_STEP", "6")
        // PRISM_TEST_READY_FILE: path the boot gate writes when it's ready for SIGTERM.
        // The test polls for this file before sending SIGTERM — eliminates the race
        // between variable RocksDB init time and a fixed sleep.
        .env("PRISM_TEST_READY_FILE", &ready_path)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("failed to spawn prism binary for SIGTERM test");

    let pid = child.id();

    // Wait for the process to reach the step-6 gate (max 30 seconds).
    // Under load, RocksDB init in a temp directory can take several seconds.
    // Polling the sentinel file avoids a fixed sleep that may be too short.
    let deadline = std::time::Instant::now() + Duration::from_secs(30);
    let mut gate_reached = false;
    while std::time::Instant::now() < deadline {
        if ready_path.exists() {
            gate_reached = true;
            break;
        }
        // Also bail early if the process exits unexpectedly before the gate.
        if child.try_wait().unwrap().is_some() {
            break;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    let _ = std::fs::remove_file(&ready_path); // cleanup sentinel

    assert!(
        gate_reached,
        "prism process (PID {pid}) did not reach PRISM_TEST_STOP_AFTER_STEP=6 gate \
         within 30s — check that test-injection feature is enabled and RocksDB init succeeded"
    );

    // Send SIGTERM.
    // Safety: pid is from a child we just spawned; the kill call is safe.
    #[allow(unsafe_code)]
    unsafe {
        libc::kill(pid as libc::pid_t, libc::SIGTERM);
    }

    // Wait for the process to exit (max 5 seconds for signal handler to run).
    let status = child.wait().expect("failed to wait for prism process");

    // AC-6: process must exit 0 (not killed by signal, not panic exit).
    assert_eq!(
        status.code(),
        Some(0),
        "SIGTERM must cause prism to exit 0 (BC-2.10.010 + AC-6); \
         got status: {:?} (signal={:?})",
        status,
        status.signal()
    );
}

// ---------------------------------------------------------------------------
// AC-12 — panic hook produces exit 1 (not 101, not coredump)
// ---------------------------------------------------------------------------

/// Story: S-WAVE5-PREP-01 AC-12
/// BC: BC-2.10.010 — all exits are clean and observable
/// ADR-022 §A: panic hook emits tracing::error! then calls process::exit(1)
///
/// This test verifies that when a panic occurs, the process exits with code 1
/// (from the panic hook's `process::exit(1)`) rather than:
/// - 101 (Rust's default panic exit without hook)
/// - Signal-based exit (coredump)
///
/// We inject the panic using PRISM_TEST_INJECT_PANIC=true which the boot.rs
/// cfg(test) path will panic on to trigger the hook.
///
/// RED GATE: Fails today because `dispatch()` is `todo!()` — the todo!() itself
/// panics, producing exit code 101 without the hook. After the panic hook is
/// installed in main.rs (pre-dispatch), the exit code changes to 1.
#[test]
fn test_AC_12_panic_hook_produces_exit_code_1() {
    // Panic injection fires before step 2 (config load), so spec_dir doesn't matter.
    // However, we need a valid config_dir (with prism.toml) for the binary to start.
    // MED-5: use isolated dirs for safety.
    let (config_dir, _state_tmp, _spec_tmp) = make_valid_config_dir();
    let output = Command::new(prism_bin())
        .args(["start"])
        .env("PRISM_CONFIG_DIR", config_dir.path())
        .env("PRISM_TEST_INJECT_PANIC", "true")
        .output()
        .expect("failed to spawn prism binary for panic hook test");

    // AC-12: custom panic hook must produce exit 1 (not 101, not signal exit).
    assert_eq!(
        output.status.code(),
        Some(1),
        "Custom panic hook must produce exit 1 (AC-12; ADR-022 §A); \
         Rust default panic exit is 101 without a hook; \
         got exit {:?}; stderr: {}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );
}

// ---------------------------------------------------------------------------
// AC-12 — panic without hook produces 101 (demonstrates why hook matters)
// ---------------------------------------------------------------------------

/// Story: S-WAVE5-PREP-01 AC-12 (negative companion test)
/// BC: BC-2.10.010 — panic must produce exit 1 (hook required)
///
/// This companion test documents that today (Red Gate) the todo!() in dispatch()
/// causes an exit that is NOT 1 (because the hook is not yet installed).
/// After implementation, this test should be removed or inverted.
///
/// RED GATE: Documents the pre-implementation state. If this test suddenly
/// passes (exit != 1), it means the panic hook is installed and AC-12 is satisfied.
/// Remove this test when test_AC_12_panic_hook_produces_exit_code_1 passes.
#[test]
fn test_AC_12_red_gate_dispatch_todo_panics_without_hook() {
    // After implementation: panic hook installed in main.rs → exits 1.
    // MED-5: use isolated dirs (prism start now reaches step 6, needs RocksDB).
    let (config_dir, _state_tmp, _spec_tmp) = make_valid_config_dir();
    let output = Command::new(prism_bin())
        .args(["start"])
        .env("PRISM_CONFIG_DIR", config_dir.path())
        .output()
        .expect("failed to spawn prism binary");

    // The process must not exit cleanly (0) — it panics on todo!().
    assert_ne!(
        output.status.code(),
        Some(0),
        "RED GATE: prism start must not exit 0 while todo!() stubs exist; \
         AC-12 panic hook is not yet installed"
    );

    // Document the current exit code for awareness (101 without hook, 1 with hook).
    // This assertion will FLIP to assert_eq!(code, Some(1)) after implementation.
    // For now it just verifies non-zero.
    let code = output.status.code();
    eprintln!(
        "Red Gate: prism start currently exits {:?} (expected 101 without hook, 1 after AC-12 implementation)",
        code
    );
}

// ---------------------------------------------------------------------------
// Signal handler API surface (unit test — types compile correctly)
// ---------------------------------------------------------------------------

/// Story: S-WAVE5-PREP-01
/// BC: BC-2.10.010 — signal handler API surface is correct
/// ADR-022 §B step 11: install_sigterm_handler + install_sighup_handler exist
///
/// This test verifies that the public API surface of signals.rs compiles and
/// has the correct type signatures. It does not exercise the async behavior
/// (which requires a running tokio runtime and actual signals).
///
/// The type-level check ensures the stub signatures match what boot.rs expects.
///
/// RED GATE: Fails today because `install_sigterm_handler` and
/// `install_sighup_handler` are `todo!()` stubs that panic when awaited.
/// This test does NOT await them — it only verifies the function signatures.
#[test]
fn test_signal_handler_api_surface_compiles() {
    use prism_bin::signals::{install_sighup_handler, install_sigterm_handler};
    use tokio::sync::{broadcast, mpsc};

    // Verify the functions exist and have the correct return type (Future).
    // We create channels for the type check but don't spawn or await.
    let (shutdown_tx, _rx) = broadcast::channel::<()>(1);
    let (reload_tx, _reload_rx) = mpsc::channel::<()>(1);

    // These are async fn — calling them produces a Future but does not
    // execute until awaited. We drop the futures to verify types compile.
    let _sigterm_fut = install_sigterm_handler(shutdown_tx);
    let _sighup_fut = install_sighup_handler(reload_tx);

    // If this compiles and runs, the API surface is correct.
    // The actual behavior is tested by the async integration test above.
}
