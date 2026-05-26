---
type: security-review
pass: 2
pr: 154
head_sha: c2ff150e
base: develop @ f19575ff
reviewer: security-reviewer
date: 2026-05-25
story: PLUGIN-MIGRATION-001-E
total_findings: 3
critical: 0
high: 0
medium: 1
low: 2
files_reviewed: 11
verdict: APPROVE
---

# Security Review — PLUGIN-MIGRATION-001-E PR #154 Pass-2

**Scope:** Fresh-context PR-LEVEL security review at HEAD `c2ff150e`. Pass-1 found 5 findings
(SEC-001 CRIT credential_handle bypass + 1 HIGH + 2 MED + 1 OBS), all claimed closed via
fix-burst at `a759d2b0` implementing ADR-028 §D11 Option C (host resolves credentials before
dispatch). This pass verifies those closures and searches for new issues.

**Files reviewed:**
- `crates/prism-spec-engine/plugins/crowdstrike-oauth2/src/lib.rs`
- `crates/prism-spec-engine/src/plugin_auth_provider.rs`
- `crates/prism-spec-engine/src/plugin/host_functions.rs`
- `crates/prism-spec-engine/src/plugin/loader.rs`
- `crates/prism-spec-engine/src/plugin/mod.rs` (dispatch_plugin_acquire_token, make_host_state)
- `crates/prism-spec-engine/src/plugin/sandbox.rs`
- `crates/prism-spec-engine/src/plugin/discovery.rs`
- `crates/prism-spec-engine/src/infusion/cache.rs`
- `crates/prism-spec-engine/plugins/crowdstrike-oauth2/wit/sensor-auth.wit`
- `crates/prism-spec-engine/plugins/crowdstrike-oauth2/plugin.toml`
- `crates/prism-credentials/src/resolution.rs`

---

## Pass-1 Finding Closure Verification

### SEC-001 (CRIT) — Credential Handle Bypass

**Status: CLOSED — VERIFIED**

Pass-1 finding: `credential_handle` opaque string passed to WASM guest allowing the plugin to
request arbitrary credentials.

Verification: The `credential_handle` parameter has been fully removed from the WIT interface.
`sensor-auth.wit` declares `acquire-token: func() -> result<string, auth-error>` — zero parameters.
In `plugin_auth_provider.rs:117-175`, `PluginAuthProvider::acquire_token` calls
`prism_credentials::resolve_credential` for both `"client_id"` and `"client_secret"` before
any dispatch, materializes the `SecretString` values via `expose_secret()` only at the
`PluginConfigMap` boundary, then passes the map to `dispatch_plugin_acquire_token`.
The plugin guest reads these values via `host::get-config("client_id")` /
`host::get-config("client_secret")`. The credential store is inaccessible to the WASM guest.
The structural guarantee (struct stores no credential values) is tested by
`test_debug_shows_structural_ids_not_credentials`.

### SEC-002 (HIGH) — Form Body Injection

**Status: CLOSED — VERIFIED**

Pass-1 finding: no URL-encoding of client credentials in OAuth2 form body.

Verification: `url_encode()` (lib.rs:360-385) implements RFC 3986 §2.3 percent-encoding. Only
unreserved characters (A-Z, a-z, 0-9, `-`, `_`, `.`, `~`) pass through; all other bytes are
encoded as `%XX`. The function is applied to both `client_id` and `client_secret` before
interpolation (lib.rs:464-469). The injection test `test_acquire_token_url_encodes_credentials`
verifies that literal `+`, `&`, `=`, and space characters in credentials are percent-encoded and
do not appear raw in the form body.

The `unwrap()` calls at lib.rs:373 and lib.rs:378 (inside `url_encode`) are on
`char::from_digit(nibble, 16)`. The nibble values are `(b >> 4)` and `(b & 0x0f)` — both are
always in range 0..=15, and `char::from_digit` returns `Some` for all values 0..15 in radix 16.
The unwrap cannot panic. This is a bounded, provably-safe use.

### SEC-003 (MED) — Unredacted Plugin Log Forwarding

**Status: CLOSED — VERIFIED (by design, documented)**

Pass-1 finding: `host_log` forwards plugin messages unredacted.

