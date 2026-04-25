# State-Manager Wave-Gate Remediation-Burst Checklist

When remediating findings from an adversarial pass and committing factory-artifacts,
the state-manager MUST update ALL of the following artifacts in a single burst.

This checklist exists because Passes 7, 10, 11, and 12 each found distinct
wave-state.yaml bookkeeping drift caused by narrow-scope remediation bursts that
missed one or more of these items.

---

## wave-state.yaml Bookkeeping (the recurring drift class)

- [ ] **Top-level `next_gate_required:`** — update to the NEXT pass (`pass_N+1_pending`) after your burst
- [ ] **`wave_1.gate_status:`** — update to `integration_gate_pass_N_remediated_awaiting_pass_N+1`
- [ ] **Add `integration_gate_pass_N:` record** with all fields:
  ```yaml
  integration_gate_pass_N: { verdict: BLOCKED|CLEAN, findings: N, remediated: N, remediation_sha: SHA, timestamp: YYYY-MM-DD, passed: true|false }
  ```
- [ ] **Extend `notes:` narrative** with a paragraph describing Pass N: outcome, findings, remediation SHA, what was fixed
- [ ] **Verify no placeholders remain** — run before commit:
  ```bash
  grep -E "TBD|TODO|FIXME|this_burst|XXX|backfill" .factory/wave-state.yaml
  # Must return empty. If not: fix or use a second commit to backfill.
  ```
- [ ] **Verify pass record count** is correct:
  ```bash
  grep -c "integration_gate_pass_[0-9]" .factory/wave-state.yaml
  # Should equal N (current pass number)
  ```

### SHA backfill protocol (when remediation_sha cannot be known pre-commit)

**Single Canonical SHA Rule (mandatory — Pass 5 structural fix):**
A burst MUST reference exactly ONE SHA value across ALL documents. Apply this discipline:

1. **Stage 1 (commit 1):** Write ALL fixes (documents, narrative, frontmatter, wave-state, hook, checklist) using the placeholder `15fa97e6` everywhere a SHA is needed. Write narrative in past-tense ("REMEDIATED", "closed", "applied") from the start — no "in progress" language. Include the past-tense narrative in Stage 1 so Stage 2 is ONLY a SHA replacement.
2. **Stage 2 (commit 2):** Get Stage 1's SHA via `git -C .factory rev-parse HEAD`. Perform a GLOBAL replacement of `15fa97e6` with that SHA across ALL documents (STATE.md, SESSION-HANDOFF.md, wave-state.yaml). This is the ONLY change in Stage 2.
3. **NO third commit.** If you discover a missed fix after Stage 2, execute `git -C .factory reset --soft HEAD~2` and redo from Stage 1. Do NOT add a third remediation commit.
4. **Confirm with user before any rebase** if Stage 2 was already pushed.

**Why not a per-document approach:** Writing SHAs document-by-document as commits land creates a SHA chain where each intermediate commit is cited in some document. The Pass 3–5 drift recurrences all had this root cause. The 15fa97e6 → global-replace approach guarantees exactly one SHA value is cited: the Stage 1 commit.

**Exactly 2-commit chain rule:** The two-commit-protocol exception in `verify-sha-currency.sh` requires:
- HEAD's commit message contains "backfill" (Stage 2 marker), AND
- HEAD^'s commit message does NOT contain "backfill" (Stage 1 must be the fix commit, not another backfill)
- If HEAD^ ALSO contains "backfill": the hook reports FAIL with MULTI_COMMIT_CHAIN_NOT_ALLOWED

Option A (preferred): Use 15fa97e6 (the canonical placeholder)
1. Commit 1: ALL fixes + `15fa97e6` everywhere
2. Commit 2: `sed -i.bak 's/15fa97e6/<STAGE1_SHA>/g' STATE.md SESSION-HANDOFF.md wave-state.yaml && rm *.bak` then commit

Option B: Leave as `TBD_backfill` and immediately dispatch the second commit in the same burst. Do not leave it across sessions.

Never leave `TBD_this_burst` — that string is visually identical to a real entry and the adversary will not catch it until the next pass.

### Recovery from 3+-commit chains

If a 3rd commit accidentally lands during Stage 1 (e.g., a pre-commit hook auto-stages a file, or a previously-modified file gets staged via `git add` autodiscovery):

