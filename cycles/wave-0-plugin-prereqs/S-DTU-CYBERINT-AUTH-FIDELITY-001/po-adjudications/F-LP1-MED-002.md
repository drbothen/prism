---
document_type: po-adjudication
finding_id: F-LP1-MED-002
story: S-DTU-CYBERINT-AUTH-FIDELITY-001
pass: local-pass-1
decision: Option B-revisited — REVERT v1.1 amendment; BC v1.0 EC-017-005 was correct
date: 2026-05-30
author: product-owner
revision: v2 (2026-05-30 — supersedes v1 adjudication which was based on fabricated evidence)
---

# PO Adjudication: F-LP1-MED-002 (v2 — Re-adjudication)

## Prior Adjudication Error (honest self-correction required by Canonical Principle Rule 4)

The v1 adjudication (D-852, 2026-05-30) chose Option A (BC amendment — impl wins) and produced BC-2.01.017 v1.1. That decision was **based on a fabricated claim** about `prism_credentials::resolve_secret` behavior: the v1 adjudication asserted that "BC-2.03.006 normalizes empty strings as not-found." This claim does not appear in BC-2.03.006's text, and it is not implemented in the production code. The v1 adjudication evidence section did not include verbatim code quotes from `resolve_secret.rs` — which is the specific failure that let the fabrication propagate. This adjudication corrects the error.

This is a self-correction under CLAUDE.md §Canonical Principle Rule 4 ("AI-built defects are the AI's responsibility to fix"). The v1 error is fully acknowledged, not minimized.

## Finding (verbatim from local-pass-1.md)

> BC vs implementation: empty-resolved api_key returns E-AUTH-005 (resolver path) but
> BC-2.01.017 §Error Cases EC-017-005 mandates E-AUTH-006. Test was tuned to behavior
> (TD-VSDD-059 paper-fix risk). Routing: product-owner (decide: BC amendment vs impl fix;
> production-grade default favors impl fix since BC is canonical source-of-truth)

## Orchestrator-Verified Ground Truth

### resolve_secret.rs direct-env branch (verbatim, lines 77-83)

```rust
// Priority 2: check {direct_env} env var
if let Ok(value) = std::env::var(direct_env) {
    return Ok(Some(SecretString::new(value.into())));
}

// Priority 3: neither set
Ok(None)
```

**There is NO `is_empty()` filter, NO whitespace check, NO normalization.** `std::env::var("FOO")` returns:
- `Ok(String)` when `FOO` is SET — including when `FOO=""` (empty string), when `FOO="   "` (whitespace), and when `FOO="valid-key"`.
- `Err(VarError::NotPresent)` when `FOO` is NOT SET.

Therefore:
- `CYBERINT_API_KEY=""` → `std::env::var` returns `Ok("")` → resolver returns `Ok(Some(SecretString("")))` → propagates to `acquire_token`'s `is_empty()` check → **E-AUTH-006**.
- `CYBERINT_API_KEY` not set → `std::env::var` returns `Err(NotPresent)` → `if let Ok(value)` arm does NOT execute → falls through to `Ok(None)` → `acquire_token` sees not-found → **E-AUTH-005**.

These are two distinct paths; the empty-string path does NOT produce `Ok(None)`.

### What BC-2.03.006 actually says about value semantics

BC-2.03.006 (version 1.3, "Credential Resolution at Sensor Query Time") postconditions verbatim:

> - The credential is resolved from the active backend using the `(client_id, sensor_id, credential_name)` namespace
> - The resolved credential is passed to the `SensorAuth` implementation (OAuth2, Cookie, Bearer) as a `SecretString`
> - Credential resolution is audit-logged (tenant, sensor, credential name -- never the value)
> - If resolution fails, the sensor query fails with a clear error before any API call is attempted

**BC-2.03.006 says nothing about normalizing empty strings as not-found.** The only "resolution fails" case it describes is when the credential is not found for the configured `credential_ref`. It does not define empty-value normalization behavior. The v1 adjudication's assertion that "BC-2.03.006 normalizes empty strings as not-found" was fabricated — no such text exists in BC-2.03.006.

### BC-2.03.009 (the file-pattern resolver — cited in resolve_secret.rs docstring)

BC-2.03.009 (`resolve-secret-env-file.md`) covers the `{NAME}_FILE` → `{NAME}` env var resolution pattern. It specifies that file contents have trailing newlines stripped (lines 69-74 of resolve_secret.rs). It also makes no claim about empty-string normalization.

## Chosen Direction: Option B-revisited (REVERT v1.1; BC v1.0 EC-017-005 was correct)

Evidence finding: the production resolver (`resolve_secret.rs` lines 78-81) returns `Ok(Some(SecretString("")))` for `CYBERINT_API_KEY=""`. BC-2.01.017 v1.0 originally specified `E-AUTH-006` for empty/whitespace api_key. That was correct. The v1.1 amendment was wrong.

There is no evidence the project wants "empty env var → not-found" semantics. BC-2.03.006 does not claim it. The code does not implement it. If the project were to adopt that policy in the future, it would require:
1. A code change to `resolve_secret.rs` adding `if value.is_empty() { return Ok(None); }` at line 79,
2. A BC-2.03.006 amendment adding a "value normalization" postcondition,
3. A new ADR or decision row recording the policy,
4. BC-2.01.017 keeping v1.1 semantics (E-AUTH-005 for empty value).

