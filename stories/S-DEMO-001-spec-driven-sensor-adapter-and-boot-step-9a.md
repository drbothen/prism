---
document_type: story
story_id: S-DEMO-001
title: "prism-bin: SpecDrivenSensorAdapter + Boot Step 9A — Bridge PipelineExecutor to AdapterRegistry (closes GAP-002-A)"
wave: 5
epic_id: E-DEMO
priority: P0
status: draft
version: "1.12"
updated: "2026-07-24"
level: "L4"
producer: story-writer
revised_by: architect
timestamp: "2026-06-05T00:00:00Z"
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
  - S-DTU-CYBERINT-AUTH-FIDELITY-001    # Must merge first: corrects prism-dtu-cyberint to use
                                         # access_token cookie (not cyberint_session) and removes
                                         # the login step. This story's Cyberint auth implementation
                                         # (StaticCookieAuthProvider + access_token injection) requires
                                         # the corrected DTU to be running. See ADR-031 §D3.
blocks:
  - S-DEMO-002   # E2E smoke test cannot run without adapters in the registry.
  - S-5.04       # Sensor health checks call the registered adapter; health subsystem is
                 # downstream of GAP-002-A closure.
  - S-5.04-FIX-001  # Spec fix story depends on this story existing as the new S-5.04 dep.
points: 11
# Points justification (revised from v1.0's 8 pts):
#   v1.3 note: CookieLoginAuthProvider (login-step, 2 pts) is REPLACED by
#   StaticCookieAuthProvider (no login step, credential-store read, ~1 pt less complexity).
#   DTU code changes are out-of-scope (in S-DTU-CYBERINT-AUTH-FIDELITY-001).
#   Net: points UNCHANGED at 11 — StaticCookieAuthProvider simpler but must depend on
#   corrected DTU + additional test coverage for access_token cookie injection path.
#   - SpecDrivenSensorAdapter struct + SensorAdapter impl (plugin auth path): ~1.5 pts
#   - BearerStaticAuthProvider for Armis + Claroty (bearer_static auth type): ~1 pt
#   - StaticCookieAuthProvider for Cyberint (cookie_roundtrip, access_token, no login step): ~1.5 pts
#     (replaces CookieLoginAuthProvider; simpler but requires build_request dispatch change)
#   - PipelineExecutor build_request auth-type-aware dispatch (Cookie: access_token=): ~0.5 pts
#   - Boot step 9A loop + AdapterRegistry::register() calls: ~1.5 pts
#   - reqwest::Client construction with 30s timeout (AD-017 compliant): ~0.5 pts
#   - BC-2.16.002 catalog row for boot.step9a.adapter_registry_populated: ~0.5 pts
#   - Red Gate tests (5 required, see ACs): ~2 pts
#   - ADR-022 §B + ADR-023 §Permitted-Patterns amendment: ~0.5 pts
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
  - "CookieRoundtrip auth: `build_request` (module-level free function in `crates/prism-spec-engine/src/pipeline.rs`) must be amended to check spec.auth_type.
    When AuthType::CookieRoundtrip, inject Cookie header (access_token={token}) instead of
    Authorization: Bearer {token}. StaticCookieAuthProvider is the production implementation;
    it reads the api_key from the credential store and returns the api_key string.
    StaticCookieAuthProvider makes NO HTTP call during acquire_token (no POST /login step — per
    ADR-031 D1-b). Cookie name is access_token (real Cyberint API; NOT cyberint_session)."
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

# S-DEMO-001 v1.10 — prism-bin: SpecDrivenSensorAdapter + Boot Step 9A (closes GAP-002-A)

**Story ID:** S-DEMO-001
**Status:** draft
**Version:** v1.10
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

**Auth model (v1.3 correction from v1.2 — DTU=true-DTU principle, ADR-031):**
- CrowdStrike: WASM `crowdstrike-oauth2.prx` plugin path via `PluginAuthProvider` (held at construction).
- Armis: `bearer_static` auth via `BearerStaticAuthProvider` constructed per-fetch from `SensorAuth` arg.
- Claroty: `bearer_static` auth via `BearerStaticAuthProvider` — same as Armis.
- Cyberint: `cookie_roundtrip` auth via `StaticCookieAuthProvider` (new, replaces CookieLoginAuthProvider)
  + `build_request` amendment to inject `Cookie: access_token={token}`.
  NOT `cyberint_session` as incorrectly specified in v1.1/v1.2.

v1.1/v1.2 §Origin was incorrect on the Cyberint cookie name. It specified `cyberint_session` because
"the DTU uses that name." Under ADR-031 DTU=true-DTU principle (user directive 2026-05-29), the DTU
must conform to the real API — not the other way around. The real API uses `access_token`
(poller-express `cookieTransport`; no login step). The DTU is corrected in
S-DTU-CYBERINT-AUTH-FIDELITY-001 before this story is implemented. This story implements
`StaticCookieAuthProvider` (no HTTP calls at acquire_token time, no login step) and
injects `Cookie: access_token={api_key}` in `build_request`.

