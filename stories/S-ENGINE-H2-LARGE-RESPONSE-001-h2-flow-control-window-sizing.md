---
document_type: story
story_id: S-ENGINE-H2-LARGE-RESPONSE-001
title: "H2 flow-control window sizing — 4 MiB initial stream/connection windows on all four production reqwest client builders (ADR-059 Option A)"
level: "L4"
wave: xdome-wave-a
epic_id: E-XDOME-EXPANSION
priority: P0
status: draft
# BC status: BC-2.16.002 v2.36 active — H2 Flow-Control Window Sizing postcondition authored
# and anchored to this story ID. Status remains draft until remove-uncertainty pass CLEAN.
producer: story-writer
timestamp: "2026-08-26T00:00:00Z"
version: "1.1"
modified: "2026-08-26"
phase: 3
cycle: v1.0.0-brownfield
inputs:
  - ".factory/specs/behavioral-contracts/BC-2.16.002-multi-step-fetch-pipeline.md"
  - ".factory/specs/architecture/decisions/ADR-059-h2-flow-control-window-large-response-reliability.md"
input-hash: "781c16c"
# input-hash: verified 2026-08-26 D-2311 — canonical tool confirms 781c16c current against BC-2.16.002 v2.37 + ADR-059 v1.1 on disk
traces_to: "BC-2.16.002"
points: 5
estimated_days: 1
tdd_mode: strict
subsystems: [SS-01, SS-16]
# Subsystem anchor justifications (ARCH-INDEX Subsystem Registry):
#   SS-01 (Sensor Adapters) owns this story's scope because the canonical production client
#     factory `build_http_client_with_custom_timeout` lives in `crates/prism-bin/src/spec_driven_adapter.rs`,
#     which is the production HTTP adapter entry point for all TOML-spec-driven sensor adapters.
#     SS-01 governs the outbound sensor HTTP client surface per ARCH-INDEX.
#   SS-16 (Spec Engine) owns this story's scope because `build_http_client_with_timeout` in
#     `crates/prism-spec-engine/src/pipeline.rs` is the HttpLookupSource infusion outbound factory,
#     and SS-16 is the canonical owner of prism-spec-engine per ARCH-INDEX.
target_module: prism-bin
crates_touched: [prism-bin, prism-spec-engine]
# crates_touched:
#   prism-bin:
#     MODIFY src/spec_driven_adapter.rs — `build_http_client_with_custom_timeout`: add 2 h2 window builder calls
#     MODIFY src/boot.rs — `plugin_load_step_with_audit`: add 2 h2 window builder calls to BOTH inline builder sites
#       (PRISM_DISABLE_PLUGIN_LOAD branch + normal branch)
#     CREATE tests/bc_2_16_002_h2_window_sizing.rs — RG-001, RG-003, RG-004 (SETTINGS-frame
#       assertion tests using h2 crate server handshake on plain TCP via http2_prior_knowledge)
#     MODIFY Cargo.toml [dev-dependencies]: add h2 = "0.4"
#   prism-spec-engine:
#     MODIFY src/pipeline.rs — `build_http_client_with_timeout`: add 2 h2 window builder calls
#     CREATE tests/bc_2_16_002_h2_window.rs — RG-002 SETTINGS-frame assertion test
#     MODIFY Cargo.toml [dev-dependencies]: add h2 = "0.4"
capabilities:
  - CAP-029
behavioral_contracts:
  - BC-2.16.002
  # BC-2.16.002 v2.36 — §Postconditions "H2 Flow-Control Window Sizing (ADR-059 §D7)":
  # All four production outbound sensor/plugin reqwest client factories MUST configure
  # 4 MiB initial stream window + 4 MiB initial connection window (Option A — fixed windows,
  # NO http2_adaptive_window). DTU test clients excluded.
  # This postcondition is the sole governing contract for this story.
verification_properties: []
holdout_scenarios: []
# holdout_scenarios: PO authors 2–4 hidden SINGLE-USE scenarios at remove-uncertainty time.
# Stored under the holdout directory; test-writer and implementer MUST NOT read them.
# Story-level holdout gate is BLOCKING before demo/push (human-approved 2026-07-13).
depends_on: []
# depends_on: No delivery-time dependency — S-ADR058-OCSF-ROUTING-001 and earlier stories
#   already merged. The http2 reqwest feature is confirmed enabled in prism-bin, prism-spec-engine,
#   and prism-sensors Cargo.toml per ADR-059 §D5 (ADR-050 amend). The two window builder methods
#   (http2_initial_stream_window_size, http2_initial_connection_window_size) are valid on
#   reqwest 0.12.28 — confirmed by ADR-059 §Context.
#   PROHIBITION: http2_adaptive_window(true) MUST NOT be added to any factory site — it overrides
#   both window setters back to the RFC default 65,535 bytes, negating the fix. See §Uncertainty
#   Resolution Claim #2. Architect chose Option A (2026-08-26 correction).
blocks: [S-CLAROTY-VULNS-001]
# blocks: S-CLAROTY-VULNS-001 is blocked from live-green on the live claroty_vulnerabilities
#   fetch because without the h2 window fix the ~1.1 MB/page response stalls for 30s
#   (DEFECT-1, ADR-059 §Context). This fix unblocks live-green.
acceptance_criteria_count: 5
risk: LOW
# Risk updated (2026-08-26 v1.1): Architect chose Option A (fixed 4 MiB windows, no adaptive).
# Mechanism is exactly two reqwest builder calls per site. Semantic ambiguity blocker from v1.0
# is resolved. SETTINGS-frame assertion Red Gate is deterministic (h2 crate inspection, not
# loopback timing). Risk downgraded to LOW.
assumption_validations:
  - claim: "reqwest 0.12.28 ClientBuilder exposes http2_initial_stream_window_size and http2_initial_connection_window_size; window setters accept `impl Into<Option<u32>>`; 4*1024*1024 = 4194304 is a valid u32 arg; both gated by the `http2` feature."
    verdict: "CONFIRMED (Context7 reqwest source docs.rs — Config fields under #[cfg(feature=\"http2\")]; docs.rs/reqwest ClientBuilder). Arg type is `impl Into<Option<u32>>`; bare literal 4*1024*1024 compiles via u32 inference."
  - claim: "Architect chose Option A: fixed 4 MiB windows WITHOUT http2_adaptive_window(true). The two explicit setters are not overridden. Initial stream and connection windows are deterministically 4 MiB."
    verdict: "RESOLVED — architect correction 2026-08-26. http2_adaptive_window(true) MUST NOT be added; it resets both windows to 65,535. Only the two explicit window setters are applied. See §Uncertainty Resolution Claim #2."
  - claim: "h2 crate server handshake on plain TCP can read the client's advertised SETTINGS_INITIAL_WINDOW_SIZE via connection.remote_settings().initial_window_size()."
    verdict: "CONFIRMED — h2 0.4.18 `h2::server::handshake(io).await` returns a `Connection`; `connection.remote_settings()` returns the client-advertised SETTINGS; `initial_window_size()` returns `Option<u32>`. Assert `== Some(4_194_304)`. FAILS at default 65,535 without the fix, PASSES at 4,194,304 after the fix."
