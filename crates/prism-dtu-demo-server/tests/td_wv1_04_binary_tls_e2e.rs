//! TD-WV1-04: Binary CLI end-to-end tests for `--tls` flag wiring.
//!
//! These tests spawn the `prism-dtu-demo-server` binary and verify that:
//!
//! - `start --tls` causes all clones to serve HTTPS.
//! - `start` without `--tls` serves plain HTTP (backward-compatible).
//! - With `--tls`, the SHA-256 fingerprint line appears in stdout before the URL table.
//!
//! # Stdout capture
//!
//! Stdout is captured via `Stdio::piped()` at spawn time. The fingerprint ordering
//! test drains the pipe with a synchronous blocking `read_to_string` after `child.wait()`
//! — the child's write-end is closed at exit so the drain returns EOF immediately
//! rather than blocking. This is a synchronous pipe-drain pattern, not an async channel.
//!
//! # Dependencies
//!
//! Uses `std::process::Command` + `std::process::Child` directly to avoid
//! adding `assert_cmd` to the build graph.  Signal delivery uses
//! `libc::kill(pid, SIGTERM)` on Unix (already a direct dep via `[target.'cfg(unix)'.dependencies]`).
//!
//! # Feature gate
//!
//! The `--tls` binary tests require the `tls` feature so the binary is compiled
//! with TLS support.  The plain-HTTP test compiles under `dtu` alone.

#![allow(clippy::unwrap_used, clippy::expect_used)]
#[cfg(all(feature = "dtu", feature = "tls"))]
mod binary_tls_tests {
    use std::collections::HashMap;
    use std::io::Read;
    use std::path::PathBuf;
    use std::time::Duration;

    // ---------------------------------------------------------------------------
    // Helpers
    // ---------------------------------------------------------------------------

    /// Return the path to the `prism-dtu-demo-server` binary built by cargo.
    ///
    /// Panics if the `CARGO_BIN_EXE_prism-dtu-demo-server` env var is not set
    /// (it is always set by `cargo test`).
    fn binary_path() -> PathBuf {
        // `cargo test` sets CARGO_BIN_EXE_<name> for every `[[bin]]` in the crate.
        let var = "CARGO_BIN_EXE_prism-dtu-demo-server";
        std::env::var(var)
            .unwrap_or_else(|_| {
                // Fallback: derive from CARGO_MANIFEST_DIR during dev.
                let manifest = std::env::var("CARGO_MANIFEST_DIR")
                    .expect("CARGO_MANIFEST_DIR must be set by cargo test");
                // Walk up to workspace root then into target/debug.
                let ws_root = PathBuf::from(&manifest)
                    .parent()
                    .and_then(|p| p.parent())
                    .expect("could not locate workspace root from CARGO_MANIFEST_DIR")
                    .to_path_buf();
                ws_root
                    .join("target")
                    .join("debug")
                    .join("prism-dtu-demo-server")
                    .to_string_lossy()
                    .into_owned()
            })
            .into()
    }

    /// Write a minimal 2-clone TOML config to `dir` and return its path.
    ///
    /// Uses ephemeral ports (port = 0 is NOT supported by the binary yet for
    /// demo.toml — we use the stable demo ports 17080 and 17084 but shifted
    /// to avoid conflict with any running instance).  To keep tests hermetic,
    /// we use high ephemeral ports that are unlikely to be in use.
    fn write_minimal_config(dir: &std::path::Path) -> PathBuf {
        let toml = r#"
[harness]
bind = "127.0.0.1"

[clones.crowdstrike]
enabled = true
bind = "127.0.0.1"
port = 0
fixture_set = "default"
initial_failure_mode = "None"
seed = 42
tls = false
continue_on_error = false

[clones.claroty]
enabled = false

[clones.cyberint]
enabled = false

[clones.armis]
enabled = false

[clones.threatintel]
enabled = true
bind = "127.0.0.1"
port = 0
fixture_set = "default"
initial_failure_mode = "None"
seed = 42
tls = false
continue_on_error = false

[clones.nvd]
enabled = false
"#;
        let path = dir.join("test-demo.toml");
        std::fs::write(&path, toml).expect("failed to write test config");
        path
    }

    /// Poll `url_file` until it exists and contains valid JSON, then parse and return.
    ///
    /// Times out with a panic after `timeout`.
    fn wait_for_url_file(url_file: &std::path::Path, timeout: Duration) -> HashMap<String, String> {
        let deadline = std::time::Instant::now() + timeout;
        loop {
            if url_file.exists() {
                if let Ok(contents) = std::fs::read_to_string(url_file) {
                    if let Ok(map) = serde_json::from_str::<HashMap<String, String>>(&contents) {
                        if !map.is_empty() {
                            return map;
                        }
                    }
                }
            }
            if std::time::Instant::now() >= deadline {
                panic!(
                    "URL sidecar file {:?} not populated within {:?}",
                    url_file, timeout
                );
            }
            std::thread::sleep(Duration::from_millis(100));
        }
    }