v1.0 §Origin was also incorrect: it stated "Armis/Claroty/Cyberint use bearer_static". Cyberint uses
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
6. `StaticCookieAuthProvider` is implemented for Cyberint (cookie_roundtrip sensor).
   This provider reads the API key from the credential store at acquire-token time; makes NO
   HTTP call during `acquire_token` (no login step); returns the raw API key as the token value.
   The DTU `prism-dtu-cyberint` has been corrected by S-DTU-CYBERINT-AUTH-FIDELITY-001 to
   accept `Cookie: access_token={api_key}` before this story is implemented.
7. `build_request` is amended to inject `Cookie: access_token={token}` when
   `spec.auth_type == CookieRoundtrip` (NOT `cyberint_session` — see ADR-031 §D3-b).

---

## Behavioral Contracts

| BC ID | Title |
|-------|-------|
| BC-2.01.013 | DataSource Trait Eliminates Per-Sensor Code Duplication |
| BC-2.11.005 | Ephemeral Materialization — Fan-Out, Normalize, Arrow RecordBatch, DataFusion MemTable |
| BC-2.06.014 | Instance Identity Resolution at Fanout — (org_id, sensor_id) Tuple Resolves to ResolvedSensorSpec |
| BC-2.22.001 | Boot Orchestration — Sequencing, Exit-Code Map, and Pre-Traffic Gate |

---

## Cyberint Cookie Auth Design (OQ-6 Resolution — v1.3 revised per ADR-031)

> **v1.3 REVISION:** This section is materially changed from v1.1/v1.2.
> v1.1/v1.2 specified `CookieLoginAuthProvider` (login step + `cyberint_session` cookie).
> That design is WRONG under ADR-031 DTU=true-DTU (user directive 2026-05-29).
> The correct design is `StaticCookieAuthProvider` + `access_token` cookie.
> The old design documentation is struck through below for traceability.

### Background: poller-express reference behavior (the ground truth)

The reference Go implementation (`poller-express`) injects the Cyberint API key as a cookie
named `access_token` on every HTTP request via a `cookieTransport` (custom `http.RoundTripper`).
There is NO login step. The API key IS the credential; it is injected directly as a cookie on
every request.

```go
func (t *cookieTransport) RoundTrip(req *http.Request) (*http.Response, error) {
    req.AddCookie(&http.Cookie{Name: "access_token", Value: t.apiKey})
    return http.DefaultTransport.RoundTrip(req)
}
```

### Cookie name: `access_token` (canonical)

| Source | Cookie name | Mechanism | Is canonical? |
|--------|-------------|-----------|---------------|
| poller-express (real API reference) | `access_token` | Static API key injected as cookie; no login step | YES — source of truth per ADR-031 |
| DTU clone v1 (pre-fix) | `cyberint_session` | UUID session token issued by POST /login; per-session | NO — DTU fidelity violation; CORRECTED by S-DTU-CYBERINT-AUTH-FIDELITY-001 |
| DTU clone v2 (post-fix) | `access_token` | Static API key accepted; no login step; per ADR-031 | YES — correct |

The v1.1/v1.2 decision that "the DTU model governs for the demo" is reversed by ADR-031.
The real API governs. The DTU must conform to the real API.

### Decision for S-DEMO-001 (v1.3)

`StaticCookieAuthProvider` is the Cyberint auth implementation:
- Reads the API key from the credential store at `acquire_token()` time (NOT at construction
  time per AD-017 — credentials never held at construction).
- Makes NO HTTP call during `acquire_token()`. No login step.
- Returns the raw API key string as the `AuthToken` value.

`build_request` injects the token as `Cookie: access_token={api_key}`.

Prerequisite: `prism-dtu-cyberint` must be corrected by `S-DTU-CYBERINT-AUTH-FIDELITY-001`
before this story is implemented. S-DEMO-001 depends_on that story.

### Pipeline-level fix: build_request must be auth-type-aware

**Root cause:** `build_request` injects ALL auth tokens as
`Authorization: Bearer {token}`, regardless of `spec.auth_type`. For `CookieRoundtrip`, the
token must be injected as `Cookie: access_token={token}`.

**Decision: pipeline auth-type-aware dispatch (unchanged from v1.1/v1.2 — method correct; cookie name corrected).**

- `build_request` signature gains `auth_type: &AuthType`.
- `issue_request_with_retry` passes `spec` down to `build_request`.
- `StaticCookieAuthProvider::acquire_token()` reads API key from credential store via the
  internal `PrismCredentialResolver`; returns it.
- On 401 (`AuthType::CookieRoundtrip`): NO retry. `issue_request_with_retry` returns
  `Err(SpecEngineError::CookieAuthFailed)` immediately — retry is provably futile for a static
  key (re-reading the same credential store will return the same invalid value).
  Cross-ref: BC-2.01.017 EC-017-002. (This is distinct from the CrowdStrike OAuth2 path, which
  DOES retry on 401 via token refresh — see AC-012 / EC-006.)

**Updated dispatch table:**

