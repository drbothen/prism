---
document_type: verification-property
level: L4
version: "1.0"
status: draft
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
proof_completed_date: null
proof_file_hash: null
lifecycle_status: draft
introduced: "2026-06-11"
modified: null
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

## Proof Harness Skeleton

```rust
// [TODO: harness skeleton — author during Phase 3 story S-3.6.01 TDD delivery]
// Method: unit_test
//
// SYMBOL RESOLUTION — test-writer must verify grounding before authoring tests
//
// TARGET FUNCTION: `POST /dtu/configure` handler in prism-dtu-harness
//   Canonical source: BC-3.6.001 §Traceability — `inject_failure` API route
//   Confirmed entry point: inject_failure call on a running clone instance
//   (grounded via BC-3.6.001 Postcondition 5 and EC-008 description).
//
// HARNESS DEPENDENCIES:
//   - A running ops clone (e.g., Jira) initialized within the test (no external service)
//   - Admin token per BC-3.6.001 Postcondition 3 (per-clone, not shared)
//   - `FailureMode::MalformedResponse` for the unsupported-mode trigger (BC-3.6.001 TV-7)
//
// REPRESENTATIVE TEST — VP-157: unsupported mode returns HTTP 400 with correct body shape
//
// #[tokio::test]
// async fn unsupported_failure_mode_returns_http_400_and_no_state_change() {
//     // 1. Build a harness with one Jira ops clone; obtain admin endpoint address
//     let harness = build_test_harness_single_jira_clone().await;
//     let clone_addr = harness.jira_admin_endpoint();
//
//     // 2. POST /dtu/configure with MalformedResponse (not in Jira supported-mode list)
//     //    BC-3.6.001 TV-7: "Jira rejects MalformedResponse mode"
//     let configure_response = post_dtu_configure(
//         clone_addr, &FailureMode::MalformedResponse
//     ).await;
//
//     // 3. Assert HTTP 400 (not HTTP 200)
//     assert_eq!(configure_response.status(), 400,
//         "unsupported mode must return HTTP 400, not silent 200 (VP-157 / EC-008)");
//
//     // 4. Assert error body shape: {"error": "unsupported_failure_mode", "mode": "MalformedResponse"}
//     let body: serde_json::Value = configure_response.json().await.unwrap();
//     assert_eq!(body["error"], "unsupported_failure_mode",
//         "error field must be 'unsupported_failure_mode' (VP-157 / BC-3.6.001 EC-008)");
//     assert_eq!(body["mode"], "MalformedResponse",
//         "mode field must echo the attempted variant name (VP-157)");
//
//     // 5. Assert no state change: subsequent normal request succeeds (EC-009)
//     let normal_response = issue_creation_request_jira(clone_addr).await;
//     assert_eq!(normal_response.status(), 200,
//         "clone must remain operational after unsupported-mode 400 (VP-157 / EC-009)");
//     // No injected failure: normal response, not a failure response
// }
//
// Kill conditions (mutation testing — these mutations MUST be caught):
//   - Change 400 response to 200 (silent ACK) → test fails on status assertion
//   - Omit "error" key from response body → test fails on body shape assertion
//   - Corrupt "mode" value (e.g., static string instead of echoing variant name)
//     → test fails on mode field assertion
//   - Apply mode to clone state despite error → test fails on subsequent normal-request assertion
```

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
| 1.0 | FB68c | 2026-07-27 | architect | F-WASE-P65-OBS-001: Initial VP file authoring. VP-INDEX row and metadata existed since D-1099 (2026-06-11) per POL-1 ID-collision correction (VP-131 had been erroneously cited for this property in BC-3.6.001 v0.5; VP-157 was allocated in BC-3.6.001 v0.6 and VP-INDEX v1.78). No metadata changes — module (prism-dtu-harness), method (unit_test), priority (P1), anchor story (S-3.6.01), and source BC (BC-3.6.001) remain as originally registered. File gap closed. |
