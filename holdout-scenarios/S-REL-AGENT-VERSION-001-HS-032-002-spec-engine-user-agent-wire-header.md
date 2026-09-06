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
version: "1.1"
status: active
used: false
last_evaluated: null
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
input-hash: "dad7dce"
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
notes: "HIDDEN, SINGLE-USE story-level holdout for S-REL-AGENT-VERSION-001 (HS-032 group). Tests Surface B wire-level correctness using override build (PRISM_BUILD_VERSION=1.0.0-beta.1): outbound HTTP requests from prism-spec-engine's build_http_client_with_timeout must carry User-Agent 'prism/1.0.0-beta.1' (PRISM_BUILD_VERSION override), NOT 'prism/0.9.0' (the CARGO_PKG_VERSION of the prism-spec-engine library crate). Key insight: on plain local dev, BOTH a migrated (env!(\"PRISM_VERSION\")) AND un-migrated (env!(\"CARGO_PKG_VERSION\")) Surface B emit 'prism/0.9.0' — the override build makes the gate discriminating. The defect this catches: pipeline.rs user-agent still using env!(\"CARGO_PKG_VERSION\") if the Surface B migration was not applied. Observed via DTU access log or request echo endpoint. Test-writer and implementer must NOT read this file."
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

**Observation mechanism:** Run a PrismQL query against a sensor with a working DTU.
The DTU logs or request-echo endpoint reveals the User-Agent header on the inbound HTTP
request from prism-spec-engine's pipeline. Alternatively, use a local echo server
(e.g., `nc -l` capturing the raw HTTP request) configured as a dummy sensor endpoint.

**BDD supplement:**