None of those changes have been made. The resolver-as-shipped propagates empty strings. **Reverting to v1.0 semantics is correct.**

## BC-2.01.017 v1.2 Changes

### EC-017-005 (final v1.2 text)

```
| EC-017-005 | Empty or whitespace-only API key: env var set to `""`, env var set to
| all-whitespace (e.g., `"   "`), or credential backend resolves to an empty/whitespace
| string | `E-AUTH-006`. `prism_credentials::resolve_secret` (lines 78-81) performs NO
| empty-string normalization on the direct-env path — `std::env::var("FOO")` returns `Ok("")`
| for `FOO=""`, which is wrapped as `Ok(Some(SecretString("")))`. This reaches `acquire_token`'s
| `is_empty()` check (or `chars().all(char::is_whitespace)` for whitespace-only) and returns
| E-AUTH-006. The resolver does NOT return `Ok(None)` for this path. Contrast with EC-017-003
| (env var not set at all → `std::env::var` returns `Err` → resolver returns `Ok(None)` →
| `E-AUTH-005`).
```

### TV-BC-2.01.017-005 (final v1.2 text)

MockCredentialResolver configured to return `Ok(SecretString(""))` → `acquire_token()` returns `Err` containing `E-AUTH-006`. The mock simulates what `resolve_secret` actually returns for an empty env var. Separate test for not-found path uses `NotFoundCredentialResolver` asserting `E-AUTH-005` (covered by TV-BC-2.01.017-004 behavior or a new split test — see implementer follow-on below).

## Error Taxonomy Alignment

- **E-AUTH-005** ("Credentials not found for ({client_id}, {sensor_id})"): fires when the credential reference resolves to `Ok(None)` — i.e., env var NOT SET (`std::env::var` returns `Err`). The "not found" path.
- **E-AUTH-006** ("Empty or invalid API key for cookie_roundtrip sensor '{sensor}' on client '{client_id}'"): fires when the resolver returns `Ok(Some(SecretString(value)))` but `value` fails validation (empty, all-whitespace, >4096 bytes, RFC 6265-illegal chars). This INCLUDES the case where env var is SET to empty string.

These semantics are internally consistent and match the production code. No error-taxonomy.md amendment is needed for this re-adjudication — the E-AUTH-006 row's ORIGINAL semantics (before v1.1 muddied them with the "not found" reframing) are correct.

## Implementer Follow-On Dispatch (REVISED from v1)

The v1 adjudication directed implementer to add `assert!(err_str.contains("E-AUTH-005"))` to `test_static_cookie_auth_provider_rejects_empty_api_key`. That directive was WRONG and must NOT be applied.

**Revised implementer follow-on:** Split `test_static_cookie_auth_provider_rejects_empty_api_key` into two tests:

### Test 1: Empty value → E-AUTH-006
```rust
#[tokio::test]
async fn test_static_cookie_auth_provider_empty_value_returns_e_auth_006() {
    // MockCredentialResolver returns Ok(Some(SecretString(""))) — simulates CYBERINT_API_KEY=""
    let resolver = MockCredentialResolver::returns_ok(SecretString::new("".into()));
    let provider = StaticCookieAuthProvider::new(resolver, "cyberint");
    let result = provider.acquire_token("client-001", "cyberint").await;
    assert!(result.is_err());
    let err_str = result.unwrap_err().to_string();
    assert!(
        err_str.contains("E-AUTH-006"),
        "empty env var value must yield E-AUTH-006 (value-validation), not E-AUTH-005 (not-found). Got: {err_str}"
    );
}
```

### Test 2: Credential not found → E-AUTH-005
```rust
#[tokio::test]
async fn test_static_cookie_auth_provider_missing_credential_returns_e_auth_005() {
    // NotFoundCredentialResolver returns Ok(None) — simulates env var not set
    let resolver = NotFoundCredentialResolver::new();
    let provider = StaticCookieAuthProvider::new(resolver, "cyberint");
    let result = provider.acquire_token("client-001", "cyberint").await;
    assert!(result.is_err());
    let err_str = result.unwrap_err().to_string();
    assert!(
        err_str.contains("E-AUTH-005"),
        "missing credential must yield E-AUTH-005 (not-found), not E-AUTH-006 (value-validation). Got: {err_str}"
    );
}
```

Both tests close the TD-VSDD-059 concern: specific error code assertions prevent future test-tuning from obscuring which code path fired. This TWO-test structure is required because the two failure modes (not-found vs empty-value) are at different code sites and require different mock setups.

## Lesson Learned (process-gap codification)

**Future PO adjudications resolving code-vs-spec conflicts MUST include verbatim code quotes from the cited code path.** The v1 adjudication's evidence section cited code from `auth_provider.rs` (the auth provider), but did NOT read or quote `crates/prism-credentials/src/resolve_secret.rs` (the credential resolver). The fabricated "BC-2.03.006 normalization" claim would have been immediately falsifiable if the adjudicator had read resolve_secret.rs lines 78-81. This is a TD-VSDD-059 variant: paper-evidence is as dangerous as paper-fix. Verbatim code quotes from the ACTUAL execution path — not from adjacent code — are required.

Add to orchestrator process-gap codification queue: "PO adjudication on code-vs-spec conflict: must quote verbatim from the specific execution path (not adjacent code, not inferred behavior). No code quote = finding not closed."
