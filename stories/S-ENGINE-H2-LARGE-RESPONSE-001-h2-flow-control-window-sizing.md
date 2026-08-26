---
document_type: story
story_id: S-ENGINE-H2-LARGE-RESPONSE-001
title: "Claroty xDome large-response transport — direct-h2 confirmation + recurrence guard (canary + timeout diagnostics + query-CLI wiring)"
level: "L3"
wave: xdome-wave-a
epic_id: E-XDOME-EXPANSION
priority: P2
status: draft
# BC status: BC-2.16.002 active — new scope traces to pipeline execution guarantee
# and error envelope postcondition. Status: draft pending remove-uncertainty pass.
producer: story-writer
timestamp: "2026-08-26T00:00:00Z"
version: "1.2"
modified: "2026-08-26"
phase: 3
cycle: v1.0.0-brownfield
inputs:
  - ".factory/specs/behavioral-contracts/BC-2.16.002-multi-step-fetch-pipeline.md"
  - ".factory/specs/architecture/decisions/ADR-050-workspace-reqwest-tls-backend.md"
input-hash: "699c122"
traces_to: "BC-2.16.002"
points: 3
estimated_days: 1
tdd_mode: facade
# tdd_mode: facade — SAC-1 relaxation rationale:
#   AC-001 is documentation-only (no code change); no Red Gate test applies.
#   AC-002 produces a canary smoke test; the live-tenant assertion is #[ignore]'d
#     per SID-1 (blocking-dependency comment required); the in-process path is
#     diagnostic scaffolding, not a behavioral defect fix.
#   AC-003 enriches existing error diagnostic fields (elapsed_ms, bytes_drained) —
#     structural plumbing on existing error types, not a new algorithm.
#   AC-004 wires an existing CLI stub to an existing service — structural wiring,
#     not new algorithm design.
#   Combined scaffold+impl delivery is appropriate per BC-8.30.001 facade criteria.
#   Mutation testing at wave gate replaces Red Gate density check.
subsystems: [SS-01, SS-16]
# Subsystem anchor justifications (ARCH-INDEX Subsystem Registry):
#   SS-01 (Sensor Adapters): owns the direct-transport confirmation and canary test
#     scope because the Claroty spec-driven adapter and its reqwest client factory
#     live in crates/prism-bin/src/spec_driven_adapter.rs (SS-01 per ARCH-INDEX).
#   SS-16 (Spec Engine): owns the timeout diagnostics enrichment scope because the
#     error surface and pipeline execution path live in prism-spec-engine (SS-16
#     per ARCH-INDEX).
target_module: prism-bin
crates_touched: [prism-bin, prism-spec-engine]
capabilities:
  - CAP-029
behavioral_contracts:
  - BC-2.16.002
  # BC-2.16.002 — Multi-Step Fetch Pipeline Execution — Sequential Steps with
  # Variable Interpolation. The h2 window postcondition added in BC-2.16.002 v2.36
  # is superseded by the DEFECT-1 investigation conclusion (transport healthy; no
  # client change adopted). This story now traces to BC-2.16.002 for: (a) the
  # general pipeline execution success guarantee (AC-002 canary), and (b) the error
  # envelope diagnostic surface (AC-003 timeout diagnostics).
verification_properties: []
holdout_scenarios: []
# holdout_scenarios: PO authors 2–4 hidden SINGLE-USE scenarios at remove-uncertainty
# time. Stored under holdout directory; test-writer and implementer MUST NOT read them.
# Story-level holdout gate is BLOCKING before demo/push (human-approved 2026-07-13).
depends_on: []
blocks: []
# blocks: REMOVED — S-CLAROTY-VULNS-001 is no longer blocked by this story.
# The h2-window hypothesis was falsified by the DEFECT-1 live investigation
# (ADR-059 WITHDRAWN v1.2). S-CLAROTY-VULNS-001 ships with direct h2 transport
# as-is under ADR-050.
acceptance_criteria_count: 4
risk: LOW
assumption_validations: []
# assumption_validations: No ASM validations — this story is a follow-up recurrence
# guard, not a hypothesis under test. The DEFECT-1 transport hypothesis was validated
# externally (live wire-evidence investigation, 2026-08-26).
risk_mitigations: []
# risk_mitigations: No R-NNN mitigations from risk register tied to this story.
# Transport risk was resolved by the DEFECT-1 investigation conclusion (direct h2
# healthy; stall was transient and resolved).
---

