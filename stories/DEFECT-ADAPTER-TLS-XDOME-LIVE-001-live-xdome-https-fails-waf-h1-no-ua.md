---
document_type: story
story_id: "DEFECT-ADAPTER-TLS-XDOME-LIVE-001"
title: "xDome transport hardening: http2 feature + User-Agent + error source-chain + error mapping (F10 + F9 bundled)"
wave: "C"
epic_id: engine-defects
priority: P1
status: ready
version: "1.24"
severity: CRIT
level: engine
producer: story-writer
timestamp: "2026-08-03"
modified: "2026-08-14"
holdout_scenarios: [HS-TLS-XDOME-001, HS-TLS-XDOME-002, HS-TLS-XDOME-003]
inputs:
  - .factory/planning/findings-remediation-2026-07-20/xdome-transport-hardening-design.md
  - .factory/planning/findings-remediation-2026-07-20/triage-capture.md
input-hash: "7f2e0df"
traces_to: []
origin_finding: "F10 (D-1889 triage 2026-07-20) + F9 (D-1889); bundled per §5 Bundling Verdict in design doc"
cycle: "v1.0.0-greenfield"
phase: 3
track: "Platform Engineering"
tdd_mode: strict
subsystems: [SS-16, SS-01, SS-22, SS-21, SS-08]
# Subsystem anchor justifications:
#   SS-16 (Spec Engine) owns prism-spec-engine/src/pipeline.rs — send-failure source-chain
#     and non-2xx body-capture changes live here per ARCH-INDEX Subsystem Registry.
#   SS-01 (Fan-Out / Cross-Client Query) owns prism-sensors/src/fanout.rs — per-target
#     WARN log addition lives in the fan-out subsystem per ARCH-INDEX.
#   SS-22 (Sensor Adapters / Boot) owns prism-bin/src/spec_driven_adapter.rs and
#     prism-bin/src/boot.rs — error mapping fix and UA addition in boot client builders
#     live in the adapter/boot layer per ARCH-INDEX.
#   SS-21 (Identity & Core Types) owns prism-core/src/error.rs and prism-core/src/lib.rs
#     — sanitize_body_snippet_bytes / sanitize_body_snippet utility functions and the
#     sanitize_body_snippet + sanitize_body_snippet_bytes re-exports in lib.rs live in the core types layer per
#     ARCH-INDEX Subsystem Registry.
#   SS-08 (Sensor Health) owns prism-mcp/src/health/connectivity.rs — the sanitize_error
#     delegation to prism_core::sanitize_body_snippet in connectivity.rs lives in
#     the sensor health subsystem per ARCH-INDEX Subsystem Registry.
crates_touched: [prism-spec-engine, prism-sensors, prism-bin, prism-core, prism-mcp]
target_module: prism-bin
behavioral_contracts:
  - BC-2.16.002
  - BC-2.08.002
  - BC-2.01.010
  - BC-2.19.001
  - BC-2.01.013
  - BC-2.16.014
# BC status (current, v1.24):
#   BC-2.16.002 (Multi-Step Fetch Pipeline Execution): v2.20, status: active
#     New §Postconditions added: HTTP Client Compliance (ADR-050 §D5/§D6), Send-Failure
#     Error Source Chain, Non-2xx Response Body Capture, AllTargetsFailed Per-Target
#     Logging. New catalog row 91: fan_out_target_failed WARN. Scope extended to include
#     prism-sensors/src/fanout.rs. Catalog updated v1.63 (send-failure integration test
#     coverage RG-009 in fix-burst pass-1).
#   BC-2.08.002 (Auth Validity Check): v1.7, status: active
#     New §Postconditions: HTTP Error Classification postcondition for
#     map_spec_engine_error_to_sensor_error. EC-08-006 added: HTTP 4xx → auth_valid: false
#     (previously incorrectly resolved to ConnectivityStatus::Down via Internal catch-all).
#     v1.5: AuthRefreshFailed and CookieAuthFailed variants added to HTTP Error
#     Classification postcondition scope (fix-burst pass-1).
#     v1.7: 5xx HTTP response classification corrected: `ConnectivityStatus::Down` → `Degraded`.
#   BC-2.01.010 (Partial Failure Handling): v1.6, status: draft, lifecycle_status: active
#     AllTargetsFailed Per-Target Logging postcondition added.
#   BC-2.01.013 (DataSource Trait Eliminates Per-Sensor Code Duplication): v1.19, status: active
#     EC-01-029: AuthRefreshFailed / CookieAuthFailed persistent-auth-failure variants
#     must map to SensorError::HttpError { status: 401 } end-to-end (fix-burst pass-1).
#   BC-2.16.014 (Declarative Auth Acquisition Token Lifecycle): v1.22, status: draft
#     New INV row: DeclarativeHttpAuthProvider auth client inherits ADR-050 §D5/§D6
#     compliance automatically via build_http_client_with_timeout (prism-spec-engine::pipeline),
#     an independent sibling with its own .user_agent() call (not propagation from prism-bin).
#     No new ACs required in this story for BC-2.16.014 scope beyond the UA propagation
#     side-effect verified by AC-UA-001.
#   BC-2.19.001 (Infusion Spec Loading — Each Field Registers Exactly One DataFusion Scalar UDF): v2.4, status: active
#     New §Error Conditions row E-INFUSE-015: `InfusionError::HttpClientBuildFailed { detail }`
#     is the correct error variant for `build_http_client_with_timeout` failure during
#     HttpLookup-type infusion spec load (RUNTIME PHASE); eliminates the E-INFUSE-009 stopgap.
#     Effectively unreachable under ADR-050 rustls-tls mandate (SID-1 unit test as
#     compensating control). Verified by RG-013 (AC-ERR-006). F-2 completion.
# S-7.01: behavioral_contracts non-empty, all canonical IDs; status: ready is valid.
# Every BC cited by at least one AC below; every AC cites a BC. Bidirectional traces
# verified per bc_array_changes_propagate_to_body_and_acs policy.
verification_properties: []
depends_on: []
blocks: []
points: 5
# Points justification:
#   Cargo.toml http2 feature edits × 4 entries: 0.5 pt
#   build_http_client_with_custom_timeout UA call: 0.3 pt
#   boot.rs two UA sites: 0.3 pt
#   pipeline.rs send-failure source-chain × 2 symmetric sites: 1.0 pt
#   pipeline.rs non-2xx body-capture × 2 symmetric sites (first request + 401-retry): 1.0 pt
#   spec_driven_adapter.rs map_spec_engine_error_to_sensor_error guard: 0.5 pt
#   fanout.rs AllTargetsFailed per-target WARN loop: 0.4 pt
#   14 Red Gate tests (RG-001..RG-014) + live verification support: 1.4 pt
#   Total: ~5.2 points
estimated_days: 1.5
risk: HIGH
# Risk justification:
#   Touches 6 production files across 3 crates; live-verification ACs (LIVE-001..003)
#   are BLOCKING — no waiver possible. F9 fix is a prerequisite for F10 diagnostic
#   coverage; partial fix leaves relay required. Changes to error mapping affect the
#   health-probe path used by all sensors (regression risk for existing tests).
assumption_validations:
  - id: AV-01
    assumption: "reqwest 0.12.28 `http2` cargo feature is additive (ALPN advertises h2 + http/1.1, falls back to HTTP/1.1 if server lacks h2), not h2-only; pulls the h2 crate; feature name is exactly `http2`."
    verdict: CONFIRMED
    source: "docs.rs/crate/reqwest/0.12.28/features; reqwest async_impl/client.rs ALPN-configuration match arm (Context7); workspace Cargo.lock reqwest 0.12.28 / h2 0.4.13"
  - id: AV-02
    assumption: "`ClientBuilder::user_agent<V: TryInto<HeaderValue>>` exists on reqwest 0.12; `.user_agent(concat!(\"prism/\", env!(\"CARGO_PKG_VERSION\")))` compiles as a &'static str; reqwest sends no default User-Agent otherwise."
    verdict: CONFIRMED
    source: "docs.rs/reqwest ClientBuilder::user_agent (documented example is the identical concat! pattern)"
  - id: AV-03
    assumption: "Adding h2-ALPN + a real User-Agent is a credible remedy for the api.claroty.com edge (AWS Global Accelerator + WAF) blocking a UA-less, HTTP/1.1-only, rustls-fingerprinted client (materially affects no-waiver AC-LIVE-001)."
    verdict: RISK
    source: "Perplexity deep research (AWS WAF JA3/JA4 docs.aws.amazon.com/waf; rustls issue #1421); relay evidence test-soc/live-soc/relay/xdome-relay.py (Python urllib: HTTP/1.1-only + default UA + OpenSSL fingerprint succeeds)"
    note: "Moderate confidence SUFFICIENT: the User-Agent is the probable load-bearing fix; h2 is not empirically necessary (working relay is HTTP/1.1-only). Residual risk on no-waiver AC-LIVE-001 from possible rustls-specific JA3/JA4 blocklisting or a non-UA header differentiator. See §Technology Assumption Validation RISK."
  - id: AV-04
    assumption: "rustls (via reqwest rustls-tls) advertises h2 in the ClientHello ALPN extension once the http2 feature is on; rustls-tls + http2 compose correctly."
    verdict: CONFIRMED
    source: "reqwest async_impl/client.rs ALPN match arm sets tls.alpn_protocols on the rustls ClientConfig (Context7); reqwest 0.12.28 http2 feature also enables hyper-rustls/http2 (docs.rs features)"
risk_mitigations:
  - risk: "AC-H2-001 / RG-008 observable is invalid as written: h2 0.4.13 is ALREADY in Cargo.lock (transitively via hyper 1.9.0), so a whole-file `grep '\"h2\"' Cargo.lock` is green BEFORE the fix and cannot serve as a Red Gate or prove reqwest's http2 feature is on."
    mitigation: "AC-H2-001 and RG-008 corrected to scope verification to reqwest's own dependency node — `cargo tree -e features -i reqwest` showing the http2 feature active, or `h2` appearing inside the `[[package]] name = \"reqwest\"` dependencies block (currently absent; verified this pass) — not a whole-file grep."
    status: applied
  - risk: "AC-LIVE-001 (BLOCKING, no-waiver) may fail if the edge fingerprints the rustls TLS ClientHello (JA3/JA4) or keys on a header other than User-Agent — neither the h2 nor the UA change addresses rustls ClientHello camouflage."
    mitigation: "RISK note added to §Technology Assumption Validation; h2 reframed as ADR-050 §D5 compliance + defense-in-depth (not the load-bearing fix); optional cheap pre-build reqwest+UA probe against api.claroty.com recommended. Surfaced to orchestrator/human per remove-uncertainty discipline; does not block materialization."
    status: surfaced
---

# DEFECT-ADAPTER-TLS-XDOME-LIVE-001: xDome Transport Hardening — http2 Feature, User-Agent, Error Source Chain, and Error Mapping (F10 + F9 Bundled)

## Authority

This story is governed by ADR-050 v2.3 and the five behavioral contracts below.
Read ADR-050 §D5 and §D6 in full before implementing:
`.factory/specs/architecture/decisions/ADR-050-workspace-reqwest-tls-backend.md`

| Artifact | Version / Status | Relevant Clause |
|----------|-----------------|-----------------|
| ADR-050 (Workspace reqwest TLS Backend) | v2.3 · ACCEPTED | §D5: `http2` feature MUST be in production `[dependencies]` reqwest entries for prism-spec-engine, prism-sensors, prism-bin. §D6 (v2.1 extended scope): ALL outbound third-party HTTP client builders MUST call `.user_agent(concat!("prism/", env!("CARGO_PKG_VERSION")))` — includes sensor/plugin outbound builders AND `build_http_client_with_timeout` in prism-spec-engine/src/pipeline.rs (infusion `HttpLookupSource` factory; added v2.1 via sibling-sweep OBS-4). §D1/§D2: rustls-tls mandatory; native-tls and aliases FORBIDDEN. |
| BC-2.16.002 (Multi-Step Fetch Pipeline Execution) | v2.20 · active | §Postconditions: HTTP Client Compliance; Send-Failure Error Source Chain; Non-2xx Response Body Capture; AllTargetsFailed Per-Target Logging. §Canonical Structured Event Catalog row 91: `fan_out_target_failed` WARN. |
| BC-2.08.002 (Auth Validity Check Per Sensor Per Client) | v1.7 · active | §Postconditions: HTTP Error Classification postcondition — `map_spec_engine_error_to_sensor_error` MUST map `SpecEngineError::HttpRequestFailed { status_code > 0 }` (including `AuthRefreshFailed` and `CookieAuthFailed` 401 variants) to `SensorError::HttpError`. EC-08-006: HTTP 4xx sensor response → `auth_valid: false` (not Down). 5xx HTTP response → `SensorError::HttpError { status >= 500 }` → `ConnectivityStatus::Degraded` (not Down) in `connectivity.rs`; verified by RG-014 (AC-ERR-007). |
| BC-2.01.010 (Partial Failure Handling) | v1.6 · draft | §Postconditions: AllTargetsFailed Per-Target Logging — each `FanOutError` MUST be logged at WARN before `AllTargetsFailed` propagates. |
| BC-2.01.013 (DataSource Trait Eliminates Per-Sensor Code Duplication) | v1.19 · active | Scope — EC-01-029: `AuthRefreshFailed` and `CookieAuthFailed` persistent-auth-failure variants MUST map to `SensorError::HttpError { status: 401 }` end-to-end, producing `auth_valid: false` at the health probe surface. |
| BC-2.16.014 (Declarative Auth Acquisition Token Lifecycle) | v1.22 · draft | INV-014-007 ADR-050 §D5/§D6 note: `DeclarativeHttpAuthProvider` inherits User-Agent and http2 automatically via `build_http_client_with_timeout` in `prism-spec-engine::pipeline` (independent sibling with its own `.user_agent()` call — not propagation from prism-bin). No separate implementation required for this BC in this story. |
| BC-2.19.001 (Infusion Spec Loading) | v2.4 · active | §Error Conditions E-INFUSE-015: `InfusionError::HttpClientBuildFailed { detail }` is the correct error variant for `build_http_client_with_timeout` failure during `HttpLookup`-type infusion spec RUNTIME PHASE; eliminates the E-INFUSE-009 stopgap. Scope: F-2 completion in this story. Verified by RG-013. |

**Bundling rationale:** F9 (error surfacing) and F10 (transport) are bundled into this story per §5 Bundling Verdict in the design doc (`xdome-transport-hardening-design.md`). `DEFECT-SENSOR-ERROR-FLATTEN-001` is superseded by this story and closed.

---

## Narrative

As a prism operator running live sensor data for client "monroe" (Claroty xDome,
`api.claroty.com`), I want direct HTTPS communication with the real xDome API to succeed
without a relay sidecar, and I want the health tool to correctly distinguish a reachable
sensor with expired credentials (`auth_valid: false`) from a genuinely unreachable sensor
(`reachable: false`), so that I can diagnose credential problems without needing to inspect
raw error logs or deploy a relay.

---

## Behavioral Contracts

