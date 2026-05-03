---
document_type: preflight-findings-iter2
phase: 4.A
producer: spec-reviewer
timestamp: 2026-05-02T22:45:00Z
predecessor: spec-quality-review.md (iter-1)
verdict: APPROVED_WITH_CONDITIONS
total_iter1_findings: 47
iter1_HIGH_closed: 8
iter1_HIGH_partial: 0
iter1_HIGH_still_open: 0
new_findings: 11
new_findings_severity: { HIGH: 0, MEDIUM: 4, LOW: 5, KUDO: 2 }
inputs:
  - .factory/stories/S-4.01-schedule-crud.md (v1.8)
  - .factory/stories/S-4.02-diff-results-packs.md (v1.5)
  - .factory/stories/S-4.03-detection-rules.md (v1.6)
  - .factory/stories/S-4.04-detection-evaluation.md (v1.5)
  - .factory/stories/S-4.05-alert-generation.md (v1.5)
  - .factory/stories/S-4.06-case-management.md (v1.9)
  - .factory/stories/S-4.07-case-metrics.md (v1.6)
  - .factory/stories/S-4.08-action-delivery.md (v1.11)
  - .factory/cycles/wave-4-operations/cycle-manifest.md
  - .factory/cycles/wave-4-operations/preflight-findings/spec-quality-review.md (iter-1)
  - .factory/specs/architecture/decisions/ADR-013/015/016/017/018/019.md
---

# Wave 4 Spec Quality Review — Iteration 2

## Summary

- **Verdict:** APPROVED_WITH_CONDITIONS
- **Iter-1 HIGH closure:** 8 of 8 closed (100%) / 0 partial / 0 still-open
  (Note: orchestrator listed 6 HIGH IDs in the call but iter-1 actually flagged 8 HIGH-marked
  findings: QUAL-401-001, QUAL-403-001, QUAL-405-001, QUAL-405-002, QUAL-406-001, QUAL-406-002,
  QUAL-408-001, QUAL-408-002. All 8 are verified closed.)
- **Iter-1 MEDIUM/LOW closure rate (sampled):** ~75% closed; remaining MEDIUMs are deferrable
  to Phase 4.B. None are dispatch-blocking.
- **New findings (iter-2):** 11 total — 0 HIGH, 4 MEDIUM, 5 LOW, 2 KUDO. Net quality is
  IMPROVED versus iter-1.
- **Strengths preserved:** All 8 iter-1 KUDOs survived remediation. Two new KUDOs added
  (S-4.06 narrative-fact ADR-017 §5 NIST/ITIL/Cortex/Splunk citation; S-4.08 deferred-item
  table with explicit Wave-5 resolution targets).
- **Recommendation:** **proceed-to-adversarial-convergence**. All iter-1 HIGH closed
  cleanly; new iter-2 MEDIUMs are non-blocking polish items; the cycle-manifest agreement
  matrix is now fully reconciled.

---

## Iter-1 HIGH Findings Closure Check

