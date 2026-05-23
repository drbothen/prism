---
document_type: story
story_id: PLUGIN-MIGRATION-001-E
title: "prism-spec-engine: CrowdStrike OAuth2 Refresh-on-401 as In-Repo .prx WASM Plugin"
wave: 1
epic_id: PLUGIN-MIGRATION-001
priority: P0
status: ready
version: "v1.2"
level: "L4"
producer: story-writer
timestamp: "2026-05-22T00:00:00Z"
modified: "2026-05-22"
tdd_mode: strict
subsystems: [SS-01, SS-16, SS-17]
# Subsystem anchor justifications:
#   SS-01 (Sensor Adapters, prism-sensors) owns CrowdStrikeAuth and SensorAuth — the open
#   trait that the .prx plugin implements. The plugin replaces CrowdStrikeAdapter's OAuth2
#   logic; SS-01 is the domain being migrated.
#   SS-16 (Spec Engine, prism-spec-engine) owns the SensorSpec parsing, auth_plugin field
#   dispatch, and the crowdstrike.sensor.toml amendment that declares the plugin reference.
#   SS-17 (WASM Plugin Runtime, prism-spec-engine plugin/) owns the PluginRuntime loader
#   and sandboxed invocation path that loads the .prx plugin built in this story.
crates_touched: [prism-spec-engine, prism-sensors]
# Note: prism-sensors is touched only for the crowdstrike.sensor.toml amendment
# (adding auth_plugin field). The .prx plugin Wasm binary and its Rust source live
# in crates/prism-spec-engine/plugins/crowdstrike-oauth2/ (in-repo plugin convention).
target_module: prism-spec-engine
capabilities: [CAP-001, CAP-029]
behavioral_contracts:
  - BC-2.01.016  # SensorAuth Open Trait — the .prx plugin implements SensorAuth externally;
                 #   post-PREREQ-E the trait is unsealed, enabling plugin-provided implementations.
                 #   auth_type_name() on the plugin implementation MUST return "oauth2_client_credentials"
                 #   matching the crowdstrike.sensor.toml auth_type value (INV-AUTH-OPEN-003 Rule A)
  - BC-2.01.013  # DataSource Trait — confirms that after this story merges, CrowdStrike's
                 #   data-fetch adapter is driven by spec + plugin, not handwritten Rust;
                 #   this BC's postcondition (no per-sensor Rust code) is progressed here
  - BC-2.16.013  # Bundled Sensor Spec Authoring and DTU-Parity Verification — crowdstrike.sensor.toml
                 #   is amended with auth_plugin field; parity test VP-148 must remain GREEN after
                 #   the amendment (plugin path produces byte-identical OCSF output to fixture)
  - BC-2.17.001  # Plugin Panic Isolation — .prx plugin panics are caught by Wasmtime sandbox;
                 #   must not propagate to the host PipelineExecutor
  - BC-2.17.006  # Plugin WIT Validation — .prx plugin must export the SensorAuth WIT interface
                 #   and pass WIT validation at load time (PluginRuntime::load_plugin gate)
  - BC-2.17.007  # Plugin Manifest Schema Validation — plugin.toml manifest must declare
                 #   format_version, plugin_id, plugin_type, allowed_urls, and pass
                 #   PluginRuntime manifest schema gate before the .wasm binary is loaded
  - BC-2.22.001  # Boot Orchestration — PluginRuntime loads .prx plugins at boot step 7.5;
                 #   crowdstrike-oauth2 plugin is discovered and loaded as part of the boot sequence
# BC STATUS NOTE: All BCs above are active (promoted at their respective story merges:
#   BC-2.01.016 + BC-2.17.001/006/007 + BC-2.22.001 active since PREREQ-E/PREREQ-D merges;
#   BC-2.01.013 + BC-2.16.013 active since PLUGIN-MIGRATION-001-D PR #153 merge 2026-05-22T09:05:47Z).
verification_properties:
  - VP-148   # VP-PLUGIN-003: DTU parity — TOML+plugin path output must match fixture reference
             #   after this story. Plugin's OAuth2 flow (token acquisition + caching) must produce
             #   the same OCSF records as the fixture JSON created against the DTU clone.
  - VP-150   # VP-PLUGIN-005: OAuth2 refresh-on-401 via declarative TOML retry policy — already
             #   implemented in PipelineExecutor::issue_request_with_retry (AC-5 of PREREQ-B);
             #   this story exercises that path end-to-end with the plugin-provided auth token.
             #   VP-150 is anchored to S-PLUGIN-PREREQ-B; this story adds the e2e integration test.
depends_on:
  - S-PLUGIN-PREREQ-D  # PluginRuntime boot wiring: .prx loader, WIT interface, sandbox established
  - S-PLUGIN-PREREQ-E  # SensorAuth unsealed: plugin can implement SensorAuth without sealed barrier
  - PLUGIN-MIGRATION-001-D  # crowdstrike.sensor.toml exists at crates/prism-sensors/specs/;
                             # 001-D authored the TOML file this story amends with auth_plugin field
blocks:
  - PLUGIN-MIGRATION-001-A  # AC-006 in 001-A is GATED on this story: 001-A must NOT delete
                             # crates/prism-sensors/src/auth/crowdstrike.rs until this story
                             # has shipped the .prx plugin replacement (ADR-028 §D10 co-merge gate)
points: 3
# Points justification:
#   - Rust source for .prx plugin (token acquisition + caching logic, SensorAuth impl): ~1 day
#     The algorithm already exists in CrowdStrikeAdapter::acquire_token + get_valid_token;
#     this is translation + WASM compilation, not algorithm design (gene-transfusion reduction)
#   - plugin.toml manifest + WIT binding + allowed_urls declaration: ~0.5 day
#   - crowdstrike.sensor.toml amendment (auth_plugin field): ~0.25 day
#   - Red Gate tests + end-to-end parity test against DTU clone: ~0.75 day
#   Total: 3 points (~1.5–2 days). Below the 13-point cap.
estimated_days: 2
risk: MEDIUM
# Risk justification: The plugin runtime (PluginRuntime) is established by PREREQ-D.
# The primary risk is WIT interface conformance — the plugin must correctly export the
# SensorAuth-equivalent WIT interface and survive WIT validation at load time. Secondary
# risk: credential handling in WASM context requires careful SecretString equivalents.
# Credential values must NOT transit AI context (AD-017) or appear in guest-memory logs.
acceptance_criteria_count: 11
red_gate_tests: 9
estimated_passes: "3-5 LOCAL adversary passes"
holdout_scenarios: []
assumption_validations:
  - "PluginRuntime::load_plugin() validates WIT interface export presence before .wasm execution"
  - "host_http_request() in WASM host enforces allowed_urls allowlist (AC-007 of PREREQ-D)"
  - "Token is stored in PluginKvStore (scoped to plugin_id) rather than in guest WASM memory"
  - "PipelineExecutor::issue_request_with_retry() handles the 401-retry using whatever token"
    the AuthProvider returns — plugin path is transparent to the retry mechanic"