| BC | Title | Version | Scope in This Story |
|----|-------|---------|---------------------|
| BC-2.16.002 | Multi-Step Fetch Pipeline Execution — Sequential Steps with Variable Interpolation | v2.20 | HTTP Client Compliance postcondition (ADR-050 §D5/§D6); Send-Failure Error Source Chain postcondition; Non-2xx Response Body Capture postcondition; AllTargetsFailed Per-Target Logging postcondition; Canonical Structured Event Catalog row 91 (`fan_out_target_failed`). |
| BC-2.08.002 | Auth Validity Check Per Sensor Per Client | v1.7 | HTTP Error Classification postcondition: `map_spec_engine_error_to_sensor_error` guard — `status_code > 0` → `SensorError::HttpError`; `status_code = 0` → `SensorError::Internal`. EC-08-006: HTTP 4xx → `auth_valid: false` (not Down). `AuthRefreshFailed` and `CookieAuthFailed` 401 variants included in scope (fix-burst pass-1). 5xx → `ConnectivityStatus::Degraded` (not Down) via `SensorError::HttpError { status >= 500 }` arm in `connectivity.rs`; verified by RG-014 (AC-ERR-007). |
| BC-2.01.010 | Partial Failure Handling for Paginated and Cross-Client Queries | v1.6 | AllTargetsFailed Per-Target Logging postcondition — each `FanOutError` WARN before propagation. |
| BC-2.01.013 | DataSource Trait Eliminates Per-Sensor Code Duplication | v1.19 | Scope (EC-01-029): `AuthRefreshFailed` / `CookieAuthFailed` persistent-auth-failure variants MUST map to `SensorError::HttpError { status: 401 }` end-to-end. Verified by RG-010 and RG-011 (fix-burst pass-1). |
| BC-2.16.014 | Declarative Auth Acquisition Token Lifecycle | v1.22 | INV-014-007 note only: `DeclarativeHttpAuthProvider` inherits UA + http2 via `build_http_client_with_timeout` in `prism-spec-engine::pipeline` (ADR-050 §D6 propagation — independent sibling with its own `.user_agent()` call). No new code required for this BC beyond the `build_http_client_with_timeout` change in `prism-spec-engine::pipeline`. |
| BC-2.19.001 | Infusion Spec Loading — Each Field Registers Exactly One DataFusion Scalar UDF | v2.4 | §Error Conditions E-INFUSE-015 row: `build_http_client_with_timeout` failure during `HttpLookup`-type infusion spec load (1 site each: `load_spec`, `load_spec_with_runtime`, `hot_reload`) returns `InfusionError::HttpClientBuildFailed { detail }`; eliminates E-INFUSE-009 stopgap. Effectively unreachable under ADR-050 `rustls-tls` mandate; testable via direct variant construction (SID-1 compensating control). When triggered during `hot_reload`, previous registry is retained per BC-2.19.004 atomicity contract. Scope: F-2 completion. |

---

## Acceptance Criteria

### Group A — Transport Fix (F10)

**AC-H2-001 — reqwest `http2` feature is active on the `reqwest` dependency node**
After adding `"http2"` to the three production `[dependencies]` reqwest feature lists, the `http2`
feature is active on the `reqwest` node in the build graph and `h2` appears as a
dependency of the `reqwest` package itself (reqwest 0.12.28's `http2` feature enables
`dep:h2`). Verified via `cargo tree -e features -i reqwest` showing the `http2` feature
on reqwest, OR by asserting `h2` appears inside the `[[package]]`/`name = "reqwest"`
`dependencies` block of `Cargo.lock` (Red Gate: RG-008).
**Observability note (remove-uncertainty pass, D-1110):** a whole-file
`grep '"h2"' Cargo.lock` / `cargo metadata | grep '"h2"'` is INVALID as the observable —
`h2 0.4.13` is ALREADY present in `Cargo.lock` transitively via `hyper 1.9.0` (whose
`http2` feature is activated by another workspace consumer), so a whole-file match is
green BEFORE this fix and does not prove reqwest's `http2` feature is enabled. The
`h2`-under-reqwest scoping (currently absent from reqwest's own dependency block —
verified this pass) is the valid red→green signal.
(traces to BC-2.16.002 HTTP Client Compliance postcondition: ADR-050 §D5 requires
http2 in production reqwest deps; the reqwest-scoped http2 feature activation is the
observable proof)

**AC-UA-001 — `build_http_client_with_custom_timeout` sets `User-Agent: prism/<version>`**
A unit test in `crates/prism-bin/src/spec_driven_adapter.rs` `#[cfg(test)] mod tests`
builds a client via `build_http_client_with_custom_timeout(Duration::from_millis(1))`,
issues a request, and asserts the outgoing `User-Agent` header begins with `"prism/"`.
This test FAILS before the `.user_agent(concat!("prism/", env!("CARGO_PKG_VERSION")))`
call is added (Red Gate: RG-006).
(traces to BC-2.16.002 HTTP Client Compliance postcondition: ADR-050 §D6 scope includes
`build_http_client_with_custom_timeout`; also traces to BC-2.16.014 INV-014-007:
`DeclarativeHttpAuthProvider` obtains its UA from the independent
`prism-spec-engine::pipeline::build_http_client_with_timeout` sibling — its OWN
`.user_agent()` call — NOT via automatic delegation from the prism-bin factory)

**AC-UA-002 — Both `boot.rs` plugin client builder sites include User-Agent (adversary-verified)**
Both `reqwest::Client::builder()` chains in `crates/prism-bin/src/boot.rs` that produce
the `PluginRuntime` HTTP client — the `PRISM_DISABLE_PLUGIN_LOAD` fast-path builder and
the normal-path builder — each include
`.user_agent(concat!("prism/", env!("CARGO_PKG_VERSION")))` before `.build()`.
Verified by adversary sweep of `boot.rs` for `user_agent` presence at both builder sites.
(traces to BC-2.16.002 HTTP Client Compliance postcondition: ADR-050 §D6 scope includes
both PluginRuntime client builders in boot.rs)

**AC-CARGO-001 — All three production reqwest entries include `"http2"`; `just check` passes**
After editing the three production `[dependencies]` reqwest entries — `prism-spec-engine`,
`prism-sensors`, and `prism-bin` (one production entry: the S-PLUGIN-PREREQ-D AC-9 shared
client) — each entry includes `"http2"` alongside `"rustls-tls"`. `just check` passes
without new compilation errors. `prism-bin`'s `[dev-dependencies]` reqwest entry (which
explicitly declares `"http2"` in its features array — harmless, not modified by this
story) and DTU `[dev-dependencies]` are NOT modified.
(traces to BC-2.16.002 HTTP Client Compliance postcondition: ADR-050 §D5 names the three
specific production entries in the three crates)

### Group B — Error Surfacing (F9)

**AC-ERR-001 — `map_spec_engine_error_to_sensor_error` with `status_code: 401` returns `SensorError::HttpError` (not `Internal`)**
Calling `map_spec_engine_error_to_sensor_error(SpecEngineError::HttpRequestFailed { status_code: 401, detail: "HTTP 401".to_string(), .. }, "claroty", "devices")` returns `SensorError::HttpError { sensor: "claroty", status: 401, body: "HTTP 401" }`.
This test FAILS before the guard arm is added (Red Gate: RG-001).
**Scope extension (fix-burst pass-1):** the persistent-auth-failure variants `AuthRefreshFailed` and `CookieAuthFailed` are matched DIRECTLY in Arm 2 of `map_spec_engine_error_to_sensor_error` (via `matches!(e, SpecEngineError::AuthRefreshFailed { .. } | SpecEngineError::CookieAuthFailed { .. })`), NOT through `HttpRequestFailed`. They are also verified by this AC via RG-010 and RG-011. Both variants MUST produce `SensorError::HttpError { status: 401 }`.
(traces to BC-2.08.002 HTTP Error Classification postcondition: `SpecEngineError::HttpRequestFailed { status_code > 0 }` MUST map to `SensorError::HttpError`; also traces to BC-2.01.013 EC-01-029: persistent-auth-failure variants map to `SensorError::HttpError { status: 401 }` end-to-end)

**AC-ERR-002 — `map_spec_engine_error_to_sensor_error` with `status_code: 0` returns `SensorError::Internal` (no regression)**
Calling `map_spec_engine_error_to_sensor_error(SpecEngineError::HttpRequestFailed { status_code: 0, detail: "connection reset".to_string(), .. }, "claroty", "devices")` returns `SensorError::Internal { .. }`.
Transport failures (no HTTP response received) continue to map to Internal.
This test FAILS if the guard is over-broad (Red Gate: RG-002).
(traces to BC-2.08.002 HTTP Error Classification postcondition: `status_code = 0` (transport failure) continues to produce `SensorError::Internal`)

**AC-ERR-003 — Error evidence captured in `HttpRequestFailed.detail` — non-2xx body snippet + send-failure source chain**
A unit test in `pipeline.rs`'s `#[cfg(test)] mod tests` drives the non-2xx response
branch with a mock returning HTTP 403 and a body `"forbidden"`. Asserts that the
returned `SpecEngineError::HttpRequestFailed.detail` includes BOTH the HTTP status code
(`"403"`) AND a non-empty body snippet (`"forbidden"`).
This test FAILS before the body-capture fix (Red Gate: RG-003).
**Scope extension (records-only, v1.7):** this AC explicitly covers BOTH arms of error evidence captured in `HttpRequestFailed.detail`:
- **non-2xx body arm** — verified by RG-003 (`test_pipeline_non_2xx_body_in_detail`), BC-2.16.002 Non-2xx Response Body Capture postcondition: on a non-2xx HTTP response, `HttpRequestFailed.detail` includes the HTTP status AND ≤256-byte sanitized body snippet.
- **send-failure source-chain arm** — verified by RG-009 (`test_BC_2_16_002_rg009_send_failure_includes_source_chain`), BC-2.16.002 Send-Failure Error Source Chain postcondition: on `.send()` failure (no HTTP response, `status_code: 0` path), the reqwest error SOURCE CHAIN is included in `detail` via `format!("{}{}", e, source)`, producing a `"; caused by:"` fragment.
(traces to BC-2.16.002 Non-2xx Response Body Capture postcondition: `HttpRequestFailed.detail`
MUST include HTTP status AND ≤256-byte sanitized body snippet;
also traces to BC-2.16.002 Send-Failure Error Source Chain postcondition: on `.send()` failure,
`HttpRequestFailed.detail` MUST include the reqwest error source chain)

**AC-ERR-004 — `AllTargetsFailed` is preceded by `fan_out_target_failed` WARN per target**
A unit test in `fanout.rs` with a tracing test subscriber drives `fanout()` to produce
`AllTargetsFailed` with a two-element `errors` vec. Asserts that exactly two
`fan_out_target_failed` WARN events were captured by the subscriber, each with
`event_type = "fan_out_target_failed"` and `sensor_id` matching the failed target.
This test FAILS before the WARN loop is added (Red Gate: RG-004).
(traces to BC-2.01.010 AllTargetsFailed Per-Target Logging postcondition AND BC-2.16.002
Canonical Structured Event Catalog row 91: `fan_out_target_failed` fields: `org_id`,
`sensor_id`, `attempts`, `is_transient`, `error`)

**AC-ERR-005 — In-process: sensor adapter with mock-4xx produces `ConnectivityStatus::Up`, `http_status: Some(<4xx>)` from `probe_connectivity`**
An in-process integration test constructs a `SpecDrivenSensorAdapter` whose underlying
fetch returns a mock HTTP 403 (RG-005 exemplar: `test_probe_connectivity_403_returns_up_not_down`),
then calls `probe_connectivity`. Asserts:
`ProbeOutcome { status: ConnectivityStatus::Up, http_status: Some(403), .. }`.
Any HTTP 4xx response triggers the `SensorError::HttpError` path — 403 is the RG-005
test value; the 401 live-token-expired case is verified end-to-end by AC-LIVE-002.
This test FAILS before the `map_spec_engine_error_to_sensor_error` guard is added,
because `SensorError::Internal` falls through to the `ConnectivityStatus::Down` catch-all
(Red Gate: RG-005).
**Scope extension (fix-burst pass-1):** the persistent-auth-failure path — `AuthRefreshFailed` and `CookieAuthFailed` matched DIRECTLY in Arm 2 of `map_spec_engine_error_to_sensor_error` (NOT through `HttpRequestFailed`) and mapped to `SensorError::HttpError { status: 401 }` — produces `ConnectivityStatus::Up` at the `probe_connectivity` surface (`ProbeOutcome` carries `status: Up` and `http_status`; no `auth_valid` field exists on `ProbeOutcome`). The `auth_valid: false` determination is derived DOWNSTREAM at the `SensorHealthChecker::check_one` / `probe_auth_with_routing` surface (per RG-007 / AC-WIRE-001), satisfying BC-2.01.013 EC-01-029 end-to-end.
(traces to BC-2.08.002 EC-08-006: HTTP 4xx sensor response → `auth_valid: false`;
`probe_connectivity` HttpError 4xx arm resolves `ConnectivityStatus::Up`, not Down;
also traces to BC-2.01.013 EC-01-029: persistent-auth-failure variants produce `auth_valid: false`
at the probe surface end-to-end)

**AC-ERR-006 — Infusion HTTP client build failure maps to E-INFUSE-015, not E-INFUSE-009 stopgap**
When `build_http_client_with_timeout` fails at any of the three firing paths — `load_spec`,
`load_spec_with_runtime`, or `hot_reload` — during an `HttpLookup`-type infusion spec load,
the error surfaces as `InfusionError::HttpClientBuildFailed { detail }`
with display `"E-INFUSE-015: infusion HTTP client build failed (TLS init): {detail}"` — NOT
the prior E-INFUSE-009 stopgap. Effectively unreachable in production under the ADR-050
`rustls-tls` mandate; testable by directly constructing the variant (SID-1 unit test as
compensating control per the unreachable-path pattern).
This test FAILS before `InfusionError::HttpClientBuildFailed` is introduced and the
E-INFUSE-009 stopgap mapping is removed (Red Gate: RG-013).
(traces to BC-2.19.001 §Error Conditions E-INFUSE-015 row: `build_http_client_with_timeout`
failure returns `InfusionError::HttpClientBuildFailed`; also traces to error-taxonomy
E-INFUSE-015)

**AC-ERR-007 — 5xx sensor HTTP response routes to `ConnectivityStatus::Degraded` (not Down) with `http_status: Some(5xx)`**
Two tests implement this AC. At the map layer: calling `map_spec_engine_error_to_sensor_error(SpecEngineError::HttpRequestFailed { status_code: 503, detail: "HTTP 503".to_string(), .. }, "claroty", "devices")` returns `SensorError::HttpError { sensor: "claroty", status: 503, body: "HTTP 503" }`, confirming the `status_code > 0` guard fires for 5xx (Red Gate: RG-014, `test_map_error_503_maps_to_http_error_503`).
At the `probe_connectivity` surface: an in-process integration test drives a wiremock 503 through `SpecDrivenSensorAdapter::probe_connectivity` and asserts `ProbeOutcome { status: ConnectivityStatus::Degraded, http_status: Some(503), .. }`, confirming that `SensorError::HttpError { status: 503 }` routes to `ConnectivityStatus::Degraded` (not Down) via the `status >= 500` arm in `connectivity.rs` (Red Gate: RG-014, `test_probe_connectivity_503_returns_degraded`).
Both tests MUST be RED before the 5xx→Degraded classification path is confirmed end-to-end.
(traces to BC-2.08.002 v1.7 HTTP Error Classification postcondition: `SpecEngineError::HttpRequestFailed { status_code >= 500 }` → `SensorError::HttpError { status >= 500 }` → `connectivity.rs` `status >= 500` arm → `ConnectivityStatus::Degraded`, NOT Down; the `status_code > 0` guard in `map_spec_engine_error_to_sensor_error` already covers 5xx; the Degraded classification is documented in BC-2.08.002 v1.7 §Postconditions)

