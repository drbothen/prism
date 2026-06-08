# Lessons Learned — wave-5-e-demo-fidelity

## Codification Candidates

### [recurrence] OCSF-CLASS-MIGRATION-001 cascade: cite-pin sweeps must be story-wide exhaustive, not incremental

**Date recorded:** 2026-06-06
**D-NNN anchor:** D-1032 (cycle-close discipline S-7.02; justified-deferred codification follow-up — not blocking OCSF merge)
**Story:** OCSF-CLASS-MIGRATION-001
**Tags:** [recurrence] [cite-pin-sweep] [sibling-sweep] [TD-VSDD-060] [streak-reset]
**Classification:** PROCESS-GAP — recurring cite-pin/sibling-sweep-miss class caused 2 streak resets in the same story's LOCAL cascade.

**Description:**

The OCSF-CLASS-MIGRATION-001 LOCAL cascade experienced 2 streak resets from the same root cause class:

- **Reset 1 at pass-6 (F-LP6-HIGH-001 HIGH):** Story-writer advanced BC-2.02.012 cite pins in an earlier burst (D-1026) by scanning for certain patterns. The sweep missed the `subsystems:` frontmatter field and justification comment bodies that referenced `SS-16` instead of the canonical `SS-02` (prism-ocsf is OCSF Normalization, not Spec Engine). Four sites were missed. Streak reset 2/3→0/3.

- **Reset 2 at pass-8 (F-LP8-MED-001 MED):** Story-writer advanced the BC-2.02.012 cite pin from v1.5→v1.6 in D-1026 by targeting the forward-looking cite sites. The trailing summary sentence `"BC is now v1.5"` in the frontmatter comment block (line 31) was not reached by the targeted sweep pattern. Streak reset 1/3→0/3.

Both resets involved the same root cause: the story-writer ran a **targeted/incremental cite-advance** (searching for specific known-stale patterns) rather than a **story-wide exhaustive sweep** (reading every occurrence of the BC name/version across the entire story file and adjacent spec files).

**Root cause of recurrence:**

Targeted sweeps (grep-and-replace on known patterns) are fast but incomplete. They find the sites the sweeper expected to exist but miss unexpected sites (comment bodies, trailing summary sentences, frontmatter fields) that hold the same staleness. The pass-8 fix-burst's response was correct: it performed a **full 35+ site version-pin audit** (22 BC-2.02.012 sites + 10 BC-2.01.013 sites), confirmed all present-tense pins, and explicitly distinguished historical hop narratives (immutable per TD-VSDD-091) from present-tense claims. Zero stale present-tense pins remained after the exhaustive sweep.

**Correct response (codified rule):**

When a story-writer performs a cite-pin advance (bumping a BC version reference in a story file), the sweep MUST:

1. Use a **story-wide exhaustive read**: read the full story file top-to-bottom, cataloging every occurrence of the BC name (e.g., `BC-2.02.012`) with its adjacent version string.
2. **Distinguish present-tense citations** (must be advanced) from **historical hop narratives** (immutable per TD-VSDD-091 — do not advance).
3. **Explicitly count and confirm**: record the total sites examined and the total sites advanced. e.g., "22 BC-2.02.012 present-tense pins confirmed v1.6; 10 BC-2.01.013 present-tense pins confirmed v1.14; 0 stale."
4. **Cross-check adjacent documents** in the story's perimeter (frontmatter comment blocks, justification comments, §Architecture Mapping tables, §Scope references) — these are frequently missed by pattern-search sweeps.
5. **Do not use grep-only as the primary sweep mechanism** for cite-pin advances when the story file is large or has multi-section structure. Read the file; grep is a verification step, not the sweep.

**Outcome:**

The pass-8 fix-burst (complete 35+ site audit) closed the recurring class. Passes 9/10/11 were all CLEAN(strict)=yes. LOCAL cascade CONVERGED.

**Codification direction (for future session-reviewer / VSDD process improvement):**

