---
document_type: pr-adversary-pass-report
pass: 3
story: S-DEMO-MULTI-TENANT-DTU-001
pr: 187
pr_head_at_review: 846c21dc
feature_branch: feature/S-DEMO-MULTI-TENANT-DTU-001
base_branch: develop
develop_head_at_review: f7400f83
clean_strict: "NO"
clean_pr_merge: "NO"
streak_before: "0/3"
streak_after: "0/3 (HIGH finding F-PR3-HIGH-001 found and CLOSED in-scope; new commit 41d093fe)"
date: 2026-06-14
producer: adversary (D-1155)
---

# PR-LEVEL Adversary Pass 3 — S-DEMO-MULTI-TENANT-DTU-001 PR #187

## Summary

**CLEAN(strict): NO** (1 HIGH finding F-PR3-HIGH-001, resolved in-scope via architect-adjudicated combined fix)
**CLEAN(PR-merge): NO** (HIGH finding present at start of pass)

Pass 3 has two parts. Part A verified the POL-32 changelog reorder from Pass-2 fix-burst. Part B conducted a full fresh-context adversarial review and found 1 HIGH finding (F-PR3-HIGH-001) plus 1 OBS.

F-PR3-HIGH-001 was closed in-scope via an architect-adjudicated combined fix across three agents (product-owner BC-2.06.017 v1.8→v1.9 + test-writer/implementer new prism-sensors integration test commit 41d093fe + story-writer story v1.12→v1.13). The finding is now CLOSED. PR-LEVEL streak remains 0/3 due to the HIGH; Pass-4 is next.

---

## Part A — Pass-2 Fix Verification

**POL-32 changelog reorder (F-PR2-MED-001 closure):**

- Story changelog: verified top row = v1.12 (highest), bottom row = v1.0 (lowest). DESCENDING per POL-32. CLOSED.
- BC-2.06.017 changelog: verified top row = v1.8 (highest), bottom row = v1.0 (lowest). DESCENDING per POL-32. CLOSED.
- Both changelogs: row content unchanged (reorder-only); all historical entries preserved.

Part A: ALL PASS-2 CLOSURES VERIFIED.

---

## Part B — Full Fresh-Context Adversarial Review

### Review scope

Full PR diff at HEAD 846c21dc. All commits in the PR:
- 9b4f4154 — initial TDD implementation (MultiInstanceServers, isolation counter, tests)
- a27b0f54 — Pass-6 Drop-only doc fix (MultiInstanceHarness)
- 96fce1ad — tls-removal no-default-features E0053 fix
- 74d0bd4c — SEC-001/SEC-002 CLOSED (validate_harness_key input validation + 6 tests)
- 89764cda — brotli pin fixes (repo-wide, not story-specific)
- 846c21dc — SEC-006 CLOSED (CWE-209 redaction) + Pass-1/Pass-2 changelog reorder

### F-PR3-HIGH-001 [HIGH] AC-006 Isolation Tests: Distinct-Listener TCP Tautology, Not FanOutTarget→base_url Routing Proof

**Severity:** HIGH
**Status:** CLOSED (combined fix commit 41d093fe; in-scope autonomous architect adjudication)

**Finding:**

The story's primary value proposition (AC-006, INV-ISOLATION-001) claims to prove that `fan_out_with_overlay_map` routes each client's query to the CORRECT per-client DTU instance based on `FanOutTarget.base_url`. However, the LOCAL convergence's F-P1-HIGH-001 fix closed only a narrower subproblem (client-side counter paper-fix → server-side AtomicU64 delta assertion). The resulting isolation tests in `crates/prism-dtu-harness/tests/` prove that:

1. Two simultaneously running ArmisClone instances on distinct ports produce distinct request-count deltas when the harness addresses them by separate `SocketAddr`.
2. Cross-count zero-leak between instances A and B under sequential calls.

This is a **distinct-listener TCP tautology**: of COURSE two separate TCP sockets on separate ports receive separate requests. The test proves socket-level separation, NOT that `fan_out_with_overlay_map` in `prism-sensors/src/fanout.rs` actually reads `FanOutTarget.base_url` and dispatches the CORRECT DTU for each org-slug.

**Structural impossibility identified (INV-PERIMETER-001):**

The story spec (Postcondition 4, BC-2.06.017 v1.8) references `FanOutTarget→base_url` routing as the spec-claimed mechanism. However, `FanOutTarget` is defined in `prism-sensors`. Per INV-PERIMETER-001, `prism-dtu-harness` and `prism-dtu-demo-server` MUST NOT import `prism-sensors` (forbidden dep direction). This means the harness test crate structurally cannot import `FanOutTarget` and cannot drive `fan_out_with_overlay_map` directly. The prior LOCAL convergence adversary (11 passes) accepted the in-crate proxy (harness-internal socket-level test) as "isolation proven" without recognizing the spec-vs-perimeter contradiction.

**ROOT CAUSE of the structural gap:**

