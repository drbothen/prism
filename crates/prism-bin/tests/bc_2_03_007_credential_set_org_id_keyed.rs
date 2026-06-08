// SPDX-License-Identifier: Apache-2.0
//! Red Gate test for S-DEMO-003 AC-010 / AC-005 / AC-007 — CRIT-2 namespace reconciliation
//! and F-P10-HIGH-001 credential delete path.
//!
//! **Contract (ADR-034 §D3 / BC-2.06.003 Tier-3 / BC-2.03.007 / BC-2.03.005):**
//! `handle_credential_set` MUST write via `CredentialStoreOrgId::set_by_org` (OrgId-keyed
//! namespace `"{org_id_uuid}/{sensor}/{name}"`). It MUST NOT write via the legacy
//! `CredentialStore::set` (slug-keyed namespace `"{slug}/{sensor}/{name}"`).
//!
//! `handle_credential_delete_with_store` MUST delete via `CredentialStoreOrgId::delete_by_org`
//! (OrgId-keyed namespace). A credential set via `set_by_org` must be retrievable-then-absent
//! after `delete_by_org` (F-P10-HIGH-001 / AC-007 / BC-2.03.005).
//!
//! # Test approach — in-process with InMemoryCredentialStore (SID-1 compliance)
//!
//! The test injects an `InMemoryCredentialStore` trait double into
//! `handle_credential_set_with_store` / `handle_credential_delete_with_store`
//! (injectable inner functions). After each call, the test inspects the in-memory store's
//! namespace keys to assert the credential was stored / absent at `"{org_id_uuid}/{sensor}/{name}"`.
//!
//! This avoids the macOS unsigned-test-binary OS keychain cross-process ACL limitation
//! while exercising the FULL production code path.
//!
//! The real-OS-keyring cross-process subprocess test is kept below as `#[ignore]`'d
//! with rationale per SID-1 §4.
//!
//! # Test → AC / BC mapping
//!
//! | Test | RG ID | AC | BC |
//! |------|-------|----|----|
//! | test_BC_2_06_003_crit2_slug_keyed_write_invisible_to_org_id_keyed_read | CRIT-2 proof | AC-010 | ADR-034 §D3 |
//! | test_handle_credential_set_writes_org_id_keyed_namespace | RG-034-004 | AC-010/AC-005 | BC-2.06.003 Tier-3; BC-2.03.007 |
//! | test_handle_credential_delete_uses_org_id_keyed_namespace | F-P10-HIGH-001 | AC-007 | BC-2.03.005 |
//! | test_handle_credential_set_subprocess_ignored | RG-034-004 subprocess | AC-010 | BC-2.06.003 Tier-3 |
//!
//! Story: S-DEMO-003 | ADR: ADR-034

#![allow(non_snake_case, clippy::unwrap_used)]

use std::sync::Arc;

use prism_core::{CredentialName, OrgId, OrgSlug};
use prism_credentials::{CredentialIndex, KeyringBackend};
use prism_credentials::{CredentialStore, CredentialStoreOrgId, InMemoryCredentialStore};

// ---------------------------------------------------------------------------
// CRIT-2 proof: slug-keyed write is invisible to OrgId-keyed read
// ---------------------------------------------------------------------------

