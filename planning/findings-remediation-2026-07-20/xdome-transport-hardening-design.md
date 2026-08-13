---
document_type: planning
title: "xDome Transport Hardening — Fix Design for F10 + F9"
created: "2026-08-12"
author: architect
origin: "DEFECT-ADAPTER-TLS-XDOME-LIVE-001 architect adjudication"
status: APPROVED
version: "1.1"
stories_governed:
  - DEFECT-ADAPTER-TLS-XDOME-LIVE-001
  - DEFECT-SENSOR-ERROR-FLATTEN-001
adr_amendment: "ADR-050 v2.0"
---

# xDome Transport Hardening — Fix Design

## 1. Problem Summary

**Finding 10 (CRIT):** Prism cannot communicate directly with the real Claroty xDome
API (`https://api.claroty.com`, behind AWS Global Accelerator). Every query returns
E-SENSOR-030 "all targets failed". A localhost HTTP/1.1 relay forwarding verbatim to
the same endpoint succeeds. User directive: eliminate the relay via a production-grade fix.

**Finding 9 (HIGH, coupled):** Even when a sensor API is reachable and returns HTTP
4xx (e.g., HTTP 401 "API token expired"), prism misclassifies it as
`reachable: false, auth_valid: null, error: "sensor_unreachable_cannot_verify"`.

Both findings were confirmed by code inspection of develop and remain unresolved at
develop@5d1a30ac7.

---

## 2. Root Causes (Code-Confirmed)

### F10 Root Causes

**RC-F10-A — No `http2` reqwest feature compiled in.**
All production reqwest deps declare `default-features = false` without re-enabling
`http2`. Prism is HTTP/1.1-only. AWS Global Accelerator prefers h2; clients not
advertising h2 via ALPN can match WAF block profiles.
Affected entries: `prism-spec-engine` [dependencies], `prism-sensors` [dependencies],
`prism-bin` [dependencies] (two entries).

**RC-F10-B — No User-Agent header.**
`build_http_client_with_custom_timeout` (sole production sensor HTTP client factory)
has no `.user_agent(...)` call. Zero UA calls exist anywhere in production crates.
UA-less + rustls/webpki fingerprint + h1-only is a recognized WAF block signature.

**RC-F10-C — Error evidence destroyed (three layers).**
- `.send().await.map_err(|e| ... detail: e.to_string())` — `to_string()` omits the
  reqwest error `source()` chain which carries the hyper/h2/TLS-level cause
- Non-2xx response bodies never read
- `FanOutError` `errors` vec in `AllTargetsFailed` logged only as count
This is the companion diagnostic issue (DEFECT-SENSOR-ERROR-FLATTEN-001 = F9).

### F9 Root Causes

**RC-F9-A — `map_spec_engine_error_to_sensor_error` always produces `SensorError::Internal`.**
This function in `crates/prism-bin/src/spec_driven_adapter.rs` receives any
`SpecEngineError` (including `HttpRequestFailed { status_code: 401, ... }`) and
unconditionally returns `SensorError::Internal { detail }`. It never produces
`SensorError::HttpError { sensor, status, body }`.

**RC-F9-B — The health classifier already handles `SensorError::HttpError` correctly.**
`probe_connectivity` in `crates/prism-mcp/src/health/connectivity.rs` has an arm:
`Err(SensorError::HttpError { status, body, .. })` → Up if 4xx, Degraded if 5xx.
This arm fires correctly. But `map_spec_engine_error_to_sensor_error` produces
`SensorError::Internal` instead, which falls to the catch-all `Err(e)` →
`ConnectivityStatus::Down`.

The single-line class fix: match `SpecEngineError::HttpRequestFailed { status_code > 0 }`
and produce `SensorError::HttpError`.

---

## 3. ADR Decision

**Amendment: ADR-050 v2.0.**
A new ADR is NOT required. ADR-050 is the workspace reqwest feature policy ADR;
adding `http2` and `user_agent` requirements extends the same policy.
File: `/Users/jmagady/Dev/prism/.factory/specs/architecture/decisions/ADR-050-workspace-reqwest-tls-backend.md`

**New decisions D5 and D6:**

