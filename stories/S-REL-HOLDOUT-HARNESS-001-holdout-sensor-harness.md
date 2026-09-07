---
document_type: story
story_id: S-REL-HOLDOUT-HARNESS-001
title: "devops/test: holdout sensor harness — registered claroty adapter + local echo (:19089) producing ≥1 row so enrich stage fires and Surface-B UA is observable end-to-end"
wave: POST-beta.1
epic_id: E-REL-IDENTITY
priority: P2
status: draft
version: "0.1"
level: "L4"
producer: product-owner
timestamp: "2026-09-06T00:00:00Z"
tdd_mode: facade
# tdd_mode: facade — holdout test-infrastructure story; no production Rust changes.
# No Red Gate tests (this story ENABLES the holdout evaluator to observe Surface-B UA end-to-end).
subsystems: [SS-22]
# Subsystem anchor justification:
#   SS-22 (Process Lifecycle) owns the boot sequence that registers sensor adapters.
#   This story wires a locally-served claroty DTU echo into prism's boot registration
#   path so the prism-spec-engine enrichment stage fires during holdout evaluation.
crates_touched: []
target_module: devops, test
capabilities: []
behavioral_contracts: []
# BC status: N/A — test infrastructure story; no subsystem behavioral contract.
# behavioral_contracts: [] — POL-14 NO-OP.
verification_properties: []
depends_on: [S-REL-AGENT-VERSION-001]
blocks: []
# Dependency anchor justification:
#   depends_on S-REL-AGENT-VERSION-001: this story exists to unblock HS-032-002/003
#     which test Surface-B UA wired by S-REL-AGENT-VERSION-001. The wire must be live
#     before the harness is meaningful to build and evaluate.
# DEFERRAL ANCHOR (Canonical Principle Rule 3):
#   Deferred post-beta.1 per human-adjudicated ACCEPT-ON-SUBSTANCE 2026-09-06 for
#   HS-032-002 (HARNESS-BLOCKED) and HS-032-003 (HARNESS-BLOCKED):
#     - Concrete future dependency: sensor adapter registration infrastructure needed
#       before Surface-B UA is observable via holdout harness
#     - Specific future story: S-REL-HOLDOUT-HARNESS-001 (this story)
#     - HS-032-002/003 remain SINGLE-USE, NOT CONSUMED; deferred, not reusable in
#       current form; re-evaluation requires this story to land first
points: 3
estimated_days: 0.5
risk: LOW
acceptance_criteria_count: 5
# Acceptance criteria: 5 provisional ACs listed in §Acceptance Criteria;
# to be ratified by story-writer before status: ready.
red_gate_tests: 0
# red_gate_tests: 0 — facade mode. No Rust code.
estimated_passes: 1
holdout_scenarios: []
assumption_validations: []
risk_mitigations:
  - "HS-032-002/003 are SINGLE-USE and NOT CONSUMED as of 2026-09-06. They must NOT
    be re-run before this story delivers the harness. Re-evaluation is gated on this
    story reaching status: merged."
  - "Harness must be a dedicated holdout-harness script / Docker Compose / test helper
    that the holdout-evaluator can reproduce independently. It must NOT read
    HS-032-002/003 scenario bodies (contamination control). The evaluator is provided
    ONLY the scenario files after this harness is live."
inputs:
  - ".factory/specs/architecture/decisions/ADR-064-pre-release-binary-version-identity.md"
  - ".factory/specs/architecture/decisions/ADR-050-workspace-reqwest-tls-backend.md"
  - ".factory/stories/S-REL-AGENT-VERSION-001-agent-version-surfaces.md"
  - ".factory/holdout-scenarios/S-REL-AGENT-VERSION-001-HS-032-002-spec-engine-user-agent-wire-header.md"
  - ".factory/holdout-scenarios/S-REL-AGENT-VERSION-001-HS-032-003-coherent-product-version-both-surfaces.md"
input-hash: "d9c89a8"
traces_to: []
cycle: "v1.0.0-surface-b-holdout-harness"
phase: "3"
---

# S-REL-HOLDOUT-HARNESS-001 — Holdout Sensor Harness for Surface-B UA Observable Evaluation

**Story ID:** S-REL-HOLDOUT-HARNESS-001
**Status:** draft
**Version:** v0.1
**Wave:** POST-beta.1
**Priority:** P2
**Points:** 3
**Beta.1-blocking:** NO — POST-beta.1. Do not implement before beta.1 ships.

---

## Stub Notice

This story is a STUB registered as the deferral anchor for HS-032-002 and HS-032-003
(human-adjudicated ACCEPT-ON-SUBSTANCE 2026-09-06). Full detail is to be authored by
story-writer before status moves to `ready`.

---

## Background

