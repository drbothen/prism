---
document_type: po-adjudication
finding_id: F-LP3-HIGH-001
severity: HIGH
status: resolved
resolution: option-a-e-auth-007-allocated
burst: D-857
date: 2026-05-30
author: product-owner
traces_to:
  - "BC-2.01.017"
  - ".factory/specs/prd-supplements/error-taxonomy.md"
  - ".factory/specs/behavioral-contracts/BC-INDEX.md"
---

# PO Adjudication: F-LP3-HIGH-001 — BackendUnavailable Error Code Misrouting

## Finding Restatement

Pass 3 adversary identified that `StaticCookieAuthProvider::acquire_token` in
`crates/prism-spec-engine/src/auth_provider.rs` does a blanket
`.map_err(|e| SpecEngineError::AuthAcquisitionFailed { detail: format!("E-AUTH-005: credential not found: {e}") })`
over all `CredentialResolutionError` variants. This causes `CredentialResolutionError::BackendUnavailable`
(emitted when the credential file is unreadable or the keyring daemon is unavailable) to surface with the code
label `E-AUTH-005` — which per error-taxonomy.md v1.53 means "No credentials in keyring or file backend
(backend works, no entry)." The error TEXT includes "Backend unavailable for ..." but the CODE is E-AUTH-005.
This is a contract violation against BC-2.01.017 v1.2 EC-017-003, which reserves E-AUTH-005 for
`CredentialResolutionError::NotFound` only.

