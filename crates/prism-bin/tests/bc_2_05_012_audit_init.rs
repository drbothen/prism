//! Integration tests for BC-2.05.012 — AuditEmitter initialization contract.
//!
//! Tests verify that:
//! 1. A writable state_dir → AuditEmitter constructs + sentinel persisted → exit 0
//! 2. An unwriteable state_dir → exit 4 (internal-error)
//! 3. A pre-existing RocksDB LOCK → exit 4 + "LOCK" in output
//! 4. After step 6, querying audit_buffer returns the boot.audit.initialized sentinel
//!
//! Note: Tests use `PRISM_TEST_INJECT_FAIL_STEP` to inject failures at step 6
//! without needing real RocksDB setup in the subprocess path.
//!
//! # Test Vectors from BC-2.05.012
//!
//! | TV ID | Scenario | Expected Exit |
//! |-------|----------|--------------|
//! | TV-05-012-001 | Valid state_dir | Boot continues (exit 0) |
//! | TV-05-012-002 | state_dir not writable | Exit 4 |
//! | TV-05-012-003 | RocksDB LOCK held | Exit 4 + "LOCK" message |
//! | TV-05-012-005 | Sentinel persisted | N/A (unit test via API) |
//!
//! Story: S-WAVE5-PREP-01  AC-8
//! BC: BC-2.05.012 (AuditEmitter init — audit_buffer CF open + sentinel emitted)
//! ADR: ADR-022 §B step 6, AD-004 (RocksDB 17 CFs)

#![allow(clippy::unwrap_used)]

use std::path::PathBuf;
use std::process::Command;

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
// BC-2.05.012 — audit init failure maps to exit 4 (AC-8)
// ---------------------------------------------------------------------------