- This pattern should become a standing story-writer discipline: after any BC version bump, perform a full story read (not grep-only) and record the exhaustive count in the fix-burst commit message.
- Adversary SAP probe extension: after a cite-pin advance, adversary MUST grep the full story file for ALL versions of the BC name (including older version strings) and verify none remain as present-tense claims.
- Consider adding a POL amendment to POL-32 or a new process rule: "cite-pin sweeps must be story-wide exhaustive with explicit count record."

**Note:** This is a justified-deferred codification follow-up per Canonical Principle Rule 3. The codification work (SAP probe extension, possible POL amendment) requires session-reviewer adjudication. It does NOT block the OCSF-CLASS-MIGRATION-001 PR cycle.

---

### [process-gap] S-DEMO-003: Linux CI keyring crate requires libdbus-1-dev — provision proactively

**Date recorded:** 2026-06-08
**D-NNN anchor:** D-1054 (S-DEMO-003 PR-LEVEL 3-CLEAN convergence checkpoint)
**Story:** S-DEMO-003
**Tags:** [process-gap] [ci] [linux] [keyring] [libdbus]
**Classification:** PROCESS-GAP — missing OS-level dependency caused initial CI failure on every new story that touches the keyring crate.

**Description:**

PR #176 (S-DEMO-003) initial CI run failed on Linux because the `keyring` crate requires `libdbus-1-dev` at build time on Linux systems but the CI environment did not have it pre-installed. The fix (devops commit 566ae8a2) added the `apt-get install libdbus-1-dev` step to the CI workflow.

**Root cause:**

The `keyring` crate uses D-Bus for credential storage on Linux. This is a native OS-level dependency that Cargo/Rust cannot automatically install. The CI workflow did not include a step to provision this dependency before the build step.

**Correct response (codified rule):**

Any story that adds or touches crates with OS-level native library dependencies (particularly `keyring`, `secret-service`, or any crate that links against D-Bus, libssl, or other system libraries) MUST verify that CI workflows include the corresponding `apt-get install` step before `cargo build` / `cargo test`. Specifically:

1. When a story's `crates_touched` includes `prism-bin`, `prism-credentials`, or any crate that transitively depends on `keyring` or `secret-service`, the devops-engineer or implementer MUST add `libdbus-1-dev` to the Linux CI provisioning step if not already present.
2. This check belongs in the story's §Implementation Notes or §CI Considerations section so devops-engineer does not need to discover it at CI time.
3. Future CI scripts should include `libdbus-1-dev` as a standing baseline dependency for all builds that touch the credentials subsystem (SS-03).

**Outcome:**

Fixed by devops-engineer at commit 566ae8a2 before PR-LEVEL adversary cascade began. CI passed subsequently.

**Codification direction (for future session-reviewer / devops-engineer):**

- Add `libdbus-1-dev` as a standing baseline dependency in `.github/workflows/` for any job that builds `prism-bin` or `prism-credentials`.
- Story template (§CI Considerations) should prompt: "Does this story touch any crate with OS-level native library deps? List them and confirm CI workflow provisions them."

---

### [process-gap] S-DEMO-003: PR-LEVEL adversary must inspect the PR branch (not main checkout) for demo evidence to avoid false POL-10 findings

**Date recorded:** 2026-06-08
**D-NNN anchor:** D-1054 (S-DEMO-003 PR-LEVEL 3-CLEAN convergence checkpoint; pass-1 false-positive)
**Story:** S-DEMO-003
**Tags:** [process-gap] [adversary] [demo-evidence] [pol-10] [false-positive] [branch-inspection]
**Classification:** PROCESS-GAP — recurring class (same root cause as S-DEMO-CLAROTY-AUDIT-DTU-001 pass-4 F-PR4-MED-001 adjudicated FALSE POSITIVE, and DRIFT-D904-002 class).

**Description:**

