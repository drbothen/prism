---
document_type: holdout-scenario
level: L3
id: "HS-032-002"
title: "S-REL-AGENT-VERSION-001: prism-spec-engine outbound HTTP User-Agent header must be prism/PRISM_VERSION, not prism/library-crate-version"
category: "behavioral-correctness"
must_pass: true
priority: P1
epic_id: "E-REL-IDENTITY"
story_source: "S-REL-AGENT-VERSION-001"
version: "1.3"
status: active
used: false
last_evaluated: "2026-09-06"
last_eval_satisfaction: null
single_use: true
producer: product-owner
timestamp: "2026-09-05T00:00:00Z"
modified: "2026-09-06"
phase: 3
inputs:
  - ".factory/specs/architecture/decisions/ADR-064-pre-release-binary-version-identity.md"
  - ".factory/specs/architecture/decisions/ADR-050-workspace-reqwest-tls-backend.md"
  - ".factory/stories/S-REL-AGENT-VERSION-001-agent-version-surfaces.md"
input-hash: "6b2e911"
traces_to: "ADR-050-D6"
behavioral_contracts: []
verification_properties: []
lifecycle_status: active
introduced: "S-REL-AGENT-VERSION-001"
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
notes: "HIDDEN, SINGLE-USE story-level holdout for S-REL-AGENT-VERSION-001 (HS-032 group). Tests Surface B wire-level correctness using override build (PRISM_BUILD_VERSION=1.0.0-beta.1): outbound HTTP requests from prism-spec-engine's build_http_client_with_timeout must carry User-Agent 'prism/1.0.0-beta.1' (PRISM_BUILD_VERSION override), NOT 'prism/0.9.0' (the CARGO_PKG_VERSION of the prism-spec-engine library crate). Key insight: on plain local dev, BOTH a migrated (env!(\"PRISM_VERSION\")) AND un-migrated (env!(\"CARGO_PKG_VERSION\")) Surface B emit 'prism/0.9.0' — the override build makes the gate discriminating. The defect this catches: pipeline.rs user-agent still using env!(\"CARGO_PKG_VERSION\") if the Surface B migration was not applied. CRITICAL OBSERVATION PATH: a plain sensor query (SELECT FROM claroty.devices) routes through prism-bin::spec_driven_adapter::build_http_client_with_timeout (already migrated, non-discriminating). ONLY an HttpLookup enrichment query triggers prism-spec-engine::pipeline::build_http_client_with_timeout (the Surface B migration target). Observed via enrichment echo server on port 19090 receiving a GET from the infusion pipe stage. Test-writer and implementer must NOT read this file."
---

# HS-032-002: prism-spec-engine outbound HTTP User-Agent header wire shape

> **WARNING:** This file is stored in `.factory/holdout-scenarios/` and must
> NEVER be shown to the implementer or test-writer agents. The information
> asymmetry between builder and evaluator is the core quality mechanism.

**Story:** S-REL-AGENT-VERSION-001 (HS-032 group)
**Must Pass:** YES (P1 — blocks story merge)
**Authority:** ADR-050 §D6 normative call:
`concat!("prism/", env!("PRISM_VERSION"))` (ADR-064 §D4 cross-reference). AND ADR-064 §D4 §Surface B injection site table:
`build_http_client_with_timeout` in `pipeline.rs`: `CARGO_PKG_VERSION` → `PRISM_VERSION`.
**Gate:** Story-level holdout gate (HS-032) — runs after LOCAL 3-CLEAN convergence,
before demo recording and PR push. SINGLE-USE. HIDDEN from test-writer and implementer.

---

## Evaluation Disposition — 2026-09-06 (HARNESS-BLOCKED / NOT CONSUMED)

**Adjudication:** HARNESS-BLOCKED — UNEVALUABLE. This is NOT a behavioral FAIL; no wrong value was observed.

**Observed reason:** Sensor adapters did not register at boot in the holdout config. "0 adapters registered" despite org loaded + specs/creds validated. Result: the claroty query returned 0 rows → the `enrich` pipe stage never executed → `prism-spec-engine::pipeline::build_http_client_with_timeout` was never constructed → the enrichment observation endpoint (:19090) was never hit. No `ENRICHMENT-UA:` line appeared.