BC-2.06.017 v1.8 Postcondition 4 demanded "fan_out_with_overlay_map routes to FanOutTarget base_url" as the verification mechanism, but INV-PERIMETER-001 forbade the harness test crate from importing FanOutTarget (prism-sensors ← prism-dtu-harness is a forbidden direction). This meant the spec's claimed proof was structurally impossible in-crate; the test silently degraded to a TCP-level tautology that the LOCAL adversary accepted.

**Resolution (architect-adjudicated, IN-SCOPE autonomous — no human authorization required):**

COMBINED fix across three specialist agents:

1. **product-owner:** BC-2.06.017 v1.8→v1.9 — Postcondition 4 and VP Catalog entries narrowed to DISTINCT-LISTENER isolation scope (what the harness tests actually prove). Architecture-Anchors narrowed to reference the REAL FanOutTarget routing proofs: (a) `prism-sensors/src/fanout.rs::test_F_LP2_CRIT_001` (merged S-CONFIG PR#155) + (b) the new `prism-sensors/tests/multi_tenant_dtu_routing_integration.rs` integration test (commit 41d093fe). INV-ISOLATION-001 invariant language UNCHANGED (it states the behavioral property, not the proof mechanism). Cross-reference added: "FanOutTarget→base_url routing proven end-to-end in prism-sensors/tests/ (permitted dep direction)."

2. **test-writer/implementer:** Added `crates/prism-sensors/tests/multi_tenant_dtu_routing_integration.rs::test_fan_out_with_overlay_map_routes_to_correct_dtu_instance`. This test drives the REAL `fan_out_with_overlay_map` through the overlay map mechanism, standing up two live ArmisClone instances (acme→port A, contoso→port B), calling `fan_out_with_overlay_map` with the overlay wiring, and asserting: acme query → ArmisClone A request_count delta=6 + ArmisClone B delta=0 (zero-leak); symmetric for contoso. `prism-sensors/Cargo.toml` gains `[dev-dependencies]` += `prism-dtu-harness` + `prism-dtu-armis` + `prism-dtu-common` (permitted INV-PERIMETER-001 direction: prism-sensors can depend on DTU harness/clones for dev/test). Feature HEAD advanced to 41d093fe.

3. **story-writer:** Story v1.12→v1.13 — AC-006 and Story-Level-Goal narrowed to reflect the two-crate proof structure; `crates_touched` += `prism-sensors`; Architecture Mapping + File Structure Reference rows added for the new integration test; Red Gate Test Plan row for `test_fan_out_with_overlay_map_routes_to_correct_dtu_instance` added.

**Test results (41d093fe):** `just check` GREEN; `cargo nextest run -p prism-sensors` includes the new integration test PASSING. Acme-to-A delta 6, B-leak 0; contoso-to-B delta 6, A-leak 0. The story's core multi-tenant routing-isolation value proposition is now GENUINELY proven end-to-end.

---

### OBS-1 [OBS] HarnessError::InvalidKey Documentation Drift in Prior Pass Report Prose

**Severity:** OBS (non-blocking)
**Status:** CLOSED (non-blocking; orchestrator-confirmed)

Prior pass report prose (pr-adversary-pass-1.md) described SEC-002 fix as using `HarnessError::InvalidKey`. The actual code (commit 74d0bd4c) uses `validate_harness_key` returning `io::Error` with `ErrorKind::InvalidInput`; functional redaction is present and load-bearing. The discrepancy is between pass-report narrative prose (not a spec artifact) and the actual code variant name. Non-blocking — pass reports are immutable historical records; the code itself is correct. Orchestrator-confirmed non-blocking. No spec or code change required.

---

## Streak Status

PR-LEVEL streak: **0/3** (F-PR3-HIGH-001 HIGH required fix; CLOSED in-scope via 41d093fe; streak resets to 0/3 per BC-5.39.001 D-779)

**NEXT:** Orchestrator pushes 41d093fe to origin/feature/S-DEMO-MULTI-TENANT-DTU-001. Then PR-LEVEL Pass 4 dispatched (fresh context; verify new integration test + BC v1.9 + story v1.13 complete and consistent).

---

## Checklist Sweep

- [x] SAP-1 (tracing emission catalog): no new `event_type=` emissions in diff. CLEAN.
- [x] SAP-2 (DTU↔TOML schema parity): no sensor TOML spec changes in diff. N/A.
- [x] INV-PERIMETER-001: new test in prism-sensors/tests/ — dev-dep direction prism-sensors→prism-dtu-harness/armis/common is PERMITTED (spec-engine/sensors/query direction is what is forbidden). CLEAN.
- [x] POL-32 changelog direction (Part A): story v1.12 top, v1.0 bottom; BC v1.8 top, v1.0 bottom — both DESCENDING. CLOSED.
- [x] EXPECTED=60: no change to non-exhaustive gate. UNCHANGED.
- [x] unwrap/expect in production paths: none introduced. CLEAN.
- [x] BC-2.06.017 v1.9 + story v1.13 consistent with each other and with code at 41d093fe. VERIFIED.