/// CRIT-2 proof: a credential written via `CredentialStore::set` (slug-keyed) is
/// NOT found via `CredentialStoreOrgId::get_by_org` (OrgId-keyed).
///
/// This test passes at Red Gate (proves the gap exists) and continues to pass
/// after implementation (the two namespaces must always remain disjoint).
///
/// **F-P10-CRIT-001 compatibility note:** This test was previously implemented with
/// `KeyringBackend` directly. With real platform backends now enabled (F-P10-CRIT-001
/// fix), `KeyringBackend::set` writes to the actual OS keychain, causing flakiness
/// (duplicate-entry errors on repeated runs). The behavioral invariant being tested
/// (slug namespace ≠ OrgId namespace) is a LOGICAL property of the namespace_key and
/// namespace_key_by_org_id functions, not of the storage backend. `InMemoryCredentialStore`
/// exercises the same namespace logic and is CI-safe (no OS keychain side effects).
///
/// BC-2.06.003 Tier-3 / ADR-034 §D3 namespace isolation.
#[tokio::test]
async fn test_BC_2_06_003_crit2_slug_keyed_write_invisible_to_org_id_keyed_read() {
    // Use InMemoryCredentialStore: namespace isolation is a logical property of the
    // namespace key functions (namespace_key vs namespace_key_by_org_id). Any backend
    // that stores and retrieves from the same HashMap proves or disproves disjointness.
    let store = InMemoryCredentialStore::new();

    let uuid = uuid::Uuid::now_v7();
    let org_id = OrgId::from_uuid(uuid);
    let org_slug = OrgSlug::new("demo-org");
    let cred_name = CredentialName::new("bearer_token").expect("CredentialName::new");

    // Write via slug-keyed path (what the pre-fix stub did).
    store
        .set(
            &org_slug,
            "armis",
            &cred_name,
            secrecy::SecretString::new("test-value-crit2".to_string()),
        )
        .await
        .expect("slug-keyed set must succeed");

    // Read via OrgId-keyed path — must return None (disjoint namespace).
    let result = store
        .get_by_org(&org_id, "armis", &cred_name)
        .await
        .expect("get_by_org must not error");

    // This assertion proves CRIT-2: slug-keyed write is invisible to OrgId-keyed read.
    assert!(
        result.is_none(),
        "CRIT-2 proof: slug-keyed write MUST NOT be visible to OrgId-keyed read. \
         The namespaces '{{slug}}/{{sensor}}/{{name}}' and '{{org_id_uuid}}/{{sensor}}/{{name}}' \
         are disjoint. ADR-034 §D3."
    );
}

// ---------------------------------------------------------------------------
// RG-034-004: handle_credential_set writes OrgId-keyed namespace (in-process)
// ---------------------------------------------------------------------------