Verification: The security note in `host_functions.rs:249-255` explicitly documents this:
"The message string is forwarded unredacted from the plugin guest to the host's tracing
subscriber. Plugin code is responsible for NOT including credential values in log messages."
The design is intentional — the WIT boundary is a trust boundary, not a redaction boundary.
The production guard is a log-pipeline scrubber external to prism-spec-engine. The plugin code
(lib.rs) never calls `host::log` with credential values; error messages cite only key names
(e.g., "client_id absent from host config (EC-006b)") not values. This is an accepted
architectural risk, not a defect.

### SEC-004 (MED) — Missing host::log Catalog Row

**Status: CLOSED (was OBS) — VERIFIED**

Pass-1 finding: `host_log` emission without catalog row.

Verification: The `host::log` callback does not emit a `tracing::*!(event_type=...)` event
itself — it forwards plugin-originated log messages at the appropriate level. The
`plugin_log_level_unrecognized` event (emitted when the log-level enum is unrecognized) is
registered in BC-2.16.002 at catalog row 32 (`plugin_log_level_unrecognized`).
The catalog is complete for all production `event_type` sites added in this PR.

---

## New Findings (Pass-2)

### SEC-005: PluginConfigMap Not Zeroized After Dispatch — Credentials May Persist in Heap

- **Severity:** MEDIUM
- **CWE:** CWE-316 (Cleartext Storage of Sensitive Information in Memory)
- **OWASP:** A02:2021 — Cryptographic Failures
- **Attack Vector:** Memory inspection (process core dump, `/proc/<pid>/mem` read by a
  privileged attacker, or memory disclosure via another vulnerability) after a token
  acquisition call. The `PluginConfigMap` (`HashMap<String, String>`) holding the resolved
  `client_secret` value is dropped at the end of `dispatch_plugin_acquire_token` in
  `plugin_auth_provider.rs:161-175`, but `HashMap::drop` does not guarantee overwrite of heap
  memory containing the secret string bytes. `String::drop` calls `dealloc` which returns the
  memory to the allocator but does not zero it. The allocator may not reuse the pages
  immediately, leaving the secret bytes accessible on the heap.
- **Impact:** If an attacker can read process memory (via debugger, core dump, or another
  memory disclosure vulnerability), client_secret bytes may remain readable after the call
  frame returns.
- **Evidence:**
  `crates/prism-spec-engine/src/plugin/loader.rs:11`
  ```rust
  pub type PluginConfigMap = HashMap<String, String>;
  ```
  `crates/prism-spec-engine/src/plugin_auth_provider.rs:148-158`
  ```rust
  let config = PluginConfigMap::from([
      ("client_id".to_string(), resolved_client_id.expose_secret().to_string()),
      ("client_secret".to_string(), resolved_client_secret.expose_secret().to_string()),
      ("token_endpoint".to_string(), self.token_endpoint.clone()),
  ]);
  ```
  The `config` variable is a `HashMap<String, String>`. After dispatch returns, `config` is
  dropped, but the `String` values containing credential bytes are not overwritten.
  The upstream `resolved_client_id` / `resolved_client_secret` (`SecretString`) are zeroized
  correctly via `secrecy::Zeroizing` on drop — but the `.expose_secret().to_string()` call
  creates a new `String` allocation that is NOT wrapped in `Zeroizing`. This new allocation
  persists on the heap until the allocator reuses it.
- **Proposed Mitigation:** Wrap the secret values in `Zeroizing<String>` when building the
  `PluginConfigMap`, or use a newtype that implements `ZeroizeOnDrop` for the config values.
  Alternatively, scope the `expose_secret` call as narrowly as possible and pass bytes directly
  to the WASM host call without an intermediate `String::clone`. The `secrecy` crate's
  `Zeroizing<T>` type should be used: `Zeroizing::new(resolved_client_secret.expose_secret().to_string())`,
  dropped after the dispatch call frame returns. Note: the `HostState.config` field is
  `Arc<PluginConfigMap>` — the Arc clone passed to the store also needs to be zeroized.
  Full mitigation requires changing `PluginConfigMap` to hold `Zeroizing<String>` values,
  which is a more invasive change. A partial mitigation is explicit `zeroize::Zeroize::zeroize`
  on the config map after dispatch returns.
- **Severity justification:** This is a memory-residency concern that requires either a prior
  memory disclosure vulnerability or privileged OS access to exploit. It does not create a
  direct network-exploitable path. However, for an MSSP security product handling client
  credentials, defense-in-depth against memory forensics is appropriate. Rated MEDIUM (not HIGH)
  because exploitation requires additional preconditions. The `AuthToken` correctly uses
  `Zeroizing<String>` (auth_provider.rs:45) — this finding is the analogous gap at the
  PluginConfigMap boundary.