1. Run `git -C .factory log --oneline -5` to inspect the chain.
2. Run `git -C .factory reset --soft HEAD~N` where N is the number of accidental commits to collapse.
3. Run `git -C .factory status` and inspect ALL staged files.
4. **Unstage any files this burst did not author** (e.g., `sidecar-learning.md` is a session-end-marker tracker — typically not part of state-manager bursts; unstage with `git -C .factory restore --staged sidecar-learning.md`).
5. Re-commit Stage 1 cleanly with only burst-authored content.
6. Document the episode in SESSION-HANDOFF.md "Recent Burst Episodes" section.

This procedure is required because:
- The hook script's multi-commit-chain detection reports MULTI_COMMIT_CHAIN_NOT_ALLOWED for bursts with more than 2 commits.
- Incidental file inclusion in Stage 1 commits creates audit-trail noise in `git show --stat`.
- The Pass 8 burst (2026-04-24) executed `git reset --soft HEAD~3` informally; this procedure is now formalized.

**Pre-burst hygiene check (MANDATORY):** Before starting Stage 1, run `git -C .factory status`. If any unrelated files are modified (sidecar-learning.md, etc.), either commit them separately first OR stash them. Do not allow pre-existing modifications to contaminate the burst commit.

---

## STATE.md Bookkeeping

- [ ] **Frontmatter `adversary_pass_N_wave_integration_gate:`** — add new entry with `{passed, findings, remediated, timestamp}`
- [ ] **Frontmatter `convergence_status:`** — advance to one of:
  - `PHASE_3_<WAVE_NAME>_GATE_PASS_N_REMEDIATED_AWAITING_PASS_N+1` (when adversary verdict was BLOCKED)
  - `PHASE_3_<WAVE_NAME>_GATE_PASS_N_CLEAN_WINDOW_K_OF_3` (when adversary verdict was CLEAN; K is the cumulative count of consecutive clean passes)
  - `PHASE_3_<WAVE_NAME>_GATE_CONVERGED` (when K reaches 3)
  where `<WAVE_NAME>` is the active wave (currently `WAVE_1_5`).
