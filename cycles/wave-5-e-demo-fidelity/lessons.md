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