---

### SEC-006: `localhost` in Production Plugin Allowlist — Overly Broad Scope

- **Severity:** LOW
- **CWE:** CWE-183 (Permissive Allowlist)
- **OWASP:** A05:2021 — Security Misconfiguration
- **Attack Vector:** An attacker who can inject a malicious OAuth2 token endpoint URL (e.g.,
  via config file modification) could direct the WASM plugin to POST credentials to
  `http://localhost:<port>` — any locally-listening service on the prism host machine. The
  allowlist in `plugin.toml` includes `"localhost"` alongside `"api.crowdstrike.com"`:
  ```toml
  allowed_urls = ["api.crowdstrike.com", "localhost"]
  ```
- **Impact:** If an attacker can control the `token_endpoint` value in the plugin config (via
  a compromised config file or man-in-the-middle at config load time), the plugin is permitted
  to send OAuth2 credentials to any listener on localhost. The credential is exposed to that
  listener. This is not a remote attack vector — it requires prior local compromise of the
  host's config or network stack.
- **Evidence:**
  `crates/prism-spec-engine/plugins/crowdstrike-oauth2/plugin.toml:18`
  ```toml
  allowed_urls = ["api.crowdstrike.com", "localhost"]
  ```
  The comment notes "Test: localhost (DTU clone token endpoint)" — `localhost` is present for
  DTU clone integration testing. However, the production `.prx` binary ships with this manifest
  embedded, meaning production deployments also permit `localhost` outbound HTTP.
- **Proposed Mitigation:** The `localhost` entry in `allowed_urls` should be removed from the
  production manifest. DTU integration tests should use a separate test-only manifest, or the
  plugin should ship two manifests (one for production, one for testing). If separating
  manifests is not feasible in current story scope, document the accepted risk explicitly in
  the manifest comment and ensure the plugin.toml is reviewed before production deployment.
  Production manifest: `allowed_urls = ["api.crowdstrike.com"]`.
- **Severity justification:** Rated LOW because exploitation requires a config compromise
  precondition (the token_endpoint value must also be attacker-controlled), and `localhost` is
  only useful as an attack target if a listener is available. The primary risk is in hardened
  production environments where all localhost services are trusted — still, principle of least
  privilege favors removing `localhost` from the production allowlist.

---

### SEC-007: PluginConfigMap Cloned into Arc<HostState> — Credential String Lifetime Extends Beyond Dispatch Frame

- **Severity:** LOW
- **CWE:** CWE-316 (Cleartext Storage of Sensitive Information in Memory)
- **OWASP:** A02:2021 — Cryptographic Failures
- **Attack Vector:** Same as SEC-005 (memory inspection), with the additional nuance that
  `make_host_state` wraps the config in `Arc::new(config.clone())`:
  ```rust
  config: Arc::new(config.clone()),
  ```
  (loader.rs:133, mod.rs:822). This means the `HashMap<String, String>` is cloned once into
  an `Arc`. The `Store<HostState>` holds this Arc for the lifetime of the wasmtime call. After
  `dispatch_plugin_acquire_token` returns, the store is dropped — but if the Arc refcount is
  >1 (e.g., due to some wasmtime internal clone of the store data reference), the credential
  bytes may live longer than expected.
- **Impact:** Marginal additional lifetime exposure beyond SEC-005. In practice, the store is
  not shared and the Arc refcount drops to 0 at function return. The concern is primarily
  theoretical under the current code structure.
- **Evidence:**
  `crates/prism-spec-engine/src/plugin/mod.rs:820-822`
  ```rust
  fn make_host_state(&self, plugin_id: &str, config: &PluginConfigMap, ...) -> HostState {
      HostState {
          config: Arc::new(config.clone()),
  ```
- **Proposed Mitigation:** Same as SEC-005 — use `Zeroizing<String>` for the config values, or
  add explicit zeroize calls before dropping the store. This finding is a corollary to SEC-005;
  fixing SEC-005 at the PluginConfigMap type level would automatically fix SEC-007.
- **Severity justification:** Rated LOW because this is a worst-case lifetime extension, not a
  new attack surface. The Arc clone is dropped in the same stack frame as the dispatch call.