During PR #176 (S-DEMO-003) PR-LEVEL adversary pass-1, the adversary raised a POL-10 demo-evidence-absent finding. The finding was adjudicated a FALSE POSITIVE: 31 demo-evidence files were committed and present at `docs/demo-evidence/S-DEMO-003/` on the PR branch at head `d1ddd00a`. The adversary globbed the main repository checkout on `develop`, which naturally did not contain the feature branch's demo-evidence directory.

**Root cause:**

The adversary used a glob or file-system read that resolved against the `develop` branch checkout (the main repository root at the time of dispatch) rather than the PR branch worktree. The demo-evidence files live on the feature branch and are not present on `develop` until the PR merges.

**Correct response (codified rule):**

When performing PR-LEVEL adversarial review for a story that includes demo evidence (POL-10 compliance check), the adversary MUST:

1. **Inspect the PR branch directly** — use `git ls-tree HEAD docs/demo-evidence/<story-id>/` on the feature branch, or read the files from the PR branch worktree (`.worktrees/<story-id>/`), NOT from the main repository root (which reflects `develop` HEAD).
2. **Verify with a git-based command**, not a filesystem glob that may resolve against the wrong working tree. Recommended: `git -C <worktree-path> ls-tree --name-only HEAD docs/demo-evidence/<story-id>/` or equivalent.
3. **If the worktree is no longer mounted** (story was merged and worktree cleaned), use `gh pr view <pr-number> --json files` or `git show <pr-head-sha>:docs/demo-evidence/<story-id>/` to confirm evidence was present at PR head.
4. If the adversary lacks direct worktree access, the orchestrator should confirm evidence presence via git and record the confirmation as an adjudication note before dismissing the finding.

**Outcome:**

The false positive was identified and adjudicated by the orchestrator. PR-LEVEL cascade continued. The process note was recorded in D-1054 and this lessons file. No cascade delay beyond pass-1.

**Codification direction (for future adversary agent prompt / POL-10 probe):**

- Adversary SAP extension: POL-10 demo-evidence probe MUST explicitly verify against `git ls-tree <pr-head> docs/demo-evidence/<story-id>/` (branch-anchored), not filesystem glob.
- This is the third recorded instance of this class: S-DEMO-CLAROTY-AUDIT-DTU-001 pass-4, S-DEMO-001 pass-1, and now S-DEMO-003 pass-1. Recurring class justifies a standing probe rule addition to the upstream adversary prompt.
- Consider adding a POL-10 amendment: "Demo-evidence presence check MUST be performed against PR branch HEAD via git-tree inspection, not filesystem glob."

---

### [high-value] S-DEMO-CLAROTY-TRAILING-SLASH-001: remove-uncertainty pre-delivery is high-value — caught the axum-0.7 Router::layer footgun before it silently no-oped in production

**Date recorded:** 2026-06-08
**D-NNN anchor:** D-1060 (S-DEMO-CLAROTY-TRAILING-SLASH-001 post-merge cycle-close)
**Story:** S-DEMO-CLAROTY-TRAILING-SLASH-001
**Tags:** [high-value] [remove-uncertainty] [axum] [middleware] [tower-http] [footgun]
**Classification:** HIGH-VALUE PROCESS IMPROVEMENT — remove-uncertainty pre-delivery caught 6 real defects in the story's own implementation guidance before TDD began.

**Description:**

`dclaude:remove-uncertainty` was applied to S-DEMO-CLAROTY-TRAILING-SLASH-001 v1.2 before dispatch, producing v1.3 with six corrections:

1. **axum-0.7 `Router::layer` runs AFTER routing (footgun):** The story originally specified `Router::new().route(...).layer(NormalizePathLayer::new())`, following a common but incorrect pattern. In axum 0.7 (and 0.8), `Router::layer` applies middleware AFTER routing resolves — meaning a path like `/api/v1/devices/` would fail to match `/api/v1/devices` BEFORE the normalizer could strip the slash. The correct pattern is to wrap the outer serve site: `axum::serve(listener, NormalizePathLayer::new().layer(router))`. This footgun would have passed all unit tests (which construct the Router correctly in isolation) while silently failing in the deployed server. The remove-uncertainty run caught this and corrected both serve sites.