risk_mitigations:
  - "RESOLVED (2026-08-26): Architect chose Option A — exactly two fixed window builder calls, NO http2_adaptive_window. The semantic ambiguity blocker from v1.0 is closed."
  - "Red Gate uses SETTINGS-frame assertion (h2 crate server inspects SETTINGS_INITIAL_WINDOW_SIZE), not loopback timing. Deterministic: FAILS at default 65,535, PASSES at 4,194,304."
---

# S-ENGINE-H2-LARGE-RESPONSE-001: H2 Flow-Control Window Sizing — 4 MiB Initial Windows on All Four Production reqwest Client Builders (Option A)

## Authority

**BC-2.16.002 v2.36 §Postconditions "H2 Flow-Control Window Sizing (ADR-059 §D7)"** is the
governing contract. Read it in full before implementing. The postcondition enumerates the four
production factory sites by function name, states the 4 MiB values, and explicitly excludes DTU
test clients. The postcondition reflects the architect's Option A choice: fixed 4 MiB windows
without `http2_adaptive_window`.

**ADR-059 §D7** is the decision that introduces this requirement. Read §Context (defect
evidence), §Decision (exact window values and factory scope), §Rationale, and §Alternatives
Considered before implementing. ADR-059 §D7 explicitly amends ADR-050 §D6 without superseding
any prior decision. The architect's Option A correction (2026-08-26) is incorporated in ADR-059
§Decision.

**ADR-050 §D6** enumerates the same four production factory sites. This story adds two new
builder calls to each of those four sites. ADR-050 §D1/D2 (rustls-tls mandatory) and §D6
(User-Agent) constraints are UNCHANGED — do not alter `.user_agent(...)`,
`default-features = false`, or `features = ["rustls-tls"]` settings.

**Confirmation experiment (ADR-059 §Rationale, updated for Option A):** Bind a plain-TCP
listener on localhost. Build the production reqwest client factory chain with
`.http2_prior_knowledge()` (plain-TCP h2, no TLS/ALPN). Use the `h2` crate server to perform
`h2::server::handshake(io).await` and inspect `connection.remote_settings().initial_window_size()`.
Assert `== Some(4_194_304)`. This SETTINGS-frame assertion FAILS before the fix (default 65,535)
and PASSES after adding both window setter calls. See §Uncertainty Resolution for the full
rationale for this design over a loopback timing test.

---

## Uncertainty Resolution (remove-uncertainty pass, 2026-08-26; updated with architect resolution)

This pass validated the ADR-059/story technology claims against LIVE library sources
(Context7 reqwest source docs.rs; Perplexity deep-research over docs.rs/hyper source/h2 docs).
Pinned versions validated: reqwest 0.12.28, h2 0.4.18, hyper 1.9.0.

### Claim #1 — builder method names, argument types, feature gate: CONFIRMED

`reqwest::ClientBuilder` exposes `http2_initial_stream_window_size` and
`http2_initial_connection_window_size`; both are gated by the `http2` cargo feature (Context7
reqwest source: the corresponding `Config` fields carry `#[cfg(feature = "http2")]`).
**Precision note:** the window setters accept `sz: impl Into<Option<u32>>` — NOT a bare `u32`.
A suffixless literal `4 * 1024 * 1024` (= `4194304`, a valid `u32`) still compiles because `u32`
is the only integer type satisfying `Into<Option<u32>>`, so inference resolves it.
_Source: Context7 `/websites/rs_reqwest` ClientBuilder; docs.rs/reqwest/latest ClientBuilder._

### Claim #2 — RESOLVED: architect chose Option A (fixed windows, NO http2_adaptive_window)

The remove-uncertainty pass found a critical semantic contradiction: `http2_adaptive_window(true)`
**OVERRIDES and discards** the explicit `http2_initial_stream_window_size` /
`http2_initial_connection_window_size` values. Verbatim reqwest doc text: _"Sets whether to use
an adaptive flow control. Enabling this will override the limits set in
`http2_initial_stream_window_size` and `http2_initial_connection_window_size`."_ The underlying
hyper implementation resets both windows to the RFC default (`SPEC_WINDOW_SIZE = 65_535`) when
adaptive is enabled. reqwest applies `adaptive_window` AFTER the explicit window setters during
`build()`, so the explicit calls become no-ops.