### Group C — Wire-Shape Assertions

**AC-WIRE-001 — Serialized MCP JSON for AC-ERR-005 scenario contains `"reachable":true` and `"auth_valid":false`**
The integration test from AC-ERR-005 MUST additionally assert on the serialized JSON
bytes emitted at the MCP wire level, confirming the exact fields the LLM agent consumes.
Assert: the serialized JSON response contains the literal string `"reachable":true` AND
`"auth_valid":false`. NULL vs absent vs boolean distinctions asserted at the wire level
per CLAUDE.md wire-shape assertion discipline (Red Gate: RG-007).
(traces to BC-2.08.002 postcondition: auth validity response format — `auth_valid: bool`
must be a JSON boolean `false`, not absent or `null`, when HTTP 4xx is received)

### Group D — SAP-1 Compliance

**AC-SAP1-001 — `fan_out_target_failed` is registered in BC-2.16.002 Canonical Structured Event Catalog in the same commit as the `fanout.rs` implementation**
The implementation commit that adds the `tracing::warn!(event_type = "fan_out_target_failed", ...)` call in `fanout()` in `crates/prism-sensors/src/fanout.rs` MUST include a corresponding row in BC-2.16.002 §Canonical Structured Event Catalog (or BC-2.01.010 §Postconditions if product-owner routes it there). The row is already present in BC-2.16.002 v2.20 (row 91 added by product-owner in v2.14, retained through v2.20). Adversary verifies via `rg 'event_type.*fan_out_target_failed' crates/` and cross-checks against the catalog.
(traces to BC-2.16.002 Canonical Structured Event Catalog (v1.63) row 91: `fan_out_target_failed`
SAP-1 obligation; PG-LP11-001 requires catalog presence in same commit as emission site)

### Group E — Live Verification (BLOCKING — NO WAIVER)

**AC-LIVE-001 (NO WAIVER) — Direct `api.claroty.com` returns ≥1 OCSF row with relay removed**
With the relay REMOVED from `/Users/jmagady/Dev/test-soc/prism-live-mcp-wrapper.sh`
(overlay `base_url` pointed directly at `https://api.claroty.com`),
`list_sensor_data sensor=claroty table=devices` for client "monroe" returns HTTP 200 and
at least 1 OCSF-normalized row. Credentials are AI-opaque (keyring
`prism/monroe/claroty/*`); the demo-recorder agent runs the binary and MUST NOT emit
credential values. This AC confirms F10 fix is live.
(traces to BC-2.16.002 HTTP Client Compliance postcondition: ADR-050 §D5/§D6-compliant
client establishes direct HTTPS with `api.claroty.com` without relay assistance)

**AC-LIVE-002 (NO WAIVER) — Invalid/expired credential → `check_sensor_health` returns `reachable: true, auth_valid: false, detail: "401"`**
With an invalid or expired Claroty credential for client "monroe", calling
`check_sensor_health sensor=claroty` returns
`{ "reachable": true, "auth_valid": false, "detail": "HTTP 401..." }` (detail contains
"401"). This confirms F9 fix is live in the production binary against the real endpoint.
Credentials are AI-opaque; values MUST NOT be emitted by the demo-recorder.
(traces to BC-2.08.002 EC-08-006: HTTP 4xx from real sensor API → `auth_valid: false`
via the live execution path through `map_spec_engine_error_to_sensor_error`)

**AC-LIVE-003 — Relay file marked deprecated / removed after AC-LIVE-001 passes**
After AC-LIVE-001 passes, the file at
`test-soc/live-soc/relay/xdome-relay.py` is either deleted or marked with a
clear comment: `# HISTORICAL: relay was required before DEFECT-ADAPTER-TLS-XDOME-LIVE-001 (2026-08-12). Direct HTTPS now works. Do not use.`
(traces to BC-2.16.002 HTTP Client Compliance postcondition: relay is no longer
required; its presence as a load-bearing tool is retracted)

---

## Red Gate Enumeration (SAC-1)

| RG-ID | Test Name | Corresponding AC | File Location |
|-------|-----------|-----------------|---------------|
| RG-001 | `test_map_error_http_401_maps_to_http_error_not_internal` | AC-ERR-001 | `crates/prism-bin/src/spec_driven_adapter.rs` `#[cfg(test)] mod tests` |
| RG-002 | `test_map_error_status_0_maps_to_internal` | AC-ERR-002 | `crates/prism-bin/src/spec_driven_adapter.rs` `#[cfg(test)] mod tests` |
| RG-003 | `test_pipeline_non_2xx_body_in_detail` | AC-ERR-003 | `crates/prism-spec-engine/src/pipeline.rs` `#[cfg(test)] mod tests` |
| RG-004 | `test_fanout_all_failed_emits_fan_out_target_failed_warn` | AC-ERR-004 | `crates/prism-sensors/src/fanout.rs` `#[cfg(test)] mod tests` |
| RG-005 | `test_probe_connectivity_403_returns_up_not_down` | AC-ERR-005 | `crates/prism-bin/tests/defect_adapter_tls_xdome_live_001.rs` (integration test, in-process) |
| RG-006 | `test_build_http_client_sends_user_agent_header` | AC-UA-001 | `crates/prism-bin/src/spec_driven_adapter.rs` `#[cfg(test)] mod tests` |
| RG-007 | `test_sensor_health_wire_shape_403_reachable_auth_invalid` | AC-WIRE-001 | `crates/prism-bin/tests/defect_adapter_tls_xdome_live_001.rs` (integration test, in-process) |
| RG-008 | `test_reqwest_http2_feature_active` | AC-H2-001 | `crates/prism-bin/tests/defect_adapter_tls_xdome_live_001.rs` (asserts `h2` inside the `reqwest` package block of `Cargo.lock`; NOT a whole-file grep — see AC-H2-001 observability note) |
| RG-009 | `test_BC_2_16_002_rg009_send_failure_includes_source_chain` | AC-ERR-003 | `crates/prism-spec-engine/tests/pipeline_http_integration.rs` (integration test — SAP-3 end-to-end coverage; defense-in-depth alongside RG-003 unit test) |
| RG-010 | `test_map_error_auth_refresh_failed_maps_to_http_error_401` | AC-ERR-001 (persistent-auth variant) | `crates/prism-bin/src/spec_driven_adapter.rs` `#[cfg(test)] mod tests` |
| RG-011 | `test_map_error_cookie_auth_failed_maps_to_http_error_401` | AC-ERR-001 (persistent-auth variant) | `crates/prism-bin/src/spec_driven_adapter.rs` `#[cfg(test)] mod tests` |
| RG-012 | `test_infusion_http_client_sends_prism_user_agent` | AC-UA-001 (infusion path — ADR-050 §D6 v2.1 extended scope) | `crates/prism-spec-engine/src/pipeline.rs` `mod infusion_http_client_user_agent_tests` |
| RG-013 | `test_infusion_http_client_build_failure_maps_to_e_infuse_015` | AC-ERR-006 | `crates/prism-spec-engine/tests/infusion_tests.rs` |
| RG-014 | `test_map_error_503_maps_to_http_error_503` (map-level); `test_probe_connectivity_503_returns_degraded` (end-to-end) | AC-ERR-007 | Map-level: `crates/prism-bin/src/spec_driven_adapter.rs` `#[cfg(test)] mod tests`; end-to-end: `crates/prism-bin/tests/defect_adapter_tls_xdome_live_001.rs` |

## BC-5.38.001 Density Check

**Red Gate test count:** 14 (RG-001..RG-014)
**Acceptance criteria count:** 16 (AC-H2-001, AC-UA-001, AC-UA-002, AC-CARGO-001, AC-ERR-001..007, AC-WIRE-001, AC-SAP1-001, AC-LIVE-001..003)
**Density:** 14 / 16 = 0.875 — PASSES (≥0.5 required by BC-5.38.001)

Note: AC-UA-002 and AC-CARGO-001 are verified by adversary sweep and `just check` build gate, not by standalone failing tests. AC-SAP1-001 is verified by adversary cross-check against BC-2.16.002 v2.20 catalog (catalog row already present; no additional failing test needed). AC-LIVE-001..003 are live-verification ACs verified by demo-recorder against the production binary; they do not correspond to Red Gate tests in the unit/integration sense.

**Fix-burst pass-1 additions (RG-009..RG-011):** RG-009 provides SAP-3 end-to-end integration coverage for AC-ERR-003 (send-failure source chain) from `crates/prism-spec-engine/tests/pipeline_http_integration.rs`; RG-003 remains as defense-in-depth unit test. RG-010 and RG-011 cover the persistent-auth-failure (`AuthRefreshFailed` / `CookieAuthFailed`) variants of AC-ERR-001 per BC-2.01.013 EC-01-029. MED-1 body-snippet sanitization tests were also delivered in the fix-burst — coverage lives in `crates/prism-spec-engine/tests/pipeline_http_integration.rs` (`test_BC_2_16_002_med1_non_2xx_body_sanitizes_control_chars_preserves_utf8`, `test_BC_2_16_002_f1_non_2xx_body_byte_cap_multibyte_utf8`) and `crates/prism-core/src/error.rs` (`test_sanitize_body_snippet_bytes_multibyte_byte_cap`, `test_sanitize_body_snippet_bytes_control_chars_replaced_before_byte_count`, `test_sanitize_body_snippet_bytes_under_limit_passthrough`, `test_sanitize_body_snippet_bytes_empty`); these exercise the sanitize path of AC-ERR-003 / EC-003 and provide additional density beyond this count.

**Pass-2 addition (RG-012):** RG-012 (`test_infusion_http_client_sends_prism_user_agent` in `crates/prism-spec-engine/src/pipeline.rs` `mod infusion_http_client_user_agent_tests`) was added by the OBS-4 fix in LOCAL adversary pass-2. It verifies ADR-050 §D6 v2.1 extended scope: the infusion `HttpLookupSource` outbound client factory (`build_http_client_with_timeout` in pipeline.rs) MUST send `User-Agent: prism/<version>`. Registered as a Red Gate against AC-UA-001 (infusion path).

**F-2 completion addition (RG-013):** RG-013 (`test_infusion_http_client_build_failure_maps_to_e_infuse_015` in `crates/prism-spec-engine/tests/infusion_tests.rs`) verifies that `InfusionError::HttpClientBuildFailed` displays as `"E-INFUSE-015: infusion HTTP client build failed (TLS init): {detail}"` and is NOT the retired E-INFUSE-009 stopgap. Because the code path is effectively unreachable under the ADR-050 `rustls-tls` mandate, the test directly constructs the variant per SID-1 unit-test-as-compensating-control pattern. Registered as a Red Gate against AC-ERR-006; traces to BC-2.19.001 §Error Conditions E-INFUSE-015 row.

**Test-writer addition (RG-014):** RG-014 (`test_map_error_503_maps_to_http_error_503` in `crates/prism-bin/src/spec_driven_adapter.rs` `#[cfg(test)] mod tests` and `test_probe_connectivity_503_returns_degraded` in `crates/prism-bin/tests/defect_adapter_tls_xdome_live_001.rs`) was added by the test-writer to cover a behavior this story activates but previously left untested: `SpecEngineError::HttpRequestFailed { status_code >= 500 }` → `SensorError::HttpError { status >= 500 }` → `connectivity.rs` `status >= 500` arm → `ConnectivityStatus::Degraded` (NOT Down) per BC-2.08.002 v1.7 §Postconditions HTTP Error Classification. The map-level test verifies the `status_code > 0` guard fires correctly for 5xx; the end-to-end test verifies the `probe_connectivity` surface resolves `ConnectivityStatus::Degraded` not `Down`. Registered as a Red Gate against AC-ERR-007.

**Ordering rule (SAC-1 rule 3):** All Red Gate test authoring tasks (Phase 1 below) MUST be dispatched and completed BEFORE implementation tasks (Phase 2). The test-writer agent works from this story's RG-001..RG-014 list and the BC texts; the implementer receives the failing test suite before touching production code.

---

## Tasks

### Phase 1: Red Gate Tests (test-writer — BEFORE Phase 2)

