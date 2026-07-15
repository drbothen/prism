---
document_type: adversarial-review
scope: LOCAL
story_id: S-MAINT-CI-DISK-EXHAUSTION-001
passes: [4]
feature_head_at_review: 7df532d0
date: 2026-07-15
clean_strict: false
clean_pr_merge: false
finding_counts:
  total: 8
  crit: 0
  high: 2
  med: 2
  low: 1
  obs: 3
  process_gap: 0
streak_after: 0/3
convergence: NOT_CONVERGED
authored_by: orchestrator-relay
---

# LOCAL Adversary Pass 4 — S-MAINT-CI-DISK-EXHAUSTION-001

---

## Pass 4 (frozen 7df532d0; fresh-context adversary; CI disk-exhaustion hardening; streak 1/3 → RESET 0/3)

**Pass result:** CLEAN(strict)=NO, CLEAN(PR-merge)=NO

**Findings:** 8 total (0 CRIT / 2 HIGH / 2 MED / 1 LOW / 3 OBS / 0 PROCESS-GAP)

**STREAK RESET: 1/3 → 0/3** — 2 HIGH + 2 MED findings; novelty HIGH (deep-dive angle: root-cause hypothesis falsification + self-match sweep residue).

**Code HEAD at review:** 7df532d0 (SAME frozen HEAD as pass-3; fresh adversary dispatched against unchanged HEAD per BC-5.39.001)

**CLEAN(strict):** NO — 2 HIGH + 2 MED + 1 LOW + 3 OBS findings present

**CLEAN(PR-merge):** NO — 2 HIGH + 2 MED merge-blocking

---

## Finding Register

### F-CIDISK-P4-HIGH-001 [HIGH] `CARGO_PROFILE_DEV_DEBUG` env var in ci.yml is a NO-OP — AC-003 as originally specified was decorative; root-cause hypothesis false

**Severity:** HIGH

**Classification:** implementation defect — env var has no effect; AC-003 was a no-op decoration; root-cause story arc was incorrect

**Description:**
The story's `§Root Cause Hypothesis` and the original AC-003 (as specified in v0.3) stated that adding `CARGO_PROFILE_DEV_DEBUG="line-tables-only"` to ci.yml would reduce debug-info artifact size as a disk-mitigation lever. This is false:

1. `.cargo/config.toml` already contains `[profile.dev] debug = "line-tables-only"` — this value was in place prior to this story and prior to the observed disk failures.
2. `.cargo/config.toml` also contains `[profile.dev.package."*"] debug = false` — this sets no debug info for ALL dependency crates, which is *stronger* than `"line-tables-only"`.
3. The `CARGO_PROFILE_DEV_DEBUG` env var would be overridden by the `debug = false` per-package entry for dependencies and merely match the already-active `"line-tables-only"` for first-party code. Net effect: **zero change to any artifact**.

Shipping `CARGO_PROFILE_DEV_DEBUG` in ci.yml would mislead future readers into believing CI was performing a debug-info reduction that had been active for weeks. This is a documentation-accuracy defect (AD-017 / POL-24 class) combined with a false-signal risk.

**PO adjudication required:** AC-003 must be redesigned. The correct AC-003 is a regression-guard: verify that `.cargo/config.toml` still contains `debug = "line-tables-only"` and `[profile.dev.package."*"]` — a pre-existing invariant that must not be silently removed. The `CARGO_PROFILE_DEV_DEBUG` env var is a FORBIDDEN PATTERN and must be noted as such in the story.

**Fix required:** Delete `CARGO_PROFILE_DEV_DEBUG` from ci.yml. Replace AC-003 with two `.cargo/config.toml` debug-invariant assertions in verify-workflow-structure (Red Gate tests 3 and 4). Add a "FORBIDDEN PATTERN" section to the story documenting `CARGO_PROFILE_DEV_DEBUG`. Rewrite root-cause hypothesis to reflect that the debug-info axis provides no available mitigation lever.

---

### F-CIDISK-P4-HIGH-002 [HIGH] Sweep leak — AC-007 semver-checks and AC-008 no-default-features verify-workflow-structure assertions still self-matching

**Severity:** HIGH

**Classification:** process-gap recurrence — fix-burst-3 sweep was incomplete; two assertions escaped

