---
document_type: adversarial-review
story_id: S-DEMO-MULTI-TENANT-DTU-001
pass: 1
scope: LOCAL
protocol: BC-5.39.001 3-CLEAN-strict
produced_by: adversary (fresh-context)
timestamp: 2026-06-13T23:55:00Z
verdict_clean_strict: "NO"
verdict_clean_pr_merge: "NO"
findings_total: 7
findings_crit: 0
findings_high: 1
findings_med: 3
findings_low: 3
all_findings_closed: true
streak_before: 0
streak_after: 0
next_pass: 2
---

# LOCAL Adversary Pass 1 — S-DEMO-MULTI-TENANT-DTU-001

**Story:** prism-dtu-demo-server + prism-dtu-harness: Per-DTU-Instance Multi-Address Binding  
**Protocol:** BC-5.39.001 3-CLEAN-strict (D-779 disambiguation — strict = zero findings of ANY severity)  
**Verdict:** CLEAN(strict)=NO / CLEAN(PR-merge)=NO  
**Streak:** 0/3 (findings present; streak cannot advance)  
**Next:** Pass 2

---

## Finding Summary

| ID | Severity | Status | Title |
|----|----------|--------|-------|
| F-P1-HIGH-001 | HIGH | CLOSED | AC-006/INV-ISOLATION-001 isolation tests were paper-fix (client-side counters) |
| F-P1-MED-001 | MED | CLOSED | Stale `Ok(HashMap::new())` after v1.2 return-type change |
| F-P1-MED-002 | MED | CLOSED | Postcondition 6 deterministic drain race |
| F-P1-MED-003 | MED | CLOSED | Missing EADDRINUSE bind-failure test |
| F-P1-LOW-001 | LOW | CLOSED | Stale "52→59" count in non-exhaustive gate crate doc/comments |
| F-P1-LOW-002 | LOW | CLOSED | Stale "Let me recount" doc artifact in non-exhaustive gate crate |
| F-P1-LOW-003 | LOW | CLOSED | Misleading watcher-task comments not matching code |
| OBS-1 | OBS | NO ACTION | architect-locked `.factory/` — state-manager owns; no implementer action |
| OBS-2 | OBS | NO ACTION | INV-PERIMETER-001 clean — perimeter-violation compile-fail gate verified clean |
| OBS-3 | OBS | NO ACTION | Regression guards meaningful — existing isolation test harness sound |
| OBS-4 | OBS | NO ACTION | Semantic anchoring coherent — BC-2.06.017 invariant names match code identifiers |
| OBS-5 | OBS | NO ACTION | BC status:draft correct pre-merge — POL-14 promotes at merge |

---

## Findings Detail

### F-P1-HIGH-001 — AC-006/INV-ISOLATION-001 Isolation Tests: Paper-Fix (Client-Side Counters)

**Severity:** HIGH  
**Status:** CLOSED  
**Classification:** SUBSTANTIVE

**Finding:** The three isolation tests introduced for AC-006 / INV-ISOLATION-001 asserted isolation using client-side counters (request counts tracked by the test harness, not the server). This is a paper-fix: the counter assertion `assert_eq!(count_b, 1)` could pass even if traffic actually leaked to instance A, because the counter only increments on explicit test-side calls — it does not verify what the server actually received.

**Fix applied by implementer:** Added a load-bearing server-side request counter to `ArmisClone`:
- `AtomicU64` field in `ArmisClone` struct (initialized to 0)
- Axum middleware layer that increments the counter on every received HTTP request
- `request_count()` method exposing the atomic value
- `GET /dtu/request-count` HTTP route returning the count as a JSON body

**Fix applied by test-writer:** Rewrote 3 isolation tests to use server-side delta assertions:
- Before each isolation sequence: snapshot `a.request_count()` and `b.request_count()`
- Send exactly 1 request to instance B
- Assert `b.request_count() - b_before == 1` (exactly 1 request received)
- Assert `a.request_count() == a_before` (delta == 0; any cross-tenant leak → counter > 0 → test fails)