**Surface-B substance guarantee (basis for human acceptance):** The inline wiremock test `test_infusion_http_client_sends_prism_user_agent` builds the REAL `build_http_client_with_timeout` client and asserts the exact `prism/{PRISM_VERSION}` User-Agent on a live HTTP request (an end-to-end wire assertion), plus the AC-007 source grep. Every LOCAL adversary pass independently verified the Surface-B migration.

**Status:** NOT consumed — `used` stays `false`. Deferred to follow-up story S-REL-HOLDOUT-HARNESS-001 (Canonical Principle Rule 3 concrete anchor). NOT reusable in current form — harness requires a sensor adapter that produces ≥1 row so the `enrich` stage fires.

---

## Scenario

This scenario validates that outbound HTTP requests made by `prism-spec-engine`'s
`build_http_client_with_timeout` carry the correct `User-Agent` header at the wire level.

**Before this story ships:** `build_http_client_with_timeout` in `pipeline.rs` uses
`env!("CARGO_PKG_VERSION")`, which resolves to the `prism-spec-engine` LIBRARY crate
version — not the product version. A sensor tenant's WAF or access log sees
`User-Agent: prism/0.9.0` (the library crate's version) on every inbound request,
even on a `v1.0.0-beta.1` tagged release. This is misleading and inconsistent.

**After this story ships:** `build_http_client_with_timeout` uses `env!("PRISM_VERSION")`,
which resolves per-crate from `crates/prism-spec-engine/build.rs`. On a plain local/develop
build (no PRISM_BUILD_VERSION set), `PRISM_VERSION` resolves to prism-spec-engine's own
`CARGO_PKG_VERSION = "0.9.0"` — identical to the pre-migration value. Cross-surface
coherence with Surface A is a property of tag builds or override builds only (EC-004,
ADR-064 §D4). This scenario therefore uses `PRISM_BUILD_VERSION=1.0.0-beta.1` as a
simulated release override: with the override set, `prism-spec-engine/build.rs` resolves
to `"1.0.0-beta.1"` and the UA becomes `prism/1.0.0-beta.1`. A non-migrated Surface B
(pipeline.rs still using `env!("CARGO_PKG_VERSION")`) ignores the override and still emits
`prism/0.9.0`, making the gate discriminating.

**The defect this scenario catches:** If `pipeline.rs` was not updated to use
`env!("PRISM_VERSION")`, the outbound UA carries `prism/0.9.0` (library CARGO_PKG_VERSION)
even when built with the PRISM_BUILD_VERSION=1.0.0-beta.1 override. If `pipeline.rs` uses
`env!("PRISM_VERSION")` but `crates/prism-spec-engine/build.rs` was not created, the binary
fails to compile (env!("PRISM_VERSION") is undefined) — also a detectable gate failure.

**Observation mechanism:** Run a PrismQL enrichment query that triggers
`prism-spec-engine::pipeline::build_http_client_with_timeout` via the HttpLookup
infusion path. An echo server on port 19090 logs the `User-Agent` header on each GET
request — this is the wire-level observation point. A plain sensor query
(`FROM claroty.devices LIMIT 1`) is NOT a valid observation point: it uses
`prism-bin::spec_driven_adapter::build_http_client_with_timeout` (already migrated,
non-discriminating). Only the HttpLookup enrichment path exercises the Surface B
migration target in `pipeline.rs`.

**BDD supplement:**

**Given** prism is built from the S-REL-AGENT-VERSION-001 story branch with
`PRISM_BUILD_VERSION=1.0.0-beta.1` (override build — see setup step 1)
**And** a sensor data server (port 19089) is running and returns a minimal Claroty
devices response with `uid = "holdout-device-ua-001"`
**And** an enrichment echo server (port 19090) is running and logs the `User-Agent`
header on each GET request
**And** prism MCP stdio is started pointing the sensor at port 19089, infusion at
port 19090, with `PRISM_DTU_MODE=true` (bypasses SSRF for localhost enrichment endpoint)
**When** the PrismQL enrichment query
`SELECT device_uid FROM claroty_devices LIMIT 1 | enrich ua_result(device_uid)`
is issued via the MCP `query` tool (triggering `prism-spec-engine`'s
`build_http_client_with_timeout` to make an HTTP GET to the enrichment echo server)
**Then** the GET request received by the enrichment echo server (port 19090) carries
`User-Agent: prism/1.0.0-beta.1`
**And** the User-Agent does NOT carry the library crate version (`prism/0.9.0`)

---

## Setup Instructions

1. Build the prism binary from the S-REL-AGENT-VERSION-001 story branch HEAD commit
   using a DISTINGUISHING version override so the gate is discriminating (a non-migrated
   Surface B emits `prism/0.9.0` ≠ the override value):
   ```
   PRISM_BUILD_VERSION=1.0.0-beta.1 cargo build -p prism-bin
   ```
   With the override, `prism-spec-engine/build.rs` emits `PRISM_VERSION=1.0.0-beta.1`.
   If `pipeline.rs` uses `env!("PRISM_VERSION")`, the enrichment HTTP GET carries
   `User-Agent: prism/1.0.0-beta.1`. If `pipeline.rs` still uses `env!("CARGO_PKG_VERSION")`,
   the enrichment GET carries `User-Agent: prism/0.9.0` — the FAIL signal.

2. Start a minimal Claroty-format sensor data server on port 19089. This provides one
   device row so the enrichment query has input data to process:
   ```bash
   python3 -c "
   import http.server, socketserver, json

   class SensorHandler(http.server.BaseHTTPRequestHandler):
       def do_POST(self):
           content_len = int(self.headers.get('Content-Length', 0))
           self.rfile.read(content_len)
           resp = json.dumps({
               'devices': [{'uid': 'holdout-device-ua-001', 'asset_id': 'A001',
                            'device_category': 'OT', 'device_subcategory': '',
                            'device_type': 'PLC', 'device_type_family': '',
                            'ip_list': [], 'labels': [], 'mac_list': [],
                            'model': '', 'network_list': [], 'os_category': '',
                            'retired': False, 'risk_score': '5.0', 'vlan_list': []}],
               'total': 1, 'page': 1}).encode()
           self.send_response(200)
           self.send_header('Content-Type', 'application/json')
           self.end_headers()
           self.wfile.write(resp)
       def log_message(self, *args): pass

   with socketserver.TCPServer(('127.0.0.1', 19089), SensorHandler) as s:
       s.serve_forever()
   " &
   SENSOR_PID=$!
   ```

3. Start the enrichment echo server on port 19090. This is the OBSERVATION POINT —
   it prints the `User-Agent` header on each GET request to stdout:
   ```bash
   python3 -c "
   import http.server, socketserver, json

   class EnrichHandler(http.server.BaseHTTPRequestHandler):
       def do_GET(self):
           ua = self.headers.get('User-Agent', '<absent>')
           print('ENRICHMENT-UA:', ua, flush=True)
           resp = json.dumps({'result': 'ok'}).encode()
           self.send_response(200)
           self.send_header('Content-Type', 'application/json')
           self.end_headers()
           self.wfile.write(resp)
       def log_message(self, *args): pass

   with socketserver.TCPServer(('127.0.0.1', 19090), EnrichHandler) as s:
       s.serve_forever()
   " &
   ECHO_PID=$!
   ```

4. Create the holdout config directory structure:
   ```bash
   mkdir -p /tmp/prism-holdout/specs /tmp/prism-holdout/infusions /tmp/prism-holdout/state
   ```

5. Write `/tmp/prism-holdout/prism.toml`:
   ```toml
   spec_dir = "./specs"
   state_dir = "./state"

   [[orgs]]
   org_id = "01906ef2-0000-7000-0000-000000000001"
   org_slug = "holdout-eval"
   ```

6. Copy the Claroty sensor spec (it uses `base_url = "${env.CLAROTY_INSTANCE_URL}"`,
   which will be satisfied by an environment variable at runtime):
   ```bash
   cp crates/prism-sensors/specs/claroty.sensor.toml /tmp/prism-holdout/specs/claroty.sensor.toml
   ```

7. Write the infusion spec to `/tmp/prism-holdout/infusions/holdout-ua.infusion.toml`:
   ```toml
   [infusion]
   infusion_id = "holdout_ua"
   name = "Holdout User-Agent Observer"
   type = "http_lookup"

   [source.http]
   base_url      = "http://127.0.0.1:19090"
   url_template  = "/lookup/${input}"
   method        = "GET"
   response_path = "$.result"

   [[infusion.fields]]
   name        = "ua_result"
   input_field = "device_uid"
   input_type  = "string"
   output_type = "string"
   description = "Triggers prism-spec-engine HttpLookup HTTP GET to port 19090; User-Agent on that request is the Surface B observation point"

   [infusion.pipe_stage]
   adds_columns = ["ua_result"]
   ```
   Note: `device_uid` is the OCSF Arrow field name for the Claroty devices `uid` column
   (derived from `ocsf_field = "device.uid"` with `ocsf_column_naming = true`).
   No `[source.credential]` block needed — the echo server requires no auth.

8. Start prism in MCP stdio mode with the holdout config:
   ```bash
   PRISM_DTU_MODE=true \
   CLAROTY_INSTANCE_URL=http://127.0.0.1:19089 \
   PRISM_CLIENTS_HOLDOUT_EVAL_SENSORS_CLAROTY_BEARER_TOKEN=dummy-holdout-token \
   ./target/debug/prism start --config-dir /tmp/prism-holdout
   ```
   - `PRISM_DTU_MODE=true`: bypasses the SSRF localhost-rejection in
     `HttpLookupSource::new()` so the enrichment endpoint at `127.0.0.1:19090` is reachable.
   - `CLAROTY_INSTANCE_URL`: directs the Claroty sensor fetch to the sensor data server on
     port 19089 (Tier 3 env var substitution in `base_url`).
   - `PRISM_CLIENTS_HOLDOUT_EVAL_SENSORS_CLAROTY_BEARER_TOKEN`: Tier 2 env var credential
     resolution for org "holdout-eval", sensor "claroty", ref "bearer_token". The sensor
     data server does not check auth.

9. Issue the MCP initialize handshake, then the enrichment query:
   ```json
   {"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"holdout-eval","version":"0.0.0"}}}
   ```
   Then the enrichment query (triggers `prism-spec-engine::pipeline::build_http_client_with_timeout`
   via the HttpLookup infusion path):
   ```json
   {
     "jsonrpc": "2.0",
     "id": 2,
     "method": "tools/call",
     "params": {
       "name": "query",
       "arguments": {
         "query": "SELECT device_uid FROM claroty_devices LIMIT 1 | enrich ua_result(device_uid)"
       }
     }
   }
   ```
   Note: query arg key is `query` (not `sql`); table name is `claroty_devices` (not `claroty.devices`); enrichment function is `ua_result` (the `name` field from the infusion spec, not the `infusion_id`).

10. Observe the enrichment echo server stdout (port 19090) for the `ENRICHMENT-UA:` line.
    - PASS (Surface B correctly migrated): `ENRICHMENT-UA: prism/1.0.0-beta.1`
    - FAIL (Surface B defect — pipeline.rs still uses CARGO_PKG_VERSION):
      `ENRICHMENT-UA: prism/0.9.0`

**Cleanup:**
```bash
kill $SENSOR_PID $ECHO_PID 2>/dev/null
```

---

## Behavioral Contract Linkage

| Source | Clause | Scenario Aspect |
|--------|--------|-----------------|
| ADR-050 §D6 | `build_http_client_with_timeout` in `crates/prism-spec-engine/src/pipeline.rs` MUST use `.user_agent(concat!("prism/", env!("PRISM_VERSION")))` | Primary assertion: UA header on wire equals "prism/1.0.0-beta.1" (with PRISM_BUILD_VERSION=1.0.0-beta.1 override) |
| ADR-064 §D4 §Surface B | `pipeline.rs` injection site: `CARGO_PKG_VERSION` → `PRISM_VERSION` | Surface B migration correctness |
| ADR-064 §D4 §Surface B | `crates/prism-spec-engine/build.rs` (new file) emits `PRISM_VERSION` | Build script must exist and emit correctly for env!("PRISM_VERSION") to resolve |

---

## Verification Approach

1. Observe the enrichment echo server (port 19090) stdout for the `ENRICHMENT-UA:` line.
   This appears immediately when `prism-spec-engine::pipeline::build_http_client_with_timeout`
   makes the HttpLookup GET request during the enrichment pipe stage execution.

2. If no `ENRICHMENT-UA:` line appears (enrichment query returned an error, sensor data
   server not reachable, or infusion spec not loaded): record SETUP-FAILURE.

3. If the query tool returns an MCP error but the sensor data server (port 19089) received
   a request (visible in sensor server logs if enabled): the sensor fetch succeeded but
   enrichment failed — check that `PRISM_DTU_MODE=true` was set and infusion TOML was
   written correctly to `/tmp/prism-holdout/infusions/holdout-ua.infusion.toml`.

4. **Primary assertion:** Assert the `ENRICHMENT-UA:` value does NOT equal `prism/0.9.0`.
   - If UA is `prism/0.9.0`: record FAIL on "library-version-absent" dimension.
     This is the Surface B defect — `pipeline.rs` still using `env!("CARGO_PKG_VERSION")`
     which resolves to prism-spec-engine's own library crate version regardless of the
     PRISM_BUILD_VERSION override.

5. **Secondary assertion:** Assert the `ENRICHMENT-UA:` value equals `prism/1.0.0-beta.1`.
   - `"1.0.0-beta.1"` is the expected `PRISM_VERSION` when built with
     `PRISM_BUILD_VERSION=1.0.0-beta.1` (setup step 1).
   - If UA is `prism/1.0.0-beta.1`: record PASS.
   - If UA starts with `prism/` with a non-empty suffix ≠ `"0.9.0"` and ≠ `"1.0.0-beta.1"`
     (e.g., `prism/1.0.0-dev`): record PARTIAL — Surface B wiring is active but the
     PRISM_BUILD_VERSION=1.0.0-beta.1 override was not picked up. Verify the build
     invocation in setup step 1 correctly exported the environment variable.

6. Quote the observed `ENRICHMENT-UA:` value in the evaluation report.

---

## Evaluation Rubric

Rate each dimension 0.0–1.0; take weighted average. Satisfying threshold: ≥ 0.75.

- **Enrichment HTTP GET received by echo server** (weight: 0.20): Does the enrichment
  query trigger an outbound HTTP GET to the echo server on port 19090 (visible as an
  `ENRICHMENT-UA:` line in the echo server stdout)?
  Full credit (1.0): `ENRICHMENT-UA:` line observed.
  Zero credit (0.0): no line observed — record SETUP-FAILURE (sensor data server, enrichment
  echo server, or infusion spec not correctly configured).

- **Library version absent from UA** (weight: 0.45): Is the User-Agent header something
  other than the pre-migration library-crate-version form?
  Full credit (1.0): UA does not match the pre-migration library version pattern.
  Zero credit (0.0): UA matches the old library version — Surface B migration not applied.

- **Correct product version in UA** (weight: 0.35): Is the User-Agent `prism/1.0.0-beta.1`
  (the injected PRISM_BUILD_VERSION override)?
  Full credit (1.0): UA = `prism/1.0.0-beta.1`.
  Partial credit (0.5): UA starts with `prism/` with a non-empty, non-library-crate suffix
  (wiring active but override not picked up; verify build invocation in setup step 1).
  Zero credit (0.0): UA is absent, empty, or malformed.

---

## Edge Conditions

- **Sensor data server (port 19089) not running:** Prism cannot fetch any device rows;
  enrichment query fails with no input. Record SETUP-FAILURE.

- **Enrichment echo server (port 19090) not running:** Prism's enrichment HTTP GET fails
  (connection refused). No `ENRICHMENT-UA:` line appears. Record SETUP-FAILURE.

- **`PRISM_DTU_MODE=true` not set:** `HttpLookupSource::new()` rejects the loopback
  address via SSRF protection. Prism fails to load the infusion spec and the enrichment
  query returns an error. No `ENRICHMENT-UA:` line appears. Record SETUP-FAILURE.

- **Infusion TOML missing or malformed:** Prism loads specs from
  `/tmp/prism-holdout/infusions/`; a missing or invalid TOML causes the infusion to fail
  silently or with an error. Pipe stage not executed. Record SETUP-FAILURE.

- **`PRISM_CLIENTS_HOLDOUT_EVAL_SENSORS_CLAROTY_BEARER_TOKEN` not set:** Credential
  resolution fails; sensor fetch returns credential error. No device rows returned; no
  enrichment triggered. Record SETUP-FAILURE.

- **User-Agent = "prism/1.0.0-dev" (unexpected):** Record PARTIAL — version wiring is
  active (env!("PRISM_VERSION") is compiled in, not env!("CARGO_PKG_VERSION")), but the
  PRISM_BUILD_VERSION=1.0.0-beta.1 override was not picked up by prism-spec-engine's
  build.rs. Check that the build invocation in setup step 1 exported the env var correctly.
  On a plain local dev build without override, prism-spec-engine's PRISM_VERSION falls
  back to its own CARGO_PKG_VERSION="0.9.0" — so "prism/1.0.0-dev" indicates an
  unexpected build configuration (e.g., an env residue from a prior build).

- **Query returns MCP result but no `ENRICHMENT-UA:` line appears:** Enrichment pipe
  stage may have been skipped (e.g., no matching rows to enrich, or infusion id mismatch
  between TOML and query). Verify infusion TOML has `infusion_id = "holdout_ua"` matching
  the `| enrich holdout_ua(device_uid)` call. Record SETUP-FAILURE if enrichment was
  clearly skipped.

---

## Failure Guidance

If this scenario fails, send to the builder (one-line, no scenario specifics):

`"HOLDOUT FAIL: HS-032-002 (satisfaction: X.XX) — prism-spec-engine outbound HTTP User-Agent header carries an unexpected version string; check build_http_client_with_timeout in crates/prism-spec-engine/src/pipeline.rs uses env!(\"PRISM_VERSION\") not env!(\"CARGO_PKG_VERSION\") (ADR-064 D4 §Surface B + ADR-050 §D6), and that crates/prism-spec-engine/build.rs exists and emits PRISM_VERSION"`

Do NOT disclose: the specific version string expected, the old library version value, or
the observation mechanism (DTU log vs echo server).

---

## Category: real-world-corpus

| Field | Description |
|-------|-------------|
| corpus_source | Two-server holdout setup: sensor data server (port 19089, returns one Claroty device row) + enrichment echo server (port 19090, logs User-Agent on GET); override build with PRISM_BUILD_VERSION=1.0.0-beta.1; triggering query `SELECT device_uid FROM claroty.devices LIMIT 1 \| enrich holdout_ua(device_uid)` |
| corpus_size | Single enrichment HTTP GET request; single User-Agent header assertion on `ENRICHMENT-UA:` output line |
| known_edge_cases | `ENRICHMENT-UA: prism/0.9.0` (library version; Surface B pre-migration defect — FAIL); `ENRICHMENT-UA:` absent (SETUP-FAILURE: enrichment path not exercised); `ENRICHMENT-UA: prism/1.0.0-dev` (wiring active but override not picked up — PARTIAL) |
| false_positive_threshold | Near-zero: `ENRICHMENT-UA: prism/1.0.0-beta.1` is an unambiguous PASS for the override build |
| false_negative_threshold | Near-zero: `ENRICHMENT-UA: prism/0.9.0` is an unambiguous Surface B defect (CARGO_PKG_VERSION not replaced by PRISM_VERSION) |

**Known-good corpus:** prism binary from this story branch built with `PRISM_BUILD_VERSION=1.0.0-beta.1`
and correct Surface B wiring (pipeline.rs uses `env!("PRISM_VERSION")` + build.rs emits
PRISM_VERSION=1.0.0-beta.1 from override) — expected enrichment echo server output:
`ENRICHMENT-UA: prism/1.0.0-beta.1`.

**Known-problematic corpus:** prism binary where `pipeline.rs` still uses
`env!("CARGO_PKG_VERSION")` (pre-migration), built with the same override — expected:
`ENRICHMENT-UA: prism/0.9.0` (library crate version; PRISM_BUILD_VERSION override is
irrelevant because CARGO_PKG_VERSION bypasses the build.rs injection entirely). This is
the primary discriminating FAIL signal. Note: a plain sensor query against this same
binary would still emit `prism/1.0.0-beta.1` (prism-bin's sensor-fetch client already
migrated) — this confirms why the enrichment path is the correct observation path.

---

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.3 | hs-032-gate-disposition-2026-09-06 | 2026-09-06 | product-owner | HARNESS-BLOCKED / UNEVALUABLE (not a behavioral FAIL). Human-adjudicated ACCEPT-ON-SUBSTANCE 2026-09-06. Observed reason: sensor adapters did not register at boot in holdout config ("0 adapters registered" despite org loaded + specs/creds validated) — claroty query returned 0 rows; `enrich` pipe stage never executed; `prism-spec-engine::pipeline::build_http_client_with_timeout` never constructed; enrichment observation endpoint (:19090) never hit. NOT consumed (used stays false). Surface-B substance guaranteed by inline wiremock test `test_infusion_http_client_sends_prism_user_agent` (builds REAL `build_http_client_with_timeout` + asserts exact `prism/{PRISM_VERSION}` UA on live HTTP wire) + AC-007 source grep + every LOCAL adversary pass. Deferred to S-REL-HOLDOUT-HARNESS-001 (Canonical Principle Rule 3 anchor). Fixed 4 §Setup interface errors: (1) `prism start --mcp-stdio` → `prism start`; (2) query arg key `sql` → `query`; (3) table `claroty.devices` → `claroty_devices`; (4) enrich function `holdout_ua` → `ua_result` (field name from infusion spec, not infusion_id). Added §Evaluation Disposition block. |
| 1.2 | f-srav-high-001-enrichment-path-redesign | 2026-09-06 | product-owner | F-SRAV-HIGH-001 fix (observation-path correction). Redesigned to use HttpLookup enrichment path, which exercises `prism-spec-engine::pipeline::build_http_client_with_timeout` — the actual Surface B migration target. Root cause of 002/003 v1.1 defect: plain sensor query (`SELECT * FROM claroty.devices`) routes through `prism-bin::spec_driven_adapter::build_http_client_with_timeout` (already migrated to PRISM_VERSION), not through the spec-engine pipeline client — observation was non-discriminating regardless of whether Surface B was migrated. Redesigned §Setup: (1) sensor data server on port 19089 (returns one minimal Claroty device row); (2) enrichment echo server on port 19090 (observation point — logs User-Agent on GET requests); (3) infusion TOML `holdout-ua.infusion.toml` with HttpLookup type pointing at port 19090; (4) PRISM_DTU_MODE=true bypasses SSRF for localhost enrichment endpoint; (5) triggering query `SELECT device_uid FROM claroty.devices LIMIT 1 \| enrich holdout_ua(device_uid)`. Updated §Scenario (observation mechanism + BDD), §Verification, §Rubric, §Edge Conditions, §real-world-corpus, notes. |
| 1.1 | hs-032-override-build-model-fix | 2026-09-06 | product-owner | HIGH defect fix (F-SRAV-HIGH-001 root cause). Rewrote to override-build model: evaluator builds with PRISM_BUILD_VERSION=1.0.0-beta.1. Expected UA changed from "prism/1.0.0-dev" (impossible on local dev — prism-spec-engine CARGO_PKG_VERSION=0.9.0) to "prism/1.0.0-beta.1". Gate is now discriminating: non-migrated Surface B (CARGO_PKG_VERSION) emits "prism/0.9.0" ≠ "prism/1.0.0-beta.1"; both pre-migration and post-migration emitted "prism/0.9.0" on plain local dev (non-discriminating). Corrected §Scenario "After this story ships" local-dev claim. Updated §Setup (build command), §BDD, §Verification, §Rubric, §Edge Conditions, §real-world-corpus, notes. |
| 1.0 | s-rel-agent-version-001-holdout-authoring | 2026-09-05 | product-owner | Initial authoring. HS-032 group for S-REL-AGENT-VERSION-001. Spec-engine outbound HTTP User-Agent wire-level test: UA must be prism/1.0.0-dev on local dev, NOT prism/{library-version}. Catches pipeline.rs CARGO_PKG_VERSION → PRISM_VERSION migration gap. ADR-050 §D6 + ADR-064 D4 §Surface B authority. Claroty DTU as primary observation target; echo server as fallback. SINGLE-USE. |