2. **`trim_trailing_slash` is strip-only:** The story description called it "normalize" which could imply bidirectional (append AND strip). Corrected to "strip trailing slash only" to prevent incorrect assumptions in test-writer.

3. **tower-http 0.5 pin:** The story referenced `NormalizePathLayer` without pinning the crate version. Research confirmed tower-http 0.5 as the correct version compatible with axum 0.7. Pinned explicitly to prevent version drift.

4. **axum-0.8 dead-path removed:** A code path referencing axum 0.8 API surface was present despite the project being on axum 0.7. Removed.

5. **EC-002 ordering:** Auth-ordering clause clarified.

6. **Tags-route enumeration:** The tags route `/api/v1/tags` was not explicitly listed; enumerated to ensure completeness.

**Root cause:**

The story was authored before the remove-uncertainty workflow was formalized. The axum-0.7 middleware-ordering footgun is a documented gotcha in axum 0.7 migration guides but is not obvious from reading the API surface alone. Without external research validation, implementation guidance for middleware positioning is high-risk.

**Correct response (codified rule):**

Run `dclaude:remove-uncertainty` on EVERY Phase C story before TDD dispatch. This is now a standing user directive recorded in STATE.md `current_step` and SESSION-HANDOFF.md §Exact Next Steps. The directive was established at D-1059 and confirmed at D-1060.

The `Router::layer`-runs-after-routing footgun is a specific class of axum 0.7 middleware risk. For any story touching middleware placement in axum 0.7+, the test-writer MUST verify that middleware wraps the outer serve call, not the inner Router.

**Outcome:**

remove-uncertainty v1.3 corrections produced a correct first implementation. LOCAL cascade reached 3-CLEAN in 7 passes (P5/6/7 strict-clean). PR-LEVEL 3-CLEAN strict in passes 2/3/4. ADR-031 §D8-b Gap-CL-001 CLOSED.

**Codification direction:**

- Standing user directive: `dclaude:remove-uncertainty` before every Phase C story dispatch.
- axum middleware placement probe: adversary SAP extension for stories touching `Router::layer` — verify NormalizePathLayer (or any request-mutating middleware) wraps the `axum::serve` call, not the inner Router.

---

### [recurrence] S-DEMO-CLAROTY-TRAILING-SLASH-001: under-swept fix recurrence — comment/label fixes must do a crate-wide grep in the first fix-burst, not file-by-file

**Date recorded:** 2026-06-08
**D-NNN anchor:** D-1060 (S-DEMO-CLAROTY-TRAILING-SLASH-001 post-merge cycle-close)
**Story:** S-DEMO-CLAROTY-TRAILING-SLASH-001
**Tags:** [recurrence] [sibling-sweep] [TD-VSDD-060] [streak-reset] [comment-label-fix]
**Classification:** PROCESS-GAP — recurring sibling-sweep-miss class; same root cause as OCSF-CLASS-MIGRATION-001 lesson above.

**Description:**

The EC-003 "tags-route mislabel" finding surfaced in 3 consecutive LOCAL passes (P1 test-table, P2 test docstring line 276, P3 clone.rs:131 source comment) because each fix-burst swept only the file it was currently editing:

- **Pass 1 fix:** Corrected the test-table label in the story spec. Did not grep crate-wide.
- **Pass 2:** Adversary found the same stale label in a test file's docstring (line 276). Fixed. Did not grep crate-wide.
- **Pass 3:** Adversary found the same stale label in a source comment in clone.rs:131. Fixed.

Three streak resets from the same root-cause class.

**Root cause:**

Comment and label fixes have a high co-occurrence rate across src+tests within the same crate. A fix that changes a conceptual label (e.g., "EC-003 tags-route") that appears in test descriptions, source comments, and spec tables MUST perform a crate-wide grep in the first fix-burst to close all occurrences simultaneously.

**Correct response (codified rule — TD-VSDD-060 extension for comment/label fixes):**