# S-ENGINE-H2-LARGE-RESPONSE-001: Claroty xDome Large-Response Transport — Direct-H2 Confirmation + Recurrence Guard

## Authority

**DEFECT-1 live investigation conclusion (2026-08-26):** A direct wire-evidence
investigation proved the h2-window hypothesis false. Direct h2 transport to
`api.claroty.com` is healthy end-to-end:
- `curl --http2` against the live endpoint succeeded.
- A faithful `reqwest` 0.12.28 repro fetched the full response without stall.
- A 6-page / ~7 MB fetch completed successfully.
- The real prism binary fetched live CVE rows across 10 pages without stall.

The stall observed at original DEFECT-1 diagnosis was transient and has since
resolved. No client transport change is adopted.

**ADR-059 WITHDRAWN v1.2:** The h2 flow-control window fix proposed in ADR-059
was withdrawn after the live investigation proved direct h2 transport healthy. No
`http2_initial_stream_window_size`, `http2_initial_connection_window_size`, or
`http2_adaptive_window` calls are adopted. Zero h2 window configuration changes
are introduced by this story.

**ADR-050** remains the governing transport ADR: rustls-tls mandatory; `http2`
feature enabled; User-Agent and 30s timeout conventions unchanged. This story
produces no amendments to ADR-050.

---

## Narrative

As a sensor pipeline operator, now that direct h2 transport to Claroty xDome is
confirmed healthy, I want a recurrence guard — a canary smoke test, richer timeout
diagnostics, and a wired `query` CLI — so that any future large-response transport
regression is immediately detected and diagnosable without requiring a standalone
repro script.

## Behavioral Contracts

| BC | Title | Version | Role |
|----|-------|---------|------|
| BC-2.16.002 | Multi-Step Fetch Pipeline Execution — Sequential Steps with Variable Interpolation | current | Pipeline execution success guarantee (AC-002 canary); error envelope diagnostic surface (AC-003 timeout diagnostics) |

## Acceptance Criteria

### AC-001: Direct-transport confirmation documented; interim relay decommissioned (traces to BC-2.16.002 pipeline configuration invariant)

The interim HTTP/1.1 relay in `prism-live-mcp-wrapper.sh` is decommissioned
(already done per DEFECT-1 investigation). Prism uses direct negotiated h2 to
`api.claroty.com` per ADR-050. The investigation conclusion and ADR-059 withdrawal
are captured in this story's §Authority section and changelog. No production code
change is required for this AC.

### AC-002: Large-response canary smoke test asserts full-body success on a ~1 MB+ Claroty vulnerabilities page (traces to BC-2.16.002 postcondition — pipeline execution success guarantee)

A repeatable smoke test in `crates/prism-bin/tests/` (or equivalent integration
test crate):
1. Fetches the Claroty vulnerabilities endpoint returning a ~1 MB+ page.
2. Asserts a non-empty body — no timeout, no zero-byte response.
3. Live-tenant portions are gated `#[ignore]` per SID-1 with comment:
   `// SID-1: live Claroty tenant required; enable when CLAROTY_API_KEY is set`.
4. An in-process variant (DTU-backed or mocked) runs without `#[ignore]` in CI.

The canary's presence ensures a future stall recurrence fails CI on the non-mocked
path before any developer encounters the regression on live tenant.

### AC-003: Timeout errors surface elapsed-to-first-byte and bytes-drained; optional h2 frame timing flag (traces to BC-2.16.002 postcondition — error envelope diagnostic fields)

When an `E-QUERY-004` or applicable `E-SENSOR` timeout fires on a large-response
fetch:
- The error detail includes `elapsed_ms` (elapsed to first byte / headers).
- The error detail includes `bytes_drained` (bytes received before timeout).
- Behind a `PRISM_H2_FRAME_TIMING=1` environment flag, per-frame h2 timing is
  surfaced for targeted debugging.
- All new diagnostic fields are anchored in `prd-supplements/error-taxonomy.md`
  under the relevant error codes (`E-QUERY-004` / applicable `E-SENSOR-*`).

These fields make a recurrence diagnosable from the error log alone.