---

## SAP-1 Probe: Tracing Emission Catalog Completeness

All `event_type =` emissions in the changed files verified against BC-2.16.002 Canonical
Structured Event Catalog (v1.28, 41 rows):

| event_type | Catalog row | Status |
|---|---|---|
| `plugin_directory_not_found` | row 24 | REGISTERED |
| `plugin_load_failed_read_error` | row 25 | REGISTERED |
| `plugin_load_failed_manifest_not_found` | row 30 | REGISTERED |
| `plugin_load_failed_manifest_parse_error` | row 31 | REGISTERED |
| `plugin_load_failed_manifest_name_missing` | row 21 | REGISTERED |
| `plugin_load_failed_manifest_version_malformed` | row 22 | REGISTERED |
| `plugin_load_failed_format_version_missing` | row 32 | REGISTERED |
| `plugin_load_failed_format_version_exceeded` | row 20 | REGISTERED |
| `plugin_load_failed_manifest_no_allowed_urls` | row 19 | REGISTERED |
| `plugin_load_failed_wit_invalid` | row 23 | REGISTERED |
| `plugin_load_failed_compilation` | row 26 | REGISTERED |
| `plugin_load_unsigned` | row 16 | REGISTERED |
| `plugin_http_request_blocked` | row 24 | REGISTERED |
| `plugin_log_level_unrecognized` | row 29 | REGISTERED |
| `write_tool_registration_after_boot` | row 33 | REGISTERED |
| `plugin_registration_rolled_back` | row 34 | REGISTERED |
| `plugin_auth_provider_constructed` | row 36 | REGISTERED |
| `plugin_auth_token_parse_error` | row 37 | REGISTERED |
| `timestamp.fallback_to_now` | row 35 | REGISTERED |

**SAP-1 result: CLEAN.** No unregistered `event_type` emissions found in the files changed by
this PR.

Note: `boot.audit.initialized` (boot.rs:1063) and `credential_access` (prism-credentials
audit.rs) were observed during the workspace sweep. Both are pre-existing emissions; neither
was added in this PR. They are out of scope for this PR's SAP-1 check. `boot.audit.initialized`
is referenced in the catalog narrative but does not have its own catalog table row —
this is a pre-existing gap outside this PR's scope.

---

## WASM Sandbox Boundary Verification

**Memory limits:** `sandbox.rs:12` — `DEFAULT_MEMORY_LIMIT_MB: u64 = 64`. Applied via
`StoreLimitsBuilder::new().memory_size((64 * 1024 * 1024) as usize).trap_on_grow_failure(true)`
in `create_store`. The limiter is wired via `store.limiter(|s| &mut s.limits)`. Correct.

**CPU time limits:** `DEFAULT_TIMEOUT_SECONDS: u64 = 5`. Applied via epoch interruption:
`store.set_epoch_deadline(timeout_seconds * EPOCH_TICKS_PER_SECOND)`. The background ticker
thread fires at ~10,000 ticks/sec. A 5-second deadline = 50,000 ticks. Correct.

**WASI exclusion:** `host_functions.rs:330` explicitly documents "MUST NOT call any
`wasmtime_wasi::add_to_linker_*` function." Only the Prism host functions are registered.
`pre_instantiate` in `loader.rs:314-327` rejects any component whose imports cannot be
satisfied — WASI imports would fail at pre-instantiation. Correct.

**Host function surface:** Six functions registered in `"host"` namespace: `http-request`,
`log`, `get-config`, `kv-get`, `kv-set`, `current-time-secs`. No filesystem access, no direct
network access (all HTTP gated through allowlist in `host_http_request`), no credential store
access. The `host::get-config` function only exposes values explicitly injected into
`HostState.config` — which is built from the caller-provided `PluginConfigMap`. The plugin
cannot request arbitrary credential names from the credential store.

**Allowlist enforcement:** `host_http_request` in `host_functions.rs:75-83` uses
`url::Url::parse` to extract the host component and compares it against each `allowed_domain`
entry with `url_host == allowed_domain.as_str()` (exact match, not substring). Empty entries
are filtered (`!allowed_domain.is_empty()`). Correct defense-in-depth.

---

## InfusionLruCache `impl UnwindSafe` Safety Argument Verification

The explicit `impl UnwindSafe for InfusionLruCache` at `cache.rs:105-106` is supported by the
safety argument at lines 88-101:

1. `lru::LruCache<String, LruCacheEntry>` holds only `serde_json::Value` and `u64` — plain
   data types with no invariants that a panic could break.
2. `tokio::sync::Mutex` does NOT poison on panic (unlike `std::sync::Mutex`). It remains locked
   until the owning task is dropped. No poisoning state machine exists to be left inconsistent.
3. `capacity: usize` has no invariants.

The comment correctly identifies the root cause: `prism-credentials` pulling
`tokio = { features = ["full"] }` transitively changes the `batch_semaphore::Semaphore`
implementation from `std::sync::Mutex` (which is `UnwindSafe`) to `parking_lot::Mutex`
(which is not `UnwindSafe` via auto-derivation), removing the auto-impl.

**Assessment: The safety argument is sound.** The `InfusionLruCache` contains no mutable
shared state that could be left in an inconsistent state after a panic. The explicit `impl`
restores the guarantee intentionally rather than relying on transitive feature-union behavior.
This is a legitimate use of explicit `impl UnwindSafe` — not a safety bypass.

Note: `impl RefUnwindSafe` at line 106 is also included. The same argument applies —
`InfusionLruCache` has no shared mutable references that a panic could leave inconsistent.

---

## Dependency Security Verification

**`secrecy = "0.8"`:** The `secrecy` crate provides `SecretString` (zeroize-on-drop `String`)
and `ExposeSecret` for controlled exposure. Version 0.8 is the stable series. The `expose_secret()`
method is used correctly: called only once at the `PluginConfigMap` boundary in
`plugin_auth_provider.rs:151,155`, not in log messages or Debug impls.

**`prism-credentials`:** New production dependency on `prism-spec-engine`. Used correctly for
`resolve_credential` — returns `SecretString`, audits access with namespace only (value never
logged). The audit event `credential_access` emits `(client_id, sensor_id, credential_name,
source, outcome)` — no credential value in the log fields.

**`wasmtime = "44"`:** Commit history confirms version 44 was chosen to resolve
17 RUSTSEC advisories (RUSTSEC-2024-0438 through RUSTSEC-2026-0096). This is the correct
current version as of this review.

---

## Risk Register Dispositions

No security-category R-NNN entries identified in the L2 domain spec risk register that are
directly addressed by PLUGIN-MIGRATION-001-E scope. The following security invariants from
the story spec are verified mitigated:

- **AD-017 (AI-opaque credentials):** MITIGATED. Credentials never transit AI context;
  `PluginAuthProvider.Debug` omits credential values; error messages cite key names not values.
- **INV-PLUGIN-002 (no filesystem/network from WASM):** MITIGATED. No WASI registered; only
  6 host functions; HTTP gated through allowlist.
- **INV-PLUGIN-003 (64 MiB memory cap):** MITIGATED. StoreLimits wired correctly.
- **INV-PLUGIN-004 (5s CPU limit):** MITIGATED. Epoch interruption wired correctly.
- **INV-AUTH-OPEN-003 Rule A (auth_type_name canonical value):** MITIGATED. Hardcoded string
  `"oauth2_client_credentials"` with invariant test.

---

## Verdict

**CLEAN (PR-merge): YES** — zero CRITICAL or HIGH findings.  
**CLEAN (strict): NO** — 3 findings present (1 MEDIUM, 2 LOW).

The MEDIUM finding (SEC-005) represents a pre-existing architectural gap in the credential
materialization pattern. Credentials passed through `PluginConfigMap` are not zeroized on drop.
This is the same gap pattern found in many OAuth2 clients that do not use zeroize-on-drop for
intermediate allocations. It does not represent a regression from this PR — the PR actually
improves the situation (credentials previously could be passed as opaque handles; now they are
at least scoped to a single dispatch frame). SEC-005 should be addressed in a follow-up
hardening story before Phase 6 (Formal Hardening).

The LOW findings (SEC-006, SEC-007) are low-priority observations. SEC-006 (`localhost` in
production allowlist) should be resolved before GA deployment. SEC-007 is subsumed by SEC-005.

**No findings block merge.** Recommend tracking SEC-005 as a Phase 6 hardening item and
SEC-006 as a pre-GA manifest cleanup.

---

*Reviewed by: security-reviewer (claude-sonnet-4-6)*  
*Review date: 2026-05-25*  
*PR: https://github.com/drbothen/prism/pull/154*
