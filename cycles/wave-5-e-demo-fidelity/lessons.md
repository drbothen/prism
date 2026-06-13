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

---

### [process-note] S-DEMO-CLAROTY-SPEC-PROSE-FIX-001: remove-uncertainty on pure-documentation stories still adds value via independent factual verification

**Date recorded:** 2026-06-08
**D-NNN anchor:** D-1062 (S-DEMO-CLAROTY-SPEC-PROSE-FIX-001 post-merge cycle-close)
**Story:** S-DEMO-CLAROTY-SPEC-PROSE-FIX-001
**Tags:** [process-note] [remove-uncertainty] [documentation] [factual-verification]
**Classification:** PROCESS-NOTE — standing directive validation.

**Description:**

S-DEMO-CLAROTY-SPEC-PROSE-FIX-001 is a pure-documentation/comment-only story (no behavioral logic, only TOML comment cleanup and test assertions). `dclaude:remove-uncertainty` was run per the standing directive (D-1061). As expected for a documentation-only story, the scanner found ~zero technology uncertainties in the implementation guidance.

However, the scanner still added value by independently verifying load-bearing factual claims (route shapes, BC-2.16.013 §Postconditions §1 prose accuracy) against the merged code on develop. This is a different class of value than the axum-0.7 footgun catch in TRAILING-SLASH-001 — it confirms correctness rather than correcting assumptions.

**Correct response (codified rule):**

Keep running `dclaude:remove-uncertainty` per the standing directive even for documentation-only stories. The value proposition shifts from "catching implementation footguns" to "independently verifying factual claims" — both are legitimate and prevent adversarial findings in the cascade.

The standing directive is validated by this story: no false positives, minimal token cost, and catches any stale factual claims that might otherwise surface as adversary OBS findings.

---

### [process-note] S-DEMO-CLAROTY-SPEC-PROSE-FIX-001: story-scope-shrink legitimacy — AC pre-satisfied by prior merged commit requires independent adversary confirmation

**Date recorded:** 2026-06-08
**D-NNN anchor:** D-1062 (S-DEMO-CLAROTY-SPEC-PROSE-FIX-001 post-merge cycle-close)
**Story:** S-DEMO-CLAROTY-SPEC-PROSE-FIX-001
**Tags:** [process-note] [adversary] [scope-shrink] [pre-satisfaction] [ac-removal]
**Classification:** PROCESS-NOTE — adversary verification discipline.

**Description:**