**Architect resolution (2026-08-26):** Option A — use ONLY the two explicit window setter calls.
`http2_adaptive_window(true)` is PROHIBITED on all four production factory sites. The client
advertises a deterministic 4 MiB `SETTINGS_INITIAL_WINDOW_SIZE`. A 1.1 MiB Claroty page fits
entirely within the first window with ZERO `WINDOW_UPDATE` round-trips required.

**Implementer MUST NOT add `http2_adaptive_window(true)` to any factory site.** Any version of
the builder chain containing `http2_adaptive_window(true)` will reset both windows to 65,535 bytes,
reproducing DEFECT-1.

_Sources: Context7 `/websites/rs_reqwest` `http2_adaptive_window`; hyper `proto/h2/client.rs`
`adaptive_window` + `SPEC_WINDOW_SIZE` (docs.rs/hyper source, Perplexity deep-research)._

### Claim #3 — RESOLVED: SETTINGS-frame assertion replaces loopback timing test

The original Red Gate design (local hyper h2 server, serve 2 MiB, assert tuned client < 5s vs
default stalls) does NOT gate on loopback. Loopback RTT is microseconds; a well-behaved hyper
server delivers 2 MiB over a 64 KiB window in milliseconds (~32 `WINDOW_UPDATE` round-trips).
The production DEFECT-1 stall is a **peer flow-control deadlock**, not generic window-limited
slowness. A compliant hyper server does not reproduce it. As designed, RG-001..RG-004 would pass
BOTH before and after the fix on loopback, so they gate nothing.

**Resolution:** Use a deterministic SETTINGS-frame assertion instead:
- Bind a plain-TCP listener on localhost.
- On the server side, use the `h2` crate: `h2::server::handshake(io).await`, then
  `connection.remote_settings().initial_window_size()` to read the value the client advertised
  in its initial `SETTINGS` frame.
- Assert `initial_window_size() == Some(4_194_304)`.
- FAILS before the fix (client advertises default 65,535). PASSES after adding both window
  setter calls to the factory (client advertises 4,194,304).

This is a deterministic gate that tests the EXACT SETTINGS the production client builder
advertises, independent of network timing.

The `h2` crate (0.4.18) is already a transitive dependency via reqwest's Cargo.lock. Add
`h2 = "0.4"` to `[dev-dependencies]` of both `prism-bin/Cargo.toml` and
`prism-spec-engine/Cargo.toml` to make it available in test crates.

_Sources: Context7/docs.rs reqwest ClientBuilder; h2 docs `server::handshake`, `Connection::remote_settings`,
`Settings::initial_window_size` (docs.rs/h2); Perplexity deep-research._

---

## Narrative

As a sensor pipeline executing against a Claroty xDome endpoint that returns ~1.1 MiB per page,
I want all production reqwest client builders to configure a 4 MiB initial h2 flow-control window,
so that MB-scale API pages are received without triggering h2 flow-control stalls that cause
E-QUERY-004 timeout errors under direct HTTPS.

## Background

Live monroe validation of S-CLAROTY-VULNS-001 revealed that `claroty_vulnerabilities`
(~1.1 MiB/page) hangs with zero bytes received over 30 seconds under direct h2 HTTPS
(DEFECT-1, ADR-059 §Context). The HTTP/2 RFC 7540 default initial window of 65,535 bytes
(~64 KiB) requires 17+ `WINDOW_UPDATE` round-trips to receive a 1.1 MiB page. A latent
flow-control issue in h2 0.4.18 causes the server to stall waiting for `WINDOW_UPDATE`
frames, resulting in the 30s timeout. Identical requests via a Python urllib HTTP/1.1
relay (no h2 flow control) succeed.

The fix is surgical: add EXACTLY TWO `reqwest::ClientBuilder` calls to ALL FOUR production
outbound sensor/plugin client factories. With a 4 MiB initial window, the entire 1.1 MiB page
fits within the first window and NO `WINDOW_UPDATE` frames are needed before all bytes arrive.
DTU test clients (wiremock/plain-HTTP) are explicitly excluded — h2 ALPN negotiation does not
apply over plain HTTP, and DTU client construction MUST NOT be altered.

**PROHIBITION: `http2_adaptive_window(true)` MUST NOT be used.** Enabling adaptive window
overrides `http2_initial_stream_window_size` and `http2_initial_connection_window_size`, resetting
both to the RFC default 65,535 bytes and negating the fix. See §Uncertainty Resolution Claim #2.

**The four production factory sites (by function name, per ADR-059 §D7 and BC-2.16.002 H2
postcondition):**
1. `build_http_client_with_custom_timeout` in `crates/prism-bin/src/spec_driven_adapter.rs`
   (canonical factory; `build_http_client_with_timeout` delegates here; `DeclarativeHttpAuthProvider`
   inherits via BC-2.16.014)
2. `plugin_load_step_with_audit` normal-branch `reqwest::Client::builder()` site in
   `crates/prism-bin/src/boot.rs` (PluginRuntime client)
3. `plugin_load_step_with_audit` PRISM_DISABLE_PLUGIN_LOAD-branch `reqwest::Client::builder()`
   site in `crates/prism-bin/src/boot.rs` (same function, disabled-plugin-load early-return path)
4. `build_http_client_with_timeout` in `crates/prism-spec-engine/src/pipeline.rs`
   (infusion `HttpLookupSource` outbound factory; sibling of site 1 per OBS-4 note)

