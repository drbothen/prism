# Session Review — PLUGIN-MIGRATION-001-D LOCAL Cascade (Option B Exit)

**Date:** 2026-05-22
**Cycle:** wave-0-plugin-prereqs
**Reviewer:** vsdd-factory:session-reviewer (claude-sonnet-4-6 adversary tier; cognitive diversity from primary cascade)
**Source inputs:**
- `.factory/cycles/wave-0-plugin-prereqs/lessons.md` entries 14-37+38
- `.factory/SESSION-HANDOFF.md` §RESUME SNAPSHOT 2026-05-22 §3
- `.factory/policies.yaml` POL-29 (v1.28)

## Cycle Summary

PLUGIN-MIGRATION-001-D ran a LOCAL impl adversary cascade of 12 passes + 11 fix-bursts, closing 49 substantive findings. The cascade was exited at pass-12 per user Option B (accept-with-codification) after 7 distinct POL-29 axis recurrences were documented. The rationale was sound: code correctness was verified since pass-8 ZERO-findings (all 3724 tests green, BCs/ADRs byte-fidelity confirmed), and the residual recurrences were purely documentation-pin-propagation with no semantic or runtime risk. The codification queue (35+ entries in lessons.md entries 14-37+38) is the structural deliverable.

Option B was NOT a defer-pattern. The 7 POL-29 recurrences were documentation hygiene in a regime where the implementation was demonstrably correct. Continuing for 3-CLEAN-ZERO convergence on purely doc-pin-propagation findings would have consumed ~3-5 additional pass/fix-burst cycles with zero improvement to runtime correctness, against a critical-path demo timeline. This is a legitimate "feature ordering is the only acceptable speed lever" application.

## Candidate Categorization

| Bucket | Count |
|---|---|
| POLICY amendment (POL-29) | 5 |
| POLICY amendment (BC-5.39.001) | 1 |
| AGENT-PROMPT amendment (adversary) | 2 |
| AGENT-PROMPT amendment (implementer) | 3 |
| AGENT-PROMPT amendment (story-writer) | 1 |
| AGENT-PROMPT amendment (orchestrator) | 1 |
| DOC update (CLAUDE.md TD-VSDD-091) | 1 |
| OBSERVATION (no codification action) | 6 |

Total actionable: 14 distinct proposals.

## GROUP A — POL-29 Amendments

### A1 (P2) — POL-29 step 8f crates/ scope
**Lesson:** 27, F-LP7-OBS-001
**Root cause:** POL-29 step 8f mandates sibling-sweep on INDEX files but does not name `crates/**/*.rs` doc-comment cite-pins as in-scope.
**Proposed delta — append to POL-29 step 8f:**
> "Step 8f EXTENSION (v1.29 — crates/ scope): when any burst bumps a BC or ADR frontmatter version (including no-semantic-change bumps), the sibling-sweep MUST include `rg \"<artifact-ID> v<old>\" crates/ --type rust` in addition to the `.factory/` sweep. If the running agent is PO (scope: .factory/ only), PO MUST surface the crates/ grep results to orchestrator with an explicit implementer handoff message in the SAME burst commit narrative. The implementer closes the crates/ cite-pins in the NEXT atomic commit. The burst is NOT considered complete until the handoff is documented, even if crates/ grep returns zero hits — the zero must be confirmed."
**Risk:** LOW — additive clause

### A2 (P2) — POL-29 step 3a registry pipe-table-cell variant
**Lesson:** 30, F-LP9-LOW-001
**Root cause:** Registry entry (c) for BC-2.16.002 did not enumerate `| <version-number> |` pipe-table-cell variant.
**Proposed delta — append to POL-29 step 3a registry entry (c) for BC-2.16.002:**
> "Additional variant form (d): `| <old_version> |` — pipe-table-cell without 'v' prefix, used in story body tables. Grep: `rg '\\| [0-9]+\\.[0-9]+ \\|' .factory/stories/` and visually confirm which cells reference BC-2.16.002."
**Risk:** LOW

### A3 (P2) — POL-29 transitive cite-pin chain (step 8j NEW)
**Lesson:** 32, F-LP10-LOW-001
**Root cause:** Steps 8a-8i handle direct downstream cites but NOT "A bumps → B cites A → C cites B → C is stale."
**Proposed delta — new step 8j after step 8i:**
> "Step 8j (v1.29 — TRANSITIVE CITE-PIN CHAIN): After step 8b finishes direct-downstream propagation for a bumped source-of-truth artifact X, state-manager MUST grep all artifacts that cite X by version AND check whether those citing artifacts are themselves cited by version elsewhere. Fixed-point iteration with MAX 3 transitive levels. Artifacts with §Changelog rows exempt per TD-VSDD-091."
**Risk:** MEDIUM — new machinery; first-application risk. **Recommend implementing as lint hook** rather than agent-executed procedure.

