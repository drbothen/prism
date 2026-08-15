# Evidence Report — DEFECT-ADAPTER-TLS-XDOME-LIVE-001

**Story:** xDome transport hardening — http2 feature + User-Agent + error source-chain + error mapping (F10 + F9 bundled)
**Branch:** `feature/DEFECT-ADAPTER-TLS-XDOME-LIVE-001`
**Demo vehicle:** VHS (terminal recordings via `cargo nextest run`)
**Recorded:** 2026-08-15

---

## Coverage Summary

| AC | Description | Evidence Artifact | Status |
|----|-------------|-------------------|--------|
| AC-WIRE-001 | 4xx sensor → MCP wire `"reachable":true` + `"auth_valid":false` + suggestion non-empty | `AC-WIRE-001-health-401-auth-invalid.{gif,webm}` | PASS |
| AC-WIRE-002 | 5xx sensor (Degraded) → MCP wire `"reachable":true` + `"auth_valid":true` + `"error":"service_unavailable"` | `AC-WIRE-002-health-5xx-degraded.{gif,webm}` | PASS |
| AC-WIRE-003 | Fleet envelope `"overall_status":"partial"`, prose summary excludes Degraded from healthy count | `AC-WIRE-002-health-5xx-degraded.{gif,webm}` | PASS |
| AC-WIRE-004 | Fleet envelope `summary_counts.healthy_count:0`, `unhealthy_count:1` for single-503 fleet | `AC-WIRE-002-health-5xx-degraded.{gif,webm}` | PASS |
| AC-QERR-001 | `query` tool `sensor_errors` carries `"<table>: HTTP <status>: <body>"` per failing target; absent on success | `AC-QERR-001-query-sensor-errors.{gif,webm}` | PASS |
| AC-QERR-002 | Cross-client partial-failure: org-a 403 → sensor_errors; org-b success → rows (regression guard) | `AC-QERR-001-query-sensor-errors.{gif,webm}` | PASS |
| AC-H2-001 | reqwest `http2` feature active on reqwest dependency node (h2 in reqwest package block) | `AC-H2-001-transport-hardening.{gif,webm}` | PASS |
| AC-UA-001 | `build_http_client_with_custom_timeout` sends `User-Agent: prism/<version>` | `AC-H2-001-transport-hardening.{gif,webm}` | PASS |
| AC-LIVE-001 | Direct `api.claroty.com` returns ≥1 OCSF row with relay removed | _live-pass note below_ | LIVE-PASS |
| AC-LIVE-002 | `check_sensor_health` with invalid/expired credential → `"reachable":true, "auth_valid":false` | _live-pass note below_ | LIVE-PASS |
| AC-LIVE-003 | `test-soc/live-soc/relay/xdome-relay.py` annotated or deleted | PENDING HUMAN ACTION (operations repo) | PENDING |

**AC-LIVE-001 note:** This AC was validated live against `api.claroty.com` for client "monroe" (STATE D-2166, LIVE-PASS, 2026-08-14). Evidence is kept locally/gitignored under `docs/demo-evidence/DEFECT-ADAPTER-TLS-XDOME-LIVE-001/live/` per AD-017 (no customer data in committed evidence). No customer records are present in this committed report.

All DTU-based acceptance criteria have recorded demos covering both success and error paths. No real customer data is present in any committed artifact.

---

## Artifacts

### AC-WIRE-001 — Health-wire 4xx → auth-invalid

**Files:**
- `AC-WIRE-001-health-401-auth-invalid.tape` — VHS tape script
- `AC-WIRE-001-health-401-auth-invalid.gif` — terminal recording
- `AC-WIRE-001-health-401-auth-invalid.webm` — archival recording

**What it shows:**

```
cargo nextest run -p prism-bin --test defect_adapter_tls_xdome_live_001 \
  -E 'test(wire_shape_403_reachable_auth_invalid) | test(wire_shape_401_auth_invalid_production_path)'
```

Two wire-shape integration tests run against an in-process prism-bin stack with a wiremock HTTP server:

- `test_sensor_health_wire_shape_403_reachable_auth_invalid` (RG-007) — sensor returns HTTP 403;
  asserts the FULL serialized JSON byte output contains `"reachable":true` AND `"auth_valid":false`.
  Before the `map_spec_engine_error_to_sensor_error` guard, a 4xx error fell through to the
  `SensorError::Internal` catch-all and produced `ConnectivityStatus::Down` → `"reachable":false`
  (wrong). After the fix the `status_code > 0` guard routes 4xx to `SensorError::HttpError`,
  producing `ConnectivityStatus::Up` → `"reachable":true` at the wire.

- `test_sensor_health_wire_shape_401_auth_invalid_production_path` — same end-to-end path via the
  production sensor spec (not a synthetic adapter), confirming the fix propagates through the
  full `SpecDrivenSensorAdapter` → `check_one` chain.

**Test result:** 2/2 PASS

**Traces to:** BC-2.08.002 §Postconditions EC-08-006 (HTTP 4xx → `auth_valid:false`)

---

### AC-WIRE-002 / AC-WIRE-003 / AC-WIRE-004 — Health-wire 5xx → Degraded (not Down)

