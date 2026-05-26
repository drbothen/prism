---
type: security-review
pass: 4
pr: 154
head_sha: 63f95759
base: develop @ f19575ff
reviewer: security-reviewer
date: 2026-05-25
story: PLUGIN-MIGRATION-001-E
total_findings: 0
critical: 0
high: 0
medium: 0
low: 0
files_reviewed: 4
verdict: APPROVE
---

# Security Review — PLUGIN-MIGRATION-001-E PR #154 Pass-4

**Scope:** Fresh-context PR-LEVEL security review at HEAD `63f95759`. Pass-3 found SEC-008 LOW
(cloned credential bytes in `HostState.config` not zeroized). SEC-008 was fixed in `63f95759`
by changing `PluginConfigMap` from `HashMap<String, String>` to `HashMap<String, SecretString>`.
This pass verifies that closure and performs a complete fresh security review.

**Files reviewed (4 changed in fix commit `63f95759`):**
- `crates/prism-spec-engine/src/plugin/loader.rs` — `PluginConfigMap` type alias changed
- `crates/prism-spec-engine/src/plugin/host_functions.rs` — `host_get_config` updated
- `crates/prism-spec-engine/src/plugin_auth_provider.rs` — `PluginConfigMap` insertion updated
- `crates/prism-spec-engine/tests/crowdstrike_oauth2_plugin_tests.rs` — test sentinel values updated

---

## SEC-008 Closure Verification

### Finding: SEC-008 (LOW) — Cloned Credential Bytes in `HostState.config` Not Zeroized

**Status: CLOSED — VERIFIED**

**Root cause (pass-3):** `PluginConfigMap = HashMap<String, String>`. When `make_host_state`
called `Arc::new(config.clone())`, it created a second heap copy of the credential `String`
values in `HostState.config`. The caller-side `zeroize()` calls in `plugin_auth_provider.rs`
only reached the outer `PluginConfigMap`, not this cloned copy. `String::drop` calls
`dealloc` but does not zero the heap pages.

**Fix applied (`63f95759`):** `PluginConfigMap` type alias changed to
`HashMap<String, SecretString>` where `SecretString = Secret<String>` (secrecy 0.8.0).

**Verification of fix correctness — secrecy 0.8.0 source analysis:**

The secrecy 0.8.0 source (confirmed at
`~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/secrecy-0.8.0/`) establishes:

1. `SecretString = Secret<String>` (src/string.rs line 9)
2. `impl CloneableSecret for String {}` — String is `Clone + Zeroize` (src/string.rs line 12)
3. `impl<S> Drop for Secret<S> where S: Zeroize` calls `self.inner_secret.zeroize()` (src/lib.rs lines 174-182)
4. `impl<S> Clone for Secret<S> where S: CloneableSecret` produces a new `Secret<String>` with
   its own independent `Drop` impl that will call `String::zeroize()` on the cloned inner string
   (src/lib.rs lines 152-161)
5. `impl DebugSecret for String {}` — `Debug` outputs `Secret([REDACTED alloc::string::String])`
   never the plaintext value (src/lib.rs lines 163-172, src/string.rs line 11)

**Consequence:** When `make_host_state` calls `Arc::new(config.clone())` at
`crates/prism-spec-engine/src/plugin/mod.rs:822`, the clone produces a new
`HashMap<String, SecretString>` where each `SecretString` is an independent `Secret<String>`
instance. When `HostState` drops (at the end of `dispatch_plugin_acquire_token`), the `Arc`
refcount decrements to zero, the `PluginConfigMap` drops, each `SecretString` drops, and each
`Secret<String>::drop` calls `String::zeroize()` — overwriting the heap bytes with zeros before
`dealloc`. This is correct-by-construction: no credential bytes from the cloned copy remain
unzeroized after the Store drops.

**Evidence confirming `PluginConfigMap` now uses `SecretString`:**

```rust
// crates/prism-spec-engine/src/plugin/loader.rs (current HEAD)
pub type PluginConfigMap = HashMap<String, SecretString>;
```

**Evidence confirming `host_get_config` uses `expose_secret()` correctly:**