**Given** prism is built from the S-REL-AGENT-VERSION-001 story branch with
`PRISM_BUILD_VERSION=1.0.0-beta.1` (override build — see setup step 1)
**And** prism MCP stdio is started with a Claroty DTU configured (the DTU serves as
the observable HTTP endpoint)
**When** a PrismQL SELECT query is issued via the MCP `query` tool against the Claroty
sensor (triggering prism-spec-engine's pipeline to make an outbound HTTP request)
**Then** the HTTP request received by the DTU (observable via DTU access log or
request-echo) carries `User-Agent: prism/1.0.0-beta.1`
**And** the User-Agent does NOT carry the library crate version (`prism/0.9.0`)

---

## Setup Instructions

1. Build the prism binary from the S-REL-AGENT-VERSION-001 story branch HEAD commit
   using a DISTINGUISHING version override so the gate is discriminating (a non-migrated
   Surface B emits `prism/0.9.0` ≠ the override value):
   ```
   PRISM_BUILD_VERSION=1.0.0-beta.1 cargo build -p prism-bin
   ```
   This simulates a release build: `prism-spec-engine/build.rs` (Surface B) reads
   `PRISM_BUILD_VERSION=1.0.0-beta.1` and emits it as `PRISM_VERSION`. If `pipeline.rs`
   uses `env!("PRISM_VERSION")`, the UA becomes `prism/1.0.0-beta.1`. If `pipeline.rs`
   still uses `env!("CARGO_PKG_VERSION")`, the UA remains `prism/0.9.0` — the FAIL signal.

2. Start the Claroty DTU (`prism-dtu-claroty`) locally using the binary from step 1. The DTU serves fixture data
   and logs incoming HTTP requests including headers.
   ```
   # Start DTU (check crates/prism-dtu-claroty/README.md or scripts for the start command)
   cargo run -p prism-dtu-claroty
   ```
   The DTU's stdout/stderr should include access log lines showing request headers.

3. Start prism in MCP stdio mode with the Claroty sensor configured to point to the
   local DTU instance:
   ```
   prism start --mcp-stdio
   ```

4. Issue a minimal PrismQL SELECT query via the MCP `query` tool that triggers the
   spec-engine pipeline to make an outbound HTTP call to the Claroty DTU:
   ```json
   {
     "jsonrpc": "2.0",
     "id": 2,
     "method": "tools/call",
     "params": {
       "name": "query",
       "arguments": { "sql": "SELECT * FROM claroty.devices LIMIT 1" }
     }
   }
   ```
   (Any table that triggers a real HTTP call to the DTU suffices. `devices` is a reliable
   target with the standard Claroty fixture.)

5. Capture the DTU access log output showing the inbound HTTP request headers.
   Look for the `User-Agent:` header in the DTU's log.

**Alternative observation method (if DTU does not log headers):**

Use a minimal echo HTTP server as the sensor endpoint:
```bash
# Start a simple request-echo server on port 19090
python3 -c "
import http.server, socketserver

class EchoHandler(http.server.BaseHTTPRequestHandler):
    def do_POST(self):
        print('User-Agent:', self.headers.get('User-Agent'))
        self.send_response(200)
        self.end_headers()
        self.wfile.write(b'{\"data\": []}')
    def log_message(self, *args): pass

socketserver.TCPServer(('', 19090), EchoHandler).serve_forever()
"
```
Configure prism's sensor spec to point to `http://localhost:19090`.

---

## Behavioral Contract Linkage

| Source | Clause | Scenario Aspect |
|--------|--------|-----------------|
| ADR-050 §D6 | `build_http_client_with_timeout` in `crates/prism-spec-engine/src/pipeline.rs` MUST use `.user_agent(concat!("prism/", env!("PRISM_VERSION")))` | Primary assertion: UA header on wire equals "prism/1.0.0-beta.1" (with PRISM_BUILD_VERSION=1.0.0-beta.1 override) |
| ADR-064 §D4 §Surface B | `pipeline.rs` injection site: `CARGO_PKG_VERSION` → `PRISM_VERSION` | Surface B migration correctness |
| ADR-064 §D4 §Surface B | `crates/prism-spec-engine/build.rs` (new file) emits `PRISM_VERSION` | Build script must exist and emit correctly for env!("PRISM_VERSION") to resolve |

---

## Verification Approach

1. Observe the DTU access log (or echo server output) for the User-Agent header on the
   inbound HTTP POST request from prism.

2. If no request arrives at the DTU (prism reports sensor error, DTU not reachable):
   record SETUP-FAILURE.

3. If a request arrives but headers are not logged by the DTU: record SETUP-FAILURE
   for header-observation dimension; use the echo server alternative.

4. **Primary assertion:** Assert User-Agent header does NOT contain the old library
   version string.
   - If UA is `prism/0.9.0`: record FAIL on "library-version-absent" dimension.
     This is the Surface B defect — pipeline.rs still using `CARGO_PKG_VERSION`.
   - Note: The exact old library version may differ from `"0.9.0"` depending on
     workspace state; the key discriminator is that it should NOT equal any library
     crate version (which would be != the product version).

5. **Secondary assertion:** Assert User-Agent header equals `prism/1.0.0-beta.1`.
   - `"1.0.0-beta.1"` is the expected `PRISM_VERSION` when built with
     `PRISM_BUILD_VERSION=1.0.0-beta.1` override (setup step 1).
   - If UA is `prism/1.0.0-beta.1`: record PASS.
   - If UA contains `"prism/"` with a non-empty, non-library-version suffix (e.g.,
     `prism/1.0.0-dev`): record PARTIAL (Surface B wiring active but override not picked
     up; verify build invocation in setup step 1).

6. Quote the observed User-Agent header value in the evaluation report.

---

## Evaluation Rubric

Rate each dimension 0.0–1.0; take weighted average. Satisfying threshold: ≥ 0.75.

- **Query executes (HTTP request made)** (weight: 0.20): Does the PrismQL query trigger
  an outbound HTTP call to the DTU (request observed in log)?
  Full credit (1.0): request observed.
  Zero credit (0.0): no request observed — record SETUP-FAILURE.

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

- **DTU not running:** Record SETUP-FAILURE, not behavioral FAIL.

- **DTU access log does not include headers:** Use the echo server alternative setup.

- **prism sensor config missing / spec-engine fails to load:** Record SETUP-FAILURE.

- **User-Agent = "prism/1.0.0-dev" (unexpected):** Record PARTIAL — version wiring is
  active (env!("PRISM_VERSION") is compiled in, not env!("CARGO_PKG_VERSION")), but the
  PRISM_BUILD_VERSION=1.0.0-beta.1 override was not picked up by prism-spec-engine's
  build.rs. Check that the build invocation in setup step 1 correctly set the environment
  variable. Note: on a plain local dev build without override, prism-spec-engine's
  PRISM_VERSION falls back to its own CARGO_PKG_VERSION="0.9.0", not "1.0.0-dev" — so
  "prism/1.0.0-dev" indicates an unexpected build configuration where the crate version
  was set to "1.0.0-dev" or an env residue from a prior build.

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
| corpus_source | Claroty DTU (`prism-dtu-claroty`) — real HTTP request/response cycle via MCP query tool; override build with PRISM_BUILD_VERSION=1.0.0-beta.1 |
| corpus_size | Single outbound HTTP request; single User-Agent header assertion |
| known_edge_cases | UA = "prism/0.9.0" (library version; pre-migration defect — FAIL); UA absent (reqwest client construction failure); UA = "prism/1.0.0-dev" (wiring active but override not picked up — PARTIAL) |
| false_positive_threshold | Near-zero: "prism/1.0.0-beta.1" is an unambiguous pass for the override build |
| false_negative_threshold | Near-zero: "prism/0.9.0" in UA is an unambiguous Surface B defect |

**Known-good corpus:** prism binary from this story branch built with `PRISM_BUILD_VERSION=1.0.0-beta.1`
and correct Surface B wiring — expected: `User-Agent: prism/1.0.0-beta.1` on every outbound
HTTP request from the spec-engine pipeline (prism-spec-engine/build.rs reads PRISM_BUILD_VERSION
override and emits it as PRISM_VERSION).

**Known-problematic corpus:** prism binary where `pipeline.rs` still uses
`env!("CARGO_PKG_VERSION")` (pre-migration), built with the same override — expected:
`User-Agent: prism/0.9.0` (library crate version; override is irrelevant because
CARGO_PKG_VERSION bypasses the build.rs injection entirely). This is the primary
discriminating FAIL signal.

---

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.1 | hs-032-override-build-model-fix | 2026-09-06 | product-owner | HIGH defect fix (F-SRAV-HIGH-001 root cause). Rewrote to override-build model: evaluator builds with PRISM_BUILD_VERSION=1.0.0-beta.1. Expected UA changed from "prism/1.0.0-dev" (impossible on local dev — prism-spec-engine CARGO_PKG_VERSION=0.9.0) to "prism/1.0.0-beta.1". Gate is now discriminating: non-migrated Surface B (CARGO_PKG_VERSION) emits "prism/0.9.0" ≠ "prism/1.0.0-beta.1"; both pre-migration and post-migration emitted "prism/0.9.0" on plain local dev (non-discriminating). Corrected §Scenario "After this story ships" local-dev claim. Updated §Setup (build command), §BDD, §Verification, §Rubric, §Edge Conditions, §real-world-corpus, notes. |
| 1.0 | s-rel-agent-version-001-holdout-authoring | 2026-09-05 | product-owner | Initial authoring. HS-032 group for S-REL-AGENT-VERSION-001. Spec-engine outbound HTTP User-Agent wire-level test: UA must be prism/1.0.0-dev on local dev, NOT prism/{library-version}. Catches pipeline.rs CARGO_PKG_VERSION → PRISM_VERSION migration gap. ADR-050 §D6 + ADR-064 D4 §Surface B authority. Claroty DTU as primary observation target; echo server as fallback. SINGLE-USE. |