- [ ] **Frontmatter `current_step:`** — update narrative to describe Pass N outcome and what was remediated
- [ ] **Frontmatter `awaiting:`** — update to outcome-neutral form ("if CLEAN...if BLOCKED...")
- [ ] **Frontmatter `convergence_window_progress:`** — update count
- [ ] **Body "Last Updated" table row** — update to describe Pass N
- [ ] **Body "Current Phase" table row** — update pass count and window
- [ ] **Body "Current Step" table row** — update
- [ ] **Body "Phase Progress" table — Wave 1 row** — add Pass N to finding progression
- [ ] **Body "Current Phase Steps" table** — append Pass N row to preserve audit trail (full history kept; header is unqualified `## Current Phase Steps — Wave 1`)
- [ ] **Session Resume Checkpoint** — replace with current checkpoint (outcome-neutral next-steps); archive old to session-checkpoints.md
- [ ] **Version bump** — minor for normal burst (X.Y → X.Y+1, using the document's current major; STATE.md and SESSION-HANDOFF.md are currently at 5.Y)

---

## SESSION-HANDOFF.md

- [ ] **Verify develop HEAD** is current
- [ ] **Verify PR count** is current
- [ ] **Verify stories_merged count** is current
- [ ] **Verify test counts** are current
- [ ] **Next session priority** uses outcome-neutral language (if CLEAN... if BLOCKED...)
- [ ] **No references** to in-progress work that is now complete
- [ ] **factory-artifacts HEAD** must be a concrete SHA, never a placeholder (`(current after this burst)`, `TBD`, etc.) — backfill with second commit if SHA not known at time of writing

---

## Outcome-Neutral Language Rule

When writing next-steps or checkpoints **before** a pass runs:

WRONG: "Pass N — 1st of 3 required clean passes"
RIGHT: "Pass N — if CLEAN, 1st of 3 clean-pass window opens; if BLOCKED, remediate + Pass N+1"

WRONG: "Pass N — 2nd of 3 required clean passes"
RIGHT: "Pass N — if CLEAN, 2nd of 3 clean passes; if BLOCKED, remediate + Pass N+1"

Outcome-presumptive language was flagged as P3WV1L-A-M-002 in Pass 12. Use neutral framing always.

---

## Pre-Commit Verification Commands

```bash
# 1. No placeholders in wave-state.yaml
grep -E "TBD|TODO|FIXME|this_burst|XXX|backfill" .factory/wave-state.yaml

# 2. Pass record count matches current pass
grep -c "integration_gate_pass_[0-9]" .factory/wave-state.yaml

# 3. next_gate_required is N+1, not N
grep "next_gate_required:" .factory/wave-state.yaml

# 4. gate_status mentions current pass (N), not N-1
grep "gate_status:" .factory/wave-state.yaml

# 5. SESSION-HANDOFF.md has current story count
grep "20/20\|stories merged" .factory/SESSION-HANDOFF.md

# 6. STATE.md version bumped
grep "^version:" .factory/STATE.md

# 7. SESSION-HANDOFF.md has no placeholder in factory-artifacts HEAD field
grep -E "current after this burst|placeholder|TBD" .factory/SESSION-HANDOFF.md
# Must return empty. If not: backfill the concrete SHA before pushing.

# 8. factory-artifacts HEAD AND develop HEAD currency check
# Canonical hook (preferred): bash .factory/hooks/verify-sha-currency.sh
# The hook encapsulates all logic below and includes the two-commit exception (with backfill guard).
# Run the hook rather than the inline grep to pick up future hook improvements automatically.
#
# EXACTLY 2-COMMIT CHAIN RULE (Pass 5 structural addition):
# The hook grants the two-commit exception ONLY when:
#   - HEAD commit message contains "backfill" (Stage 2 marker)
#   - HEAD^ commit message does NOT contain "backfill" (Stage 1 is a fix, not another backfill)
# If HEAD^ ALSO contains "backfill", the hook reports FAIL: MULTI_COMMIT_CHAIN_NOT_ALLOWED.
# A burst MUST be exactly 2 commits: 1 fix (Stage 1) + 1 backfill (Stage 2). No extensions.
# Use the 15fa97e6 placeholder in Stage 1; replace globally in Stage 2.
#
# After each factory-artifacts commit, check that STATE.md + SESSION-HANDOFF.md SHAs are current.
# Note on two-commit protocol: commit 2's SHA will always be one ahead of the SHA commit 2 cites
# (written during commit 1's context). The hook grants this exception ONLY when HEAD's commit
# message contains "backfill" AND HEAD^ does not — preventing the exception from masking incomplete
# Stage-2 execution or multi-commit chain extensions.
ACTUAL_FA=$(git -C .factory rev-parse HEAD)
ACTUAL_DEV=$(git rev-parse develop)
CITED_FA_STATE=$(grep -oE 'factory-artifacts HEAD[^0-9a-f]*[0-9a-f]{8}' .factory/STATE.md | head -1 | grep -oE '[0-9a-f]{8}$')
CITED_DEV_STATE=$(grep -oE 'develop_head: "?[0-9a-f]{8}' .factory/STATE.md | head -1 | grep -oE '[0-9a-f]{8}$')
CITED_FA_HANDOFF=$(grep -oE 'factory-artifacts HEAD:? ?\|? ?`?[0-9a-f]{8}' .factory/SESSION-HANDOFF.md | head -1 | grep -oE '[0-9a-f]{8}$')
CITED_DEV_HANDOFF=$(grep -oE 'develop HEAD:? ?\|? ?`?[0-9a-f]{8}' .factory/SESSION-HANDOFF.md | head -1 | grep -oE '[0-9a-f]{8}$')
[ "${ACTUAL_FA:0:8}" = "$CITED_FA_STATE" ] && [ "${ACTUAL_FA:0:8}" = "$CITED_FA_HANDOFF" ] \
  && [ "${ACTUAL_DEV:0:8}" = "$CITED_DEV_STATE" ] && [ "${ACTUAL_DEV:0:8}" = "$CITED_DEV_HANDOFF" ] \
  || echo "STALE SHA drift detected"

# 9. waves: map completeness check
# Ensure wave-state.yaml waves: map contains entries for all documented waves
python3 -c "
import yaml
with open('.factory/wave-state.yaml') as f:
    d = yaml.safe_load(f)
expected = {'wave_0a','wave_0b','wave_0c','wave_0_retrospective','wave_1','wave_1_5','wave_2','wave_3','wave_4','wave_5','wave_6'}
actual = set(d.get('waves', {}).keys())
missing = expected - actual
assert not missing, f'Missing waves: {missing}'
"
```

---

## Failure Modes Observed (Prior Drift Instances)

| Pass | What Was Missed | Root Cause |
|------|----------------|------------|
| Pass 7 | wave-state.yaml gate_status stale | Narrow fix; did not sweep bookkeeping fields |
| Pass 10 | wave-state.yaml 7 consecutive pass records missing | Large remediation burst; bookkeeping treated as secondary |
| Pass 11 | pass_10 remediation_sha left as `TBD_this_burst` | SHA not known pre-commit; no backfill protocol followed |
| Pass 12 | pass_11 record entirely missing; gate_status+next_gate_required stale; notes ended at Pass 10 | Burst did not use a checklist |
| Pass 1 (WV1.5) | develop_head stale post-PR #41 merge; Session Resume Checkpoint and SESSION-HANDOFF.md cited pre-merge SHA | Command #8 only checked factory-artifacts HEAD, not develop HEAD; extended in v5.2 |
| Passes 3–5 (WV1.5) | SHA-drift recurred 5 consecutive times despite hook creation (Pass 3) and tightening (Pass 4); Pass 4 burst chain extended to 4 commits creating multi-SHA fragmentation | SHAs applied document-by-document DURING burst instead of using 15fa97e6 placeholder + global replace AFTER; structural fix: Single Canonical SHA Rule + exactly-2-commit-chain enforcement in hook |
| Pass 6 (WV1.5) | NEW defect class — cross-record SHA contamination: STATE.md frontmatter Pass 3 entry held remediation_sha 3e2359ac (Pass 4 Stage 1 SHA) instead of b1b145b3 (per wave-state.yaml gate_pass_3) | Pass 5 single-canonical-SHA discipline only swept the CURRENT burst's SHA; did not check historical pass record SHAs in STATE.md frontmatter against wave-state.yaml records. Manual orchestrator-executed remediation per user directive corrected cite + added Schema Semantics Clarification below. |

**Pattern:** Every drift instance was caused by a remediation burst that fixed the adversary findings but did not sweep all 4 wave-state.yaml bookkeeping items. This checklist is the structural fix.

## Schema Semantics Clarification (Pass 6 structural addition)

**`remediation_sha` semantic for partially-or-multi-pass-closed records:**

When a burst closes findings from MULTIPLE prior passes (because earlier remediation was incomplete), each affected pass record's `remediation_sha` is set to the SHA of the **closing burst's Stage 1 commit**. Subsequent re-closures DO NOT advance the SHA backward.

Example: Pass 3 was incompletely remediated at b1b145b3 (Stage 2 tense-flip skipped). Pass 4 burst at 99563fd1 closed both Pass 3 leftovers AND Pass 4 findings. Per the rule:
- `gate_pass_3.remediation_sha` = `b1b145b3` (the SHA where Pass 3 was first remediated, even partially)
- `gate_pass_4.remediation_sha` = `99563fd1` (the SHA where Pass 4 was remediated)
- The Pass 4 record's `notes` field documents that 99563fd1 also closed Pass 3 leftovers

**Cross-record SHA verification (NEW command #10):**

Before pushing a state-manager burst, verify STATE.md frontmatter `adversary_*_pass_N_*.remediation_sha` matches `waves.wave_X.gate_pass_N.remediation_sha` for every pass N. Drift between these is the Pass 6 H-001 defect class.

```bash
# Cross-record SHA verification — verifies STATE.md frontmatter Pass-N entries
# match wave-state.yaml `gate_pass_N.remediation_sha` records.
#
# Note on awk pattern: `/^  wave_1_5:/,/^  wave_2:/` extracts the wave_1_5
# subtree by literal block boundaries. This is correct AS LONG AS wave_2 exists
# in wave-state.yaml as the immediate successor block. When Wave 2 is added,
# verify the wave_2: block sits immediately after wave_1_5: in file order; if
# not, update the terminator pattern to the actual successor wave name.
#
# A previous version of this command used `/^  wave_[^_]/` as the terminator,
# which silently collapsed the range to a single line (because `wave_1_5` also
# matches `wave_[^_]`). Fixed 2026-04-24 in pre-Wave-2 audit remediation (ebf7c63c).
#
# EXTRACTION FIX (HIGH-001 2nd-order residual, 3f2c7003): Both STATE.md and
# wave-state.yaml use inline single-line YAML records. In passes 4-9 the field order
# is `remediation_pr: null, remediation_sha: <sha>` — so `grep -oE '[0-9a-f]{8}|null'`
# matched the first hex-or-null token, which was `null` from `remediation_pr:`, not
# the actual SHA from `remediation_sha:`. Fixed by using sed to explicitly target
# `remediation_sha: ` and extract only the value that follows, regardless of field order.
for pass in $(awk '/^  wave_1_5:/,/^  wave_2:/' .factory/wave-state.yaml | grep -oE '^    gate_pass_[0-9]+:' | grep -oE '[0-9]+' | sort -n); do
  state_sha=$(grep "adversary_wave_1_5_gate_pass_${pass}_wave_integration_gate:" .factory/STATE.md \
    | sed -nE 's/.*remediation_sha: ([0-9a-f]+).*/\1/p')
  # IMPORTANT: When Wave 2 adds its first `gate_pass_N:` record, the grep anchor below
  # will match BOTH Wave 1.5 and Wave 2 gate_pass records. At that point, scope the
  # extraction to the wave_1_5 block via the awk range — which is already applied here.
  yaml_sha=$(awk '/^  wave_1_5:/,/^  wave_2:/' .factory/wave-state.yaml \
    | grep "^    gate_pass_${pass}:" \
    | sed -nE 's/.*remediation_sha: ([0-9a-f]+).*/\1/p')
  if [ "$state_sha" = "$yaml_sha" ]; then
    echo "pass_${pass}: STATE=$state_sha YAML=$yaml_sha AGREE"
  else
    echo "DRIFT pass_${pass}: STATE=$state_sha vs YAML=$yaml_sha"
  fi
done
```

**Single Canonical SHA Rule (Pass 5 structural addition):** A burst MUST reference exactly ONE SHA across ALL documents. Use `15fa97e6` placeholder in Stage 1 everywhere a SHA is needed. Stage 2 performs a GLOBAL replacement with Stage 1's actual SHA. NO third commit. If a third commit becomes necessary, `git reset --soft HEAD~2` and redo from Stage 1.

**Two-commit protocol exception:** When using the two-commit SHA backfill protocol, command #8 (and `verify-sha-currency.sh`) grants a one-commit drift exception ONLY when HEAD's commit message contains "backfill" AND HEAD^'s message does NOT contain "backfill". This prevents the exception from masking multi-commit chain extensions (MULTI_COMMIT_CHAIN_NOT_ALLOWED). If the exception fires but HEAD^ also has "backfill" in its message, treat as FAIL and investigate.

**Cite-locked state rule (2026-04-25 structural addition):** After Stage 2 backfill commit, factory-artifacts is in a "cite-locked" state. Any subsequent commit (chore, sidecar, gitignore, etc.) MUST be either (a) part of a new two-commit protocol burst, OR (b) followed immediately by a SHA-citation refresh burst before session-end. Do NOT push factory-artifacts with stale citations. Root cause of this rule: orchestrator added 2 chore commits (45efbab7/b75fb772) after the cascade-closure Stage 2 backfill (2ef24502), advancing HEAD past the cited SHA (13b5ca69) and causing the verify-sha-currency.sh hook to FAIL with "factory-artifacts SHA in SESSION-HANDOFF.md is stale".

---

## gate_status Hook Contract (2026-04-24 structural addition)

NOTE — gate_status hook contract: The wave-gate-prerequisite hook accepts
ONLY the literal tokens `passed` or `deferred` for `gate_status`. When a wave
gate converges, set `gate_status: passed` and preserve semantic verdict
strings (e.g., `integration_gate_CONVERGED_3of3`) in a sibling field
`gate_outcome`. The hook does NOT inspect `gate_outcome` — it is for human/audit
context only. This applies to BOTH per-wave `gate_status` and top-level
`wave_X_gate_status` fields.

Additionally, the validate-wave-gate-completeness.sh PostToolUse hook requires:
1. A `gate_report:` path alongside every `gate_status: passed` in the per-wave block.
2. The referenced file must exist and contain evidence for all 6 gates
   (Gate 1: Test Suite, Gate 2: DTU Validation, Gate 3: Adversarial Review,
    Gate 4: Demo Evidence, Gate 5: Holdout Evaluation, Gate 6: State Update).

**Pre-Wave-N+1 dispatch check (added 2026-04-24 — missed by pre-Wave-2 audit):**
Before dispatching Wave N+1 stories, verify every completed wave has:
```bash
python3 -c "
import yaml
with open('.factory/wave-state.yaml') as f:
    state = yaml.safe_load(f)
for name, data in state['waves'].items():
    status = data.get('gate_status', 'unknown')
    report = data.get('gate_report', 'MISSING')
    print(f'{name}: gate_status={status} gate_report={report}')
"
# wave_1 and wave_1_5 must show gate_status=passed with a valid gate_report path.
```
