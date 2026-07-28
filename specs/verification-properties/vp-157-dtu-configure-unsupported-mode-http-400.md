---
document_type: verification-property
level: L4
version: "1.1"
status: active
producer: architect
timestamp: 2026-07-27T00:00:00Z
phase: wave-a
inputs:
  - specs/behavioral-contracts/BC-3.6.001-per-org-failure-injection.md
  - stories/S-3.6.01-hs-006-refresh.md
input-hash: "pending"
traces_to: architecture/verification-architecture.md
source_bc: BC-3.6.001
source_invariant: null
module: prism-dtu-harness
priority: P1
proof_method: unit_test
verification_method: unit_test
feasibility: feasible
verification_lock: false
proof_completed_date: "2026-07-27"
proof_file_hash: null
lifecycle_status: active
introduced: "2026-06-11"
modified: "2026-07-27"
deprecated: null
deprecated_by: null
replacement: null
retired: null
withdrawn: null
withdrawal_reason: null
removed: null
removal_reason: null
---

# VP-157: DTU Configure — Unsupported Failure Mode Returns HTTP 400 with No State Change

## Property Statement

For `prism-dtu-harness`, when `POST /dtu/configure` is called with a `FailureMode`
variant that is NOT listed in the target clone's supported-mode table (BC-3.6.001
Invariant 5), the configure endpoint MUST return **HTTP 400** with a JSON body matching
the shape `{"error": "unsupported_failure_mode", "mode": "<variant-name>"}`, and the
clone's internal failure state MUST remain unchanged.

Specifically:
1. **HTTP status:** The response to the unsupported-mode configure call is HTTP 400 (not
   HTTP 200 — there is no silent ACK).
2. **Error body shape:** The response body contains exactly `"error": "unsupported_failure_mode"`
   and `"mode": "<VariantName>"` where `<VariantName>` is the name of the unsupported
   `FailureMode` variant submitted.
3. **Idempotent state:** The clone's internal failure state after the HTTP 400 response is
   identical to its state before the call. Subsequent requests from the clone to its target
   upstream return normal (non-failure) responses exactly as if no configure call had occurred.

This property prevents silent failure-mode injection: without this guard, a test using an
unsupported mode would receive HTTP 200 but the failure would never be injected, producing
a false-green test that silently passes without exercising the intended failure path.

**Scope note — per-clone coverage:** BC-3.6.001 Invariant 5 defines the supported-mode
table per clone type. This VP covers the guard logic for any clone type where the submitted
mode is not in that clone's supported list. Representative test vector: Jira clone +
`FailureMode::MalformedResponse` (BC-3.6.001 TV-7), which is not in the Jira supported-mode
list. Full per-clone coverage is incremental — at least one clone type must exercise this
path; all clone types reach full coverage once each unsupported-mode variant is exercised
per BC-3.6.001 VP-157 annotation.

## Source Contract

- **Anchor Story:** `S-3.6.01`
  — Anchor justification (POL-5): `S-3.6.01` is the delivery story for BC-3.6.001 ops-clone
  failure injection, which introduces Postcondition 5 (HTTP 400 for unsupported modes) as
  the primary new behavioral surface. EC-008 and EC-009 in BC-3.6.001 are the covering
  error-condition rows. TV-7 is the canonical test vector (Jira + MalformedResponse).
- **Source BC:** BC-3.6.001 v1.17 — per-org failure injection; Postcondition 5
  (`POST /dtu/configure` with unsupported mode returns HTTP 400), EC-008 (unsupported mode
  → HTTP 400 with `unsupported_failure_mode` body), EC-009 (no state change after 400).
- **Module:** prism-dtu-harness
- **Category:** Error handling / Test infrastructure correctness

## Proof Method

| Method | Tool | Bounded? | Coverage |
|--------|------|----------|----------|
| unit_test | tokio::test (async HTTP) or synchronous handler invocation | Yes — finite set of (clone type, unsupported mode) combinations | HTTP 400 status; `{"error": "unsupported_failure_mode", "mode": "..."}` body shape; no state change on subsequent normal request |

**Why unit_test:** The `POST /dtu/configure` handler performs a synchronous mode-table
lookup and immediate 400/200 branch — no external I/O, no async network. The handler can
be tested by calling it directly with an `inject_failure(mode)` call against a running
clone, which is cheaper than a full harness integration test. This matches BC-3.6.001's
characterization of VP-157 as a unit test ("unit test (per ops clone — Jira, PagerDuty,
Slack; once-per-unsupported-mode until full coverage is ported)").

## Proof Evidence

**Status: PROVEN.** S-3.6.01 merged via PR #83. The property is fully covered by tests
in `crates/prism-dtu-harness/tests/bc_3_6_001_ops_clone_failure_modes.rs`.

### Proven Tests (Phase 1 — Postcondition 5 primary coverage)

