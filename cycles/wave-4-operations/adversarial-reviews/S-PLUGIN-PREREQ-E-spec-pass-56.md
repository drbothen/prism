---
review_id: S-PLUGIN-PREREQ-E-spec-pass-56
review_type: spec-adversarial
pass_number: 56
sequence_attempt: 9
streak_pre_pass: 1/3
streak_post_pass: 0/3
date: 2026-05-16
reviewer: vsdd-factory:adversary
related_state_decision: D-666
related_session_tasks: SESSION-D664-TASKS.md
fix_burst_id: FB44
fix_burst_committed: see-git-log
verdict: BLOCKED
findings_critical: 0
findings_high: 1
findings_medium: 0
findings_low: 0
observations_count: 0
novelty_score: HIGH
vectors_fired: [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]
vectors_yielding_findings: [5]
artifacts_reviewed_count: 21
---

# Adversarial Review — Pass 56 (PREREQ-E Spec, Phase 1d)

## Verdict: BLOCKED — 1 HIGH

Pass-56 surfaces a single structural HIGH finding: F-LP56-HIGH-001 — production call site for `mark_query_phase_started()` is unspecified, and the story's Architecture Compliance Rule forbids the only viable call site (`boot.rs`). The defect makes `E-PLUGIN-020` (post-boot write-tool registration rejection) structurally unreachable in production and reduces AC-9's third test to in-test-direct-invocation paper-fix territory (TD-VSDD-059 risk class).

Streak resets 1/3 → 0/3 (10th recurrence of the streak-reset pattern that has prevailed across the 9th 3-CLEAN sequence attempt).

## Findings Breakdown

| Severity | Count |
|----------|-------|
| Critical | 0 |
| High | 1 |
| Medium | 0 |
| Low | 0 |
| Observations | 0 |

## F-LP56-HIGH-001 — Production call site for `mark_query_phase_started()` is unspecified; story scope rule forbids the only viable call site

**Affected artifacts (pre-FB44):**
- `stories/S-PLUGIN-PREREQ-E-...md` v1.22 (lines 22, 132, 188–193, 357)
- `specs/architecture/decisions/ADR-026-sensorauth-unsealing.md` v1.14 (§D7, lines 296–298)
- `specs/behavioral-contracts/BC-2.16.012-plugin-registry-dispatch-migration.md` v1.16 (§Edge Cases EC-016-012-005)

**Severity:** HIGH
**Vector:** 5 (BC §Architecture Anchors completeness — surfaced as a lateral call-graph concern)
**Novelty:** HIGH — structural call-graph defect orthogonal to the cite-pin / phrasing-form / bullet-label / changelog-cell defect family that dominated passes 27–55

Task 7b directs the implementer to add `pub fn mark_query_phase_started()` and asserts the flag must be set "as the first act of step 8, before any QueryEngine construction proceeds." Step 8 is implemented in `crates/prism-bin/src/boot.rs`. The story's Architecture Compliance Rule forbade boot.rs modification AND `crates_touched` did not enumerate prism-bin — making the only viable production call site explicitly out-of-scope.

Consequences:
- No production caller exists for `mark_query_phase_started()`
- `QUERY_PHASE_STARTED` AtomicBool remains `false` for the lifetime of every production process
- E-PLUGIN-020 post-boot registration rejection branch is unreachable in production
- AC-9 third test passes via in-test direct invocation only — paper-fix territory

**Routing recommendation:** Architect + product-owner joint decision. Three options identified: (A) designate boot.rs as permitted single-line modification site, (B) move the call inside QueryEngine::new() with weakened temporal claim, (C) retire E-PLUGIN-020 and the AtomicBool.

## FB44 Closure (Pre-Persisted)

**Option chosen by architect at FB44:** A — designate boot.rs as permitted single-line modification site.

**Architect rationale:** The Architecture Compliance Rule forbidding boot.rs modification was authored in FB36 to prevent scope creep; Task 7b was added subsequently and introduced a boot-sequence obligation the rule did not anticipate. The rule is self-defeating in its current form — it simultaneously requires a flag to be set "before QueryEngine construction" AND forbids the only code location that executes before QueryEngine construction. Option A treats this as a wiring obligation belonging in boot.rs and adds a single designated insertion point per CLAUDE.md "wiring not redesign" Standing Rule 3 §4.

**FB44 staged edits (committed in this burst):**
- ADR-026 v1.14 → v1.15 — §D7 designates `prism_query::invalidation::mark_query_phase_started()` as production call site, invoked as first statement of step-8 in boot.rs before `QueryEngine::new()`
- BC-2.16.012 v1.16 → v1.17 — EC-016-012-005 designation; §Architecture Anchors and §Postconditions VP-156 row ADR-026 D7 pins advanced v1.10 → v1.15 (POL-23 sibling-sweep within architect's domain)
- VP-156 v0.8 → v0.9 — 4 live-narrative ADR-026 D7 pins advanced v1.10 → v1.15 (POL-23 sibling-sweep)
- Story S-PLUGIN-PREREQ-E v1.22 → v1.23 — Architecture Compliance Rule replaced (boot.rs MAY → ONE designated insertion); Task 7b appended with production-caller spec; AC-9 third-test rewritten to require invocation via public `mark_query_phase_started()` + WARN event emission assertion; `crates_touched` adds `prism-bin`
- STORY-INDEX v2.126 → v2.127 — row update + crates_touched column propagation
- BC-INDEX, ARCH-INDEX, VP-INDEX — derivative bumps (state-manager same-burst per POL-9)

## Per-Vector Trajectory

| Vector | Focus | Result |
|--------|-------|--------|
| 1 | Red Gate ↔ AC ↔ BC postcondition bijection | CLEAN |
| 2 | VP §Property Statement formal-logic precision | CLEAN |
| 3 | ADR §References ↔ Holdout Scenarios subsection completeness | CLEAN (ADR-026/027 lack explicit HS subsection — N/A applied) |
| 4 | error-taxonomy.md E-SPEC-NNN namespace audit | CLEAN |
| 5 | BC §Architecture Anchors completeness (broad call-graph audit) | **1 HIGH** — F-LP56-HIGH-001 |
| 6 | BC §Traceability L2 Invariants | CLEAN |
| 7 | Story §Token Budget cell consistency | CLEAN |
| 8 | VP §Verification Method ↔ proof-method dispatch | CLEAN |
| 9 | HS Preconditions / Source-of-Truth / Verification Method tri-coherence | CLEAN |
| 10 | STORY-INDEX ↔ story frontmatter ↔ BC-INDEX promotion-state coherence | CLEAN |

## Novelty Assessment

HIGH. The finding required call-graph reasoning ("trace `mark_query_phase_started()` from declaration to every required call site") which was not part of passes 27–55's vector rotation (those focused on syntactic cite-pin / phrasing-form / bullet-label / cell-count axes). FB37 architect adjudication tightened temporal narrative ("first act of step 8") but treated the call site as implicit. Subsequent passes accepted the post-FB37 narrative without re-deriving the call-graph.

## Confidence

HIGH. Evidence concrete (file paths + line numbers + direct quotes). Contradiction internal to story spec (Task 7b ↔ Architecture Compliance Rule ↔ `crates_touched`) mirrored in ADR-026 / BC-2.16.012. No external assumptions required.