- [ ] **RG-001** `test_map_error_http_401_maps_to_http_error_not_internal`: in `spec_driven_adapter.rs` inline tests; call `map_spec_engine_error_to_sensor_error(SpecEngineError::HttpRequestFailed { status_code: 401, detail: "HTTP 401".to_string(), .. }, "claroty", "<table_name>")`; assert returns `SensorError::HttpError { sensor: "claroty", status: 401, body: "HTTP 401" }`. Must be RED before Phase 2.
- [ ] **RG-002** `test_map_error_status_0_maps_to_internal`: in `spec_driven_adapter.rs` inline tests; call with `status_code: 0, detail: "connection reset".to_string()`; assert returns `SensorError::Internal { .. }`. Must be RED (or pass once guard exists but status_code=0 branch is correct).
- [ ] **RG-003** `test_pipeline_non_2xx_body_in_detail`: in `pipeline.rs` inline tests; mock HTTP 403 response with body `b"forbidden"`; assert `HttpRequestFailed.detail` contains `"403"` and `"forbidden"`. Must be RED before Phase 2.
- [ ] **RG-004** `test_fanout_all_failed_emits_fan_out_target_failed_warn`: in `fanout.rs` inline tests (`mod fan_out_target_failed_warn_tests`); use `tracing_test` subscriber; drive `fanout()` to produce `AllTargetsFailed` with two-element `errors` vec; assert exactly two `fan_out_target_failed` WARN events captured. Must be RED before Phase 2.
- [ ] **RG-005** `test_probe_connectivity_403_returns_up_not_down`: in-process integration test (`crates/prism-bin/tests/defect_adapter_tls_xdome_live_001.rs`); drive mock-403 through `SpecDrivenSensorAdapter`; call `probe_connectivity`; assert `status: ConnectivityStatus::Up`, `http_status: Some(403)`. Must be RED before Phase 2.
- [ ] **RG-006** `test_build_http_client_sends_user_agent_header`: in `spec_driven_adapter.rs` inline tests; build client via `build_http_client_with_custom_timeout(Duration::from_millis(1))`; issue request to local wiremock that captures headers; assert `User-Agent` starts with `"prism/"`. Must be RED before Phase 2.
- [ ] **RG-007** `test_sensor_health_wire_shape_403_reachable_auth_invalid`: in-process integration test (`crates/prism-bin/tests/defect_adapter_tls_xdome_live_001.rs`) extending RG-005; serialize the `ProbeOutcome` to JSON via the MCP wire path; assert the byte string contains `"reachable":true` AND `"auth_valid":false`. Must be RED before Phase 2.
- [ ] **RG-008** `test_reqwest_http2_feature_active`: verify reqwest's `http2` feature is active on the reqwest node — via `cargo tree -e features -i reqwest` (assert `http2` present on reqwest) or a test asserting `h2` appears inside the `[[package]]`/`name = "reqwest"` `dependencies` block of `Cargo.lock` (currently absent — verified in the D-1110 remove-uncertainty pass). Must be RED before the http2 feature is added. **Do NOT use a whole-file `grep '"h2"' Cargo.lock`** — `h2 0.4.13` is already present transitively via `hyper 1.9.0`, so a whole-file match is green before the fix (see AC-H2-001 observability note). Acceptable as a CI build-gate check.
- [ ] **RG-009** `test_BC_2_16_002_rg009_send_failure_includes_source_chain`: integration test in `crates/prism-spec-engine/tests/pipeline_http_integration.rs`; drives the pipeline send-failure path end-to-end with a mock HTTP endpoint that aborts the connection; asserts `HttpRequestFailed.detail` contains the source-chain `"; caused by:"` fragment. SAP-3 end-to-end coverage (public-surface entry, not synthetic AST injection); complements RG-003 unit test. Must be RED before source-chain fix. Traces to AC-ERR-003; BC-2.16.002 Send-Failure Error Source Chain postcondition.
- [ ] **RG-010** `test_map_error_auth_refresh_failed_maps_to_http_error_401`: in `spec_driven_adapter.rs` inline tests; call `map_spec_engine_error_to_sensor_error` with a `SpecEngineError::AuthRefreshFailed{..}` variant directly (Arm 2, NOT through `HttpRequestFailed`); assert returns `SensorError::HttpError { sensor: ..., status: 401, body: ... }`. Must be RED before the Arm 2 guard handles this path. Traces to AC-ERR-001 (persistent-auth variant); BC-2.01.013 EC-01-029; BC-2.08.002 HTTP Error Classification.
- [ ] **RG-011** `test_map_error_cookie_auth_failed_maps_to_http_error_401`: in `spec_driven_adapter.rs` inline tests; same structure as RG-010 but for a `SpecEngineError::CookieAuthFailed{..}` variant directly (Arm 2, NOT through `HttpRequestFailed`). Must be RED before the Arm 2 guard handles this path. Traces to AC-ERR-001 (persistent-auth variant); BC-2.01.013 EC-01-029; BC-2.08.002 HTTP Error Classification.
- [ ] **RG-012** `test_infusion_http_client_sends_prism_user_agent`: in `crates/prism-spec-engine/src/pipeline.rs` `mod infusion_http_client_user_agent_tests`; builds a client via the infusion `HttpLookupSource` client factory (`build_http_client_with_timeout`); asserts outgoing `User-Agent` header begins with `"prism/"`. Must be RED before the `.user_agent(...)` call is added to pipeline.rs. Traces to AC-UA-001 (infusion path); ADR-050 §D6 v2.1 extended scope (`build_http_client_with_timeout` in prism-spec-engine is now in-scope per v2.1 sibling-sweep addition). Added by OBS-4 fix in LOCAL adversary pass-2.
- [ ] **RG-013** `test_infusion_http_client_build_failure_maps_to_e_infuse_015`: in `crates/prism-spec-engine/tests/infusion_tests.rs`; directly construct `InfusionError::HttpClientBuildFailed { detail: "forced test failure for TLS init".to_string() }` per SID-1 compensating-control pattern (path is effectively unreachable under ADR-050 rustls-tls mandate); assert (1) `display.starts_with("E-INFUSE-015:")`, (2) display contains the `detail` string. The E-INFUSE-009→E-INFUSE-015 stopgap retirement is verified by INSPECTION of the three `.map_err(|e| InfusionError::HttpClientBuildFailed { detail: e })` sites in `crates/prism-spec-engine/src/infusion/mod.rs` (`load_spec`, `load_spec_with_runtime`, `hot_reload`) — not by a runtime assertion — because the build-failure path is unreachable under the ADR-050 rustls-tls mandate (SID-1 compensating control). Must be RED before `InfusionError::HttpClientBuildFailed` variant is introduced. Traces to AC-ERR-006; BC-2.19.001 §Error Conditions E-INFUSE-015 row; error-taxonomy E-INFUSE-015.
- [ ] **RG-014** `test_map_error_503_maps_to_http_error_503` (map-level) + `test_probe_connectivity_503_returns_degraded` (end-to-end): Map-level — in `spec_driven_adapter.rs` inline tests; call `map_spec_engine_error_to_sensor_error(SpecEngineError::HttpRequestFailed { status_code: 503, detail: "HTTP 503".to_string(), .. }, "claroty", "devices")`; assert returns `SensorError::HttpError { sensor: "claroty", status: 503, body: "HTTP 503" }`. End-to-end — in-process integration test (`crates/prism-bin/tests/defect_adapter_tls_xdome_live_001.rs`); drive wiremock-503 through `SpecDrivenSensorAdapter::probe_connectivity`; assert `ProbeOutcome { status: ConnectivityStatus::Degraded, http_status: Some(503), .. }` (NO `auth_valid` field on `ProbeOutcome`). Both MUST be RED before Phase 2. Traces to AC-ERR-007; BC-2.08.002 v1.7 HTTP Error Classification postcondition (5xx → `ConnectivityStatus::Degraded`, not Down).

### Phase 2: Implementation (implementer — AFTER Phase 1 RED gate confirmed)

**T-A: Cargo.toml Feature Edits**
- [ ] **T-A01** `crates/prism-spec-engine/Cargo.toml`: Add `"http2"` to `reqwest` features. Current features list: `["json", "rustls-tls", "gzip", "deflate", "brotli"]`. New: `["json", "rustls-tls", "http2", "gzip", "deflate", "brotli"]`.
- [ ] **T-A02** `crates/prism-sensors/Cargo.toml`: Add `"http2"` to `reqwest` features. Current: `["json", "rustls-tls"]`. New: `["json", "rustls-tls", "http2"]`.
- [ ] **T-A03** `crates/prism-bin/Cargo.toml` — sensor adapter entry (comment references S-PLUGIN-PREREQ-D AC-9): Add `"http2"` to `["rustls-tls"]`. New: `["rustls-tls", "http2"]`.
- [ ] **T-A04** `crates/prism-bin/Cargo.toml` — `[dev-dependencies]` reqwest entry: explicitly declares `"http2"` in its features array (`["json", "rustls-tls", "http2"]`); NO modification required by this story. Verify the entry still lists `"http2"` after T-A01..T-A03 edits (expected: already present).

**T-B: User-Agent (spec_driven_adapter.rs)**
- [ ] **T-B01** Symbol `build_http_client_with_custom_timeout` in `crates/prism-bin/src/spec_driven_adapter.rs`: Insert `.user_agent(concat!("prism/", env!("CARGO_PKG_VERSION")))` in the `reqwest::Client::builder()` chain before `.timeout(timeout)`. `concat!` produces a `&'static str`, zero allocation. This single change propagates to: `build_http_client_with_timeout` (thin wrapper) and all `SpecDrivenSensorAdapter` fetch clients ONLY (within prism-bin). `DeclarativeHttpAuthProvider` (prism-spec-engine) is NOT a propagation target of this prism-bin change — it obtains its UA from the independent `prism-spec-engine::pipeline::build_http_client_with_timeout` sibling, which carries its own `.user_agent()` call (cross-crate dependency from prism-bin to prism-spec-engine is impossible; the two paths are patched independently per BC-2.16.014 INV-014-007 two-path description). Do NOT cite file line numbers — cite the symbol name `build_http_client_with_custom_timeout` per TD-VSDD-091.

**T-C: User-Agent (boot.rs)**
- [ ] **T-C01** Symbol sites in `crates/prism-bin/src/boot.rs` — two `reqwest::Client::builder()` chains producing the `PluginRuntime` HTTP client: (a) the `PRISM_DISABLE_PLUGIN_LOAD` fast-path builder and (b) the normal-path builder. Add `.user_agent(concat!("prism/", env!("CARGO_PKG_VERSION")))` to each chain before `.build()`. Cite by symbol/context, not line numbers (TD-VSDD-091).

**T-D: Send-Failure Error Source Chain (pipeline.rs)**
- [ ] **T-D01** In `crates/prism-spec-engine/src/pipeline.rs`, at each send-failure site (both the first-request send and the 401-retry send) where `status_code: 0` is set and `detail: e.to_string()` is used, replace `e.to_string()` with the source-chain format:
  ```rust
  format!(
      "{}{}",
      e,
      std::error::Error::source(&e)
          .map(|s| format!("; caused by: {s}"))
          .unwrap_or_default()
  )
  ```
  This includes the hyper/h2/TLS-level source chain that `.to_string()` omits. Apply symmetrically to BOTH send sites (first request and 401-retry). Cite by symbol context (symbol: send-failure arm in `issue_request_with_retry` function), not file line numbers.

**T-E: Non-2xx Response Body Capture (pipeline.rs)**
- [ ] **T-E01** In `crates/prism-spec-engine/src/pipeline.rs`, at the `if !status.is_success()` branch that currently returns `detail: format!("HTTP {status}")`, add best-effort body capture:
  - Read the response body bytes: `.bytes().await.ok()`
  - Cap to ≤256 bytes; strip control characters via `prism_core::sanitize_body_snippet_bytes` (byte-based, ≤256-byte cap, `floor_char_boundary` truncation — per BC-2.16.002 Non-2xx Response Body Capture postcondition)
  - Update `detail` to: `format!("HTTP {status}: {sanitized_body}")` (falls back to `format!("HTTP {status}")` if body is empty)
  - A secondary body-read failure MUST NOT replace the primary status-code error
  - Apply this fix symmetrically to BOTH non-2xx branches (first-request non-2xx and 401-retry non-2xx). Cite by symbol context (the `is_success()` check in `issue_request_with_retry`), not line numbers.

**T-F: Error Mapping Fix (spec_driven_adapter.rs)**
- [ ] **T-F01** Symbol `map_spec_engine_error_to_sensor_error` in `crates/prism-bin/src/spec_driven_adapter.rs`: Insert a guard arm before the `SensorError::Internal` return:
  ```rust
  if let SpecEngineError::HttpRequestFailed { status_code, ref detail, .. } = e {
      if status_code > 0 {
          return SensorError::HttpError {
              sensor: sensor_id.to_string(),
              status: status_code,
              body: detail.clone(),
          };
      }
  }
  // status_code = 0 (transport error) and all other SpecEngineError variants → Internal
  SensorError::Internal { detail: format!("SpecDrivenSensorAdapter: {e}") }
  ```
  Discriminator: `status_code = 0` = no HTTP response (transport failure) → `Internal`; `status_code > 0` = HTTP response received → `HttpError`. The `probe_connectivity` `HttpError 4xx` arm then correctly resolves `ConnectivityStatus::Up` (via `ProbeOutcome.status`); `auth_valid: false` is determined DOWNSTREAM at the `SensorHealthChecker::check_one` / `probe_auth_with_routing` surface.

**T-G: AllTargetsFailed Per-Target Logging (fanout.rs)**
- [ ] **T-G01** Symbol site constructing `SensorError::AllTargetsFailed { count, errors }` in `crates/prism-sensors/src/fanout.rs`. Before returning `Err(SensorError::AllTargetsFailed { count, errors })`, insert the WARN loop:
  ```rust
  for err in &errors {
      tracing::warn!(
          event_type = "fan_out_target_failed",
          org_id = %err.org_id,
          sensor_id = %err.sensor_id,
          attempts = err.retry_metadata.attempts,
          is_transient = err.retry_metadata.is_transient,
          error = %err,
          "fan-out target failed"
      );
  }
  ```
  SAP-1 obligation: `fan_out_target_failed` catalog row 91 is already present in BC-2.16.002 v2.20 (row 91 added by product-owner in v2.14, retained through v2.20). Adversary verifies catalog↔emission match via `rg 'event_type.*fan_out_target_failed' crates/` per AC-SAP1-001. The `AllTargetsFailed` Display MUST remain count-only (`"E-SENSOR-030: all fan-out targets failed ({count} errors)"`) per BC-2.10.007 Rule 1.

**T-H: Run Red Gate tests — confirm all GREEN**
- [ ] Run `just iter prism-bin` and `just iter prism-sensors` and `just iter prism-spec-engine` to confirm all Red Gate tests pass green.
- [ ] Run `just check` to confirm `just check` passes including h2 in `Cargo.lock`.

### Phase 3: Live Verification (demo-recorder — AFTER Phase 2 and LOCAL 3-CLEAN)

- [ ] **T-LV01** Remove relay from `/Users/jmagady/Dev/test-soc/prism-live-mcp-wrapper.sh`; set overlay `base_url` directly to `https://api.claroty.com`. Run `list_sensor_data sensor=claroty table=devices` for client "monroe". Assert ≥1 OCSF row returned. Credentials AI-opaque from keyring; values MUST NOT be emitted. Closes AC-LIVE-001.
- [ ] **T-LV02** With invalid/expired Claroty credential, run `check_sensor_health sensor=claroty` for client "monroe". Assert `reachable: true, auth_valid: false` with "401" in detail. Closes AC-LIVE-002.
- [ ] **T-LV03** Mark `test-soc/live-soc/relay/xdome-relay.py` deprecated with historical comment. Closes AC-LIVE-003.

---

## Architecture Mapping

| Component | Module | File | Pure/Effectful |
|-----------|--------|------|---------------|
| `build_http_client_with_custom_timeout` (UA + h2 fix) | `prism-bin` | `crates/prism-bin/src/spec_driven_adapter.rs` | Effectful (constructs `reqwest::Client` with I/O-capable state) |
| PluginRuntime client builders (UA fix) | `prism-bin` | `crates/prism-bin/src/boot.rs` | Effectful (constructs `reqwest::Client` with I/O-capable state) |
| `map_spec_engine_error_to_sensor_error` (error mapping fix) | `prism-bin` | `crates/prism-bin/src/spec_driven_adapter.rs` | Pure (pattern match → variant construction; no I/O) |
| `issue_request_with_retry` send-failure arm (source-chain fix) | `prism-spec-engine` | `crates/prism-spec-engine/src/pipeline.rs` | Effectful (async HTTP I/O) |
| `issue_request_with_retry` non-2xx branch (body-capture fix) | `prism-spec-engine` | `crates/prism-spec-engine/src/pipeline.rs` | Effectful (async HTTP I/O + best-effort body read) |
| `fanout()` AllTargetsFailed WARN loop | `prism-sensors` | `crates/prism-sensors/src/fanout.rs` | Effectful (tracing WARN emission; structured side effect) |

## Purity Classification

- **Pure functions** (no I/O, deterministic): `map_spec_engine_error_to_sensor_error`, the new guard arm logic, error-format string construction in T-D01/T-E01.
- **Effectful functions** (I/O, network, tracing): all `reqwest::Client::builder()` chains, `issue_request_with_retry` (HTTP), `fanout()` (async fan-out + tracing). Test doubles use wiremock/tracing_test subscriber to isolate effects.

---

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Non-2xx body-read fails (`bytes().await` returns `Err`) | Best-effort: fall back to `"HTTP {status}"` without body snippet; primary status-code error is NOT replaced |
| EC-002 | Non-2xx body is empty (`b""`) | `detail` is `"HTTP {status}"` with no body snippet appended |
| EC-003 | Non-2xx body exceeds 256 bytes | Body capped at 256 bytes before inclusion in detail; control chars stripped; body snippet truncated gracefully |
| EC-004 | `status_code = 0` reaches `map_spec_engine_error_to_sensor_error` (transport failure — TCP reset, DNS failure, TLS handshake failure) | Guard arm skipped; `SensorError::Internal` returned; `probe_connectivity` resolves `ConnectivityStatus::Down` (correct) |
| EC-005 | `SpecEngineError` variant other than `HttpRequestFailed` reaches `map_spec_engine_error_to_sensor_error` | Guard arm skipped (`if let` does not match); falls through to `SensorError::Internal` (existing behavior preserved) |
| EC-006 | `AllTargetsFailed` with empty `errors` vec | WARN loop does not execute (zero iterations); no `fan_out_target_failed` events emitted; `AllTargetsFailed` is returned as before |
| EC-007 | Fan-out produces N > `MAX_FANOUT_CONCURRENCY` (10) failed targets | WARN loop emits N WARN events; N is bounded by `MAX_FANOUT_CONCURRENCY = 10`; no rate-limit issue |
| EC-008 | reqwest error chain has no `source()` (e.g., simple `InvalidUrl`) | `std::error::Error::source(&e).map(...)` returns `None`; `.unwrap_or_default()` produces `""`; detail is `format!("{e}")` (no `"; caused by:"` suffix) |