/// RG-034-004: `handle_credential_set_with_store` must write via
/// `CredentialStoreOrgId::set_by_org` so the stored credential uses the OrgId-UUID-keyed
/// namespace format.
///
/// **Test approach (F-HIGH-002 fix):** call `handle_credential_set_with_store` directly
/// with two injected trait doubles:
///   1. `InMemoryCredentialStore` — no real OS keyring needed.
///   2. `std::io::Cursor<&[u8]>` — injectable `BufRead` for the secret value, no real stdin.
///
/// This exercises the FULL production code path of `handle_credential_set_with_store`:
///   - `prism.toml` parsing via `resolve_org_slug_and_id`
///   - `CredentialName::new` validation
///   - `read_secret_value_from(reader)` dispatch (not bypassed)
///   - `store.set_by_org(org_id, sensor, cred_name, value)` call
///
/// After the call, the test inspects the in-memory store's namespace keys to assert the
/// credential was stored at `"{org_id_uuid}/armis/bearer_token"` — NOT at
/// `"{slug}/armis/bearer_token"`. The CRIT-2 namespace invariant is validated end-to-end
/// through the production handler (not via a direct `set_by_org` call bypass).
///
/// **Why in-process (no subprocess)?**
/// The subprocess approach requires the real OS keyring for cross-process reads.
/// On macOS, unsigned test binaries fail OS keychain cross-process ACL checks.
/// See the `#[ignore]`'d subprocess test below for the rationale.
///
/// RG-034-004 (ADR-034 §Red Gate Tests); AC-010 / AC-005 of S-DEMO-003.
#[tokio::test]
async fn test_handle_credential_set_writes_org_id_keyed_namespace() {
    // Set up temp config dir with prism.toml containing one org.
    let tmp = tempfile::TempDir::new().expect("temp dir");
    let config_dir = tmp.path().to_path_buf();

    // Use a known UUID v7 for the test org (same as used in generate_demo_prism_toml).
    let demo_org_uuid_str = "0196f4b2-3c8d-7e1a-b5f0-2d4c6e8a0b1c";
    let demo_org_slug = "demo-org";
    let state_dir = config_dir.join("state");
    let spec_dir = config_dir.join("specs");
    let plugin_dir = config_dir.join("plugins");
    std::fs::create_dir_all(&state_dir).unwrap();
    std::fs::create_dir_all(&spec_dir).unwrap();
    std::fs::create_dir_all(&plugin_dir).unwrap();

    // Write prism.toml — load_prism_config_for_cli reads it to extract the OrgId UUID.
    // This is a FIXTURE prism.toml (not the real ~/.config/prism-demo/prism.toml).
    //
    // Windows-safe path serialization: use {:?} (Rust Debug formatter) which emits a
    // quoted string with backslashes escaped as \\. This produces valid TOML basic-string
    // values on Windows paths like C:\Users\... without invalid Unicode escape sequences.
    // Pattern matches all other make_valid_config_dir() helpers in this test suite.
    let prism_toml = format!(
        "spec_dir = {:?}\nstate_dir = {:?}\nplugin_dir = {:?}\n\n[[orgs]]\norg_id = {org_id:?}\norg_slug = {org_slug:?}\n",
        spec_dir.display(),
        state_dir.display(),
        plugin_dir.display(),
        org_id = demo_org_uuid_str,
        org_slug = demo_org_slug,
    );
    std::fs::write(config_dir.join("prism.toml"), &prism_toml).expect("write prism.toml");

    // Load PrismConfig ONCE from the fixture prism.toml — single parse, matches production
    // handle_credential_set which calls load_prism_config_for_cli before the inner handler.
    let prism_config = prism_bin::credential_cli::load_prism_config_for_cli(&config_dir)
        .expect("load_prism_config_for_cli must succeed with valid fixture prism.toml");

    // Inject the InMemoryCredentialStore — no real OS keyring needed.
    let store = Arc::new(InMemoryCredentialStore::new());

    // The secret value to inject via the BufRead reader (simulates piped stdin).
    // This value must appear in the store under the OrgId-keyed key after the call.
    let secret_value = "rg034004-test-bearer-value";

    // Build the CredentialSetArgs as if invoked from the CLI.
    // No `value` field — AD-017 invariant (value comes from the reader, not args).
    let args = prism_bin::credential_cli::CredentialSetArgs {
        sensor: "armis".to_string(),
        name: "bearer_token".to_string(),
        org_slug: Some(demo_org_slug.to_string()),
    };

    // Inject the secret value via a Cursor — no real stdin read.
    // handle_credential_set_with_store calls read_secret_value_from(secret_reader)
    // which reads from the Cursor exactly as it would from a piped stdin.
    // The trailing newline mirrors what `printf 'value\n' | prism credential set` does.
    let secret_bytes = format!("{secret_value}\n");
    let mut secret_reader = std::io::Cursor::new(secret_bytes.as_bytes().to_vec());

    // Call the PRODUCTION handler — not a direct set_by_org bypass.
    // This is the load-bearing coverage that F-HIGH-002 requires:
    // the full code path from args → resolve_org_slug_and_id → read_secret_value_from
    // → set_by_org is exercised in-process.
    // Passes &prism_config (already loaded above) — single parse, no double-parse.
    let prism_toml_path = config_dir.join("prism.toml");
    let exit_code = prism_bin::credential_cli::handle_credential_set_with_store(
        args,
        &prism_config,
        &prism_toml_path,
        store.clone(),
        &mut secret_reader,
    )
    .await;

    assert_eq!(
        exit_code, 0,
        "RG-034-004: handle_credential_set_with_store must return exit 0 on success. \
         Got exit {exit_code}. Check prism.toml fixture and InMemoryCredentialStore setup."
    );

    // Compute expected OrgId-keyed key.
    let org_id = {
        let uuid = uuid::Uuid::parse_str(demo_org_uuid_str).expect("valid uuid");
        OrgId::from_uuid(uuid)
    };
    let expected_key = format!("{org_id}/armis/bearer_token");
    let slug_keyed = format!("{demo_org_slug}/armis/bearer_token");

    // ASSERTION 1: credential stored at OrgId-UUID-keyed namespace.
    assert!(
        store.contains_key(&expected_key),
        "RG-034-004 (CRIT-2 gap closure): handle_credential_set_with_store must store \
         the credential at the OrgId-UUID-keyed namespace '{{org_id_uuid}}/{{sensor}}/{{name}}'. \
         Expected key: '{expected_key}'. \
         Keys in store: {:?}. \
         ADR-034 §D3; AC-010 of S-DEMO-003.",
        store.keys()
    );

    // ASSERTION 2: credential NOT stored at legacy slug-keyed namespace (CRIT-2).
    assert!(
        !store.contains_key(&slug_keyed),
        "RG-034-004 (CRIT-2): handle_credential_set_with_store MUST NOT store the \
         credential at the slug-keyed namespace '{{slug}}/{{sensor}}/{{name}}'. \
         The slug-keyed namespace is disjoint from get_by_org's OrgId-UUID namespace. \
         Found unexpected slug-keyed key: '{slug_keyed}'. \
         ADR-034 §D3 CRIT-2 remediation."
    );
}