**Two builder calls to add to EACH site:**
```rust
.http2_initial_stream_window_size(4 * 1024 * 1024)     // 4 MiB stream window — ADR-059 Option A
.http2_initial_connection_window_size(4 * 1024 * 1024) // 4 MiB connection window — ADR-059 Option A
// NOTE: http2_adaptive_window(true) MUST NOT be added — it overrides both setters to 65,535
```

Add these calls between `.user_agent(...)` / `.timeout(...)` and `.build()` at each site.
The ordering within the builder chain does not affect correctness (neither call depends on the
other).

**Story-level holdout gate:** After LOCAL 3-CLEAN adversary convergence and BEFORE demo
recording / push to origin, the holdout-evaluator runs hidden SINGLE-USE scenarios. The
gate is BLOCKING — unsatisfied scenarios reset the LOCAL streak per BC-5.39.001.

## Behavioral Contracts

| BC | Title | Version | Role |
|----|-------|---------|------|
| BC-2.16.002 | Multi-Step Fetch Pipeline Execution — Sequential Steps with Variable Interpolation | v2.36 | §Postconditions "H2 Flow-Control Window Sizing (ADR-059 §D7)": four production factory sites MUST configure 4 MiB initial stream/connection windows (Option A — no adaptive window); DTU clients excluded |

## Acceptance Criteria

### AC-001: `build_http_client_with_custom_timeout` configures 4 MiB h2 windows; client advertises `SETTINGS_INITIAL_WINDOW_SIZE == 4,194,304` (traces to BC-2.16.002 postcondition — H2 Flow-Control Window Sizing §D7, factory site 1)

`build_http_client_with_custom_timeout` in `crates/prism-bin/src/spec_driven_adapter.rs`
includes both h2 window builder calls.
Behavioral proof: a client constructed using this factory's builder chain advertises
`SETTINGS_INITIAL_WINDOW_SIZE = 4,194,304` in its h2 `SETTINGS` frame, as observed by an
`h2` crate server via `connection.remote_settings().initial_window_size() == Some(4_194_304)`.

**Test:** `test_BC_2_16_002_h2_stream_window_settings_spec_driven_adapter`

### AC-002: `plugin_load_step_with_audit` normal-branch client builder configures 4 MiB h2 windows; client advertises `SETTINGS_INITIAL_WINDOW_SIZE == 4,194,304` (traces to BC-2.16.002 postcondition — H2 Flow-Control Window Sizing §D7, factory site 2)

The `reqwest::Client::builder()` chain in `plugin_load_step_with_audit` normal-branch
(post-PRISM_DISABLE_PLUGIN_LOAD early-return) in `crates/prism-bin/src/boot.rs` includes
both h2 window builder calls. Behavioral proof: same `SETTINGS` assertion —
`initial_window_size() == Some(4_194_304)`.

**Test:** `test_BC_2_16_002_h2_stream_window_settings_boot_normal_branch`

### AC-003: `plugin_load_step_with_audit` PRISM_DISABLE_PLUGIN_LOAD-branch client builder configures 4 MiB h2 windows; client advertises `SETTINGS_INITIAL_WINDOW_SIZE == 4,194,304` (traces to BC-2.16.002 postcondition — H2 Flow-Control Window Sizing §D7, factory site 3)

The `reqwest::Client::builder()` chain in the `PRISM_DISABLE_PLUGIN_LOAD == "1"` early-return
branch of `plugin_load_step_with_audit` in `crates/prism-bin/src/boot.rs` includes both h2
window builder calls. Behavioral proof: same `SETTINGS` assertion —
`initial_window_size() == Some(4_194_304)`.

**Test:** `test_BC_2_16_002_h2_stream_window_settings_boot_disable_plugin_branch`

### AC-004: `build_http_client_with_timeout` in prism-spec-engine pipeline.rs configures 4 MiB h2 windows; client advertises `SETTINGS_INITIAL_WINDOW_SIZE == 4,194,304` (traces to BC-2.16.002 postcondition — H2 Flow-Control Window Sizing §D7, factory site 4)

`build_http_client_with_timeout` in `crates/prism-spec-engine/src/pipeline.rs` includes
both h2 window builder calls. Behavioral proof: same `SETTINGS` assertion —
`initial_window_size() == Some(4_194_304)`.

**Test:** `test_BC_2_16_002_h2_stream_window_settings_spec_engine_pipeline`

### AC-005: DTU test clients (wiremock/plain-HTTP) are NOT modified; existing wiremock-based tests continue to pass (traces to BC-2.16.002 postcondition — H2 Flow-Control Window Sizing §D7 DTU exclusion clause)

No `.http2_initial_stream_window_size` or `.http2_initial_connection_window_size` calls are
added to test-only or DTU client builders (wiremock plain-HTTP clients in
`crates/prism-dtu-claroty/`, `crates/prism-dtu-crowdstrike/`, etc.).
All existing wiremock-based integration tests continue to pass without modification.
This is verified by `just check` passing with no wiremock-test regressions.

**Test:** `test_BC_2_16_002_h2_window_existing_wiremock_tests_unmodified` (sentinel: compile
and run one representative wiremock integration test to confirm plain-HTTP path unchanged)

## Red Gate Tests

The SETTINGS-frame assertion design: the test creates a plain-TCP listener; builds a reqwest
client using the production factory's builder chain plus `.http2_prior_knowledge()` (forces h2
over plain TCP without TLS/ALPN); the server uses the `h2` crate to handshake and read the
client's advertised `SETTINGS_INITIAL_WINDOW_SIZE`. FAILS before fix (advertises 65,535 bytes —
the RFC default). PASSES after fix (advertises 4,194,304 bytes).