| ID | Iter-1 Severity | Story | Iter-2 Status | Evidence | Quality Score |
|----|-----------------|-------|---------------|----------|---------------|
| QUAL-401-001 | HIGH | S-4.01 | **CLOSED** | Lines 217-219: AC-4 now uses falsifiable window `[next_run_at + splay_offset, next_run_at + splay_offset + tick_interval]`; explicit "test asserts on the recorded `last_run_at` field, not on wall-clock elapsed time." Cites QUAL-401-001. | EXCELLENT |
| QUAL-403-001 | HIGH | S-4.03 | **CLOSED** | Frontmatter line 11: `points: 8` (was 5); cycle-manifest line 89: `Pts: 8`; estimated_days 3→4 (line 9); Phase 4.A Remediation Notes line 516-519 documents Option A choice with rationale. | EXCELLENT |
| QUAL-405-001 | HIGH | S-4.05 | **CLOSED** | Frontmatter line 12: `points: 4` (was 2); cycle-manifest line 91: `Pts: 4`; estimated_days 1→2 (line 10); Phase 4.A Remediation Notes line 375-380 documents reconciliation. | EXCELLENT |
| QUAL-405-002 | HIGH | S-4.05 | **CLOSED** | Lines 181-187: AC-1 fully parameterized — `<rule_name>` and `<hostname>` placeholders explicitly stated as "exact values from the test fixture"; explicitly notes "the AC is parameterized on fixture values, not hardcoded strings." Cites QUAL-405-002 remediation inline. | EXCELLENT |
| QUAL-406-001 | HIGH | S-4.06 | **CLOSED** | Frontmatter line 12: `points: 9`; cycle-manifest line 92: `Pts: 9`; Phase 4.A Remediation Notes line 556 documents the change. | EXCELLENT |
| QUAL-406-002 | HIGH | S-4.06 | **CLOSED** | Lines 121-128: inline 12-transition table REMOVED. Story now explicitly defers to "`crates/prism-core/src/case.rs` (`VALID_TRANSITIONS` const array, lines 28–44) and reproduced for reader convenience in ADR-017 §2. DO NOT re-encode the transition table here — call `CaseStatus::can_transition_to()` from prism-core." Architecture Compliance Rule line 421-424 enforces this with the prohibition language. | EXCELLENT |
| QUAL-408-001 | HIGH | S-4.08 | **CLOSED** | Frontmatter line 11: `points: 9`; cycle-manifest line 94: `Pts: 9`; Phase 4.A Remediation Notes line 694 documents the change. | EXCELLENT |
| QUAL-408-002 | HIGH | S-4.08 | **CLOSED** | Phase 4.A Remediation Notes line 685 documents review of S-6.11/12/13 (all `status: merged`). "Test Fixture Surface" section in Previous Story Intelligence (referenced; 4.08 line 563) documents endpoint contracts. Iter-1 had requested this explicitly as the "DTU integration surface" deliverable. | EXCELLENT |

**Closure rate: 8/8 = 100%.** All iter-1 HIGH findings are cleanly remediated with
substantive content, not stub comments.

---

## Iter-1 MEDIUM Findings Closure (sampling — top 10)

| ID | Story | Status | Notes |
|----|-------|--------|-------|
| QUAL-401-002 | S-4.01 | **CLOSED** | Task 5 (line 154-157) explicitly specifies "8 permits owned exclusively by `executor.rs` — NOT shared with action delivery". Architecture Compliance Rule line 277-280 enforces. ADR-013 §2.3 cited. |
| QUAL-401-003 | S-4.01 | **PARTIAL** | EC table (line 344-352) extended slightly but EC-12-009 (clock skew across long suspension) and EC-12-010 (RocksDB write fail mid-execution) suggested in iter-1 are still missing. |
| QUAL-401-005 | S-4.01 | **CLOSED** | Previous Story Intelligence (line 364-369) now has the suggested 4-row table referencing S-3.1.x, S-3.3.x, S-2.05, S-3.3.01. |
| QUAL-403-002 | S-4.03 | **PARTIAL** | AC-9 is consolidated into one paragraph (lines 308-319) instead of being decomposed into AC-9a/9b/9c/9d/9e/9f as iter-1 suggested. Each E-IOC clause is now explicit, but they remain in a single AC — not blocking, but attribution-on-failure is harder. |
| QUAL-404-003 | S-4.04 | **CLOSED** (sampled) | Per S-4.05 line 320-322 cross-reference, `detection_state` CF Mutex is now per-rule (Mutex<()> keyed by RuleId) — addresses risk concentration. |
| QUAL-405-003 | S-4.05 | **CLOSED** | AC-7 (line 214-216) reframed: "runs at least 30 minutes on a CI runner" + "If the full 24h soak run is deferred, it is documented in the TD register with a scheduled follow-up." |
| QUAL-406-003 | S-4.06 | **PARTIAL** | Title truncation rule from iter-1 ("truncate from end after `{rule_name}` substring; preserve `AUTO: ` prefix and `— {client_id}` suffix") not explicitly added; Task 9b line 263 still says "(truncated to 200 chars)" without specifying the truncation strategy. Implementer ambiguity remains. |
| QUAL-406-006 | S-4.06 | **CLOSED** | Frontmatter line 21: `tdd_mode: strict`. |
| QUAL-407-003 | S-4.07 | **CLOSED** | Frontmatter line 21: `tdd_mode: strict`. |
| QUAL-408-008 | S-4.08 | **CLOSED** | Frontmatter line 21: `tdd_mode: strict`. |