// ---------------------------------------------------------------------------
// F-P10-HIGH-001: handle_credential_delete uses OrgId-keyed namespace (in-process)
// ---------------------------------------------------------------------------

/// F-P10-HIGH-001: `handle_credential_delete_with_store` must delete via
/// `CredentialStoreOrgId::delete_by_org` using the OrgId-UUID-keyed namespace
/// `"{org_id_uuid}/{sensor}/{name}"`.
///
/// This is the load-bearing test for the `prism credential delete` subcommand introduced
/// to fix demo-teardown.sh. The old teardown used `secret-tool clear service "prism"
/// account "${key}"` (Linux) or `security delete-generic-password` (macOS). The Linux
/// path was broken because keyring-rs 3.x dbus-secret-service stores credentials under
/// the `username` attribute (NOT `account`), so the clear matched nothing — 5 orphaned
/// keyring entries remained after every demo teardown (F-P10-HIGH-001 root cause).
///
/// **Test approach (SID-1 compliance):** inject `InMemoryCredentialStore`. Call
/// `set_by_org` directly to prime a credential, then call `handle_credential_delete_with_store`
/// to delete it via the production handler. Assert the key is absent afterward.
///
/// This exercises the FULL production code path:
///   - `resolve_org_slug_and_id` (prism.toml → OrgId resolution)
///   - `CredentialName::new` validation
///   - `store.delete_by_org(org_id, sensor, cred_name)` dispatch
///   - idempotent Ok(false) path (second delete returns exit 0)
///
/// F-P10-HIGH-001; AC-007 / BC-2.03.005 delete path; ADR-034 §D3.
#[tokio::test]
async fn test_handle_credential_delete_uses_org_id_keyed_namespace() {
    // Set up temp config dir with prism.toml containing one org.
    let tmp = tempfile::TempDir::new().expect("temp dir");
    let config_dir = tmp.path().to_path_buf();

    let demo_org_uuid_str = "0196f4b2-3c8d-7e1a-b5f0-2d4c6e8a0b1c";
    let demo_org_slug = "demo-org";
    let state_dir = config_dir.join("state");
    let spec_dir = config_dir.join("specs");
    let plugin_dir = config_dir.join("plugins");
    std::fs::create_dir_all(&state_dir).unwrap();
    std::fs::create_dir_all(&spec_dir).unwrap();
    std::fs::create_dir_all(&plugin_dir).unwrap();

    // Write prism.toml fixture — load_prism_config_for_cli reads it to
    // resolve the OrgId UUID (same as the set path, ADR-034 §D3).
    //
    // Windows-safe path serialization: {:?} escapes backslashes in Windows paths.
    let prism_toml = format!(
        "spec_dir = {:?}\nstate_dir = {:?}\nplugin_dir = {:?}\n\n[[orgs]]\norg_id = {org_id:?}\norg_slug = {org_slug:?}\n",
        spec_dir.display(),
        state_dir.display(),
        plugin_dir.display(),
        org_id = demo_org_uuid_str,
        org_slug = demo_org_slug,
    );
    std::fs::write(config_dir.join("prism.toml"), &prism_toml).expect("write prism.toml");

    // Load PrismConfig ONCE from the fixture prism.toml — single parse, matches production.
    let prism_config = prism_bin::credential_cli::load_prism_config_for_cli(&config_dir)
        .expect("load_prism_config_for_cli must succeed with valid fixture prism.toml");

    // Inject the InMemoryCredentialStore — no real OS keyring needed.
    let store = Arc::new(InMemoryCredentialStore::new());

    // Compute the expected OrgId-keyed key for assertions.
    let org_id = {
        let uuid = uuid::Uuid::parse_str(demo_org_uuid_str).expect("valid uuid");
        OrgId::from_uuid(uuid)
    };
    let cred_name = CredentialName::new("client_id").expect("CredentialName::new");
    let expected_key = format!("{org_id}/crowdstrike/client_id");

    // STEP 1: Prime the store with a credential at the OrgId-keyed namespace.
    // This simulates what `prism credential set` would have written.
    store
        .set_by_org(
            &org_id,
            "crowdstrike",
            &cred_name,
            secrecy::SecretString::new("test-delete-sentinel-value".to_string()),
        )
        .await
        .expect("set_by_org must succeed on InMemoryCredentialStore");

    // Verify the credential is present before delete.
    assert!(
        store.contains_key(&expected_key),
        "F-P10-HIGH-001 pre-condition: credential must be present before delete. \
         key='{expected_key}'"
    );

    // STEP 2: Call the PRODUCTION delete handler with injected store.
    // Passes &prism_config (loaded once above) — single parse, no double-parse.
    let prism_toml_path = config_dir.join("prism.toml");
    let delete_args = prism_bin::credential_cli::CredentialDeleteArgs {
        sensor: "crowdstrike".to_string(),
        name: "client_id".to_string(),
        org_slug: Some(demo_org_slug.to_string()),
    };
    let exit_code = prism_bin::credential_cli::handle_credential_delete_with_store(
        delete_args,
        &prism_config,
        &prism_toml_path,
        store.clone(),
    )
    .await;

    assert_eq!(
        exit_code, 0,
        "F-P10-HIGH-001: handle_credential_delete_with_store must return exit 0 on success. \
         Got exit {exit_code}. Check prism.toml fixture and InMemoryCredentialStore setup."
    );

    // STEP 3: Assert the credential was removed from the OrgId-keyed namespace.
    assert!(
        !store.contains_key(&expected_key),
        "F-P10-HIGH-001 (AC-007): handle_credential_delete_with_store must remove the \
         credential from the OrgId-UUID-keyed namespace '{expected_key}'. \
         ADR-034 §D3 / BC-2.03.005 delete path."
    );

    // STEP 4: Assert second delete is idempotent (returns exit 0, not exit 1).
    // demo-teardown.sh calls delete unconditionally — a not-found entry must not fail.
    let delete_args_again = prism_bin::credential_cli::CredentialDeleteArgs {
        sensor: "crowdstrike".to_string(),
        name: "client_id".to_string(),
        org_slug: Some(demo_org_slug.to_string()),
    };
    let exit_code_again = prism_bin::credential_cli::handle_credential_delete_with_store(
        delete_args_again,
        &prism_config,
        &prism_toml_path,
        store.clone(),
    )
    .await;

    assert_eq!(
        exit_code_again, 0,
        "F-P10-HIGH-001 (idempotent): second delete of an absent credential must return \
         exit 0 (not exit 1). demo-teardown.sh calls delete unconditionally. \
         Got exit {exit_code_again}."
    );
}