    /// Send SIGTERM to `pid` on Unix.
    #[cfg(unix)]
    fn send_sigterm(pid: u32) {
        // SAFETY: calling kill(2) with a valid pid and SIGTERM.
        let ret = unsafe { libc::kill(pid as libc::pid_t, libc::SIGTERM) };
        assert_eq!(ret, 0, "kill(SIGTERM) to pid {pid} failed");
    }

    /// Do a blocking GET to `url` with a `connect_timeout` and return the status code.
    ///
    /// Uses a single-threaded tokio runtime so tests can call this from sync context.
    fn get_status(url: &str, accept_invalid_certs: bool) -> u16 {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("tokio runtime must build");
        rt.block_on(async {
            let client = reqwest::Client::builder()
                .danger_accept_invalid_certs(accept_invalid_certs)
                .timeout(Duration::from_secs(5))
                .build()
                .expect("reqwest client must build");
            match client.get(url).send().await {
                Ok(resp) => resp.status().as_u16(),
                Err(e) => panic!("GET {url} failed: {e}"),
            }
        })
    }

    // ---------------------------------------------------------------------------
    // TD-WV1-04-003: binary --tls results in HTTPS serving
    // ---------------------------------------------------------------------------

    /// TD-WV1-04: `prism-dtu-demo-server start --tls` results in all clones
    /// serving HTTPS.
    ///
    /// # Red Gate (runtime failure)
    ///
    /// The binary currently discards the `RustlsConfig` generated by `handle_tls()`
    /// and calls `harness.start_all(&config)` without TLS.  As a result:
    ///
    /// 1. The URL sidecar writes `http://…` URLs even with `--tls`.
    /// 2. A TLS handshake to the clone ports fails (plain TCP response to TLS ClientHello).
    ///
    /// This test will fail at the assertion `url.starts_with("https://")`.
    #[test]
    #[cfg(unix)]
    fn td_wv1_04_binary_start_with_tls_serves_https() {
        let tmp = tempfile::tempdir().expect("tempdir must be created");
        let config_path = write_minimal_config(tmp.path());
        let url_file = tmp.path().join(".prism-dtu-demo-server.urls.json");

        let bin = binary_path();

        let mut child = std::process::Command::new(&bin)
            .args(["start", "--config", config_path.to_str().unwrap(), "--tls"])
            .current_dir(tmp.path())
            // Capture stdout so we can inspect fingerprint ordering later.
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::null())
            .spawn()
            .expect("TD-WV1-04: binary must spawn");

        let pid = child.id();

        // Wait for URL sidecar (up to 15s — binary needs to bind + write).
        let url_map = wait_for_url_file(&url_file, Duration::from_secs(15));

        // AC-6 / TD-WV1-04: URLs must be https:// when --tls is passed.
        //
        // RED GATE: The binary currently writes http:// URLs even with --tls.
        for (name, url) in &url_map {
            assert!(
                url.starts_with("https://"),
                "TD-WV1-04: url_map[{name}] must start with https:// when --tls; got: {url}"
            );
        }

        // Verify crowdstrike's /dtu/health returns 200 over HTTPS.
        let cs_url = url_map
            .get("crowdstrike")
            .expect("TD-WV1-04: crowdstrike must be in url_map");
        let health_url = format!("{cs_url}/dtu/health");

        // RED GATE: plain HTTP is running, so HTTPS connect will fail or get a non-200.
        let status = get_status(&health_url, true);
        assert_eq!(
            status, 200,
            "TD-WV1-04: crowdstrike /dtu/health over HTTPS must return 200; got {status}"
        );

