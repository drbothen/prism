---
story_id: "S-MAINT-ECRED-TAXONOMY-SYNC-001"
story_version: "v1.1"
story_title: "E-CRED Namespace Reconciliation — Canonical E-CRED-001..010 per ADR-035"
evidence_date: "2026-06-07"
branch: "feature/S-MAINT-ECRED-TAXONOMY-SYNC-001"
head_commit: "c63b126e"
demo_recorder: "demo-recorder"
---

# Demo Evidence Report — S-MAINT-ECRED-TAXONOMY-SYNC-001 v1.1

## Recordings Produced

| File | Type | Contents |
|------|------|----------|
| `AC-RG-001-red-gate-tests-green.gif` | GIF (PR embed) | 5 Red Gate tests — prism-core + prism-credentials — all PASS |
| `AC-RG-001-red-gate-tests-green.webm` | WebM (archival) | same |
| `AC-RG-001-red-gate-tests-green.tape` | VHS script source | reproducible |
| `AC-DS-002-runtime-display-strings.gif` | GIF (PR embed) | Canonical Display strings + no-collision grep evidence |
| `AC-DS-002-runtime-display-strings.webm` | WebM (archival) | same |
| `AC-DS-002-runtime-display-strings.tape` | VHS script source | reproducible |

---

## AC Coverage Table

| AC ID | Description | Evidence | Type |
|-------|-------------|----------|------|
| **AC-001** | `CredentialEncryptionError` renumbered to E-CRED-006 | `AC-DS-002-*` — `grep -n 'E-CRED-00[567]' crates/prism-core/src/error.rs` shows `#[error("E-CRED-006: credential encryption error: {reason}")]`; `AC-RG-001-*` — `test_ac5_prism_error_display_e_cred_006_encryption` PASS | Recording + Test |
| **AC-002** | `EncryptionKeyMissing` renumbered to E-CRED-007 | `AC-DS-002-*` — same grep shows `#[error("E-CRED-007: encryption key not configured: {reason}")]`; `AC-RG-001-*` — `test_ac5_prism_error_display_e_cred_007_key_missing` PASS | Recording + Test |
| **AC-003** | `resolve_secret.rs` file-I/O errors emit canonical E-CRED-005 | `AC-DS-002-*` — `grep -n 'E-CRED-005' crates/prism-credentials/src/resolve_secret.rs` shows all 3 canonical strings (file-missing, is-directory, read-failed); `AC-RG-001-*` — `test_BC_2_03_009_resolve_secret_file_io_emits_e_cred_005` PASS | Recording + Test |
| **AC-004** | `KeyringError` variant retired and removed | `AC-RG-001-*` — `test_e_cred_keyring_error_variant_retired` PASS (RG-ECRED-004); spec authority: `crates/prism-core/src/error.rs` — `rg 'KeyringError' crates/prism-core/src/error.rs` returns zero hits post-deletion | Test (compile-time guard) |
| **AC-005** | `error_mapping.rs` `KeyringError` arm removed | Spec authority: `crates/prism-mcp/src/error_mapping.rs` — `rg 'KeyringError' crates/prism-mcp/src/error_mapping.rs` returns zero hits; `just check` exits 0 confirming exhaustive match (pre-PR gate, not recorded separately — covered by `just check` in story delivery) | Code verification |
| **AC-006** | `error-taxonomy.md` E-CRED section rewritten to canonical E-CRED-001..010 | Spec authority: `.factory/specs/prd-supplements/error-taxonomy.md` — wholesale rewrite to canonical 10-row table per ADR-035 §D1; adversarial passes 1-12 (LOCAL cascade) confirm correctness | Spec artifact |
| **AC-007** | Existing E-CRED-001 test unaffected | `AC-RG-001-*` — `test_ac5_prism_error_display_e_cred_001` PASS (unmodified; RG-ECRED-001 baseline preserved) | Test |
| **AC-008** | `BC-2.06.003` and `ADR-034 §D4` updated to E-CRED-008 | Spec authority: `.factory/specs/behavioral-contracts/BC-2.06.003-credential-reference-resolution.md` Tier-3 postconditions cite E-CRED-008; `.factory/specs/architecture/decisions/ADR-034-tier3-keyring-resolution-org-id-threading.md` §D4 all three E-CRED-005 occurrences updated to E-CRED-008 | Spec artifact |
| **AC-009** | No-collision invariant — E-CRED-005 limited to `resolve_secret.rs` on develop | `AC-DS-002-*` — `git grep 'E-CRED-005' -- crates/` shows hits only in `crates/prism-credentials/src/resolve_secret.rs` (3 hits); `AC-RG-001-*` — `test_e_cred_005_emitted_only_by_file_io_path` PASS (RG-ECRED-005 workspace scan) | Recording + Test |
| **AC-010** | `bc_2_03_009_resolve_secret.rs` loose assertions tightened to `E-CRED-005` | `AC-RG-001-*` — `test_BC_2_03_009_rejects_nonexistent_file_with_credential_error` and `test_BC_2_03_009_rejects_directory_path_with_credential_error` PASS with strict `E-CRED-005` assertions (10/10 BC_2_03_009 tests PASS) | Test |
| **AC-011** | Downstream `.factory/specs` E-CRED-002 citations correct (closes DF-PASS3-001) | Spec authority: `.factory/specs/architecture/security-architecture.md` v1.2 — resolution-chain "Not found" node cites E-CRED-002; `.factory/specs/prd-supplements/interface-definitions.md` v2.7 — `credential_status` errors array cites E-CRED-002. Verified by adversarial sweep (LOCAL passes, story v1.1 changelog entry). | Spec artifact |