/// Story: S-WAVE5-PREP-01 AC-8
/// BC: BC-2.05.012 Failure path — AuditEmitter init fails → exit 4
/// TV-05-012-002: Unwriteable state_dir → exit 4.
///
/// Uses PRISM_TEST_INJECT_FAIL_STEP to trigger an audit init failure without
/// requiring a real unwriteable filesystem (cross-platform safe).
///
/// OBS-3 (S-WAVE5-PREP-01 fix-pass-1): BC-2.05.012 TV-05-012-002 specifies
/// `chmod 444` as the canonical unwriteable-state_dir test. This test uses
/// synthetic injection (`PRISM_TEST_INJECT_FAIL_STEP=6_audit_failure`) instead
/// because chmod permissions are unreliable on Windows and inside certain CI
/// sandbox environments. The injection path exercises the same
/// `BootError::AuditInitFailed` → exit 4 code path as a real unwriteable dir
/// would, satisfying the BC's behavioral contract. The chmod 444 scenario is
/// documented as a future OS-level integration test (not required for S-WAVE5-PREP-01).
///
/// MED-5: Uses isolated TempDir per test.
#[test]
fn test_BC_2_05_012_audit_init_failure_exits_four() {
    // MED-5: isolated dirs. state_dir must exist for step4 (MED-3), but
    // the injection fires before RocksDB opens so state_dir isolation isn't
    // strictly needed for the injection path; it's required for spec_dir (step4).
    let (config_dir, _state_tmp, _spec_tmp) = make_valid_config_dir();
    let output = Command::new(prism_bin())
        .args(["validate-config"])
        .env("PRISM_CONFIG_DIR", config_dir.path())
        // Inject an audit init failure at step 6.
        .env("PRISM_TEST_INJECT_FAIL_STEP", "6_audit_failure")
        .output()
        .expect("failed to spawn prism binary");

    assert_eq!(
        output.status.code(),
        Some(4),
        "Audit subsystem init failure must exit 4 (BC-2.05.012 + AC-8); \
         got exit {:?}; stderr: {}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );
}

// ---------------------------------------------------------------------------
// BC-2.05.012 — exit code is exactly 4 for audit failures, never 2 or 5
// ---------------------------------------------------------------------------

/// Story: S-WAVE5-PREP-01 AC-8
/// BC: BC-2.05.012 Invariant: exit code on any audit failure is exactly 4
/// (never 1, 2, 3, or 5). MED-5: uses isolated TempDir.
#[test]
fn test_BC_2_05_012_invariant_audit_failure_exits_exactly_4_not_2_or_5() {
    let (config_dir, _state_tmp, _spec_tmp) = make_valid_config_dir();
    let output = Command::new(prism_bin())
        .args(["validate-config"])
        .env("PRISM_CONFIG_DIR", config_dir.path())
        .env("PRISM_TEST_INJECT_FAIL_STEP", "6_audit_failure")
        .output()
        .expect("failed to spawn prism binary");

    let code = output.status.code().unwrap_or(-1);
    assert_ne!(
        code, 2,
        "Audit failure must not produce exit 2; BC-2.05.012 invariant"
    );
    assert_ne!(
        code, 5,
        "Audit failure must not produce exit 5; BC-2.05.012 invariant"
    );
    assert_ne!(
        code, 1,
        "Audit failure must not produce exit 1 (panic); BC-2.05.012 invariant"
    );
    assert_eq!(
        code, 4,
        "Audit failure must produce exit 4 exactly; BC-2.05.012 invariant"
    );
}

// ---------------------------------------------------------------------------
// BC-2.05.012 — RocksDB LOCK held → exit 4 + LOCK in message
// ---------------------------------------------------------------------------

/// Story: S-WAVE5-PREP-01
/// BC: BC-2.05.012 EC-05-012-006: RocksDB LOCK file exists → exit 4 + actionable message
/// TV-05-012-003: Pre-existing LOCK → exit 4 + "Another Prism process may be running"
/// MED-5: uses isolated TempDir.
#[test]
fn test_BC_2_05_012_rocksdb_lock_held_exits_four_with_lock_message() {
    let (config_dir, _state_tmp, _spec_tmp) = make_valid_config_dir();
    let output = Command::new(prism_bin())
        .args(["validate-config"])
        .env("PRISM_CONFIG_DIR", config_dir.path())
        // Inject a LOCK-held failure — the boot.rs cfg(test) path simulates this.
        .env("PRISM_TEST_INJECT_FAIL_STEP", "6_rocksdb_lock")
        .output()
        .expect("failed to spawn prism binary");

    assert_eq!(
        output.status.code(),
        Some(4),
        "RocksDB LOCK held must exit 4 (BC-2.05.012 EC-05-012-006); \
         got exit {:?}",
        output.status.code()
    );

    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    // BC-2.05.012 EC-05-012-006: "Another Prism process may be running. Check {state_dir}/LOCK"
    assert!(
        combined.to_lowercase().contains("lock")
            || combined.to_lowercase().contains("another")
            || combined.to_lowercase().contains("process"),
        "Error output must mention LOCK or another running process; \
         BC-2.05.012 EC-05-012-006; got: {combined}"
    );
}

// ---------------------------------------------------------------------------
// BC-2.05.012 — AuditInitFailed BootError variant maps to exit 4
// ---------------------------------------------------------------------------

/// Story: S-WAVE5-PREP-01 AC-8
/// BC: BC-2.05.012 — AuditInitFailed variant exit code
///
/// Unit test verifying that BootError::AuditInitFailed maps to exit code 4
/// via the canonical exit_code() method.
///
/// RED GATE: This test verifies the already-implemented BootError::exit_code()
/// method. It passes today (not a Red Gate test) because exit_codes.rs and
/// BootError are implemented. Kept to document the traceability.
#[test]
fn test_BC_2_05_012_boot_error_audit_init_failed_exit_code_is_4() {
    use prism_bin::BootError;

    let err =
        BootError::AuditInitFailed("RocksDB CF open error: No space left on device".to_string());
    assert_eq!(
        err.exit_code(),
        4,
        "AuditInitFailed exit_code() must return 4 (internal-error); \
         BC-2.05.012 + AC-8"
    );

    // Also verify the display format includes the variant name for log traceability.
    let msg = err.to_string();
    assert!(
        msg.contains("audit-init-failed"),
        "AuditInitFailed display must include 'audit-init-failed' for log traceability; \
         got: {msg}"
    );
}

// ---------------------------------------------------------------------------
// BC-2.05.012 — sentinel event schema (TV-05-012-006)
// ---------------------------------------------------------------------------

/// Story: S-WAVE5-PREP-01
/// BC: BC-2.05.012 §boot.audit.initialized Sentinel Event Schema
/// TV-05-012-005 + TV-05-012-006: Sentinel persisted to RocksDB with all required fields.
///
/// F-PASS2-OBS-1 (S-WAVE5-PREP-01 fix-pass-2): This test now performs a full
/// sentinel readback from the RocksDB `audit_buffer` CF after `validate-config`
/// exits 0. It:
/// 1. Spawns `validate-config` (boot steps 1-6).
/// 2. Opens the RocksDB database at `state_dir`.
/// 3. Scans the `audit_buffer` CF for entries with prefix "audit:".
/// 4. Deserializes the entry via bincode to `prism_storage::audit_buffer::AuditEntry`.
/// 5. Parses the payload as the BC-specified schema (event_type, timestamp RFC 3339,
///    prism_version, config_dir, org_count, boot_step=6).
/// 6. Asserts all required fields are present and have correct values.
///
/// Required sentinel schema fields per BC-2.05.012 §Postconditions:
/// - event_type: "boot.audit.initialized"
/// - timestamp: RFC 3339 (F-PASS2-HIGH-2)
/// - prism_version: semver from CARGO_PKG_VERSION
/// - config_dir: redacted hash (never the raw path)
/// - org_count: "1" (one org in the test fixture)
/// - boot_step: "6"
#[test]
fn test_BC_2_05_012_sentinel_schema_has_required_fields() {
    use prism_core::StorageDomain;
    use prism_storage::audit_buffer::AuditEntry as StorageAuditEntry;
    use prism_storage::backend::RocksStorageBackend;
    use prism_storage::rocksdb_backend::RocksDbBackend;

    // MED-5: isolated per-test dirs to avoid parallel RocksDB LOCK collisions.
    let (config_dir, state_tmp, _spec_tmp) = make_valid_config_dir();

    let output = Command::new(prism_bin())
        .args(["validate-config"])
        .env("PRISM_CONFIG_DIR", config_dir.path())
        .output()
        .expect("failed to spawn prism binary");

    // Real-polarity assertion (POL-17): boot step 6 is implemented → exit 0.
    assert_eq!(
        output.status.code(),
        Some(0),
        "validate-config must exit 0 after boot step 6 implementation; \
         BC-2.05.012 TV-05-012-006: sentinel emitted, audit subsystem initialized; \
         stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // F-PASS2-OBS-1: open RocksDB from the test, scan audit_buffer CF, verify sentinel.
    // The subprocess wrote to state_dir/prism.db — open the same DB for readback.
    let backend = RocksDbBackend::open(state_tmp.path().to_path_buf())
        .expect("must be able to open RocksDB at state_dir after validate-config exits 0");

    // Scan the audit_buffer CF for sentinel entries.
    let entries = backend
        .scan(StorageDomain::AuditBuffer, b"audit:")
        .expect("scan of audit_buffer CF must succeed");

    assert!(
        !entries.is_empty(),
        "audit_buffer CF must contain at least one entry after boot step 6; \
         BC-2.05.012 TV-05-012-005 (sentinel persisted)"
    );

    // Find the boot.audit.initialized sentinel entry.
    let mut found_sentinel: Option<StorageAuditEntry> = None;
    for (_key, value) in &entries {
        let decoded: Result<(StorageAuditEntry, _), _> =
            bincode::serde::decode_from_slice(value, bincode::config::standard());
        if let Ok((entry, _)) = decoded {
            if entry
                .payload
                .get("event_type")
                .map(|v| v == "boot.audit.initialized")
                .unwrap_or(false)
            {
                found_sentinel = Some(entry);
                break;
            }
        }
    }

    let sentinel = found_sentinel.expect(
        "boot.audit.initialized sentinel must be present in audit_buffer CF; \
         BC-2.05.012 TV-05-012-006 (sentinel schema)",
    );

    // BC-2.05.012 §Postconditions: verify all required sentinel schema fields.
    let payload = &sentinel.payload;

    // event_type: must be exactly "boot.audit.initialized"
    assert_eq!(
        payload.get("event_type").map(String::as_str),
        Some("boot.audit.initialized"),
        "Sentinel must have event_type='boot.audit.initialized'; BC-2.05.012"
    );

    // timestamp: must be present and RFC 3339 (parseable by chrono)
    // F-PASS2-HIGH-2: timestamp was missing in fix-pass-1; now enforced.
    let timestamp_str = payload
        .get("timestamp")
        .expect("Sentinel must have 'timestamp' field; F-PASS2-HIGH-2 / BC-2.05.012");
    chrono::DateTime::parse_from_rfc3339(timestamp_str)
        .expect("Sentinel timestamp must be a valid RFC 3339 string; F-PASS2-HIGH-2 / BC-2.05.012");

    // prism_version: must be non-empty semver string
    let prism_version = payload
        .get("prism_version")
        .expect("Sentinel must have 'prism_version' field; BC-2.05.012");
    assert!(
        !prism_version.is_empty(),
        "Sentinel prism_version must be non-empty; BC-2.05.012"
    );

    // config_dir: must be present (as a hash — BC-2.05.012 invariant: path MUST be redacted)
    let config_dir_field = payload
        .get("config_dir")
        .expect("Sentinel must have 'config_dir' field; BC-2.05.012");
    assert!(
        !config_dir_field.is_empty(),
        "Sentinel config_dir must be non-empty; BC-2.05.012"
    );
    // Verify config_dir is redacted (not the raw path): must not contain path separators
    // that would indicate the raw filesystem path was stored.
    assert!(
        !config_dir_field.contains('/') && !config_dir_field.contains('\\'),
        "Sentinel config_dir must be a hash (not the raw path); \
         BC-2.05.012 §Invariants: 'config_dir MUST be redacted (only a hash or basename)'; \
         got: {config_dir_field}"
    );

    // org_count: must be "1" (one org in the test fixture)
    assert_eq!(
        payload.get("org_count").map(String::as_str),
        Some("1"),
        "Sentinel org_count must be '1' (one org in test fixture); BC-2.05.012"
    );

    // boot_step: must be "6" per ADR-022 §B step numbering
    assert_eq!(
        payload.get("boot_step").map(String::as_str),
        Some("6"),
        "Sentinel boot_step must be '6' (ADR-022 §B step 6); BC-2.05.012"
    );
}
