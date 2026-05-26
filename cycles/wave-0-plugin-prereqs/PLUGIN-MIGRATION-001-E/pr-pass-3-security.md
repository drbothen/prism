---
type: security-review
pass: 3
pr: 154
head_sha: 6238d5b1
base: develop @ f19575ff
reviewer: security-reviewer
date: 2026-05-25
story: PLUGIN-MIGRATION-001-E
total_findings: 1
critical: 0
high: 0
medium: 0
low: 1
files_reviewed: 4
verdict: APPROVE
---

# Security Review — PLUGIN-MIGRATION-001-E PR #154 Pass-3

**Scope:** Fresh-context PR-LEVEL security review at HEAD `6238d5b1`. Pass-2 found 3 findings
(SEC-005 MED credential residency + SEC-006 LOW localhost allowlist + SEC-007 LOW Arc lifetime),
all claimed closed in fix-burst at `6238d5b1`. This pass verifies those closures and performs
a complete fresh review of the changed files.

**Files reviewed (4 changed in fix-burst):**
- `crates/prism-spec-engine/src/plugin_auth_provider.rs` (SEC-005 zeroize fix)
- `crates/prism-spec-engine/plugins/crowdstrike-oauth2/plugin.toml` (SEC-006 localhost removal)
- `crates/prism-spec-engine/plugins/crowdstrike-oauth2/src/lib.rs` (PR-WRN-2 unwrap hygiene)
- `crates/prism-bin/tests/plugin_boot_tests.rs` (credential_handle → sensor_id rename)

---

## Pass-2 Finding Closure Verification

### SEC-005 (MED) — PluginConfigMap Not Zeroized After Dispatch

**Status: PARTIALLY CLOSED — RESIDUAL LOW FINDING (see SEC-008 below)**

Pass-2 finding: `.expose_secret().to_string()` creates a `String` heap allocation that is NOT
wrapped in `Zeroizing<T>`, leaving credential bytes on the heap after `config` drops.

**What the fix does:** `plugin_auth_provider.rs` now holds `mut config` and calls
`zeroize::Zeroize::zeroize()` on the `"client_id"` and `"client_secret"` entries in the outer
`HashMap` after `dispatch_plugin_acquire_token` returns and before the `?` operator fires:

```rust
if let Some(v) = config.get_mut("client_id") { v.zeroize(); }
if let Some(v) = config.get_mut("client_secret") { v.zeroize(); }
Ok(AuthToken::new(dispatch_result?))
```

This is a meaningful improvement: the outer `config` allocation in `plugin_auth_provider.rs`
is now deterministically zeroed before `config` drops. The comment accurately documents the
CWE-316 threat model and the choice of explicit `zeroize()` over `Zeroizing<T>`.

**Residual gap (SEC-008):** The fix zeroizes the outer copy only. Inside
`dispatch_plugin_acquire_token`, `make_host_state` calls `Arc::new(config.clone())` — creating
a second heap copy of the credential `String` values. This copy lives in `HostState.config`
inside the wasmtime `Store`. The `Store` is dropped at the end of `dispatch_plugin_acquire_token`
before the outer zeroize calls execute. However, `HashMap::drop` → `String::drop` → `dealloc`
does NOT zero the heap pages. The allocator returns the memory but the credential bytes remain
readable until reuse.

The commit message claims "SEC-007: subsumed by SEC-005" — this claim is accurate for the
architectural intention (changing `PluginConfigMap` to use `Zeroizing<String>` values would
fix both) but is inaccurate for the chosen implementation (explicit `get_mut().zeroize()` only
reaches the outer map, not the clone). The cloned copy in `HostState.config` is unzeroized.

This is a residual LOW severity finding (not a regression from pass-2 — it was captured as
SEC-007 previously). The attack requires memory inspection of a deallocated-but-not-reused heap
region, which requires either OS-level privilege or a prior memory disclosure vulnerability.
Rated LOW for the same reasons as SEC-007 in pass-2.

---

### SEC-006 (LOW) — `localhost` in Production Plugin Allowlist

**Status: CLOSED — VERIFIED**

`plugin.toml` now reads:
```toml
allowed_urls = ["api.crowdstrike.com"]
```

`localhost` has been removed. The comment documents the rationale:
```
# SEC-006 (CWE-183): localhost removed from production allowlist — principle of least privilege.
# DTU integration tests use a separate in-memory manifest (SENSOR_AUTH_MANIFEST constant in
# plugin_boot_tests.rs) that includes localhost for the DTU clone token endpoint.
```