| ID | Test name | Test type | What it gates |
|----|-----------|-----------|---------------|
| RG-001 | `test_BC_2_16_002_h2_stream_window_settings_spec_driven_adapter` | Integration — prism-bin tests; h2 crate SETTINGS assertion on plain TCP via `http2_prior_knowledge` | AC-001: `build_http_client_with_custom_timeout` factory chain advertises `SETTINGS_INITIAL_WINDOW_SIZE == Some(4_194_304)`. Fails before fix (65,535 default), passes after adding both window calls. |
| RG-002 | `test_BC_2_16_002_h2_stream_window_settings_spec_engine_pipeline` | Integration — prism-spec-engine tests; same h2 SETTINGS harness | AC-004: `build_http_client_with_timeout` (pipeline.rs) factory chain advertises `SETTINGS_INITIAL_WINDOW_SIZE == Some(4_194_304)`. Fails before fix, passes after. |
| RG-003 | `test_BC_2_16_002_h2_stream_window_settings_boot_normal_branch` | Integration — prism-bin tests; mirrors `plugin_load_step_with_audit` normal-branch builder chain + h2 SETTINGS assertion | AC-002: PluginRuntime normal-branch builder advertises `SETTINGS_INITIAL_WINDOW_SIZE == Some(4_194_304)`. Fails before fix, passes after. |
| RG-004 | `test_BC_2_16_002_h2_stream_window_settings_boot_disable_plugin_branch` | Integration — prism-bin tests; mirrors `plugin_load_step_with_audit` PRISM_DISABLE_PLUGIN_LOAD-branch builder chain + h2 SETTINGS assertion | AC-003: PluginRuntime DISABLE-branch builder advertises `SETTINGS_INITIAL_WINDOW_SIZE == Some(4_194_304)`. Fails before fix, passes after. |
| RG-005 | `test_BC_2_16_002_h2_window_existing_wiremock_tests_unmodified` | Integration — prism-bin, representative wiremock test (plain HTTP); verifies DTU exclusion | AC-005: existing plain-HTTP wiremock integration test compiles and passes; no h2 window calls added to test-only clients. Passes both before and after fix — regression sentinel, NOT a Red Gate. |

**BC-5.38.001 density check:** 4 Red Gate tests (RG-001..RG-004) / 5 acceptance criteria = 0.8 ≥ 0.5 threshold. PASS.
(RG-005 is a regression sentinel that passes before and after; it does not count as a Red Gate test.)

**Test crate dev-dependency additions required:**
- `crates/prism-bin/Cargo.toml` `[dev-dependencies]`: add `h2 = "0.4"` (for RG-001, RG-003, RG-004 in `tests/bc_2_16_002_h2_window_sizing.rs`)
- `crates/prism-spec-engine/Cargo.toml` `[dev-dependencies]`: add `h2 = "0.4"` (for RG-002 in `tests/bc_2_16_002_h2_window.rs`)

h2 0.4.18 is already a transitive Cargo.lock dependency via reqwest; adding it to `[dev-dependencies]` exposes the crate in test builds without pulling a new version.

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| `build_http_client_with_custom_timeout` | `crates/prism-bin/src/spec_driven_adapter.rs` | Pure (builder construction; no I/O) |
| `build_http_client_with_timeout` (prism-bin wrapper) | `crates/prism-bin/src/spec_driven_adapter.rs` | Pure (delegates to `build_http_client_with_custom_timeout`) |
| `plugin_load_step_with_audit` normal-branch builder | `crates/prism-bin/src/boot.rs` | Pure (builder chain; result used in effectful boot step) |
| `plugin_load_step_with_audit` DISABLE-branch builder | `crates/prism-bin/src/boot.rs` | Pure (builder chain; result used in effectful boot step) |
| `build_http_client_with_timeout` (pipeline.rs) | `crates/prism-spec-engine/src/pipeline.rs` | Pure (builder construction; no I/O) |
| h2 SETTINGS test server harness (new, test-only) | `crates/prism-bin/tests/bc_2_16_002_h2_window_sizing.rs` | Effectful (TCP listener + h2 server handshake; test-only) |
| h2 SETTINGS test server harness (new, test-only) | `crates/prism-spec-engine/tests/bc_2_16_002_h2_window.rs` | Effectful (TCP listener + h2 server handshake; test-only) |

Architecture section references:
- `architecture/module-decomposition.md` §SS-01 Sensor Adapters (prism-bin; spec_driven_adapter, boot)
- `architecture/module-decomposition.md` §SS-16 Spec Engine (prism-spec-engine; pipeline)
- ADR-059 §D7 — H2 window sizing decision, four factory sites, Option A, exclusion scope
- ADR-050 §D1/D2 (rustls-tls mandatory), §D5 (http2 feature), §D6 (User-Agent, factory enumeration)

---

## Purity Classification

| Element | Classification | Rationale |
|---------|---------------|-----------|
| The two `reqwest::ClientBuilder` window calls added to all four production factory sites | **Pure** | Builder-chain configuration only; constructs a `Client` value with no I/O side effects at call time. |
| `build_http_client_with_custom_timeout` / `build_http_client_with_timeout` (both crates) | **Pure** | Client construction; no network I/O until the returned client is used. |
| `plugin_load_step_with_audit` builder chains (both branches) | **Pure (builder), Effectful (enclosing boot step)** | The builder chain is pure; it is consumed inside an effectful boot step whose error semantics are unchanged (EC-004). |
| h2 SETTINGS test server + TCP listener (new, test-only) | **Effectful** | Binds a real `TcpListener` and drives h2 handshake I/O; confined to the test harness. |

No pure-core / effectful-I/O boundary is crossed by this story: all production changes are pure
builder configuration on the existing effectful outbound HTTP path. The only new effectful code
is test-only (the h2 SETTINGS harness).