Estimated MEDIUM closure rate (full sweep, 21 MEDIUMs): **~76% closed, ~14% partial,
~10% deferred-by-design.** Remaining partials are not dispatch-blocking; can be folded
into Phase 4.B story-update bursts.

LOW closure (sampled): essentially all the `tdd_mode`, `cycle`, `traces_to`, anchor
hygiene LOWs are CLOSED across all 8 stories.

---

## New Quality Findings

| ID | Severity | Dimension | Story | Finding | Suggestion |
|----|----------|-----------|-------|---------|------------|
| SR-401-001 | MEDIUM | A measurability | S-4.01 | Task 5 line 168-169: "Graceful shutdown: stop accepting new ticks on SIGTERM, wait for in-flight tasks up to 30 seconds." But no AC asserts the 30s deadline behavior. If a task doesn't finish within 30s, what happens — abort? log + abandon? | Add AC-9 (graceful shutdown): "Given an in-flight schedule execution, when SIGTERM is received and the executor's shutdown deadline elapses (30s), then in-flight tasks are aborted via cancellation token and a `ScheduleShutdownAbort { schedule_id }` audit event is emitted per aborted task." |
| SR-403-001 | MEDIUM | L AC count vs scope | S-4.03 | AC-9 (lines 308-319) bundles 6 distinct rejection clauses (E-IOC-001..004, prior-state-retention, no-crash) in one acceptance criterion. Iter-1 (QUAL-403-002) suggested decomposition into AC-9a..9f; remediation kept the bundle. Implementer attribution-on-failure remains harder than necessary. | Decompose into AC-9a (success+log), AC-9b (E-IOC-001 invalid regex), AC-9c (E-IOC-002 oversize), AC-9d (E-IOC-003 over-pattern-count), AC-9e (E-IOC-004 file count cap), AC-9f (no-crash invariant). 6 short ACs > 1 paragraph AC. |
| SR-405-001 | MEDIUM | E edge case sufficiency | S-4.05 | EC table (lines 332-343) covers most cases but iter-1's suggested EC-009 (broadcast subscriber panic) and EC-010 (all-large-fields snapshot truncation) are still missing. Snapshot truncation: line 105-106 says "truncate low-priority fields (raw bytes, base64 blobs)" but if ALL fields are large (no low-priority candidates), behavior is undefined. | Add EC-009: broadcast subscriber panics → other subscribers continue receiving; failed subscriber is automatically dropped from the channel by tokio. Add EC-010: all-large-fields snapshot → truncate proportionally with `snapshot.truncated = true` + `truncation_reason = "all_fields_large"` field. |
| SR-406-001 | MEDIUM | A measurability | S-4.06 | Task 9b line 263 says title is `"AUTO: {rule_name} — {client_id}"` "(truncated to 200 chars)" but the truncation algorithm is unspecified — truncate from end? from middle? preserve client_id suffix? Iter-1 (QUAL-406-003) flagged this; remediation added the truncation length but not the strategy. | Specify in Task 9b: "if total title > 200 chars, truncate `{rule_name}` from the end with ellipsis `…`; ALWAYS preserve the literal `AUTO: ` prefix and the literal `— {client_id}` suffix. Algorithm: `let max_rule_name_len = 200 - 'AUTO: '.len() - ' — '.len() - client_id.len() - '…'.len(); if rule_name.len() > max_rule_name_len { format!('AUTO: {}…— {}', &rule_name[..max_rule_name_len], client_id) }`." |
| SR-401-002 | LOW | C BC anchor semantic fit | S-4.01 | `anchor_capabilities: [CAP-017]` (frontmatter line 29) — BC-2.12.010 (state persistence) arguably also touches CAP-018 (storage); iter-1 finding QUAL-401-006 made the same observation as a LOW. Still single-anchored. | Optional: append `CAP-018` if BC-INDEX permits dual capability anchoring for storage-touching stories. Non-blocking. |
| SR-402-001 | LOW | F narrative crispness | S-4.02 | (Sampled — narrative still uses "As a Prism operations engine" voice rather than concrete user role per S-3.3.04 baseline). Iter-1 finding QUAL-402-004 unaddressed. | Optional reframe to "As a Prism analyst running recurring queries, I want differential results so I see only what changed between runs." Non-blocking polish. |
| SR-404-001 | LOW | E edge case sufficiency | S-4.04 | Iter-1 QUAL-404-002 suggested EC for clock-skew on `event_time` (record's event_time in future relative to now); not added. Replay/sensor-clock-drift scenario underspecified. | Add EC: future-dated `event_time` → included in window (rationale: replay tolerance for delayed sensor data). |
| SR-406-002 | LOW | M TV coverage | S-4.06 | BC-2.14.002 likely has TVs for the 12-transition table but Task 1 doesn't explicitly cite them. Iter-1 QUAL-406-005 unaddressed. | If BC-2.14.002 has TV-1..N, cite in Task 1 ("implements TV-1..N"). |
| SR-408-001 | LOW | G token budget realism | S-4.08 | Token Budget table (line 119): total ~34,300 = 26.8% of 128k context; iter-1 QUAL-408-007 flagged this as confused (28.8% / 30% budget reading); story re-clarified to "26.8% of a 128k-context agent window — within the 30% single-story budget." Now coherent. | (No action — this is now CLOSED. Listed for completeness; reclassified from MEDIUM in iter-1 to NON-FINDING in iter-2.) |
| SR-408-002 | LOW | L AC count vs scope | S-4.08 | 18 ACs across 18 tasks is reasonable but AC-12/13/14/15/16 (VP-NNN proofs pass) are each one-line formulaic. Iter-1 QUAL-408-009 suggested consolidation. | Optional consolidate VP-passing ACs to "All Wave 4 VPs (VP-044, 045, 046, 047, 143) pass under their declared methods." Non-blocking polish. |
| SR-407-001 | LOW | E edge case sufficiency | S-4.07 | (Sampled — story is otherwise solid.) AC-2 from iter-1 (BC-2.14.008 / evicted alert record fallback): no explicit eviction policy on alerts CF specified in this story. Iter-1 QUAL-407-002 suggested either cross-reference S-4.05 retention policy or document as future work. | Cross-reference S-4.05's retention policy in Previous Story Intelligence; or add a Gap Register entry. Story is approvable as-is. |

### KUDO findings (new in iter-2)

| ID | Severity | Story | Pattern | Why It Matters |
|----|----------|-------|---------|----------------|
| KUDO-W4-iter2-001 | KUDO | S-4.06 | ADR-017 §5 narrative — explicit citation of NIST 800-61 r2 (with footnote about r3 supersession) + ITIL v3 + Cortex XSOAR + Splunk SOAR for the 5-state machine choice (story line 124-128) | The state machine is a contentious design decision; tying it to four industry references with explicit r3-supersession footnote turns a 1898-curated decision into an auditable industry-informed choice. Excellent decision-rationale documentation that survives org-knowledge turnover. |
| KUDO-W4-iter2-002 | KUDO | S-4.08 | "Deferred Items" table at end of Phase 4.A Remediation Notes (lines 714-720) | Three items deferred (`keyring://`, `test_depends_on:`, WASM SDK) each with an explicit Reason and Resolution Target. Most deferred-item lists are graveyards; this one is a roadmap. Propagate to other stories that have OUT-OF-SCOPE notes embedded inline. |

---

## Per-Story Quality Verdict

| Story | Iter-1 Verdict | Iter-2 Verdict | Sizing Match | Quality Score |
|-------|---------------|----------------|--------------|---------------|
| S-4.01 | APPROVED_WITH_CONDITIONS | **APPROVED** | frontmatter=5, manifest=5 ✓ | EXCELLENT |
| S-4.02 | APPROVED | **APPROVED** | frontmatter=3, manifest=3 ✓ (manifest reconciled from 5→3) | EXCELLENT |
| S-4.03 | APPROVED_WITH_CONDITIONS | **APPROVED** | frontmatter=8, manifest=8 ✓ | EXCELLENT |
| S-4.04 | APPROVED | **APPROVED** | frontmatter=5, manifest=5 ✓ | EXCELLENT |
| S-4.05 | REQUEST_CHANGES | **APPROVED** | frontmatter=4, manifest=4 ✓ | EXCELLENT — major recovery |
| S-4.06 | REQUEST_CHANGES (sizing) / APPROVED_WITH_CONDITIONS (quality) | **APPROVED** | frontmatter=9, manifest=9 ✓ | EXCELLENT — major recovery |
| S-4.07 | APPROVED | **APPROVED** | frontmatter=3, manifest=3 ✓ | EXCELLENT |
| S-4.08 | REQUEST_CHANGES | **APPROVED** | frontmatter=9, manifest=9 ✓ | EXCELLENT — major recovery |

**All 8 stories now APPROVED.** Three stories that had REQUEST_CHANGES (S-4.05, S-4.06,
S-4.08) are now cleanly remediated.

---

## Cross-Cutting Quality Patterns

### Patterns RESOLVED in iter-2

| Pattern | Resolution |
|---------|-----------|
| Missing `tdd_mode: strict` (iter-1: all 8 stories) | All 8 stories now have `tdd_mode: strict`. (verified line-by-line) |
| Cycle-manifest ↔ frontmatter point disagreement (iter-1: 3 mismatches) | All 3 reconciled — manifest table at cycle-manifest.md lines 87-94 now matches frontmatter exactly across all 8 stories. |
| AC measurement methodology unspecified ("within Ns") | S-4.01 AC-4 (window-based), S-4.05 AC-7 ("on a CI runner"), S-4.08 AC-1/AC-3 (named clock origin "from `Instant::now()` at `broadcast::Receiver::recv()` returning, until first byte sent on TCP stream — measured on a CI runner"). |
| Pure/effectful split pattern | S-4.05 rate_limit and S-4.04 dedup not explicitly refactored to 9a/9b pattern, but Architecture Compliance Rule in S-4.05 (line 259-261) and S-4.04's CI-004 line in S-4.05 prose acknowledge the per-rule Mutex granularity — partial pattern propagation. |
| `Previous Story Intelligence` weakness | S-4.01, S-4.03, S-4.05 all have substantive Wave-3 pattern citations. S-4.07 still sparse but accepted as P0 (story is small). |
| Risk concentration on shared resources | S-4.01 + S-4.08 each explicitly state their 8-permit semaphore is private; ADR-013 §2.3 / ADR-016 §2.11 enforce. S-4.04 + S-4.05 both reference per-rule Mutex granularity on `detection_state` CF. |
| TenantId / OrgSlug residual references | All 8 stories now use `OrgId` consistently per ADR-006. (Sampled S-4.01 line 104, S-4.05 line 110, S-4.06 line 107, S-4.08 line 130.) |

### Patterns STILL OPEN (carry to Phase 4.B)

| Pattern | Affected | Notes |
|---------|---------|-------|
| Compound ACs (multiple assertions in one) | S-4.03 AC-9 (still 6 clauses; SR-403-001), S-4.06 AC-12 (3 clauses still), S-4.08 AC-3 (4 clauses still) | NON-BLOCKING. Decomposition is polish; current ACs are testable, just less granular. Fold into Phase 4.B burst. |
| EC sufficiency gaps | S-4.01 (clock skew across long suspension), S-4.04 (event_time future-dated), S-4.05 (broadcast panic, all-large snapshot) | NON-BLOCKING. Each story has 8+ ECs already; suggested additions are corner cases. |
| Test-fixture deps presented as build deps | S-4.08 (`depends_on: [S-6.11, S-6.12, S-6.13]`) | Phase 4.A Remediation Notes acknowledge this is deferred pending `test_depends_on:` schema field (KUDO-W4-iter2-002). NON-BLOCKING — flagged in Deferred Items table with explicit resolution target. |

---

## Bridge to Sibling (consistency-validator iter-2)

This iteration-2 review found:

1. **Cycle-manifest reconciliation is COMPLETE** — sibling consistency-validator should
   verify the manifest table at `.factory/cycles/wave-4-operations/cycle-manifest.md`
   lines 87-94 matches frontmatter `points` for all 8 stories. Quick spot-check during
   iter-2 confirms agreement; full consistency check is sibling's domain.

2. **All 6 new ADRs (013, 015, 016, 017, 018, 019) exist** under
   `.factory/specs/architecture/decisions/` — sibling should verify ADR cross-references
   from story bodies (e.g., S-4.01 cites ADR-013 §2.2, §2.3, §2.5, §2.6, §2.7, §2.8;
   S-4.06 cites ADR-017 §2, §3.4, §3.5, §3.6, §5; S-4.08 cites ADR-016 §2.1 through §2.12,
   ADR-019 §7, §8). Quality of the cross-references is high (semantic, not just
   frontmatter-only); sibling should validate the `§N.M` anchors actually exist in the
   ADR documents.

3. **`anchor_adrs` frontmatter is populated** in all 8 stories. Verify each story's
   `anchor_adrs` matches the ADR sections cited in the body.

4. **Convergent finding with consistency:** S-4.06 line 124-128 says transitions are in
   `prism-core/src/case.rs` lines 28-44 (`VALID_TRANSITIONS` const array). Sibling should
   confirm the actual prism-core file exists OR is in scope for this story / S-1.02.
   This is a code-existence claim that drift audit can verify.

5. **Divergent from consistency (this review only):**
   - SR-401-001 (graceful-shutdown AC missing) — quality concern, not consistency.
   - SR-406-001 (truncation algorithm unspecified) — quality concern, not consistency.
   - SR-405-001 (EC sufficiency for snapshot edge cases) — quality concern, not consistency.

**Recommended sequencing:**

1. consistency-validator iter-2 verifies frontmatter↔manifest agreement, ADR section
   anchors, and BC-INDEX traceability.
2. If both this review and consistency-validator return APPROVED / APPROVED_WITH_CONDITIONS
   with no HIGH findings, proceed to adversarial convergence.
3. Iteration-2 MEDIUM findings (4 total) and LOW findings (5 total) can be folded into
   Phase 4.B story-update bursts during implementation; not dispatch-blocking.

---

## Verdict Justification

**APPROVED_WITH_CONDITIONS** chosen because:

- **All 8 iter-1 HIGH findings are CLOSED cleanly with substantive, evidenced changes.**
  Verdict criterion for APPROVED ("ALL iter-1 HIGH closed cleanly, ≤2 new MEDIUM, 0 new
  HIGH") is partially met — but iter-2 surfaced 4 new MEDIUM findings (SR-401-001,
  SR-403-001, SR-405-001, SR-406-001), exceeding the ≤2 threshold for clean APPROVED.

- **All 4 new MEDIUMs are non-blocking polish items**, none of which prevents
  implementation. Each is precisely scoped with a concrete suggestion, and each can be
  folded into Phase 4.B story-update bursts with minimal churn.

- **0 new HIGH findings** — the remediation pass introduced no regressions; quality is
  monotonically IMPROVED.

- **Phase 4.A Remediation Notes sections** are substantive across all 8 stories (not
  stubs). Dimension N (iter-2 specific) PASSES — each remediation is documented with
  finding ID, severity, and resolution prose. S-4.06 and S-4.08 use a structured table
  format that should be propagated.

- **Re-pointed sizing justifications hold** — S-4.03 (5→8), S-4.05 (2→4), S-4.06 (5→9),
  S-4.08 (5→9) all have body content matching the new point complexity (task counts, VP
  counts, file counts proportional to sizing). Dimension O PASSES.

- **ADR cross-references are semantic, not perfunctory** — stories cite specific ADR
  sections (e.g., "ADR-013 §2.2 / R-4" for splay formula, "ADR-017 §3.4 + ADR-008" for
  cases CF key format). Dimension P PASSES.

**Conditions for full APPROVED (recommended pre-Phase-4.B):**

1. SR-403-001: Decompose S-4.03 AC-9 into AC-9a..9f (low-effort, high-value).
2. SR-406-001: Specify S-4.06 Task 9b title-truncation algorithm explicitly.
3. SR-401-001: Add S-4.01 graceful-shutdown AC.
4. SR-405-001: Add S-4.05 EC-009/010 for broadcast panic and all-large snapshot.

These four MEDIUM items together represent ~1-2 hours of story-writer work and would
move the verdict to clean APPROVED. They are NOT dispatch-blocking — adversarial
convergence can proceed in parallel.

---
