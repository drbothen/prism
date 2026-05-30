---
document_type: po-adjudication
finding_id: F-LP1-MED-002
story: S-DTU-CYBERINT-AUTH-FIDELITY-001
pass: local-pass-1
decision: Option A — BC amendment (impl wins)
date: 2026-05-30
author: product-owner
---

# PO Adjudication: F-LP1-MED-002

## Finding (verbatim from local-pass-1.md line 45)

> BC vs implementation: empty-resolved api_key returns E-AUTH-005 (resolver path) but
> BC-2.01.017 §Error Cases EC-017-005 mandates E-AUTH-006. Test was tuned to behavior
> (TD-VSDD-059 paper-fix risk). Routing: product-owner (decide: BC amendment vs impl fix;
> production-grade default favors impl fix since BC is canonical source-of-truth)

## Decision: Option A — BC amendment (impl wins)

The BC error-cases table entry for EC-017-005 is **factually incorrect** about when
E-AUTH-006 fires. The implementation is semantically correct. Amend BC-2.01.017
EC-017-005 to match implementation behavior.

## Evidence

### E-AUTH-005 and E-AUTH-006 semantic definitions (error-taxonomy.md v1.53)

- **E-AUTH-005**: "Credentials not found for ({client_id}, {sensor_id})" — resolver found
  NO entry in keyring or file backend. The credential reference does not resolve at all.
- **E-AUTH-006**: "Empty or invalid API key for cookie_roundtrip sensor '{sensor}' on
  client '{client_id}'" — resolver SUCCEEDED but returned an empty string, all-whitespace
  string, string exceeding 4096 bytes, or string with illegal RFC 6265 cookie characters.

These are **two distinct failure modes** at two distinct code sites.

### Implementation: auth_provider.rs lines 351-369 (resolve → validate split)

```rust
// SITE 1: resolver.resolve() fails (credential not found) → E-AUTH-005
let secret = resolver
    .resolve(&client_id_str, &sensor_id, "api_key")
    .await
    .map_err(|e| SpecEngineError::AuthAcquisitionFailed {
        detail: format!("E-AUTH-005: credential not found: {e}"),
    })?;

// SITE 2: resolver succeeds but api_key is empty/whitespace → E-AUTH-006
let api_key = secret.expose_secret().to_string();
if api_key.is_empty() || api_key.chars().all(char::is_whitespace) {
    return Err(SpecEngineError::AuthAcquisitionFailed {
        detail: "E-AUTH-006: api_key is empty or all-whitespace".to_string(),
    });
}
```

The implementation is architecturally correct. The two-site split is the intended design:
E-AUTH-005 = resolver lookup failure; E-AUTH-006 = resolver success with invalid value.

### EC-017-005 in BC-2.01.017 (the defective entry)

> EC-017-005 | API key string contains only whitespace | `E-AUTH-006` (empty/invalid API key).
> Whitespace-only credentials are semantically equivalent to empty.

The **condition** ("API key string contains only whitespace") is the scenario where the
resolver SUCCEEDS and returns an all-whitespace string. The implementation correctly
returns E-AUTH-006 for this case (Site 2 above). So far, no conflict.

**The adversary's finding references the "resolver path" returning E-AUTH-005.** This is
the additional scenario where an EMPTY ENV VAR causes the resolver to treat the credential
as not-found (E-AUTH-005 at Site 1), rather than returning an empty string to Site 2.

Reading `crates/prism-spec-engine/src/auth_provider.rs` unit test
`test_static_cookie_auth_provider_rejects_empty_api_key` (lines 778-802), the test
author noted this explicitly:

> // Empty string: resolve_secret returns Ok(None) → NotFound path, not E-AUTH-006.
> // An empty env var is treated as "not set" by resolve_secret (it filters empty strings).
> // This is correct per BC-2.03.006 semantics — the E-AUTH-005 path fires.

This confirms: the credential backend (`prism_credentials::resolve_credential`) FILTERS
empty env-var values at the resolver level, treating them as "not found" (returns `Err`)
rather than returning `Ok("")` to the auth provider. So an EMPTY api_key value never
reaches Site 2 — it is caught at Site 1 as E-AUTH-005.

### The BC authoring error

BC-2.01.017 §Error Cases row 1 and EC-017-005 were authored assuming the resolver would
return `Ok("")` for an empty credential, which would then be caught at Site 2 (E-AUTH-006).
The actual credential backend (per BC-2.03.006) filters empty values at the resolver layer,
so `Ok("")` never arrives at `acquire_token`. The conceptual model in the BC (resolver
returns empty string → E-AUTH-006) does not match the actual credential-backend contract.

### Is this a TD-VSDD-059 paper-fix?

No. The adversary's concern is legitimate — the test
`test_static_cookie_auth_provider_rejects_empty_api_key` asserts `result.is_err()` without
asserting the specific error code. The test does NOT assert E-AUTH-005 vs E-AUTH-006, so
the TD-VSDD-059 paper-fix smell is valid for the TEST, not for the BC amendment. The BC
amendment makes the contract accurate; the implementer should also strengthen the test
to assert `E-AUTH-005` in the empty-string case. Documented in follow-on below.