---

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Sensor API uses HTTP/1.1 (no h2 negotiated) | h2 window settings silently ignored; HTTP/1.1 fallback proceeds normally; no regression |
| EC-002 | Sensor API uses h2 with small pages (< 64 KiB, e.g., claroty_alerts ~5 KB) | 4 MiB initial window means fewer `WINDOW_UPDATE` frames; pure performance improvement; no behavioral change |
| EC-003 | DTU wiremock server (plain HTTP over TCP) | DTU test clients NOT modified; h2 window settings do not apply to plain HTTP; no change to test plumbing |
| EC-004 | `reqwest::Client::builder()` returns `Err` during boot | The two new calls do not change error semantics; if the builder fails, the same `BootError::InternalError` is returned as before (EC-D-009 handling unchanged) |
| EC-005 | Inadvertent addition of `http2_adaptive_window(true)` to a factory site | This MUST NOT happen. If present, adaptive mode overrides both window setters to 65,535 bytes — reproducing DEFECT-1. The RG-001..RG-004 SETTINGS assertions will fail (they assert 4,194,304) because adaptive resets to 65,535. This provides a compile-time-visible code smell AND a Red Gate catch. |

## Token Budget Estimate

| Item | Estimated tokens |
|------|-----------------|
| This story spec | ~7,000 |
| ADR-059 (full) | ~3,000 |
| BC-2.16.002 §Postconditions H2 section (targeted read) | ~1,000 |
| ADR-050 §D5/D6 sections (confirmation of builder scope) | ~1,500 |
| `crates/prism-bin/src/spec_driven_adapter.rs` (factory function context) | ~3,000 |
| `crates/prism-bin/src/boot.rs` (`plugin_load_step_with_audit` function, both branches) | ~3,500 |
| `crates/prism-spec-engine/src/pipeline.rs` (`build_http_client_with_timeout` function context) | ~2,500 |
| h2 SETTINGS test harness (new; h2 crate server pattern) | ~3,000 |
| Existing test files for regression context | ~2,000 |
| BC files (1 BC) | ~1,000 |
| **Total estimate** | **~27,500 tokens** |

Well within 20-30% of a 200K context window.

## Tasks

- [ ] **Task 1 (Red Gate — test first):** Build the h2 SETTINGS test harness helper in
  `crates/prism-bin/tests/bc_2_16_002_h2_window_sizing.rs`. The harness is a reusable async
  helper function:
  1. Bind a `TcpListener` on `127.0.0.1:0` (ephemeral port).
  2. Accept one connection.
  3. Use `h2::server::handshake(stream).await` to perform the h2 handshake.
  4. Read `connection.remote_settings().initial_window_size()` — this is the value the client
     advertised in its `SETTINGS` frame.
  5. Return the window size to the calling test for assertion.
  The harness uses plain TCP (no TLS) and requires the client to connect with
  `http2_prior_knowledge()`. Add `h2 = "0.4"` and `tokio = { version = "1", features = ["full"] }`
  to `crates/prism-bin/Cargo.toml` `[dev-dependencies]`.

- [ ] **Task 2 (Red Gate — test first):** Write RG-001:
  `test_BC_2_16_002_h2_stream_window_settings_spec_driven_adapter`
  in `crates/prism-bin/tests/bc_2_16_002_h2_window_sizing.rs`.
  Mirror the `build_http_client_with_custom_timeout` builder chain (same `.user_agent(...)`,
  `.timeout(...)` calls) PLUS `.http2_prior_knowledge()` for plain-TCP h2. Use this client
  against the SETTINGS harness server. Assert `window_size == Some(4_194_304)`. MUST FAIL
  before Task 6 (factory builder chain lacks the two window calls; advertises default 65,535).

- [ ] **Task 3 (Red Gate — test first):** Write RG-003 and RG-004:
  `test_BC_2_16_002_h2_stream_window_settings_boot_normal_branch` and
  `test_BC_2_16_002_h2_stream_window_settings_boot_disable_plugin_branch`
  in the same test file. Mirror the respective `plugin_load_step_with_audit` builder chains
  from `boot.rs` (each branch independently) plus `.http2_prior_knowledge()`. Assert
  `window_size == Some(4_194_304)`. Both MUST FAIL before Task 7.

- [ ] **Task 4 (Red Gate — test first):** Write RG-002:
  `test_BC_2_16_002_h2_stream_window_settings_spec_engine_pipeline`
  in `crates/prism-spec-engine/tests/bc_2_16_002_h2_window.rs` (new file).
  Add `h2 = "0.4"` to `crates/prism-spec-engine/Cargo.toml` `[dev-dependencies]`.
  Mirror the `build_http_client_with_timeout` builder chain from `pipeline.rs` plus
  `.http2_prior_knowledge()`. Assert `window_size == Some(4_194_304)`. MUST FAIL before Task 8.

- [ ] **Task 5 (Red Gate — test first):** Write RG-005:
  `test_BC_2_16_002_h2_window_existing_wiremock_tests_unmodified`. This is a regression
  sentinel (NOT a Red Gate — it passes before and after the fix). Call or reference a
  representative wiremock-based integration test. The purpose is to confirm that no DTU/wiremock
  client was modified by this story. Mark it clearly as a regression sentinel, not a Red Gate.