This makes isolation load-bearing: the test fails mechanically if any cross-tenant routing leak occurs.

---

### F-P1-MED-001 — Stale `Ok(HashMap::new())` After v1.2 Return-Type Change

**Severity:** MED  
**Status:** CLOSED  
**Classification:** SUBSTANTIVE

**Finding:** After the D-1145 API adjudication changed `start_instances` return type from `HashMap<String,SocketAddr>` to `MultiInstanceServers`, two sites still returned `Ok(HashMap::new())` — the pre-v1.2 error-path return. These would cause a compile error under v1.2 (type mismatch) but were present in code that had not yet been rebuilt post-adjudication.

**Fix applied by product-owner:** BC-2.06.017 v1.2→v1.3:
- EC-017-002 updated: error-path return type corrected from `Ok(HashMap::new())` to the canonical `BindFailure` variant of `MultiInstanceBindError`
- TV-017-002 updated: test vector table row corrected to match the new error-path return type
- Both sites now consistently specify `Err(MultiInstanceBindError::BindFailure(...))` for the no-instances-bound case

**Fix applied by story-writer:** S-DEMO-MULTI-TENANT-DTU-001 v1.4→v1.5:
- EC-002 error table row corrected to match BC-2.06.017 v1.3 EC-017-002 semantics
- Inline annotation referencing `Ok(HashMap::new())` removed

**Fix applied by implementer:** Added doc comment to `multi_instance.rs` citing the v1.3 error-path return semantics, so future maintainers cannot regress without seeing the explicit annotation.

---

### F-P1-MED-002 — Postcondition 6 Deterministic Drain Race

**Severity:** MED  
**Status:** CLOSED  
**Classification:** SUBSTANTIVE

**Finding:** The Postcondition 6 contract (graceful drain on `servers.shutdown()`) had a race: the implementation awaited the watcher task rather than calling `clone.stop().await` on all bound instances before returning `BindFailure`. If a partial-bind failure occurred (some instances bound, some failed), the already-bound instances were not gracefully stopped — they became detached tasks (the zombie-server / port-leak problem the D-1145 API adjudication was meant to close).

**Fix applied by implementer:** Reworked the error path in `start_instances`:
- On `BindFailure`, iterate over all successfully-bound instances collected so far
- Call `clone.stop().await` on each, in sequence, before constructing the `BindFailure` return value
- This causes real port release (the OS frees the bound port) rather than a vacuous watcher-await
- The `Drop` impl on `MultiInstanceServers` was also verified to call the same shutdown path for the normal-exit case, ensuring the drain is deterministic in both success and partial-failure paths

---

### F-P1-MED-003 — Missing EADDRINUSE Bind-Failure Test

**Severity:** MED  
**Status:** CLOSED  
**Classification:** SUBSTANTIVE

**Finding:** There was no test exercising the `EADDRINUSE` bind-failure path (the OS refusing a port that is already in use). Without this test, Postcondition 6 + the error path fix in F-P1-MED-002 had no concrete Red Gate coverage — a future regression could silently reintroduce the race.

**Fix applied by test-writer:** Added two new tests per TV-017-005:
1. **Demo-server bind-failure test:** Binds a `TcpListener` on an ephemeral port, then calls `start_instances` with that same port in the config. Asserts `BindFailure` is returned and that no instances remain bound after the call (port released by the cleanup path from F-P1-MED-002).
2. **Harness bind-failure aggregation test:** Exercises `MultiInstanceHarness` with one valid and one EADDRINUSE config entry. Asserts the harness returns `HarnessError::BindFailure(Vec<BindError>)` with exactly one entry, and that the valid instance is also stopped (no partial-bind leak).

---

### F-P1-LOW-001 — Stale "52→59" Count in Non-Exhaustive Gate Crate