### A4 (P2) — POL-29 2nd-order propagation (step 8d supplement)
**Lesson:** 34, F-LP11-MED-001 (D-774)
**Root cause:** Step 8e fixed-point only iterates on original source-of-truth artifact. Secondary artifacts that gain a §Changelog row from a sweep should join the iteration queue.
**Proposed delta — amend POL-29 step 8d:**
> "Step 8d SUPPLEMENT (v1.29 — 2nd-order side-effect bump): when step 8b/8h applies a pin sweep to a NON-source-of-truth artifact B, and that sweep causes B's frontmatter to bump, state-manager MUST immediately add B's artifact ID to the step 8e iteration queue as if B were a source-of-truth artifact for this burst. Evidence: F-LP11-MED-001, 9-site BC-2.16.013 v1.14→v1.15 propagation gap, D-774."
**Risk:** LOW

### A5 (P3) — POL-29 self-2nd-order propagation (step 8i supplement)
**Lesson:** 38 (D-775)
**Root cause:** Within-body self-cite to own frontmatter version is uncovered. Single occurrence in cascade.
**Proposed delta — amend POL-29 step 8i:**
> "Step 8i SUPPLEMENT (v1.29 — within-file self-version-cite): when state-manager bumps any artifact's frontmatter `version:` field, grep the artifact's OWN body (non-§Changelog) for self-citations to the OLD version string. Patterns: `v<old_version>` bare, `version <old_version>`, `v<old_version>` in markdown bold/header context."
**Risk:** LOW

## GROUP B — BC-5.39.001 Amendment

### B1 (P1) — Convergence criterion strict/lenient disambiguation
**Lessons:** 28, 31, 33 (passes 7-9)
**Root cause:** Two concurrent convergence definitions caused communications mismatch — adversary marked CLEAN under lenient (CRIT+HIGH+MED zero), orchestrator dispatched fix-burst under strict (ANY-severity zero).
**Proposed amendment to BC-5.39.001 §Convergence Criteria:**
> "CLEAN for streak-advancement means ZERO findings of ANY severity (CRIT + HIGH + MED + LOW + OBS + PROCESS-GAP). Lenient criterion (zero CRIT+HIGH+MED) is acceptable as PR-merge gate threshold ONLY — it does not advance the 3-CLEAN streak. Adversary report must specify: 'CLEAN (strict): yes/no — CLEAN (PR-merge): yes/no'."
**Routing:** product-owner agent
**Risk:** LOW — clarification only

## GROUP C — Agent Prompt Amendments

### C1 (P1) — Adversary: tracing emission catalog completeness probe
**Lessons:** 16, 19 (2 recurrences)
**Proposed adversary probe addition:** Standing probe — grep `event_type =` in `crates/`. For each value, verify BC-2.16.002 §Postconditions row exists with field schema, audit role, recurrence policy. Same-commit catalog row required for emissions added in branch; removal-via-?-propagation exempt per D-765 precedent.

### C2 (P1) — Adversary: DTU↔TOML schema parity probe (sensor-spec stories)
**Lesson:** 24 (F-LP3 pass-3)
**Proposed adversary probe addition:** For each sensor TOML modified, read DTU clone's `types.rs` + `routes/<table>.rs`. Verify column name/type parity. TOML column with no DTU equivalent = P1 CRITICAL. DTU field with no TOML column = MEDIUM. Read source files; do NOT rely on story descriptions. **(Pass-3 caught 4 CRITICAL findings via this probe — without it they would have shipped as silent runtime failures.)**

### C3 (P1) — Implementer: no-ignored-test rationalization prohibition
**Lesson:** 17 (D-764)
**Proposed implementer prompt addition:** "When no failing test drives a spec-required behavior because integration tests are #[ignore]'d, this is NOT justification to defer. Add a unit test that drives the behavior without external dependency. 'Deferred to non-ignored test' is ONLY valid if a specific story and specific test name are identified."

### C4 (P2) — Implementer: cite-version-current discipline
**Lesson:** 23 body (F-LP4-HIGH-001)
**Proposed implementer prompt addition:** "Before authoring any commit citing a versioned spec artifact, READ the artifact's current frontmatter `version:` and cite THAT version. After fix-burst: grep for burst-start version to confirm zero live-narrative hits."