Verification: `plugin_boot_tests.rs:1382` confirms the separate test-only in-memory constant:
```rust
allowed_urls = ["api.crowdstrike.com", "localhost"]
```
This is correct: the test manifest is an in-memory TOML string compiled into the test binary
and is NOT the production `plugin.toml`. The shipped `.prx` binary embeds only
`["api.crowdstrike.com"]`.

---

### SEC-007 (LOW) — Arc Lifetime Extension of Credential String

**Status: PARTIALLY CLOSED — See SEC-008 (residual from SEC-005 partial fix)**

As documented above, the outer `PluginConfigMap` is now zeroized. The `Arc<PluginConfigMap>`
clone in `HostState` is dropped when `dispatch_plugin_acquire_token` returns (before the outer
zeroize runs), but the cloned credential bytes are not zeroed. The residual finding is captured
as SEC-008 below. No new attack surface was added by the fix.

---

## New Findings (Pass-3)

### SEC-008: Cloned Credential Bytes in `HostState.config` Not Zeroized — Residual from SEC-005/SEC-007 Partial Fix

- **Severity:** LOW
- **CWE:** CWE-316 (Cleartext Storage of Sensitive Information in Memory)
- **OWASP:** A02:2021 — Cryptographic Failures
- **Attack Vector:** Same memory inspection vector as SEC-005/SEC-007 (core dump, debugger,
  `/proc/<pid>/mem` read, or a prior memory disclosure vulnerability). The specific location is
  the heap region previously holding `HostState.config` — the `Arc<PluginConfigMap>` clone
  created by `make_host_state`. This clone is dropped (not zeroed) at the end of
  `dispatch_plugin_acquire_token`. The allocator reclaims the pages but the credential bytes
  are not overwritten.
- **Impact:** If an attacker can read process memory, `client_secret` bytes from the clone
  that lived in `HostState.config` may remain readable until the allocator reuses the pages.
  Same exploitability preconditions as SEC-005: requires OS-level privilege or a prior
  vulnerability chain. Does not create a network-exploitable path.
- **Evidence:**
  `crates/prism-spec-engine/src/plugin/mod.rs` — `make_host_state`:
  ```rust
  fn make_host_state(&self, plugin_id: &str, config: &PluginConfigMap, ...) -> HostState {
      HostState {
          config: Arc::new(config.clone()),  // <-- clone is NOT zeroized on drop
          ...
      }
  }
  ```
  `crates/prism-spec-engine/src/plugin_auth_provider.rs` — outer zeroize:
  ```rust
  // These zeroize calls only reach the outer HashMap, not the Arc<PluginConfigMap> clone.
  if let Some(v) = config.get_mut("client_id") { v.zeroize(); }
  if let Some(v) = config.get_mut("client_secret") { v.zeroize(); }
  ```
  The clone is dropped when `dispatch_plugin_acquire_token` returns (before these calls run),
  but `String::drop` only calls `dealloc` — it does not zero the heap bytes.
- **Proposed Mitigation:** The complete fix is to change `PluginConfigMap` from
  `HashMap<String, String>` to a wrapper type that zeroizes its credential values on drop.
  Two viable approaches:
  1. Define a `SecretConfigMap` wrapper holding `HashMap<String, secrecy::Zeroizing<String>>`,
     which auto-zeroizes all values on drop regardless of which copy drops first.
  2. Alternatively, add a zeroize call inside `dispatch_plugin_acquire_token` on the
     `host_state.config` entries before the Store drops — but this requires `Arc::try_unwrap`
     or `Arc::make_mut` which may clone again if the refcount is >1.
  Option 1 is cleanest and fixes SEC-005 + SEC-008 at the type level. This was the "more
  invasive change" alluded to in SEC-005's pass-2 report.
- **Severity justification:** Rated LOW (not MEDIUM) because: (a) this is a residual from the
  previously accepted SEC-007 LOW finding; (b) the pass-2 fix substantially reduced the exposure
  window (the outer copy IS now zeroed; only the clone remains); (c) exploitation requires
  OS-level access or a prior memory disclosure vulnerability. The pass-2 rating of LOW for
  SEC-007 carries forward for this residual.
- **Is this a regression from pass-2?** No. SEC-007 was already present in pass-2 and rated
  LOW. The fix partially addressed it by zeroing the outer copy. SEC-008 is the remaining
  gap from the same root cause, reclassified as a distinct finding to avoid confusion with
  the now-verified closure of the outer copy.

---

## PR-WRN-2: `unwrap()` → `unwrap_or('?')` in `url_encode()`

**Status: CLOSED — VERIFIED**