| Test Function | Clone | Unsupported Mode | Asserts |
|---|---|---|---|
| `test_BC_3_6_001_jira_unsupported_mode_returns_400` | Jira | MalformedResponse (BC-3.6.001 TV-7) | HTTP 400; `{"error":"unsupported_failure_mode","mode":"..."}` body; no state change |
| `test_BC_3_6_001_pagerduty_unsupported_mode_returns_400` | PagerDuty | unsupported mode | HTTP 400; body shape; no state change |
| `test_BC_3_6_001_slack_unsupported_mode_returns_400` | Slack | unsupported mode | HTTP 400; body shape; no state change |

### Proven Tests (Phase 2 — per-clone extension, covering remaining BC-3.6.001 VP-157 annotation)

| Test Function | Clone | Asserts |
|---|---|---|
| `test_BC_3_6_001_claroty_unsupported_auth_mode_returns_400` | Claroty | HTTP 400; body shape |
| `test_BC_3_6_001_armis_unsupported_auth_mode_returns_400` | Armis | HTTP 400; body shape |
| `test_BC_3_6_001_armis_native_unknown_mode_returns_400_correct_body` | Armis native | HTTP 400; `{"error":"unsupported_failure_mode","mode":"..."}` body (wire-shape assertion) |
| `test_BC_3_6_001_crowdstrike_clone_unsupported_auth_mode_returns_400` | CrowdStrike | HTTP 400; body shape |
| `test_BC_3_6_001_generic_handler_nvd_unsupported_auth_mode_returns_400` | Generic/NVD | HTTP 400; body shape |

### Test Infrastructure (verified real symbols)

- **`configure_failure`** — async helper in `bc_3_6_001_ops_clone_failure_modes.rs`
  that POSTs to `POST /dtu/configure` with a given `dtu_type`, `org_id`, and `FailureMode`
  variant; returns HTTP response.
- **`jira_create_issue`** — async helper in `bc_3_6_001_ops_clone_failure_modes.rs`
  that sends a create-issue request to a running Jira clone; returns status code `u16`.
  Used in "no state change" assertions after a 400 response (EC-009).

### Kill Conditions (mutation targets, BC-3.6.001 Postcondition 5)

- Returning HTTP 200 instead of 400 → `test_BC_3_6_001_jira_unsupported_mode_returns_400` fails on status assertion
- Omitting `"error"` key from body → body shape assertion fails
- Not echoing variant name in `"mode"` → mode field assertion fails
- Applying mode to clone state despite 400 → `jira_create_issue` no-state-change assertion fails

## Feasibility Assessment

| Factor | Assessment | Notes |
|--------|------------|-------|
| Bounded inputs? | Yes | Finite set of (clone type, FailureMode variant) pairs; test covers at least Jira + MalformedResponse per BC-3.6.001 TV-7 |
| Tool support? | Full | tokio::test; serde_json; standard reqwest client in test harness; all in workspace |
| Execution time budget | < 5 seconds | Clone startup + single HTTP round-trip; lightweight compared to full integration tests |
| Assumptions required | One | Admin token per BC-3.6.001 Postcondition 3 must be retrievable from the harness; confirmed by BC-3.6.001 §Invariant description |
| State inspection | Indirect | "No state change" is verified by asserting a subsequent normal request succeeds (EC-009) — clone remains operational, no failure injected |
| Platform constraint | None | Pure Rust async test; all platforms |

## Lifecycle

| Event | Date | Actor |
|-------|------|-------|
| registered in VP-INDEX | 2026-06-11 | state-manager (D-1099 — POL-1 ID-collision correction; VP-131 was erroneously cited for this property; VP-157 allocated as the correct sequential ID) |
| file authored | 2026-07-27 | architect (F-WASE-P65-OBS-001 — VP-INDEX row existed since D-1099 but no VP file was ever created) |

## Changelog

| Version | Burst | Date | Author | Notes |
|---------|-------|------|--------|-------|
| 1.1 | FB70 | 2026-07-27 | architect | F-WASE-P66-MED-003: Promoted `draft` → `active`. S-3.6.01 merged via PR #83; property is fully proven. Phantom symbols removed from proof harness skeleton (`build_test_harness_single_jira_clone`, `post_dtu_configure`, `issue_creation_request_jira` — zero occurrences in crates/). Replaced with real proof evidence citing verified test functions (`test_BC_3_6_001_jira_unsupported_mode_returns_400`, `test_BC_3_6_001_pagerduty_unsupported_mode_returns_400`, `test_BC_3_6_001_slack_unsupported_mode_returns_400`, plus five Phase 2 extension tests) and real helpers (`configure_failure`, `jira_create_issue`) from `bc_3_6_001_ops_clone_failure_modes.rs`. `proof_completed_date` set to 2026-07-27. |
| 1.0 | FB68c | 2026-07-27 | architect | F-WASE-P65-OBS-001: Initial VP file authoring. VP-INDEX row and metadata existed since D-1099 (2026-06-11) per POL-1 ID-collision correction (VP-131 had been erroneously cited for this property in BC-3.6.001 v0.5; VP-157 was allocated in BC-3.6.001 v0.6 and VP-INDEX v1.78). No metadata changes — module (prism-dtu-harness), method (unit_test), priority (P1), anchor story (S-3.6.01), and source BC (BC-3.6.001) remain as originally registered. File gap closed. |
