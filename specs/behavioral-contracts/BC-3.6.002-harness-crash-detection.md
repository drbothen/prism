---
document_type: behavioral-contract
level: L3
bc_id: BC-3.6.002
title: Harness Crash Detection
version: "0.4"
status: PROPOSED
producer: product-owner
timestamp: 2026-04-27T00:00:00
phase: 3.A
wave: 3
inputs: [.factory/specs/architecture/decisions/ADR-011-harness-isolation-modes.md]
input-hash: "c1610fc"
traces_to: ".factory/specs/architecture/decisions/ADR-011-harness-isolation-modes.md"
origin: greenfield
extracted_from: null
subsystem: SS-01
capability: CAP-036
authors: [product-owner]
related_decisions: [D-044, D-045]
related_adrs: [ADR-011]
inherits_from: null
superseded_by: null
lifecycle_status: active
introduced: wave-3
modified: []
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-3.6.002: Harness Crash Detection

## Description

The `Harness` monitors each clone's Tokio `JoinHandle` after startup. If a clone task
exits unexpectedly — due to a panic, returning `Err`, or completing `Ok` before the test
finishes — the harness marks that `(OrgId, DtuType)` as crashed. The next harness
operation targeting that clone returns `HarnessError::CloneCrashed { org_id, dtu_type,
cause }` with the exit cause string (panic message or error description). This converts
otherwise-silent `ConnectionRefused` errors into explicit diagnostic failures, preventing
the class of debugging waste where a clone panic is attributed to "network instability" or
"port allocation race" (ADR-011 §2.6, §3.3).

## Preconditions

1. A `Harness` has been built via `HarnessBuilder::build().await` — all clone Tokio tasks
   are running and their `JoinHandle`s are stored in the harness.
2. The harness polls each clone's task handle for unexpected exit via a non-blocking
   `try_recv` on the crash notification channel on every harness operation (ADR-011 §3.3).
3. The crash notification channel is a `tokio::sync::watch` or equivalent that the clone
   task signals before unwinding — this gives the harness access to the panic message or
   `Err` value.
4. The `dtu` feature flag is enabled.

## Postconditions

1. When a clone task exits unexpectedly (panic, `Err`, or premature `Ok`), the harness
   detects the exit within 1 second of the exit event and marks that `(OrgId, DtuType)`
   as crashed in its internal state.
2. The next call to any harness operation that targets the crashed `(OrgId, DtuType)` —
   including `inject_failure`, `clear_failure`, or any sensor query routed to that clone —
   returns `Err(HarnessError::CloneCrashed { org_id, dtu_type, cause })` where `cause`
   is a non-empty string containing the panic message or error description.
3. A clone crash does not affect other `(OrgId, DtuType)` pairs in the same harness;
   operations targeting non-crashed clones continue to succeed.
4. `drop(harness)` after a clone crash completes cleanly — no zombie Tokio tasks remain;
   shutdown senders for non-crashed clones are consumed normally; the already-exited clone
   task requires no additional teardown.
5. The `HarnessError::CloneCrashed` error includes at minimum: the `OrgId`, the `DtuType`,
   and the last available diagnostic string (panic payload or `Err` display).

## Invariants

1. No harness operation silently swallows a clone crash and returns a `None` result or
   `ConnectionRefused` error — every operation checks the crash state before making an
   HTTP connection attempt.
2. Once a clone is marked crashed, its crash state is permanent for the harness lifetime;
   there is no automatic recovery or restart of a crashed clone.
3. Crash detection operates via non-blocking `try_recv` — it does not block the calling
   test thread waiting for a crash that may never occur.