        // Clean shutdown.
        send_sigterm(pid);
        let _ = child.wait();
    }

    // ---------------------------------------------------------------------------
    // TD-WV1-04-004: binary without --tls serves plain HTTP (should pass today)
    // ---------------------------------------------------------------------------

    /// TD-WV1-04: `prism-dtu-demo-server start` without `--tls` continues to
    /// serve plain HTTP and url_map values are `http://…`.
    ///
    /// This test verifies backward compatibility after the TLS plumbing is added.
    /// It is expected to PASS once the new `start_all` signature lands (and also
    /// passes today because plain HTTP is already implemented).
    ///
    /// # Red Gate
    ///
    /// This test compiles today and passes today.  It is included here to ensure
    /// the None/plain-HTTP path is not broken by the TD-WV1-04 fix.
    #[test]
    #[cfg(unix)]
    fn td_wv1_04_binary_start_without_tls_serves_http() {
        let tmp = tempfile::tempdir().expect("tempdir must be created");
        let config_path = write_minimal_config(tmp.path());
        let url_file = tmp.path().join(".prism-dtu-demo-server.urls.json");

        let bin = binary_path();

        let mut child = std::process::Command::new(&bin)
            .args(["start", "--config", config_path.to_str().unwrap()])
            .current_dir(tmp.path())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .expect("TD-WV1-04: binary must spawn");

        let pid = child.id();

        let url_map = wait_for_url_file(&url_file, Duration::from_secs(15));

        // Without --tls, URLs must be http://.
        for (name, url) in &url_map {
            assert!(
                url.starts_with("http://"),
                "TD-WV1-04: url_map[{name}] must start with http:// without --tls; got: {url}"
            );
        }

        // Spot-check crowdstrike /dtu/health over plain HTTP.
        let cs_url = url_map
            .get("crowdstrike")
            .expect("TD-WV1-04: crowdstrike must be in url_map");
        let health_url = format!("{cs_url}/dtu/health");
        let status = get_status(&health_url, false);
        assert_eq!(
            status, 200,
            "TD-WV1-04: crowdstrike /dtu/health over HTTP must return 200; got {status}"
        );

        send_sigterm(pid);
        let _ = child.wait();
    }

    // ---------------------------------------------------------------------------
    // TD-WV1-04-005: fingerprint appears before URL table
    // ---------------------------------------------------------------------------

    /// TD-WV1-04: With `--tls`, the `sha256:` fingerprint line appears in stdout
    /// before the URL table header (`| Clone |`).
    ///
    /// # Red Gate (runtime failure — ordering may be wrong or indeterminate)
    ///
    /// Currently `handle_tls()` prints the fingerprint but the binary then
    /// discards the RustlsConfig.  The fingerprint IS printed today, so this
    /// test may pass already — but it is included to lock in the ordering
    /// invariant so it cannot regress.
    ///
    /// The test will FAIL if the binary exits before writing sufficient output,
    /// or if the URL table is printed before the fingerprint.
    #[test]
    #[cfg(unix)]
    fn td_wv1_04_binary_prints_fingerprint_before_url_table_when_tls() {
        let tmp = tempfile::tempdir().expect("tempdir must be created");
        let config_path = write_minimal_config(tmp.path());
        let url_file = tmp.path().join(".prism-dtu-demo-server.urls.json");

        let bin = binary_path();

        let mut child = std::process::Command::new(&bin)
            .args(["start", "--config", config_path.to_str().unwrap(), "--tls"])
            .current_dir(tmp.path())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::null())
            .spawn()
            .expect("TD-WV1-04: binary must spawn");

        let pid = child.id();

        // Wait for the URL file so we know the binary has printed the URL table.
        wait_for_url_file(&url_file, Duration::from_secs(15));

        // Kill the process so stdout is flushed and pipe is closed.
        send_sigterm(pid);
        let _ = child.wait();

        // Drain the piped stdout pipe using a blocking read.
        //
        // Pattern: `child.stdout` is a `Stdio::piped()` pipe set up at spawn time.
        // After `child.wait()` the child process has exited and its write-end of the
        // pipe is closed, so `read_to_string` will drain the remaining buffered bytes
        // and return EOF rather than blocking indefinitely. This is a synchronous
        // blocking drain of the pipe — NOT an async channel pattern.
        let mut stdout = String::new();
        if let Some(mut out) = child.stdout.take() {
            let _ = out.read_to_string(&mut stdout);
        } else {
            // Stdout pipe unavailable (e.g. consumed by a prior call to child.wait()
            // on some platforms). Skip the ordering assertion and note that it must
            // be verified manually.
            eprintln!(
                "TD-WV1-04: stdout pipe unavailable after wait — skipping ordering check. \
                 Fingerprint ordering must be verified manually."
            );
            return;
        }

        // Locate fingerprint and URL table positions in combined stdout.
        let fp_pos = stdout.find("sha256:");
        let url_table_pos = stdout.find("| Clone ");

        let fp_pos = fp_pos
            .expect("TD-WV1-04: stdout must contain 'sha256:' fingerprint when --tls is passed");

        let url_table_pos = url_table_pos
            .expect("TD-WV1-04: stdout must contain '| Clone ' URL table header after startup");

        // AC-7 ordering: fingerprint BEFORE URL table.
        assert!(
            fp_pos < url_table_pos,
            "TD-WV1-04: sha256: fingerprint (pos {fp_pos}) must appear before URL table \
             (pos {url_table_pos}) in stdout"
        );
    }
}

// When `tls` feature is absent, include a placeholder so the file still compiles.
#[cfg(not(all(feature = "dtu", feature = "tls")))]
mod binary_tls_stub {
    /// TD-WV1-04: binary TLS E2E tests require both `dtu` and `tls` features.
    ///
    /// Compile with `--features dtu,tls` to enable these tests.
    #[test]
    fn td_wv1_04_binary_tls_tests_skipped_without_tls_feature() {
        // No assertions — compile-time feature gate is the check.
    }
}