- **D5:** `http2` feature MUST be in production `[dependencies]` reqwest entries for
  `prism-spec-engine`, `prism-sensors`, and `prism-bin`. DTU `[dev-dependencies]` excluded.

- **D6:** All sensor/plugin outbound `reqwest::Client::builder()` chains MUST call
  `.user_agent(concat!("prism/", env!("CARGO_PKG_VERSION")))` before `.build()`.
  Scope: `build_http_client_with_custom_timeout` (covers sensor adapter clients and
  `DeclarativeHttpAuthProvider` via delegation) + both plugin client builders in boot.rs.

**native-tls explicitly rejected (ADR-050 v2.0 Alt-D):** The finding floated this;
it is prohibited under D2, does not address the ALPN root cause, reintroduces 65s
Keychain init overhead, and reopens the MSSP credential MITM interception path.

---

## 4. BC / Spec Touchpoint Map

### 4.1 BCs Requiring Product-Owner Amendment

**BC-2.16.002 — Multi-Step Fetch Pipeline Execution (MUST amend)**

Missing postconditions to add:

| Postcondition | Required Content |
|---------------|-----------------|
| HTTP client compliance | HTTP requests are issued via an ADR-050 §D5/§D6-compliant client (h2+UA) via `build_http_client_with_custom_timeout`. |
| send-failure detail | When `.send()` fails, `SpecEngineError::HttpRequestFailed.detail` MUST include the reqwest error source chain (not only `e.to_string()`). |
| non-2xx response body | When non-2xx received, `HttpRequestFailed.detail` MUST include HTTP status code AND capped (≤256 bytes) sanitized upstream response body. |
| AllTargetsFailed logging | When `AllTargetsFailed` is emitted, each `FanOutError` in the `errors` vec MUST be logged at WARN with `event_type = "fan_out_target_failed"` before returning. E-SENSOR-030 Display remains count-only per BC-2.10.007 Rule 1. |

Structured Event Catalog entry to add (SAP-1 requirement):
- `fan_out_target_failed` | warn | `fan_out` in `crates/prism-sensors/src/fanout.rs` | fields: `org_id`, `sensor_id`, `attempts`, `is_transient`, `error` (FanOutError Display) | trigger: every FanOutError in errors vec before AllTargetsFailed returns
- Catalog scope MUST be extended to include `prism-sensors/src/fanout.rs`

Alternative home: product-owner may instead register this event in BC-2.01.010 §Postconditions
and extend that BC's catalog scope. Either home is acceptable; one canonical home must be chosen.

**BC-2.08.002 — Auth Validity Check (MUST amend)**

Anchor for DEFECT-SENSOR-ERROR-FLATTEN-001. New postcondition to add:
`SpecEngineError::HttpRequestFailed { status_code > 0, detail }` from `PipelineExecutor::execute`
MUST be mapped by `map_spec_engine_error_to_sensor_error` to
`SensorError::HttpError { sensor, status: status_code, body: detail }` (not `SensorError::Internal`).
`status_code = 0` (transport failure, no HTTP response) continues to map to `SensorError::Internal`.

**BC-2.16.014 — Declarative Auth Acquisition Token Lifecycle (review only)**

`DeclarativeHttpAuthProvider` uses `build_http_client_with_timeout()` internally. After
D6 is applied to `build_http_client_with_custom_timeout`, the auth client automatically
gets User-Agent + h2. No content change required; adding a note to INV-014-007
referencing ADR-050 §D5/§D6 is optional but recommended for completeness.

**BC-2.01.010 — Partial Failure Handling (minor amendment)**

Add postcondition or invariant: "When `AllTargetsFailed` is emitted, the per-target
`FanOutError` details MUST be logged before the error propagates." Reinforces the
BC-2.16.002 catalog entry.

### 4.2 Error Taxonomy Amendment Required

**File:** `.factory/specs/prd-supplements/error-taxonomy.md`

**E-SENSOR-030 row amendment:** Current row states "per-target errors carried in
`errors` field (audit log only; not interpolated into Display)" — describes intended
behavior that does NOT currently happen (errors vec is never logged). Add: "Emitting
`AllTargetsFailed` MUST be preceded by a `fan_out_target_failed` WARN log per target
(BC-2.16.002 catalog row `fan_out_target_failed`)."

