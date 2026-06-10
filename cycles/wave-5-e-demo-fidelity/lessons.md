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