- [ ] **Task 6 (Implementation — prism-bin spec_driven_adapter.rs):** Add the two h2 window
  builder calls to `build_http_client_with_custom_timeout` in
  `crates/prism-bin/src/spec_driven_adapter.rs`. Insert between `.timeout(timeout)` and
  `.build()`:
  ```rust
  .http2_initial_stream_window_size(4 * 1024 * 1024)     // ADR-059 §D7 Option A: 4 MiB stream window
  .http2_initial_connection_window_size(4 * 1024 * 1024) // ADR-059 §D7 Option A: 4 MiB connection window
  // NOTE: http2_adaptive_window(true) MUST NOT be added — overrides both to 65,535
  ```
  Also update the RG-001 test mirror to include the same two calls (so the test now reflects the
  fixed factory). Run `just iter prism-bin` — RG-001 MUST turn GREEN.

- [ ] **Task 7 (Implementation — prism-bin boot.rs):** Add the two h2 window builder calls to
  BOTH `reqwest::Client::builder()` sites inside `plugin_load_step_with_audit` in
  `crates/prism-bin/src/boot.rs`:
  - Site 1: the PRISM_DISABLE_PLUGIN_LOAD early-return branch builder
  - Site 2: the normal-branch builder
  Same two calls, same comment (cite ADR-059 §D7 Option A). Also update the RG-003 and RG-004
  test mirrors to include the same two calls. Run `just iter prism-bin` — RG-003 and RG-004
  MUST turn GREEN. Confirm RG-001 still GREEN (no regression from boot.rs change).

- [ ] **Task 8 (Implementation — prism-spec-engine pipeline.rs):** Add the two h2 window
  builder calls to `build_http_client_with_timeout` in `crates/prism-spec-engine/src/pipeline.rs`.
  Same two calls, same comment (cite ADR-059 §D7 Option A). Also update the RG-002 test mirror
  to include the same two calls. Run `just iter prism-spec-engine` — RG-002 MUST turn GREEN.

- [ ] **Task 9 (SAP-1 self-check):** Confirm that no new `tracing::*!(event_type = ...)` emissions
  are added by this story (pure builder-chain change; no event catalog rows affected). If any
  new emission is inadvertently added, register it in BC-2.16.002 §Postconditions Canonical
  Structured Event Catalog (PG-LP11-001). BC-2.16.002 v2.36 SAP-1 declaration states:
  "ADR-059 introduces NO new `event_type` values; catalog count unchanged at 96."

- [ ] **Task 10 (TD-VSDD-060 sibling-sweep):** `build_http_client_with_custom_timeout` and
  `build_http_client_with_timeout` (prism-bin) are a sibling pair — the wrapper delegates to
  the implementation. The two h2 calls are added ONLY to `build_http_client_with_custom_timeout`
  (the implementation); the wrapper inherits them automatically. Confirm this is correct and
  that no other reqwest builder sites in prism-bin or prism-spec-engine were overlooked. Run:
  `rg 'reqwest::Client::builder' crates/prism-bin crates/prism-spec-engine --type rust`
  and verify all production-outbound sites (NOT test-only) have the two h2 window calls after
  this task. Confirm NO site has `http2_adaptive_window` (forbidden).

- [ ] **Task 11 (Final gate):** Run `just check` (full workspace). Confirm all RG tests pass:
  RG-001, RG-002, RG-003, RG-004 (SETTINGS assertions) and RG-005 (regression sentinel).
  Confirm no new `unwrap()`/`expect()` on `Result` in production code paths. Confirm no
  `reqwest` test-client or DTU-client builders were modified (SAP-2 compliance: DTU clone
  routes are read-only for this story). Confirm `http2_adaptive_window` does NOT appear in
  any of the four production factory sites. After `just check` passes, hold for story-level
  holdout gate before pushing to origin.

## Previous Story Intelligence

1. **ADR-050 §D5 (DEFECT-ADAPTER-TLS-XDOME-LIVE-001):** Established that `http2` reqwest
   feature flag is required in prism-bin, prism-spec-engine, and prism-sensors Cargo.toml.
   That feature is already enabled. The current fix adds runtime builder configuration on top
   of the compile-time feature flag — no Cargo.toml changes required for production code.
   (Dev-dependency `h2 = "0.4"` additions are test-only and do not affect production builds.)