---

## Token Budget Estimate

| Artifact | Estimated Tokens | Notes |
|----------|-----------------|-------|
| This story file | ~6,000 | |
| BC-2.16.002 v2.20 (HTTP Client Compliance + AllTargetsFailed postconditions + catalog row 91) | ~30,000 | Large BC; catalog scope is wide |
| BC-2.08.002 v1.7 (HTTP Error Classification postcondition + AuthRefreshFailed/CookieAuthFailed scope + 5xx classification correction) | ~6,500 | Targeted amendment + persistent-auth variants |
| BC-2.01.010 v1.6 (AllTargetsFailed Per-Target Logging postcondition) | ~5,000 | |
| BC-2.01.013 v1.19 (DataSource Trait Eliminates Per-Sensor Code Duplication — scope: EC-01-029) | ~4,000 | Targeted EC row; fix-burst pass-1 alignment |
| BC-2.16.014 v1.22 (INV-014-007 ADR-050 §D5/§D6 note) | ~18,000 | Large BC; only INV note relevant |
| BC-2.19.001 v2.4 (§Error Conditions E-INFUSE-015 row — targeted scope) | ~3,000 | Only E-INFUSE-015 row and adjacent context needed; full BC is large |
| ADR-050 v2.3 (§D5/§D6 new decisions + rationale; §D6 scope extended in v2.1; §D5 production-entry count corrected to 3 in v2.2) | ~10,000 | Reference for Cargo.toml + UA changes |
| `spec_driven_adapter.rs` (`map_spec_engine_error_to_sensor_error` + `build_http_client_with_custom_timeout`) | ~8,000 | Two target functions |
| `pipeline.rs` (send-failure arm + non-2xx branch × 2) | ~30,000 | Large file; two symmetric fix sites |
| `fanout.rs` (`AllTargetsFailed` construction site) | ~8,000 | |
| `boot.rs` (two PluginRuntime builder sites) | ~5,000 | |
| Cargo.toml files × 3 (prism-spec-engine, prism-sensors, prism-bin) | ~3,000 | Small changes |
| Test infrastructure (tracing_test subscriber, wiremock, reqwest test client) | ~5,000 | Existing deps |
| error-taxonomy.md E-SENSOR-030 row | ~2,000 | Reference for AllTargetsFailed Display contract |
| `prism-core/src/error.rs` (sanitize functions + InfusionError variant) | ~4,000 | sanitize_body_snippet_bytes / sanitize_body_snippet + InfusionError::HttpClientBuildFailed |
| `prism-core/src/lib.rs` (re-export) | ~1,000 | sanitize_body_snippet + sanitize_body_snippet_bytes pub use re-exports |
| `prism-mcp/src/health/connectivity.rs` (sanitize_error delegation) | ~3,500 | char-based ≤512-cap delegation to prism_core::sanitize_body_snippet |
| **Total estimated** | **~152,000** | Updated for prism-core and prism-mcp additions; well within one context window |

---

## Previous Story Intelligence

Prior art from closely related stories:

- `PLUGIN-MIGRATION-001-D` (merged): pattern for `pipeline.rs` call-site changes. Use `just iter prism-spec-engine` for inner-loop iteration; full `just check` only once at end of fix-burst per TDD inner-loop discipline (CLAUDE.md §Build & Test).
- `S-DEMO-FIDELITY-REMEDIATION-001` (merged, PR cf66151f): first application of ADR-050 D1-D4. Pattern for Cargo.toml reqwest feature edits across multiple crates — copy the feature list edit pattern from that commit.
- `PLUGIN-MIGRATION-001-B` / `PLUGIN-MIGRATION-001-C`: patterns for `spec_driven_adapter.rs` changes involving `SensorError` variants and `map_spec_engine_error_to_sensor_error`.
- `S-DEMO-CLAROTY-PAGINATION-001`: pattern for in-process integration tests with a mock `SpecDrivenSensorAdapter`; reuse the test harness structure.

`DEFECT-SENSOR-ERROR-FLATTEN-001` (stub, superseded): this story supersedes it per the §5 Bundling Verdict in the design doc. `DEFECT-SENSOR-ERROR-FLATTEN-001` had `status: draft` and no ACs authored; it is closed as superseded by this story.

---

## Architecture Compliance Rules

Violations of these rules are P1 findings in adversarial review.

1. **ADR-050 §D1/§D2 — rustls-tls mandatory, native-tls FORBIDDEN**: No Cargo.toml entry must switch to `native-tls`, `default-tls`, `native-tls-alpn`, or `native-tls-vendored`. The `http2` feature is additive alongside the existing `rustls-tls`. Do NOT remove `default-features = false`.

2. **ADR-050 §D5/§D6 scope boundary — DTU dev-deps excluded**: The `http2` feature and `.user_agent(...)` call are required in production `[dependencies]` entries only. DTU crate `[dev-dependencies]` reqwest entries MUST NOT be modified by this story.

3. **ADR-022 / Arc-DI — no placeholder-construct anti-pattern**: No new `Arc::new(Something::placeholder())` in any production boot path. This story adds only error-propagation guards, a feature flag, and a builder chain call — no new Arc-injected types.

4. **TD-VSDD-091 / Anti-volatile-pin — no `file.rs:NNN` line cites**: All implementation guidance (task descriptions, comments) MUST cite function/symbol names and behavioral anchors, NOT line numbers. The two send-failure sites and two non-2xx sites in `pipeline.rs` are cited by symbol (`issue_request_with_retry`) and branch semantics, not line numbers.

5. **AD-017 / AI-opaque credentials — no credential values in tracing events**: The `fan_out_target_failed` WARN event's `error` field uses `%err` (the `FanOutError` Display), which MUST NOT include credential values. The `org_id` and `sensor_id` are config identifiers (safe per AD-017). Live-verification agents run the binary from keyring credentials and MUST NOT echo values.

6. **BC-5.39.001 / Spec-first gate**: Do NOT amend any spec artifact (BC, ADR, error-taxonomy) from the implementation task. BCs were amended by product-owner and ADR by architect pre-story (see §Authority). If a spec defect is discovered during implementation, STOP and report to orchestrator per CLAUDE.md Companion Principle rule 2.

7. **SAP-1 standing probe — event catalog completeness**: `fan_out_target_failed` catalog row 91 is already present in BC-2.16.002 v2.20 (added in v2.14). The adversary verifies this in every cascade pass: `rg 'event_type.*fan_out_target_failed' crates/` must produce exactly one match (in `fanout.rs`); BC-2.16.002 row 91 must be present. A mismatch (emission without catalog row or catalog row without emission) is a P1 finding.

8. **SAP-3 standing probe — spec-arm reachability**: RG-005 and RG-007 are in-process integration tests that drive `probe_connectivity` from the public surface (not synthetic AST injection). This satisfies SAP-3 primary coverage. RG-001 and RG-002 (unit tests of `map_spec_engine_error_to_sensor_error` directly) are defense-in-depth per SAP-3 rule 2.

9. **SID-2 / Composed-output assertions**: RG-007 asserts on the FULL serialized JSON wire bytes (`"reachable":true` and `"auth_valid":false` as literal substrings), not only on pre-serialization Rust struct fields. Component-only assertions on `ProbeOutcome` fields are insufficient; AC-WIRE-001 requires wire-level JSON assertion.

10. **`#[non_exhaustive]` discipline — no new pub types**: This story adds no new public TOML-deserialized types or pub-API surface types. `EXPECTED_SYMBOLS` in `scripts/check-non-exhaustive-per-symbol.py` does not change. `scripts/check-non-exhaustive.sh` must pass; the gate is Layer 1 equality, so both a removed annotation and an unregistered new type fail CI.

11. **prism-sensors perimeter**: `crates/prism-sensors` MUST NOT gain a dependency on `prism-query` or `prism-mcp`. The `fanout.rs` WARN addition uses only `tracing` (already in the dep graph). Verify `crates/prism-sensors/Cargo.toml` after T-G01.

---

## Library & Framework Requirements

| Library | Version / Source | Purpose |
|---------|-----------------|---------|
| `reqwest` | `default-features = false, features = ["rustls-tls", "http2", ...]` per each Cargo.toml | h2 ALPN negotiation (ADR-050 §D5) |
| `tracing` | as pinned in workspace `Cargo.toml` | `fan_out_target_failed` WARN event (T-G01) |
| `tracing-test` | as pinned in workspace `Cargo.toml` (or `tracing_subscriber` test subscriber) | RG-004 subscriber capture |
| `wiremock` | as pinned in workspace `Cargo.toml` (existing dep in prism-bin tests) | RG-005, RG-006, RG-007 header/response mocking |
| `std::error::Error` | stdlib | `source()` chain traversal in T-D01 |

**No new crate dependencies added.** All libraries above are existing workspace deps. Do not introduce new crate dependencies in any of the three affected Cargo.toml files beyond the `"http2"` reqwest feature addition.

---

## File Structure Requirements

### Files to CREATE

None. All changes are additions to existing production modules and inline test blocks.

### Files to MODIFY

| File | Change Summary |
|------|----------------|
| `crates/prism-spec-engine/Cargo.toml` | Add `"http2"` to `reqwest` features (T-A01) |
| `crates/prism-sensors/Cargo.toml` | Add `"http2"` to `reqwest` features (T-A02) |
| `crates/prism-bin/Cargo.toml` | Add `"http2"` to the production reqwest entry (T-A03); verify dev-dep entry unchanged (T-A04) |
| `crates/prism-bin/src/spec_driven_adapter.rs` | Add `.user_agent(...)` to `build_http_client_with_custom_timeout` (T-B01); add guard arm to `map_spec_engine_error_to_sensor_error` (T-F01); add RG-001, RG-002, RG-006, RG-010, RG-011, RG-014 (map-level: `test_map_error_503_maps_to_http_error_503`) inline tests |
| `crates/prism-bin/src/boot.rs` | Add `.user_agent(...)` to two PluginRuntime builder sites (T-C01) |
| `crates/prism-spec-engine/src/pipeline.rs` | Replace `e.to_string()` with source-chain format at two send-failure sites (T-D01); add body-capture at two non-2xx branches (T-E01); add RG-003 inline test; add `.user_agent(...)` to infusion `build_http_client_with_timeout` builder (ADR-050 §D6 v2.1); add RG-012 test in `mod infusion_http_client_user_agent_tests` |
| `crates/prism-sensors/src/fanout.rs` | Add per-target WARN loop before `AllTargetsFailed` return (T-G01); add RG-004 inline test |
| `crates/prism-bin/tests/defect_adapter_tls_xdome_live_001.rs` | Add RG-005 (`test_probe_connectivity_403_returns_up_not_down`), RG-007 (`test_sensor_health_wire_shape_403_reachable_auth_invalid`), RG-008 (`test_reqwest_http2_feature_active`), and RG-014 (`test_probe_connectivity_503_returns_degraded`) in-process integration tests |
| `crates/prism-spec-engine/tests/pipeline_http_integration.rs` | Add RG-009 (`test_BC_2_16_002_rg009_send_failure_includes_source_chain`) integration test (fix-burst pass-1); add MED-1 sanitization tests `test_BC_2_16_002_med1_non_2xx_body_sanitizes_control_chars_preserves_utf8` and `test_BC_2_16_002_f1_non_2xx_body_byte_cap_multibyte_utf8` (fix-burst pass-1) |
| `crates/prism-core/src/error.rs` | Add `InfusionError::HttpClientBuildFailed { detail: String }` variant with `#[error("E-INFUSE-015: infusion HTTP client build failed (TLS init): {detail}")]` (F-2 E-INFUSE-015 completion); add `sanitize_body_snippet_bytes` (byte-based, ≤256-byte cap, `floor_char_boundary` truncation, control-char strip) and `sanitize_body_snippet` (char-based, ≤512-char cap) utility functions called by pipeline.rs non-2xx body-capture (T-E01) |
| `crates/prism-core/src/lib.rs` | Add `pub use error::{sanitize_body_snippet, sanitize_body_snippet_bytes}` re-exports (single `pub use` block) so callers reach `prism_core::sanitize_body_snippet` (connectivity.rs char-based path) and `prism_core::sanitize_body_snippet_bytes` (pipeline.rs byte-based path) without reaching into the `error` submodule directly |
| `crates/prism-mcp/src/health/connectivity.rs` | Update `sanitize_error` function to delegate to `prism_core::sanitize_body_snippet` (char-based ≤512-char cap at this call site, vs pipeline.rs byte-based ≤256-byte cap via `sanitize_body_snippet_bytes`); add doc comment referencing this story |
| `crates/prism-spec-engine/src/infusion/mod.rs` | Rewire 3 `build_http_client_with_timeout()` call sites: `load_spec` (1 site), `load_spec_with_runtime` (1 site), and `hot_reload` (1 site): `.map_err(\|e\| InfusionError::HttpClientBuildFailed { detail: e })` replacing E-INFUSE-009 stopgap; error-taxonomy v2.74 |
| `crates/prism-spec-engine/tests/infusion_tests.rs` | Add RG-013 (`test_infusion_http_client_build_failure_maps_to_e_infuse_015`) — direct variant construction + Display prefix + detail interpolation assertions (SID-1 pattern); stopgap retirement confirmed by inspection of `infusion/mod.rs` mapping sites |
| `test-soc/live-soc/relay/xdome-relay.py` | Add deprecation comment (T-LV03, post-live-verification) |

### Files NOT to Modify

| File | Reason |
|------|--------|
| `.factory/specs/behavioral-contracts/BC-2.16.002-multi-step-fetch-pipeline.md` | Frozen at v2.20. Transport postconditions and catalog row 91 already authored by product-owner. |
| `.factory/specs/behavioral-contracts/BC-2.08.002-auth-validity-check.md` | Frozen at v1.7. HTTP Error Classification postcondition (including AuthRefreshFailed/CookieAuthFailed scope and 5xx classification correction) already authored. |
| `.factory/specs/behavioral-contracts/BC-2.01.010-partial-failure-handling.md` | Frozen at v1.6. AllTargetsFailed Per-Target Logging postcondition already authored. |
| `.factory/specs/behavioral-contracts/BC-2.01.013-datasource-trait-adapter-pattern.md` | Frozen at v1.19. EC-01-029 persistent-auth-failure alignment already authored by product-owner. |
| `.factory/specs/behavioral-contracts/BC-2.16.014-declarative-auth-acquisition-token-lifecycle.md` | Frozen at v1.22. INV-014-007 note already added by product-owner. |
| `.factory/specs/behavioral-contracts/BC-2.19.001-infusion-spec-loading.md` | Frozen at v2.4. E-INFUSE-015 §Error Conditions row already authored by product-owner. |
| `.factory/specs/architecture/decisions/ADR-050-workspace-reqwest-tls-backend.md` | Frozen at v2.3. D5 (3 production entries) and D6 decisions already added by architect. |
| `.factory/specs/prd-supplements/error-taxonomy.md` | E-SENSOR-030 row already amended by product-owner. Do NOT amend. |
| `crates/prism-dtu-*/Cargo.toml` | DTU dev-deps excluded from ADR-050 §D5 scope. |
| `scripts/check-non-exhaustive-per-symbol.py` | No new pub types introduced; count unchanged. |