4. The crash cause string is captured at exit time (from the clone's task), not reconstructed
   after the fact; if the panic payload is not a string, the cause is `"(non-string panic
   payload)"`.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Clone panics mid-test with a string message | `cause` field contains the panic string; `CloneCrashed` returned on next operation targeting that clone |
| EC-002 | Clone panics with a non-string payload (e.g., `panic!(42)`) | `cause` is `"(non-string panic payload)"`; `CloneCrashed` still returned |
| EC-003 | Clone exits `Ok` prematurely (graceful exit before test ends) | Treated identically to a crash; `CloneCrashed` returned with `cause = "task exited Ok before test completion"` |
| EC-004 | Clone crashes; test immediately calls `drop(harness)` | Drop completes cleanly; already-exited task is a no-op for teardown; non-crashed clones shut down gracefully |
| EC-005 | Two clones crash simultaneously | Both are marked crashed independently; `CloneCrashed` returned for each; other clones unaffected |
| EC-006 | Crash detected during `inject_failure` call | `inject_failure` returns `Err(HarnessError::CloneCrashed { ... })`; no `POST /dtu/configure` is attempted |
| EC-007 | Clone crash occurs between two consecutive test assertions | First assertion sees HTTP 200 (before crash); second assertion returns `CloneCrashed` (detected within 1s); no silent data return after crash |

## Canonical Test Vectors

| Scenario | Setup | Action | Expected Result | Pass Condition |
|----------|-------|--------|----------------|----------------|
| TV-1: Panic detection | harness(OrgA:Claroty); force clone panic via test hook | Wait up to 1s; query OrgA | `HarnessError::CloneCrashed` | Error within 1s; cause is non-empty string |
| TV-2: Cause string preserved | harness(OrgA:Armis); panic with message "index out of bounds at row 42" | Query OrgA after crash | `CloneCrashed { cause: "index out of bounds at row 42" }` | Cause string matches panic message verbatim |
| TV-3: Non-crashed clone unaffected | harness(OrgA:Claroty, OrgB:Claroty); crash OrgA | Query OrgB after OrgA crash | HTTP 200 from OrgB | OrgB responds normally; only OrgA returns `CloneCrashed` |
| TV-4: Clean drop after crash | harness(OrgA:Claroty); crash clone; drop harness | Check for zombie tasks | No running tasks | All handles joined; no leaked Tokio tasks |
| TV-5: Premature Ok exit | harness(OrgA:Cyberint); clone task returns `Ok(())` before test ends | Query OrgA | `CloneCrashed { cause: "task exited Ok before test completion" }` | Error returned; no connection attempt made |
| TV-6: CloneCrashed during inject_failure | harness(OrgA:CrowdStrike); crash clone; call inject_failure on OrgA | `inject_failure` return value | `Err(HarnessError::CloneCrashed { ... })` | No HTTP call made; error propagated |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-131 | A clone panic is detected within 1s of task exit | integration test (measure time from forced panic to `CloneCrashed` return) |
| VP-132 | `drop(harness)` after any number of clone crashes completes without hanging | integration test (timeout-gated drop) |
| VP-133 | No harness operation returns `ConnectionRefused` when a crashed clone is targeted — it always returns `CloneCrashed` | integration test (assert error variant) |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-036 ("Multi-Tenant DTU Test Harness") per capabilities.md §CAP-036 |
| Capability Anchor Justification | CAP-036 ("Multi-Tenant DTU Test Harness") per capabilities.md §CAP-036 — this BC specifies the harness's crash monitoring and diagnostic surfacing behavior, a core sub-capability of the proposed CAP-036 multi-tenant test harness. No existing CAP-001 through CAP-035 covers test-infrastructure crash detection. |
| L2 Domain Invariants | n/a (harness is test infrastructure; no DI-NNN enforced) |
| Architecture Module | prism-dtu-harness/src/harness.rs (JoinHandle monitoring per ADR-011 §2.6); crash notification channel |
| Stories | S-3.3.03, S-3.6.01, S-3.6.02 |

## Related BCs

- BC-3.5.001 — crash detection applies within logical-mode harness instances
- BC-3.5.002 — crash detection applies within network-mode harness instances
- BC-3.6.001 — a crashed clone hit by `inject_failure` returns `CloneCrashed` (EC-006 above)

## Architecture Anchors

- `architecture/decisions/ADR-011-harness-isolation-modes.md#26-crash-detection` — defines `HarnessError::CloneCrashed`, task handle monitoring, and the motivation (silent `ConnectionRefused` prevention)
- `architecture/decisions/ADR-011-harness-isolation-modes.md#33-clone-crash-during-test-produces-misleading-assertion` — threat model entry this BC mitigates

## Story Anchor

S-3.3.03, S-3.6.01, S-3.6.02

## VP Anchors

- VP-131 — integration_test: clone panic detected within 1s of task exit
- VP-132 — integration_test: drop(harness) after any number of clone crashes completes without hanging
- VP-133 — integration_test: targeted crashed clone returns CloneCrashed, never ConnectionRefused

## BC Changelog

| Version | Change |
|---------|--------|
| v0.4 | m-001 (Pass 6): `input-hash` populated: SHA1 of input file path (first 7 chars = `8606916`). |
| v0.3 | M-004/Audit-5 (Pass 5): Frontmatter `title:` corrected to title-case to match H1 heading. `traces_to:` corrected from `specs/domain-spec/capabilities.md` to `.factory/specs/architecture/decisions/ADR-011-harness-isolation-modes.md`. |
| v0.2 | Initial authoring from ADR-011. |