### 4.3 Story Stubs to Update

`DEFECT-ADAPTER-TLS-XDOME-LIVE-001`: populate `behavioral_contracts:` after PO BC authorship.
Advance from `status: draft` to `status: ready` after ADR adjudication (done) + PO BC authorship
+ RG list populated per SAC-1.

`DEFECT-SENSOR-ERROR-FLATTEN-001`: to be closed as superseded and absorbed into
DEFECT-ADAPTER-TLS-XDOME-LIVE-001 (see §5 Bundling Verdict).

---

## 5. Bundling Verdict

**VERDICT: Bundle F9 + F10 into a single story delivery via DEFECT-ADAPTER-TLS-XDOME-LIVE-001.**

Close DEFECT-SENSOR-ERROR-FLATTEN-001 as superseded by the bundled story.

Rationale:

1. **Shared root diagnostic.** Both findings were discovered on the same day in the
   same live-client debug session. They describe two layers of the same defect: transport
   failure (F10) and diagnostic blindness that hid it (F9).

2. **Live-verification ACs require both.** AC-LIVE-001 (rows returned directly from
   `api.claroty.com`) and AC-LIVE-002 (`auth_valid: false` on 401) must both pass in
   the same PR to declare the relay eliminated.

3. **F9 fix is a prerequisite for F10 diagnostic coverage.** Without error surfacing,
   any remaining transport issue is invisible. The relay was required precisely because
   error detail was swallowed.

4. **Delivery efficiency.** Both fixes touch the same functions. Splitting into separate
   PRs doubles the review surface and requires a hard dependency gate.

---

## 6. Delivery Vehicle Recommendation

**Standard per-story TDD pipeline for DEFECT-ADAPTER-TLS-XDOME-LIVE-001.**
NOT `fix-pr-delivery` (that skill is for 1-2 file adversarial-review findings).

This story requires:
- Full TDD: Red Gate tests first (SAC-1), then implementation
- Cargo.toml changes across 3 crates (4 entries total)
- Code changes across 4 files (spec_driven_adapter.rs, boot.rs, pipeline.rs, fanout.rs)
- LOCAL adversary 3-CLEAN (BC-5.39.001)
- Story-level holdout gate (product-owner authors hidden scenarios)
- Mandatory live-verification ACs that require demo-recorder with real sensor

**Priority:** P0 CRIT. **Wave:** C.

---

## 7. Acceptance Criteria

### AC Group A — Transport Fix (F10)

**AC-H2-001:** The `h2` crate appears as a direct dependency in Cargo.lock.
Red Gate: `test_h2_in_cargo_lock` or CI check via `cargo metadata`.

**AC-UA-001:** Unit test in `spec_driven_adapter.rs` `#[cfg(test)] mod tests` asserts
that a client built by `build_http_client_with_custom_timeout(Duration::from_millis(1))`
sends `User-Agent: prism/{VERSION}`. FAILS before fix (no `.user_agent(...)` call).

**AC-UA-002:** Both plugin http_client builder sites in boot.rs include
`.user_agent(concat!("prism/", env!("CARGO_PKG_VERSION")))`. Verified by adversary sweep.

**AC-CARGO-001:** All four production reqwest entries include `"http2"`. `just check` passes.

### AC Group B — Error Surfacing (F9)

**AC-ERR-001:** `map_spec_engine_error_to_sensor_error` with
`HttpRequestFailed { status_code: 401, detail: "HTTP 401" }` returns
`SensorError::HttpError { sensor: "claroty", status: 401, body: "HTTP 401" }`.
FAILS before fix.

**AC-ERR-002:** `map_spec_engine_error_to_sensor_error` with
`HttpRequestFailed { status_code: 0, detail: "connection reset" }` returns
`SensorError::Internal { .. }` (transport errors, no HTTP response code → Internal).
FAILS if regression introduced.

**AC-ERR-003:** Unit test in pipeline.rs: when non-2xx response is received, the
`SpecEngineError::HttpRequestFailed.detail` includes both HTTP status code AND a
non-empty body snippet. FAILS before fix.

**AC-ERR-004:** Unit test in fanout.rs: when `AllTargetsFailed` is about to be
returned with non-empty `errors` vec, `fan_out_target_failed` WARN events are emitted
for each error (captured via tracing test subscriber). FAILS before fix.