**Files:**
- `AC-WIRE-002-health-5xx-degraded.tape` — VHS tape script
- `AC-WIRE-002-health-5xx-degraded.gif` — terminal recording
- `AC-WIRE-002-health-5xx-degraded.webm` — archival recording

**What it shows:**

```
cargo nextest run -p prism-bin --test defect_adapter_tls_xdome_live_001 \
  -E 'test(BC_2_08_002_degraded)'
```

Two wire-shape integration tests with a wiremock 503 server:

- `test_BC_2_08_002_degraded_reachable_wire_shape` (RG-019, AC-WIRE-002) — drives the full path
  `probe_connectivity` → `SensorHealthChecker::check_one` → MCP serialization; asserts the FULL
  serialized JSON byte output contains `"reachable":true` AND `"auth_valid":true` AND
  `"error":"service_unavailable"` AND does NOT contain `"reachable":false`. Before the
  `let reachable = probe.connectivity != ConnectivityStatus::Down` fix in `check_one`, a
  Degraded sensor (HTTP 503) serialized as `"reachable":false` (the Down wire shape) — wrong.

- `test_BC_2_08_002_degraded_envelope_summary_matches_overall_status` (RG-020, AC-WIRE-003/004) —
  drives the full path through `check_sensor_health` MCP handler; asserts the FULL
  `SensorHealthStructuredContent` envelope byte output contains:
  - `"overall_status":"partial"` (not "unhealthy")
  - prose summary does NOT contain `"1 of 1"` healthy count (Degraded ≠ fully healthy)
  - `"healthy_count":0` in `summary_counts`
  - `"unhealthy_count":1` in `summary_counts`
  Before the `SensorHealthResult::is_fully_healthy()` refactor (T-REFACTOR-1) the three-site
  predicate `reachable==true && auth_valid==true && rate_limit.is_none()` (missing
  `&& error.is_none()`) miscounted a Degraded sensor as fully healthy at all three sites:
  `health/mod.rs::aggregate`, `server.rs` summary string, and `resources.rs::HealthSummary`.

**Test result:** 2/2 PASS

**Traces to:** BC-2.08.002 §Postconditions EC-08-009 (v1.8, HS-007 re-gate)

---

### AC-QERR-001 / AC-QERR-002 — Query tool `sensor_errors` wire

**Files:**
- `AC-QERR-001-query-sensor-errors.tape` — VHS tape script
- `AC-QERR-001-query-sensor-errors.gif` — terminal recording
- `AC-QERR-001-query-sensor-errors.webm` — archival recording

**What it shows:**

```
cargo nextest run -p prism-mcp \
  -E 'test(BC_2_11_001_query_sensor_errors) | test(BC_2_11_001_EC_11_091)'
```

Two integration tests that drive the `query` MCP tool and assert on the FULL serialized JSON
wire output:

- `test_BC_2_11_001_query_sensor_errors_surfaces_per_target_http_detail` (RG-016, AC-QERR-001):
  - EC-11-088: sensor returns HTTP 403 with non-empty body → `sensor_errors` entry is
    `"claroty_devices: HTTP 403: <sanitized-body>"` — per-target table name + status + body snippet
  - EC-11-089: sensor returns HTTP 503 with empty body → entry is `"claroty_devices: HTTP 503"` (status-only)
  - Absence invariant: success response contains NO `sensor_errors` key (not `null`, not `[]`)

- `test_BC_2_11_001_EC_11_091_cross_client_partial_failure_sensor_errors_http_format` (RG-017, AC-QERR-002):
  - `clients=["org-a","org-b"]`: org-a's 403 → `sensor_errors: ["xdome_devices: HTTP 403: Unauthorized"]`
  - org-b succeeds → `rows` is non-empty
  - Entry does NOT contain `"sensor error"` (the old `AllTargetsFailed` Display form)
  - This is a regression guard — RG-017 was expected GREEN and confirms existing behavior is preserved.

**Test result:** 2/2 PASS

**Traces to:** BC-2.11.001 §Postconditions sensor_errors wire; EC-11-088/089/090/091

---

### AC-H2-001 / AC-UA-001 — ADR-050 transport hardening

**Files:**
- `AC-H2-001-transport-hardening.tape` — VHS tape script
- `AC-H2-001-transport-hardening.gif` — terminal recording
- `AC-H2-001-transport-hardening.webm` — archival recording

**What it shows:**

Two tests prove the ADR-050 transport hardening is in place:

```
cargo nextest run -p prism-bin --test defect_adapter_tls_xdome_live_001 \
  -E 'test(reqwest_http2_feature_active)'

cargo nextest run -p prism-bin -E 'test(test_build_http_client_sends_user_agent_header)'
```

- `test_reqwest_http2_feature_active` (RG-008, AC-H2-001): reads `Cargo.lock` and asserts that
  `h2` appears inside the `[[package]]` `name = "reqwest"` dependencies block — NOT a
  whole-file grep (h2 was already transitive via hyper; a whole-file match would be green before
  the fix). The scoped assertion proves reqwest's own `http2` feature gate is active.