**Severity:** LOW  
**Status:** CLOSED  
**Classification:** COSMETIC

**Finding:** Two locations in the `tests/external/non-exhaustive-violation/` crate (source comment + a doc-string) still referenced `52→59` as the expected-count migration. After D-1144 re-baselined to `52→59` and D-1145 re-baselined again to `52→60`, the final correct count is `EXPECTED=60`. The stale `52→59` in doc artifacts would mislead future contributors about the migration delta.

**Fix applied by implementer:** Corrected both sites to `52→60`.

---

### F-P1-LOW-002 — Stale "Let me recount" Doc Artifact in Non-Exhaustive Gate Crate

**Severity:** LOW  
**Status:** CLOSED  
**Classification:** COSMETIC

**Finding:** A comment in the non-exhaustive-violation crate included the phrase "Let me recount" — a draft-time artifact that leaked into committed code. It carried no semantic meaning and would appear in `cargo doc` output.

**Fix applied by implementer:** Removed the draft-time artifact. The comment was rewritten to a concise, accurate description of the expected count and the story that owns each category of compile-fail arm.

---

### F-P1-LOW-003 — Misleading Watcher-Task Comments

**Severity:** LOW  
**Status:** CLOSED  
**Classification:** COSMETIC

**Finding:** Three comments in the watcher-task implementation described the task as "monitoring the shutdown channel" when in fact the channel is used to signal shutdown to the server, not to monitor it. The inversion would mislead future readers about the data-flow direction.

**Fix applied by implementer:** Rewrote all three comments to accurately describe the channel's role: the watcher task holds the `broadcast::Receiver<ShutdownSignal>` and passes it to `axum::with_graceful_shutdown` — the server exits when the sender fires, not the other way around.

---

## Observations (No Action)

**OBS-1:** `.factory/` is architect-locked. State-manager is the sole owner of STATE.md and cycle artifacts. No implementer action required.

**OBS-2:** INV-PERIMETER-001 verified clean. The `tests/external/perimeter-violation/` gate does not acquire `prism-dtu-demo-server` or `prism-dtu-harness` as direct dependencies; the perimeter is correctly maintained.

**OBS-3:** Existing isolation test harness structure is sound — the paper-fix in F-P1-HIGH-001 was in the assertion mechanism, not the harness topology. After the server-side counter fix, the harness topology is correct and the regression guards are meaningful.

**OBS-4:** Semantic anchoring is coherent. BC-2.06.017 invariant names (`INV-ISOLATION-001`, `INV-COMPAT-001`, `INV-ERR-003-COMPAT`, `INV-PERIMETER-001`, `INV-NONEXHAUSTIVE-001`) match the code identifiers used in test names and doc comments. No naming drift.

**OBS-5:** BC-2.06.017 `status: draft` is correct pre-merge per POL-14. The BC promotes to `active` at `S-DEMO-MULTI-TENANT-DTU-001` merge.

---

## Post-Fix Verification

| Check | Result |
|-------|--------|
| `just check` (full workspace) | GREEN — 4292 passed, 45 skipped, 0 failed |
| Non-exhaustive gate `EXPECTED=60` | EXACT — 60 compile-fail arms confirmed |
| SAP-1 sweep (`rg 'event_type\s*=' crates/ --type rust`) | CLEAN — zero `event_type` emissions in new code |
| 18 multi_instance / harness tests | ALL PASS |
| 2 new bind-failure tests (TV-017-005) | ALL PASS |
| SAP-2 DTU↔TOML schema parity | N/A — no TOML sensor spec modified in this story |

---

## Cascade Status

**Pass 1:** SUBSTANTIVE (1H + 3M + 3L — all CLOSED by fix-burst)  
**Streak:** 0/3 (findings present; strict criterion requires zero findings of any severity)  
**Next:** Pass 2 — fresh-context adversarial review of post-fix-burst code state  
**develop_head:** f7400f83 (UNCHANGED — story not yet merged; MID-DELIVERY)