`lib.rs` now uses `.unwrap_or('?')` on both `char::from_digit` calls:
```rust
char::from_digit((b >> 4) as u32, 16).unwrap_or('?').to_ascii_uppercase()
char::from_digit((b & 0x0f) as u32, 16).unwrap_or('?').to_ascii_uppercase()
```

This is mathematically infallible (nibble values are always 0..=15, well within radix 16's
valid range) — the `'?'` branch is unreachable. The change removes the `unwrap()` pattern
for lint hygiene. The security analysis from pass-2 is unchanged: this cannot panic in practice,
and the use of `'?'` as a fallback is sensible for "logically impossible" paths. No security
concern.

---

## SAP-1 Probe: Tracing Emission Catalog Completeness

No new `event_type =` emissions were added in the fix-burst commit (`6238d5b1`). The four
changed files introduce only code-level changes (zeroize calls, manifest edit, unwrap_or,
test rename) — no new tracing instrumentation.

SAP-1 scope limited to files changed in this pass's fix-burst. The full workspace SAP-1 was
performed in pass-2 and found CLEAN (all 19 event_type values registered).

Additional observation from workspace sweep: `plugin_load_disabled_via_envvar` in
`boot.rs:1163` is registered in BC-2.16.002 (confirmed). Pre-existing emission, not added
in this PR. No gap.

**SAP-1 result: CLEAN (no new emissions in pass-3 fix-burst).**

---

## WASM Sandbox Boundary Verification (Re-check)

No changes to `sandbox.rs`, `host_functions.rs`, `loader.rs`, or `mod.rs` dispatch logic
in the fix-burst. The pass-2 sandbox verification (memory limits 64 MiB via `StoreLimitsBuilder`,
CPU time 5s via epoch interruption, WASI exclusion, 6-function host surface) remains valid.

The allowlist enforcement in `host_http_request` is unchanged — exact host string comparison
via `url::Url::parse`, empty-entry filter, default-deny semantics. No regression.

---

## Credential Flow Verification

**Host-resolves-before-dispatch path (ADR-028 §D11 Option C):** unchanged and verified.
`PluginAuthProvider::acquire_token` resolves both credentials from `prism_credentials` before
calling `dispatch_plugin_acquire_token`. The WASM guest has zero access to the credential
store — `host::get-config` only returns values explicitly placed in `HostState.config` by
the caller. The `acquire-token` WIT export takes zero parameters (confirmed in pass-1/pass-2);
no credential handle is passed across the WIT boundary.

---

## Risk Register Dispositions

Security-category invariants verified (unchanged from pass-2):

- **AD-017 (AI-opaque credentials):** MITIGATED. `PluginAuthProvider.Debug` omits credential
  values (struct stores no credential data). `resolve_credential` audit events cite key names
  only, never values.
- **INV-PLUGIN-002 (no filesystem/network from WASM):** MITIGATED. No WASI registered.
- **INV-PLUGIN-003 (64 MiB memory cap):** MITIGATED. StoreLimits wired correctly.
- **INV-PLUGIN-004 (5s CPU limit):** MITIGATED. Epoch interruption wired correctly.
- **INV-AUTH-OPEN-003 Rule A (auth_type_name canonical value):** MITIGATED.

---

## Verdict

**CLEAN (strict): NO** — 1 finding present (SEC-008 LOW, residual from SEC-005/SEC-007 partial fix).

**CLEAN (PR-merge): YES** — zero CRITICAL, HIGH, or MEDIUM findings.

SEC-005 (MED) is confirmed CLOSED: the outer `PluginConfigMap` credential entries are
explicitly zeroized before drop. This is a genuine security improvement over pass-2.

SEC-006 (LOW) is confirmed CLOSED: `localhost` is removed from the production `plugin.toml`
allowlist. DTU tests correctly use a separate in-memory manifest.

SEC-007 (LOW) is PARTIALLY CLOSED: the outer copy fix covers the primary residency concern.
SEC-008 (LOW) captures the residual cloned-copy gap that remains.

SEC-008 does NOT block merge. The fix-burst made a meaningful improvement; the remaining
gap (the clone in `HostState.config`) was already present and accepted as LOW in pass-2
(as SEC-007). The recommended full fix — using `Zeroizing<String>` values throughout
`PluginConfigMap` — should be tracked as a Phase 6 hardening item alongside the original
SEC-005 note from pass-2.

**No findings block merge. APPROVE with one tracked LOW.**

---

*Reviewed by: security-reviewer (claude-sonnet-4-6)*
*Review date: 2026-05-25*
*PR: https://github.com/drbothen/prism/pull/154*