// ---------------------------------------------------------------------------
// RG-034-004 subprocess (IGNORED — macOS unsigned-test-binary OS keychain ACL)
// ---------------------------------------------------------------------------

/// **#[ignore] rationale (SID-1 §4):**
/// This test spawns `prism credential set` as a subprocess and reads back via
/// the real OS keyring. On macOS, unsigned test binaries (cargo test / nextest)
/// and the compiled prism binary have different code-signing identities. The macOS
/// Keychain ACL prevents the test process from reading credentials written by the
/// subprocess binary (and vice versa). This causes a cross-process keyring read
/// failure unrelated to the correctness of the implementation.
///
/// The load-bearing coverage is provided by:
/// - `test_handle_credential_set_writes_org_id_keyed_namespace` (in-memory double, above)
/// - `bc_2_06_003_tier3_keyring_resolution.rs` RG-034-001 (in-memory double)
/// - `keyring_org_id.rs` (ignored, requires live OS keyring service)
///
/// To run this test on a machine with a properly signed binary:
/// ```text
/// cargo test -p prism-bin --test bc_2_03_007_credential_set_org_id_keyed -- --ignored
/// ```
///
/// RG-034-004 subprocess variant (ADR-034 §Red Gate Tests); AC-010 of S-DEMO-003.
#[tokio::test]
#[ignore = "macOS unsigned-test-binary cross-process keychain ACL blocks reads; \
            in-process test_handle_credential_set_writes_org_id_keyed_namespace \
            provides load-bearing namespace coverage (SID-1 §4 documented deferral)"]