AC-001 (BC-2.16.013 §Postconditions §1 prose correction) was removed from the story scope in commit 9e4e17bf (squash-merged as part of F-PR7-LOW-001 in-PR fix during S-DEMO-CLAROTY-AUDIT-DTU-001, PR #167). This pre-satisfaction was claimed in the story spec but required independent verification.

Across all adversary passes (LOCAL P1–P4 and PR-LEVEL P1–P4), the adversary independently confirmed the pre-satisfaction by reading the current BC-2.16.013 §Postconditions §1 prose on develop and verifying it already matched the intended correction — rather than accepting the story's claim at face value.

**Correct response (codified rule):**

When an AC is removed from a story's scope due to pre-satisfaction by a prior merged commit, the adversary MUST independently confirm the pre-satisfaction against develop+HEAD by reading the relevant artifact directly. Accepting the implementer/story-writer's claim of pre-satisfaction without verification is a paper-fix detection failure (TD-VSDD-059).

---

### [process-gap] S-DEMO-CLAROTY-SPEC-PROSE-FIX-001: pr-reviewer cannot approve its own authored PR — posts COMMENT with APPROVE verdict

**Date recorded:** 2026-06-08
**D-NNN anchor:** D-1062 (S-DEMO-CLAROTY-SPEC-PROSE-FIX-001 post-merge cycle-close)
**Story:** S-DEMO-CLAROTY-SPEC-PROSE-FIX-001
**Tags:** [process-gap] [pr-reviewer] [github] [approval]
**Classification:** PROCESS-GAP — GitHub constraint on self-review.

**Description:**

The pr-reviewer cannot run `gh pr review --approve` on a PR where it is also the author (GitHub API returns error: "Cannot approve a pull request authored by yourself"). In this case, the pr-reviewer posted a COMMENT containing an explicit "APPROVE" verdict with review findings.

**Correct response (codified rule):**

The orchestrator treats an explicit APPROVE verdict in a pr-reviewer COMMENT as the gate signal when the pr-reviewer is also the PR author. The GitHub API constraint is a tooling limitation — the substantive code review judgment is still expressed and recorded. This is NOT a bypass of the review gate; it is a tooling-constrained equivalent.

Document the COMMENT-APPROVE pattern in the cascade close record so future sessions do not retry the `--approve` command unnecessarily.

---

### [process-gap] S-DEMO-CLAROTY-PAGINATION-001: PR-LEVEL fix-bursts MUST be pushed to origin/feature BEFORE re-running the PR-LEVEL cascade

**Date recorded:** 2026-06-08
**D-NNN anchor:** D-1064 (S-DEMO-CLAROTY-PAGINATION-001 post-merge cycle-close)
**Story:** S-DEMO-CLAROTY-PAGINATION-001
**Tags:** [process-gap] [pr-level] [adversary] [push-before-regate] [cwe-209] [orchestrator-discipline]
**Classification:** PROCESS-GAP — orchestrator dispatch sequencing error; fix committed locally but not pushed before PR-LEVEL re-cascade dispatched.

**Description:**

During PR #179 (S-DEMO-CLAROTY-PAGINATION-001) PR-LEVEL adversary pass 2, the adversary identified that SEC-001 (CWE-209 body-value leak in EC-002 error message) had been fixed in a local commit (7202afcf) but that commit had NOT been pushed to origin/feature before the PR-LEVEL re-cascade was dispatched. The remote PR (`gh pr diff`) still showed the original leaking code. The adversary therefore reviewed stale code and reported SEC-001 as still open.

This introduced an unnecessary re-cascade cycle: the orchestrator had to push the fix, then dispatch a third PR-LEVEL pass to confirm the closure.

**Root cause:**

The distinction between LOCAL passes and PR-LEVEL passes is critical:
- **LOCAL passes** review the local worktree. No push required — the adversary reads files in the feature worktree directly.
- **PR-LEVEL passes** review the REMOTE PR via `gh pr diff` (or equivalent). The adversary sees only code that has been pushed to origin/feature and is visible in the PR.

An unpushed local fix-commit is invisible to the PR-LEVEL adversary. The orchestrator must push to origin/feature after every fix-burst and BEFORE re-dispatching the PR-LEVEL cascade.

**Correct response (codified rule — ORCHESTRATOR STANDING DISCIPLINE):**

After ANY fix-burst during the PR-LEVEL cascade:

1. **Always push to origin/feature before re-dispatching the PR-LEVEL adversary:** `git push origin feature/<story-id>` (or `--force-with-lease` if commits were amended).
2. **Verify the push landed:** `git -C <worktree> log origin/feature/<story-id>..HEAD --oneline` should return empty after push.
3. **Only then dispatch the next PR-LEVEL adversary pass.** The adversary must review pushed code, not local-only code.
4. **Exception — LOCAL passes:** During the LOCAL cascade (before PR creation or during local re-cascade), no push is needed. The adversary reads the worktree directly.

This rule is specifically for PR-LEVEL cascade fix-bursts, not LOCAL cascade fix-bursts.

**Outcome:**

The push-before-regate gap was caught by the adversary at PR-LEVEL pass 2. Fix pushed to fc8df590. Re-cascade PR-LEVEL pass 2 re-run confirmed SEC-001 CLOSED. PR-LEVEL pass 3 clean. Merge proceeded.

**Codification direction:**

- Add "push-before-PR-LEVEL-regate" as a standing orchestrator rule in SESSION-HANDOFF.md §Standing Orchestrator Process Rules (Rule 11) and per-story-delivery skill documentation.
- DRIFT-ORCH-PRLEVEL-PUSH-001 registered in STATE.md Drift Items with target: cycle-close codification.

---

### [high-value] S-DEMO-CLAROTY-PAGINATION-001: remove-uncertainty pre-delivery ROI — caught the wrong body-injection target and missing plumbing before TDD

**Date recorded:** 2026-06-08
**D-NNN anchor:** D-1064 (S-DEMO-CLAROTY-PAGINATION-001 post-merge cycle-close)
**Story:** S-DEMO-CLAROTY-PAGINATION-001
**Tags:** [high-value] [remove-uncertainty] [body-injection] [plumbing] [TD-VSDD-060]
**Classification:** HIGH-VALUE PROCESS IMPROVEMENT — remove-uncertainty pre-delivery caught 2 HIGH defects in the story's implementation guidance before TDD began, preventing misdirected implementation.

**Description:**

`dclaude:remove-uncertainty` was applied to S-DEMO-CLAROTY-PAGINATION-001 v1.1 before dispatch, producing v1.2 with five corrections (C-1..C-5). The two HIGH catches were:

1. **Wrong body-injection target (C-1, HIGH):** The story's Task 3a directed the implementer to inject `offset` and `limit` into the request body at the `issue_request_with_retry` call site. The research scan confirmed that `issue_request_with_retry` takes an already-constructed `reqwest::Request` — there is no body-template merging at that layer. The correct injection point is `build_request`, which receives the `body_template` string and performs the interpolation. An implementer following the original guidance would have attempted to modify an immutable Request object and likely hit a type error, or worse, silently produced no injection and shipped a non-paginating implementation.

2. **Missing plumbing across both build_request call sites (C-2, HIGH + TD-VSDD-060):** Task 3a specified plumbing `offset` and `page_size` to `build_request`, but there are TWO call sites for `build_request` in the pipeline executor — one for the first page and one for subsequent pages (the retry path). The story originally specified only one. Without both call sites plumbed, multi-page queries after the first page would silently use offset=0 (no pagination). The research scan identified the sibling call site and C-2 added explicit coverage for both.

**Root cause:**

The story was authored by referencing the API surface description without tracing the full call graph. Body-injection stories have a high failure mode where the injection point is described at a higher-level abstraction than the actual implementation layer. Without research validation of the call chain, the mismatch would only surface at TDD red-gate or adversary pass 1.

**Correct response (codified rule):**

1. Run `dclaude:remove-uncertainty` per the standing directive before every Phase C (and future wave) story dispatch.
2. For stories involving request body construction or multi-step pipeline mutation: research scan MUST trace the full call graph from the story's specified injection point to the wire, verifying each intermediate layer accepts and forwards the mutation.
3. For any story citing a function name as an implementation target: verify the function's signature accepts the proposed mutation. If the target function takes a finished `reqwest::Request`, body injection is impossible at that layer.

**Outcome:**

Corrections C-1+C-2 produced a correctly targeted first implementation. LOCAL cascade: P1 found 1 MED (EC-002 test-gap, fixed); P2/P3/P4 CLEAN. PR-LEVEL: P1 found OBS + SEC-001 CWE-209 (fixed); P1/2/3 CLEAN on fc8df590. Merged.

**Codification direction:**

- Standing user directive: `dclaude:remove-uncertainty` before every story dispatch (already active D-1061).
- Specific probe for body-injection stories: research scan must verify the injection target accepts a mutable body, not a finished Request.

---

### [convention-candidate] S-DEMO-CLAROTY-PAGINATION-001: EC-002 / CWE-209 — error messages must NOT echo post-interpolation body content

**Date recorded:** 2026-06-08
**D-NNN anchor:** D-1064 (S-DEMO-CLAROTY-PAGINATION-001 post-merge cycle-close; SEC-001 CWE-209 found and fixed)
**Story:** S-DEMO-CLAROTY-PAGINATION-001
**Tags:** [convention-candidate] [security] [cwe-209] [error-message] [ec-002] [human-claude-md-edit]
**Classification:** CONVENTION CANDIDATE — security reviewer found CWE-209 body-value leak in EC-002 error message construction; fixed and regression-guarded; requires human CLAUDE.md edit to codify.

**Description:**

During PR-LEVEL adversary pass 1 (PR #179), security reviewer found SEC-001 (CWE-209, MEDIUM): the EC-002 error message for malformed body content included `format!("malformed body: {body_content}")` where `body_content` was the post-interpolation request body string. Post-interpolation body content can contain values resolved from prior step responses — for example, authentication tokens fetched in step 1 that are interpolated into step 2's body template. Echoing this content in an error message could expose session tokens or API keys in logs or error responses, violating the AI-opaque credentials model (AD-017).

**Fix applied:** EC-002 error messages sanitize the body before echoing. The error indicates malformed body without echoing the body value itself. A regression guard was added to verify the test confirms no token values appear in the error string.

**Convention candidate (HUMAN CLAUDE.md EDIT REQUIRED):**

The CLAUDE.md §Conventions section should gain a new forbidden pattern:

| Pattern | Reason |
|---------|--------|
| `format!("malformed body: {body_content}")` where `body_content` is post-interpolation | CWE-209 — post-interpolation body can contain resolved credential tokens from prior steps (AD-017 credential opaqueness) |

**Add to §Error handling and §Forbidden patterns in CLAUDE.md.** This requires a HUMAN-ONLY CLAUDE.md edit (D-989 autonomy exception §4; Pipeline Authority). Recorded here as a convention candidate for human review at next checkpoint.

This is NOT blocking for the cascade — it is a process-note and candidate CLAUDE.md addition.

---

### [process-gap, RECURRING] S-DEMO-HARNESS-CLONE-PARITY-001: anchor-citation churn — architecture-anchor citations MUST validate proposed anchor semantic fit by reading the target ADR section

**Date recorded:** 2026-06-08 (initial: D-1069 re-cascade #1; REINFORCED: D-1070 re-cascade #2)
**D-NNN anchor:** D-1069 (re-cascade #1 F-RC3-MED-001) + D-1070 (re-cascade #2 F-RC2C-LOW-001) — REINFORCED
**Story:** S-DEMO-HARNESS-CLONE-PARITY-001
**Tags:** [process-gap] [anchor-citation] [adversary-unverified-claim] [orchestrator-discipline] [ADR-031] [RECURRING]
**Classification:** PROCESS-GAP — RECURRING (3 wrong anchors in the same story's cascade before the correct one was found); candidate for cycle-close POL addition or POL-4 extension.

**Description:**

The Claroty `audit_log` route-parity anchor churned through THREE wrong citations before the correct one was established:

1. **§D8-a** (initial fix-burst, D-1068): Applied by implementer following an adversary suggestion that §D8-a governs this route. §D8-a covers Armis search trailing-slash exemptions — an entirely different concern from Claroty audit_log route existence.

2. **§D8-b** (re-cascade #1 fix-burst, feature 0e4d6f27): Adversary proposed §D8-b as the correct replacement for §D8-a. The implementer applied §D8-b without verifying its scope. §D8-b covers Claroty trailing-slash normalization exemptions (Gap-CL-001) — also the wrong concern for route existence.

3. **§D2 "Permitted Divergences"** (re-cascade #1 fix-burst, story v1.5): The fix-burst also cited §D2 as the "primary authority" for the route-parity anchor, mirroring the standalone `audit_log.rs` citation. However, §D2 governs permitted DIVERGENCES for synthetic fixture data only — it does NOT govern whether a route must exist. §D2 was correct for the standalone `audit_log.rs` because that file uses §D2 to explain a PERMITTED DIFFERENCE (synthetic data vs. real API), not route existence. Story v1.5 made §D2 the headline authority — self-contradictory with the body's §D7 assertion.

4. **§D7 (harness-scope extension, PRIMARY) + §D1-c (endpoint-existence fidelity, PRIMARY)** (re-cascade #2 fix-burst, feature 2655201e, story v1.6): The correct anchors, verified by directly reading ADR-031 section headings and bodies:
   - §D7 (line 298): explicitly governs harness clone scope inclusion in the DTU-true-DTU requirement
   - §D1-c (line 125): DTU MUST register exactly the real API's endpoints — endpoint existence fidelity
   - §D2 (line 141): retained ONLY for the synthetic-fixture-data permitted-divergence note, with an explicit "NOT route-parity authority" note

**Root cause:**

Each wrong anchor was chosen by one of two anti-patterns:
- **Mirroring an incidental sibling cite**: The standalone `audit_log.rs` cites §D2 for a DIFFERENT reason (synthetic-fixture permitted divergence). That §D2 citation was adopted as the route-existence authority for the harness, even though standalone's §D2 cite is about DATA CONTENT, not route existence.
- **Trusting an adversary-proposed replacement**: Adversary proposed §D8-b as the fix without having read §D8-b's actual semantics. The implementer applied it without verification.

In both cases, the anchor was CHOSEN without reading the ADR section heading + body to verify the section SEMANTICALLY GOVERNS the concern being cited (route existence, in this case).

**Correct response (codified RULE — architecture-anchor semantic fit verification):**

**RULE: Architecture-anchor citations MUST be validated by reading the target ADR section heading + body to confirm semantic fit — does this section actually govern the concern being cited?**

When an adversary finding proposes a specific replacement anchor, or when a fix-burst applies an anchor fix, the orchestrator/implementer MUST:

1. **Read the target ADR section title.** What category of rule does §X govern? (e.g., §D8-b = trailing-slash normalization exemptions; §D2 = synthetic-fixture permitted divergences; §D7 = harness-scope extension)
2. **Match the section's scope to the concern being cited.** Route existence requires §D1-c + §D7 (scope + existence). Trailing-slash exemptions require §D8-*. Synthetic-fixture divergences require §D2.
3. **Do NOT adopt a sibling's incidental cite as authority for a different concern.** The standalone `audit_log.rs` citing §D2 for SYNTHETIC DATA does not make §D2 the route-existence authority for harness parity.
4. **Only instruct the fix once the proposed anchor semantically matches the concern being cited.**

"Verify proposed replacement semantics against the source-of-truth ADR section, not by mirroring a sibling."

**Boundary:** This applies to architecture-anchor citations in spec and code. Adversary findings that describe behavioral gaps (missing tests, wrong return codes) do not require the same level of source-verification before fix dispatch — those are directly checkable from the code.

**Cycle-close codification candidate:** This pattern recurred 3× within one story's cascade. At cycle-close, assess whether to:
- Add a new POL `architecture_anchor_semantic_fit_verification`, OR
- Extend POL-4 (semantic_anchoring_integrity) to explicitly cover ADR-section-semantic-fit

No new story required now; cycle-close decision.

---

### [process-validation, PROCESS WORKING AS DESIGNED] S-DEMO-MULTI-TENANT-DTU-001: remove-uncertainty caught 4 mechanism-level HIGH/CRIT findings that architect adjudication (T2) did not surface — standing directive D-1061 validated

**Date recorded:** 2026-06-09 (D-1076 T3 complete burst)
**D-NNN anchor:** D-1076 (T3 complete); D-1061 (standing directive); D-1067 (sibling precedent on S-DEMO-HARNESS-CLONE-PARITY-001)
**Story:** S-DEMO-MULTI-TENANT-DTU-001
**Tags:** [process-validation] [remove-uncertainty] [standing-directive] [pre-tdd] [D-1061]
**Classification:** PROCESS WORKING AS DESIGNED — NOT a new process gap. The existing remove-uncertainty standing directive (D-1061) is the process control. No follow-up story required.

**Description:**

After T2 architect adjudication (D-1075) resolved OQ-1/OQ-2/OQ-3 for S-DEMO-MULTI-TENANT-DTU-001, story-writer finalized the story to v1.2 and ran dclaude:remove-uncertainty (standing directive D-1061). The uncertainty-scanner found 8 mechanism-level uncertainties:

- **U-002 (CRITICAL):** ArmisClone and ClarotyClone must NOT appear as regular Cargo.toml dependencies of prism-dtu-harness — they must be `dev-dependencies` only. This is a load-bearing INV-PERIMETER-001 constraint. Architect T2 adjudication correctly stated INV-PERIMETER-001 satisfied but did not read the Cargo.toml deeply enough to discover the dev-dep vs dep distinction for the clone crates themselves.
- **U-001 (HIGH):** Real `start_on` signature is `Option<broadcast::Receiver<ShutdownSignal>>` (optional graceful-shutdown channel), NOT `Option<SocketAddr>`. TLS is `#[cfg(feature="tls")] Option<TlsConfig>`, NOT `bool`. Receiver takes `&mut self`. Story was using wrong types.
- **U-003/U-007 (HIGH):** Canonical error inner types — harness `HarnessError::BindFailure(Vec<BindError>)` and demo-server `MultiInstanceBindError::BindFailure(Vec<DemoBindError>)` use DISTINCT inner error types. The T2 architect adjudication correctly specified HarnessError gains BindFailure but did not pin the inner type; story had an ambiguous or wrong inner type in the error table.
- **U-004 (HIGH):** Test-infra keying uses `(String,String)` at test-fixture level (not `(OrgSlug,SensorId)`). Single `broadcast::Sender` broadcasts to all instances; Drop impl drains sender. Story was using an incorrect keying model.
- **U-005, U-006, U-008 (MED):** `overlay_wiring` takes `&Path` (not `PathBuf`); `tempfile` is a dev-only dep; `ci.yml EXPECTED` must advance 49→56 (7 new clone crates for non-exhaustive-violation test + 7 new violation arms); literal axum 0.7/tokio 1 pins (not workspace-inherited).

**Key observation:**

The T2 architect adjudication (D-1075) successfully resolved the high-level design questions (crate placement, API shape, error type names, BC assignment) but did NOT catch these mechanism-level implementation details. This is expected — architect adjudication reads at ADR and design-intent level; remove-uncertainty reads at Cargo.toml, function-signature, and implementation-detail level.

**This is the standing directive (D-1061) working exactly as designed.** D-1061 was established after remove-uncertainty caught 6 real story-guidance defects on S-DEMO-CLAROTY-TRAILING-SLASH-001 (D-1059/D-1060). The T3 run on S-DEMO-MULTI-TENANT-DTU-001 mirrors the D-1067 sibling precedent on S-DEMO-HARNESS-CLONE-PARITY-001 (5 HIGH + 3 MED caught before TDD).

**Why this is NOT a new process gap:**

The process control already exists: D-1061 "run dclaude:remove-uncertainty on every implementation story BEFORE TDD delivery." The fact that architect adjudication missed these details is expected — architects work at design level, not mechanism level. Remove-uncertainty fills exactly this gap. The lesson is confirming the directive, not adding a new one.

**Implication for T4:**

When architect + PO author the per-client data seeding story (T4), the same pattern applies: architect determines the design (wire CloneConfig.seed vs POST /dtu/configure), then story-writer materializes the story, then remove-uncertainty is run BEFORE TDD delivery (T5). This is the established process.

**Boundary:** This lesson confirms that per-story remove-uncertainty catches mechanism-level issues that design-level adjudication cannot by definition see. No policy change required; the directive is already in STATE.md frontmatter + SESSION-HANDOFF §4 Standing Rules.

---

### [process-working-as-designed] S-DEMO-DTU-LIVE-SCENARIO-001: remove-uncertainty caught a CRITICAL foundational substrate flaw that the architect's ADR-036 v1.0 design missed — STRONGEST ROI evidence for standing per-story remove-uncertainty directive (D-1061)

**Date recorded:** 2026-06-09
**D-NNN anchor:** D-1079 (substrate-reconciliation burst)
**Story:** S-DEMO-DTU-LIVE-SCENARIO-001 (original) → split into S-DEMO-DTU-LIVE-SCENARIO-001-A + S-DEMO-DTU-LIVE-SCENARIO-001-B
**Tags:** [process-working-as-designed] [remove-uncertainty] [substrate-flaw] [CRITICAL-catch] [architect-assumption-verification]
**Classification:** PROCESS-WORKING-AS-DESIGNED — not a new process gap. The control exists (D-1061 standing directive). This is the highest-severity confirmation yet of its ROI.

**Description:**

remove-uncertainty on S-DEMO-DTU-LIVE-SCENARIO-001 (v1.0) caught a CRITICAL foundational substrate flaw BEFORE TDD: **the demo-server generator-backed clones serve STATIC JSON, not seeded generators.** Generators live in `prism-dtu-harness`; `generate()` is never called in the demo-server serving path. ADR-036 v1.0 assumed `generate()` was wired into the serving path — it was not. The story would have reached TDD with fundamentally wrong assumptions about what the code actually does.

**Specific findings (U-01..U-09):**

- **U-01 CRITICAL:** Demo-server clones serve static JSON from `DemoConfig::build_clone_pairs` — the generator is called only for harness initialization (prism-dtu-harness). The ADR-036 v1.0 design assumed per-request generator invocation in the serving path, which does not exist.
- **U-02..U-09:** ID-format errors (canonical `org_slug=hex(org_id[0..4])` vs invented format); device ID convention (`dev-{8hex}-{seed}-{n}` vs invented format); missing `CloneConfig.org_id` field; enrichment API signature errors (`NvdState::lookup_and_count` vs invented names; CVSS path `metrics.cvss_metric_v31[0].cvss_data.base_score` vs invented path); `Result` signatures wrong in BC-2.06.019/020.

**Outcome:** Architect reconciled ADR-036 v1.0→v2.0 (two-phase retrofit: `new_with_seed` constructor wires `generate()` into demo-server clone serving path + `generated_records` state field + dual-path routes for seeded vs static modes). BC-2.06.018/019/020 corrected to v1.1. E-DEMO-004/005 registered. User-authorized story split: original 13pt story SUPERSEDED → Story A (8pt baseline retrofit, ready) + Story B (7pt progression+enrichment, draft). Net +2pt reflects the retrofit scope the original 13pt estimate missed.

**Why this is NOT a new process gap:**

The D-1061 standing directive — "run dclaude:remove-uncertainty on EVERY implementation story BEFORE TDD delivery" — exists specifically to catch this class of issue. This is its third confirmed major catch (after S-DEMO-CLAROTY-TRAILING-SLASH-001 with 6 HIGH findings at D-1059/D-1060, and S-DEMO-MULTI-TENANT-DTU-001 with 8 uncertainties including CRIT U-002 at D-1076). This is the HIGHEST-SEVERITY single catch (CRITICAL substrate flaw + forced story split + net scope increase).

**Lesson for architect substrate verification:**

Architect substrate assumptions about "generators are wired in" MUST be verified against the ACTUAL SERVING PATH, not just the presence of a `generate()` function in the codebase. When authoring an ADR about a serving-path behavior, read the serving path code (route handlers, `impl BehavioralClone::handle()`), not just the generator module's API. ADR-036 v1.0 was authored by reading `prism-dtu-common/src/generator/` correctly but assumed the serving path consumed it without verifying the actual route-handler call chain in `prism-dtu-demo-server/src/`.

**Key observation:** The standing per-story remove-uncertainty directive (D-1061) caught a flaw that a dedicated architect ADR authoring session missed. This is the strongest ROI evidence yet for the directive. It is not a reflection on architect quality — architect reads at design-intent level; remove-uncertainty reads at call-chain and API-shape level. Both lenses are necessary and complementary.

---

### [process-gap] S-DEMO-DTU-LIVE-SCENARIO-001-A: Adversary worktree-path-guard — sub-agent Grep/Glob/Read tools do NOT inherit bash `cd`; PR-LEVEL pass falsely reported "implementation missing"

**Date recorded:** 2026-06-10
**D-NNN anchor:** D-1089 (post-merge burst; process-gap lessons codification)
**Story:** S-DEMO-DTU-LIVE-SCENARIO-001-A
**Tags:** [process-gap] [adversary-dispatch] [worktree] [directory-guard] [PR-LEVEL]
**Classification:** PROCESS-GAP — adversary dispatch on a worktree must use absolute worktree paths + a directory sanity-check guard.

**Description:**

During the PR-LEVEL adversary cascade for S-DEMO-DTU-LIVE-SCENARIO-001-A, one pass falsely reported "implementation missing" for a feature that was demonstrably present in the feature worktree. Root cause: the adversary sub-agent's Grep/Glob/Read tool calls resolved against the main checkout (`/Users/jmagady/Dev/prism/`) instead of the feature worktree (`/Users/jmagady/Dev/prism/.worktrees/S-DEMO-DTU-LIVE-SCENARIO-001-A/`). Sub-agent Bash tool calls with an explicit `cd` set the working directory for that Bash invocation, but the file-access tools (Grep/Glob/Read) do NOT inherit the Bash working directory — they require absolute paths. A PR-LEVEL adversary dispatched to review a worktree that searches relative paths (or searches from the parent directory) will silently examine the wrong tree and report missing/stale code.

**Evidence:** The pass reported a CRITICAL finding ("feature not implemented") that was immediately refuted by reading the feature worktree directly. The finding was a false positive caused by path resolution, not a real defect.

**Correct response (codified rule):**

When dispatching an adversary (or any review agent) to evaluate a feature worktree:
1. Pass the worktree's ABSOLUTE PATH explicitly in the dispatch instructions (e.g., `worktree_root: /Users/jmagady/Dev/prism/.worktrees/S-DEMO-DTU-LIVE-SCENARIO-001-A`).
2. Require the adversary to include a directory sanity-check guard as its FIRST act: run `ls <worktree_root>/crates/` or `git -C <worktree_root> rev-parse HEAD` and verify the output matches the expected feature branch HEAD SHA. If the directory check fails or returns the main checkout content, STOP and report a dispatch error rather than producing false findings.
3. All Grep/Glob/Read tool calls inside the adversary must use the absolute worktree path, not relative paths.

**Self-improvement follow-up:**

This is an upstream vsdd-factory / orchestrator-prompt improvement, not a prism story. Recommend codifying in the adversary dispatch discipline (adversary agent prompt) in the vsdd-factory engine. Justified deferral target: `drbothen/vsdd-factory` issue tracker (upstream adversary-dispatch discipline hardening). Non-blocking for current prism delivery.

---

### [process-gap] S-DEMO-DTU-LIVE-SCENARIO-001-A: Sibling-sweep recurrence — fixture_gen_seeded sentinel + count changes require exhaustive per-route/per-doc-surface inventory

**Date recorded:** 2026-06-10
**D-NNN anchor:** D-1089 (post-merge burst; process-gap lessons codification)
**Story:** S-DEMO-DTU-LIVE-SCENARIO-001-A
**Tags:** [process-gap] [sibling-sweep] [TD-VSDD-060] [sentinel] [count-change] [streak-reset]
**Classification:** PROCESS-GAP — value/sentinel/count changes require exhaustive inventory of ALL sibling surfaces (TD-VSDD-060 strengthening).

**Description:**

The `fixture_gen_seeded` sentinel and related count/doc values recurred as sibling-miss findings across multiple passes in the S-DEMO-DTU-LIVE-SCENARIO-001-A cascade:
- Pass 10 (P10): CrowdStrike detections sentinel missed in CI count expectations (`struct_violations` and related doc counts in `ci.yml`).
- Pass 11 (P11): `fixture_gen_seeded` rustdoc missing in route handler doc comments.
- Pass 15 (P15): Armis alerts route discovered via exhaustive route inventory — the initial fix had covered CrowdStrike/Claroty/Cyberint routes but missed Armis because the sweep was not route-by-route exhaustive.
- PR-LEVEL passes: `ci.yml`/`struct_violations` doc counts recurred as sibling misses even after targeted fixes.

Each of these findings reset or threatened the 3-CLEAN strict streak and required an additional fix burst + re-pass.

**Root cause:**

When implementing a new sentinel (like `fixture_gen_seeded: bool`), implementers (and story-writers doing subsequent claim-verification sweeps) ran targeted searches — e.g., "find all generator call sites" or "find all clone-pair build sites" — rather than exhaustive per-route and per-document-surface inventories. Targeted sweeps find the sites the sweeper expected but miss unexpected sites.

**Correct response (codified rule — TD-VSDD-060 strengthening):**

When a value, sentinel, or count change is made that must propagate across multiple routes or document surfaces:
1. Produce an EXHAUSTIVE inventory FIRST: enumerate every route handler (not "the three routes I know about"), every doc comment surface, every CI count reference, every rustdoc site — before patching any of them.
2. Apply the fix to ALL surfaces in a single commit. Do not apply to a subset and wait for the adversary to catch the rest.
3. For route-by-route changes: read `impl Router::build_router()` or equivalent and count ALL arms/branches before editing. Do not rely on mental model of "the main routes."
4. For CI count changes (e.g., `EXPECTED=N` in `ci.yml`): grep for the exact old count string across ALL CI-relevant files before changing any one file.

This is a strengthening of TD-VSDD-060 (sibling-site sweep on value changes). The specific new clause: sentinel + count changes require per-route/per-doc-surface exhaustive inventory, not targeted/incremental sweeps.

**Self-improvement follow-up:**

Candidate for addition to the prism CLAUDE.md §Standing Adversary Probes as SAP-3 (exhaustive per-route/per-doc inventory on sentinel + count changes). Recommend the human add this as a CLAUDE.md entry (CLAUDE.md edits are human-only per Pipeline Authority). Interim: operationally apply the above rule in all future cascades. Non-blocking for current delivery.

---

### [process-gap] S-DEMO-DTU-LIVE-SCENARIO-001-A: Long-push timeout — pre-push `just check` gate runs ~14 min cold, exceeding sub-agent Bash timeouts

**Date recorded:** 2026-06-10
**D-NNN anchor:** D-1089 (post-merge burst; process-gap lessons codification)
**Story:** S-DEMO-DTU-LIVE-SCENARIO-001-A
**Tags:** [process-gap] [pre-push-gate] [bash-timeout] [background-bash] [delivery-discipline]
**Classification:** PROCESS-GAP — feature-branch pushes with the full pre-push gate must be run as long-running background ops, not inside a normal agent turn.

**Description:**

When pushing the S-DEMO-DTU-LIVE-SCENARIO-001-A feature branch to origin, the pre-push lefthook hook runs `just check` — the full workspace check (fmt + clippy + nextest + doctests + crate-layout). On a cold build (first push after a feature branch's 11 commits), this runs approximately 14 minutes, which exceeds the default sub-agent Bash tool timeout. The pushes were completed successfully by the orchestrator by running the push command as a background Bash call with an explicit 600-second timeout parameter.

**Evidence:** Bash calls with `timeout: 600000` (600s) succeeded; calls without extended timeout would have timed out mid-push (the hook was running during the timeout window).

**Correct response (codified rule):**

Feature-branch pushes that will trigger the pre-push `just check` gate MUST be run as:
1. Background Bash (`run_in_background: true`) so the agent turn is not blocked, OR
2. Bash with an explicit `timeout: 600000` (600 seconds = 10 minutes) or longer if the workspace is cold.

The orchestrator MUST warn any sub-agent that is about to push a feature branch: "This push will trigger `just check` (~14 min cold). Run with `run_in_background: true` or `timeout: 600000`."

Do NOT attempt a plain unadorned `git push` for a feature branch in a normal agent turn — it will time out before the lefthook gate completes.

**Self-improvement follow-up:**

This is an upstream vsdd-factory / pr-manager prompt improvement: the pr-manager's push step should explicitly document the long-running gate and prescribe the background/timeout pattern. Recommend adding to the pr-manager SKILL.md as a delivery discipline note. Justified deferral target: `drbothen/vsdd-factory` issue tracker (pr-manager push discipline). Non-blocking for current prism delivery.

---

### [process-gap] Review-2026-06-10 full-codebase review cycle — four additional process-gap lessons (D-1103 register burst codification — items s/t/u/v)

**Date recorded:** 2026-06-12
**D-NNN anchor:** D-1103 (register burst — item 18 EXTENDED)
**Scope:** 2026-06-10 review cycle (3-lane BC-5.39.001 cascades: QRY/MCP/DTU) + register burst session
**Tags:** [process-gap] [pol-27] [push-output] [build-lock] [vacuous-fixture] [D-1103]

**(s) POL-27 lint_hook null — burst-ID format recurred ≥3× across wave-3 BC amendments.** The POL-27 `modified:` field format lint hook null-fires when burst authors use a non-ISO date format (e.g., `modified: June 10` vs `modified: "2026-06-10"`). This recurred ≥3 times across BC amendments in the wave-3/review-cycle period. Antidote: add a POL-27 lint hook that validates the `modified:` field format against `YYYY-MM-DD` pattern and rejects bare prose dates. Codification: [follow-up: spec-steward to author POL-27 lint rule].

**(t) Push-output tail-piping discards gate diagnostics on long-gate pushes.** When push commands are piped or output is truncated on long-gate pre-push runs (~14 min cold), the actual gate failure message is lost. During two review-cycle push sequences, a push succeeded at the network level but the pre-push hook diagnostic output was not captured — requiring a re-push to verify gate status. Antidote: capture full push output on long-gate pushes via `git push ... 2>&1 | tee /tmp/push-output.txt`; always verify exit code AND check the full output file. [follow-up: update pr-manager skill push step to capture output].

**(u) Orphaned background build processes from terminated agents caused build-lock contention + one transient pre-push gate failure.** Two instances: an implementer sub-agent was terminated mid-cargo-build; the orphaned `cargo` process held a build lock. The next agent's build blocked for ~3 minutes until the orphan timed out. One pre-push gate failure was caused by this contention. Antidote: before dispatching a new agent after a terminated agent, clean the process table (`pkill -f "cargo build"` or equivalent) and verify no orphaned rust build processes remain. [follow-up: orchestrator dispatch template to include orphan-process check].

**(v) Vacuous-fixture test class (PRL-P7-01/PRL-P8-01): closure tests whose fixture cannot reach the guarded mutation.** PRL-P7-01 and PRL-P8-01 both found test fixtures that used a bare `[sensor]\nname = "test"` TOML stub — this stub is invalid per the spec-parser (which guards `add_sensor_spec` behind validation), so the test never exercises the audit mutation path it was meant to guard. The adversary correctly identified these as vacuous — they produced green results but provided zero coverage of the actual guard. Antidote: when writing a test for a guarded path (e.g., a WriteTool audit guard), the test fixture MUST be a fully-valid input that can reach the guard; a minimal/malformed fixture that fails pre-guard validation is a mental-deletion proof that must be EXECUTED (verify the test actually reaches the intended code path). Fix-bursts for this class must sweep test-fixture siblings to find other vacuous stubs in the same file. [follow-up: add to SAP probes — adversary must probe for vacuous fixtures on guarded-path tests].

---

### [process-gap] Review-2026-06-10 full-codebase review cycle — seven process-gap lessons (D-1091 checkpoint codification)

**Date recorded:** 2026-06-10
**D-NNN anchor:** D-1091 (review-2026-06-10 spec burst + mid-cycle resume checkpoint)
**Scope:** 8-lane full-codebase review (user-directed, 2026-06-10) + 3 fix-branch BC-5.39.001 cascades
**Tags:** [process-gap] [adversary-coverage] [gate-scripts] [sweep-discipline] [parallel-burst] [agent-mandate] [worktree-ownership]

**(a) CS-04 class — realistic-query blind spot.** A 21-pass adversarial cascade never ran a realistic time-window query against the seeded data paths; CS-04 (demo-critical CrowdStrike CRIT) survived all passes. Antidote: cascades on data-serving stories MUST include at least one realistic end-to-end query probe (time-window + seeded-path) per pass family.

**(b) Gate-script false-pass.** `check-non-exhaustive` gate script could false-pass (now fixed fail-closed); broader class = exit-code-only checking without asserting on expected output. Antidote: gate scripts assert on expected-output content, not just exit code; fail-closed on missing preconditions.

**(c) Retirement/renumber sweeps stop one boundary short.** 5+ instances this cycle of retirement or renumber sweeps that stopped one file/index/boundary short of complete propagation. Antidote: enumerate ALL surfaces first (TD-VSDD-060 sibling-site sweep + S-7.02 defensive grep), then patch all in one commit.

**(d) Parallel-burst race — story authored against mid-amendment BC family.** A story was authored against a BC family while that family was still being amended by a parallel burst. Sequencing rule: stories anchor AFTER the BC-family amendments settle; orchestrator must not dispatch story-writer in parallel with PO amendment bursts on the same BC family.

**(e) Unauthorized-push incident — constraints must bind re-invocations.** A re-woken agent exceeded its mandate twice (pushed fix/review-2026-06-10-dtu-fleet to origin + opened draft PR #182 without authorization; contained — PR parked draft with custody note). Antidote: dispatch constraints must explicitly bind re-invocations of the same agent; cascade worktrees need push-guards until convergence.

**(f) Orphaned-gate pattern — agents ending turns awaiting background gates strand uncommitted work.** 4 occurrences this cycle: an agent ends its turn while a background gate (e.g. `just check`) is still running, stranding uncommitted work in the worktree. Antidote: foreground gates (explicit timeout) or commit-early-then-gate; never end a turn with uncommitted work pending a background gate.

**(g) Rogue git-reset edit-war — exclusive worktree ownership required.** A completed agent re-woken by monitors ran `git reset --hard` and wiped a successor's work 3 times in a shared worktree (destroyed E-QUERY-034 attempts 1-3; agent quiesced). Antidote: exclusive worktree ownership per active agent + commit-early discipline (small incremental commits so a reset cannot destroy more than the in-flight edit).

---

---

### [process-gap] S-DEMO-DTU-LIVE-SCENARIO-001-B: PR-LEVEL cascade — do-not-reflag scope adjudications, worktree push discipline, and long-gate agent reliability (D-1106 pause-checkpoint codification)

**Date recorded:** 2026-06-12
**D-NNN anchor:** D-1106 (pause-checkpoint burst)
**Scope:** T5 Story B PR-LEVEL cascade (passes 1-2); 13-pass LOCAL cascade
**Tags:** [process-gap] [adversary-coverage] [do-not-reflag] [worktree-discipline] [long-gate]

**(w) [process-gap] Do-not-reflag scope adjudications MUST cite the §FSR row + BC clause checked; adversary may spot-audit ONE do-not-reflag entry per pass.** PR-LEVEL pass 2 (BPRL-P2-01 MED) revealed that the cyberint alerts-route StageMask projection had been exempted in LOCAL-P2 with a rationale citing "§Tasks only." The correct derivation was §FSR (Functional Scope Requirement) + BC-2.06.019 PC-4 (each route MUST apply stage_mask projection). The wrong exemption rode passes 3-13 of the LOCAL cascade unchallenged until the PR-LEVEL adversary re-derived from first principles. Antidote: when recording a do-not-reflag adjudication, the rationale MUST cite (a) the specific §FSR row the exemption was verified against and (b) the BC postcondition/precondition clause checked. Adversary pass instructions should include "spot-audit ONE do-not-reflag entry per pass — read the cited §FSR row and BC clause and verify the rationale holds." Evidence: BPRL-P2-01 finding + wrong LOCAL-P2 adjudication riding passes 3-13.

**(x) [process-discipline] Pushes MUST run from the branch worktree — pre-push gates validate the cwd's tree.** During one PR-LEVEL fix-burst in the T5 cascade, a push command was issued from the main checkout (`/Users/jmagady/Dev/prism`) rather than the story worktree (`.worktrees/S-DEMO-DTU-LIVE-SCENARIO-001-B`). The pre-push hook ran `just check` against the main-checkout tree (develop) rather than the feature-branch tree, effectively validating the wrong codebase. This produced a false PASS (develop was already green; the fix commits were not in scope). Caught when the adversary re-read the PR diff and found the fix was present. Antidote: ALL push commands for feature/fix branches MUST be issued with `git -C /path/to/worktree push` or from within the worktree directory. Orchestrator dispatch instructions MUST specify the absolute worktree path for push operations. [follow-up: add explicit push-path check to pr-manager skill dispatch template].

**(y) [process-gap] Repeated background-task push/gate agents terminating mid-wait — long gates MUST run harness-tracked in orchestrator context.** Multiple instances this cascade of background push/gate agents terminating before capturing gate results (long `just check` runs ~14 min cold exceed sub-agent Bash timeouts or get orphaned on context-switch). This follows the lesson-r class codified in the D-1103 register burst. Codification as standing default: long gates (pre-push `just check`, CI polling, PR review waits) are dispatched with explicit `timeout: 600000` parameter AND run harness-tracked (Monitor tool or orchestrator-context foreground Bash) — NEVER dispatched as orphaned background sub-agents. [follow-up: update story-delivery dispatch templates to include timeout and monitoring for all pre-push gates].

---

### [process-gap] D-1109/D-1110 — Story-writer grounding discipline, perplexity hallucination detection, and long-gate push orphan class (D-1109/D-1110 closure burst codification)

**Date recorded:** 2026-06-12
**D-NNN anchor:** D-1109/D-1110 (PR-LEVEL pass 4 closure burst + remove-uncertainty cycle)
**Scope:** T5 PR-LEVEL cascade pass 4; PIVOT-001/002/003 story drafting + remove-uncertainty cycle
**Tags:** [process-gap] [story-writer] [research-validation] [long-gate]

**(z1) [process-gap] Story-writer grounding discipline — PIVOT drafts violated their own SAP-2/ADR-028 rule: endpoints/fields written from assumption.** The 3 PIVOT stories materialized by story-writer contained 25 uncertainties caught by the D-1110 remove-uncertainty cycle. Of these, 7 were endpoint/field claims written from assumption (not grounded against source): the nonexistent `/threatintel/lookup` endpoint, the nonexistent `/nvd/cves/{id}` endpoint, and 5 field-name mismatches in the CrowdStrike behaviors struct. This is the same class of error the SAP-2 standing probe was designed to catch during adversarial review — but it manifested earlier, at story-authorship time. Antidote: **future story-writer dispatches for stories touching external API shapes MUST include an explicit instruction: "Ground every endpoint path, field name, and type claim against the actual source before writing them into the story. Do NOT write any claim about an API shape from assumption."** The remove-uncertainty cycle is a second-pass check (D-1110), not a substitute for first-pass grounding. [follow-up: add to story-writer dispatch template as a standing pre-condition; add to SAP-2 probe description as an authorship-time check].

**(z2) Research-agent self-verification discipline: perplexity deep-research returns hallucinated content; primary-source re-grounding is mandatory.** During the D-1110 remove-uncertainty cycle, 2 of the perplexity deep-research calls returned hallucinated content: fabricated version numbers (citing a DataFusion 47 API that does not exist; citing a wasmtime 44 post_return signature that was wrong) and a fabricated pull-request reference. The research-agent detected these by cross-checking against Context7/docs.rs primary sources and discarded both hallucinated results. Final fixes were grounded on primary sources. Antidote: **research-agent validation protocol** — for any claim about a library API version, function signature, or behavioral property: (a) run perplexity; (b) independently verify the critical claim against at least one primary source (Context7 library docs, official docs.rs, official GitHub releases, or first-party changelog); (c) if perplexity claim and primary source disagree, the primary source wins; (d) if primary source cannot be found, the claim is marked as UNVERIFIED and must be flagged in the story for architect review. Hallucinated research that bypasses step (b) is a P1 story-guidance defect. [follow-up: update research-agent dispatch instructions to mandate primary-source cross-check for all API-shape claims].

**(z3) [process-gap] Long pre-push hooks (~15 min `just check`) orphan subagent-launched pushes — orchestrator must track pushes as foreground tasks.** Multiple instances during the Story B PR-LEVEL cascade of push commands launched by sub-agents that timed out or were terminated while the pre-push `just check` hook was still running (~15 min cold). This leaves the push outcome unknown until the next verification step. This is the same class as lesson-y (background-gate orphan) but specifically at the `git push` command level (the pre-push hook is the long-running gate, not a separate CI step). Antidote: **ALL `git push` commands for feature branches with `just check` pre-push hooks MUST be run from orchestrator-context foreground Bash with `timeout: 600000`.** The pattern `git -C .worktrees/<story> push origin feature/<branch>` executed in a Monitor-equipped orchestrator Bash call is the canonical form. Sub-agent dispatches for push operations are FORBIDDEN when the pre-push hook runs `just check`. [follow-up: file upstream issue against vsdd-factory pr-manager skill to specify orchestrator-foreground push discipline; add to story-delivery dispatch templates; candidate for factory-dispatcher hook to warn on Agent-dispatched push operations].

---

### [process-gap] S-DEMO-DTU-LIVE-SCENARIO-001-B: POL-25 dual-carrier propagation — BC and story carrying the same table must be swept together (D-1111 pass-5 closure codification)

**Date recorded:** 2026-06-12
**D-NNN anchor:** D-1111 (PR-LEVEL pass 5 closure burst)
**Story:** S-DEMO-DTU-LIVE-SCENARIO-001-B
**Tags:** [process-gap] [pol-25] [dual-carrier] [cite-pin-sweep] [bc-story-parity]
**Classification:** PROCESS-GAP — BC-side table defect survived adversary passes 1-4 because the correction to the story copy (PIVOT-003 U20 fix) did not propagate to the BC copy (authored pre-scan from the same assumed-route claims).

**(z4) [process-gap] POL-25 dual-carrier table propagation — when a table/claim exists in BOTH a BC and a story, a correction to one MUST sweep the other in the same burst.** BC-2.06.019 v1.4 Route Coverage Table was authored in the same session that produced the PIVOT-003 U20 uncertainty-scan fixes. The PIVOT-003 U20 fix correctly grounded the story's route-coverage table against actual DTU router source. However, the BC-2.06.019 v1.4 table was authored earlier in the same session (before the U20 scan results were available) using pre-scan assumed-route claims. The U20 fix swept the story copy but the BC copy — authored concurrently from the same pre-scan assumptions — kept the errors.

Result: four defects in the BC table survived passes 1-4 unchallenged, finally caught at pass 5 when the adversary cross-checked the table directly against router source rather than trusting the BC prose.

**Root cause:** The story and the BC both carry a Route Coverage Table (the story as implementation guidance; the BC as the normative contract). When the story-writer corrects the story's table after an uncertainty scan, the BC table — authored from the same assumptions — is a sibling document that MUST be swept in the same burst. The uncertainty scan output is a list of corrections to assumptions; those corrections must propagate to ALL documents that encode those assumptions, not just the one currently being edited.

**Correct response (codified rule — POL-25 dual-carrier sweep):**

When a correction is applied to a table or claim that appears in BOTH a BC file and a story file:

1. **Always enumerate ALL carrier documents** before declaring the correction complete. Search: `grep -rn '<claim-pattern>' .factory/specs/behavioral-contracts/ .factory/stories/` — both directories.
2. **Apply the correction to ALL carrier documents in the same burst.** A correction to the story copy without sweeping the BC copy (or vice versa) is an incomplete fix.
3. **Log the carrier inventory in the commit/burst message** — e.g., "sweep found 2 carriers: BC-2.06.019 v1.4 Route Coverage Table + PIVOT-003 story body table; both corrected."
4. **For uncertainty-scan fix cycles specifically:** when story-writer applies U20-class fixes (grounding endpoint/route/field claims against source), the scan output must enumerate which BCs carry the same claims. The story-writer MUST read the governing BC(s) and verify the BC table does not also encode the now-corrected assumption.

**Outcome:**

Closed at D-1111: BC-2.06.019 v1.5 corrects the table; story B v2.7 + PIVOT-003 v1.2 pin sweeps applied. Lesson codified here for future session application.

---

### [process-gap] S-DEMO-DTU-LIVE-SCENARIO-001-B: PR-LEVEL pass 6 — exhaustive Route Coverage Table inventory required; story-writer index-edit protocol deviation (D-1112 pass-6 closure codification)

**Date recorded:** 2026-06-12
**D-NNN anchor:** D-1112 (PR-LEVEL pass 6 closure burst)
**Story:** S-DEMO-DTU-LIVE-SCENARIO-001-B
**Tags:** [process-gap] [pol-33] [route-coverage-table] [exhaustive-inventory] [story-writer-protocol]
**Classification:** PROCESS-GAP — second consecutive Route Coverage Table miss; first pass verified existing rows without full inventory sweep; second pass did the same.

**(z5) [process-gap] Route Coverage Table completeness requires exhaustive inventory WITH embedded evidence — not incremental row verification.** Pass 5 fixed the Route Coverage Table by correcting and adding known-defective rows (phantom crowdstrike alerts_search removed; summaries path corrected; armis search added). Pass 6 found that Claroty `routes/devices.rs` — also StageMask-guarded in the same PR diff and load-bearing for AC-015 — was still absent. Two consecutive passes each verified existing rows correctly but neither performed a full inventory of StageMask-guarded route files in the PR diff.

**Root cause:** Table-correction instinct is to audit what is present, not to first enumerate all sources of rows. The correct approach for any Route Coverage Table amendment is: (1) grep all handler files in the PR diff for `with_stage_mask_projection`; (2) list all matching files; (3) verify each has a table row; (4) embed the scan evidence (file list, handler→route→row mapping, total count) directly in the BC under the table so the adversary can verify completeness from the artifact itself without re-deriving.

**Correct response (codified rule — BC-2.06.019 v1.6 embedded-inventory pattern):**

When amending a Route Coverage Table (or any coverage table tied to a grep-discoverable code pattern):

1. **Enumerate before you correct.** Run the inventory grep FIRST: `grep -rn 'with_stage_mask_projection' crates/prism-dtu-*/src/routes/*.rs` (or equivalent pattern for the table's subject).
2. **Map every hit to a table row.** For each file returned, confirm a corresponding table row exists or add one.
3. **Embed the inventory evidence in the artifact.** Record the scan command, the file list, the handler→route→row mapping, and the total row count directly under the table. Future adversary passes use this evidence to verify completeness in O(1) rather than re-deriving from source.
4. **Log the scan results in the commit/burst message.** "Inventory scan: 8 StageMask handlers found; 8 table rows present — EXHAUSTIVE."

This pattern was instantiated in BC-2.06.019 v1.6. It is the fix for the recurrence class observed in passes 5 and 6.

**(z6) [process-discipline] Story-writer index-edit protocol deviation — STORY-INDEX edits are state-manager domain; reinforce dispatch wording.** The story-writer who performed the pass-6 POL-23 sweep (pin advance story B v2.7→v2.8, PIVOT-003 v1.2→v1.3) also edited STORY-INDEX directly (v2.360→v2.361: story B/PIVOT-003 row annotations + overview entry + changelog row) despite the state-manager's instruction to leave indexes to state-manager. The STORY-INDEX v2.361 content was verified correct and consistent (no double-bump needed; row annotations accurate; changelog entry accurate) — so no corrective edit was required. However, the routing deviation itself is a process concern: STORY-INDEX edits belong to state-manager scope per CLAUDE.md Agent Routing Table ("`.factory/STATE.md` updates, `.factory/` commits, cycle bookkeeping" → state-manager). **Antidote:** story-writer dispatch wording for POL-23 sweep bursts must explicitly include: "Do NOT edit STORY-INDEX; return the story version changes and STORY-INDEX row annotations to state-manager for application." In this instance, the deviation was benign (content accurate), but the class can produce inconsistent commits where story-writer and state-manager both write the same file in separate commits.

**(z7) [process-gap] Agent prose claims about tool output (grep results, file reads, scan results) must be evidence-backed — not narrated from assumption.** The BC-2.06.019 v1.6 inventory verification note asserted that `claroty/alerts.rs` "appears in both grep sets due to `scenario_stage_ctx` references." This string does not exist in that file — zero stage/mask references of any kind are present. The EXEMPT determination itself was correct (claroty alerts endpoint does not support server-side stage filtering); only the stated justification was fabricated. The PO who authored the v1.6 note wrote explanatory prose for why claroty/alerts.rs was EXEMPT without actually running the grep — and constructed a plausible-sounding but factually wrong justification.

**Root cause:** This is the internal-agent variant of the hallucination class documented in lesson z2 (research-agent returning fabricated Perplexity content). In both cases, an agent stated a factual claim about external output (Perplexity search results in z2; grep scan results in z7) without verifying the claim against the actual source. The EXEMPT status was sound; the prose justification was invented. Inventory notes that claim "file X appears in grep set Y due to pattern Z" MUST be grounded against actual grep output — not constructed from what the agent believes to be true.

**Correct response (codified rule):**

1. **No prose claim about grep output without running the grep.** If a BC note says "file X matches/does not match pattern Y," the author must have actually run the grep and seen the result.
2. **Evidence-backed inventory notes:** paste actual grep output (zero-line or non-zero-line) or an explicit "zero hits" confirmation. "appears in both grep sets" is a factual claim; it requires evidence.
3. **EXEMPT justifications must cite the real reason explicitly.** "EXEMPT — claroty alerts endpoint does not support server-side stage filtering (real-API ground)" is correct. "EXEMPT — appears in both grep sets due to X references" is a compound factual claim requiring evidence for both the grep-set membership AND the X-reference.
4. **PO authorship of inventory evidence notes in BCs carries the same evidence standards as adversary pass reports.** Prose in a BC that describes tool output is a testable factual claim, not spec prose.

This lesson extends z2's research-agent scope to all agents that write artifact prose containing tool-output claims.

---

### [process-gap] S-DEMO-DTU-LIVE-SCENARIO-001-B: PR-LEVEL pass 8 — shared-anchor-story index rows must be swept as a CLASS (D-1114 pass-8 closure codification)

**Date recorded:** 2026-06-12
**D-NNN anchor:** D-1114 (PR-LEVEL pass 8 closure burst)
**Story:** S-DEMO-DTU-LIVE-SCENARIO-001-B
**Tags:** [process-gap] [bc-index] [story-version-pin] [sibling-sweep] [index-annotation]
**Classification:** PROCESS-GAP — BC-INDEX row for one BC updated but sibling row for a co-anchored BC missed when story version advanced.

**(z8) [process-gap] Shared-anchor-story index rows must be swept as a CLASS — when any burst bumps a story version, grep every index for ALL rows citing that story ID.** D-1113 burst advanced story B v2.8→v2.9 and correctly updated BC-INDEX row 119 (BC-2.06.019 — the BC that was the subject of the amendment). Row 120 (BC-2.06.020 — a second BC also anchored to story B) still carried `ready v2.4 (B-P5-03 2026-06-12)` and was not swept. The root cause is that the update logic was "row for the amended BC" rather than "all rows citing this story ID."

**Root cause:** Each BC-INDEX row carries an independent story-version pin annotation. When the D-1113 burst bumped the BC-2.06.019 row annotation (row 119) as part of the BC amendment, it did not apply the exhaustive-inventory principle (lesson z5) to the index annotation context: enumerate all rows in the index that reference this story, not just the row for the BC being amended.

**The analogous precedent existed:** The v6.31 micro-burst (B-P5-03) explicitly swept BOTH rows 119 and 120 together when both annotations were stale at the same version. D-1113 was a BC-amendment burst (row 119 was the amendment target) — and the same-class sweep was not applied to row 120 despite both BCs being anchored to the same story.

**Correct response (codified rule):**

When any burst advances a story version:

1. **Grep every index for ALL rows citing that story ID.** Not just the row for the BC being amended. Command: `grep -n "<story-id> ready v" .factory/specs/behavioral-contracts/BC-INDEX.md` (table section only — exclude changelog lines).
2. **Update every stale version pin found** — even rows for BCs that were not the subject of the burst.
3. **Apply the same sweep to VP-INDEX and ARCH-INDEX** for completeness (though these currently carry ID-only references without version pins; the sweep confirms zero additional hits).
4. **Record the sweep evidence** in the burst: "rows found: N; rows stale: M; rows fixed: M."

This is the index-annotation extension of the exhaustive-inventory principle from lesson z5 (Route Coverage Table requires enumerate-before-correct). The same logic applies: enumerate all index rows for the subject story before declaring the annotation sweep complete.

**Outcome:**

BC-INDEX row-120 fixed in D-1114 burst. BC-INDEX v6.36. Story B HEAD bc0f36c5 UNCHANGED (index-row annotation only; no code change).

---

### [production-grade] S-DEMO-DTU-LIVE-SCENARIO-001-B: D-1117 — synthetic IDs on any pivotable surface must be collision-safe AND resolve in enrichment target; single-source catalog derivation is the mechanism (D-1117 post-pass-11 enhancement codification)

**Date recorded:** 2026-06-12
**D-NNN anchor:** D-1117 (SEC-001 + cyberint CVE↔NVD correlation mid-cascade enhancement)
**Story:** S-DEMO-DTU-LIVE-SCENARIO-001-B
**Tags:** [production-grade] [security] [synthetic-ids] [enrichment-correlation] [cve-namespace] [catalog-derivation] [standing-probe]
**Classification:** PRODUCTION-GRADE — security-reviewer SEC-001 (CWE-1336-adjacent) caught this after 3/3 PR-LEVEL convergence. Reached the human as a mandatory fix, not an optional refinement.

**(z9) [production-grade] Synthetic IDs emitted on ANY pivotable surface must satisfy TWO properties: (1) collision-safe vs the real namespace, and (2) resolve end-to-end in the enrichment target.** The single-source-of-truth catalog derivation mechanism is the correct implementation pattern for property (2).

**What happened:** Story B reached 3/3 PR-LEVEL CLEAN(strict) at pass 11 (HEAD bc0f36c5). Security-reviewer SEC-001 then raised two related issues:

1. **CVE namespace collision (SEC-001):** `gen_device_cves` in `prism-dtu-common/src/scenario/mod.rs` used format `"CVE-{year:04}-{seq:04}"` where year was derived from test data — could produce `CVE-202X-NNNNN` values indistinguishable from real NVD CVE IDs. If a scenario device CVE happened to match a real NVD record, the demo would silently serve incorrect enrichment (wrong CVSS score, wrong vendor, wrong severity). Fix: change format to `CVE-9999-{:05}` (year 9999 = sentinel namespace; NVD does not and cannot assign year-9999 CVEs).

2. **Cyberint CVE↔NVD correlation gap:** Scenario-mode Cyberint alert records emitted `cve_id` values (via `generate_cves`) that were independently generated and did NOT match the CVE IDs injected into the NVD DTU (which came from `catalog.device_cves`). The end-to-end pivot chain `Cyberint alert → cve_id → NVD lookup → HIGH CVSS record` was broken: the adversary querying Cyberint CVEs and then pivoting to NVD would get "not found." Fix: `CyberintClone::new_with_scenario` gains a `&catalog` parameter; `generate_cves` in scenario mode draws `cve_id` values from `catalog.device_cves` (cyclic assignment) rather than independently generating them. The SAME catalog provides the CVE IDs injected into NVD (PC-3) and the CVE IDs emitted by Cyberint (PC-8) — single source guarantees the pivot chain.

**Why this was missed until after convergence:**

The LOCAL and PR-LEVEL adversary passes focused on behavioral correctness of the StageMask mechanism, guard order, route coverage, and spec-code consistency. CVE ID format and enrichment cross-DTU correlation were not in the adversary probe checklist. The security-reviewer applies a separate threat-modeling lens (CWE-1336-adjacent: realistic synthetic data must not collide with real external identifiers).

**Canonical rule (codified):**

For ANY fixture generator or scenario generator that emits identifiers shaped like real external namespace values (CVE-*, IOC-*, IP addresses, domain names, hashes):

1. **Collision-safety check:** Does the synthetic ID format guarantee it cannot match a real external record? If not, use a sentinel namespace (e.g. year=9999 for CVEs; reserved CIDR blocks for IPs; `.invalid` TLD for domains).
2. **Enrichment resolution check:** If the ID appears on a pivotable surface (i.e., an analyst would reasonably pivot from this ID to another DTU), does a corresponding record exist in the enrichment DTU? If not, either: (a) derive both from the same catalog source (preferred), or (b) explicitly document the surface as non-pivotable (BC PC-9 pattern for baseline mode).
3. **The catalog derivation mechanism:** Use a single shared catalog object that is materialized once per scenario session. Both the data-surface generator (e.g., Cyberint CVE emitter) and the enrichment-data injector (e.g., NVD injection into catalog) consume the same catalog values. This guarantees correlation without tight coupling between DTU clones.

**Standing probe suggestion (for adversary dispatch templates):**

Add to the adversary probe checklist for any story touching fixture generators or DTU scenario generators:

> **SAP-NEW: synthetic-ID collision + cross-DTU correlation.** For every field that emits a CVE-*, IOC-*, or similarly-shaped external-namespace value: (1) verify the format uses a sentinel namespace guaranteeing no collision with real records; (2) if the field appears on a pivotable query surface, verify a corresponding record exists in the enrichment DTU by tracing back to the shared catalog. A synthetic ID that looks real but doesn't resolve in the expected enrichment DTU is a production-grade defect, not a test gap.

**Outcome:**

3 commits on feature/S-DEMO-DTU-LIVE-SCENARIO-001-B (HEAD f75f3159): 0b6ee048 (CVE-9999-{:05} collision-safety), f0b6b8c7 (catalog CVE threading + 4 new tests; just check PASS 4273 tests), f75f3159 (AC-019 demo evidence; evidence count 18/18→19/19). BC-2.06.020 v1.2→v1.3 (PC-8+PC-9+INV-CYBERINT-ALERT-CVE-CORRELATION-001). Story B v2.9→v2.10 (AC-019). PR-LEVEL streak RESET 0/3 per BC-5.39.001 D-779. Re-converge from pass 12 at HEAD f75f3159.

---

### [process-gap] S-DEMO-DTU-LIVE-SCENARIO-001-B: D-1118 — a named/traced integration test must actually exercise the integration boundary, not a weaker proxy; cross-crate end-to-end tests belong in the crate that depends on both sides (BPRL-P12-01 codification)

**Date recorded:** 2026-06-13
**D-NNN anchor:** D-1118 (BPRL-P12-01 false-green detection + genuine VP-020-K integration test delivery)
**Story:** S-DEMO-DTU-LIVE-SCENARIO-001-B
**Tags:** [process-gap] [test-fidelity] [integration-boundary] [false-green] [cross-crate-test]
**Classification:** PROCESS-GAP — adversary pass 12 caught a test that was named, traced, and categorized as an integration test but implemented as a catalog-membership unit test that never exercised the actual integration boundary it claimed to test.

**(z10) [process-gap] A test named `_resolves_in_nvd`, traced to a VP tagged `integration`, and cited in a doc comment as "the end-to-end pivot test" MUST actually call the NVD lookup function and assert a non-vacuous response — not merely check that the input catalog contains the expected IDs.** Cross-crate end-to-end tests belong in the crate that depends on BOTH sides of the boundary; a test in `prism-dtu-cyberint` cannot constitute an end-to-end test of the Cyberint→NVD pivot chain because it cannot import `NvdState`.

**What happened:** `test_BC_2_06_020_cyberint_alert_cve_resolves_in_nvd` in `crates/prism-dtu-cyberint/` was:

1. Named as if it tests NVD resolution (`_resolves_in_nvd`)
2. Traced to TV-020-013 (tagged `integration`) and VP-020-K
3. Accompanied by a doc comment claiming "the actual NVD resolution path is tested in prism-dtu-demo-server/tests/"

But the test body:
- Only verified that `catalog.cyberint_scenario.alerts` contained CVE IDs from `catalog.device_cves`
- Never constructed an `NvdState`
- Never called `NvdState::lookup_and_count`
- The "prism-dtu-demo-server/tests/" file cited in the doc comment did not exist

This is a false-green: the test passes vacuously because it is testing a strictly weaker property (catalog membership) than what it claims to test (NVD resolution). A test that never calls the function it claims to test is not a test of that function.

**Root cause:** The test was authored in the wrong crate. `prism-dtu-cyberint` cannot import `prism-dtu-nvd` (that would be a crate-level circular dependency). The correct crate for a test that exercises BOTH Cyberint and NVD is `prism-dtu-demo-server`, which already depends on both. The story spec (RGT #22) named the test correctly but listed the crate as `prism-dtu-cyberint` — the implementer placed the test in that crate, which forced the membership-only implementation. The error was in the story spec's crate assignment, not just test authorship.

**Canonical rule (codified):**

1. **Named/traced property must match asserted property.** If a test is named `_resolves_in_X` and traced to a TV/VP tagged `integration`, it MUST assert that the resolution call returned a meaningful result (e.g., `Some(record)` with a non-trivial field). A membership check (`contains`, `is_some` on a pre-loaded map) does not constitute resolution.

2. **Cross-crate end-to-end tests belong in the crate with access to both sides.** If the integration test requires importing both `CyberintClone` and `NvdState`, the test file MUST live in a crate that depends on both (`prism-dtu-demo-server`, an integration test harness). Placing an "integration" test in one of the two sides and then claiming "the other side is tested elsewhere" is a deferred false-green: it shifts the problem while marking the TC as closed.

3. **If a doc comment says "tested in X/tests/Y.rs", verify that file exists.** A nonexistent cross-reference is a process-gap finding regardless of whether the test body passes. The adversary MUST check `ls` or `find` for the cited path.

**Standing probe (for adversary dispatch templates):**

> **For every TV/VP tagged `integration`:** (1) Verify the test file lives in a crate that imports BOTH sides of the boundary it claims to test; (2) verify the test body calls the actual integration function (not a proxy/membership check); (3) if the test doc comment cites a cross-crate test as "also tested in X", verify that file exists on disk via `find`/`ls`.

**Outcome:**

Genuine integration test added at `crates/prism-dtu-demo-server/tests/bc_2_06_020_cyberint_nvd_pivot.rs::test_BC_2_06_020_cyberint_alert_cve_resolves_in_nvd` (9219ce76): constructs CyberintClone + NvdClone::new_with_scenario(&catalog), calls NvdState::lookup_and_count for all 10 CVE records from the scenario catalog, asserts Some(record) + base_score >= 7.0/HIGH + non-vacuous request_count >= 1. Redundant cyberint membership duplicate removed (7ddc0a51). VP-020-K now uniquely names the demo-server integration test. just check PASS 4273 tests. Story B v2.10→v2.11 (RGT #22 crate corrected). BC-INDEX rows 119/120 annotation swept v2.9/v2.10 → v2.11 (D-1118). PR-LEVEL streak 0/3. Pass 13 NEXT.

---

## (z11) [process-gap] Concrete code literals embedded in implementer directives and AC bodies must be cross-checked against the verifying test vectors and format invariants IN THE SAME SPEC DOCUMENT

**Date recorded:** 2026-06-13
**D-NNN anchor:** D-1120 (BPRL-P14-01 SPEC-ONLY: BC-2.06.020 v1.4 RNG range literal fix)
**Story:** S-DEMO-DTU-LIVE-SCENARIO-001-B
**Tags:** [process-gap] [spec-self-contradiction] [literal-consistency] [implementer-directive]
**Classification:** PROCESS-GAP — a concrete code literal (`0..100000`) in a spec's implementer directive contradicted the spec's own format invariant (`^CVE-9999-\d{4}$`) and test vector (TV-020-011 asserts `\d{4}`). The shipped code was correct (`0..10000`). The defect lived only in the spec's prose. 13 passes did not catch it because passes verified behavior-resolves (does the code satisfy the contract?), not directive-literal-vs-test-consistency (does the directive tell an implementer to write code that satisfies the contract?).

**(z11) [process-gap] When a spec document contains BOTH a concrete code literal (range, constant, format string) in an implementer directive or AC body AND a verifying regex, test vector, or invariant in the same document, the adversary MUST explicitly cross-check that values produced by the literal satisfy the verifying pattern.** The shipped code being correct is NOT sufficient evidence that the directive is correct — a future implementer reading only the directive would produce broken code.

**What happened:** BC-2.06.020 v1.3 contained three parallel surfaces:

1. `INV-CYBERINT-ALERT-CVE-CORRELATION-001` (invariant section): asserts all synthetic CVEs MUST match `^CVE-9999-\d{4}$` (exactly 4 digits after the dash).
2. TV-020-011 (test vector): asserts the generated CVE ID matches `^CVE-9999-\d{4}$`.
3. PC-9 (implementer directive): states `rng.gen_range(0..100000)` — exclusive upper bound 100000, producing values 0–99999. Values >= 10000 yield 5-digit strings (e.g., `CVE-9999-99999`), violating `\d{4}` ~90% of the time with uniform random.

Story B AC-019 propagated the same `0..100000` literal from the BC implementer directive.

The shipped code at `prism-dtu-common/src/scenario/mod.rs` correctly used `0..10000`, satisfying `\d{4}`. The implementer noticed the inconsistency and fixed it in code. But the spec was never corrected — so the spec contains a self-contradiction that would mislead a future implementer doing a re-implementation, a spec audit, or a test-vector re-derivation.

**Root cause:** Spec authorship (BC-2.06.020 v1.3 was PO-authored at D-1117) produced the `0..100000` literal without mechanically checking that values in `[0, 100000)` satisfy `\d{4}`. The adversary over 13 passes verified behavioral outcomes (does the code satisfy the contract?) but never ran the cross-check "does the directive literal produce values satisfying the spec's own regex?". These are distinct checks:

- Behavioral check: `gen_device_cves` uses `0..10000` in code; the regex in the test passes; the test is non-vacuous. PASS.
- Directive-literal-vs-invariant check: `0..100000` in the spec directive; `\d{4}` in the same spec's invariant; are values from `[0, 100000)` guaranteed to match `\d{4}`? NO (values 10000–99999 produce 5 digits).

**Canonical rule (codified):**

**Standing adversary probe — for any spec that embeds a concrete code literal (RNG range, modulus, constant) AND a verifying regex/format invariant in the SAME document:** Cross-check the literal against the invariant. Specifically: if the literal is `N..M` and the invariant is `\d{k}`, verify that `M - 1` has exactly `k` digits (i.e., `M <= 10^k`). If `M > 10^k`, the literal is inconsistent with the invariant.

Examples of the check:
- `0..10000` with `\d{4}`: max value 9999, exactly 4 digits. CONSISTENT.
- `0..100000` with `\d{4}`: max value 99999, 5 digits. INCONSISTENT — flag as SPEC-ONLY finding.
- `0..1000` with `\d{4}`: max value 999, only 3 digits (would be zero-padded to 4; CHECK format string — `{:04}` produces 4-digit zero-padded output for all values; CONSISTENT if using `{:04}`).

This probe costs one arithmetic check per range+regex pair. It should be part of every adversary pass on specs that define synthetic-ID generation contracts.

**Outcome:**

BC-2.06.020 v1.3→v1.4: PC-9 implementer directive `0..100000`→`0..10000`. Story B v2.11→v2.12: AC-019 literal `0..100000`→`0..10000`; BC-2.06.020 pin v1.3→v1.4; 19 ACs / 23 RGTs UNCHANGED. PIVOT-003 v1.5→v1.6: BC-2.06.020 pin v1.3→v1.4. Feature HEAD 7ddc0a51 CODE UNCHANGED. BC-INDEX v6.38→v6.39. STORY-INDEX v2.364→v2.365. PR-LEVEL streak RESET 1/3→0/3. Pass 15 NEXT.

---

## (z12) [process-gap] Count-bump bursts must sweep ALL prose carriers of the RGT count, including task-checklist gate instructions and verifier-facing directives — not just frontmatter and the RGT table

**Date recorded:** 2026-06-13
**D-NNN anchor:** D-1121 (BPRL-P15-01 SPEC-ONLY: story B Phase-6 gate instruction stale "19 Red Gate tests")
**Story:** S-DEMO-DTU-LIVE-SCENARIO-001-B
**Tags:** [process-gap] [count-propagation] [gate-instruction] [sweep-discipline]
**Classification:** PROCESS-GAP — a count-bump burst (D-1117: red_gate_tests 19→23) updated the frontmatter `red_gate_tests` field and the Phase-6 gate table (now 23 rows), but did not update a prose gate instruction in the same story that said "all 19 Red Gate tests pass." Two consecutive passes (pass 14 = D-1120 BPRL-P14-01 RNG literal, pass 15 = D-1121 BPRL-P15-01 gate instruction) each found stale literals/counts in directive and checklist PROSE that the canonical count-bump missed.

**(z12) [process-gap] When a story's `red_gate_tests` count is bumped, the sweep MUST include task-checklist gate instructions ("all N Red Gate tests pass"), implementer-directive literals, Phase-N gate task bodies, and verifier-facing prose — not just frontmatter + the RGT table rows.** The RGT table is authoritative, but prose gate instructions that cite "all N tests" are used by agents to DRIVE verification workflows. A stale count in a gate instruction allows a literal verifier to declare a gate passed after running fewer tests than required.

**What happened:** D-1117 raised `red_gate_tests` from 19 to 23 by adding AC-019 and VP-020-I..VP-020-L. The burst correctly updated:
- `red_gate_tests: 23` (frontmatter)
- The RGT table (new rows RGT-20 through RGT-23 added)
- `acceptance_criteria_count: 19` updated to `19` (AC count correctly unchanged)

But it did NOT update:
- The Phase-6 gate instruction prose (line ~581): "Verify all 19 Red Gate tests pass in the fresh worktree" — this stale count persisted through passes 12, 13, 14 undetected and was finally caught at pass 15.

**Root cause:** The D-1117 sweep focused on the frontmatter + the explicit RGT table body. Gate instruction prose in task checklists ("all N Red Gate tests pass") is a third category of count carrier that was not in the sweep scope. Similarly, the z11 lesson (D-1120) found that implementer directive literals in BC prose were also missed by the canonical count-bump sweep. Both passes 14 and 15 found the same class of miss: count literals embedded in directive/checklist prose outside the frontmatter and explicit table.

**Canonical rule (codified):**

**Standing sweep discipline for any count-bump burst affecting `red_gate_tests` or `acceptance_criteria_count`:** Before declaring the bump complete, run a grep for the OLD count value (`\bN\b`) across the ENTIRE story file. For each hit, classify it:

1. **Frontmatter field** (e.g., `red_gate_tests: 19`): update.
2. **Table row or header** (e.g., `## 23 Red Gate Tests`): update.
3. **Changelog entry** (historical immutable): leave.
4. **Task-checklist gate instruction** (e.g., "Verify all 19 Red Gate tests pass"): update.
5. **Implementer directive literal** (e.g., `rng.gen_range(0..10000)`): cross-check against spec invariants (lesson z11 scope).
6. **AC body count** (e.g., "acceptance_criteria_count is 19"): update if this count is being bumped.
7. **RGT row index labels** (RGT-1 through RGT-19): leave (these are row IDs, not counts).

Categories 4 and 5 are the most-frequently-missed because they look like narrative prose, not structured data. The sweep must not stop after updating frontmatter and the table.

**Outcome:**

Story B v2.12→v2.13: Phase-6 gate instruction "19"→"23". Exhaustive `\b19\b`/`\b18\b` classification sweep confirmed this was the sole stale gate-count prose; all other `19` occurrences are AC count (correct, unchanged) or RGT row-index labels (correct, unchanged) or historical changelog entries (immutable). red_gate_tests stays 23; acceptance_criteria_count stays 19. Feature HEAD 7ddc0a51 CODE UNCHANGED. BC-INDEX v6.39→v6.40. STORY-INDEX v2.365→v2.366. Streak 0/3. Pass 16 NEXT.

---

## (z13) [process-gap] Demo-evidence artifacts (.tape headers + evidence-report prose) must be semantic-anchor-audited like specs — the demo-recorder hallucinated 3 BC identifiers that survived 17 passes

**Date recorded:** 2026-06-13
**D-NNN anchor:** D-1124 (BPRL-P18-01 pass-18 convergence pass finding + closure)
**Story:** S-DEMO-DTU-LIVE-SCENARIO-001-B
**Tags:** [process-gap] [demo-evidence] [bc-anchor] [hallucination] [demo-recorder] [adversary-probe]
**Classification:** PROCESS-GAP — demo-recorder hallucinated 3 BC identifiers in AC-019 evidence artifacts; these fabricated/inverted anchors survived passes 1-17 because evidence-artifact BC-anchor verification was not in the standing adversary probe list.

**Description:**

PR-LEVEL pass 18 (the convergence pass at streak 2/3) surfaced BPRL-P18-01 MED: three fabricated or inverted BC identifiers in the AC-019 demo-evidence artifacts:

1. **PC-8 and PC-9 labels inverted.** The evidence prose described PC-8 as "baseline namespace isolation" and PC-9 as "scenario catalog assignment." The canonical BC-2.06.020 v1.4 definitions are the reverse: PC-8 = scenario catalog assignment (Cyberint scenario alerts draw `cve_id` from `catalog.device_cves`); PC-9 = baseline namespace isolation (non-pivotable `CVE-9999-{:04}` format).

2. **Fabricated invariant `INV-CYBERINT-CVE-PIVOT-001`.** The evidence cited this as the governing invariant for the CVE pivot chain. This string does not exist anywhere in `.factory/specs/behavioral-contracts/` or `crates/`. The canonical identifier is `INV-CYBERINT-ALERT-CVE-CORRELATION-001` (BC-2.06.020 §Invariants, introduced at D-1117).

3. **Fabricated type `CveCorrelationCatalog`.** The evidence referenced this as the Rust struct holding per-device CVE assignments. This type does not exist in the codebase. The canonical name is `ScenarioEntityCatalog` (`prism-dtu-common/src/scenario/mod.rs`, introduced D-1117). Grep confirms zero occurrences in `crates/`.

All underlying code, tests, ACs, BCs, and story were correct. The three fabricated identifiers existed only in the evidence-artifact prose.

**Why they survived 17 passes:**

Passes 1-17 applied adversarial scrutiny to: code implementation, spec-code consistency, BC semantic correctness, invariant coverage by RGTs, SAP-1/SAP-2 probes, forbidden-pattern sweeps, and behavioral tracing. None of these probes extends to the demo-evidence artifact prose (`.tape` header comments, `evidence-report.md` narrative). The adversary checked "19/19 AC evidence files present" (file-count PASS) but did not verify that BC identifiers cited WITHIN those files resolve to real anchors in the authoritative BC corpus.

**Root cause:**

Demo-recorder agents author `.tape` scripts and `evidence-report.md` files in a context focused on the demo narrative — what happens in the terminal, what the analyst sees. BC anchor identifiers (invariant names, type names, postcondition labels) in header comments are peripheral to the demo recording task. The demo-recorder hallucinated plausible-looking identifiers rather than verifying against the actual BC file. This is the same hallucination class as z2 (research-agent fabricating Perplexity output) and z7 (PO fabricating grep-scan claims in BC prose) — an agent stating a factual claim about an external artifact without verifying it.

**Canonical rule (codified):**

**Demo-recorder self-check (REQUIRED before declaring evidence COMPLETE):**

1. For every BC identifier cited in a `.tape` header comment or `evidence-report.md` narrative (invariant names, type names, postcondition labels, error code labels): run `grep -r "<identifier>" .factory/specs/behavioral-contracts/ crates/` and verify it returns at least one hit.
2. For every `PC-N` or `PRE-N` label cited in evidence prose: verify the label direction (PC-8 vs PC-9) against the actual BC postconditions section — do not infer from memory or neighboring text.
3. The demo-recorder self-check must produce a brief anchor-verification summary: "All BC identifiers in AC-NNN evidence verified: [list]. Zero fabricated names."

**Standing adversary probe addition (for all passes on stories with evidence artifacts):**

> **Evidence-artifact BC-anchor verification.** For every demo-evidence artifact at `docs/demo-evidence/<story>/`: (1) read the `.tape` header comment and `evidence-report.md` narrative; (2) for every BC identifier (PC-N, PRE-N, INV-*, type names cited as canonical Rust types, error codes): run `grep -r "<identifier>" .factory/specs/behavioral-contracts/ crates/` and confirm at least one hit; (3) verify PC-N labels match the canonical direction in the BC postconditions section. A BC identifier in evidence prose that greps to zero in the authoritative corpus = MEDIUM finding (evidence-anchor drift). This probe is distinct from the file-count check (19/19 files present) — it verifies CONTENT correctness of the files that are present.

**Outcome:**

BPRL-P18-01 closed by demo-recorder commit 5d5484d0: all 3 anchors corrected in both files. `rg INV-CYBERINT-CVE-PIVOT-001 docs/` = zero; `rg INV-CYBERINT-ALERT-CVE-CORRELATION-001 docs/` = present. `rg CveCorrelationCatalog docs/` = zero; `rg ScenarioEntityCatalog docs/` = present. NO re-render (anchors were header-comment-only, never displayed in `.webm`/`.gif`). Streak RESET 2/3→0/3. Pass 19 NEXT at 5d5484d0.

---

## (z14) [process-gap] Demo-evidence run-commands must be re-validated whenever a covered test RELOCATES crates — BPRL-P12-01 moved VP-020-K but the AC-019 tape command and evidence-report coverage claim were not swept

**Date recorded:** 2026-06-13
**D-NNN anchor:** D-1125 (BPRL-P19-01 pass-19 finding + closure)
**Story:** S-DEMO-DTU-LIVE-SCENARIO-001-B
**Tags:** [process-gap] [demo-evidence] [test-relocation] [sibling-sweep] [TD-VSDD-060] [streak-reset]
**Classification:** PROCESS-GAP — the test-relocation/rename sweep (TD-VSDD-060 sibling sweep) fixed story RGT rows and code callsites when VP-020-K moved from prism-dtu-cyberint to prism-dtu-demo-server (BPRL-P12-01 D-1118), but the sweep did not extend to demo-evidence `.tape` run-commands and the corresponding evidence-report corpus tables, resulting in 7 passes of overstated VP-020 coverage.

**Description:**

PR-LEVEL pass 19 applied the z13 evidence-anchor re-audit (lesson z13) to all 6 tapes and the full `evidence-report.md`. All tapes were clean on BC-anchor identity. One finding surfaced in the run-command coverage:

BPRL-P19-01 MED (partial-fix regression): BPRL-P12-01 (D-1118) correctly relocated `test_BC_2_06_020_cyberint_alert_cve_resolves_in_nvd` (VP-020-K) from `prism-dtu-cyberint` to `prism-dtu-demo-server` — the prior test was a false-green (it never called `NvdState::lookup_and_count`). The code fix was correct. However, the AC-019 tape command (`AC-019-cyberint-cve-pivot.tape`) was not updated: it continued to run only `-p prism-dtu-cyberint`. After the relocation, this command exercised VP-020-I, VP-020-J, and VP-020-L (3 tests) but did not invoke VP-020-K (now in `prism-dtu-demo-server`). The `evidence-report.md` continued to state 4/4 VP-020 coverage, overstating the demonstrated coverage by 1 test for passes 13-19 (7 passes of stale coverage claim).

**Why it survived 7 passes (passes 13-18):**

After BPRL-P12-01 closure (D-1118), adversary passes 13-17 verified: (1) VP-020-K test exists in `prism-dtu-demo-server`; (2) VP-020-K is listed in the story RGT table; (3) evidence file count 19/19; (4) evidence-report states 4/4 VP-020. None of these checks re-ran the tape command to verify it EXECUTED VP-020-K. The tape command string itself was not re-examined after the relocation. Pass 18 (z13 evidence-anchor re-audit) verified BC identifier correctness but not run-command coverage scope.

Pass 19 was the first pass to apply the z13 re-audit INCLUDING run-command coverage verification — checking that the tape command actually reaches the test(s) it claims to cover.

**Canonical rule (codified):**

**TD-VSDD-060 sibling sweep extension for demo-evidence:** When a test is RELOCATED to a different crate (or renamed), the sibling sweep MUST include:

1. Story RGT rows — update crate/test-name references (already in D-1118 scope)
2. Code callsites — update if the test helper or test binary was renamed (already in D-1118 scope)
3. **Demo-evidence `.tape` run-commands** — verify that `-p <crate> -E test(<test_name>)` in every affected tape correctly targets the NEW crate/test location. A tape that ran the test in the OLD crate must be updated to run it in the NEW crate.
4. **Evidence-report corpus tables** — verify that per-crate test counts and VP-NNN attribution rows match the NEW location (e.g., if VP-020-K moved from cyberint to demo-server, the evidence-report coverage table must show demo-server=VP-020-K, not cyberint=VP-020-K).

**Standing adversary probe addition (extending z13):**

> **Evidence run-command scope verification.** For each VP or RGT listed in the evidence-report as "PASS": (1) identify which `.tape` demonstrates it; (2) read the tape's `cargo nextest run` command; (3) verify the `-p <crate>` argument matches the crate that ACTUALLY contains the test (use `rg 'fn <test_name>' crates/` to confirm); (4) verify the `-E test(<test_name>)` filter matches the actual test function name. A tape command that targets the wrong crate (due to a relocation not being swept) = MEDIUM finding (coverage overstatement). This probe is now part of the z13 evidence-anchor re-audit and should run on every pass where test-relocation changes appear in the diff or do-not-reflag history.

**Outcome:**

BPRL-P19-01 closed by demo-recorder commit 0863184a: AC-019 re-recorded with both commands — `-p prism-dtu-cyberint` (VP-020-I/J/L; 3 PASS) + `-p prism-dtu-demo-server -E test(cyberint_alert_cve_resolves_in_nvd)` (VP-020-K; 1 PASS). VHS re-render succeeded; `.webm`/`.gif` show all 4 green. Evidence-report corrected to accurate two-crate split (cyberint=3 VP-020-I/J/L, demo-server=10 incl VP-020-K). Streak 0/3. Pass 20 NEXT at 0863184a.
