---
document_type: story
story_id: S-DEMO-001
title: "prism-bin: SpecDrivenSensorAdapter + Boot Step 9A — Bridge PipelineExecutor to AdapterRegistry (closes GAP-002-A)"
wave: 5
epic_id: E-DEMO
priority: P0
status: draft
version: "1.2"
level: "L4"
producer: story-writer
revised_by: architect
timestamp: "2026-05-29T00:00:00Z"
tdd_mode: strict
subsystems: [SS-01, SS-16, SS-22]
# Subsystem anchor justifications:
#   SS-01 (Sensor Adapters) owns the SensorAdapter trait that SpecDrivenSensorAdapter implements;
#     this story closes the registry gap so fan_out() can resolve (org, sensor) → adapter.
#   SS-16 (Spec Engine) owns PipelineExecutor and the SensorSpec catalog that drives dispatch;
#     SpecDrivenSensorAdapter wraps PipelineExecutor, so SS-16 is a direct consumer.
#   SS-22 (Binary Entrypoint) owns boot.rs and the boot step sequencing contract (BC-2.22.001);
#     boot step 9A is a new sequencing invariant that must be registered in the ADR-022 §B table.
crates_touched: [prism-bin, prism-spec-engine]
target_module: prism-bin
capabilities: [CAP-001, CAP-015, CAP-029, CAP-034]
behavioral_contracts:
  - BC-2.01.013  # DataSource Trait Eliminates Per-Sensor Code Duplication — SpecDrivenSensorAdapter
                 # is the spec-driven implementation of the SensorAdapter interface.
  - BC-2.11.005  # Ephemeral Materialization — fan_out() calls registry.get(org_id, sensor_id);
                 # after this story, get() returns SpecDrivenSensorAdapter for every (org, sensor)
                 # pair in the spec catalog, enabling end-to-end live data flow.
  - BC-2.06.014  # Instance Identity Resolution at Fanout — (org_id, sensor_id) tuple resolves
                 # to ResolvedSensorSpec; SpecDrivenSensorAdapter receives that ResolvedSensorSpec
                 # (with per-org base_url overlay from ADR-029) at construction time during boot 9A.
  - BC-2.22.001  # Boot Orchestration Sequencing — boot step 9A must be registered in the
                 # ADR-022 §B sequencing invariant table between step 7.5b and step 9.
verification_properties:
  - VP-148  # VP-PLUGIN-003 DTU parity — existing parity tests exercise the pipeline path that
            # SpecDrivenSensorAdapter delegates to; this story wires the adapter into the registry
            # so parity tests exercise the full round-trip path.
depends_on:
  - PLUGIN-MIGRATION-001-A   # Must merge first: cleans up legacy hardcoded adapter code so the
                              # registry is unambiguously empty and owned by spec-driven path only.
  - PLUGIN-MIGRATION-001-E   # Must merge first: provides CrowdStrike OAuth2 PluginAuthProvider
                              # at boot step 7.5b; SpecDrivenSensorAdapter for CrowdStrike holds
                              # this Arc<PluginAuthProvider> and ignores the SensorAuth argument.
  - S-CONFIG-MULTI-TENANT-OVERRIDE-001  # Must merge first: per-org overlay loading (ADR-029)
                                         # produces ResolvedSensorSpec map; boot 9A iterates this
                                         # map to construct one SpecDrivenSensorAdapter per (org, sensor).
blocks:
  - S-DEMO-002   # E2E smoke test cannot run without adapters in the registry.
  - S-5.04       # Sensor health checks call the registered adapter; health subsystem is
                 # downstream of GAP-002-A closure.
  - S-5.04-FIX-001  # Spec fix story depends on this story existing as the new S-5.04 dep.
points: 11
# Points justification (revised from v1.0's 8 pts — 3 pts added for Cyberint cookie auth work):
#   - SpecDrivenSensorAdapter struct + SensorAdapter impl (plugin auth path): ~1.5 pts
#   - BearerStaticAuthProvider for Armis + Claroty (bearer_static auth type): ~1 pt
#   - CookieLoginAuthProvider for Cyberint (cookie_roundtrip auth type):
#       • login step in TOML spec (POST /login → capture Set-Cookie) OR
#       • PipelineExecutor build_request auth-type-aware dispatch to Cookie header: ~2 pts
#     (see §Cyberint Cookie Auth Design — architect recommends TOML login-step approach)
#   - Boot step 9A loop + AdapterRegistry::register() calls: ~1.5 pts
#   - reqwest::Client construction with 30s timeout (AD-017 compliant): ~0.5 pts
#   - BC-2.16.002 catalog row for boot.step9a.adapter_registry_populated: ~0.5 pts
#   - Red Gate tests (4 required, see ACs): ~2 pts
#   - ADR-022 §B + ADR-023 §Permitted-Patterns amendment: ~0.5 pts
#   - ADR-028 §D10 amendment (cookie_roundtrip login-step design): ~1 pt
#   - OQ-1 resolution (bearer_static per-fetch construction): ~0.5 pts
#   Total: 11 points (~3 days of focused TDD work)
estimated_days: 3
risk: HIGH
# Risk justification (upgraded from MEDIUM in v1.0):
#   The primary risk is the Cyberint cookie_roundtrip auth path — the pipeline executor
#   currently only injects Authorization: Bearer {token} in build_request(). For
#   CookieRoundtrip, the auth token must be delivered as a Cookie header, not a Bearer
#   header. Two design options exist (see §Cyberint Cookie Auth Design). The recommended
#   approach (TOML login-step) requires a new FetchStep that captures Set-Cookie response
#   headers, which is beyond the current PipelineExecutor step model (steps only extract
#   response bodies via JSONPath). This makes Option B (pipeline-level auth-type-aware
#   dispatch) the production-grade choice despite being more invasive. See risk mitigations.
acceptance_criteria_count: 12
red_gate_tests: 5
estimated_passes: "3-4 LOCAL adversary passes"
holdout_scenarios: []
assumption_validations: []
risk_mitigations:
  - "Crate boundary: SpecDrivenSensorAdapter MUST live in prism-bin (NOT prism-sensors). prism-sensors
    must not import prism-spec-engine per ADR-023 §D3 Forbidden Dependencies. The struct is defined
    in crates/prism-bin/src/spec_driven_adapter.rs where both prism-sensors and prism-spec-engine
    are already workspace deps."
  - "BearerStatic auth: SpecDrivenSensorAdapter for Armis/Claroty holds an enum-strategy field
    (AuthStrategy::BearerStatic { token_from_sensor_auth: bool }) and constructs BearerStaticAuthProvider
    at fetch() call time from the SensorAuth argument. Token is not held at construction time."
  - "CookieRoundtrip auth: PipelineExecutor::build_request must be amended to check spec.auth_type.
    When AuthType::CookieRoundtrip, inject Cookie header (cyberint_session={token}) instead of
    Authorization: Bearer {token}. The AuthProvider for Cyberint performs the POST /login step
    and returns the session token value (without the cookie name prefix). CookieLoginAuthProvider
    is the production implementation; it calls POST /login, parses Set-Cookie: cyberint_session=,
    and returns the token string."
  - "reqwest::Client timeout: SpecDrivenSensorAdapter::new() constructs reqwest::Client with
    .timeout(Duration::from_secs(30)) per CLAUDE.md conventions. Missing this is a P2 finding."
  - "ADR-028 amendment: The §D10 note on cookie_roundtrip auth must be expanded to document
    the header-injection design decision (not login-step TOML approach). See §Cyberint Cookie Auth
    Design for the architect's recommendation and rationale."