**AC-ERR-005:** Integration test (in-process): when sensor adapter `fetch()` is driven
with a mock returning HTTP 401, `probe_connectivity` returns
`ProbeOutcome { status: ConnectivityStatus::Up, http_status: Some(401) }`.
FAILS before fix (returns Down via catch-all).

### AC Group C — Wire-Shape Assertions

**AC-WIRE-001:** AC-ERR-005 integration test MUST assert the serialized MCP JSON
response contains `"reachable": true` and `"auth_valid": false` at the wire level.
Source: CLAUDE.md wire-shape assertion discipline.

### AC Group D — SAP-1 Compliance

**AC-SAP1-001:** `fan_out_target_failed` event_type is registered in BC-2.16.002
§Canonical Structured Event Catalog (or BC-2.01.010) in the same commit as the
implementation. Adversary verifies via `rg 'event_type.*fan_out_target_failed' crates/`.

### AC Group E — Live Verification (BLOCKING)

**AC-LIVE-001 (NO WAIVER):** With relay REMOVED from `prism-live-mcp-wrapper.sh`
(overlay `base_url` → `https://api.claroty.com`), `list_sensor_data sensor=claroty
table=devices` for client "monroe" returns HTTP 200 + at least 1 OCSF row.
Credentials AI-opaque (keyring `prism/monroe/claroty/*`); values MUST NOT be emitted.

**AC-LIVE-002 (NO WAIVER):** With an invalid/expired claroty credential,
`check_sensor_health sensor=claroty` for client "monroe" returns `reachable: true,
auth_valid: false` with detail containing HTTP 401. Confirms F9 fix is live.

**AC-LIVE-003:** After AC-LIVE-001 passes, relay file `test-soc/live-soc/relay/xdome-relay.py`
is either removed or marked deprecated with a clear comment that it is historical-only.

---

## 8. Red Gate Enumeration (SAC-1)

| RG-ID | Test name | Corresponding AC |
|-------|-----------|-----------------|
| RG-001 | `test_map_error_http_401_maps_to_http_error_not_internal` | AC-ERR-001 |
| RG-002 | `test_map_error_status_0_maps_to_internal` | AC-ERR-002 |
| RG-003 | `test_pipeline_non_2xx_body_in_detail` | AC-ERR-003 |
| RG-004 | `test_fanout_all_failed_emits_fan_out_target_failed_warn` | AC-ERR-004 |
| RG-005 | `test_probe_connectivity_403_returns_up_not_down` | AC-ERR-005 |
| RG-006 | `test_build_http_client_sends_user_agent_header` | AC-UA-001 |
| RG-007 | `test_sensor_health_wire_shape_403_reachable_auth_invalid` | AC-WIRE-001 |

BC-5.38.001 density: 7 Red Gate tests for 7 ACs. Test-authoring tasks MUST precede
implementation tasks in story decomposition (SAC-1 rule 3).

---

## 9. Implementation Task List

### 9.1 Cargo.toml Feature Edits

**`crates/prism-spec-engine/Cargo.toml`**
Add `"http2"` to `reqwest` features (current: `["json", "rustls-tls", "gzip", "deflate", "brotli"]`).

**`crates/prism-sensors/Cargo.toml`**
Add `"http2"` to `reqwest` features (current: `["json", "rustls-tls"]`).

**`crates/prism-bin/Cargo.toml` — two entries**
- Sensor adapter entry (comment references S-PLUGIN-PREREQ-D AC-9): add `"http2"` to `["rustls-tls"]`
- Main dep entry: add `"http2"` to `["json", "rustls-tls"]`

### 9.2 User-Agent (prism-bin/src/spec_driven_adapter.rs)

**Symbol:** `build_http_client_with_custom_timeout`

Insert `.user_agent(concat!("prism/", env!("CARGO_PKG_VERSION")))` in the builder chain
before `.timeout(timeout)`. `concat!` produces a `&'static str`, zero allocation.
This single change propagates to: `build_http_client_with_timeout` (thin wrapper),
all `SpecDrivenSensorAdapter` clients, and `DeclarativeHttpAuthProvider` (calls
`build_http_client_with_timeout()` internally per BC-2.16.014 INV-014-007).