### AC-004: `query` CLI subcommand wired to QueryEngine in all three binaries; `exit 4` stub replaced (traces to BC-2.16.002 postcondition — pipeline execution reachability)

All three prism binaries (`prism`, `prism-dev`, `prism-demo`) currently stub the
`query` subcommand with `"not yet wired; exit 4"`. After this story:
- The `query` subcommand is wired to the `QueryEngine` (or equivalent pipeline
  entrypoint) in all three binaries.
- The wired subcommand serves as the execution vehicle for the AC-002 in-process
  canary.
- `just check` passes; no regressions in existing query tests.

## Red Gate / Tests (tdd_mode: facade)

**SAC-1 rationale for facade mode:** This story is scaffolding and diagnostic
plumbing across four ACs, not a single-algorithm defect fix. AC-001 is
documentation-only. AC-002 is a canary registration (live path `#[ignore]`'d per
SID-1; in-process path is structural). AC-003 enriches existing error type fields.
AC-004 wires an existing stub to an existing service. Combined scaffold+impl
delivery is appropriate per BC-8.30.001. Mutation testing at wave gate replaces
Red Gate density check. No enumerated RG-001..RG-NNN list is required per
BC-8.30.001 invariant 2 and SAC-1 §3.

**Test expectations by AC:**

| AC | Test type | What it covers |
|----|-----------|---------------|
| AC-001 | None (documentation-only) | Investigation conclusion captured in §Authority + changelog |
| AC-002 | Integration (in-process, DTU-backed) + `#[ignore]`'d live-tenant smoke | Fetches ~1 MB+ page; asserts non-empty body; `#[ignore]` path per SID-1 |
| AC-003 | Unit (prism-spec-engine) | Timeout error structs carry `elapsed_ms` + `bytes_drained`; error-taxonomy entries exist |
| AC-004 | Integration (prism-bin) | `query` CLI invocation exercises QueryEngine pipeline; no panic; existing query tests pass |

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| Canary smoke test | `crates/prism-bin/tests/claroty_large_response_canary.rs` (new) | Effectful (network I/O; `#[ignore]` for live path) |
| Timeout diagnostics fields | `crates/prism-spec-engine/src/pipeline.rs` | Pure (field addition to existing error struct) |
| `query` CLI wiring | `crates/prism-bin/src/main.rs` (and dev/demo equivalents) | Effectful (CLI entrypoint wired to existing effectful QueryEngine) |

Architecture section references:
- `architecture/module-decomposition.md` §SS-01 Sensor Adapters (canary + CLI wiring)
- `architecture/module-decomposition.md` §SS-16 Spec Engine (timeout diagnostics)
- ADR-050 §D1/D2/D5/D6 — governing transport ADR; unchanged by this story

## Purity Classification

| Element | Classification | Rationale |
|---------|---------------|-----------|
| Canary smoke test | Effectful | Network I/O (DTU or real tenant); live path `#[ignore]`'d per SID-1 |
| Timeout diagnostic fields (`elapsed_ms`, `bytes_drained`) | Pure (struct definition) | Field addition to existing error struct; no I/O side effects at definition time |
| `PRISM_H2_FRAME_TIMING` flag read | Effectful (env read) | Environment variable read at runtime; confined to the diagnostic code path |
| `query` CLI wiring | Effectful (enclosing CLI handler) | Wires stub to existing effectful QueryEngine; no purity boundary crossed |

No new pure-core / effectful-I/O boundary is introduced by this story. All new
effectful code is either test-only (canary) or delegates to the existing
effectful QueryEngine and error-reporting paths.

---

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Live Claroty tenant not available during CI | AC-002 live path is `#[ignore]`; CI passes; in-process canary covers regression detection |
| EC-002 | Timeout fires before first byte received | `elapsed_ms` reflects time-to-first-byte timeout; `bytes_drained = 0`; clear diagnostic |
| EC-003 | `query` subcommand called with malformed PrismQL | Existing error handling applies; no regression from wiring change |
| EC-004 | `PRISM_H2_FRAME_TIMING` not set | Default behavior unchanged; per-frame timing is opt-in |

## Token Budget Estimate