HS-032-002 and HS-032-003 (S-REL-AGENT-VERSION-001 holdout gate, 2026-09-06) evaluated as
**HARNESS-BLOCKED** under human-adjudicated ACCEPT-ON-SUBSTANCE disposition. Both scenarios
test Surface B of the ADR-064 §D4 agent-facing version migration: the `prism-spec-engine`
outbound HTTP `User-Agent` header emitted by `pipeline::build_http_client_with_timeout`.

Root cause of blockage: `prism start` booted with 0 sensor adapters registered. The claroty
DTU local echo server was not wired into the prism config used by the holdout harness. As a
result, `SELECT device_uid FROM claroty_devices LIMIT 1` returned 0 rows, the `| enrich`
stage never executed, `build_http_client_with_timeout` was never called, and the enrichment
echo server (port 19090) never received any request — making the Surface-B UA unobservable.

Surface-B behavioral substance is independently guaranteed by:
- Inline wiremock test `test_infusion_http_client_sends_prism_user_agent` (passes in LOCAL 3-CLEAN)
- AC-007 (`concat!("prism/", env!("PRISM_VERSION"))`) verified via source grep
- Every LOCAL adversary pass (no UA-correctness finding)

---

## Narrative

As a holdout evaluator verifying Surface-B (prism-spec-engine outbound HTTP User-Agent),
I want a reproducible harness that starts prism with a registered claroty adapter and
an enrichment echo server, so that I can observe the `User-Agent: prism/<VERSION>` header
on the wire and evaluate HS-032-002 and HS-032-003 to completion.

---

## Origin

Human-adjudicated ACCEPT-ON-SUBSTANCE 2026-09-06 for HS-032-002 (HARNESS-BLOCKED) and
HS-032-003 (HARNESS-BLOCKED) per Canonical Principle Rule 3. Registered as the concrete
future story anchor required for the deferral to be valid.

---

## Authority

- ADR-064 §D4 (Surface B: `prism-spec-engine` user-agent)
- ADR-050 §D6 (`build_http_client_with_timeout` MUST use `concat!("prism/", env!("PRISM_VERSION"))`)
- HS-032-002 HARNESS-BLOCKED disposition (2026-09-06)
- HS-032-003 HARNESS-BLOCKED disposition (2026-09-06)
- Canonical Principle Rule 3: this story is the deferral anchor for HS-032-002/003

---

## Behavioral Contracts

This story has no subsystem behavioral contracts. Authority is ADR-064 §D4 and ADR-050 §D6.

| Architecture Source | Clause |
|---------------------|--------|
| ADR-064 §D4 | Surface B = prism-spec-engine outbound HTTP User-Agent |
| ADR-050 §D6 | `build_http_client_with_timeout` MUST use `concat!("prism/", env!("PRISM_VERSION"))` |

---

## Token Budget Estimate

| Artifact | Estimated Tokens |
|----------|-----------------|
| This story spec | ~2,000 |
| HS-032-002 scenario | ~1,500 |
| HS-032-003 scenario | ~1,200 |
| ADR-064 §D4 section | ~500 |
| ADR-050 §D6 section | ~400 |
| Total | ~5,600 |

Well within the 30% context window budget.

---

## Red Gate Test List (SAC-1)

**tdd_mode: facade — no Rust Red Gate tests.**

This story produces a harness script/setup, not Rust code. Verification is:
- Harness starts without error
- `prism start` logs show ≥1 adapter registered
- MCP query returns ≥1 row from `claroty_devices`
- Enrichment echo server captures a request with correct `User-Agent`

---

## Tasks

TBD — to be authored by story-writer before status: ready.

Minimum expected tasks:

1. **Write local claroty echo server script (port 19089):** Serve ≥1 device row from
   `GET /v1/devices` compatible with the Claroty DTU response shape.

2. **Write enrichment echo server script (port 19090):** Accept HTTP POST, record
   `User-Agent` header, return valid enrichment JSON response.

3. **Write prism config fragment:** Sensor config registering claroty adapter against
   `:19089` + http_lookup infusion TOML pointing to `:19090` (function name `ua_result`).

4. **Write harness launch script:** Starts both echo servers, starts `prism start` with
   harness config using `PRISM_BUILD_VERSION=<target-version>` override.

5. **Write evaluator runbook (NOT scenario text — harness setup only):** Instructions for
   reproducing the harness. Contamination control: must not include HS-032-002/003 scenario
   text.

6. **Verify AC-001 through AC-005.**

---

## Acceptance Criteria

Provisional ACs — to be ratified by story-writer before status: ready.

### AC-001: Local claroty echo server serves ≥1 device row
`GET http://localhost:19089/v1/devices` returns HTTP 200 with ≥1 device in the response body.
(traces to HS-032-002/003 §Setup §Step 4)

### AC-002: prism start registers ≥1 adapter at boot
`prism start` with harness config logs show ≥1 sensor adapter registered (not "0 adapters registered").
(traces to HS-032-002/003 §Setup §Step 6)