### 9.3 User-Agent (prism-bin/src/boot.rs)

**Two sites** — both precede `PluginRuntime::new_with_audit_sink`:
- The builder in the `PRISM_DISABLE_PLUGIN_LOAD` fast-path
- The builder in the normal-path

Add `.user_agent(concat!("prism/", env!("CARGO_PKG_VERSION")))` to each.

### 9.4 send-failure Error Source Chain (prism-spec-engine/src/pipeline.rs)

**Symbol:** The function containing `.send().await.map_err(|e| SpecEngineError::HttpRequestFailed { ..., status_code: 0, detail: e.to_string() })` — there are symmetric send sites for first request and 401-retry.

At each send-failure site (all `status_code: 0` send-failure variants), replace
`e.to_string()` with:
```rust
format!(
    "{}{}",
    e,
    std::error::Error::source(&e)
        .map(|s| format!("; caused by: {s}"))
        .unwrap_or_default()
)
```
This includes the hyper/h2/TLS-level source chain that `.to_string()` omits.

### 9.5 Non-2xx Response Body Capture (prism-spec-engine/src/pipeline.rs)

**Symbol:** The `if !status.is_success()` branch currently returning
`detail: format!("HTTP {status}")`.

Before returning the error, read and sanitize the response body (cap 256 bytes,
strip control chars to prevent prompt injection — same pattern as `sanitize_error`
in `prism-mcp/src/health/connectivity.rs`). Include in detail:
`detail: format!("HTTP {status}: {sanitized_body}")` (empty body falls back to
`format!("HTTP {status}")`).

Use `.bytes().await.ok()` — body read is best-effort; a secondary failure must NOT
replace the primary status-code error.

Apply this fix to BOTH: the first-request non-2xx branch AND the 401-retry non-2xx
branch (symmetric sites).

### 9.6 Error Mapping Fix (prism-bin/src/spec_driven_adapter.rs)

**Symbol:** `map_spec_engine_error_to_sensor_error`

Insert a guard arm before the `SensorError::Internal` return:
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
// status_code = 0 (transport error) and all other variants → Internal
SensorError::Internal { detail: format!("SpecDrivenSensorAdapter: ... {e}") }
```

Discriminator: `status_code = 0` = transport failure (no HTTP response) → Internal
(connectivity: Down is correct). `status_code > 0` = HTTP response received →
HttpError (health classifier arm fires, 4xx → auth_valid: false).

### 9.7 AllTargetsFailed Per-Target Logging (prism-sensors/src/fanout.rs)

**Symbol:** The site constructing `SensorError::AllTargetsFailed { count, errors }`.

Before returning `Err(SensorError::AllTargetsFailed { count, errors })`:
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
SAP-1 obligation: register `fan_out_target_failed` in BC-2.16.002 §Canonical Structured
Event Catalog in the same commit.

---

## 10. Post-Implementation Checks

- `just check` passes including h2 in Cargo.lock as direct dep
- SAP-1 adversary probe: `rg 'event_type.*fan_out_target_failed' crates/` confirms emission;
  BC-2.16.002 or BC-2.01.010 catalog entry present
- SAP-2 probe: not applicable (no sensor TOML changes)
- Live-verification agent: AC-LIVE-001/002/003 — relay removed, real endpoint tested
  with AI-opaque credentials from keyring; values never emitted

---

## Changelog

| Version | Date | Author | Change |
|---------|------|--------|--------|
| 1.1 | 2026-08-13 | product-owner | **pass-3 F-1 citation corrections in §8 Red Gate Enumeration.** Three stale test names corrected to match canonical code names (SAC-1 — code is authoritative): RG-003 `test_pipeline_non_2xx_detail_includes_body_snippet` → `test_pipeline_non_2xx_body_in_detail`; RG-005 `test_probe_connectivity_401_returns_up_auth_invalid` → `test_probe_connectivity_403_returns_up_not_down`; RG-007 `test_sensor_health_wire_shape_auth_invalid` → `test_sensor_health_wire_shape_403_reachable_auth_invalid`. Canonical map from story v1.3. |
| 1.0 | 2026-08-12 | architect | Initial approved design document. |