### C5 (P2) — Implementer: class-sibling sweep mandate
**Lesson:** 23/D-766 (F-LP3-HIGH-002/004)
**Proposed implementer prompt addition:** "When fixing any defect that is a member of a known class (copy-paste across N sensors, type-mismatch in N locations, missing field in N TOML blocks), execute workspace-wide grep for ALL OTHER INSTANCES before declaring complete. Report sibling sweep with pre/post counts in commit narrative."

### C6 (P2) — Story-writer + CLAUDE.md TD-VSDD-091: function-name anchors in task-body attestations
**Lesson:** 26 (F-LP6-LOW-001)
**Proposed story-writer amendment:** Task-body attestation cites use function-name or module-level anchors (e.g., `SpecLoader::load_all`), NOT line numbers.
**Proposed CLAUDE.md TD-VSDD-091 amendment:** Add bullet: "Task-body 'verified by reading the file' attestations ARE IN SCOPE — use function-name anchors, not line numbers."

### C7 (P3) — Orchestrator: parallel-burst git stash discipline
**Lesson:** 20 (D-765)
**Proposed orchestrator amendment:** When dispatching ≥2 parallel agents on the same worktree, include in each brief: "Before committing, run `git stash --keep-index` to isolate staged changes from in-progress unstaged changes. After commit, `git stash pop`."

### C8 (P2) — PO: BC-bump → ADR-cite routing
**Lesson:** 25/D-768 (2 recurrences)
**Option A (PO scope extension, MEDIUM risk):** PO advances ADR cite-pins that are PURELY version-string updates (no semantic change) in the same burst as BC bump.
**Option B (handoff message only, LOW risk):** PO enumerates ADR cite-pins in orchestrator handoff message with pre-drafted architect dispatch text.
**Recommendation:** Apply Option B immediately; route Option A to architect adjudication.

## GROUP D — DOC Update (combined with C6)

## GROUP E — Over-Codification Assessment

POL-29 at v1.28 has 9 step-8 substeps spanning ~4,500 words. The 7 recurrences in this cascade happened DESPITE v1.28 machinery. **Root cause is tooling, not policy.** Natural-language fixed-point grep procedures cannot be enforced reliably by agents.

**Long-term recommendation:** Create maintenance story for `.factory/hooks/validate-cite-pin-completeness.sh` — a pre-commit lint hook executing POL-29 step 8 grep logic deterministically. After hook ships, POL-29 steps 8a-8j become descriptive documentation, not executable agent instructions.

**Proposed story stub:** S-MAINT-POL29-HOOK-001 — implement validate-cite-pin-completeness.sh. Success: <5s cold workspace, exit-1 with specific stale cite-pin locations when violations exist, exit-0 when clean. Target wave: maintenance burst or Wave 5.

Amendments A1-A5 should still apply (they close real gaps), but they are tactical mitigation; the strategic fix is the lint hook.

## Priority Routing

### ROUTE NOW (recommended)

| ID | Priority | Action | Routing |
|---|---|---|---|
| C2 | P1 | DTU↔TOML schema parity adversary probe | adversary prompt owner |
| C1 | P1 | Tracing-emission catalog adversary probe | adversary prompt owner |
| C3 | P1 | No-ignored-test prohibition implementer prompt | implementer prompt owner |
| B1 | P1 | BC-5.39.001 strict/lenient disambiguation | product-owner → BC-5.39.001 |
| A1 | P2 | POL-29 step 8f crates/ scope mandate | policy-add or state-manager |

### QUEUE FOR MAINTENANCE BURST

A2, A3, A4, A5 (POL-29 amendments); C4, C5, C6, C7, C8 (additional agent prompt amendments); S-MAINT-POL29-HOOK-001 (lint hook story).

## TD Register Assessment

None of the 14 proposals qualify for TD entries per CLAUDE.md Canonical Principle Rule 3. All are immediately actionable spec/prompt amendments. The lint hook (S-MAINT-POL29-HOOK-001) is a future story, not a TD entry.

## File References (absolute paths)

- `/Users/jmagady/Dev/prism/.factory/cycles/wave-0-plugin-prereqs/lessons.md` — codification queue source
- `/Users/jmagady/Dev/prism/.factory/SESSION-HANDOFF.md` — §RESUME SNAPSHOT 2026-05-22 §3
- `/Users/jmagady/Dev/prism/.factory/policies.yaml` — POL-29 at v1.28
- `/Users/jmagady/Dev/prism/CLAUDE.md` — TD-VSDD-091 section
- `/Users/jmagady/Dev/prism/.factory/STATE.md` — pipeline state v7.463