**Description:**
Fix-burst-3 claimed to make all 7 verify-workflow-structure assertions self-match-proof. Fresh-context pass-4 found that the AC-007 (semver-checks) and AC-008 (no-default-features) assertions still used patterns that could self-match within ci.yml. The `^\s+uses:` structural anchor was applied to the insightsengineering action (AC-002) but not propagated to the step-name checks for AC-007 and AC-008 assertions.

This is a production-grade sweep completeness failure: a claim of "all 7 in-scope assertions fixed" that missed 2 of 7.

**Fix required:** Apply `^\s+- name:` structural anchor to AC-007 and AC-008 assertions in verify-workflow-structure. Confirm with negative-proof comments. Comprehensive re-verify: run each assertion in isolation against a copy of ci.yml with and without the target, confirm count matches expectation.

---

### F-CIDISK-P4-MED-001 [MED] `swap-storage: true` in insightsengineering action vs documented OOM risk for doctest compilation

**Severity:** MED

**Classification:** implementation defect vs spec — action configured with swap-storage:true; AC-002 spec requires false

**Description:**
At frozen 7df532d0, the insightsengineering/disk-space-reclaimer invocation in ci.yml set `swap-storage: true`. The AC-002 spec (v0.4) explicitly requires `swap-storage: false` with the rationale: "swap preserved as OOM headroom (see EC-008)." Doctest compilation in CI has exhibited OOM failures under high concurrency; removing the swap file as an OOM buffer risks turning disk failures into OOM failures.

**Fix required:** Set `swap-storage: false` in the insightsengineering action inputs. Ensure EC-008 "swap trade-off" rationale is documented in the story.

---

### F-CIDISK-P4-MED-002 [MED] `test-no-default-features` sibling job unprotected — disk-reclaim + preflight absent

**Severity:** MED

**Classification:** implementation defect — AC-002 spec explicitly covers both linux-test AND test-no-default-features; only one job was instrumented

**Description:**
The `test-no-default-features` job runs on `ubuntu-latest` and is subject to the same disk-exhaustion risk as the primary linux-test leg. AC-001 and AC-002 spec language explicitly requires both jobs to carry the preflight, reclaim, and post-reclaim gate steps. At frozen 7df532d0, only the linux-test job matrix leg had been instrumented; test-no-default-features was missing all three steps.

**Fix required:** Mirror all disk-hardening steps (preflight, disk-space-reclaimer with ≥25 GB gate, failure annotation) from the linux-test job into the test-no-default-features job. Update verify-workflow-structure assertion counts: the count ≥ 2 form in AC-001/AC-002 assertions already encodes this requirement — the fix brings the implementation into alignment.

---

### F-CIDISK-P4-LOW-001 [LOW] Summary line miscount — story §Acceptance Criteria count header out of sync with actual AC count

**Severity:** LOW

**Classification:** documentation accuracy — stale count header

**Description:**
The story `acceptance_criteria_count` frontmatter field and/or the §Summary line counted 5 ACs, but the redesigned spec (post-pass-4 adjudication) has restructured AC-003 into two verify-workflow-structure assertions (Red Gate tests 3 and 4), with EC-003/004/006 retired and EC-008 added. The count and the Red Gate test count (originally 2) must be updated to reflect 4 Red Gate tests.

**Fix required:** Update `red_gate_tests: 2 → 4` in frontmatter. Update AC text to reflect 4 Red Gate tests: AC-001 assertion = Red Gate test 1; AC-002 assertion = Red Gate test 2; AC-003 assertion 3 (line-tables-only) = Red Gate test 3; AC-003 assertion 4 (package override section) = Red Gate test 4.

---

### F-CIDISK-P4-OBS-001 [OBS] EC-003/004/006 reference CARGO_PROFILE_DEV_DEBUG behavior that is now a forbidden pattern

**Severity:** OBS

**Classification:** spec consistency — error cases reference a now-forbidden implementation approach

**Description:**
EC-003, EC-004, and EC-006 documented error conditions related to `CARGO_PROFILE_DEV_DEBUG` value ranges and precedence. With F-CIDISK-P4-HIGH-001 adjudicated and `CARGO_PROFILE_DEV_DEBUG` declared a FORBIDDEN PATTERN, these error cases have no target behavior to specify and should be retired.

**Fix required:** Retire EC-003, EC-004, EC-006. Add EC-008 documenting the swap-storage:false trade-off (OOM headroom preservation).

---

### F-CIDISK-P4-OBS-002 [OBS] AC-001/AC-002 assertions use exact-match form instead of count ≥ 2 where both jobs must be covered

**Severity:** OBS