- `test_build_http_client_sends_user_agent_header` (RG-006, AC-UA-001): starts a wiremock server,
  sends a request via `build_http_client_with_custom_timeout(Duration::from_secs(5))`, and
  captures the request headers; asserts the outgoing `User-Agent` header starts with `"prism/"`.
  Before adding `.user_agent(concat!("prism/", env!("CARGO_PKG_VERSION")))` to the builder chain,
  reqwest sent no `User-Agent` by default — the Claroty WAF/edge rejected the request.

**Test result:** 2/2 PASS

**Traces to:** BC-2.16.002 HTTP Client Compliance postcondition (ADR-050 §D5/§D6)

---

## AC-LIVE-001 — Live Verification Note

AC-LIVE-001 (NO WAIVER) was validated live against `api.claroty.com` for client "monroe" with
the relay removed. Result: HTTP 200 with ≥1 OCSF-normalized row returned. Recorded in STATE
D-2166 (LIVE-PASS, 2026-08-14). The live evidence is stored locally under
`docs/demo-evidence/DEFECT-ADAPTER-TLS-XDOME-LIVE-001/live/` (gitignored) — no customer
records are present in this committed report per AD-017.

AC-LIVE-002 (NO WAIVER) was validated live against `api.claroty.com` for client "monroe" with
an expired credential. Result: `"reachable":true, "auth_valid":false`, suggestion
`"Check credentials — sensor rejected authentication"`. Recorded in STATE D-2165
(LIVE-PASS, 2026-08-15). The live evidence is stored locally under
`docs/demo-evidence/DEFECT-ADAPTER-TLS-XDOME-LIVE-001/live/` (gitignored) — no customer
records are present in this committed report per AD-017.

AC-LIVE-003 (relay file marked deprecated) status: PENDING HUMAN ACTION. The relay file at
`test-soc/live-soc/relay/xdome-relay.py` requires annotation:
`# HISTORICAL: relay was required before DEFECT-ADAPTER-TLS-XDOME-LIVE-001 (2026-08-12). Direct HTTPS now works. Do not use.`
This file is in the operations repo (test-soc/live-soc), outside the prism diff. The human
must apply the annotation or delete the file before this AC can be closed.

---

## Implementation Notes

**Demo vehicle rationale:** This story implements transport-layer and error-mapping changes
across `prism-bin`, `prism-mcp`, `prism-spec-engine`, and `prism-sensors`. There is no
standalone CLI binary invocation that exercises all ACs. The correct evidence vehicle is
`cargo nextest run` against the in-process integration test harness, which starts real
servers (wiremock + prism-dtu-claroty) on ephemeral ports and fires real HTTP requests.
This is equivalent in evidential weight to VHS recordings of a CLI binary — the harness
exercises the exact same code paths that a production prism binary would.

**DTU-only constraint satisfied:** All recordings use wiremock or the DTU clone for fault
injection. No live tenant credentials, no `test-soc/live-soc/clients/*.env` reads, no
keyring access. Placeholder credentials only per AD-017.

**Wire-assertion discipline:** Every test that appears in the recordings asserts on the FULL
serialized JSON byte output — not pre-serialization Rust structs — per CLAUDE.md
wire-shape assertion discipline (2026-07-13, human-approved).

---

## Self-Audit Checklist

- [x] AC-WIRE-001, AC-WIRE-002, AC-WIRE-003, AC-WIRE-004, AC-QERR-001, AC-QERR-002, AC-H2-001, AC-UA-001 each have a recorded demo covering the success path
- [x] Error paths recorded: AC-WIRE-001 shows `auth_valid:false` (error scenario); AC-WIRE-002 shows Degraded-not-Down (error scenario); AC-QERR-001 shows sensor_errors (error scenario)
- [x] Every artifact is linked to a specific AC via the naming convention
- [x] No source code or test files were modified
- [x] Evidence lives under `docs/demo-evidence/DEFECT-ADAPTER-TLS-XDOME-LIVE-001/` (story-scoped, not flat)
- [x] VHS was used (not plain text captures); both `.gif` and `.webm` produced per tape
- [x] Recordings reflect actual test execution. Captured against branch state 8dd8d4285 (the commit immediately preceding the recording commit edfe0a612). Behavioral changes in subsequent fix commits (Arm 2 body → String::new(); Option<String> → String return type; assertion strengthening; doc comment corrections) do not affect any recorded assertion. Evidence validity confirmed at pr-reviewer cycle 3.
- [x] No live tenant data — DTU/wiremock only; AD-017 satisfied
- [x] Nothing committed under `docs/demo-evidence/DEFECT-ADAPTER-TLS-XDOME-LIVE-001/live/`
- [x] AC-LIVE-001 noted as LIVE-PASS with evidence kept locally/gitignored
- [x] AC-LIVE-002 — LIVE-PASS (D-2165, 2026-08-15): expired monroe token → reachable:true + auth_valid:false + suggestion 'Check credentials — sensor rejected authentication'
- [ ] AC-LIVE-003 — PENDING HUMAN ACTION (relay annotation in test-soc/live-soc/relay/xdome-relay.py required)