risk_mitigations:
  - "ADR-028 §D10 co-merge gate: 001-A deletion of crowdstrike.rs is BLOCKED until this story
    merges. The depends_on in PLUGIN-MIGRATION-001-A frontmatter encodes the gate."
  - "WIT validation at load: PluginRuntime::load_plugin() calls WIT validation (AC-14 of PREREQ-D /
    BC-2.17.006); the plugin binary is rejected before execution if the interface is wrong."
  - "Credential opaqueness: client_secret credential resolved via CredentialRef at host layer;
    the WASM plugin never receives the raw secret — it receives an opaque token handle that the
    host resolves to the actual value before injecting into the POST /oauth2/token form body."
  - "VP-148 parity regression: after the TOML amendment, existing DTU-parity tests (authored in
    001-D) MUST remain green — the plugin path must produce byte-identical OCSF output to fixture."
inputs:
  - "crates/prism-sensors/src/auth/crowdstrike.rs"
  - "crates/prism-sensors/specs/crowdstrike.sensor.toml"
  - "crates/prism-dtu-crowdstrike/src/routes/oauth.rs"
  - "crates/prism-dtu-crowdstrike/src/routes/mod.rs"
  - ".factory/specs/behavioral-contracts/BC-2.01.016-sensor-auth-open-trait-contract.md"
  - ".factory/specs/behavioral-contracts/BC-2.16.013-bundled-sensor-spec-dtu-parity.md"
  - ".factory/specs/behavioral-contracts/BC-2.17.001-plugin-panic-isolation.md"
  - ".factory/specs/behavioral-contracts/BC-2.17.006-plugin-wit-validation.md"
  - ".factory/specs/behavioral-contracts/BC-2.17.007-plugin-manifest-schema-validation.md"
  - ".factory/specs/behavioral-contracts/BC-2.22.001-boot-orchestration.md"
  - ".factory/specs/architecture/decisions/ADR-028-toml-spec-grounding-vs-dtu-routes.md"
  - ".factory/stories/S-PLUGIN-PREREQ-D-plugin-runtime-boot-wiring.md"
  - ".factory/stories/S-PLUGIN-PREREQ-E-unseal-sensor-auth-deprecate-customadapter.md"
  - ".factory/stories/PLUGIN-MIGRATION-001-A-delete-4-named-auth-modules-and-replace-init-registry-for-org.md"
input-hash: null
traces_to: []
cycle: "v1.0.0-greenfield"
phase: 3
---

# PLUGIN-MIGRATION-001-E: CrowdStrike OAuth2 Refresh-on-401 as In-Repo .prx WASM Plugin