| AuthType | Header injected |
|----------|----------------|
| `CookieRoundtrip` | `Cookie: access_token={token}` |
| `BearerStatic` | `Authorization: Bearer {token}` |
| `Oauth2ClientCredentials` | `Authorization: Bearer {token}` |
| `CustomViaPlugin` | `Authorization: Bearer {token}` |

**Crate ownership:** `build_request` is in `prism-spec-engine`. `StaticCookieAuthProvider` is
a new type in `prism-spec-engine/src/auth_provider.rs`. Unlike `BearerStaticAuthProvider`
(which lives in `prism-bin` because it bridges `SensorAuth↔AuthProvider`), `StaticCookieAuthProvider`
is a pure-prism-spec-engine type that reads credentials and injects them as cookies.
Mark it `pub` — it is a production type, not feature-gated.

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
- Auth strategy from construction time is `StaticCookie`: use the held `Arc<StaticCookieAuthProvider>`.

The `SpecDrivenSensorAdapter` holds an enum `AdapterAuthStrategy`:
```rust
enum AdapterAuthStrategy {
    Plugin(Arc<dyn AuthProvider>),        // CrowdStrike: held PluginAuthProvider
    BearerStatic,                         // Armis/Claroty: extracted at fetch() from SensorAuth
    StaticCookie(Arc<dyn AuthProvider>),  // Cyberint: held StaticCookieAuthProvider
                                          // NOTE: was CookieLogin in v1.1/v1.2; renamed per ADR-031
                                          // StaticCookieAuthProvider makes NO HTTP calls at acquire_token;
                                          // reads api_key from credential store and returns it directly.
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
    let org_id = org_registry.resolve(org_slug)
        .ok_or_else(|| BootError::OrgNotFound { slug: org_slug.clone() })?;
    for (_sensor_id, spec) in resolved_specs.iter() {
        let adapter = build_adapter(spec, &plugin_auth_providers, &http_client)?;
        // AdapterRegistry::register is 2-arg: register(org_id, adapter).
        // sensor_id is derived from adapter.sensor_type() internally — do not pass it explicitly.
        adapter_registry.register(org_id, Arc::new(adapter));
    }
}
```

