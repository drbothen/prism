// SPDX-License-Identifier: Apache-2.0
//! Red Gate test for S-DEMO-003 AC-010 / AC-005 — CRIT-2 namespace reconciliation.
//!
//! **Contract (ADR-034 §D3 / BC-2.06.003 Tier-3 / BC-2.03.007):**
//! `handle_credential_set` MUST write via `CredentialStoreOrgId::set_by_org` (OrgId-keyed
//! namespace `"{org_id_uuid}/{sensor}/{name}"`). It MUST NOT write via the legacy
//! `CredentialStore::set` (slug-keyed namespace `"{slug}/{sensor}/{name}"`).
//!
//! **Red Gate discipline (BC-5.38.001):**
//! This test FAILS before implementation because the current `handle_credential_set`
//! writes via `CredentialStore::set` (slug-keyed), which is invisible to `get_by_org`.
//! The final assertion checks that an OrgId-keyed read finds the entry written by the
//! subprocess — but the subprocess wrote slug-keyed, so `get_by_org` returns None.
//!
//! **After implementation (S-DEMO-003 green phase):**
//! `handle_credential_set` writes via `CredentialStoreOrgId::set_by_org` → the entry
//! is stored at `"{org_id_uuid}/armis/bearer_token"`. `get_by_org` finds it → Some.
//!
//! # Test → AC / BC mapping
//!
//! | Test | RG ID | AC | BC |
//! |------|-------|----|----|
//! | test_handle_credential_set_writes_org_id_keyed_keyring_entry | RG-034-004 | AC-010 / AC-005 | BC-2.06.003 Tier-3; BC-2.03.007 |
//!
//! Story: S-DEMO-003 | ADR: ADR-034

#![allow(non_snake_case, clippy::unwrap_used)]

use std::io::Write;

use prism_core::{CredentialName, OrgId, OrgSlug};
use prism_credentials::{CredentialIndex, CredentialStore, CredentialStoreOrgId, KeyringBackend};

// ---------------------------------------------------------------------------
// RG-034-004 Part 1: CRIT-2 — slug-keyed write is invisible to OrgId-keyed read
// ---------------------------------------------------------------------------