**Story ID:** PLUGIN-MIGRATION-001-E
**Status:** ready
**Version:** v1.2
**Wave:** 1 (blocked on PLUGIN-MIGRATION-001-D; D merged via PR #153 2026-05-22)

---

## Origin

Registered in STORY-INDEX at D-333/D-334 (2026-05-10) as the Wave 1 CrowdStrike plugin migration
story. Scope established by:

- **ADR-028 §D2** (auth_type Grounding Rule): CrowdStrike's `auth_type = "oauth2_client_credentials"`
  is the D-747 LOCKED value. The OAuth2 client-credentials flow cannot be expressed in pure TOML
  declarative syntax — it requires conditional state (token cache), network I/O (POST /oauth2/token),
  and a single-retry-on-401 semantic. A `.prx` WASM plugin is the correct architecture for this logic.
- **ADR-028 §D10** (Co-Merge Contract): PLUGIN-MIGRATION-001-A's deletion of
  `crates/prism-sensors/src/auth/crowdstrike.rs` is GATED on this story shipping the `.prx`
  plugin replacement. See §Dependencies below.
- **S-PLUGIN-PREREQ-D** (PR #149, merged `ec90fe8f`): The `.prx` loader, WIT interface validation,
  sandbox isolation, and `PluginKvStore` per-plugin key-value store are all established. This story
  consumes those primitives.
- **S-PLUGIN-PREREQ-E** (PR #151, merged `80ebe794`): `SensorAuth` is unsealed — a `.prx` plugin
  can implement `SensorAuth` without hitting the sealed-trait compile barrier.
- **Dispatch context 2026-05-22**: Authored in parallel with PLUGIN-MIGRATION-001-A (factory commit
  `e62ee028`) per user decision to parallel-author both stories for simultaneous implementation.

---

## Story-Level Goal

At merge, a `.prx` WASM plugin named `crowdstrike-oauth2` exists at
`crates/prism-spec-engine/plugins/crowdstrike-oauth2/` (source) and compiles to a Wasmtime
Component binary. The plugin implements the SensorAuth WIT interface, performs OAuth2 token
acquisition via `POST /oauth2/token`, caches the token with TTL (expiry minus 30s buffer), and
supports the single-retry-on-401 semantic by providing a fresh token when the
`PipelineExecutor::issue_request_with_retry` retry path calls `auth_provider.acquire_token()`.

`crates/prism-sensors/specs/crowdstrike.sensor.toml` is amended to declare
`auth_plugin = "crowdstrike-oauth2"`, signaling the spec engine to route auth through the plugin
rather than through the (soon-to-be-deleted) `CrowdStrikeAuth` / `CrowdStrikeAdapter` Rust code.

VP-148 (DTU parity) remains GREEN after this amendment.

---

## Narrative

As the Prism platform, I want CrowdStrike's OAuth2 client-credentials authentication logic
expressed as an in-repo `.prx` WASM plugin loaded at boot, so that `CrowdStrikeAdapter`'s
hardcoded Rust auth module can be safely deleted in PLUGIN-MIGRATION-001-A without breaking
CrowdStrike sensor query capability.

---

## Scope

### In-Scope

- Author the Rust source for the `crowdstrike-oauth2` plugin under
  `crates/prism-spec-engine/plugins/crowdstrike-oauth2/src/lib.rs` (or equivalent path per
  in-repo plugin convention — implementer must verify by reading the plugin runtime's discovery
  path in `crates/prism-spec-engine/src/plugin/discovery.rs`).
- Author `plugin.toml` manifest declaring `plugin_id = "crowdstrike-oauth2"`,
  `plugin_type = "sensor_auth"`, `format_version = 1`, and `allowed_urls` listing the CrowdStrike
  OAuth2 token endpoint host (`api.crowdstrike.com` and DTU-local host for test).
- Implement the SensorAuth WIT interface in the plugin: `auth_type_name() -> "oauth2_client_credentials"`
  and `acquire_token(credential_handle) -> Result<Token, AuthError>`.
- Token caching logic (equivalent to `CrowdStrikeAdapter::get_valid_token`): use `PluginKvStore`
  scoped to plugin_id to persist the cached token and expiry timestamp between calls. TTL = `expires_in`
  seconds from token response minus 30s buffer (matching `CachedToken::is_valid()` semantics in
  the legacy adapter).
- Single-retry-on-401: `PipelineExecutor::issue_request_with_retry` already handles the retry
  mechanic — it calls `auth_provider.acquire_token()` on 401. The plugin's `acquire_token()`
  MUST force a cache bypass on this call (i.e., always re-issue `POST /oauth2/token` when called
  directly, regardless of cache validity). Cache hits are used only by the pre-request `get_token()`
  path; `acquire_token()` is the forced-refresh entrypoint. This mirrors the dual-path design in
  the legacy adapter (`get_valid_token` vs `acquire_token`).
- Amend `crates/prism-sensors/specs/crowdstrike.sensor.toml` to add
  `auth_plugin = "crowdstrike-oauth2"` field in the `[auth]` section (or equivalent per TOML
  grammar — implementer must verify the `SensorSpec` struct's auth_plugin field path in
  `spec_parser.rs`).
- Credential handling: the WASM plugin receives an opaque `credential_handle` string that the host
  resolves to `client_id` and `client_secret` via the keyring/credential store. The guest WASM
  code never sees the raw secret value — the host exposes a `host_resolve_credential(handle) ->
  (client_id, token_endpoint_url)` WIT function that performs the resolution. The actual POST
  form body `client_secret` value is injected by the host via `host_http_request()` with the
  secret field replaced by the credential handle resolution result. See §Credential Handling
  Design below.
- Red Gate tests asserting plugin load, token acquisition, TTL cache semantics, and end-to-end
  parity against the CrowdStrike DTU clone.
- `just check` passing GREEN workspace-wide.

### Out-of-Scope

- Deletion of `crates/prism-sensors/src/auth/crowdstrike.rs` — that is 001-A's scope and is
  EXPLICITLY GATED on this story merging first per ADR-028 §D10 co-merge contract.
- `PipelineExecutor::issue_request_with_retry` internals — the retry mechanic is already
  implemented (S-PLUGIN-PREREQ-B AC-5; VP-150); this story only wires the plugin as the
  auth provider.
- Plugin signing infrastructure — unsigned `.prx` plugins load with WARN + audit log entry per
  PREREQ-D AC-4 / BC-2.17.001; this is the established v1.0 behavior.
- Hot-reload watcher for the `crowdstrike-oauth2` plugin — deferred to S-1.12-FOLLOWUP scope
  per PREREQ-D §Out of Scope.
- Extension of CrowdStrike DTU clone for the incidents table endpoint (DTU-EXT-001 gap) — tracked
  separately; incidents table remains a documented-gap entry in crowdstrike.sensor.toml per
  ADR-028 §D9.

---

## Credential Handling Design

Per AD-017 (AI-opaque credential model) and the project memory `project_ai_opaque_credentials.md`:
credential values MUST NOT transit AI context or appear in any log/debug output.

The WASM plugin cannot directly access the keyring. The host function design established by
PREREQ-D (`host_functions.rs`) provides the following integration:

1. The TOML spec's `credential_refs` declares a named credential reference (e.g., `crowdstrike_oauth2`).
2. At plugin invocation, the host resolves the credential reference to a `client_id` string
   (non-secret; safe to pass to guest) and an opaque `secret_handle` (NOT the raw secret — a
   handle the host holds in memory and uses to form the token request body server-side).
3. The plugin calls `host_http_request(method="POST", url="/oauth2/token", body_template)` where
   `body_template` uses a host-substitution placeholder for `client_secret`. The host
   `host_http_request` implementation resolves the placeholder and constructs the actual form body
   before issuing the real HTTP request.
4. The token response (`access_token`, `expires_in`) is returned to the guest as a plain string.
   The guest stores it in `PluginKvStore` via the `host_kv_store_set(key, value)` host function.
5. On cache hit, the guest reads the cached token via `host_kv_store_get(key)` and returns it
   without issuing a network request.

This design means the WASM guest code never holds `client_secret` in guest memory — the secret
stays in host memory (resolved from keyring) and is injected by the trusted host layer when
issuing the actual HTTP request.

**Security invariant (traces to BC-2.17.001 sandbox isolation):** If the guest panics during
credential resolution, the host catches the trap via Wasmtime epoch or linear-memory overflow;
credentials held in host memory are NOT accessible to the panicking guest.

---

## Behavioral Contracts

| BC ID | Version | Title | Subsystem | Role in This Story |
|-------|---------|-------|-----------|-------------------|
| BC-2.01.016 | 1.11 | SensorAuth Open Trait — Plugin-Implementable Auth Contract (No Sealed Marker) | SS-01 | **Primary** — plugin implements the open `SensorAuth` WIT equivalent; `auth_type_name()` returns `"oauth2_client_credentials"` satisfying INV-AUTH-OPEN-003 Rule A. The plugin author (this story) implements the open trait post-unsealing. |
| BC-2.01.013 | 1.6 | DataSource Trait Eliminates Per-Sensor Code Duplication | SS-01 | **Awareness** — this story progresses the BC's goal: CrowdStrike data fetch no longer requires handwritten Rust adapters; the plugin replaces CrowdStrikeAdapter's auth logic, with pipeline dispatch handling the rest. |
| BC-2.16.013 | 1.16 | Bundled Sensor Spec Authoring and DTU-Parity Verification — 4 Initial Sensors | SS-16 | **Amendment required** — crowdstrike.sensor.toml is amended with `auth_plugin` field; VP-148 parity test must remain GREEN. Any new fixture needed for plugin-path parity is added here. |
| BC-2.17.001 | current | Plugin Panic Isolation | SS-17 | **Sandbox invariant** — `.prx` plugin panics must be caught; credentials in host memory must not leak to panicking guest. |
| BC-2.17.006 | current | Plugin WIT Validation | SS-17 | **Load gate** — plugin must export the SensorAuth WIT interface; PluginRuntime rejects at load if WIT validation fails. |
| BC-2.17.007 | 1.4 | Plugin Manifest Schema Validation | SS-17 | **Load gate** — `plugin.toml` manifest must pass schema validation: `format_version = 1`, `plugin_id`, `plugin_type`, `allowed_urls` fields required. |
| BC-2.22.001 | current | Boot Orchestration | SS-22 | **Boot sequence** — `crowdstrike-oauth2` plugin discovered and loaded at step 7.5 per BC-2.22.001 §Sequencing Invariant. Plugin load failure is a recoverable boot step failure (WARN + continue) unless `PRISM_DISABLE_PLUGIN_LOAD` is set. |

---

## Token Budget Estimate

| Artifact | Estimated Tokens |
|----------|-----------------|
| This story spec | ~5,000 |
| BC-2.01.016 (open SensorAuth contract, full read) | ~5,000 |
| BC-2.16.013 (bundled spec parity, full read) | ~4,000 |
| BC-2.17.001/006/007 (sandbox + WIT + manifest, full reads) | ~3,000 |
| ADR-028 (spec grounding, full read for §D2/§D10) | ~3,000 |
| crowdstrike.rs (existing OAuth2 impl, full read — gene-transfusion source) | ~2,500 |
| DTU clone oauth.rs + routes/mod.rs (endpoint reference) | ~1,500 |
| prism-spec-engine/src/plugin/ (loader.rs, mod.rs, host_functions.rs — partial read) | ~4,000 |
| spec_parser.rs (auth_plugin field search — partial) | ~2,000 |
| S-PLUGIN-PREREQ-D story (plugin runtime contract reference) | ~3,000 |
| PLUGIN-MIGRATION-001-A story (AC-006 gate context) | ~2,000 |
| Test pattern files (prism-spec-engine/tests/plugin_tests.rs, parity/) | ~3,000 |
| Cargo.toml files (prism-spec-engine, new plugin crate) | ~1,000 |
| **Total estimate** | **~39,000** |
| Agent context window (claude-sonnet-4-6) | ~200,000 |
| **% of context window** | **~20%** |

This story is within the 20–30% target. The bulk is the legacy adapter (gene-transfusion source),
plugin runtime contracts, and test pattern files.

---

## Acceptance Criteria

Each AC traces to the specific BC clause it satisfies. Implementer ticks off as each Red Gate
test passes.

### AC-001: Plugin Source Exists at Canonical Location and Compiles (traces to BC-2.17.007 postcondition — manifest validation passes; BC-2.17.006 postcondition — WIT export present)

`crates/prism-spec-engine/plugins/crowdstrike-oauth2/src/lib.rs` (or equivalent per plugin
discovery path) exists. The associated `plugin.toml` manifest declares:
- `format_version = 1` (CURRENT_SUPPORTED_VERSION per `prism-spec-engine/src/plugin/mod.rs`)
- `plugin_id = "crowdstrike-oauth2"`
- `plugin_type = "sensor_auth"` (or equivalent WIT plugin type constant)
- `allowed_urls` containing `"api.crowdstrike.com"` (production host) and the DTU test host
  (parameterized by env or test fixture)

`cargo build -p crowdstrike-oauth2-plugin` (or equivalent compile target) exits 0.

**Red Gate Test 1:** `test_PLUGIN_MIGRATION_001_E_001_plugin_compiles_and_manifest_validates` —
asserts `PluginRuntime::load_plugin(crowdstrike_oauth2_path)` returns `Ok(...)` without panic,
verifying WIT validation + manifest schema gate both pass.

### AC-002: Plugin `auth_type_name()` Returns Canonical Value (traces to BC-2.01.016 postcondition — `auth_type_name()` returns the `&'static str` for this auth type; INV-AUTH-OPEN-003 Rule A)

When the loaded plugin is invoked via `PluginRuntime::call_auth_type_name()` (or equivalent WIT
dispatch), it returns the string `"oauth2_client_credentials"`.

This value MUST match `crowdstrike.sensor.toml`'s `auth_type = "oauth2_client_credentials"` field
(ADR-028 §D2 LOCKED value). Mismatch would cause `E-SPEC-012` at spec-load time.

**Red Gate Test 2:** `test_PLUGIN_MIGRATION_001_E_002_auth_type_name_returns_oauth2_client_credentials` —
loads plugin, dispatches WIT call, asserts returned string equals `"oauth2_client_credentials"`.

### AC-003: Token Acquisition via `POST /oauth2/token` Against DTU Clone (traces to BC-2.01.016 preconditions satisfied — `SensorAuth` open and implementable; BC-2.16.013 postcondition — plugin path produces correct OCSF output from DTU-grounded endpoints)

When `plugin.acquire_token(credential_handle)` is called:
1. Plugin issues `POST /oauth2/token` via `host_http_request()` with `client_id` + `client_secret`
   form params.
2. Plugin parses `access_token` string and `expires_in` integer from the JSON response.
3. Plugin returns the `access_token` string as the bearer token.
4. DTU clone (`oauth::token` handler in `crates/prism-dtu-crowdstrike/src/routes/oauth.rs`) responds
   `200 OK` with `{"access_token": "dtu-fake-cs-token", "token_type": "bearer", "expires_in": 3600}`.

**Red Gate Test 3:** `test_PLUGIN_MIGRATION_001_E_003_acquire_token_calls_oauth2_token_endpoint` —
starts CrowdStrike DTU clone, calls `acquire_token`, asserts DTU received `POST /oauth2/token`
and returned `access_token = "dtu-fake-cs-token"`.

### AC-004: Token Cached with TTL; Subsequent Calls Within TTL Reuse Cache (traces to BC-2.01.016 invariant — SensorAuth contract is stateless from the trait perspective; BC-2.17.001 — KV store state between calls is guest-maintained)

After a successful `acquire_token`:
1. The plugin stores token + expiry in `PluginKvStore` via `host_kv_store_set("token", ...)` and
   `host_kv_store_set("expires_at_secs", ...)`.
2. A subsequent call to `get_token()` (the non-forced path) within the TTL reads from KV store via
   `host_kv_store_get("token")` and returns the cached value WITHOUT issuing a new HTTP request to
   `/oauth2/token`.
3. Cache TTL = `expires_in` seconds minus 30 seconds buffer (matching `CachedToken::is_valid()`
   semantics in the legacy adapter — RFC 6749 recommends early expiry handling).

**Red Gate Test 4:** `test_PLUGIN_MIGRATION_001_E_004_token_cached_within_ttl_no_second_request` —
acquires token, then calls `get_token()` within TTL; asserts DTU `/oauth2/token` was called exactly
ONCE (not twice). DTU request counter provides the assertion signal.

### AC-005: Expired Token Triggers Re-acquisition (traces to BC-2.01.016 — `acquire_token()` forced-refresh path)

When the KV-cached token's `expires_at_secs` is in the past:
1. `get_token()` detects cache miss (expiry elapsed).
2. Plugin issues a new `POST /oauth2/token` request.
3. New token overwrites the KV cache entry.

**Red Gate Test 5:** `test_PLUGIN_MIGRATION_001_E_005_expired_token_triggers_reacquisition` —
acquires token, mutates KV store to set `expires_at_secs` to a past timestamp, then calls
`get_token()`; asserts DTU `/oauth2/token` was called twice.

### AC-006: 401 Response Triggers Token Refresh + Single Retry via PipelineExecutor (traces to BC-2.01.016 — `acquire_token()` forced-refresh; VP-150 / VP-PLUGIN-005 — OAuth2 refresh-on-401 via PipelineExecutor retry path)

`PipelineExecutor::issue_request_with_retry` (established in PREREQ-B, AC-5) calls
`auth_provider.acquire_token()` when the primary request returns HTTP 401. The plugin's
`acquire_token()` implementation is the forced-refresh entrypoint — it MUST bypass the KV cache
and always issue a new `POST /oauth2/token` request, returning a fresh token.

End-to-end scenario:
1. Plugin acquires initial token.
2. DTU configured to return 401 on first detection query request (failure injection via
   `POST /dtu/configure {"auth_mode": "reject"}` followed by `{"auth_mode": "normal"}`).
3. `PipelineExecutor::execute()` for CrowdStrike detections table receives 401 on first attempt.
4. Pipeline calls `auth_provider.acquire_token()` → plugin issues fresh `POST /oauth2/token` → new
   token returned → retry succeeds.
5. Second 401 (if retry also returns 401) propagates as `SpecEngineError::AuthRefreshFailed`
   (AC-5 abort condition from PREREQ-B, unchanged).

**Red Gate Test 6:** `test_PLUGIN_MIGRATION_001_E_006_401_triggers_plugin_token_refresh_and_retry` —
starts DTU clone, configures single-401 failure injection, runs `PipelineExecutor::execute()` for
CrowdStrike detections table via plugin auth path; asserts (a) final result is `Ok(records)` with
non-empty OCSF output, (b) `/oauth2/token` was called twice (initial + refresh), (c) detection
query endpoint was called twice (initial 401 + retry 200).

### AC-007: `crowdstrike.sensor.toml` Amended with `auth_plugin` Field (traces to BC-2.16.013 postcondition — spec file amended; ADR-028 §D2 — auth declaration grounds through plugin path)

`crates/prism-sensors/specs/crowdstrike.sensor.toml` is amended to declare:
```toml
auth_plugin = "crowdstrike-oauth2"
```
in the appropriate location per the `SensorSpec` struct's `auth_plugin` field (implementer must
verify the field path in `spec_parser.rs` — if the field does not yet exist, it must be added to
`SensorSpec` with `#[serde(default)]` as a `Option<String>`, emitting `E-SPEC-012` if an unknown
plugin ID is declared).

**Red Gate Test 7:** `test_PLUGIN_MIGRATION_001_E_007_crowdstrike_toml_declares_auth_plugin` —
parses `crowdstrike.sensor.toml` via `SpecLoader::load_all()`, asserts the resulting `SensorSpec`
has `auth_plugin == Some("crowdstrike-oauth2")` and `auth_type == AuthType::Oauth2ClientCredentials`.

### AC-008: VP-148 Parity Test Remains GREEN After TOML Amendment (traces to BC-2.16.013 INV-PARITY-001 — replacement-before-deletion enforcement; VP-148 must be verified for all 4 sensors)

After amending `crowdstrike.sensor.toml` with `auth_plugin`, the DTU-parity tests authored in
PLUGIN-MIGRATION-001-D (under `crates/prism-spec-engine/tests/parity/`) continue to pass. The
plugin path produces OCSF output byte-identical (post-canonicalization per TS-PLUGIN-PARITY-001)
to the reference fixture JSON at
`crates/prism-dtu-crowdstrike/fixtures/parity/reference-ocsf/<table>.json`.

If the existing parity test used a `NullAuthProvider`, it must be updated to use the
`crowdstrike-oauth2` plugin auth path. The fixture JSON is NOT regenerated — it remains the
committed reference from 001-D.

**Red Gate Test 8:** `test_PLUGIN_MIGRATION_001_E_008_vp148_parity_green_after_toml_amendment` —
runs the CrowdStrike DTU-parity integration test (or asserts the existing test from 001-D still
passes) with the amended TOML + plugin auth path; asserts OCSF output matches fixture reference.

### AC-009: Plugin Loaded at Boot Step 7.5 (traces to BC-2.22.001 §Sequencing Invariant — plugin-load step 7.5 completes before MCP server bind)

When `prism start` is invoked with `crowdstrike-oauth2` plugin on the allowed list,
`PluginRuntime::load_plugin` loads the binary at boot step 7.5. Unsigned plugin load emits
`tracing::warn!(event_type = "plugin_load_unsigned", plugin_id = "crowdstrike-oauth2")` per
PREREQ-D AC-4 / BC-2.17.001 (unsigned v1.0 behavior).

**Red Gate Test 9:** `test_PLUGIN_MIGRATION_001_E_009_plugin_loaded_at_boot_step_7_5_emits_warn` —
uses existing PREREQ-D boot integration test harness to assert the plugin is loaded and the
unsigned WARN event is emitted with `plugin_id = "crowdstrike-oauth2"`.

### AC-010: Credential Opaqueness — Token Value Not Logged (traces to BC-2.01.016 postcondition — `auth_type_name()` Debug safety; AD-017 AI-opaque credential model)

The plugin's debug output and any `tracing::*!` events emitted during `acquire_token()` or
`get_token()` MUST NOT contain the raw `access_token` string or `client_secret` value.

`PluginKvStore` entries are plain strings (no `SecretString` wrapper in WASM guest context).
The host's `host_kv_store_set()` implementation MUST treat keys matching `"token"` pattern as
credential-class data and omit them from log output. Implementer must verify the
`PluginKvStore::set()` implementation in `loader.rs` does not log KV values.

**Red Gate Test 10 (security):** `test_PLUGIN_MIGRATION_001_E_010_token_not_in_tracing_output` —
captures tracing subscriber output during plugin `acquire_token()` execution against DTU clone;
asserts the returned `access_token` value does not appear verbatim in any log line.

### AC-011: `just check` Workspace-Wide GREEN (traces to BC-2.16.013 postcondition — CI gate; project production-grade default per CLAUDE.md)

`just check` (fmt + clippy + nextest + doctests + crate-layout) exits 0 after all story changes
are applied. No new `#[allow(clippy::...)]` attributes introduced without justification. No
`unwrap()` / `expect()` in non-test plugin host-binding code. All `#[non_exhaustive]` requirements
satisfied for any new public types (per CLAUDE.md conventions).

---

## Edge Cases

| ID | Description | Expected Behavior | Test Reference |
|----|-------------|-------------------|----------------|
| EC-001 | `POST /oauth2/token` returns HTTP 401 (invalid client credentials) | Plugin returns `AuthError::InvalidCredentials`; PipelineExecutor propagates as `SpecEngineError::AuthRefreshFailed`; no token cached | `test_acquire_token_EC_001_401_returns_invalid_credentials` |
| EC-002 | `POST /oauth2/token` returns HTTP 200 but response body is not valid JSON | Plugin returns `AuthError::ResponseParse`; host logs `tracing::error!(event_type = "plugin.auth_token_parse_error")`; no token cached | `test_acquire_token_EC_002_invalid_json_returns_response_parse` |
| EC-003 | Token response is missing `access_token` field | Plugin returns `AuthError::ResponseParse` with detail "missing access_token field"; no token cached | `test_acquire_token_EC_003_missing_access_token_returns_response_parse` |
| EC-004 | Token response `expires_in` field is missing or zero | Plugin defaults to 1799 seconds TTL (matching legacy `CrowdStrikeAdapter::acquire_token` `unwrap_or(1799)` semantics); token is cached | `test_acquire_token_EC_004_missing_expires_in_defaults_to_1799`, `test_acquire_token_EC_004_zero_expires_in_defaults_to_1799` |
| EC-005 | KV store at 1MB limit when trying to cache a new token | Plugin returns `AuthError::Internal("kv_store size limit exceeded")`; query fails with structured error; no silent truncation | `test_acquire_token_EC_005_kv_set_error_propagates` |
| EC-006 | Plugin binary is missing from expected path at boot | `PluginRuntime::load_plugin` fails; boot step 7.5 emits ERROR and continues (plugin unavailable, not a fatal boot failure per PREREQ-D AC-3 semantics) | wasm32 Guest impl / WAT-fixture / integration tests (not closed in FB-IMPL-4) |
| EC-007 | Plugin loaded but WIT validation fails (wrong export signature) | `PluginRuntime::load_plugin` returns `Err(PluginError::WitValidationFailed)`; plugin not registered; boot continues | wasm32 Guest impl / WAT-fixture / integration tests (not closed in FB-IMPL-4) |
| EC-008 | `host_http_request()` to `/oauth2/token` is rejected by allowlist (wrong host) | Host returns `Err(PluginError::SandboxViolation)`; plugin returns `AuthError::Internal`; no token acquired | wasm32 Guest impl / WAT-fixture / integration tests (not closed in FB-IMPL-4) |
| EC-009 | Double 401: both initial request and refresh request return 401 | PipelineExecutor's `issue_request_with_retry` propagates `SpecEngineError::AuthRefreshFailed` (AC-5 of PREREQ-B; unchanged behavior) | wasm32 Guest impl / WAT-fixture / integration tests (not closed in FB-IMPL-4) |

**Defense-in-depth note (FB-IMPL-5):** `test_acquire_token_non_2xx_returns_response_parse` covers the case where the token endpoint returns a non-2xx status other than 401 (e.g. HTTP 503). This exercises the status-check branch BEFORE JSON parsing — a separate code path from EC-002 (200 + invalid JSON). Non-2xx is not a named EC row because it is primarily a defense-in-depth path (the production CrowdStrike OAuth2 endpoint rarely returns non-401 errors); EC-002 is the operationally more likely scenario where a proxy or WAF returns 200 with an error body.

---

## Tasks

Numbered in TDD discipline order: stubs → Red Gate failing → TDD green. Implementer works
through these sequentially; each Red Gate test must be failing (not compile-error-failing) before
implementation makes it pass.

### Task 0: Read Source Files Before Writing Any Code

1. Read `crates/prism-sensors/src/auth/crowdstrike.rs` (gene-transfusion source for OAuth2 logic)
2. Read `crates/prism-dtu-crowdstrike/src/routes/oauth.rs` (DTU endpoint spec)
3. Read `crates/prism-spec-engine/src/plugin/loader.rs` (PluginKvStore, PluginMetadata, LoadedPlugin)
4. Read `crates/prism-spec-engine/src/plugin/mod.rs` (PluginRuntime, PLUGIN_HTTP_CLIENT_TIMEOUT_SECS,
   CURRENT_SUPPORTED_VERSION)
5. Read `crates/prism-spec-engine/src/plugin/host_functions.rs` (host_kv_store_set/get, host_http_request)
6. Read `crates/prism-spec-engine/src/plugin/discovery.rs` (plugin discovery path convention)
7. Read `crates/prism-spec-engine/src/spec_parser.rs` (SensorSpec struct — verify auth_plugin field
   presence; if absent, note it must be added)

### Task 1: Add `auth_plugin` Field to `SensorSpec` (if absent)

If `SensorSpec` in `spec_parser.rs` does not have an `auth_plugin: Option<String>` field:
1. Add the field with `#[serde(default)]` — backward compatible; existing TOML files without the
   field parse to `None`.
2. Add a validation rule: if `auth_plugin` is `Some(id)` and `id` is not registered in
   `PluginRuntime.registry`, emit `E-SPEC-012` at spec-load time with message
   `"auth_plugin '{id}' is not a registered plugin"`.
3. Add a unit test asserting `SensorSpec` with no `auth_plugin` parses to `None`.

### Task 2: Scaffold Plugin Crate

1. Create `crates/prism-spec-engine/plugins/crowdstrike-oauth2/` directory.
2. Create `Cargo.toml` for the plugin crate (WASM Component target).
3. Create `plugin.toml` manifest with:
   - `format_version = 1`
   - `plugin_id = "crowdstrike-oauth2"`
   - `plugin_type = "sensor_auth"`
   - `allowed_urls = ["api.crowdstrike.com", "localhost"]`  (localhost for DTU test)
4. Create `src/lib.rs` with `todo!()` stubs for all exported WIT functions.

### Task 3: Write All Red Gate Tests (Stub Phase)

Write tests 1–10 in `crates/prism-spec-engine/tests/plugin_integration_tests.rs` (or a new
`tests/crowdstrike_oauth2_plugin_tests.rs`). Each test MUST compile and FAIL (not error) with
`todo!()` stubs in place — this is the Red Gate requirement. Do not run `just check` until all
tests are written and failing.

### Task 4: Implement WIT Interface Exports

1. Implement `auth_type_name() -> &str { "oauth2_client_credentials" }` export.
2. Implement `acquire_token(credential_handle: &str) -> Result<String, AuthError>`:
   - Call `host_resolve_credential(credential_handle)` → gets `client_id`, keeps `client_secret`
     opaque via host injection.
   - Call `host_http_request("POST", token_url, form_body_with_placeholder)`.
   - Parse JSON response; extract `access_token` and `expires_in`.
   - Call `host_kv_store_set("token", access_token_value)`.
   - Call `host_kv_store_set("expires_at_secs", computed_expiry.to_string())`.
   - Return `Ok(access_token)`.
3. Implement `get_token(credential_handle: &str) -> Result<String, AuthError>`:
   - Read `host_kv_store_get("expires_at_secs")`; if present and in future, return cached token.
   - Otherwise fall through to `acquire_token(credential_handle)` (cache bypass).
4. Run `cargo nextest run -p prism-spec-engine -E 'test(PLUGIN_MIGRATION_001_E_002)'` — must PASS.

### Task 5: Implement Token Caching

Implement TTL check in `get_token()`:
- Parse `expires_at_secs` from KV store as `u64`.
- Compare to current Unix timestamp (via `host_current_time_secs()` host function — implementer
  must verify this host function exists in `host_functions.rs`; if absent, use an approximate
  approach via the DTU clock or a guest timestamp built from Wasmtime epoch).
- Run tests 4 and 5 — must PASS.

### Task 6: Amend `crowdstrike.sensor.toml`

Add `auth_plugin = "crowdstrike-oauth2"` to the appropriate section of
`crates/prism-sensors/specs/crowdstrike.sensor.toml`. Verify that:
1. `SpecLoader::load_all()` parses the amended TOML without errors.
2. `SensorSpec.auth_plugin == Some("crowdstrike-oauth2")`.
3. Run test 7 — must PASS.

### Task 7: End-to-End Parity and Retry Tests

1. Run the existing CrowdStrike DTU-parity test from PLUGIN-MIGRATION-001-D against the amended
   TOML + plugin path (test 8). If the parity test used `NullAuthProvider`, replace with plugin
   auth provider. Assert fixture parity holds — if it fails, debug the plugin token path against
   the DTU before proceeding.
2. Implement the 401-retry scenario (test 6): configure DTU failure injection, run
   `PipelineExecutor::execute()`, assert both `/oauth2/token` calls and the detection query retry.

### Task 8: Security Test and Boot Test

1. Implement token-not-in-tracing test (test 10) using a capturing tracing subscriber.
2. Run boot integration test (test 9) using the PREREQ-D harness — assert plugin loaded with WARN.

### Task 9: Pre-Push Gate

`just check` — fix any remaining issues (fmt, clippy, layout). No `--no-verify`. Must be CLEAN
before declaring implementation done.

---

## Previous Story Intelligence

S-PLUGIN-PREREQ-D (PR #149, merged `ec90fe8f`) established the plugin runtime foundation. Key
lessons that apply here:

1. **`allowed_urls` is `Vec<String>`, not `Option<Vec<String>>`** — AC-7 of PREREQ-D changed the
   type from `Option<Vec<String>>` (old) to `Vec<String>` (new) with default-deny semantics. The
   plugin manifest's `allowed_urls` field must be an explicit list. An empty list `allowed_urls = []`
   blocks ALL outbound HTTP. This story's plugin MUST declare `allowed_urls` with at least
   `"api.crowdstrike.com"`.

2. **`format_version = 1` is the current cap** — `CURRENT_SUPPORTED_VERSION` in `plugin/mod.rs` is
   `1`. Manifests with `format_version > 1` are rejected with `E-PLUGIN-014 FormatVersionExceeded`.

3. **Boot step 7.5 sequencing** — plugins load between storage init (step 7) and query-engine init
   (step 8). This story's plugin must be discoverable at the path `PluginRuntime::discover_plugins()`
   scans (verify `discovery.rs`).

4. **Unsigned WARN is the v1.0 behavior** — do not attempt to sign the plugin; the WARN + audit log
   event is expected and tested (PREREQ-D AC-4).

5. **`PluginKvStore` is scoped to `plugin_id`** — KV keys are automatically prefixed with
   `"crowdstrike-oauth2:"` by the `PluginKvStore::set()` implementation. The guest code uses
   bare keys like `"token"` and `"expires_at_secs"`; the host scopes them automatically.

6. **`MAX_REQUESTS_PER_PIPELINE = 10_000`** — plugin outbound HTTP calls count against this cap.
   For CrowdStrike's OAuth2 flow (1 token request + N detection/device requests), the cap is
   not a concern in practice.

PLUGIN-MIGRATION-001-D (PR #153, merged `3f2de889`) established `crowdstrike.sensor.toml` at
`crates/prism-sensors/specs/`. The amendment in this story (adding `auth_plugin`) must not break
any of the 001-D assertions (especially AC-001: `tables.len() == 3`, AC-002: `auth_type ==
"oauth2_client_credentials"`, AC-008: `just check` GREEN).

---

## Architecture Compliance Rules

Extracted from `architecture/module-decomposition.md` and relevant ADRs. Violations are P0 PR blockers.

| Rule | Source | Enforcement |
|------|--------|-------------|
| No `prism-sensors` dev-dep on `prism-spec-engine` in non-test code | ADR-028 §D3; 001-D §Forbidden Dependencies | Build-time: `cargo metadata` check in CI |
| Plugin WASM binary compiled to `wasm32-wasi` or `wasm32-unknown-unknown` component target | BC-2.17.006 WIT validation requires Component Model ABI | `cargo build --target wasm32-...` CI step |
| `#[non_exhaustive]` on all public structs/enums in the plugin crate's host-facing API surface | CLAUDE.md conventions; prism-spec-engine compile-fail gate count is 32+ (EXPECTED=32 in ci.yml) | `tests/external/non-exhaustive-violation/` compile-fail gate |
| `allowed_urls` in plugin manifest must be explicit `Vec<String>` — not empty unless plugin makes no outbound calls | BC-2.17.007 postcondition; PREREQ-D AC-7 default-deny | WIT manifest validator at load time |
| OAuth2 `client_secret` value MUST NOT appear in any `tracing::*!` event or `Debug` impl | AD-017 AI-opaque credential model | AC-010 Red Gate Test 10 |
| No `println!` in production plugin code | CLAUDE.md conventions | `clippy::print_stdout` lint |
| `auth_type_name()` return value MUST match `crowdstrike.sensor.toml`'s `auth_type` field | INV-AUTH-OPEN-003 Rule A; BC-2.01.016 postcondition | Red Gate Test 2 |
| Plugin `acquire_token()` MUST bypass KV cache (always re-issue token request) | 401-retry semantic — forced-refresh entrypoint | Red Gate Test 6 |

### Forbidden Dependencies

The plugin crate MUST NOT depend on:
- `prism-sensors` — would create a circular dependency (sensors crate is what the plugin replaces)
- `prism-core` internal types beyond `PluginError` and `SensorId` (if needed at all)
- `secrecy` crate in WASM guest code — `secrecy` relies on `zeroize` which has WASM compatibility
  constraints; credential handling in WASM uses opaque handle strings, not `SecretString`

---

## Library and Framework Requirements

| Library | Version | Justification |
|---------|---------|---------------|
| `wasmtime` | per `prism-spec-engine/Cargo.toml` (workspace pin) | Plugin runtime; do not add a separate wasmtime dep in the plugin crate — it's WASM guest code, not host |
| `serde_json` | workspace pin | Token response JSON parsing in plugin guest |
| `wit-bindgen` | per workspace pin if present; else per PREREQ-D convention | WIT bindings generation for the plugin guest |
| `reqwest` | workspace pin — host only; NOT in plugin guest | Token HTTP request is issued via `host_http_request()` host function, NOT via reqwest in guest |

Do NOT pin new library versions in the plugin crate. Use workspace-inherited versions. If
`wit-bindgen` is not yet in the workspace, add it at the version PREREQ-D already uses (check
`crates/prism-spec-engine/Cargo.toml` for existing wit-bindgen usage).

---

## File Structure Requirements

| File | Action | Notes |
|------|--------|-------|
| `crates/prism-spec-engine/plugins/crowdstrike-oauth2/Cargo.toml` | CREATE | Plugin crate manifest; WASM target |
| `crates/prism-spec-engine/plugins/crowdstrike-oauth2/plugin.toml` | CREATE | Plugin runtime manifest; format_version=1 |
| `crates/prism-spec-engine/plugins/crowdstrike-oauth2/src/lib.rs` | CREATE | Plugin guest implementation |
| `crates/prism-spec-engine/plugins/crowdstrike-oauth2/wit/` | CREATE | WIT interface definitions (sensor-auth.wit) |
| `crates/prism-sensors/specs/crowdstrike.sensor.toml` | MODIFY | Add `auth_plugin = "crowdstrike-oauth2"` field |
| `crates/prism-spec-engine/src/spec_parser.rs` | MODIFY | Add `auth_plugin: Option<String>` field to `SensorSpec` (if absent) |
| `crates/prism-spec-engine/tests/crowdstrike_oauth2_plugin_tests.rs` | CREATE | Red Gate tests 1–10 |
| `crates/prism-spec-engine/tests/parity/crowdstrike_parity.rs` | MODIFY | Update to use plugin auth path (was NullAuthProvider) |

**Note on plugin directory path:** Before creating these files, implementer MUST read
`crates/prism-spec-engine/src/plugin/discovery.rs` to confirm the canonical discovery path. If the
convention differs from `plugins/`, use the actual convention. Do not invent a directory name.

---

## Dependencies

### Depends On

| Story | Reason |
|-------|--------|
| S-PLUGIN-PREREQ-D | `.prx` loader, WIT validation, sandbox, PluginKvStore established. Plugin builds on these primitives — cannot be implemented without them. |
| S-PLUGIN-PREREQ-E | `SensorAuth` unsealed — plugin can implement the auth trait without sealed-trait barrier. |
| PLUGIN-MIGRATION-001-D | `crowdstrike.sensor.toml` exists at `crates/prism-sensors/specs/`. This story amends that file. Without 001-D having authored the TOML file, this story has no file to amend. |

### Blocks

| Story | Reason (ADR-028 §D10 co-merge contract) |
|-------|----------------------------------------|
| PLUGIN-MIGRATION-001-A | **AC-006 of PLUGIN-MIGRATION-001-A is GATED on this story.** Per ADR-028 §D10, the deletion of `crates/prism-sensors/src/auth/crowdstrike.rs` by 001-A MUST NOT proceed until this story has shipped the `.prx` plugin replacement. The depends_on entry `PLUGIN-MIGRATION-001-E` in 001-A frontmatter encodes this gate. |

### Co-Merge Note (ADR-028 §D10)

PLUGIN-MIGRATION-001-A and PLUGIN-MIGRATION-001-D MUST be deployed to production simultaneously
(per §D10 co-merge contract). After this story merges, the full co-merge unit for production
deployment is: 001-D (TOML specs + parity tests; already merged PR #153) + 001-E (plugin
replacement) + 001-A (deletion of hardcoded auth modules). Development and CI builds may have
stories merged independently; the regression risk is PRODUCTION deployment.

---

## Known Gaps

| Gap | Scope | Resolution Target |
|-----|-------|-------------------|
| CrowdStrike `incidents` table endpoint (DTU-EXT-001) | No DTU route registered for incidents; parity test for incidents remains `#[ignore]` | Follow-up DTU-EXT-001 story |
| Plugin signing for production-grade unsigned bypass | v1.0 unsigned WARN behavior accepted per PREREQ-D AC-4 / TD-PLUGIN-SIGNING-001 | Future plugin-signing story |
| Hot-reload watcher for `crowdstrike-oauth2` plugin | Out of scope per PREREQ-D §Out of Scope | S-1.12-FOLLOWUP |
| `host_current_time_secs()` host function may not yet exist | If absent, Task 5 must implement a workaround (e.g., embed timestamp in token acquisition, accept ~30s precision loss in TTL, or add the host function in scope) | In-scope: implementer adds if missing |

---

## Source Citations

| Artifact | Version / SHA | Purpose |
|----------|--------------|---------|
| `crates/prism-sensors/src/auth/crowdstrike.rs` | develop@`f19575ff` (post PR #153) | Gene-transfusion source: `CrowdStrikeAdapter::acquire_token`, `get_valid_token`, `CachedToken::is_valid` |
| `crates/prism-sensors/specs/crowdstrike.sensor.toml` | develop@`f19575ff` | File being amended; authored in 001-D |
| `crates/prism-dtu-crowdstrike/src/routes/oauth.rs` | develop@`f19575ff` | DTU token endpoint behavior: `oauth::token` handler |
| `crates/prism-dtu-crowdstrike/src/routes/mod.rs` | develop@`f19575ff` | DTU router; `/oauth2/token` route registration |
| `crates/prism-spec-engine/src/plugin/mod.rs` | develop@`f19575ff` | `PluginRuntime`, `CURRENT_SUPPORTED_VERSION = 1`, `PLUGIN_HTTP_CLIENT_TIMEOUT_SECS = 30` |
| `crates/prism-spec-engine/src/plugin/loader.rs` | develop@`f19575ff` | `PluginKvStore`, `PluginMetadata`, `LoadedPlugin` |
| `ADR-028` | v1.10 (2026-05-21) | §D2 auth_type LOCKED, §D10 co-merge contract |
| `BC-2.01.016` | v1.11 | SensorAuth open trait contract |
| `BC-2.16.013` | v1.16 | Bundled sensor spec parity; INV-PARITY-001 |
| `BC-2.17.007` | v1.4 | Plugin manifest schema validation |
| RFC 6749 (OAuth 2.0 Authorization Framework) | — | Client credentials grant spec; token expiry semantics |
| RFC 6750 (OAuth 2.0 Bearer Token Usage) | — | Bearer token in Authorization header |

---

## Changelog

| Version | Date | Author | Change |
|---------|------|--------|--------|
| v1.0 | 2026-05-22 | story-writer | Initial authoring — full sprint-ready story per dispatch 2026-05-22 |