The canonical translation method is `OrgRegistry::resolve(slug) -> Option<OrgId>` (per D-922
adjudication). `id_for_slug` is NOT the correct method name — do not add it. Use `resolve`
directly; skip-and-continue on `None` matches EC-003.

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
DTU clone running and accepting `Cookie: access_token={api_key}` (corrected by
S-DTU-CYBERINT-AUTH-FIDELITY-001 per ADR-031 §D3-a — no login step required).
When: `SpecDrivenSensorAdapter::fetch()` is called for Cyberint.
Then: The adapter uses its held `StaticCookieAuthProvider`, which (a) reads the Cyberint API key
from the credential store at acquire-token time, (b) makes NO HTTP call during acquire_token
(no login step — this is the real Cyberint API behavior per poller-express `cookieTransport`),
(c) returns the API key string as the token value.
The pipeline injects it as `Cookie: access_token={api_key}` (NOT `cyberint_session` — that was
the DTU's pre-fix incorrect model; see ADR-031 §D3 and §Cyberint Cookie Auth Design).
The pipeline does NOT call `Authorization: Bearer`.
The response contains `Vec<RecordBatch>` from the DTU clone.
(traces to BC-2.01.013 postcondition 4; closes cookie_roundtrip gap in pipeline per ADR-031)
Red Gate test: `test_BC_2_01_013_spec_driven_adapter_cyberint_cookie_auth_injects_access_token_cookie`

### AC-004: Boot step 9A registers exactly N adapters (N = eligible adapters after skips)
Given: `spec_catalog` has M resolved sensor specs for each of K orgs.
When: Boot step 9A runs.
Then: `AdapterRegistry` contains exactly N entries, where N = (per-org × per-sensor specs) minus
sensors skipped because their `PluginAuthProvider` is missing (EC-004) or their `auth_type` is
unsupported (EC-007). Each registered entry corresponds to one (OrgId, SensorId) pair.
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

### AC-009: StaticCookieAuthProvider injects Cookie: access_token header, not Authorization Bearer
Given: A `SpecDrivenSensorAdapter` for Cyberint is called with a running DTU clone (corrected by
S-DTU-CYBERINT-AUTH-FIDELITY-001 to accept `Cookie: access_token={api_key}`).
When: `StaticCookieAuthProvider::acquire_token()` is called.
Then:
(a) `StaticCookieAuthProvider` reads the Cyberint API key from the credential store (NO HTTP call;
    NO `POST /login`; NO `Set-Cookie` header parsing).
(b) The HTTP request to `GET /api/v1/alerts` carries `Cookie: access_token={api_key}`,
    NOT `Authorization: Bearer {api_key}`, NOT `Cookie: cyberint_session={anything}`.
(closes the `build_request` cookie-injection gap per §Cyberint Cookie Auth Design and ADR-031 §D3-b)

**Implementation constraint:** The cookie name is `access_token` — the real Cyberint API cookie
name per poller-express `cookieTransport`. Any implementation that uses `cyberint_session` as the
cookie name is WRONG under ADR-031 D1-a. The adversary will probe for this specifically.

### AC-010: Adapter fetch returns OCSF-conformant Arrow RecordBatches (BC-2.01.013 v1.9 OCSF Conformance Clause items 1–3)
Given: A `SpecDrivenSensorAdapter::fetch()` call is made for any of the 4 sensors and the
`PipelineExecutor` successfully fetches and normalizes data.
When: The returned `Vec<RecordBatch>` is inspected.
Then ALL three conformance requirements MUST hold — any single failure is NON-CONFORMANT:

(a) **Spec-declared data columns (item 1):** Every column declared in the sensor's TOML
`[[tables.columns]]` spec MUST appear in the returned Arrow schema via `ColumnMapper`
field-by-field mapping. A RecordBatch that contains only OCSF envelope fields (`category_uid`,
`class_uid`, `_sensor`) while dropping the spec-declared sensor payload columns is
NON-CONFORMANT. The conformance test MUST assert that every column name from the spec appears
in the returned Arrow schema.

(b) **OCSF envelope derivation (item 2):** `class_uid` MUST be derived from the sensor's
declared `ocsf_class` TOML field via `EventClassSelector::select_by_class_name(ocsf_class)`
(the class-name→uid compile-time constant table in `crates/prism-ocsf/src/class_selector.rs`).
`category_uid` is then derived as `class_uid / 1000`. No TOML field named `ocsf_category`
exists — category is never read from the spec directly. An implementation that reads
`category_uid`/`class_uid` verbatim from the raw vendor JSON, or calls
`EventClassSelector::select(sensor_id, &table.ocsf_class)` (the record-type-token overload,
which yields `class_uid = 0` for real class-name strings), is NON-CONFORMANT per
BC-2.01.013 v1.9 (D-925 arch-adjudication). The conformance test MUST assert that
`category_uid` and `class_uid` equal the values produced by
`EventClassSelector::select_by_class_name(ocsf_class)` and `class_uid / 1000` respectively,
not values copied from the raw record. Example: spec declares `ocsf_class = "security_finding"`;
correct `class_uid = 2001`, correct `category_uid = 2` (NOT values from the raw vendor JSON).

(c) **`_sensor` virtual column (item 3):** The `_sensor` virtual column MUST be present and
set to the sensor's canonical `SensorId` string (e.g., `"crowdstrike"`), injected by the
normalization layer. The conformance test MUST assert `_sensor` is present with the correct
sensor ID.

**Conformance test requirement (minimum gate per BC-2.01.013 v1.9 Conformance Clause):**
The test for this AC MUST construct a `SpecDrivenSensorAdapter`, drive it against a mock
`PipelineExecutor` returning a representative raw API response, and assert all three of:
(a) all spec-declared column names appear in the returned Arrow schema,
(b) `class_uid` equals `EventClassSelector::select_by_class_name(ocsf_class)` and
`category_uid` equals `class_uid / 1000` (per BC-2.01.013 v1.9); neither is copied from
the raw record, and
(c) `_sensor` is present with the correct sensor ID.

(traces to BC-2.01.013 v1.9 OCSF Conformance Clause items 1–3; postcondition; traces to BC-2.11.005 postcondition — virtual fields injected by the normalization layer)

**SCOPE NOTE — Query-Param Push-Down is OUT OF SCOPE for this story (D-924, historical):**
`SpecDrivenSensorAdapter::fetch()` did NOT translate `limit`, `cursor`, `start_time`, or
`end_time` from the query caller's parameters into sensor-native API request parameters at the
time S-DEMO-001 was implemented. DataFusion applied `LIMIT` predicates and time-window
post-filters over the fully materialized Arrow RecordBatch. This was correct behavior —
push-down is an optimization (BC-2.11.007 invariant), not a correctness requirement.

**Historical accuracy note (v1.11, 2026-06-05):** Push-down IS now implemented by
S-DEMO-QUERY-PUSHDOWN-001 per BC-2.01.013 v1.12. The AC-011 scope-out above is historical to
S-DEMO-001 and does not apply to implementations built after S-DEMO-QUERY-PUSHDOWN-001 merges.
Test-writers working on S-DEMO-001 MUST NOT assert that `fetch()` passes `limit` or cursor
values to the sensor API (that is S-DEMO-QUERY-PUSHDOWN-001's responsibility, not this story's).

### AC-011: No `todo!()` or `unimplemented!()` in adapter, boot step 9A, or StaticCookieAuthProvider (POL-12)
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
| `StaticCookieAuthProvider` lives in `prism-spec-engine/src/auth_provider.rs` | Crate cohesion — it reads credentials and injects them as cookies, belongs near PipelineExecutor | Not in `prism-bin`; makes NO HTTP calls during acquire_token |
| `BearerStaticAuthProvider` lives in `prism-bin/src/spec_driven_adapter.rs` | It bridges `SensorAuth` (prism-sensors) ↔ `AuthProvider` (prism-spec-engine); only prism-bin imports both | ADR-023 §Permitted Patterns |
| `prism-bin` may import both `prism-sensors` and `prism-spec-engine` | ADR-023 §Permitted Patterns | Existing workspace deps already present in prism-bin |
| `SensorAdapter::fetch(&dyn SensorAuth)` arg ignored for plugin-authed sensors | ADR-028 §D10 | Documented in impl with inline comment citing ADR-028 §D10 |
| `reqwest::Client` MUST set `.timeout(Duration::from_secs(30))` | CLAUDE.md Conventions | Adversary probes for missing timeout on every pass |
| `boot.step9a.adapter_registry_populated` MUST have a BC-2.16.002 catalog row | SAP-1 (standing probe) | Adversary greps `event_type =` on every pass |
| Boot step 9A MUST appear between steps 7.5b and 9 in ADR-022 §B table | BC-2.22.001 + ADR-022 | ADR-022 §B amendment required in same PR |
| `build_request` MUST dispatch header by auth_type | ADR-031 §D3-b + §Cyberint Cookie Auth Design | `CookieRoundtrip → Cookie: access_token={token}` (NOT `cyberint_session`); BearerStatic → Authorization: Bearer |
| `StaticCookieAuthProvider::acquire_token` MUST NOT make HTTP calls | ADR-031 D1-b | Static cookie injection requires no login step; any HTTP call during acquire_token is a fidelity violation |
| Cookie name for CookieRoundtrip MUST be `access_token` | ADR-031 D1-a; poller-express `cookieTransport` | `cyberint_session` is WRONG; adversary probes for this specifically per SAP-2 |

---

## Library & Framework Requirements

| Library | Version | Purpose |
|---------|---------|---------|
| `prism-spec-engine` (workspace) | current workspace path | PipelineExecutor, AuthProvider trait, StaticCookieAuthProvider |
| `prism-sensors` (workspace) | current workspace path | SensorAdapter trait and SensorAuth types |
| `reqwest` | workspace version | HTTP client inside PipelineExecutor (StaticCookieAuthProvider does NOT use reqwest at acquire_token time) |
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
| `crates/prism-spec-engine/src/auth_provider.rs` | MODIFY | Add `StaticCookieAuthProvider` (production impl; NOT feature-gated; NO HTTP calls at acquire_token; per ADR-031 D1-b) |
| `crates/prism-spec-engine/src/pipeline.rs` | MODIFY | Amend `build_request` to accept auth_type; inject Cookie header for CookieRoundtrip |
| `.factory/specs/behavioral-contracts/BC-2.16.002-*.md` | MODIFY | Add catalog row for `boot.step9a.adapter_registry_populated` event per SAP-1 obligation |
| `.factory/specs/architecture/decisions/ADR-022-*.md` | MODIFY | Add boot step 9A to §B sequencing table |
| `.factory/specs/architecture/decisions/ADR-023-*.md` | MODIFY | Add `SpecDrivenSensorAdapter` + `StaticCookieAuthProvider` + `BearerStaticAuthProvider` to §Permitted Patterns list |
| `.factory/specs/architecture/decisions/ADR-028-*.md` | MODIFY | Add §D12: cookie_roundtrip production vs DTU design divergence |

---

## Tasks

1. **Read** `crates/prism-sensors/src/traits.rs` — understand `SensorAdapter::fetch()` and `SensorAuth` signatures.
2. **Read** `crates/prism-spec-engine/src/pipeline.rs` — understand `PipelineExecutor::execute()` + `build_request` function.
3. **Read** `crates/prism-spec-engine/src/auth_provider.rs` — understand `AuthProvider` trait and existing implementations.
4. **Read** `crates/prism-spec-engine/src/spec_parser.rs` — understand `AuthType` enum variants; confirm `CookieRoundtrip` variant exists.
5. **Read** `crates/prism-dtu-cyberint/src/routes/alerts.rs` — understand the DTU's (corrected)
   request handling: the DTU post-S-DTU-CYBERINT-AUTH-FIDELITY-001 accepts `Cookie: access_token={api_key}`
   and has NO `/login` endpoint. Do NOT look for `POST /login`, `Set-Cookie: cyberint_session`, or
   `extract_session_token()` — those were the pre-fix DTU patterns; they do not exist in the corrected DTU
   that this story targets. If you see them, stop and verify S-DTU-CYBERINT-AUTH-FIDELITY-001 has merged.
6. **Read** `crates/prism-bin/src/boot.rs` — locate step 7.5b and the GAP-002-A comment. Understand `ResolvedSensorSpec` map structure.
7. **Read** `S-CONFIG-MULTI-TENANT-OVERRIDE-001` story — confirm the exact output type of the overlay loader (map key type: `OrgSlug` vs `OrgId`).
8. **Amend** `crates/prism-spec-engine/src/pipeline.rs` — refactor `build_request` to accept `auth_type: &AuthType` and dispatch:
   - `CookieRoundtrip` → `Cookie: access_token={token}` (real Cyberint API cookie name per ADR-031 D1-a; NOT `cyberint_session`)
   - All other variants → `Authorization: Bearer {token}` (existing behavior unchanged)
   - Update all callers of `build_request` to pass `spec.auth_type` (two call sites: `issue_request_with_retry`).
9. **Implement** `StaticCookieAuthProvider` in `prism-spec-engine/src/auth_provider.rs`:
   - Fields: `sensor_id: SensorId` (for credential store lookup). The credential resolver is
     constructed INTERNALLY via `PrismCredentialResolver::new()` — it is NOT injected through
     the production constructor.
   - Production constructor: `new(sensor_id: SensorId) -> Self` (1-arg). This is the constructor
     boot step 9A calls — no `credential_resolver` parameter.
   - Test-only constructor: `new_with_resolver(sensor_id: SensorId, resolver: Arc<dyn CredentialResolver>) -> Self`
     — available only under `#[cfg(any(test, feature = "test-helpers"))]`. Do NOT call this from
     production boot code.
   - `AuthProvider::acquire_token()` — uses the internal resolver to look up `api_key` for
     `sensor_id`; returns `AuthToken::new(api_key)`. Makes NO HTTP call. No `credential_resolver`
     is threaded in from the boot callsite — it is entirely internal.
   - Error path: credential resolve failure → `SpecEngineError::AuthAcquisitionFailed`.
   - This replaces the v1.1/v1.2 `CookieLoginAuthProvider` (which made HTTP calls to `POST /login`).
   - INVARIANT: `acquire_token` must never make an HTTP call. If this invariant is violated, the
     adversary will flag it as an ADR-031 D1-b violation.
10. **Write stub** `crates/prism-bin/src/spec_driven_adapter.rs` with `todo!()` bodies (Red Gate setup).
11. **Write Red Gate tests** (see AC-001, AC-003, AC-004, AC-005 test names) — all must fail (RED) before implementation.
12. **Implement** `SpecDrivenSensorAdapter`:
    - Fields: `sensor_spec: Arc<ResolvedSensorSpec>`, `auth_strategy: AdapterAuthStrategy`, `http_client: reqwest::Client`.
      (No `executor: Arc<PipelineExecutor>` field — `PipelineExecutor::execute` is called as a
      static/associated method; the executor is not held on the struct.)
    - `AdapterAuthStrategy` enum: `Plugin(Arc<dyn AuthProvider>)`, `BearerStatic`, `StaticCookie(Arc<dyn AuthProvider>)`.
      (Variant is `StaticCookie`, NOT `CookieLogin` — renamed per ADR-031; see OQ-1 Resolution.)
    - `SensorAdapter::fetch()` — dispatch by `auth_strategy`; for BearerStatic, extract token from `&dyn SensorAuth` arg and construct `BearerStaticAuthProvider` per-call.
    - `SensorAdapter::sensor_type()` — return `SensorId::from(self.sensor_spec.sensor_id.as_str())`.
13. **Implement** `BearerStaticAuthProvider` in `crates/prism-bin/src/spec_driven_adapter.rs`:
    - Thin wrapper implementing `AuthProvider`; stores the bearer token string; returns it via `acquire_token`.
14. **Implement boot step 9A** in `boot.rs`:
    - After step 7.5b: iterate `resolved_sensor_specs` map (translate OrgSlug → OrgId if needed).
    - For each `(org_id, sensor_id, resolved_spec)`: inspect `resolved_spec.auth_type`:
      - `CustomViaPlugin`: look up `plugin_auth_providers.get(&sensor_id)` → `AdapterAuthStrategy::Plugin`.
      - `BearerStatic`: → `AdapterAuthStrategy::BearerStatic`.
      - `CookieRoundtrip`: construct `StaticCookieAuthProvider::new(sensor_id)` (1-arg — resolver
        is internal; do NOT thread `credential_resolver` from the boot callsite) → `AdapterAuthStrategy::StaticCookie`.
      - Others: log E-SPEC-012 (auth type mismatch); skip.
    - Register adapters; emit `boot.step9a.adapter_registry_populated` event.
15. **Amend BC-2.16.002** — add catalog row for `boot.step9a.adapter_registry_populated` per SAP-1.
16. **Amend ADR-022 §B** — add boot step 9A to the sequencing invariant table.
17. **Amend ADR-023 §Permitted Patterns** — add `SpecDrivenSensorAdapter`, `StaticCookieAuthProvider`, `BearerStaticAuthProvider` patterns.
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
| EC-001 | Sensor spec has `auth_type = "cookie_roundtrip"` but credential store has no `api_key` entry for the sensor | `StaticCookieAuthProvider::acquire_token` returns `SpecEngineError::AuthAcquisitionFailed` with detail "api_key not found in credential store"; adapter not registered |
| EC-002 | Cyberint DTU returns 401 (invalid `access_token` cookie value) | `PipelineExecutor::issue_request_with_retry` returns `Err(SpecEngineError::CookieAuthFailed)` IMMEDIATELY — NO retry, NO second `acquire_token` call. Retry is provably futile: `StaticCookieAuthProvider` holds a static key and would return the identical value on a second call. Per BC-2.01.017 EC-017-002: on 401 for `AuthType::CookieRoundtrip`, the pipeline short-circuits with `CookieAuthFailed` rather than looping. (Contrast with EC-006: the CrowdStrike OAuth2 path DOES retry on 401 via token refresh.) |
| EC-003 | org_registry has orgs but spec_catalog is empty | Boot step 9A produces 0 registrations; no error; boot continues to step 9 |
| EC-004 | PluginAuthProvider fails at step 7.5b | Step 7.5b handles; step 9A skips sensors with missing auth providers |
| EC-005 | ResolvedSensorSpec has per-org base_url overlay — StaticCookieAuthProvider does not use base_url | StaticCookieAuthProvider does NOT need base_url (no HTTP calls at acquire_token). The overlay is used by PipelineExecutor for the actual fetch requests. This EC is not a risk for StaticCookieAuthProvider. |
| EC-006 | Double-401 from sensor API during SpecDrivenSensorAdapter::fetch() | PipelineExecutor handles the refresh; on second 401, returns SpecEngineError::AuthRefreshFailed. Adapter propagates this. |
| EC-007 | Sensor spec has unsupported `auth_type` (e.g., `api_key`) — not yet implemented in SpecDrivenSensorAdapter | Boot logs E-SPEC-012 (auth type mismatch); adapter NOT registered for that sensor; boot continues |
| EC-008 | StaticCookieAuthProvider 401: 401 from Cyberint DTU mid-pipeline | `issue_request_with_retry` returns `Err(SpecEngineError::CookieAuthFailed)` IMMEDIATELY — NO retry. For `AuthType::CookieRoundtrip`, any 401 response is terminal: retry is futile because `StaticCookieAuthProvider::acquire_token` always returns the same static credential from the store. There is no "session expiry" concept and no refresh path. Cross-ref: BC-2.01.017 EC-017-002. Note: `AuthRefreshFailed` applies only to OAuth2 token-refresh paths (e.g., CrowdStrike plugin — see EC-006). |

---

## Spec Updates Required (same PR)

| Document | Amendment |
|----------|-----------|
| ADR-022 §B | Add boot step 9A (`spec_driven_adapter_registry_populate`) between step 7.5b and step 9 |
| ADR-023 §Permitted Patterns | Add `SpecDrivenSensorAdapter`, `StaticCookieAuthProvider`, `BearerStaticAuthProvider` pattern notes |
| ADR-022 §F | Update comment that `adapter_registry` is empty at step 9 — post-story it is populated |
| ADR-028 §D12 | ALREADY annotated `[SUPERSEDED by ADR-031 §D4 2026-05-29]` in factory-artifacts burst. No further amendment needed in this story's PR. |
| ADR-031 | ALREADY authored in factory-artifacts burst. Cite in story implementation notes. |
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
| 1.12 | 2026-07-24 | story-writer | F-WASE-P52-LOW-001 POL-29 class sweep, burst wave-a-spec-evolution-fix-burst-41: 6 occurrences of stale `PipelineExecutor::build_request` qualifier corrected to free-function citation — first mention uses `build_request` (module-level free function in `crates/prism-spec-engine/src/pipeline.rs`), subsequent mentions plain `build_request`. No ACs, BCs, or behavioral semantics changed. |
| 1.11 | 2026-06-05 | story-writer | Minimal historical-accuracy correction (F-PUSHDOWN-008 cross-story drift, PO-surfaced): AC-010 SCOPE NOTE updated — stale "Test-writers MUST NOT assert fetch() passes limit/cursor" instruction now clarified as historical to S-DEMO-001 only; notes that push-down IS implemented by S-DEMO-QUERY-PUSHDOWN-001 per BC-2.01.013 v1.12. No ACs, BCs, or semantics changed. Story is MERGED (PR #166) — this is a documentation-only correction for auditable accuracy. |
| 1.10 | 2026-06-01 | story-writer | OBS-P3-001 version-pin sweep: AC-010 heading, conformance test requirement label, traces-to line, and Pagination/Push-Down Scope Clause reference all updated from BC-2.01.013 v1.8 → v1.9. Historical changelog entries (1.5 and 1.6 rows) are untouched — they record what was written when the BC was at v1.8. |
| 1.9 | 2026-05-31 | story-writer | POL-32 hygiene (F-PASS1-MED-002): changelog reordered to monotonic DESCENDING per POL-32 (newest first). AC-004 clarification (F-PASS1-OBS-002): N defined as (per-org × per-sensor specs) minus sensors skipped per EC-004 (missing PluginAuthProvider) or EC-007 (unsupported auth_type); aligns with Red Gate test expectation of 2 (armis×2; crowdstrike skipped). |
| 1.8 | 2026-05-31 | story-writer | ADV-P06 exhaustive closure sweep (ADV-P06-MED-001 + ADV-P06-MED-002). MED-001: EC-002 and EC-008 corrected from "retry → AuthRefreshFailed" to NO-RETRY → CookieAuthFailed per BC-2.01.017 EC-017-002; §Cyberint Cookie Auth Design "On 401" bullet corrected to match no-retry semantics. MED-002: Task 9 corrected — StaticCookieAuthProvider production constructor is 1-arg `new(sensor_id)` (resolver internal via PrismCredentialResolver); test-only `new_with_resolver(sensor_id, resolver)` named and feature-gated; no `credential_resolver` injected from boot callsite. Task 14 corrected — `StaticCookieAuthProvider::new(sensor_id)` (1-arg; no `Arc::clone(&credential_resolver)`). Task 5 rewritten — stale instruction to read OLD DTU `POST /login` / `cyberint_session` pattern replaced with corrected-DTU reading guidance. No prescriptive stale references remain after this sweep. |
| 1.7 | 2026-05-31 | story-writer | ADV-P05 drift sweep: fixed 4 stale-design locations missed by v1.3 sweep. HIGH-001: risk_mitigations CookieRoundtrip entry corrected — cookie name `access_token` (not `cyberint_session`), no `POST /login` step, no self-contradiction. HIGH-002: Task 8 cookie name `access_token` (not `cyberint_session`); Task 12 `AdapterAuthStrategy::StaticCookie` (not `CookieLogin`); removed stale `executor: Arc<PipelineExecutor>` field description. LOW-002: OQ-2 pseudo-code `register(org_id, sensor_id.clone(), Arc::new(adapter))` → 2-arg `register(org_id, Arc::new(adapter))`. LOW-001: changelog rows reordered to monotonic ascending (1.0→1.7). |
| 1.6 | 2026-05-31 | story-writer | AC-010(b) story↔BC drift fix (OBS-PASS4-002): dropped non-existent `ocsf_category` TOML field reference; aligned derivation to BC-2.01.013 v1.9 semantics — `class_uid` from `EventClassSelector::select_by_class_name(ocsf_class)`, `category_uid = class_uid / 1000`; named the wrong-overload anti-pattern (`select(sensor_id, class_name_string)` yields 0) per D-925. Conformance test assertion (b) tightened to match corrected derivation path. |
| 1.5 | 2026-05-31 | story-writer | AC-010 rewritten (adversary pass-2, F-001-R + F-003-R closure): old envelope-only wording replaced with BC-2.01.013 v1.8 OCSF Conformance Clause items 1–3 verbatim-aligned requirements — (a) all spec-declared columns survive via ColumnMapper (envelope-only output is NON-CONFORMANT), (b) category_uid/class_uid derived by OcsfNormalizer not read from raw record (raw-copy is NON-CONFORMANT), (c) _sensor virtual column = canonical SensorId. Conformance test requirement added (minimum gate per BC-2.01.013 v1.8). Scope note added: query-param push-down (limit/cursor/time-window) OUT OF SCOPE per BC-2.01.013 v1.8 Pagination/Push-Down Scope Clause (D-924), deferred to S-DEMO-QUERY-PUSHDOWN-001; DataFusion applies LIMIT post-materialization. |
| 1.4 | 2026-05-31 | story-writer | OQ-2 factual-accuracy fix (adversary pass-1, D-922 adjudication): `OrgRegistry::id_for_slug` → `OrgRegistry::resolve(slug) -> Option<OrgId>` as the canonical existing method. Removed instruction to add `id_for_slug`. Skip-and-continue on `None` matches EC-003. |
| 1.3 | 2026-05-29 | architect | **DTU=true-DTU REVERSAL (ADR-031 user directive 2026-05-29).** `cyberint_session` decision reversed. ALL Cyberint references updated: `CookieLoginAuthProvider` (login-step) → `StaticCookieAuthProvider` (no login step, credential-store read); cookie name `cyberint_session` → `access_token`; `build_request` dispatch `Cookie: cyberint_session` → `Cookie: access_token`. §Origin updated. §Cyberint Cookie Auth Design section rewritten. AC-003/AC-009 rewritten. OQ-1 enum variant `CookieLogin` → `StaticCookie`. EC-001/EC-002/EC-005/EC-008 updated. Library/framework table updated. ADR-028 §D12 → pre-authored in factory-artifacts burst (now annotated SUPERSEDED). ADR-031 cited throughout. New depends_on: S-DTU-CYBERINT-AUTH-FIDELITY-001 (P0-pre-demo-BLOCKING — DTU correction required before implementation). |
| 1.2 | 2026-05-29 | architect | AC-003/AC-009: tightened `cyberint_session` cookie name specification per fidelity audit (POLLER-DTU-FIDELITY-AUDIT-2026-05-29). Added Set-Cookie parse constraint to AC-009 (cookie name `cyberint_session` not `access_token` for DTU demo path). Cross-poller audit surfaced Gap-CS-001/CL-002/CL-003/CL-004/CL-005; TOML fixes applied to crowdstrike.sensor.toml and claroty.sensor.toml in same burst. |
| 1.1 | 2026-05-29 | architect | Corrected Cyberint auth model (cookie_roundtrip ≠ bearer_static); resolved OQ-1, OQ-2, OQ-6; added CookieLoginAuthProvider design; amended build_request pipeline gap; added new AC-003/AC-009/AC-012; revised points 8→11; risk MEDIUM→HIGH |
| 1.0 | 2026-05-29 | story-writer | Initial draft — all 4 sensors scope per user 2026-05-29 decision |