inputs:
  - "crates/prism-bin/src/boot.rs"
  - "crates/prism-sensors/src/lib.rs"
  - "crates/prism-sensors/src/traits.rs"
  - "crates/prism-spec-engine/src/pipeline.rs"
  - "crates/prism-spec-engine/src/auth_provider.rs"
  - "crates/prism-spec-engine/src/spec_parser.rs"
  - "crates/prism-dtu-cyberint/src/routes/auth.rs"
  - "crates/prism-dtu-cyberint/src/routes/alerts.rs"
  - ".factory/specs/behavioral-contracts/BC-2.01.013-datasource-trait-adapter-pattern.md"
  - ".factory/specs/behavioral-contracts/BC-2.11.005-ephemeral-materialization.md"
  - ".factory/specs/behavioral-contracts/BC-2.06.014-instance-identity-resolution-at-fanout.md"
  - ".factory/specs/behavioral-contracts/BC-2.22.001-boot-orchestration.md"
  - ".factory/specs/architecture/decisions/ADR-022-production-runtime-wiring.md"
  - ".factory/specs/architecture/decisions/ADR-023-plugin-only-sensor-architecture.md"
  - ".factory/specs/architecture/decisions/ADR-028-toml-spec-grounding-vs-dtu-routes.md"
  - ".factory/specs/architecture/decisions/ADR-029-multi-tenant-sensor-endpoint-overrides.md"
  - ".factory/proposals/E2E-DEMO-WIRING-PLAN.md"
  - ".factory/stories/S-CONFIG-MULTI-TENANT-OVERRIDE-001-per-org-sensor-endpoint-overlay-loading.md"
  - ".factory/stories/PLUGIN-MIGRATION-001-A-delete-4-named-auth-modules-and-replace-init-registry-for-org.md"
  - ".factory/stories/PLUGIN-MIGRATION-001-E-crowdstrike-oauth2-refresh-on-401-prx-wasm-plugin.md"
  - ".factory/semport/poller-express/poller-express-broad-sweep.md"
input-hash: null
traces_to: []
cycle: "v1.0.0-brownfield"
phase: 3
---

# S-DEMO-001 v1.2 — prism-bin: SpecDrivenSensorAdapter + Boot Step 9A (closes GAP-002-A)

**Story ID:** S-DEMO-001
**Status:** draft
**Version:** v1.2
**Wave:** 5
**Priority:** P0
**Points:** 11

---

## Origin

New story required to close GAP-002-A per architect proposal E2E-DEMO-WIRING-PLAN.md §2.
The comment at `boot.rs` in the GAP-002-A region named `S-5.04-SENSOR-HEALTH-ADAPTER-DISPATCH` as the
target, but that story ID does not exist. GAP-002-A is independent of S-5.04; it must
be closed here first. User scope decision 2026-05-29: all 4 sensors (CrowdStrike + Armis +
Claroty + Cyberint).

**Auth model (v1.1 correction from v1.0):**
- CrowdStrike: WASM `crowdstrike-oauth2.prx` plugin path via `PluginAuthProvider` (held at construction).
- Armis: `bearer_static` auth via `BearerStaticAuthProvider` constructed per-fetch from `SensorAuth` arg.
- Claroty: `bearer_static` auth via `BearerStaticAuthProvider` — same as Armis.
- Cyberint: `cookie_roundtrip` auth via `CookieLoginAuthProvider` (new) + `PipelineExecutor::build_request`
  amendment to inject `Cookie: cyberint_session={token}` instead of `Authorization: Bearer`.
  NOT `bearer_static` as incorrectly stated in v1.0.