```rust
// crates/prism-spec-engine/src/plugin/host_functions.rs
pub fn host_get_config(state: &HostState, key: &str) -> Option<String> {
    use secrecy::ExposeSecret;
    state.config.get(key).map(|s| s.expose_secret().to_owned())
}
```

The doc comment reads: "We call `.expose_secret()` here — at the last possible moment before
handing the value to the WASM guest — so the plaintext `String` exists only for the duration
of this call frame." This is correct. The returned `String` is an ephemeral copy owned by
the WIT dispatch closure; it is dropped after being copied into the WIT `Val::String` result
slot. This is an accepted and unavoidable residual of passing the secret to the WASM guest
(the guest must receive the plaintext to make the HTTP call). The window is bounded to the
function call frame — the minimum possible exposure.

**Evidence confirming `plugin_auth_provider.rs` builds `PluginConfigMap` with `SecretString`:**

```rust
// crates/prism-spec-engine/src/plugin_auth_provider.rs
use secrecy::{ExposeSecret, SecretString};
let config = PluginConfigMap::from([
    (
        "client_id".to_string(),
        SecretString::new(resolved_client_id.expose_secret().to_owned()),
    ),
    (
        "client_secret".to_string(),
        SecretString::new(resolved_client_secret.expose_secret().to_owned()),
    ),
    (
        "token_endpoint".to_string(),
        SecretString::new(self.token_endpoint.clone()),
    ),
]);
```

The intermediate bare `String` from `.expose_secret().to_owned()` is immediately consumed
by `SecretString::new(...)` — it is not a named binding and the compiler constructs the
`Secret<String>` without an additional heap allocation. The three explicit `zeroize()` calls
from pass-2 have been removed (they are no longer needed since `SecretString` handles
zeroization unconditionally for every copy).

**AD-017 compliance check:** `PluginAuthProvider::Debug` was verified in pass-3 and is
unchanged. The struct does NOT store credential values — `client_id` and `client_secret`
are resolved at dispatch time and live only in the `PluginConfigMap` local binding. The
`Debug` impl explicitly documents: "credential values never in Debug output (AD-017)."

**SEC-008 conclusion:** CLOSED. The fix is correct-by-construction. Every copy of the
`PluginConfigMap` — the caller's local binding AND the `Arc::new(config.clone())` in
`HostState.config` — will zeroize its credential bytes on drop via `SecretString`'s
`Drop` impl.

---

## Prior Finding Status Summary

| Finding | Severity | Status |
|---------|----------|--------|
| SEC-005 (MED) — PluginConfigMap not zeroized after dispatch | MEDIUM | CLOSED (pass-3 + pass-4) |
| SEC-006 (LOW) — localhost in production allowlist | LOW | CLOSED (pass-3) |
| SEC-007 (LOW) — Arc lifetime extension of credential String | LOW | CLOSED (pass-4, subsumed by SEC-008 fix) |
| SEC-008 (LOW) — Cloned credential bytes in HostState.config not zeroized | LOW | CLOSED (pass-4) |

---

## Fresh Review — Security Areas Examined

### OWASP Top 10 / CWE Coverage

**A02:2021 — Cryptographic Failures (CWE-316):** SEC-008 closed above. No remaining
instances of `HashMap<String, String>` in production credential paths. All `PluginConfigMap`
instances use `SecretString` values. Verified across all files in the SEC-008 fix commit.

**A03:2021 — Injection (CWE-78/CWE-89):** The WASM sandbox boundary is unchanged. Plugin
outbound HTTP is still validated via `host_http_request` allowlist enforcement (exact host
comparison, no substring matching). No new injection surfaces introduced.

**A05:2021 — Security Misconfiguration (CWE-16):** `token_endpoint` is now also wrapped in
`SecretString`, which is conservative and correct — the token endpoint URL is not strictly
secret but wrapping it costs nothing and ensures the map type is uniform. No misconfiguration
introduced.

**A07:2021 — Identification and Authentication Failures (CWE-287):** The credential
resolution path (`prism_credentials::resolve_credential`) is unchanged. The `PluginAuthProvider`
correctly resolves both `client_id` and `client_secret` from `prism_credentials` before
dispatch. No authentication bypass introduced.