### AC-003: MCP query returns ≥1 row from claroty_devices
`SELECT device_uid FROM claroty_devices LIMIT 1` via MCP query tool returns ≥1 row.
(traces to HS-032-002 §BDD §Step 2)

### AC-004: Enrichment echo server receives a request on enrich query
`SELECT device_uid FROM claroty_devices LIMIT 1 | enrich ua_result(device_uid)` causes the
enrichment echo server (port 19090) to receive ≥1 HTTP request.
(traces to HS-032-002 §BDD §Step 3)

### AC-005: Enrichment echo server records User-Agent containing "prism/"
The captured `User-Agent` header from the enrichment echo server contains the prefix `"prism/"`.
(traces to ADR-050 §D6)

---

## Holdout Gate Re-evaluation Sequencing

After this story merges:

1. Holdout-evaluator receives HS-032-002 and HS-032-003 scenario files (first time — SINGLE-USE)
2. Holdout-evaluator stands up harness via the evaluator runbook (harness setup doc only; NOT scenario bodies)
3. Holdout-evaluator runs each scenario
4. PASS → scenarios marked CONSUMED; Surface-B gate closes
5. FAIL → findings routed via VSDD feedback loop as OBSERVED BEHAVIOR ONLY (never scenario text)

---

## Previous Story Intelligence

| Story | Key Decisions | Patterns Established | Gotchas Discovered |
|-------|--------------|---------------------|-------------------|
| S-REL-AGENT-VERSION-001 | Surface-B wired via per-crate build.rs + pipeline.rs UA | `build_http_client_with_timeout` owns UA emission | 0 adapters at boot blocks enrichment stage entirely |
| S-REL-BVERSION-INJECT-001 | PRISM_BUILD_VERSION override enables version discrimination | Use `PRISM_BUILD_VERSION=<target>` in harness | Harness must use override-build model matching HS-032-001 PASS |

---

## Architecture Compliance Rules

| Rule | Source | Enforcement |
|------|--------|-------------|
| Harness must NOT read HS-032-002/003 scenario bodies | Contamination control policy (CLAUDE.md holdout gate) | AC-005: evaluator runbook is harness setup only |
| HS-032-002/003 are SINGLE-USE; not re-runnable before harness lands | append_only_numbering + holdout gate policy | HS-032-002/003 status: NOT CONSUMED; blocked until this story merges |
| Surface-B observation requires enrichment stage to fire | ADR-064 §D4 §Surface B | AC-003 (query returns row) + AC-004 (enrichment request received) |

---

## Library & Framework Requirements

| Dependency | Version | Notes |
|------------|---------|-------|
| echo server | any | Minimal HTTP server; Python `http.server`, netcat, or similar |
| prism binary | current develop HEAD | Must have S-REL-AGENT-VERSION-001 merged |

---

## File Structure Requirements

| File | Action | Notes |
|------|--------|-------|
| `scripts/holdout-harness/claroty-echo.py` (or `.sh`) | Create | Local claroty echo server for harness |
| `scripts/holdout-harness/enrich-echo.py` (or `.sh`) | Create | Enrichment echo server recording UA |
| `scripts/holdout-harness/prism-harness.toml` | Create | Sensor config registering claroty against :19089 + http_lookup to :19090 |
| `scripts/holdout-harness/README.md` | Create | Evaluator runbook (harness setup only; no scenario text) |

---

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| Claroty echo script (port 19089) | scripts/holdout-harness | Effectful (listens on socket; serves HTTP) |
| Enrichment echo script (port 19090) | scripts/holdout-harness | Effectful (listens on socket; records headers) |
| prism harness config | scripts/holdout-harness | Pure (TOML config; no runtime behavior) |

---

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| Echo server scripts | effectful-shell | Network I/O; side-effectful by design |
| prism harness config (TOML) | pure-core | Static configuration; no runtime behavior |

---

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Claroty echo server port 19089 already in use | Harness script detects and fails fast with clear error |
| EC-002 | Enrichment echo server port 19090 already in use | Harness script detects and fails fast with clear error |
| EC-003 | prism binary not built with S-REL-AGENT-VERSION-001 merged | Harness setup doc must specify minimum: `cargo build --release` on develop HEAD after S-REL-AGENT-VERSION-001 merges |
| EC-004 | PRISM_BUILD_VERSION not set | Without override, UA = "prism/0.1.0" (CARGO_PKG_VERSION fallback); harness must set override to the target version for AC-005 to be discriminating |
| EC-005 | claroty_devices returns 0 rows despite adapter registered | DTU echo response shape mismatch; verify GET /v1/devices response structure matches claroty DTU contract |

---

## Changelog

| Version | Date | Author | Change |
|---------|------|--------|--------|
| 0.1 | 2026-09-06 | product-owner | Stub registered as deferral anchor for HS-032-002/003 HARNESS-BLOCKED (human-adjudicated ACCEPT-ON-SUBSTANCE 2026-09-06); 5 provisional ACs; full detail TBD by story-writer before status: ready |