---

## Forbidden Dependencies

The following modules/packages MUST NOT appear as new dependencies in any of the three affected Cargo.toml files after this story:

- `native-tls` / `default-tls` / `native-tls-alpn` / `native-tls-vendored` (ADR-050 §D2: FORBIDDEN workspace-wide; causes ~65s macOS Keychain init + MITM proxy interception path)
- `prism-query` in `prism-sensors` (perimeter violation: SS-01 fan-out MUST NOT depend on the query engine)
- `prism-mcp` in `prism-sensors` or `prism-spec-engine` (spec-engine and sensors must not depend on MCP layer)

---

## UX Screen References

N/A — no UX surface. All changes are internal to the transport, error-propagation, and sensor adapter layers. The only operator-visible outputs are structured health tool responses (`auth_valid`, `reachable`, `detail`) which are already governed by BC-2.08.002.

---

## Dependency Graph Edges

```
DEFECT-ADAPTER-TLS-XDOME-LIVE-001 (this story)
  depends_on: []           — no hard dependencies; can enter any wave
  blocks:     []           — no other stories blocked on this defect fix
```

This story supersedes `DEFECT-SENSOR-ERROR-FLATTEN-001` (stub, closed as superseded per design doc §5 Bundling Verdict).

---

## Story-Level Holdout Gate

Per the project's story-level holdout gate protocol (CLAUDE.md §Story-Level Holdout Gate, D-1715/D-1716):

The product-owner authors **2–4 HIDDEN, SINGLE-USE holdout scenarios** for this story at materialization time. These scenarios are stored in the holdout directory (not readable by test-writer or implementer) and executed by the holdout-evaluator AFTER LOCAL 3-CLEAN and BEFORE demo recording / push.

**Implementer / test-writer MUST NOT:**
- Read, reference, or attempt to infer the holdout scenario content
- Write tests designed to match the holdout scenarios
- Author "holdout-adjacent" tests based on speculation about scenario content

**The gate is BLOCKING.** Any unsatisfied scenario routes findings back through the VSDD feedback loop as OBSERVED BEHAVIOR ONLY (never scenario text). BC-5.39.001 LOCAL streak resets to 0/3 on any holdout failure.

---

## Technology Assumption Validation (Remove-Uncertainty Pass — D-1110, 2026-08-12)

Validated by research-agent against reqwest docs (Context7 / docs.rs), the workspace
`Cargo.lock` (reqwest 0.12.28, h2 0.4.13, hyper 1.9.0), the working relay source
(`test-soc/live-soc/relay/xdome-relay.py`), and web research on AWS WAF / edge TLS
fingerprinting (Perplexity deep research). Every claim is source-anchored; version
facts are pinned. Corrections and the RISK note below have been applied to the ACs and
frontmatter of this story.

### A1 — reqwest `http2` feature semantics — **CONFIRMED**

- Feature name is exactly `http2` (not `hyper-h2`). Source: docs.rs/crate/reqwest/0.12.28/features; reqwest source optional-features list (Context7).
- `http2` is **additive, not h2-only.** reqwest's default `HttpVersionPref` is `All`; with `http2` on, ALPN advertises `["h2", "http/1.1"]` and negotiates h2 only if the server offers it, else falls back to HTTP/1.1. `http2_only(true)` is set ONLY when the caller explicitly selects `HttpVersionPref::Http2` (via `.http2_prior_knowledge()`), which this story does NOT do. Source: reqwest `async_impl/client.rs` ALPN + http2 builder match arms (Context7).
- WITHOUT the feature, reqwest advertises ONLY `http/1.1` in ALPN — the `"h2"` arm is `#[cfg(feature = "http2")]`. So the current production clients are HTTP/1.1-only at ALPN, consistent with the root-cause hypothesis. Source: same ALPN match arm (Context7).
- `http2` pulls `h2`: reqwest 0.12.28's `http2` feature enables `dep:h2` (^0.4) plus `hyper/http2`, `hyper-util/http2`, `hyper-rustls/http2`. Source: docs.rs/crate/reqwest/0.12.28/features.

### A2 — `.user_agent()` API — **CONFIRMED**

- `reqwest::ClientBuilder::user_agent<V: TryInto<HeaderValue>>(self, value: V)` exists on reqwest 0.12. A `&'static str` satisfies the bound, so `.user_agent(concat!("prism/", env!("CARGO_PKG_VERSION")))` compiles (zero-allocation `&'static str`). The docs' own example is byte-for-byte this pattern (`concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"))`). Source: docs.rs/reqwest ClientBuilder::user_agent (Context7).
- reqwest sends **no** default `User-Agent` — one is present only if set via `.user_agent()` or `default_headers`. The story premise (current clients send no UA) is consistent with reqwest's documented behavior. Confidence: high. Source: docs.rs/reqwest ClientBuilder (no default-UA behavior documented); web corroboration.

### A3 — Plausibility of the fix against the api.claroty.com edge — **RISK (moderate confidence SUFFICIENT)**

Empirical evidence from the currently-working relay materially informs — and partially
mitigates — the generic WAF concern, but a residual risk remains that the human should
weigh before build.

**What the working relay tells us.** `xdome-relay.py` reaches `https://api.claroty.com`
via Python `urllib.request`. That client is: (a) **HTTP/1.1-only** (urllib does not speak
HTTP/2); (b) sends a **default `User-Agent: Python-urllib/3.x`** (the relay sets only
`Content-Type` + `Authorization`, so urllib attaches its default UA); (c) presents a
**non-browser OpenSSL TLS fingerprint**. It succeeds.

**Implications:**
1. **HTTP/2 is NOT empirically required** to pass the edge — a HTTP/1.1-only client
   already works. The `http2` feature addition is therefore an **ADR-050 §D5 compliance +
   robustness change and defense-in-depth**, NOT the empirically load-bearing fix. (It
   remains mandatory per frozen ADR-050 — do NOT remove it.)
2. The edge **accepts a non-browser TLS fingerprint** (urllib/OpenSSL). This substantially
   lowers the "the WAF requires a browser JA3/JA4" failure mode that generic analysis
   raises: rustls is also non-browser, and the edge is demonstrably not browser-whitelisting.
3. The most conspicuous difference between the working client (urllib, **has** a UA) and
   the failing client (reqwest, **no** UA) is the **`User-Agent`**. This makes the
   `.user_agent()` change the **probable load-bearing fix** and raises confidence AC-LIVE-001 can pass.

**Residual risk (why AC-LIVE-001, no-waiver, still cannot be guaranteed a priori):**
- The specific **rustls JA3/JA4** could be independently blocklisted even though urllib's
  OpenSSL fingerprint is not. AWS WAF (the likely edge behind AWS Global Accelerator for
  `api.claroty.com`) exposes JA3/JA4 as a first-class match field and can block/rate-limit
  on it; adding `h2`-ALPN + a UA changes the JA3/JA4 hash but does NOT make rustls browser-
  or urllib-identical (standard rustls offers no ClientHello camouflage — maintainers route
  fingerprint-evasion to specialized libraries). Sources: AWS WAF JA3/JA4 docs
  (docs.aws.amazon.com/waf — JA4Fingerprint / FieldToMatch / logging-fields / bot-control);
  rustls issue #1421. Probability: **LOW** (edge tolerates non-browser urllib), non-zero,
  unprovable without a live test.
- The differentiator could be a header other than UA (e.g. `Accept-Encoding`, header
  casing/ordering) that the UA change does not address. Probability: **LOW-MODERATE**.

**Recommendation (cheap, optional pre-build de-risk):** before committing to the no-waiver
live gate, run a one-off diagnostic probe against `https://api.claroty.com` from a minimal
reqwest client with ONLY `.user_agent(...)` added (HTTP/1.1, rustls). If it clears the edge
(any origin-level response, e.g. a 401 rather than a WAF 403), AC-LIVE-001 is very likely to
pass and h2 is confirmed non-load-bearing. If it still 403s at the WAF, the edge is
fingerprinting rustls and the fix is insufficient — escalate before building. Diagnostic
only; touches no production code.

**Verdict:** the bundled fix is **plausibly sufficient (moderate confidence)** — the UA
change is the probable remedy and the relay evidence rules out the strongest
"browser-JA3-required" failure mode. AC-LIVE-001 remains a genuine, non-waivable live gate
whose outcome cannot be proven before execution. Surfaced to orchestrator/human per the
remove-uncertainty discipline; does not block materialization.

### A4 — rustls + ALPN `h2` composition — **CONFIRMED**

- reqwest's `rustls-tls` + `http2` compose correctly: reqwest sets `tls.alpn_protocols` on
  the rustls `ClientConfig` from `http_version_pref`; with both features and default pref
  `All`, rustls advertises `h2` then `http/1.1` in the ClientHello ALPN extension. reqwest
  0.12.28's `http2` feature also enables `hyper-rustls/http2`, wiring the rustls connector for
  h2. Sources: reqwest `async_impl/client.rs` ALPN match arm (Context7); docs.rs/crate/reqwest/0.12.28/features.

### Correction applied — AC-H2-001 / RG-008 observable

`h2 0.4.13` is ALREADY in `Cargo.lock` (transitively via `hyper 1.9.0`; reqwest's own
`dependencies` block does NOT currently list `h2` — verified this pass). A whole-file
`grep '"h2"' Cargo.lock` is therefore green before the fix and is INVALID as a Red Gate.
AC-H2-001 and RG-008 have been corrected to scope the observable to the reqwest node
(`cargo tree -e features -i reqwest`, or `h2` inside reqwest's own Cargo.lock dependency
block). See the AC-H2-001 observability note.

---

## Version History