**Classification:** test-coverage gap — exact count is brittle; count ≥ 2 form is the specified contract

**Description:**
Following the self-match-proof refactor, the AC-001 and AC-002 assertions used `[ "$count" -eq 2 ]` (exact match) rather than `[ "$count" -ge 2 ]` (count ≥ 2). The spec requires `count ≥ 2` to allow for future additional Linux job legs without breaking the assertion; exact-match-2 would spuriously fail if a third Linux leg is added.

**Fix required:** Replace `[ "$count" -eq 2 ]` with `[ "$count" -ge 2 ]` in both AC-001 and AC-002 verify-workflow-structure assertions.

---

### F-CIDISK-P4-OBS-003 [OBS] AC-007/AC-008 anchor patterns not ratified in story spec

**Severity:** OBS

**Classification:** spec documentation gap — assertion anchor patterns implemented without spec documentation

**Description:**
The final self-match-proof anchor patterns for AC-007 (semver-checks) and AC-008 (no-default-features) assertions were implemented in ci.yml without corresponding spec updates in the story documenting the anchor form and the self-match-proof rationale. Future PO/adversary review would not be able to verify the pattern against the spec.

**Fix required:** Add the anchor pattern and self-match-proof proof comment to the AC-007 and AC-008 spec blocks in the story, matching the format of AC-001/AC-002.

---

## Fix-Burst 4 Closure Audit

All 8 findings were closed in fix-burst-4 via PO adjudications (story-writer) + implementer:

**PO adjudications — story v0.5→v0.6:**
- F-CIDISK-P4-HIGH-001: AC-003 redesigned → two `.cargo/config.toml` debug-invariant assertions (Red Gate tests 3+4); `CARGO_PROFILE_DEV_DEBUG` added to FORBIDDEN PATTERN table; root-cause hypothesis rewritten; `red_gate_tests: 2→4`
- F-CIDISK-P4-HIGH-002: AC-007/AC-008 anchor patterns ratified in spec with proof comments
- F-CIDISK-P4-MED-001: `swap-storage: false` confirmed in spec; EC-008 swap trade-off rationale added
- F-CIDISK-P4-MED-002: `test-no-default-features` explicitly named in AC-001/AC-002/AC-004 job scope
- F-CIDISK-P4-LOW-001: `acceptance_criteria_count` and `red_gate_tests` frontmatter corrected
- F-CIDISK-P4-OBS-001: EC-003/EC-004/EC-006 retired; EC-008 swap trade-off added
- F-CIDISK-P4-OBS-002: AC-001/AC-002 spec updated to `count ≥ 2` form
- F-CIDISK-P4-OBS-003: AC-007/AC-008 anchor spec blocks added with proof comments

**implementer @0d1add9f:**
- `CARGO_PROFILE_DEV_DEBUG` env var removed from ci.yml
- AC-003 replaced with two `.cargo/config.toml` debug-invariant grep assertions in verify-workflow-structure
- `swap-storage: false` set in insightsengineering action
- `test-no-default-features` job instrumented with all three disk-hardening steps
- AC-001/AC-002 assertions updated to `[ "$count" -ge 2 ]` form
- AC-007/AC-008 assertions re-anchored with `^\s+- name:` structural prefix + proof comments
- Run-block: 13 assertions all exit 0 on @0d1add9f

**Result after FB-4:** HEAD @0d1add9f on maintenance/ci-disk-hardening (LOCAL-ONLY; not pushed). Streak RESET 0/3 (FB-4 commit advances HEAD). Pass-5 dispatched.

---

## Standing Probe Results

**SAP-1:** N/A — `.github/workflows/ci.yml` only; no `event_type =` assignments.

**SAP-2:** N/A — no sensor TOML spec modifications.

**SID-1:** N/A — verify-workflow-structure bash assertions, not `#[ignore]`'d Rust tests.

---

## Convergence Assessment

**Pass 4 on frozen 7df532d0:** NOT CLEAN strict (2 HIGH + 2 MED + 1 LOW + 3 OBS); novelty HIGH (root-cause falsification + sweep residue); streak RESET 1/3 → 0/3

**Cascade tally at FB-4 close:** 4 passes / 4 fix-bursts.

**New HEAD after FB-4:** @0d1add9f (LOCAL-ONLY; not pushed).

**NEXT:** LOCAL pass 5 on frozen @0d1add9f (streak 0/3; BC-5.39.001 requires 3 consecutive CLEAN(strict) passes).