v1.0 §Origin was incorrect: it stated "Armis/Claroty/Cyberint use bearer_static". Cyberint uses
`auth_type = "cookie_roundtrip"` per D-737 LOCKED (ADR-028 §D2; TOML spec wins per CLAUDE.md §SoT #7).

---

## Narrative

As the Prism platform engineering team, I want `PipelineExecutor::execute()` bridged to the
`AdapterRegistry` via a `SpecDrivenSensorAdapter` struct, so that every sensor spec loaded at
boot produces a registered adapter and `tool_query "FROM crowdstrike_detections LIMIT 5"` (or
any of the other 3 sensors) returns real Arrow data from the DTU clone rather than `AdapterNotFound`.

---

## Story-Level Goal

After this story merges:
1. `crates/prism-bin/src/spec_driven_adapter.rs` exists with a `SpecDrivenSensorAdapter` struct
   implementing `dyn SensorAdapter` by delegating to `PipelineExecutor::execute()`.
2. Boot step 9A in `boot.rs` iterates the `ResolvedSensorSpec` map (from S-CONFIG-MULTI-TENANT-OVERRIDE-001)
   and registers one `SpecDrivenSensorAdapter` per (OrgId, SensorId) pair.
3. `fan_out()` in `prism-query/src/materialization.rs` returns live Arrow batches for all 4 sensors.
4. GAP-002-A comment in boot.rs is removed and replaced with a working implementation.
5. `BearerStaticAuthProvider` is implemented for Armis + Claroty (bearer_static sensors).
6. `CookieLoginAuthProvider` is implemented for Cyberint (cookie_roundtrip sensor).
7. `PipelineExecutor::build_request` is amended to inject `Cookie` header when `spec.auth_type == CookieRoundtrip`.

---

## Behavioral Contracts

| BC ID | Title |
|-------|-------|
| BC-2.01.013 | DataSource Trait Eliminates Per-Sensor Code Duplication |
| BC-2.11.005 | Ephemeral Materialization — Fan-Out, Normalize, Arrow RecordBatch, DataFusion MemTable |
| BC-2.06.014 | Instance Identity Resolution at Fanout — (org_id, sensor_id) Tuple Resolves to ResolvedSensorSpec |
| BC-2.22.001 | Boot Orchestration — Sequencing, Exit-Code Map, and Pre-Traffic Gate |

---

## Cyberint Cookie Auth Design (OQ-6 Resolution)

### Background: poller-express reference behavior

The reference Go implementation (`poller-express`) injects the Cyberint API key as a cookie
named `access_token` on every HTTP request via a `cookieTransport` (custom `http.RoundTripper`).
There is NO actual round-trip login step in poller-express — the name `cookie_roundtrip` is
misleading. The API key is static and injected directly as a cookie value on every request
without any prior authentication exchange.

The DTU clone (`prism-dtu-cyberint`) implements a DIFFERENT model: it requires a `POST /login`
step that returns `Set-Cookie: cyberint_session={uuid}`. Subsequent requests must include this
session token as `Cookie: cyberint_session={token}`. The DTU uses `cyberint_session` as the
cookie name, not `access_token`.

### Cookie name reconciliation: `access_token` vs `cyberint_session`

| Source | Cookie name | Mechanism |
|--------|-------------|-----------|
| poller-express (real API reference) | `access_token` | Static API key injected as cookie; no login step |
| DTU clone (`prism-dtu-cyberint`) | `cyberint_session` | UUID session token issued by POST /login; per-session |

The two models are intentionally different: the real API uses a static key; the DTU simulates
a stateful session to test cookie handling behavior more faithfully. For the demo (which runs
against the DTU clone), the DTU model governs.

**Decision for S-DEMO-001:** The `CookieLoginAuthProvider` performs `POST {base_url}/login` and
parses `Set-Cookie: cyberint_session={token}` from the response. It returns the token string
(not the full `cyberint_session={token}` form — just the value). The `PipelineExecutor::build_request`
amendment injects it as `Cookie: cyberint_session={token}`.

For production readiness (real Cyberint API): a future story will implement a second mode —
`StaticCookieAuthProvider` — that injects the API key as `Cookie: access_token={api_key}` without
any login step. This maps to the poller-express behavior for the real API. The two modes are
controlled by a new TOML field `auth_cookie_name` (or distinguished by a future `auth_type`
value `cookie_static`). This is NOT in-scope for S-DEMO-001 — the demo only needs to work
against the DTU clone.

**Action item (follow-up ADR, not blocking S-DEMO-001):** ADR-028 should be amended with §D12
to document the real-API vs DTU cookie model divergence and the path to production-grade real
Cyberint auth. The architect will amend ADR-028 §D12 as part of this story's spec updates.

### Pipeline-level fix: build_request must be auth-type-aware

**Root cause discovered:** `PipelineExecutor::build_request` (in `prism-spec-engine/src/pipeline.rs`)
currently injects ALL auth tokens as `Authorization: Bearer {token}`, regardless of `spec.auth_type`.
This is incorrect for `CookieRoundtrip` auth — the token must be injected as `Cookie: cyberint_session={token}`.

**Two design options:**

**Option A (TOML login-step approach):** Add a `login` FetchStep to the Cyberint TOML spec that
performs `POST /login` and captures the `Set-Cookie` header into a step variable. Subsequent steps
reference that variable as the cookie value. This requires PipelineExecutor to extract response
headers (not just JSON body via JSONPath) into step variables — a capability it currently lacks.

**Option B (pipeline auth-type-aware dispatch):** Amend `build_request` to accept `&SensorSpec`
(or `AuthType`) and dispatch the auth header based on auth_type:
- `BearerStatic`, `Oauth2ClientCredentials`, `CustomViaPlugin`: `Authorization: Bearer {token}`
- `CookieRoundtrip`: `Cookie: cyberint_session={token}`
- `ApiKey`: `{key_header}: {token}` (future, not in-scope)

`CookieLoginAuthProvider` performs the login step and returns the session token. The pipeline
calls `CookieLoginAuthProvider::acquire_token` before the steps loop (per the existing eager
token acquisition pattern) and injects it as a cookie on every step request.

**Architect recommendation: Option B.**

Option A requires a non-trivial PipelineExecutor capability extension (header capture into step
vars). Option B is a minimal targeted change to `build_request` that respects the existing
auth-type-aware design intent of the `AuthType` enum. The `AuthType` enum already models all
four variants; `build_request` ignoring auth_type was an implementation gap, not an architectural
decision. Closing it in Option B is the production-grade fix.

**Option B change surface:**
- `build_request` function signature gains `auth_type: &AuthType` (or `spec: &SensorSpec`).
- `issue_request_with_retry` passes `spec` down to `build_request` (already passes `spec` for error reporting).
- `CookieLoginAuthProvider` implements `AuthProvider`: calls `POST {spec.base_url}/login`, parses
  `Set-Cookie: cyberint_session={value}`, returns `AuthToken::new(value)`.
- On 401: the existing 401-retry logic re-calls `auth_provider.acquire_token` (which re-logins).
  This is correct for cookie auth — a 401 means the session expired; re-login gets a fresh token.

**Crate ownership note:** `build_request` is in `prism-spec-engine`. `CookieLoginAuthProvider` is
also a new type in `prism-spec-engine/src/auth_provider.rs` (not in `prism-bin`). It is a
production implementation of `AuthProvider` used by `SpecDrivenSensorAdapter` when `auth_type ==
CookieRoundtrip`. Unlike `BearerStaticAuthProvider` (which lives in `prism-bin` because it
bridges the SensorAuth↔AuthProvider interface), `CookieLoginAuthProvider` makes HTTP calls and
belongs in `prism-spec-engine` near `PipelineExecutor`. Mark it `pub` (not feature-gated) — it is
a production type.

---

## OQ-1 Resolution: BearerStaticAuthProvider construction timing

**Question:** Should `BearerStaticAuthProvider` be constructed per-fetch (Option A) vs. held as
an enum-strategy field on `SpecDrivenSensorAdapter` (Option B)?

**Decision: Option A (per-fetch construction from SensorAuth arg).**

Rationale: At boot step 9A, the bearer token for Armis/Claroty is not known — it lives in the
credential store and is resolved at query time when `SensorAuth::BearerStatic { token }` arrives
at `SpecDrivenSensorAdapter::fetch()`. The adapter cannot hold the token at construction time;
it must extract it from the `&dyn SensorAuth` argument at call time. Per-fetch construction is
cheap (struct with one String) and avoids the complexity of an enum-strategy field.

**Implementation:** `SpecDrivenSensorAdapter::fetch()` matches on `auth: &dyn SensorAuth`:
- Downcast to `SensorAuth::BearerStatic { token }`: construct `BearerStaticAuthProvider::new(token)`.
- Auth strategy from construction time is `PluginAuth`: use the held `Arc<PluginAuthProvider>`.
- Auth strategy from construction time is `CookieAuth`: use the held `Arc<CookieLoginAuthProvider>`.

The `SpecDrivenSensorAdapter` holds an enum `AdapterAuthStrategy`:
```rust
enum AdapterAuthStrategy {
    Plugin(Arc<dyn AuthProvider>),   // CrowdStrike: held PluginAuthProvider
    BearerStatic,                    // Armis/Claroty: extracted at fetch() from SensorAuth
    CookieLogin(Arc<dyn AuthProvider>), // Cyberint: held CookieLoginAuthProvider
}
```

This is cleaner than per-call downcasting on `&dyn SensorAuth`.

---

## OQ-2 Resolution: ResolvedSensorSpec iteration type at boot step 9A

**Question:** Is the iteration map keyed by `OrgSlug` or `OrgId`? Is translation needed?

**Decision:** Read the `S-CONFIG-MULTI-TENANT-OVERRIDE-001` story's output types before
implementing boot step 9A. Based on current prism-spec-engine types, `ResolvedSensorSpec` is
produced from TOML overlays that are keyed by `OrgSlug` (directory name under `customers/`).
The `AdapterRegistry` is keyed by `OrgId`. Therefore, boot step 9A must translate:

```rust
for (org_slug, resolved_specs) in resolved_sensor_specs.iter() {
    let org_id = org_registry.id_for_slug(org_slug)
        .ok_or_else(|| BootError::OrgNotFound { slug: org_slug.clone() })?;
    for (sensor_id, spec) in resolved_specs.iter() {
        let adapter = build_adapter(spec, &plugin_auth_providers, &http_client)?;
        adapter_registry.register(org_id, sensor_id.clone(), Arc::new(adapter));
    }
}
```

The `OrgRegistry::id_for_slug(slug)` method must exist (from S-CONFIG-MULTI-TENANT-OVERRIDE-001).
If it does not exist yet, add it to `OrgRegistry` in the same PR (production-grade principle).

**Type verification required:** Before implementing, read the actual output type of
S-CONFIG-MULTI-TENANT-OVERRIDE-001's overlay loader function to confirm the map key type.
If the map is already keyed by `OrgId`, the `id_for_slug` translation is not needed.

---

## Acceptance Criteria

### AC-001: SpecDrivenSensorAdapter delegates to PipelineExecutor for CrowdStrike (plugin auth path)
Given: CrowdStrike sensor spec loaded at boot with `auth_plugin = "crowdstrike-oauth2"` and a
corresponding `PluginAuthProvider` constructed at step 7.5b.
When: `SpecDrivenSensorAdapter::fetch()` is called for CrowdStrike.
Then: The adapter calls `PipelineExecutor::execute()` passing its held `Arc<PluginAuthProvider>`
(the `SensorAuth` arg is ignored for plugin-authed sensors); returns `Vec<RecordBatch>` in
OCSF-normalized shape.
(traces to BC-2.01.013 postcondition 4)
Red Gate test: `test_BC_2_01_013_spec_driven_adapter_crowdstrike_delegates_to_pipeline_executor`

### AC-002: SpecDrivenSensorAdapter delegates for bearer_static sensors (Armis/Claroty)
Given: Armis or Claroty sensor spec loaded at boot (auth_type is `bearer_static`, no `auth_plugin` field).
When: `SpecDrivenSensorAdapter::fetch()` is called with a `SensorAuth::BearerStatic { token }` arg.
Then: The adapter constructs a `BearerStaticAuthProvider` from the token in the `SensorAuth` arg,
calls `PipelineExecutor::execute()` with it, and returns `Vec<RecordBatch>`.
(traces to BC-2.01.013 postcondition 4)
Red Gate test: `test_BC_2_01_013_spec_driven_adapter_bearer_static_extracts_token_from_sensor_auth`

### AC-003: SpecDrivenSensorAdapter delegates for cookie_roundtrip sensor (Cyberint)
Given: Cyberint sensor spec loaded at boot (auth_type is `cookie_roundtrip`, no `auth_plugin` field);
DTU clone running and accepting `POST /login` → `Set-Cookie: cyberint_session={token}`.
When: `SpecDrivenSensorAdapter::fetch()` is called for Cyberint.
Then: The adapter uses its held `CookieLoginAuthProvider`, which (a) calls `POST {base_url}/login`,
(b) parses the `Set-Cookie` response header, (c) extracts the value of the cookie named
`cyberint_session` (NOT `access_token` — that is the real Cyberint API cookie name, which differs
from the DTU's session cookie; see §Cyberint Cookie Auth Design for the reconciliation),
(d) returns the token string.
The pipeline injects it as `Cookie: cyberint_session={token}` (NOT `Authorization: Bearer`).
The response contains `Vec<RecordBatch>` from the DTU clone.
(traces to BC-2.01.013 postcondition 4; closes cookie_roundtrip gap in pipeline)
Red Gate test: `test_BC_2_01_013_spec_driven_adapter_cyberint_cookie_auth_injects_cookie_header`

### AC-004: Boot step 9A registers exactly N adapters (N = sum of per-org × per-sensor specs)
Given: `spec_catalog` has M resolved sensor specs for each of K orgs.
When: Boot step 9A runs.
Then: `AdapterRegistry` contains exactly sum(org_sensors) entries (one per (OrgId, SensorId) pair).
The `boot.step9a.adapter_registry_populated` structured event is emitted with fields `sensor_count`
and `org_count` per BC-2.16.002 catalog row added in this story.
(traces to BC-2.22.001 postcondition)
Red Gate test: `test_BC_2_22_001_boot_step9a_registers_correct_adapter_count`

### AC-005: Adapter registration is per-org with correct overlay applied
Given: demo-org has a `customers/demo-org/crowdstrike.sensor.toml` overlay setting
`base_url = "http://127.0.0.1:<PORT>"`.
When: Boot step 9A constructs the CrowdStrike adapter for demo-org.
Then: The adapter's internal `PipelineExecutor` uses the overlay `base_url` (from the
`ResolvedSensorSpec`), not the production base URL from the type spec.
(traces to BC-2.06.014 precondition 1)
Red Gate test: `test_BC_2_06_014_boot_step9a_uses_resolved_spec_overlay_url`

### AC-006: Empty spec_catalog → AdapterRegistry remains empty; no error
Given: `spec_catalog` is empty at boot (no TOML specs loaded).
When: Boot step 9A runs.
Then: `AdapterRegistry` is empty; no error is emitted; boot continues to step 9 (MCP server start).
(traces to BC-2.22.001 postcondition)

### AC-007: AdapterRegistry::get(org_id, sensor_id) returns adapter for registered pairs
Given: Boot step 9A completed.
When: `AdapterRegistry::get(org_id, sensor_id)` is called for any (org, sensor) pair present in
both `spec_catalog` and `org_registry`.
Then: Returns `Some(Arc<dyn SensorAdapter>)` — a `SpecDrivenSensorAdapter` for that (org, sensor).
(traces to BC-2.11.005 precondition)

### AC-008: BearerStaticAuthProvider correctly converts SensorAuth to AuthHeader
Given: A `SpecDrivenSensorAdapter` for a bearer_static sensor is called with `SensorAuth::BearerStatic { token: "test-token-abc" }`.
When: The adapter internally constructs `BearerStaticAuthProvider` and calls `PipelineExecutor::execute()`.
Then: The HTTP request emitted by `PipelineExecutor` carries `Authorization: Bearer test-token-abc`
header per the BearerStatic path in `build_request`.
(traces to BC-2.01.013 precondition)

### AC-009: CookieLoginAuthProvider injects Cookie header, not Authorization Bearer
Given: A `SpecDrivenSensorAdapter` for Cyberint is called with a running DTU clone.
When: `CookieLoginAuthProvider::acquire_token()` is called; the DTU clone's `POST /login` responds with
`Set-Cookie: cyberint_session=some-uuid`.
Then:
(a) `CookieLoginAuthProvider` parses the cookie name `cyberint_session` from the `Set-Cookie` header
    (the DTU uses `cyberint_session`; the real Cyberint API uses `access_token` — a different cookie
    name; see §Cyberint Cookie Auth Design. The implementation must parse `cyberint_session` for the
    DTU demo path).
(b) The HTTP request to `GET /api/v1/alerts` carries `Cookie: cyberint_session=some-uuid`,
    NOT `Authorization: Bearer some-uuid`.
(closes the `build_request` cookie-injection gap per §Cyberint Cookie Auth Design)

**Implementation constraint:** `CookieLoginAuthProvider` must extract the `cyberint_session` cookie
from the `Set-Cookie` header value. The cookie parser must handle the standard `Set-Cookie` format:
`cyberint_session=<value>; Path=/; HttpOnly`. Extracting `<value>` by splitting on `=` and then
on `;` is sufficient (no exotic cookie attributes in the DTU response).

### AC-010: Adapter fetch returns OCSF-normalized Arrow RecordBatches
Given: A `SpecDrivenSensorAdapter::fetch()` call is made for any of the 4 sensors.
When: The `PipelineExecutor` successfully fetches and normalizes data.
Then: The returned `Vec<RecordBatch>` contains at least the OCSF hot fields: `category_uid`,
`class_uid`; the `_sensor` virtual column identifies the sensor_id; no raw API response fields
transit the return value without OCSF normalization.
(traces to BC-2.11.005 postcondition)

### AC-011: No `todo!()` or `unimplemented!()` in adapter, boot step 9A, or CookieLoginAuthProvider (POL-12)
Given: The implementation is complete and all 4 sensor paths are exercised by tests.
When: The codebase is searched for `todo!()` or `unimplemented!()` in the new files.
Then: Zero occurrences found.
(traces to BC-2.22.001 invariant)

### AC-012: Adapter handles plugin-auth-failure (double-401 → AuthExpired error)
Given: CrowdStrike DTU clone returns 401 on the initial fetch AND on the refresh attempt.
When: `SpecDrivenSensorAdapter::fetch()` calls `PipelineExecutor::execute()`.
Then: The adapter returns `Err(SpecEngineError::AuthRefreshFailed)` (or the canonical error code
from the error taxonomy); no panic; the response envelope wraps the error correctly per BC-2.10.007.
(traces to BC-2.01.013 error case)

---

## Architecture Compliance Rules

| Rule | Source | Enforcement |
|------|--------|-------------|
| `SpecDrivenSensorAdapter` MUST live in `prism-bin`, NOT `prism-sensors` | ADR-023 §D3 Forbidden Dependencies | Build fails if `prism-sensors/Cargo.toml` gains dep on `prism-spec-engine` |
| `CookieLoginAuthProvider` lives in `prism-spec-engine/src/auth_provider.rs` | Crate cohesion — it makes HTTP calls, belongs near PipelineExecutor | Not in `prism-bin` |
| `BearerStaticAuthProvider` lives in `prism-bin/src/spec_driven_adapter.rs` | It bridges `SensorAuth` (prism-sensors) ↔ `AuthProvider` (prism-spec-engine); only prism-bin imports both | ADR-023 §Permitted Patterns |
| `prism-bin` may import both `prism-sensors` and `prism-spec-engine` | ADR-023 §Permitted Patterns | Existing workspace deps already present in prism-bin |
| `SensorAdapter::fetch(&dyn SensorAuth)` arg ignored for plugin-authed sensors | ADR-028 §D10 | Documented in impl with inline comment citing ADR-028 §D10 |
| `reqwest::Client` MUST set `.timeout(Duration::from_secs(30))` | CLAUDE.md Conventions | Adversary probes for missing timeout on every pass |
| `boot.step9a.adapter_registry_populated` MUST have a BC-2.16.002 catalog row | SAP-1 (standing probe) | Adversary greps `event_type =` on every pass |
| Boot step 9A MUST appear between steps 7.5b and 9 in ADR-022 §B table | BC-2.22.001 + ADR-022 | ADR-022 §B amendment required in same PR |
| `PipelineExecutor::build_request` MUST dispatch header by auth_type | This story (§Cyberint Cookie Auth Design) | CookieRoundtrip → Cookie header; BearerStatic → Authorization: Bearer |

---

## Library & Framework Requirements

| Library | Version | Purpose |
|---------|---------|---------|
| `prism-spec-engine` (workspace) | current workspace path | PipelineExecutor, AuthProvider trait, CookieLoginAuthProvider |
| `prism-sensors` (workspace) | current workspace path | SensorAdapter trait and SensorAuth types |
| `reqwest` | workspace version | HTTP client inside PipelineExecutor + CookieLoginAuthProvider login step |
| `tokio` | workspace version | async runtime for PipelineExecutor::execute() |
| `arrow` | workspace version | RecordBatch return type |
| `tracing` | workspace version | boot.step9a.adapter_registry_populated event emission |

Version source: `Cargo.toml` workspace `[dependencies]` table. Do not pin versions independently.

---

## File Structure Requirements

| File | Action | Purpose |
|------|--------|---------|
| `crates/prism-bin/src/spec_driven_adapter.rs` | CREATE | New module: `SpecDrivenSensorAdapter` struct + `SensorAdapter` impl + `BearerStaticAuthProvider` wrapper |
| `crates/prism-bin/src/lib.rs` (or `main.rs`) | MODIFY | Add `mod spec_driven_adapter;` |
| `crates/prism-bin/src/boot.rs` | MODIFY | Add boot step 9A; remove GAP-002-A comment; emit event |
| `crates/prism-spec-engine/src/auth_provider.rs` | MODIFY | Add `CookieLoginAuthProvider` (production impl; NOT feature-gated) |
| `crates/prism-spec-engine/src/pipeline.rs` | MODIFY | Amend `build_request` to accept auth_type; inject Cookie header for CookieRoundtrip |
| `.factory/specs/behavioral-contracts/BC-2.16.002-*.md` | MODIFY | Add catalog row for `boot.step9a.adapter_registry_populated` event per SAP-1 obligation |
| `.factory/specs/architecture/decisions/ADR-022-*.md` | MODIFY | Add boot step 9A to §B sequencing table |
| `.factory/specs/architecture/decisions/ADR-023-*.md` | MODIFY | Add `SpecDrivenSensorAdapter` + `CookieLoginAuthProvider` to §Permitted Patterns list |
| `.factory/specs/architecture/decisions/ADR-028-*.md` | MODIFY | Add §D12: cookie_roundtrip production vs DTU design divergence |

---

## Tasks

1. **Read** `crates/prism-sensors/src/traits.rs` — understand `SensorAdapter::fetch()` and `SensorAuth` signatures.
2. **Read** `crates/prism-spec-engine/src/pipeline.rs` — understand `PipelineExecutor::execute()` + `build_request` function.
3. **Read** `crates/prism-spec-engine/src/auth_provider.rs` — understand `AuthProvider` trait and existing implementations.
4. **Read** `crates/prism-spec-engine/src/spec_parser.rs` — understand `AuthType` enum variants; confirm `CookieRoundtrip` variant exists.
5. **Read** `crates/prism-dtu-cyberint/src/routes/auth.rs` and `alerts.rs` — understand the DTU's `POST /login` → `Set-Cookie: cyberint_session` pattern and the `extract_session_token()` function.
6. **Read** `crates/prism-bin/src/boot.rs` — locate step 7.5b and the GAP-002-A comment. Understand `ResolvedSensorSpec` map structure.
7. **Read** `S-CONFIG-MULTI-TENANT-OVERRIDE-001` story — confirm the exact output type of the overlay loader (map key type: `OrgSlug` vs `OrgId`).
8. **Amend** `crates/prism-spec-engine/src/pipeline.rs` — refactor `build_request` to accept `auth_type: &AuthType` and dispatch:
   - `CookieRoundtrip` → `Cookie: {sensor's cookie name}={token}` (cookie name: `cyberint_session`)
   - All other variants → `Authorization: Bearer {token}` (existing behavior unchanged)
   - Update all callers of `build_request` to pass `spec.auth_type` (two call sites: `issue_request_with_retry`).
9. **Implement** `CookieLoginAuthProvider` in `prism-spec-engine/src/auth_provider.rs`:
   - Fields: `login_url: String` (constructed as `{spec.base_url}/login`), `http_client: reqwest::Client`.
   - Constructor `new(base_url: &str) -> Self` — builds with `.timeout(Duration::from_secs(30))`.
   - `AuthProvider::acquire_token()` — POSTs `{}` to `{login_url}`, extracts `cyberint_session` value from `Set-Cookie` header, returns `AuthToken::new(session_token)`.
   - Error path: missing `Set-Cookie` header → `SpecEngineError::AuthAcquisitionFailed`.
10. **Write stub** `crates/prism-bin/src/spec_driven_adapter.rs` with `todo!()` bodies (Red Gate setup).
11. **Write Red Gate tests** (see AC-001, AC-003, AC-004, AC-005 test names) — all must fail (RED) before implementation.
12. **Implement** `SpecDrivenSensorAdapter`:
    - Fields: `sensor_spec: Arc<ResolvedSensorSpec>`, `auth_strategy: AdapterAuthStrategy`, `executor: Arc<PipelineExecutor>`, `http_client: reqwest::Client`.
    - `AdapterAuthStrategy` enum: `Plugin(Arc<dyn AuthProvider>)`, `BearerStatic`, `CookieLogin(Arc<dyn AuthProvider>)`.
    - `SensorAdapter::fetch()` — dispatch by `auth_strategy`; for BearerStatic, extract token from `&dyn SensorAuth` arg and construct `BearerStaticAuthProvider` per-call.
    - `SensorAdapter::sensor_type()` — return `SensorId::from(self.sensor_spec.sensor_id.as_str())`.
13. **Implement** `BearerStaticAuthProvider` in `crates/prism-bin/src/spec_driven_adapter.rs`:
    - Thin wrapper implementing `AuthProvider`; stores the bearer token string; returns it via `acquire_token`.
14. **Implement boot step 9A** in `boot.rs`:
    - After step 7.5b: iterate `resolved_sensor_specs` map (translate OrgSlug → OrgId if needed).
    - For each `(org_id, sensor_id, resolved_spec)`: inspect `resolved_spec.auth_type`:
      - `CustomViaPlugin`: look up `plugin_auth_providers.get(&sensor_id)` → `AdapterAuthStrategy::Plugin`.
      - `BearerStatic`: → `AdapterAuthStrategy::BearerStatic`.
      - `CookieRoundtrip`: construct `CookieLoginAuthProvider::new(&resolved_spec.base_url)` → `AdapterAuthStrategy::CookieLogin`.
      - Others: log E-SPEC-012 (auth type mismatch); skip.
    - Register adapters; emit `boot.step9a.adapter_registry_populated` event.
15. **Amend BC-2.16.002** — add catalog row for `boot.step9a.adapter_registry_populated` per SAP-1.
16. **Amend ADR-022 §B** — add boot step 9A to the sequencing invariant table.
17. **Amend ADR-023 §Permitted Patterns** — add `SpecDrivenSensorAdapter`, `CookieLoginAuthProvider`, `BearerStaticAuthProvider` patterns.
18. **Amend ADR-028** — add §D12: document real-API vs DTU cookie model divergence; note `access_token` (real API) vs `cyberint_session` (DTU); path to production-grade real Cyberint auth via `StaticCookieAuthProvider` in a future story.
19. **Run tests**: `just iter prism-bin` and `just iter prism-spec-engine` — all Red Gate tests GREEN.
20. **Run** `just check` — final pre-push gate.

---

## Previous Story Intelligence

- **S-CONFIG-MULTI-TENANT-OVERRIDE-001** (depends_on): Must merge first. Provides the `ResolvedSensorSpec` map and per-org overlay loading (ADR-029). Read that story's output types before writing step 9A.
- **PLUGIN-MIGRATION-001-A** (depends_on): Must merge first. Deletes legacy hardcoded adapter construction. After it merges, `AdapterRegistry` is unambiguously empty at boot step 9.
- **PLUGIN-MIGRATION-001-E** (depends_on): Must merge first. Wires `crowdstrike-oauth2.prx` WASM plugin to `PluginAuthProvider` construction at boot step 7.5b.
- **S-PLUGIN-PREREQ-B** (merged): Delivered `PipelineExecutor::execute()`. Read its implementation before writing the adapter.
- **S-5.01-FOLLOWUP-MCP-BOOT** (merged): Delivered the complete rmcp MCP server. Boot step 9 (MCP server start) is fully implemented.
- **PLUGIN-MIGRATION-001-D** (merged): Delivered the 4 TOML sensor specs. Cyberint spec has `auth_type = "cookie_roundtrip"` (D-737 LOCKED).

---

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Sensor spec has `auth_type = "cookie_roundtrip"` but DTU login endpoint returns non-200 | `CookieLoginAuthProvider::acquire_token` returns `SpecEngineError::AuthAcquisitionFailed`; adapter not registered |
| EC-002 | `POST /login` succeeds but `Set-Cookie` header is absent or has no `cyberint_session` cookie | `CookieLoginAuthProvider::acquire_token` returns `AuthAcquisitionFailed` with detail "missing cyberint_session cookie" |
| EC-003 | org_registry has orgs but spec_catalog is empty | Boot step 9A produces 0 registrations; no error; boot continues to step 9 |
| EC-004 | PluginAuthProvider fails at step 7.5b | Step 7.5b handles; step 9A skips sensors with missing auth providers |
| EC-005 | ResolvedSensorSpec has per-org base_url overlay but CookieLoginAuthProvider not updated | CRITICAL: CookieLoginAuthProvider MUST use `resolved_spec.base_url` (with overlay applied), not the type spec base_url. Verify at AC-005. |
| EC-006 | Double-401 from sensor API during SpecDrivenSensorAdapter::fetch() | PipelineExecutor handles the refresh; on second 401, returns SpecEngineError::AuthRefreshFailed. Adapter propagates this. |
| EC-007 | Sensor spec has unsupported `auth_type` (e.g., `api_key`) — not yet implemented in SpecDrivenSensorAdapter | Boot logs E-SPEC-012 (auth type mismatch); adapter NOT registered for that sensor; boot continues |
| EC-008 | CookieLoginAuthProvider 401-retry: Cyberint session cookie expires mid-pipeline | Existing `issue_request_with_retry` calls `acquire_token` again (re-logins); fresh session cookie returned; pipeline retries once. |

---

## Spec Updates Required (same PR)

| Document | Amendment |
|----------|-----------|
| ADR-022 §B | Add boot step 9A (`spec_driven_adapter_registry_populate`) between step 7.5b and step 9 |
| ADR-023 §Permitted Patterns | Add `SpecDrivenSensorAdapter`, `CookieLoginAuthProvider`, `BearerStaticAuthProvider` pattern notes |
| ADR-022 §F | Update comment that `adapter_registry` is empty at step 9 — post-story it is populated |
| ADR-028 §D12 (NEW) | Document real-API `access_token` cookie vs DTU `cyberint_session` cookie divergence; path to production-grade `StaticCookieAuthProvider` |
| BC-2.16.002 Structured Event Catalog | Add `boot.step9a.adapter_registry_populated` row with `sensor_count`, `org_count` fields |

---

## Token Budget Estimate

| Context source | Estimated tokens |
|----------------|-----------------|
| This story spec | ~5,000 |
| BC files (4 BCs) | ~6,000 |
| crates/prism-bin/src/boot.rs (large) | ~12,000 |
| crates/prism-spec-engine/src/pipeline.rs (to amend build_request) | ~8,000 |
| crates/prism-spec-engine/src/auth_provider.rs | ~4,000 |
| crates/prism-spec-engine/src/spec_parser.rs (AuthType enum) | ~2,000 |
| crates/prism-sensors/src/traits.rs | ~2,000 |
| crates/prism-dtu-cyberint/src/routes/auth.rs + alerts.rs | ~3,000 |
| ADR-022, ADR-023, ADR-028, ADR-029 (relevant sections) | ~8,000 |
| S-CONFIG-MULTI-TENANT-OVERRIDE-001 story (type reference) | ~4,000 |
| PLUGIN-MIGRATION-001-A + 001-E stories (context) | ~6,000 |
| Test outputs (cargo nextest) | ~2,000 |
| **Total estimate** | **~62,000 tokens (~24% of 256K context)** |

Within the 20-30% budget. Single-story delivery is viable; consider splitting off pipeline amendment
if context pressure is felt during implementation.

---

## Changelog

| Version | Date | Author | Notes |
|---------|------|--------|-------|
| 1.0 | 2026-05-29 | story-writer | Initial draft — all 4 sensors scope per user 2026-05-29 decision |
| 1.1 | 2026-05-29 | architect | Corrected Cyberint auth model (cookie_roundtrip ≠ bearer_static); resolved OQ-1, OQ-2, OQ-6; added CookieLoginAuthProvider design; amended build_request pipeline gap; added new AC-003/AC-009/AC-012; revised points 8→11; risk MEDIUM→HIGH |
| 1.2 | 2026-05-29 | architect | AC-003/AC-009: tightened `cyberint_session` cookie name specification per fidelity audit (POLLER-DTU-FIDELITY-AUDIT-2026-05-29). Added Set-Cookie parse constraint to AC-009 (cookie name `cyberint_session` not `access_token` for DTU demo path). Cross-poller audit surfaced Gap-CS-001/CL-002/CL-003/CL-004/CL-005; TOML fixes applied to crowdstrike.sensor.toml and claroty.sensor.toml in same burst. |