| Version | Date | Author | Change |
|---------|------|--------|--------|
| 1.24 | 2026-08-14 | story-writer | Records-hygiene sweep (TD-VSDD-096, TD-VSDD-091, F-P30-LOW-001 + F-P30-LOW-002). Two §Version History volatile-cite fixes. (1) F-P30-LOW-001: v1.18 row had quoted the three arm-5 L-cite values it removed from v1.15 — re-described the removal using symbol/anchor-only prose referencing AC-UA-001, T-B01, and the CORRECT-framing note; no quoted L-cites remain in the row. (2) F-P30-LOW-002: v1.22 row cited a feature-branch SHA as "Code HEAD" in backticks — replaced with SHA-free form "Code HEAD unchanged (spec-only fix)." per TD-VSDD-091. Also escaped two unescaped `\|e\|` pipes in the v1.22 row's Rust closure snippet (pre-existing table cell-count defect, same edit). `version:` v1.23 → v1.24; `# BC status` label updated to match. TD-VSDD-097 3d sweep: sibling-pair (comprehensive sweep of all §Version History rows for arm-5 L-cites and git-HEAD SHA cites — only the two declared violations found; all other hex strings confirmed as preserved input-hash content-hashes or merged-PR references) — DONE; downstream-copy (STORY-INDEX carries no copy of the volatile-cite strings — CLEAR) — CLEAR; mandate-anchor (no new MUST introduced) — N/A. No content/mechanism/algorithm/API-contract changes — records-tier prose only. |
| 1.23 | 2026-08-14 | story-writer | Records-only BC version pin propagation (TD-VSDD-096). BC-2.16.002 v2.19 → v2.20 (SHA-de-reference records-hygiene, no behavior change) propagated to 9 body locations: `# BC status` frontmatter comment, §Authority table, §Behavioral Contracts table, §BC-5.38.001 density note (AC-SAP1-001 sentence), §Tasks T-G01 SAP-1 obligation paragraph (both v2.19 occurrences on single line), §Token Budget table, §Architecture Compliance Rule 7, §Files NOT to Modify. BC-2.01.013 v1.18 → v1.19 (same) propagated to 5 body locations: `# BC status` frontmatter comment, §Authority table, §Behavioral Contracts table, §Token Budget table, §Files NOT to Modify. `# BC status` header label bumped (current, v1.19) → (current, v1.23). Internal catalog version `(v1.63)` in AC-SAP1-001 traces note left untouched (distinct from BC-2.16.002 document version). Historical §Version History rows left untouched per task instruction. TD-VSDD-097 3d sweep: sibling-pair (all v2.19/v1.18 non-historical occurrences — 9+5 pin sites updated above; historical §Version History rows accurately record the versions in force at time of authorship — UNCHANGED; internal catalog `(v1.63)` — UNTOUCHED, confirmed distinct) — DONE; downstream-copy (STORY-INDEX BC pin cells are state-manager responsibility — see downstream report) — DEFERRED-TO-STATE-MANAGER; mandate-anchor (no new MUST introduced) — N/A. No AC semantics, verification obligations, or RG assignments changed. |
| 1.22 | 2026-08-13 | story-writer | STORY-SPEC task-body accuracy (F-P26-MED-001). §Tasks RG-013 entry: removed mandate for assertion (3) "variant does NOT match `InfusionError::HttpLookupFailed`" — this assertion was removed in pass-25 as a TD-VSDD-059 tautology (locally-constructed `HttpClientBuildFailed` variant can never match `HttpLookupFailed`; compile-time-constant-true, zero regression value). Replaced with an inspection note: E-INFUSE-009→E-INFUSE-015 stopgap retirement is confirmed by INSPECTION of the three `.map_err(\|e\| InfusionError::HttpClientBuildFailed { detail: e })` sites in `crates/prism-spec-engine/src/infusion/mod.rs` (`load_spec`, `load_spec_with_runtime`, `hot_reload`) per SID-1 compensating control. §Files to Modify `infusion_tests.rs` row updated: "stopgap-retirement assertion" description corrected to "detail interpolation assertions; stopgap retirement confirmed by inspection of `infusion/mod.rs` mapping sites" (downstream copy target sweep). TD-VSDD-097 3d sweep: sibling-pair (all RG-013 body descriptions swept — §BC-5.38.001 density note "and is NOT the retired E-INFUSE-009 stopgap" is a descriptive summary of test intent, confirmed via inspection, not a mandate for a runtime assertion — CORRECT, no change; §Red Gate table row carries no assertion count — CORRECT, no change; §Tasks RG-013 FIXED; §Files-to-Modify `infusion_tests.rs` row FIXED as downstream copy target) — DONE; downstream-copy (STORY-INDEX carries no "stopgap-retirement assertion" phrase — CLEAR; §Files-to-Modify `infusion_tests.rs` row FIXED — DONE) — DONE; mandate-anchor (no new MUST introduced) — N/A. No behavioral contracts, AC semantics, RG assignments, or RG count changed. Code HEAD unchanged (spec-only fix). |
| 1.21 | 2026-08-13 | story-writer | RG-014 and AC-ERR-007 addition. Added AC-ERR-007 (Group B) covering 5xx sensor HTTP response → `ConnectivityStatus::Degraded` (not Down) with `http_status: Some(5xx)`; traces to BC-2.08.002 v1.7 HTTP Error Classification postcondition. Added RG-014 (`test_map_error_503_maps_to_http_error_503` map-level in `spec_driven_adapter.rs` `mod tests` + `test_probe_connectivity_503_returns_degraded` end-to-end in `tests/defect_adapter_tls_xdome_live_001.rs`) to Red Gate table, descriptive paragraphs, and Phase 1 task list. BC-5.38.001 density recomputed: 14/16 = 0.875 (was 13/15 = 0.867). SAC-1 ordering rule range updated RG-001..RG-013 → RG-001..RG-014. BC-2.08.002 scope rows in §Authority and §Behavioral Contracts extended with 5xx→Degraded arm referencing BC-2.08.002 v1.7 §Postconditions. Points comment updated RG-001..RG-013 (1.3 pt) → RG-001..RG-014 (1.4 pt); total ~5.1→~5.2. §Files to Modify `spec_driven_adapter.rs` and `defect_adapter_tls_xdome_live_001.rs` rows updated with RG-014 test references. No BC array changes; `behavioral_contracts:` frontmatter unchanged. TD-VSDD-097 3d sweep: sibling-pair (all live RG-range/count references swept — frontmatter points comment UPDATED; §Red Gate table EXTENDED; §BC-5.38.001 density UPDATED; §ordering rule UPDATED; §Phase 1 tasks EXTENDED; §Authority BC-2.08.002 scope EXTENDED; §Behavioral Contracts BC-2.08.002 scope EXTENDED; §Files to Modify two rows UPDATED; historical §Version History RG counts accurate-at-time-of-writing, UNCHANGED; AC-ERR-007 ProbeOutcome assertion confirmed: `ProbeOutcome { status: ConnectivityStatus::Degraded, http_status: Some(503), .. }` — real fields `status/latency_ms/probed_at/http_status/error/is_rate_limited/rate_limit_retry_after_ms`, NO phantom `auth_valid` field) — DONE; downstream-copy (STORY-INDEX v1.20 bracket carries no RG count or density; prior historical brackets are accurate-at-time-of-writing records, NOT updated retroactively; state-manager adds v1.21 bracket in next burst) — CLEAR; mandate-anchor (no new MUST introduced) — N/A. |
| 1.20 | 2026-08-13 | story-writer | STORY-SPEC accuracy correction (F-P24-LOW-001). AC-ERR-005 phantom-field fix: (1) AC-ERR-005 body assertion — removed non-existent `auth_valid: Some(false)` field from `ProbeOutcome` literal; corrected to `ProbeOutcome { status: ConnectivityStatus::Up, http_status: Some(403), .. }` matching the real struct (`status`, `latency_ms`, `probed_at`, `http_status`, `error`, `is_rate_limited`, `rate_limit_retry_after_ms`) and exemplar test `test_probe_connectivity_403_returns_up_not_down`. (2) AC-ERR-005 scope-extension sentence — reworded to attribute `ConnectivityStatus::Up` to the `probe_connectivity` surface and `auth_valid: false` determination to the DOWNSTREAM `SensorHealthChecker::check_one` / `probe_auth_with_routing` surface (per RG-007 / AC-WIRE-001); BC-2.01.013 EC-01-029 end-to-end trace preserved. (3) T-F01 discriminator note — corrected parallel attribution: `probe_connectivity` correctly resolves `ConnectivityStatus::Up` only; `auth_valid: false` determination attributed to DOWNSTREAM `check_one` / `probe_auth_with_routing`. TD-VSDD-097 3d sweep: sibling-pair (`ProbeOutcome` literals swept — AC-ERR-005 body INCORRECT→FIXED; RG-005 task `status: Up, http_status: Some(403)` CORRECT (no `auth_valid`); `auth_valid` references swept — AC-ERR-005 body INCORRECT→FIXED; AC-ERR-005 scope-extension INCORRECT→FIXED; T-F01 discriminator note INCORRECT→FIXED; AC-ERR-005 traces `at the probe surface end-to-end` CORRECT; AC-WIRE-001 wire-level JSON CORRECT; AC-LIVE-002 wire-level JSON CORRECT; §Authority BC-2.01.013 `at the health probe surface` CORRECT; all other `auth_valid` occurrences in §BC status, §Behavioral Contracts, Narrative, §Architecture Compliance Rule 9, T-LV02, §UX Screen References — CORRECT) — DONE; downstream-copy (STORY-INDEX carries no `ProbeOutcome` literal or `auth_valid`-in-ProbeOutcome phrase — CLEAR) — CLEAR; mandate-anchor (no new MUST introduced) — N/A. No behavioral contracts, AC semantics, or RG assignments changed. Code HEAD unchanged. |
| 1.19 | 2026-08-13 | story-writer | F-P19-LOW-001 + BC-2.08.002 v1.6→v1.7 pin propagation. (1) F-P19-LOW-001: §Tasks Phase 1 RG-001 bullet pseudo-code arg order corrected — sensor-id-first 2-arg form `map_spec_engine_error_to_sensor_error("claroty", SpecEngineError::HttpRequestFailed{...})` replaced with error-first 3-arg form `map_spec_engine_error_to_sensor_error(SpecEngineError::HttpRequestFailed { .. }, "claroty", "<table_name>")` matching actual signature `(e: SpecEngineError, sensor_id: &str, table_name: &str)` in `spec_driven_adapter.rs`. AC-ERR-001 body text and AC-ERR-002 body text already used correct error-first 3-arg form — unchanged. Delivered RG-001 test in code already correct — story-spec only. (2) BC-2.08.002 pin propagated v1.6→v1.7 (`§Postconditions` 5xx classification: `ConnectivityStatus::Down` → `Degraded`): 5 non-historical body sites updated — `# BC status` frontmatter comment (version number + v1.7 description added), §Authority table, §Behavioral Contracts table, §Token Budget table, §Files NOT to Modify table. `# BC status` header label updated (current, v1.14) → (current, v1.19). TD-VSDD-097 3d sweep: sibling-pair (`map_spec_engine_error_to_sensor_error` call-form refs — AC-ERR-001 error-first 3-arg CORRECT; AC-ERR-002 error-first 3-arg CORRECT; RG-002 abbreviated no-order issue; RG-001 task bullet FIXED; `BC-2.08.002 v1.6` non-historical pins — 5 sites all updated to v1.7; 5xx→`ConnectivityStatus::Down` as correct outcome: NOT asserted anywhere in story body — CLEAN) — DONE; downstream-copy (STORY-INDEX BC-2.08.002 pin: state-manager responsibility per task instruction) — DEFERRED-TO-STATE-MANAGER; mandate-anchor (no new MUST introduced) — N/A. |
| 1.18 | 2026-08-13 | story-writer | Records-only (TD-VSDD-096, F-P18-LOW-001 + F-P18-LOW-002). Two records-tier corrections. (1) F-P18-LOW-001: §Red Gate Enumeration §Ordering rule (SAC-1 rule 3) paragraph stated test-writer works from "RG-001..RG-011" list — stale range corrected to "RG-001..RG-013" (RG-012 infusion UA and RG-013 E-INFUSE-015 were registered in v1.3 and v1.12 respectively; §BC-5.38.001 density check and Red Gate table already stated 13 tests / RG-001..RG-013 correctly; only the ordering-rule paragraph was stale). (2) F-P18-LOW-002: §Version History v1.15 row contained three TD-VSDD-091 volatile line-cites in record-tier changelog prose (arm-5 `L<NNN>` form retired by 2026-07-24 amendment to TD-VSDD-091): stripped the three volatile L-cite groups that had accompanied AC-UA-001, T-B01, and the CORRECT-framing note; symbol anchors (AC-UA-001, T-B01) retained; CORRECT-framing note retained. TD-VSDD-097 3d sweep: sibling-pair (all RG-range refs swept — §points comment RG-001..RG-013 CORRECT; §BC-5.38.001 density RG-001..RG-013 CORRECT; §ordering rule RG-001..RG-011 STALE corrected; v1.3 historical RG-001..RG-012 and v1.0 historical RG-001..RG-008 are accurate-at-time-of-writing historical records unchanged; all §Version History rows scanned for `L<NNN>` volatile cites — only v1.15 row had them; all three removed) — DONE; downstream-copy (STORY-INDEX.md DEFECT-ADAPTER-TLS-XDOME-LIVE-001 entry carries no copies of stale RG range or volatile L-cites; wave-state.yaml CLEAR) — CLEAR; mandate-anchor (no new MUST introduced) — N/A. No behavioral contracts, AC semantics, or RG assignments changed. |
| 1.17 | 2026-08-13 | story-writer | STORY-SPEC accuracy correction (F-P17-MED-001 + sibling sweep). Wrong function name `sanitize_body_snippet_bytes` replaced with `sanitize_body_snippet` at every connectivity.rs / `sanitize_error` / char-based ≤512-char site; wrong lib.rs re-export description (bytes-only) corrected to name both `sanitize_body_snippet` and `sanitize_body_snippet_bytes`. Supersedes the v1.16 inaccuracy on those two surfaces. Six sites fixed: (1) SS-08 subsystem justification (frontmatter comment) — delegation target `prism_core::sanitize_body_snippet_bytes` → `prism_core::sanitize_body_snippet`; (2) SS-21 subsystem justification (frontmatter comment) — lib.rs re-export description `sanitize_body_snippet_bytes re-export` → `sanitize_body_snippet + sanitize_body_snippet_bytes re-exports`; (3) §Token Budget lib.rs row — `sanitize_body_snippet_bytes pub use re-export` → `sanitize_body_snippet + sanitize_body_snippet_bytes pub use re-exports`; (4) §Token Budget connectivity.rs row — `prism_core::sanitize_body_snippet_bytes` → `prism_core::sanitize_body_snippet`; (5) §Files-to-Modify lib.rs row — `pub use error::sanitize_body_snippet_bytes` → `pub use error::{sanitize_body_snippet, sanitize_body_snippet_bytes}`; (6) §Files-to-Modify connectivity.rs row — `prism_core::sanitize_body_snippet_bytes (char-based ≤512-char cap)` → `prism_core::sanitize_body_snippet (char-based ≤512-char cap)`. TD-VSDD-097 3d sweep: sibling-pair (all 12 `sanitize_` occurrences swept; 6 incorrect fixed; 6 correct unchanged) — DONE; downstream-copy (STORY-INDEX changelog entries are historical records, not prescriptive; wave-state.yaml CLEAR) — CLEAR; mandate-anchor (no new MUST) — N/A. No behavioral contracts, AC semantics, or RG assignments changed. |
| 1.16 | 2026-08-13 | story-writer | Machine-readable footprint corrected (F-P16-HIGH-001). (1) `crates_touched` expanded: added `prism-core` (sanitize_body_snippet_bytes / sanitize_body_snippet utility functions in `error.rs` + sanitize_body_snippet_bytes re-export in `lib.rs`) and `prism-mcp` (sanitize_error delegation in `health/connectivity.rs`). (2) `subsystems` expanded: added SS-21 (Identity & Core Types — prism-core utility functions per ARCH-INDEX Subsystem Registry) and SS-08 (Sensor Health — prism-mcp sensor health subsystem per ARCH-INDEX Subsystem Registry). (3) §Files-to-Modify: expanded `prism-core/src/error.rs` row to cover sanitize_body_snippet_bytes / sanitize_body_snippet functions (previously listed only the InfusionError variant); added `prism-core/src/lib.rs` row (re-export) and `prism-mcp/src/health/connectivity.rs` row (sanitize_error delegation, char-based ≤512-char cap vs pipeline.rs byte-based ≤256-byte cap). (4) §Token Budget: added rows for `prism-core/src/error.rs`, `prism-core/src/lib.rs`, and `prism-mcp/src/health/connectivity.rs`; updated total ~143,500 → ~152,000. No behavioral contracts, AC semantics, or RG assignments changed. |
| 1.15 | 2026-08-13 | story-writer | Records-only (TD-VSDD-096, LOCAL pass-14 F-1). DD-9 delegation-vehicle framing corrected exhaustively in AC-UA-001 trace note and T-B01 — partial-fix miss from v1.14. (1) AC-UA-001 trace note: replaced "`DeclarativeHttpAuthProvider` calls `build_http_client_with_timeout()` internally and inherits the UA change automatically via the delegation chain" with "`DeclarativeHttpAuthProvider` obtains its UA from the independent `prism-spec-engine::pipeline::build_http_client_with_timeout` sibling — its OWN `.user_agent()` call — NOT via automatic delegation from the prism-bin factory". (2) T-B01: removed `DeclarativeHttpAuthProvider` from the "propagates to" list; added explicit statement that it is NOT a propagation target of the prism-bin change — covered SEPARATELY by the independent `prism-spec-engine::pipeline::build_http_client_with_timeout` sibling (cross-crate dependency from prism-bin to prism-spec-engine is impossible). Exhaustive grep results: "delegation chain" — 1 hit (AC-UA-001, fixed); "propagates to" — 1 hit (T-B01, fixed); "inherits ... automatically via build_http_client_with_timeout (prism-spec-engine::pipeline)" — CORRECT framing, unchanged; v1.14 version history record — NOT MODIFIED. No AC semantics, verification obligations, or RG assignments changed. |
| 1.14 | 2026-08-13 | story-writer | Records-only (TD-VSDD-096, LOCAL pass-13 F-1/F-2). DD-9 delegation-vehicle propagation to 3 BC-2.16.014 summary surfaces: (1) `# BC status` frontmatter comment — corrected `build_http_client_with_custom_timeout delegation chain` to `build_http_client_with_timeout (prism-spec-engine::pipeline), an independent sibling with its own .user_agent() call (not propagation from prism-bin)`. (2) §Authority table BC-2.16.014 row — corrected `via \`build_http_client_with_custom_timeout\` delegation` to `via \`build_http_client_with_timeout\` in \`prism-spec-engine::pipeline\` (independent sibling with its own \`.user_agent()\` call — not propagation from prism-bin)`. (3) §Behavioral Contracts table BC-2.16.014 row — corrected `via \`build_http_client_with_custom_timeout\` delegation chain` to `via \`build_http_client_with_timeout\` in \`prism-spec-engine::pipeline\` (ADR-050 §D6 propagation — independent sibling with its own \`.user_agent()\` call)`; corrected `beyond the \`build_http_client_with_custom_timeout\` change` to `beyond the \`build_http_client_with_timeout\` change in \`prism-spec-engine::pipeline\``. `# BC status` header label fixes: stale `(as of 2026-08-13 amendments, v1.2)` updated to `(current, v1.14)`; BC-2.16.002 catalog label `v1.64` corrected to `v1.63` (consistent with AC-SAP1-001 body reference at §Group D). Final grep confirms no other location names `build_http_client_with_custom_timeout` as the DeclarativeHttpAuthProvider delegation vehicle. AC-UA-001/T-B01 (already correct: reference `build_http_client_with_timeout()` at prism-spec-engine::pipeline path) unchanged. No AC semantics, verification obligations, or RG assignments changed. |
| 1.13 | 2026-08-13 | story-writer | Records-only (TD-VSDD-096, F-P-MED-001). E-INFUSE-015 firing-path enumeration corrected to 3 paths (1/1/1 distribution). (1) AC-ERR-006 body: enumerated all three firing paths (`load_spec`, `load_spec_with_runtime`, `hot_reload`) replacing the vague "RUNTIME PHASE" description that omitted `load_spec`. (2) §Files to Modify `infusion/mod.rs` row: corrected from "`load_spec_with_runtime` (2 sites) and `hot_reload` (1 site)" to "`load_spec` (1 site), `load_spec_with_runtime` (1 site), and `hot_reload` (1 site)" — matching BC-2.19.001 v2.4, error-taxonomy v2.74, and code ground truth in `infusion/mod.rs` (verified: one `build_http_client_with_timeout()` call in each of the three functions). (3) BC-2.19.001 pin v2.3→v2.4 propagated to 5 locations: `# BC status` frontmatter comment, §Authority table, §Behavioral Contracts table, Token Budget, §Files NOT to Modify. (4) §Behavioral Contracts table BC-2.19.001 scope cell: corrected from "RUNTIME PHASE (`load_spec_with_runtime` / `hot_reload`)" (two-path omission, missing `load_spec`) to "(1 site each: `load_spec`, `load_spec_with_runtime`, `hot_reload`)" — same class as items (1) and (2); confirmed this is the only remaining two-path occurrence in the story. Other pins verified unchanged: BC-2.16.002 v2.19, BC-2.08.002 v1.6, BC-2.01.010 v1.6, BC-2.01.013 v1.18, BC-2.16.014 v1.22, ADR-050 v2.3. No AC semantics, verification obligations, or RG assignments changed. |
| 1.12 | 2026-08-13 | story-writer | E-INFUSE-015 / F-2 completion: BC-2.19.001 propagated to body (bc_array_changes_propagate_to_body_and_acs policy). (1) BC-2.19.001 v2.3 added to `# BC status` frontmatter comment, §Authority table, §Behavioral Contracts table (scope: E-INFUSE-015 error surface / infusion load path / F-2 completion). (2) AC-ERR-006 added to Group B — `InfusionError::HttpClientBuildFailed` maps to E-INFUSE-015, not E-INFUSE-009 stopgap; traces to BC-2.19.001 §Error Conditions E-INFUSE-015 row and error-taxonomy E-INFUSE-015; Red Gate RG-013. (3) RG-013 (`test_infusion_http_client_build_failure_maps_to_e_infuse_015` in `crates/prism-spec-engine/tests/infusion_tests.rs`) registered in Red Gate enumeration table and Phase 1 task list. SID-1 direct-construction pattern noted (path effectively unreachable under rustls-tls). (4) BC-5.38.001 density recomputed: 13/15 = 0.867 (was 12/14 = 0.857). F-2 completion density note added. (5) Token Budget: BC-2.19.001 row added (~3,000 tokens); total updated ~140,500 → ~143,500. (6) Files to Modify: 3 E-INFUSE-015 wiring rows added (`prism-core/src/error.rs` HttpClientBuildFailed variant; `infusion/mod.rs` 3 call-site rewires; `infusion_tests.rs` RG-013 test). (7) Files NOT to Modify: BC-2.19.001 freeze row added. (8) Points comment updated: 12 RGs → 13 RGs (1.2 pt → 1.3 pt; total ~5.0 → ~5.1). All other pins verified unchanged: BC-2.16.002 v2.19, BC-2.08.002 v1.6, BC-2.01.010 v1.6, BC-2.01.013 v1.18, BC-2.16.014 v1.22, ADR-050 v2.3. error-taxonomy not version-pinned in body (no body reference by version number). |
| 1.11 | 2026-08-13 | story-writer | Records-only corrections (TD-VSDD-096). F-P-LOW-001: AC-WIRE-001 and RG-007 Phase-1 task text wire-byte literals corrected from spaced form (`"reachable": true`, `"auth_valid": false`) to compact serde_json form (`"reachable":true`, `"auth_valid":false`) — 4 locations: AC-WIRE-001 heading, AC-WIRE-001 body, RG-007 task bullet, Architecture Compliance Rule 9. AC-LIVE-002 prose `{ "reachable": true, ... }` intentionally unchanged (illustrative format, not a byte assertion). BC-2.16.002 v2.18 → v2.19 (row-91 disclosure amendment) propagated to 9 body locations: `# BC status` frontmatter comment, §Authority table, §Behavioral Contracts table, BC-5.38.001 density note, AC-SAP1-001 body (2 occurrences), T-G01 task (2 occurrences), Token Budget table, Architecture Compliance Rule 7, §Files NOT to Modify. Other pins verified unchanged: BC-2.08.002 v1.6, BC-2.01.010 v1.6, BC-2.01.013 v1.18, BC-2.16.014 v1.22, ADR-050 v2.3. No AC semantics, verification obligations, or RG assignments changed. |
| 1.10 | 2026-08-13 | story-writer | Records-only pin propagation (TD-VSDD-096, POL-23). BC-2.16.014 v1.21 → v1.22 (DD-9 delegation-claim accuracy fix) propagated to 5 locations: `# BC status` frontmatter comment, §Authority table, §Behavioral Contracts table, §Token Budget Estimate table, §Files NOT to Modify. Other pins verified unchanged: BC-2.16.002 v2.18, BC-2.08.002 v1.6, BC-2.01.010 v1.6, BC-2.01.013 v1.18, ADR-050 v2.3. input-hash updated c10bbb4 → 7f2e0df (pre-existing drift; inputs changed since last artifact production). No AC semantics, verification obligations, or RG assignments changed. |
| 1.9 | 2026-08-13 | story-writer | Records-only (TD-VSDD-096, F-P-LOW-002). MED-1 body-snippet sanitization-test attribution corrected in two locations: (1) BC-5.38.001 Density Check note: removed false "in `spec_driven_adapter.rs` inline tests" attribution; correctly attributes to `crates/prism-spec-engine/tests/pipeline_http_integration.rs` (`test_BC_2_16_002_med1_non_2xx_body_sanitizes_control_chars_preserves_utf8`, `test_BC_2_16_002_f1_non_2xx_body_byte_cap_multibyte_utf8`) and `crates/prism-core/src/error.rs` (`test_sanitize_body_snippet_bytes_*`). (2) §Files to Modify `spec_driven_adapter.rs` row: removed false "add MED-1 body-snippet sanitization test (fix-burst pass-1)" claim; `pipeline_http_integration.rs` row updated to include MED-1 sanitization test coverage. No AC semantics, verification obligations, or RG assignments changed. |
| 1.8 | 2026-08-13 | story-writer | Records-only corrections (TD-VSDD-096). F-1 (LOW): T-A04 and AC-CARGO-001 "Cargo feature-unification" description corrected to "explicitly declares `\"http2\"` in its features array" — ground truth `crates/prism-bin/Cargo.toml` `[dev-dependencies]` reqwest entry shows `features = ["json", "rustls-tls", "http2"]` (explicit literal, not unification). F-3 (MED): AC-ERR-005 title generalized from "mock-401 / Some(401)" to "mock-4xx / Some(<4xx>)"; explicit assertion updated to `Some(403)` to match RG-005 (`test_probe_connectivity_403_returns_up_not_down`); 401 production-path note retained in scope extension and AC-LIVE-002 reference. F-4 (LOW): T-E01 sanitize-fn reference corrected from `sanitize_error` in `prism-mcp/src/health/connectivity.rs` to `prism_core::sanitize_body_snippet_bytes` per BC-2.16.002 Non-2xx Response Body Capture postcondition. Pin propagation: ADR-050 v2.2→v2.3 (4 locations: §Authority prose, §Authority table, Token Budget, §Files NOT to Modify); BC-2.16.002 v2.17→v2.18 (5 locations: frontmatter comment, §Authority table, §Behavioral Contracts table, Token Budget, §Files NOT to Modify). BC-2.08.002 v1.6, BC-2.01.010 v1.6, BC-2.01.013 v1.18, BC-2.16.014 v1.21 verified unchanged. |
| 1.7 | 2026-08-13 | story-writer | Records-only corrections (TD-VSDD-096). F-1 (MAJOR): AC-ERR-003 dual-arm scope alignment — title updated to "Error evidence captured in `HttpRequestFailed.detail` — non-2xx body snippet + send-failure source chain"; scope extension paragraph added explicitly covering both arms: non-2xx body arm (RG-003, BC-2.16.002 Non-2xx Response Body Capture postcondition) and send-failure source-chain arm (RG-009, BC-2.16.002 Send-Failure Error Source Chain postcondition); traces note updated to cite both postconditions. No AC semantics, verification obligations, or RG assignments changed. BC pin propagation: BC-2.01.013 v1.17→v1.18 (5 locations: frontmatter comment, §Authority, §Behavioral Contracts, Token Budget, §Files NOT to Modify); BC-2.16.014 v1.20→v1.21 (5 locations same). BC-2.16.002 v2.17, BC-2.08.002 v1.6, BC-2.01.010 v1.6, ADR-050 v2.2 verified unchanged. |
| 1.6 | 2026-08-13 | story-writer | LOCAL adversary pass-6 records-only correction (TD-VSDD-096). F-1 (MED): persistent-auth mechanism prose corrected in 4 locations to match code (Arm 2 direct variant-matching, not `HttpRequestFailed`). (1) AC-ERR-001 scope-extension: removed "(which surface as `HttpRequestFailed { status_code: 401 }` at the mapping boundary)"; replaced with explicit Arm 2 `matches!()` description. (2) AC-ERR-005 scope-extension: replaced "produces the same … observable" framing with explicit "Arm 2 … NOT through `HttpRequestFailed`" mechanism. (3) RG-010 task bullet: replaced "`AuthRefreshFailed`-derived `HttpRequestFailed { status_code: 401 }`" with "`SpecEngineError::AuthRefreshFailed{..}` variant directly (Arm 2, NOT through `HttpRequestFailed`)". (4) RG-011 task bullet: same correction for `CookieAuthFailed`. Verified against `spec_driven_adapter.rs` Arm 2 `matches!()` guard in worktree — code is authoritative (SAC-1). No AC semantics, verification obligations, or RG assignments changed. |
| 1.5 | 2026-08-13 | story-writer | LOCAL adversary pass-5 records-only corrections (TD-VSDD-096). F-P5-MED-001 (MEDIUM): removed stale duplicate `holdout_scenarios: []` frontmatter key (lines were: populated `[HS-TLS-XDOME-001, HS-TLS-XDOME-002, HS-TLS-XDOME-003]` after `modified:` and empty `[]` before `assumption_validations:`); kept the populated occurrence; no other duplicate top-level frontmatter keys found. F-P5-LOW-001 (LOW): corrected illustrative call signature in AC-ERR-001 and AC-ERR-002 prose from 2-arg sensor-id-first form to real 3-arg error-first form (`map_spec_engine_error_to_sensor_error(SpecEngineError::..., sensor_id, table_name)`) verified against `spec_driven_adapter.rs` fn signature; no AC semantics change. |
| 1.4 | 2026-08-13 | story-writer | LOCAL adversary pass-4 records-only corrections (TD-VSDD-096). F-1 (MED): BC-2.01.013 canonical H1 corrected to "DataSource Trait Eliminates Per-Sensor Code Duplication" in §Authority table, §Behavioral Contracts table Title column, and `# BC status` frontmatter comment; §Files NOT to Modify file path corrected to `BC-2.01.013-datasource-trait-adapter-pattern.md`; Token Budget parenthetical updated. F-2 (LOW): BC/ADR version pins re-synced to current frontmatter: BC-2.16.002 v2.15→v2.17, BC-2.08.002 v1.5→v1.6, BC-2.01.010 v1.5→v1.6, ADR-050 v2.1→v2.2 (all five table locations, Token Budget, §Files NOT to Modify, and inline body refs). F-3 (LOW): AC-CARGO-001 count corrected four→three production entries; prism-bin entry count corrected two→one production + one harmless dev-dep; T-A04 reframed as verify-only task; §Files to Modify description updated; AC-H2-001 inline "four" corrected. No AC-semantic or verification changes. |
| 1.3 | 2026-08-13 | story-writer | LOCAL adversary pass-2 reconciliation. RG names reconciled to code (code is authoritative per SAC-1): RG-003 → `test_pipeline_non_2xx_body_in_detail`; RG-005 → `test_probe_connectivity_403_returns_up_not_down`; RG-007 → `test_sensor_health_wire_shape_403_reachable_auth_invalid`; RG-008 file location updated to `crates/prism-bin/tests/defect_adapter_tls_xdome_live_001.rs`. RG-012 (`test_infusion_http_client_sends_prism_user_agent`, `crates/prism-spec-engine/src/pipeline.rs` `mod infusion_http_client_user_agent_tests`) registered for OBS-4 fix — verifies ADR-050 §D6 v2.1 infusion client UA. ADR-050 pin updated v2.0 → v2.1 in §Authority and §Files NOT to Modify (§D6 scope extended to include infusion `build_http_client_with_timeout`). BC-5.38.001 density recomputed: 12/14 = 0.857 (was 11/14 = 0.786). Points comment updated to RG-001..RG-012. |
| 1.2 | 2026-08-13 | story-writer | Fix-burst pass-1 spec changes. Added RG-009 (`test_BC_2_16_002_rg009_send_failure_includes_source_chain` in `crates/prism-spec-engine/tests/pipeline_http_integration.rs`), RG-010 (`test_map_error_auth_refresh_failed_maps_to_http_error_401` in `spec_driven_adapter.rs`), RG-011 (`test_map_error_cookie_auth_failed_maps_to_http_error_401` in `spec_driven_adapter.rs`). Added BC-2.01.013 v1.17 to `behavioral_contracts` array and §Authority + §Behavioral Contracts tables (EC-01-029 persistent-auth-failure alignment). Updated BC version pins: BC-2.16.002 →v2.15, BC-2.08.002 →v1.5 (AuthRefreshFailed/CookieAuthFailed scope added). Extended AC-ERR-001 and AC-ERR-005 to reference persistent-auth-failure path and add BC-2.01.013 EC-01-029 traces. Recomputed BC-5.38.001 density: 11/14 = 0.786 (was 8/14 = 0.571). Updated §Files to Modify (RG-009 integration test file added) and §Files NOT to Modify (BC-2.01.013 freeze row + updated version pins). MED-1 body-snippet sanitization test noted in §Files to Modify and density note. |
| 1.1 | 2026-08-12 | research-agent | Remove-uncertainty pass (D-1110 pass-1). Validated 4 technology assumptions: A1 (http2 feature semantics), A2 (user_agent API), A4 (rustls+ALPN composition) CONFIRMED against reqwest 0.12.28 docs (Context7) + workspace Cargo.lock; A3 (fix plausibility vs api.claroty.com WAF edge) rated RISK/moderate-confidence-sufficient via Perplexity deep research + working-relay evidence. Corrected AC-H2-001 + RG-008: whole-file `grep '"h2"' Cargo.lock` observable is invalid (h2 already present transitively via hyper 1.9.0); scoped verification to the reqwest node. Added §Technology Assumption Validation with a labeled RISK note (rustls JA3/JA4 + non-UA-header residual risk on no-waiver AC-LIVE-001; h2 reframed as ADR-050 compliance/defense-in-depth, UA as probable load-bearing fix). Populated `assumption_validations` + `risk_mitigations` frontmatter. No changes to spec artifacts (BC/ADR/error-taxonomy) — those remain frozen per §Files NOT to Modify. |
| 1.0 | 2026-08-12 | story-writer | Full implementation story authored from design gate D-2111 (APPROVED). Supersedes stub v0.1. Bundles F10 (transport: http2 feature + User-Agent) + F9 (error surfacing: source-chain + body-capture + error mapping + AllTargetsFailed WARN). 14 ACs across Groups A–E. 8 Red Gate tests (RG-001..RG-008). BC-5.38.001 density 0.571. SAC-1 compliant (enumerated RG list + density check + red-then-green task ordering). §Authority cites ADR-050 v2.0, BC-2.16.002 v2.14, BC-2.08.002 v1.4, BC-2.01.010 v1.5, BC-2.16.014 v1.20. DEFECT-SENSOR-ERROR-FLATTEN-001 superseded and closed. |
| 0.1 | 2026-08-03 | story-writer | Initial registration stub from D-1889 triage (F10); records corrected primary-fix-path (http2+UA, not native-tls); architect adjudication framing; no ACs or implementation guidance. |