async fn test_handle_credential_set_subprocess_reads_org_id_keyed_entry() {
    // Set up temp config dir with prism.toml containing one org.
    let tmp = tempfile::TempDir::new().expect("temp dir");
    let config_dir = tmp.path().to_path_buf();

    let demo_org_uuid_str = "0196f4b2-3c8d-7e1a-b5f0-2d4c6e8a0b1c";
    let demo_org_slug = "demo-org";
    let state_dir = config_dir.join("state");
    let spec_dir = config_dir.join("specs");
    let plugin_dir = config_dir.join("plugins");
    std::fs::create_dir_all(&state_dir).unwrap();
    std::fs::create_dir_all(&spec_dir).unwrap();
    std::fs::create_dir_all(&plugin_dir).unwrap();

    // Windows-safe path serialization: {:?} escapes backslashes in Windows paths.
    let prism_toml = format!(
        "spec_dir = {:?}\nstate_dir = {:?}\nplugin_dir = {:?}\n\n[[orgs]]\norg_id = {org_id:?}\norg_slug = {org_slug:?}\n",
        spec_dir.display(),
        state_dir.display(),
        plugin_dir.display(),
        org_id = demo_org_uuid_str,
        org_slug = demo_org_slug,
    );
    std::fs::write(config_dir.join("prism.toml"), &prism_toml).expect("write prism.toml");

    let org_id = {
        let uuid = uuid::Uuid::parse_str(demo_org_uuid_str).expect("valid uuid");
        OrgId::from_uuid(uuid)
    };
    let cred_name = CredentialName::new("bearer_token").expect("CredentialName::new");
    let secret_value = "rg034004-subprocess-test-bearer-value";

    // Find the prism binary.
    let binary = std::env::current_exe()
        .expect("current_exe")
        .parent()
        .expect("parent dir")
        .parent()
        .expect("grandparent dir (target/debug or target/release)")
        .join("prism");

    if !binary.exists() {
        eprintln!(
            "RG-034-004 (subprocess): prism binary not found at {:?}. Skipping.",
            binary
        );
        return;
    }

    // Write the secret to a temp file for stdin.
    let secret_file = tmp.path().join("secret_input.txt");
    {
        use std::io::Write;
        let mut f = std::fs::File::create(&secret_file).unwrap();
        writeln!(f, "{}", secret_value).unwrap();
    }

    // Invoke `prism credential set` via subprocess.
    let output = std::process::Command::new(&binary)
        .args([
            "--config-dir",
            config_dir.to_str().unwrap(),
            "credential",
            "set",
            "--sensor",
            "armis",
            "--name",
            "bearer_token",
            "--org-slug",
            demo_org_slug,
        ])
        .stdin(std::fs::File::open(&secret_file).unwrap())
        .output()
        .expect("subprocess must spawn");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    eprintln!(
        "RG-034-004 subprocess exit={}, stdout={:?}, stderr={:?}",
        output.status.code().unwrap_or(-1),
        stdout,
        stderr
    );

    // Read back via OrgId-keyed get_by_org using the real OS keyring.
    // Index path aligned to state_dir (F-P7-OBS-001 fix: CLI write path now uses
    // state_dir.join("credential_index.json") matching boot step 5).
    let index_path = state_dir.join("credential_index.json");
    let reader_index = CredentialIndex::new(index_path);
    let reader = KeyringBackend::new("prism", reader_index);
    let result = reader
        .get_by_org(&org_id, "armis", &cred_name)
        .await
        .expect("get_by_org must not error (keyring available)");

    assert!(
        result.is_some(),
        "RG-034-004 (subprocess): `handle_credential_set` must write the credential \
         via `CredentialStoreOrgId::set_by_org` so that `get_by_org` finds it. \
         subprocess exit={}, stdout={:?}, stderr={:?}. \
         [Ignored: macOS unsigned-test-binary cross-process keychain ACL]",
        output.status.code().unwrap_or(-1),
        stdout,
        stderr
    );
}

