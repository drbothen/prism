//! Regression test for BC-2.10.006 — MCP stdout channel purity (stdio transport).
//!
//! Story: S-DEMO-004
//! BC: BC-2.10.006 §Postconditions (stdout purity / stderr-only logging)
//! Regression: M2-02 / LOCAL adversary pass-2 finding (S-DEMO-004)
//!
//! # Invariant (BC-2.10.006 §Postconditions / §Invariants)
//!
//! All tracing output MUST go exclusively to stderr. stdout is reserved for the
//! MCP JSON-RPC channel in `prism start` mode; any tracing bytes on stdout corrupt
//! the MCP framing and cause every connected client to receive garbled JSON responses.
//!
//! Invariant source: BC-2.10.006 §Postconditions:
//!   - "stdout is reserved exclusively for MCP JSON-RPC protocol messages"
//!   - "All logging, diagnostics, and metrics are written to stderr (via `tracing_subscriber`)"
//!
//! BC-2.10.006 §Invariants:
//!   - "stdout purity: no non-MCP content ever written to stdout"
//!
//! # Enforcement site
//!
//! The invariant is enforced at boot.rs `step1_init_tracing` (BC-2.22.001 §step1),
//! which calls `.with_writer(std::io::stderr)` on every `tracing_subscriber::fmt` layer.
//! Without this, `tracing-subscriber 0.3.x` defaults to `io::stdout`.
//!
//! # Test profile
//!
//! This test runs in the STANDARD nextest profile (no `required-features`,
//! no `#[ignore]`). It requires only the `prism` binary and a minimal temp config —
//! NO DTU clones, no external services.
//!
//! Discovery: auto-discovered by cargo (no `[[test]]` entry required) and included
//! in `just iter prism-bin` (no `--all-features` needed).

#![allow(clippy::unwrap_used)]

use std::{path::PathBuf, process::Command};

// ---------------------------------------------------------------------------
// Helpers (mirrored from bc_2_22_001_boot_orchestration.rs / inline here to
// avoid a required-features dependency on the injection-gated test binary)
// ---------------------------------------------------------------------------

/// Create an isolated temp config dir for a single test.
///
/// Returns (config_dir, state_dir, spec_dir) TempDirs — keep all alive for test duration.
/// MED-5 precedent: isolated per-test dirs to avoid parallel RocksDB LOCK collisions.
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

/// Locate the `prism` binary under test.
fn prism_bin() -> PathBuf {
    if let Ok(path) = std::env::var("CARGO_BIN_EXE_prism") {
        return PathBuf::from(path);
    }
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/debug/prism")
}

// ---------------------------------------------------------------------------
// BC-2.10.006 — MCP stdout channel purity: tracing must never pollute stdout
// ---------------------------------------------------------------------------