| Item | Estimated tokens |
|------|-----------------|
| This story spec | ~3,500 |
| ADR-050 §D1/D2/D5/D6 (confirmation read) | ~1,000 |
| BC-2.16.002 §Postconditions (targeted read) | ~800 |
| `crates/prism-bin/src/main.rs` CLI stub context | ~2,000 |
| `crates/prism-spec-engine/src/pipeline.rs` error type context | ~2,000 |
| `prd-supplements/error-taxonomy.md` E-QUERY-004 / E-SENSOR entries | ~1,500 |
| **Total estimate** | **~10,800 tokens** |

Well within 20-30% of a 200K context window.

## Tasks

- [ ] **Task 1 (Documentation — AC-001):** Verify that `prism-live-mcp-wrapper.sh`
  no longer contains the HTTP/1.1 relay workaround. Confirm direct h2 is in use.
  Record verification in task completion note. No code change required.

- [ ] **Task 2 (Scaffolding + test — AC-002):** Create
  `crates/prism-bin/tests/claroty_large_response_canary.rs`. Write an in-process
  test (DTU-backed or mocked) asserting a ~1 MB+ page fetch returns a non-empty
  body without timeout. Add a `#[ignore]`'d live-tenant companion with the SID-1
  comment citing blocking dependency. Run `just iter prism-bin` to confirm the
  in-process test passes.

- [ ] **Task 3 (Plumbing — AC-003):** Locate the timeout error struct(s) for
  `E-QUERY-004` and applicable `E-SENSOR-*` codes in
  `crates/prism-spec-engine/src/pipeline.rs` and in the error taxonomy. Add
  `elapsed_ms: u64` and `bytes_drained: u64` fields. Update
  `prd-supplements/error-taxonomy.md` entries. Add the opt-in
  `PRISM_H2_FRAME_TIMING` flag for per-frame h2 timing. Write unit tests
  validating the new fields are populated on timeout. Run
  `just iter prism-spec-engine`.

- [ ] **Task 4 (Wiring — AC-004):** Locate the `query` subcommand stubs in each
  of the three binary `main.rs` files. Wire each to the `QueryEngine` (or
  equivalent pipeline entrypoint). The wired subcommand serves as the execution
  vehicle for the AC-002 in-process canary. Run `just iter prism-bin` — all
  existing query tests must pass; the `query` stub exit must be gone.

- [ ] **Task 5 (SAP-1 self-check):** Confirm that any new
  `tracing::*!(event_type = ...)` emissions added by AC-003 diagnostics have
  corresponding rows in BC-2.16.002 §Postconditions Canonical Structured Event
  Catalog per PG-LP11-001. If AC-003 adds no new `event_type` values, record
  "no new catalog rows required."

- [ ] **Task 6 (Final gate):** Run `just check` (full workspace). All tests pass.
  No `unwrap()`/`expect()` on `Result` in new production code paths. Confirm zero
  occurrences of `http2_initial_stream_window_size`,
  `http2_initial_connection_window_size`, or `http2_adaptive_window` in any
  production file touched by this story — no window-mechanism remnants.

## Previous Story Intelligence

1. **S-CLAROTY-VULNS-001 (DEFECT-1 live investigation):** The 30s stall on
   `claroty_vulnerabilities` was investigated with `curl --http2`, a reqwest
   0.12.28 repro, a 6-page / ~7 MB fetch, and the real prism binary across 10
   CVE pages. All succeeded over direct h2. The stall was transient and resolved.
   ADR-059 was withdrawn. Direct transport is healthy under ADR-050.