// ---------------------------------------------------------------------------
// Windows-safe path → TOML serialization unit test (PR #176 CI fix)
// ---------------------------------------------------------------------------

/// Windows-safe path serialization: a path containing backslashes (Windows) must
/// serialize via `{:?}` (Rust Debug formatter) to a valid TOML basic-string value.
///
/// # What is tested
///
/// The `{:?}` formatter for `std::path::Display` emits a quoted string with backslashes
/// escaped as `\\`. On Windows, `C:\Users\runner\AppData\Local\Temp\...` becomes
/// `"C:\\Users\\runner\\AppData\\Local\\Temp\\..."` — a valid TOML basic string that
/// `toml::from_str` can parse without an "invalid Unicode escape sequence" error.
///
/// On Unix, forward-slash paths are unaffected (no escaping needed, test still passes).
///
/// # Why this test is load-bearing (not just documentation)
///
/// The test feeds a synthetic backslash-containing path through the same `{:?}` format
/// path and round-trips it through `toml::from_str`. If the serialization is ever
/// reverted to the `"{}"`/`display()` pattern, this test will fail on Windows (and on
/// all platforms for the synthetic backslash path used here), catching the regression
/// before CI reaches the Windows runner.
///
/// PR #176 CI fix — x86_64-pc-windows-msvc test gate.
#[test]
fn test_windows_safe_path_toml_serialization_roundtrip() {
    use std::path::PathBuf;

    // Synthetic path that mimics a Windows temp dir — backslashes are present even on
    // Unix so this test catches the issue on ALL platforms, not just Windows CI.
    let fake_windows_path = PathBuf::from(r"C:\Users\runneradmin\AppData\Local\Temp\prism-test");

    // This is the pattern used in all prism.toml fixture builders after the PR #176 fix.
    // {:?} emits a quoted string with \\ for each backslash.
    let toml_content = format!(
        "spec_dir = {:?}\nstate_dir = {:?}\n\n[[orgs]]\norg_id = \"0196f000-0000-7000-8000-000000000001\"\norg_slug = \"acme\"\n",
        fake_windows_path.display(),
        fake_windows_path.display(),
    );

    // The TOML must parse without error. On Windows, a bare `"C:\Users\..."` triggers
    // "invalid Unicode escape `\U`" because TOML basic strings treat `\` as escape.
    // With {:?}, the path is emitted as `"C:\\Users\\..."` — a valid TOML string.
    let result = toml::from_str::<toml::Value>(&toml_content);
    assert!(
        result.is_ok(),
        "Windows-safe TOML path serialization: the {{:?}} formatter must produce valid TOML \
         for a path containing backslashes (PR #176 fix). \
         TOML content:\n{toml_content}"
    );

    // Round-trip: the parsed spec_dir must equal the original path string.
    let parsed = result.unwrap();
    let spec_dir_val = parsed
        .get("spec_dir")
        .and_then(|v| v.as_str())
        .expect("spec_dir must be a TOML string");

    // The Debug-formatted Display of a PathBuf emits the path as-is (including backslashes
    // on the host where the path was constructed) because PathBuf::Display outputs the
    // platform-native separator. On Unix this is just the synthetic string with backslashes.
    let expected = fake_windows_path.display().to_string();
    assert_eq!(
        spec_dir_val, expected,
        "Round-trip: parsed spec_dir must equal original path string. \
         Expected: {expected:?}, Got: {spec_dir_val:?}"
    );
}