/// CRIT-2 proof: a credential written via `CredentialStore::set` (slug-keyed) is
/// NOT found via `CredentialStoreOrgId::get_by_org` (OrgId-keyed).
///
/// This test passes at Red Gate (it proves the gap exists) and continues to pass
/// after implementation (the two namespaces must always remain disjoint).
///
/// BC-2.06.003 Tier-3 / ADR-034 §D3 namespace isolation.
#[tokio::test]
async fn test_BC_2_06_003_crit2_slug_keyed_write_invisible_to_org_id_keyed_read() {
    let tmp = tempfile::TempDir::new().expect("temp dir");

    let broken_index = CredentialIndex::new(tmp.path().join("broken_index.json"));
    let broken_keyring = KeyringBackend::new("prism-test", broken_index);

    let uuid = uuid::Uuid::now_v7();
    let org_id = OrgId::from_uuid(uuid);
    let org_slug = OrgSlug::new("demo-org");
    let cred_name = CredentialName::new("bearer_token").expect("CredentialName::new");

    // Write via slug-keyed path (what the current stub does).
    broken_keyring
        .set(
            &org_slug,
            "armis",
            &cred_name,
            secrecy::SecretString::new("test-value-crit2".to_string()),
        )
        .await
        .expect("slug-keyed set must succeed");

    // Read via OrgId-keyed path — must return None (disjoint namespace).
    let result = broken_keyring
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
// Note: The "OrgId-keyed write IS visible to OrgId-keyed read" proof is covered
// by prism-credentials tests (bc_3_2_002_trait_impl, bc_3_2_002_org_id_namespace).
// That test suite has proper keychain access for the prism-credentials test binary.
// The prism-bin test binary may not have macOS Keychain access to write directly.
// prism-bin tests focus on the subprocess (the production binary), which does.
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// RG-034-004 Part 3: handle_credential_set subprocess writes OrgId-keyed entry
//
// RED GATE: This test FAILS because handle_credential_set uses slug-keyed write.
// The subprocess writes to "{slug}/armis/bearer_token" (legacy namespace).
// The keyring reader looks for the entry at "{org_id_uuid}/armis/bearer_token".
// get_by_org returns None → assertion fails.
//
// After implementation: handle_credential_set writes via set_by_org →
// "{org_id_uuid}/armis/bearer_token" → get_by_org finds it → assertion passes.
// ---------------------------------------------------------------------------

/// BC-2.06.003 Tier-3 + BC-2.03.007 + ADR-034 §D3:
/// `handle_credential_set` must write the credential via `CredentialStoreOrgId::set_by_org`
/// (OrgId-keyed). After writing, `KeyringBackend::get_by_org` must find the entry.
///
/// **Red Gate failure:** the subprocess writes via `CredentialStore::set` (slug-keyed).
/// `get_by_org` returns None → assertion fails.
///
/// **After implementation:** subprocess writes via `set_by_org` (OrgId-keyed).
/// `get_by_org` finds the entry → assertion passes.
///
/// RG-034-004 (ADR-034 §Red Gate Tests); AC-010 / AC-005 of S-DEMO-003.
#[tokio::test]
async fn test_handle_credential_set_writes_org_id_keyed_keyring_entry() {
    // Set up temp config dir with prism.toml containing one org.
    let tmp = tempfile::TempDir::new().expect("temp dir");
    let config_dir = tmp.path().to_path_buf();

    // Use a known UUID v7 for the test org.
    let demo_org_uuid_str = "0196f4b2-3c8d-7e1a-b5f0-2d4c6e8a0b1c";
    let demo_org_slug = "demo-org";
    let state_dir = config_dir.join("state");
    let spec_dir = config_dir.join("specs");
    let plugin_dir = config_dir.join("plugins");
    std::fs::create_dir_all(&state_dir).unwrap();
    std::fs::create_dir_all(&spec_dir).unwrap();
    std::fs::create_dir_all(&plugin_dir).unwrap();

    let prism_toml = format!(
        r#"spec_dir = "{spec}"
state_dir = "{state}"
plugin_dir = "{plugin}"

[[orgs]]
org_id = "{org_id}"
org_slug = "{org_slug}"
"#,
        spec = spec_dir.display(),
        state = state_dir.display(),
        plugin = plugin_dir.display(),
        org_id = demo_org_uuid_str,
        org_slug = demo_org_slug,
    );
    std::fs::write(config_dir.join("prism.toml"), &prism_toml).expect("write prism.toml");

    let org_id = {
        let uuid = uuid::Uuid::parse_str(demo_org_uuid_str).expect("valid uuid");
        OrgId::from_uuid(uuid)
    };
    let cred_name = CredentialName::new("bearer_token").expect("CredentialName::new");
    let secret_value = "rg034004-test-bearer-value";

    // Find the prism binary.
    let binary = std::env::current_exe()
        .expect("current_exe")
        .parent()
        .expect("parent dir")
        .parent()
        .expect("grandparent dir (target/debug or target/release)")
        .join("prism");

    if !binary.exists() {
        // prism binary not built — skip the subprocess path.
        // The Red Gate is still asserted via the namespace check below.
        eprintln!(
            "RG-034-004: prism binary not found at {:?}. \
             The Red Gate assertion is still exercised via direct namespace check.",
            binary
        );
        // Simulate the BROKEN path (what the stub does): slug-keyed write.
        let index_path = config_dir.join("credential_index.json");
        let broken_index = CredentialIndex::new(index_path.clone());
        let broken_keyring = KeyringBackend::new("prism-test", broken_index);
        let org_slug = OrgSlug::new(demo_org_slug);
        broken_keyring
            .set(
                &org_slug,
                "armis",
                &cred_name,
                secrecy::SecretString::new(secret_value.to_string()),
            )
            .await
            .expect("slug-keyed set in test fixture");

        // Check OrgId-keyed read: must return None (CRIT-2 — slug write invisible to OrgId read).
        let reader_index = CredentialIndex::new(index_path);
        let reader = KeyringBackend::new("prism-test", reader_index);
        let result = reader
            .get_by_org(&org_id, "armis", &cred_name)
            .await
            .expect("get_by_org must not error");

        // RED GATE ASSERTION: slug-keyed write → OrgId-keyed read returns None.
        // This fails until handle_credential_set uses set_by_org.
        assert!(
            result.is_some(),
            "RG-034-004 (CRIT-2 gap closure): `handle_credential_set` must write the credential \
             via `CredentialStoreOrgId::set_by_org` so that `get_by_org` finds it. \
             Current stub uses `CredentialStore::set` (slug-keyed) — invisible to OrgId-keyed read. \
             Got None (entry not found at OrgId-keyed namespace). \
             ADR-034 §D3; AC-010 of S-DEMO-003. [no-binary fallback path]"
        );
        return;
    }

    // Write the secret to a temp file for stdin.
    let secret_file = tmp.path().join("secret_input.txt");
    {
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

    // Log subprocess output for diagnostics.
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    eprintln!(
        "RG-034-004 subprocess exit={}, stdout={:?}, stderr={:?}",
        output.status.code().unwrap_or(-1),
        stdout,
        stderr
    );

    // Read back via OrgId-keyed get_by_org.
    // The subprocess writes via handle_credential_set. In the stub, this uses
    // slug-keyed CredentialStore::set → invisible to OrgId-keyed get_by_org.
    // After the implementer's fix: set_by_org → OrgId-keyed → found.
    let index_path = config_dir.join("credential_index.json");
    let reader_index = CredentialIndex::new(index_path);
    let reader = KeyringBackend::new("prism", reader_index);
    let result = reader
        .get_by_org(&org_id, "armis", &cred_name)
        .await
        .expect("get_by_org must not error (keyring available)");

    // RED GATE ASSERTION — fails until handle_credential_set uses set_by_org.
    assert!(
        result.is_some(),
        "RG-034-004 (CRIT-2 gap closure): `handle_credential_set` must write the credential \
         via `CredentialStoreOrgId::set_by_org` so that `get_by_org` finds it. \
         Current implementation uses `CredentialStore::set` (slug-keyed) — invisible to \
         OrgId-keyed read. Got None. \
         subprocess exit={}, stdout={:?}, stderr={:?}. \
         Implementer must replace `CredentialStore::set` with `CredentialStoreOrgId::set_by_org` \
         in `handle_credential_set` (ADR-034 §D3; BC-2.06.003 Tier-3; AC-010 of S-DEMO-003).",
        output.status.code().unwrap_or(-1),
        stdout,
        stderr
    );
}