---

## Test Run Results

### prism-core — e_cred filter (RG-ECRED-001/002/004/005 + AC-007)

Command: `cargo nextest run -p prism-core -E 'test(e_cred)'`

```
Starting 5 tests across 20 binaries (241 tests skipped)
    PASS [   0.010s] (1/5) prism-core::e_cred_namespace_invariants test_e_cred_keyring_error_variant_retired
    PASS [   0.011s] (2/5) prism-core::ac_5_prism_error_display test_ac5_prism_error_display_e_cred_007_key_missing
    PASS [   0.011s] (3/5) prism-core::ac_5_prism_error_display test_ac5_prism_error_display_e_cred_001
    PASS [   0.011s] (4/5) prism-core::ac_5_prism_error_display test_ac5_prism_error_display_e_cred_006_encryption
    PASS [   0.067s] (5/5) prism-core::e_cred_namespace_invariants test_e_cred_005_emitted_only_by_file_io_path
────────────
 Summary [   0.069s] 5 tests run: 5 passed, 241 skipped
```

Result: **5/5 PASS**

### prism-credentials — BC_2_03_009 filter (RG-ECRED-003 + AC-010)

Command: `cargo nextest run -p prism-credentials -E 'test(BC_2_03_009)'`

```
Starting 10 tests across 9 binaries (116 tests skipped)
    PASS [   0.012s] ( 1/10) prism-credentials::bc_2_03_009_resolve_secret test_BC_2_03_009_direct_env_var_used_when_no_file_env
    PASS [   0.012s] ( 2/10) prism-credentials::bc_2_03_009_resolve_secret test_BC_2_03_009_rejects_nonexistent_file_with_credential_error
    PASS [   0.012s] ( 3/10) prism-credentials::bc_2_03_009_resolve_secret test_BC_2_03_009_neither_set_returns_none
    PASS [   0.012s] ( 4/10) prism-credentials::bc_2_03_009_resolve_secret test_BC_2_03_009_rejects_directory_path_with_credential_error
    PASS [   0.012s] ( 5/10) prism-credentials::bc_2_03_009_resolve_secret test_BC_2_03_009_invariant_resolved_value_is_secret_string
    PASS [   0.012s] ( 6/10) prism-credentials::bc_2_03_009_resolve_secret test_BC_2_03_009_resolve_secret_file_io_emits_e_cred_005
    PASS [   0.013s] ( 7/10) prism-credentials::bc_2_03_009_resolve_secret test_BC_2_03_009_empty_file_resolves_to_empty_secret
    PASS [   0.013s] ( 8/10) prism-credentials::bc_2_03_009_resolve_secret test_BC_2_03_009_file_env_var_reads_file_and_strips_newline
    PASS [   0.013s] ( 9/10) prism-credentials::bc_2_03_009_resolve_secret test_BC_2_03_009_trailing_newline_stripped_from_file_content
    PASS [   0.013s] (10/10) prism-credentials::bc_2_03_009_resolve_secret test_BC_2_03_009_file_wins_when_both_set
────────────
 Summary [   0.013s] 10 tests run: 10 passed, 116 skipped
```