When fixing a comment, label, or conceptual description that could appear in multiple files:

1. **First fix-burst MUST run a crate-wide grep sweep** across `src/**` and `tests/**` for the stale label/string before committing.
2. Fix ALL occurrences in one commit. Do not assume "I fixed the one the adversary pointed to; the rest are fine."
3. Record the sweep result in the fix-burst commit message: e.g., "grep EC-003 tags-route across crate: 3 occurrences found, 3 fixed (spec table, test docstring, source comment)."

This is an explicit extension of TD-VSDD-060 (sibling-site sweep on value changes) to cover comment and label changes — not just function signatures and constants.

**Outcome:**

After P3 the sweep was complete. P4/5/6/7 were all CLEAN(strict)=yes. LOCAL cascade CONVERGED.

---

### [process-note] S-DEMO-CLAROTY-TRAILING-SLASH-001: S-7.02 cycle-closing check — no process-gap follow-up story required

**Date recorded:** 2026-06-08
**D-NNN anchor:** D-1060 (S-DEMO-CLAROTY-TRAILING-SLASH-001 post-merge cycle-close; S-7.02 checklist)
**Story:** S-DEMO-CLAROTY-TRAILING-SLASH-001
**Tags:** [process-note] [s-7.02] [cycle-close] [process-gap]
**Classification:** PROCESS-NOTE — S-7.02 cycle-closing check result.

**Description:**

The LOCAL and PR-LEVEL adversary cascade findings for S-DEMO-CLAROTY-TRAILING-SLASH-001 were reviewed for `[process-gap]`-tagged items. Determination: all findings were **content defects** (incorrect implementation guidance, stale labels, missing test sweeps) rather than process-gap class issues that would require a follow-up story.

The two lessons above (remove-uncertainty high-value; under-swept fix recurrence) are codification entries, not follow-up stories. They document process improvements but do not require new stories because:

1. The remove-uncertainty directive is already a standing user directive (STATE.md current_step + SESSION-HANDOFF.md §Exact Next Steps).
2. The crate-wide sweep extension of TD-VSDD-060 is a discipline rule, not a story. It is codified here for future sessions.

**No follow-up story required from this cycle-close.**

---

### [process-note] S-DEMO-CLAROTY-TRAILING-SLASH-001: Red Gate evidence

**Date recorded:** 2026-06-08
**D-NNN anchor:** D-1060 (S-DEMO-CLAROTY-TRAILING-SLASH-001 post-merge cycle-close)
**Story:** S-DEMO-CLAROTY-TRAILING-SLASH-001
**Tags:** [process-note] [red-gate] [tdd]
**Classification:** PROCESS-NOTE — Red Gate baseline for record.

**Description:**

3 trailing-slash tests failed with HTTP 404 assertion errors against unmodified clone.rs (proper Red Gate — tests drove implementation). 3 regression guards passed at baseline. This confirms the Red Gate protocol was followed: tests were written first, observed to fail against the existing implementation, then code was written to make them pass.

---

### [process-note] S-DEMO-CLAROTY-TRAILING-SLASH-001: orchestrator cwd-drift hazard in verification commands

**Date recorded:** 2026-06-08
**D-NNN anchor:** D-1060 (S-DEMO-CLAROTY-TRAILING-SLASH-001 post-merge cycle-close)
**Story:** S-DEMO-CLAROTY-TRAILING-SLASH-001
**Tags:** [process-note] [orchestrator] [cwd] [worktree]
**Classification:** PROCESS-NOTE — orchestrator operational discipline.

**Description:**

During delivery, a verification command ran in the main repository root instead of the story worktree due to persisted shell cwd. This produced a misleading "file not found" result that required re-running the command with an explicit `cd <worktree>` prefix.

**Correct response:** Always pin `cd <absolute-worktree-path>` explicitly at the start of any verification command sequence when working across multiple worktrees. Do not assume shell cwd persists correctly between agent dispatches.

This is a low-severity orchestrator-process note. No streak reset. No spec impact.