2. **S-ADR058-OCSF-ROUTING-001 (merged PR #242):** Confirmed that direct-HTTPS
   `claroty_alerts` (~5 KB/page) works fine with the existing h2 feature. The
   vulnerability stall was a transient anomaly, not a structural transport defect.

3. **ADR-050 §D5:** The `http2` reqwest feature is already enabled in prism-bin,
   prism-spec-engine, and prism-sensors. No Cargo.toml changes required for AC-002
   or any other AC in this story.

## Architecture Compliance Rules

From ADR-050 §D1/D2 (unchanged):
- `default-features = false, features = ["rustls-tls"]` MUST remain on all
  `reqwest` dep entries. Do NOT alter TLS configuration.

From ADR-050 §D6 (unchanged):
- `.user_agent(concat!("prism/", env!("CARGO_PKG_VERSION")))` MUST be preserved.
- `.timeout(Duration::from_secs(30))` MUST remain on all production clients.

From DEFECT-1 investigation conclusion (2026-08-26):
- `http2_initial_stream_window_size`, `http2_initial_connection_window_size`, and
  `http2_adaptive_window` MUST NOT be added to any factory site. Their presence
  would contradict the investigation conclusion and reintroduce the withdrawn
  ADR-059 mechanism.

## Library & Framework Requirements

| Library | Version | Notes |
|---------|---------|-------|
| `reqwest` | 0.12.28 (Cargo.lock) | No new builder calls. Existing `http2` feature, `rustls-tls`, User-Agent, and timeout conventions unchanged per ADR-050. |
| `tokio` | workspace version | Existing async runtime for integration and unit tests |

No new production Cargo.toml dependency entries required.

## File Structure Requirements

| Action | File path | Notes |
|--------|-----------|-------|
| CREATE | `crates/prism-bin/tests/claroty_large_response_canary.rs` | AC-002 canary smoke test (in-process + `#[ignore]`'d live path) |
| MODIFY | `crates/prism-spec-engine/src/pipeline.rs` | AC-003 — add `elapsed_ms` / `bytes_drained` to timeout error structs |
| MODIFY | `.factory/specs/prd-supplements/error-taxonomy.md` | AC-003 — add new diagnostic fields to E-QUERY-004 / applicable E-SENSOR-* entries |
| MODIFY | `crates/prism-bin/src/main.rs` (and dev/demo equivalents) | AC-004 — wire `query` CLI subcommand to QueryEngine |

Files that MUST NOT be modified in ways that introduce window-mechanism calls:
- Any existing `reqwest::ClientBuilder` chain — zero h2 window configuration calls added

## Forbidden Dependencies

`prism-spec-engine` MUST NOT gain any new dependency on `prism-bin`. New test
files in `crates/prism-bin/tests/` MUST NOT import from `prism-spec-engine`
internal modules (direction: prism-bin → prism-spec-engine, not the reverse).

---

## References

- ADR-059 WITHDRAWN v1.2 — h2 window fix withdrawn; live investigation proved transport healthy
- ADR-050 §D1/D2/D5/D6 — governing transport ADR; rustls-tls mandatory; h2 feature; User-Agent
- BC-2.16.002 current — Multi-Step Fetch Pipeline Execution (pipeline success guarantee; error envelope)
- `.factory/specs/prd-supplements/error-taxonomy.md` — E-QUERY-004 / E-SENSOR-* definitions
- S-CLAROTY-VULNS-001 — story whose live investigation falsified the h2-window hypothesis

---

## Changelog

| Version | Date | Author | Notes |
|---------|------|--------|-------|
| 1.2 | 2026-08-26 | story-writer | WHOLESALE RE-SCOPE: ADR-059 withdrawn v1.2 after live wire-evidence investigation proved direct h2 transport to api.claroty.com healthy (curl --http2, reqwest 0.12.28 repro, 6-page/~7 MB fetch, real prism binary across 10 CVE pages all succeeded; stall was transient and resolved). Entire h2-window mechanism removed: 4 MiB SETTINGS_INITIAL_WINDOW_SIZE, RG-001..RG-005, prism-bin/prism-spec-engine h2 dev-deps, SETTINGS-frame assertion harness, Option A prohibition on http2_adaptive_window — all deleted. Story re-scoped to: AC-001 direct-transport confirmation (no code change); AC-002 large-response canary smoke test; AC-003 timeout diagnostics enrichment (elapsed_ms, bytes_drained, PRISM_H2_FRAME_TIMING flag); AC-004 query CLI wiring. Priority P0→P2. tdd_mode strict→facade (SAC-1: scaffolding/diagnostic work, not defect-fix TDD; mutation testing at wave gate). blocks:[S-CLAROTY-VULNS-001] removed (no longer blocking). |
| 1.1 | 2026-08-26 | story-writer | Architect correction: Option A adopted (two fixed window calls only, http2_adaptive_window PROHIBITED). Red Gate redesigned from loopback timing to SETTINGS-frame assertion via h2 crate. Risk downgraded to LOW. |
| 1.0 | 2026-08-26 | story-writer | Initial authoring — ADR-059 §D7 implementation story. 5 ACs, 5 RGTs, density 0.8. |