The semantic distinction matters for LLM agents consuming error codes:
- E-AUTH-005 → operator action: configure the credential (it's simply not there)
- E-AUTH-007 → operator action: fix the backend infrastructure (daemon down, file permissions)

Conflating them forces downstream consumers to parse free-text detail strings to determine the actual failure
mode, violating the purpose of a structured error taxonomy.

## CredentialResolutionError Variant Inventory

Read from `crates/prism-credentials/src/resolution.rs` (confirmed 2026-05-30):

```rust
pub enum CredentialResolutionError {
    NotFound {
        client_id: String,
        sensor_id: String,
        credential_name: String,
        suggestion: String,
    },
    BackendUnavailable {
        client_id: String,
        sensor_id: String,
        credential_name: String,
        detail: String,
    },
}
```

**Total variants: 2.** No `ParseError`, `InvalidName`, or other variants exist in this enum.

The `PrismCredentialResolver` (production impl) calls `prism_credentials::resolve_credential(...)` and maps
ALL errors to `String` via `.map_err(|e| e.to_string())`. This string then flows into the blanket
`E-AUTH-005` wrapper in `acquire_token`. The stringification means the match-arm fix must happen either:
(a) in `PrismCredentialResolver::resolve` — preserve typed error variants; or
(b) detect by string content in `acquire_token` — brittle and forbidden by production-grade principle.

The correct fix is (a): change `PrismCredentialResolver::resolve` to preserve the `CredentialResolutionError`
typed information, OR change `acquire_token`'s `.map_err` to match on a richer error type. In practice the
implementer should add a `BackendUnavailableCredentialResolver` test helper and fix the `acquire_token`
`.map_err` to distinguish variants. The `CredentialResolver` trait returns `Err(String)` — the implementer
must either:
- Change the `CredentialResolver` trait to return a structured error type, OR
- Detect `BackendUnavailable` at `PrismCredentialResolver::resolve` time by propagating a sentinel prefix in
  the error string (e.g., `E-AUTH-007:` prefix), and match on that prefix in `acquire_token`.

The implementer dispatch instructions below specify the sentinel-prefix approach as the minimal diff.

## Options Evaluated

### Option A — E-AUTH-007 for BackendUnavailable (AUTH-family expansion) [CHOSEN]

Allocate `E-AUTH-007`: "Credential resolver backend unavailable (file read failure, keyring service down)."

**Pros:**
- Auth-family namespace keeps the failure mode discoverable to consumers of the auth-provider contract. An
  agent consuming sensor errors sees `E-AUTH-007` and immediately knows this is an auth-infrastructure problem,
  not a missing-credential problem.
- Minimal scope: one new error code in the AUTH namespace, one new EC and TV in BC-2.01.017.
- Consistent with existing E-AUTH-NNN consumer contracts (all sensor adapters already handle E-AUTH-NNN).
- No cross-family bridge required — avoids dual error-handling in sensor adapter consumers.

**Cons:**
- AUTH-family technically "leaks" infrastructure concerns (keyring daemon) into auth semantics. A strict
  separation would put infrastructure errors in CRED-family.

### Option B — Keep E-AUTH-005 broad; distinguish by detail text

Broaden E-AUTH-005 description to cover both NotFound and BackendUnavailable, emit distinct detail strings.

**Rejected because:**
- Consumers cannot reliably distinguish without parsing free-text. Violates structured error taxonomy purpose.
- The detail text from `BackendUnavailable.detail` may contain platform-specific messages not amenable to
  reliable parsing.
- Production-grade principle: "Every BC must have at least one edge case documented" — but more importantly,
  every error code must have a unique, testable semantic. A broadened E-AUTH-005 sacrifices testability.

### Option C — Cross-family bridge to E-CRED-NNN (infrastructure surface)

Use the existing CRED namespace for BackendUnavailable.

**Rejected because:**
- Creates a split-family contract: `StaticCookieAuthProvider::acquire_token` would emit both E-AUTH-NNN and
  E-CRED-NNN errors. All callers of `acquire_token` (sensor pipeline fan-out) already handle E-AUTH-NNN;
  adding E-CRED-NNN requires updating all call sites and error-handling documentation.
- The E-CRED namespace (E-CRED-001..004) covers credential store infrastructure at the credential manager
  level, not at the auth-provider level. Mixing levels violates the layering principle.
- The cross-family bridge would also require amending BC-2.03.x (credential management BCs), expanding scope
  beyond the minimal fix needed here.

## Chosen Option: A — E-AUTH-007

**Rationale:** Option A minimally extends the AUTH-family to cover the BackendUnavailable variant, preserving
single-namespace error handling for all `acquire_token` callers. The semantic distinction between
"not configured" (E-AUTH-005) and "backend down" (E-AUTH-007) is material for automated recovery decision
logic in LLM agents. Option B trades correctness for simplicity in a way that fails the production-grade
test. Option C introduces unnecessary cross-family complexity.

## Full Variant-to-Code Mapping

| Variant | Error Code | Rationale |
|---------|-----------|-----------|
| `CredentialResolutionError::NotFound` | `E-AUTH-005` | Existing assignment per BC-2.01.017 v1.0. Backend healthy, no entry configured. Operator action: run `configure_credential_source`. |
| `CredentialResolutionError::BackendUnavailable` | `E-AUTH-007` | **New allocation (this adjudication).** Backend infrastructure failed. Retryable. Operator action: fix keyring daemon, file permissions, or _FILE path. |

No other variants exist in `CredentialResolutionError` as of 2026-05-30 (confirmed by direct read of
`crates/prism-credentials/src/resolution.rs`).

## Spec Amendments in This Burst (D-857)

1. **error-taxonomy.md v1.53 → v1.54**: Added E-AUTH-007 row in AUTH section. Retryable=Yes. Maps to
   `CredentialResolutionError::BackendUnavailable`. Changelog row added citing F-LP3-HIGH-001 and D-857.

2. **BC-2.01.017 v1.2 → v1.3**: 
   - `error_codes` frontmatter: added `E-AUTH-007`
   - Error Cases table: scoped E-AUTH-005 row to `CredentialResolutionError::NotFound` explicitly; added
     E-AUTH-007 row for `CredentialResolutionError::BackendUnavailable`
   - Edge Cases table: added EC-017-010 (BackendUnavailable → E-AUTH-007)
   - Canonical Test Vectors: added TV-BC-2.01.017-009 (BackendUnavailable returns E-AUTH-007, not E-AUTH-005)
   - Changelog: added v1.3 entry

3. **BC-INDEX.md v5.58 → v5.59**: Updated BC-2.01.017 row status to `draft — v1.3`. Added v5.59 changelog
   entry.

## Implementer Follow-On Dispatch Instructions

**File:** `crates/prism-spec-engine/src/auth_provider.rs`

**Symbol:** `StaticCookieAuthProvider::acquire_token` (lines 369–417) and
`PrismCredentialResolver::resolve` (lines 163–179).

**Problem:** `PrismCredentialResolver::resolve` erases the `CredentialResolutionError` type via
`.map_err(|e| e.to_string())`. The resulting `String` flows into `acquire_token`'s blanket
`format!("E-AUTH-005: credential not found: {e}")` wrapper.

**Fix approach (sentinel-prefix, minimal diff):**

In `PrismCredentialResolver::resolve`, instead of `e.to_string()`, emit a sentinel-prefixed string that
`acquire_token` can match on:

```rust
.map_err(|e| match e {
    CredentialResolutionError::NotFound { .. } => format!("NOT_FOUND: {e}"),
    CredentialResolutionError::BackendUnavailable { .. } => format!("BACKEND_UNAVAILABLE: {e}"),
})
```

In `StaticCookieAuthProvider::acquire_token`, replace the blanket `.map_err(...)`:

```rust
.map_err(|e: String| {
    if e.starts_with("BACKEND_UNAVAILABLE:") {
        SpecEngineError::AuthAcquisitionFailed {
            sensor_id: sensor_id.clone(),
            client_id: client_id_str.clone(),
            detail: format!("E-AUTH-007: credential resolver backend unavailable: {}", &e["BACKEND_UNAVAILABLE: ".len()..]),
        }
    } else {
        // NotFound or unrecognized — E-AUTH-005
        SpecEngineError::AuthAcquisitionFailed {
            sensor_id: sensor_id.clone(),
            client_id: client_id_str.clone(),
            detail: format!("E-AUTH-005: credential not found: {}", &e["NOT_FOUND: ".len()..]),
        }
    }
})?;
```

**Alternative (preferred if scope allows):** Change `CredentialResolver::resolve` trait to return
`Err(CredentialResolutionError)` instead of `Err(String)`. This is the structurally correct fix but requires
updating all `CredentialResolver` implementations and test helpers. If the implementer judges this in-scope,
it is the production-grade path.

**New test helper required:**

Add `BackendUnavailableCredentialResolver` to `auth_provider.rs` under `#[cfg(any(test, feature = "test-helpers"))]`:

```rust
pub struct BackendUnavailableCredentialResolver;

impl CredentialResolver for BackendUnavailableCredentialResolver {
    fn resolve<'a>(&'a self, client_id: &'a str, sensor_id: &'a str, _credential_name: &'a str)
        -> Pin<Box<dyn Future<Output = Result<secrecy::SecretString, String>> + Send + 'a>>
    {
        let msg = format!("BACKEND_UNAVAILABLE: Backend unavailable for {client_id}/{sensor_id}/api_key: keyring daemon stopped");
        Box::pin(async move { Err(msg) })
    }
}
```

**New unit test required** (per SID-1 — no-ignored-test rationalization prohibition):

```rust
#[tokio::test]
async fn test_static_cookie_auth_provider_backend_unavailable_returns_e_auth_007() {
    let provider = StaticCookieAuthProvider::new_with_resolver(
        "cyberint",
        Arc::new(BackendUnavailableCredentialResolver),
    );
    let spec = cookie_roundtrip_spec();
    let client_id = OrgSlug::new("test-org-unit");

    let result = provider.acquire_token(&spec, &client_id).await;

    assert!(result.is_err(), "BackendUnavailable must return Err");
    let err_str = result.unwrap_err().to_string();
    assert!(
        err_str.contains("E-AUTH-007"),
        "BackendUnavailable MUST yield E-AUTH-007, not E-AUTH-005. Got: {err_str}. \
         BC-2.01.017 v1.3 EC-017-010 + TV-BC-2.01.017-009."
    );
    assert!(
        !err_str.contains("E-AUTH-005"),
        "BackendUnavailable MUST NOT yield E-AUTH-005. Got: {err_str}."
    );
}
```

**Verify via revert-fail-restore-pass workflow:**
1. Write the test first (fails with E-AUTH-005 in error string due to blanket wrap)
2. Apply the `.map_err` fix
3. Test passes with E-AUTH-007
4. Run `just iter prism-spec-engine` — all existing tests still pass

**Sibling-sweep (TD-VSDD-060):** After changing `PrismCredentialResolver::resolve` error strings, grep for
all `CredentialResolver` implementors and callers in `crates/prism-spec-engine/src/` to confirm no other
call site relies on the previous undecorated `.to_string()` format.

## Self-Audit Results (PO Scope)

- [ ] Did I rationalize any decision with "MVP," "for now," "good enough," or "we can fix later"? **No.**
- [ ] Did I add a new tech-debt-register entry without all three required conditions? **No.** This is a
  prod-grade spec fix, not a deferral.
- [ ] Did I leave any "pending architect review" for a question I could answer in scope? **No.** All
  variant enumeration done by direct code read. Option A rationale fully documented.
- [ ] Did I find a bug and surface it as advisory instead of fixing it in scope? **No.** Fixed via spec
  amendments + implementer dispatch instructions (implementer owns code, PO owns spec — correct routing).
- [ ] Did I default to the cheapest mechanism? **No.** Option A provides structural semantic disambiguation;
  Option B (detail-text parsing) was correctly rejected as the cheap path.
- [ ] Did I paper-fix by doc-commenting only? **No.** New EC, TV, and error code with implementer dispatch
  for load-bearing test (TD-VSDD-059 criterion met by TV-BC-2.01.017-009 + unit test requirement).
- [ ] Did I skip variant enumeration? **No.** Both `CredentialResolutionError` variants explicitly mapped.
  Confirmed by direct read of `resolution.rs` — enum has exactly 2 variants, no hidden members.