### Information Disclosure via Debug/Display

`SecretString`'s `Debug` impl outputs `Secret([REDACTED alloc::string::String])` — never
the plaintext value (verified against secrecy 0.8.0 source). `HostState` does not derive
`Debug`. `PluginAuthProvider::Debug` is a hand-written impl that omits all credential
fields. No information disclosure path exists through Debug/Display.

### Test Sentinel Values

The test file update wraps sentinel values `"id"` and `"secret"` in `SecretString::new()`.
These are test-only values, not production credentials. The update is correct and necessary
for the type change to compile.

### Dependency Security

secrecy 0.8.0 (checksum `9bd1c54ea06cfd2f6b63219704de0b9b4f72dcc2b8fdef820be6cd799780e91e`),
zeroize 1.8.2 (checksum `b97154e67e32c85465826e8bcc1c59429aaaf107c1e4a9e53c8d8ccd5eff88d0`).
No new dependencies introduced by this fix commit — both were already present in the
workspace. No CVEs against secrecy 0.8.0 or zeroize 1.8.2 are known as of this review date.

---

## SAP-1 Probe: Tracing Emission Catalog Completeness

No new `event_type =` emissions were added in the SEC-008 fix commit (`63f95759`). The
four changed files introduce only type-level changes (`SecretString` wrappers, `expose_secret()`
calls, explicit `zeroize()` removal) — no new tracing instrumentation.

**SAP-1 result: CLEAN (no new emissions in pass-4 fix commit).**

---

## WASM Sandbox Boundary Verification

No changes to `sandbox.rs`, `host_functions.rs` allowlist logic, `loader.rs` HostState
structure (other than the `config` field type), or `mod.rs` dispatch logic in the fix commit.
Pass-3 sandbox verification remains valid:
- Memory limit: 64 MiB via `StoreLimitsBuilder` (INV-PLUGIN-003)
- CPU time: 5s via epoch interruption (INV-PLUGIN-004)
- WASI: not registered (INV-PLUGIN-002)
- Host surface: 6 functions (`http-request`, `log`, `get-config`, `kv-get`, `kv-set`, `current-time-secs`)
- Allowlist enforcement: exact host comparison via `url::Url::parse`

No regression.

---

## Risk Register Dispositions

Security-category invariants verified (unchanged from pass-3):

- **AD-017 (AI-opaque credentials):** MITIGATED. `PluginAuthProvider.Debug` omits credential
  values (struct stores no credential data). `SecretString.Debug` outputs `[REDACTED ...]`.
  Credential values never transit AI context.
- **INV-PLUGIN-002 (no filesystem/network from WASM without host intermediation):** MITIGATED.
  No WASI registered. Outbound HTTP gated through allowlist.
- **INV-PLUGIN-003 (64 MiB memory cap):** MITIGATED. StoreLimits wired correctly, unchanged.
- **INV-PLUGIN-004 (5s CPU limit):** MITIGATED. Epoch interruption wired correctly, unchanged.
- **INV-AUTH-OPEN-003 Rule A (auth_type_name canonical value):** MITIGATED. Unchanged.

---

## Verdict

**CLEAN (strict): YES** — zero findings of any severity.

**CLEAN (PR-merge): YES** — zero CRITICAL, HIGH, MEDIUM, LOW, OBS, or PROCESS-GAP findings.

The SEC-008 fix is correct-by-construction. The type-level approach (`SecretString` values
throughout `PluginConfigMap`) is architecturally superior to the pass-2 explicit-zeroize
approach: it eliminates the partial-coverage gap (caller-only zeroing vs. all-copy zeroing)
and removes the manual bookkeeping burden. Every copy of the credential map — regardless of
how many `Arc::clone` calls occur — will zeroize its backing heap pages on drop.

No new findings. All prior findings confirmed closed. **APPROVE.**

---

*Reviewed by: security-reviewer (claude-sonnet-4-6)*
*Review date: 2026-05-25*
*PR: https://github.com/drbothen/prism/pull/154*