2. **S-ADR058-OCSF-ROUTING-001 (merged PR #242):** Confirmed that direct-HTTPS claroty_alerts
   (~5 KB/page) works fine with the existing h2 feature. The vulnerability (~1.1 MiB/page)
   stall is a NEW defect class — h2 flow-control window exhaustion, not an h2-absence problem.
   The two defects are independent.

3. **OBS-4 test precedent (prism-spec-engine/src/pipeline.rs):** There is already a test
   `test_build_http_client_with_timeout_returns_ok_under_rustls` and a UA test for
   `build_http_client_with_timeout`. The new RG-002 test in `tests/bc_2_16_002_h2_window.rs`
   is a behavioral companion. Add it near the existing OBS-4 tests to maintain test locality.

4. **`test_BC_2_01_013_build_http_client_with_custom_timeout_accepts_duration`:** Existing test
   in prism-bin that already exercises `build_http_client_with_custom_timeout`. The new RG-001
   test exercises the SETTINGS outcome (h2 advertised window size) rather than just
   construction success. Both tests coexist.

## Architecture Compliance Rules

From ADR-059 §D7 (Option A):
- Both h2 window builder calls (`http2_initial_stream_window_size(4 * 1024 * 1024)`,
  `http2_initial_connection_window_size(4 * 1024 * 1024)`) MUST be added to ALL FOUR
  production factory sites. Omitting any site or any call is a policy violation (ADR-059 §D7 is a MUST).
- `http2_adaptive_window(true)` MUST NOT be present at any factory site. Its presence overrides
  both window setters to 65,535 bytes, reproducing DEFECT-1. This is a correctness requirement,
  not a style preference.
- DTU test clients MUST NOT receive these calls. h2 prior_knowledge does not apply to
  plain-HTTP wiremock.

From ADR-050 §D1/D2 (unchanged):
- `default-features = false, features = ["rustls-tls"]` MUST remain on all `reqwest` dep
  entries. Do NOT add `native-tls` features. No Cargo.toml changes are needed for production code.

From ADR-050 §D6 (unchanged):
- `.user_agent(concat!("prism/", env!("CARGO_PKG_VERSION")))` MUST be preserved on all
  four sites. Do NOT remove or reorder existing builder calls.

## Library & Framework Requirements

| Library | Version | Source |
|---------|---------|--------|
| `reqwest` | 0.12.28 (Cargo.lock) | `http2_initial_stream_window_size` and `http2_initial_connection_window_size` are valid builder methods gated by `http2` feature. Window setters take `impl Into<Option<u32>>`; `4 * 1024 * 1024` compiles via u32 inference. PROHIBITION: `http2_adaptive_window(true)` MUST NOT be used — it overrides both window setters to 65,535 (see §Uncertainty Resolution Claim #2; architect chose Option A). Verified via Context7 reqwest source + docs.rs. |
| `h2` | 0.4.18 (transitive via reqwest; add `h2 = "0.4"` to [dev-dependencies] of prism-bin AND prism-spec-engine) | Test-only: `h2::server::handshake(stream).await` performs the h2 server handshake over plain TCP; `connection.remote_settings().initial_window_size()` returns the `Option<u32>` value the client advertised in its initial `SETTINGS` frame. Used in RG-001..RG-004 SETTINGS assertions. |
| `tokio` | workspace Cargo.lock version | Async test runtime for h2 SETTINGS harness tests |

No new production Cargo.toml dependency entries required. The h2 builder methods are already
available via the `http2` feature flag enabled in ADR-050 §D5.

## File Structure Requirements

| Action | File path | Notes |
|--------|-----------|-------|
| MODIFY | `crates/prism-bin/src/spec_driven_adapter.rs` | Add 2 h2 window builder calls to `build_http_client_with_custom_timeout` between `.timeout(timeout)` and `.build()` |
| MODIFY | `crates/prism-bin/src/boot.rs` | Add 2 h2 window builder calls to BOTH `reqwest::Client::builder()` sites in `plugin_load_step_with_audit` |
| MODIFY | `crates/prism-spec-engine/src/pipeline.rs` | Add 2 h2 window builder calls to `build_http_client_with_timeout` between existing `.user_agent(...)` / `.timeout(...)` and `.build()` |
| MODIFY | `crates/prism-bin/Cargo.toml` | Add `h2 = "0.4"` to `[dev-dependencies]` |
| MODIFY | `crates/prism-spec-engine/Cargo.toml` | Add `h2 = "0.4"` to `[dev-dependencies]` |
| CREATE | `crates/prism-bin/tests/bc_2_16_002_h2_window_sizing.rs` | RG-001, RG-003, RG-004, RG-005; h2 SETTINGS harness using `h2::server::handshake` |
| CREATE | `crates/prism-spec-engine/tests/bc_2_16_002_h2_window.rs` | RG-002 SETTINGS assertion test for `build_http_client_with_timeout` |

Files that MUST NOT be modified:
- `crates/prism-dtu-claroty/` — DTU exclusion; no production or test changes
- `crates/prism-dtu-crowdstrike/` — DTU exclusion
- Any other `crates/prism-dtu-*/` — DTU exclusion

## Forbidden Dependencies

`prism-spec-engine` MUST NOT gain any new dependency on `prism-bin` (direction is
prism-bin → prism-spec-engine, not the reverse). The RG-002 test lives in prism-spec-engine's
own `tests/` directory and calls only `build_http_client_with_timeout` directly from within
its own crate.

---

## References

- BC-2.16.002 v2.36 §Postconditions "H2 Flow-Control Window Sizing (ADR-059 §D7)" — governing postcondition
- ADR-059 §D7 — Decision: 4 MiB initial stream/connection windows (Option A); four factory sites; DTU exclusion
- ADR-059 §Rationale — Confirmation experiment specification (SETTINGS-frame assertion via h2 crate)
- ADR-050 §D5/D6 — http2 feature enabled; factory sites enumerated; rustls-tls / User-Agent unchanged
- `crates/prism-bin/src/spec_driven_adapter.rs §build_http_client_with_custom_timeout` — factory site 1
- `crates/prism-bin/src/boot.rs §plugin_load_step_with_audit` — factory sites 2+3
- `crates/prism-spec-engine/src/pipeline.rs §build_http_client_with_timeout` — factory site 4
- S-CLAROTY-VULNS-001 — blocked story that this fix unblocks

---

## Changelog

| Version | Date | Author | Notes |
|---------|------|--------|-------|
| 1.1 | 2026-08-26 | story-writer | Architect correction: Option A adopted (two fixed window calls only, http2_adaptive_window PROHIBITED). Red Gate redesigned from loopback timing to SETTINGS-frame assertion via h2 crate. RG test names updated to `_settings_` pattern. 4 RG tests (density 0.8) + 1 regression sentinel. h2 dev-dep added to prism-bin and prism-spec-engine. TD-VSDD-091 compliance: removed volatile line-number reference from boot.rs task. Risk downgraded from MEDIUM to LOW. |
| 1.0 | 2026-08-26 | story-writer | Initial authoring — ADR-059 §D7 implementation story. 5 ACs, 5 RGTs, density 1.0. SAC-1 compliant (enumerated RG list + density check + red-then-green task ordering). SAC-2 N/A (ADR-059 is authored by architect; `anchor_stories` already lists this story ID). SAP-1 no-new-catalog-row declared. |