/// Story: S-DEMO-004
/// BC: BC-2.10.006 §Postconditions (stdout purity / stderr-only logging)
/// Enforcement site: boot.rs `step1_init_tracing` (BC-2.22.001 §step1)
/// Regression: M2-02 / LOCAL adversary pass-2 finding (S-DEMO-004)
///
/// # What this test guards
///
/// Spawns `prism start` with `RUST_LOG=info` and `PRISM_LOG_FORMAT=json` — conditions
/// under which the tracing subscriber IS active and would emit structured JSON log
/// lines to stdout IF `.with_writer(std::io::stderr)` were absent from any fmt layer.
///
/// The test asserts that stdout captures ZERO bytes carrying tracing fingerprints.
///
/// # Discriminating power (why it would fail if the guard were removed)
///
/// If `.with_writer(std::io::stderr)` were removed from either `LogFormat::Json`
/// or `LogFormat::Pretty` arm of `step1_init_tracing` (boot.rs), the very first
/// `tracing::info!("Prism v{}", ...)` call emits a JSON log line like
/// `{"timestamp":"…","level":"INFO","message":"Prism v…"}` onto stdout.
/// The assertions below check for `"level"` and `"message"` — fields that appear in
/// EVERY tracing JSON log line — so the test would immediately fail on the first
/// log emission.
///
/// # Boot path
///
/// `make_valid_config_dir()` produces a valid `prism.toml` with one org and no sensor
/// TOML specs. With `stdin = /dev/null`, `prism start`:
///   1. Runs step 1 (tracing init — emits "Prism v…" INFO log to stderr, NOT stdout).
///   2. Runs steps 2–6 (config load, org registry, sensor spec load [empty], cred init,
///      audit init). These may succeed or fail depending on storage init.
///   3. Exits when stdin EOF triggers MCP server shutdown.
///
/// Crucially, step 1 fires BEFORE the process exits, so the tracing subscriber IS
/// exercised and any stdout leak would be detected.
///
/// # regression: M2-02 / MCP stdout JSON-RPC channel must stay free of tracing
/// # (boot.rs with_writer(stderr))
#[test]
fn test_BC_2_10_006_tracing_never_pollutes_mcp_stdout() {
    // MED-5: isolated per-test dirs to avoid parallel RocksDB LOCK collisions.
    let (config_dir, _state_tmp, _spec_tmp) = make_valid_config_dir();

    let output = Command::new(prism_bin())
        .args(["start"])
        .env("PRISM_CONFIG_DIR", config_dir.path())
        // RUST_LOG=info: force tracing to be active and emit at info level.
        // Without with_writer(stderr), every log line would land on stdout.
        .env("RUST_LOG", "info")
        // PRISM_LOG_FORMAT=json: structured JSON lines — the easiest format to
        // detect. A tracing leak produces `{"level":"INFO","message":"Prism v..."}`.
        .env("PRISM_LOG_FORMAT", "json")
        // stdin=null: no MCP client; serve_stdio exits on stdin EOF so the process
        // completes without needing a connected client or a timeout.
        .stdin(std::process::Stdio::null())
        .output()
        .expect("failed to spawn prism binary");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // ---- Assert 1: no tracing JSON field "level" on stdout ----
    //
    // Every tracing JSON log line (compact or structured) contains the "level" field.
    // If tracing leaks to stdout, the boot line `"Prism v..."` makes stdout contain
    // `"level"` before any MCP client connects.
    assert!(
        !stdout.contains("\"level\""),
        "tracing JSON field 'level' must not appear on stdout; \
         stdout is reserved for MCP JSON-RPC framing \
         (BC-2.10.006 §Postconditions: stdout purity / stderr-only logging); \
         regression: M2-02 (boot.rs with_writer(stderr) on all fmt layers); \
         stdout captured:\n{stdout}"
    );

    // ---- Assert 2: no tracing JSON field "message" on stdout ----
    //
    // Every tracing JSON log line contains a "message" field. Belt-and-suspenders
    // check complementing Assert 1.
    assert!(
        !stdout.contains("\"message\""),
        "tracing JSON field 'message' must not appear on stdout; \
         BC-2.10.006 §Postconditions: all logging to stderr; \
         regression: M2-02 (boot.rs with_writer(stderr)); \
         stdout captured:\n{stdout}"
    );

    // ---- Assert 3: no event_type tracing fields on stdout ----
    //
    // Prism's structured events use `event_type = "..."` fields (BC-2.16.002).
    // Any such field appearing on stdout indicates a tracing leak.
    assert!(
        !stdout.contains("event_type"),
        "tracing structured field 'event_type' must not appear on stdout; \
         BC-2.10.006 §Invariants: stdout purity — no non-MCP content ever written to stdout; \
         regression: M2-02; \
         stdout captured:\n{stdout}"
    );

    // ---- Assert 4: no tracing pretty-format level markers on stdout ----
    //
    // This guard catches leakage when PRISM_LOG_FORMAT is overridden to "pretty"
    // externally. Pretty-format lines contain `  INFO `, `  WARN `, `  ERROR `.
    assert!(
        !stdout.contains("  INFO") && !stdout.contains("  WARN") && !stdout.contains("  ERROR"),
        "tracing pretty-format level markers must not appear on stdout; \
         BC-2.10.006 §Postconditions: all diagnostics to stderr; \
         regression: M2-02 (boot.rs with_writer(stderr)); \
         stdout captured:\n{stdout}"
    );

    // ---- Assert 5: every non-empty line on stdout is valid MCP JSON-RPC ----
    //
    // With stdin=null (no MCP client), the MCP server starts but stdin EOF causes
    // immediate shutdown before any client sends requests. stdout must therefore be
    // empty OR contain only valid MCP JSON-RPC framing (lines starting with `{"jsonrpc"`).
    // Any non-empty, non-jsonrpc line is definitionally a tracing or non-protocol leak.
    for line in stdout.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        assert!(
            trimmed.starts_with("{\"jsonrpc\""),
            "non-MCP-JSON-RPC content found on stdout — this is a protocol-channel leak; \
             BC-2.10.006 §Invariants: stdout purity — no non-MCP content ever written to stdout; \
             regression: M2-02; \
             offending line: {trimmed:?}; \
             full stdout:\n{stdout}"
        );
    }
}