## Semantic justification for Option A

E-AUTH-005 is the SEMANTICALLY CORRECT code for the "empty env-var / empty resolver value"
failure mode because `prism_credentials::resolve_credential` (per BC-2.03.006) normalizes
empty string credentials as "not found." The resolver returns `Err(not_found)` — not
`Ok("")` — making this a credential-not-found failure, not a value-validation failure.
E-AUTH-006 correctly covers the case where the resolver succeeds with a non-empty but
invalid value (semicolons, length exceeded, all-whitespace where the credential backend
did NOT filter it). These are distinct failure modes that do not overlap in practice.

The BC author conflated "empty API key value" (a value-validation concern, E-AUTH-006)
with "API key value that is empty because the credential backend filtered it as not-found"
(a lookup concern, E-AUTH-005). The implementation correctly distinguishes them.

## BC-2.01.017 amendment required

### EC-017-005 (replace)

**Old text:**
> EC-017-005 | API key string contains only whitespace | `E-AUTH-006` (empty/invalid API key).
> Whitespace-only credentials are semantically equivalent to empty.

**New text (to be applied by product-owner in this commit):**
> EC-017-005 | Credential backend returns no value for the api_key reference (empty env var,
> missing keyring entry, or env var set to empty string — all treated as "not found" by
> `prism_credentials::resolve_credential` per BC-2.03.006) | `E-AUTH-005`. The resolver
> returns `Err(not_found)` before the value-validation branch; the empty-string value
> never reaches the whitespace/length/character checks. Contrast with EC-017-004 (non-empty
> value with illegal characters → E-AUTH-006) and the Error Cases table row for all-whitespace
> keys resolved as Ok (rare; requires a credential backend that does NOT filter whitespace —
> not the default prism_credentials behavior).

### Error Cases table row for E-AUTH-006 (add clarifying note)

The existing E-AUTH-006 row: "Credential resolver returns an empty string value for the API
key" should be amended to: "Credential resolver returns a non-empty but invalid value: all
whitespace, exceeds 4096 bytes, or contains RFC 6265-illegal characters. NOTE: prism_credentials
treats empty strings as not-found (E-AUTH-005); E-AUTH-006 fires when the resolver succeeds
with a non-empty invalid value." This preserves the existing semantics while clarifying the
boundary.

### Test strengthening (implementer follow-on)

The test `test_static_cookie_auth_provider_rejects_empty_api_key` (auth_provider.rs line 778)
asserts `result.is_err()` but does not assert `E-AUTH-005`. Implementer must add:
```rust
let err_str = result.unwrap_err().to_string();
assert!(err_str.contains("E-AUTH-005"), "empty env var must yield E-AUTH-005: {err_str}");
```
This closes the TD-VSDD-059 concern. See Follow-on dispatch below.

## Authority citations

- CLAUDE.md §Source-of-Truth Precedence rule 7 (code-vs-spec → SPEC wins) does NOT apply
  here because the BC-2.01.017 EC-017-005 error is a BC authoring error against a different
  spec layer: BC-2.03.006 (credential resolution contract). The BC-2.01.017 failure mode
  description is factually wrong about the semantics of the credential backend. The
  implementation correctly reflects BC-2.03.006's normalization behavior. This is a case
  where TWO specs disagree (BC-2.01.017 vs BC-2.03.006), and the later/more-specific spec
  (BC-2.03.006 for the credential backend) governs the resolution path. BC-2.01.017 must
  be updated to align with BC-2.03.006.
- CLAUDE.md §Canonical Principle Rule 1 (no MVP deferrals): fixing the BC now, not deferring
  to Phase 5 adversarial refinement.
- CLAUDE.md §Canonical Principle Rule 4 (AI-built defects are AI's responsibility to fix):
  the BC authoring error was AI-produced; amending it in-scope is the correct path.

## Follow-on dispatch (implementer)

Implementer must add E-AUTH-005 assertion to
`crates/prism-spec-engine/src/auth_providers/mod.rs` (or `src/auth_provider.rs` in the
worktree) unit test `test_static_cookie_auth_provider_rejects_empty_api_key`:

File: `crates/prism-spec-engine/src/auth_provider.rs`
Function: `test_static_cookie_auth_provider_rejects_empty_api_key` (line ~778)
Change: After `assert!(result.is_err(), ...)`, add:
```rust
let err_str = result.unwrap_err().to_string();
assert!(
    err_str.contains("E-AUTH-005"),
    "empty env var resolves to not-found (E-AUTH-005), not empty-value (E-AUTH-006). Got: {err_str}"
);
```
This closes TD-VSDD-059 for the test. The BC amendment (this commit) closes the spec-side
gap. Both changes must land before the story is marked GREEN.

Note to orchestrator: dispatch implementer for the test-assertion strengthening after this
commit is verified. The BC amendment is in `.factory/`; the test change is in the worktree.
These are two different artifact domains — BC amendment is product-owner scope (done here),
test assertion is implementer scope (follow-on).