Result: **10/10 PASS**

---

## Runtime Display String Evidence

The following canonical Display strings are emitted by the code at branch HEAD c63b126e.
No credential values appear in any string (AD-017 compliant — all strings are error
message templates, not credential resolution outputs).

### E-CRED-006 — CredentialEncryptionError (AC-001)

Source: `crates/prism-core/src/error.rs` line 155

```
#[error("E-CRED-006: credential encryption error: {reason}")]
```

### E-CRED-007 — EncryptionKeyMissing (AC-002)

Source: `crates/prism-core/src/error.rs` line 160

```
#[error("E-CRED-007: encryption key not configured: {reason}")]
```

### E-CRED-005 — CredentialFileIo strings (AC-003, all three sub-cases)

Source: `crates/prism-credentials/src/resolve_secret.rs`

```
line 41: "E-CRED-005: credential file I/O error for '{}': file does not exist (configured in env var '{}')"
line 52: "E-CRED-005: credential file I/O error for '{}': path is a directory, not a regular file"
line 63: "E-CRED-005: credential file I/O error for '{}': read failed: {}"
```

### No-collision verification (AC-009)

`git grep 'E-CRED-005' -- crates/` on branch HEAD returns exactly:

```
crates/prism-credentials/src/resolve_secret.rs:41:  "E-CRED-005: ...file does not exist..."
crates/prism-credentials/src/resolve_secret.rs:52:  "E-CRED-005: ...path is a directory..."
crates/prism-credentials/src/resolve_secret.rs:63:  "E-CRED-005: ...read failed..."
```

No hit in `crates/prism-core/src/error.rs` or any other crate. Collision fully resolved.

---

## Spec-Only ACs (no code path — evidence is spec artifact)

| AC | Spec File | Verifying Mechanism |
|----|-----------|---------------------|
| AC-004 | `crates/prism-core/src/error.rs` — `KeyringError` variant absent | `test_e_cred_keyring_error_variant_retired` (compile-time guard via `rg` subprocess in RG-ECRED-004) |
| AC-005 | `crates/prism-mcp/src/error_mapping.rs` — dead arm deleted | `just check` exit 0 (exhaustive match verified by compiler) |
| AC-006 | `.factory/specs/prd-supplements/error-taxonomy.md` — canonical 10-row E-CRED table | Adversarial sweep passes 1-12 (LOCAL cascade); `rg 'E-CRED-005.*keyring\|E-CRED-001.*keyring' .factory/specs/` returns zero hits |
| AC-008 | `BC-2.06.003` + `ADR-034 §D4` — all E-CRED-005 keyring cites → E-CRED-008 | `rg 'E-CRED-005' .factory/specs/behavioral-contracts/BC-2.06.003*` and `rg 'E-CRED-005' .factory/specs/architecture/decisions/ADR-034*` return zero hits |
| AC-011 | `security-architecture.md` v1.2 + `interface-definitions.md` v2.7 — not-found node cites E-CRED-002 | DF-PASS3-001 closure; story v1.1 changelog entry; `rg 'E-CRED-001' .factory/specs/architecture/security-architecture.md` (resolution-chain context) returns zero hits |

---

## AD-017 Compliance Note

All recordings and transcripts in this evidence folder show only error message
template strings — format strings with `{reason}`, `{}`, `{path}` placeholders.
No resolved credential value, no auth token, no API key, and no secret material
appears in any recording. AD-017 is not violated.
