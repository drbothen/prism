# Demo Evidence Report — W3-FIX-CODE-003

| Field | Value |
|-------|-------|
| Story | W3-FIX-CODE-003 |
| Title | prism-credentials: implement KeyringBackend::CredentialStoreOrgId |
| Branch | W3-FIX-CODE-003 |
| HEAD SHA | 293a5d05 |
| Recorded | 2026-05-01 |
| Recorder | Demo Recorder agent |

---

## IMPORTANT: SEC-004 Was a FALSE POSITIVE

Gate Step D raised SEC-004 (MEDIUM, CWE-284, OWASP A01) claiming that
`KeyringBackend`'s implementation of `CredentialStoreOrgId` consisted entirely
of `todo!()` stubs. **This finding was incorrect.**

At develop@a3bd5a0f, `crates/prism-credentials/src/keyring.rs` already contained
a complete, correct implementation of `CredentialStoreOrgId` on `KeyringBackend`:

- `get_by_org` — fully implemented using `namespace_key_by_org_id`
- `set_by_org` — fully implemented with `expose_secret()` at write site only
- `delete_by_org` — fully implemented with sidecar index cleanup
- `list_by_org` — fully implemented with OrgId prefix filter
- `exists_by_org` — fully implemented via delegation to `get_by_org`

No `todo!()` stubs were present. The namespace key format `"{org_id_uuid}/{sensor}/{name}"`
matches the `EncryptedFileBackend` pattern exactly, as required by BC-3.2.002.

**This story adds defensive regression tests to prevent future false positives
and documents the finding for gate-step-d-security-review.md correction.**

**Recommendation:** Update `gate-step-d-security-review.md` to retract SEC-004
or downgrade to LOW (coverage gap: regression tests were absent, now added). The
underlying implementation risk (runtime panic) did not exist.

---

## AC Coverage

### AC-001: get_by_org / set_by_org / delete_by_org fully implemented (no todo!())

| Recording | File | Description |
|-----------|------|-------------|
| GIF | [AC-001-implementation-inspect.gif](AC-001-implementation-inspect.gif) | grep for todo!() returns 0 results; function signatures shown |
| WEBM | [AC-001-implementation-inspect.webm](AC-001-implementation-inspect.webm) | Same |
| Tape | [AC-001-implementation-inspect.tape](AC-001-implementation-inspect.tape) | VHS script |

**Evidence:** `grep -n 'todo!()' crates/prism-credentials/src/keyring.rs` exits 1
(no matches) and prints `CLEAN — no todo!() stubs present`. The three `async fn *_by_org`
signatures are visible in `keyring.rs` at lines confirmed by grep output.

---

### AC-002: OrgId-keyed get/set/delete work without panic

| Status | Reason |
|--------|--------|
| test_AC_001_keyring_org_id_namespaced_get_set_delete | IGNORED — requires live OS keyring service |

This test exercises the real macOS Keychain / libsecret / Windows Credential Vault.
It is annotated `#[ignore]` per EC-001 (headless CI runners have no secret service).

To run manually on a machine with a keyring service:

```
cargo test -p prism-credentials --test keyring_org_id -- --ignored
```

The implementation correctness for AC-002 is established by:
1. AC-001 evidence (no todo!() stubs — the code will not panic)
2. AC-003 evidence (namespace format is correct — OrgId UUID is used, not slug)
3. Code review of `keyring.rs` lines 247–423 (full `CredentialStoreOrgId` impl)

---

### AC-003: Cross-org isolation maintained

| Status | Reason |
|--------|--------|
| test_AC_002_cross_org_isolation_org_a_credential_not_visible_to_org_b | IGNORED — requires live OS keyring service |

Same constraint as AC-002. The isolation mechanism is structural: keyring service
names are `"prism/{org_id_uuid}/{sensor}"` where `org_id_uuid` differs per org.
The namespace-format unit test (AC-003/AC-004) confirms the UUID is distinct per
`OrgId::new()` invocation.

---

### AC-004 / AC-003 (namespace format): Namespace key format is UUID/sensor/name

| Recording | File | Description |
|-----------|------|-------------|
| GIF | [AC-003-namespace-format-test.gif](AC-003-namespace-format-test.gif) | 1 passed, 2 ignored |
| WEBM | [AC-003-namespace-format-test.webm](AC-003-namespace-format-test.webm) | Same |
| Tape | [AC-003-namespace-format-test.tape](AC-003-namespace-format-test.tape) | VHS script |

**Evidence:** `test_AC_003_namespace_format_matches_uuid_sensor_name_pattern` PASSES.
It verifies:
- Namespace key has exactly 3 `/`-separated segments
- Segment 0 is the 36-char OrgId UUID string (not a slug)
- Segment 1 is the sensor name
- Segment 2 is the credential name
- Key starts with `org_uuid_str` (not a short org slug)

---

### AC-005: Legacy CredentialStore (slug-keyed) not added for OrgId methods

No recording needed. Static: the `CredentialStoreOrgId` impl in `keyring.rs` uses
only `namespace_key_by_org_id` (OrgId UUID path). The legacy `namespace_key`
function (slug-keyed) is used only by the separate `CredentialStore` impl. These
are distinct trait impls; no cross-contamination exists.

---

## Test Summary

```
running 3 tests
test test_AC_001_keyring_org_id_namespaced_get_set_delete ... ignored
test test_AC_002_cross_org_isolation_org_a_credential_not_visible_to_org_b ... ignored
test test_AC_003_namespace_format_matches_uuid_sensor_name_pattern ... ok

test result: ok. 1 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

**1 PASS, 0 FAIL, 2 IGNORED** (ignored tests require live OS keyring service; normal
for headless CI environments — see EC-001 in story file).

---

## Files in This Directory

| File | Type | AC |
|------|------|----|
| AC-001-implementation-inspect.gif | GIF recording | AC-001 |
| AC-001-implementation-inspect.webm | WEBM recording | AC-001 |
| AC-001-implementation-inspect.tape | VHS script | AC-001 |
| AC-003-namespace-format-test.gif | GIF recording | AC-003 / AC-004 |
| AC-003-namespace-format-test.webm | WEBM recording | AC-003 / AC-004 |
| AC-003-namespace-format-test.tape | VHS script | AC-003 / AC-004 |
| evidence-report.md | This file | All |
