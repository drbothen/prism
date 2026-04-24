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

Option A (preferred): Two-commit sequence
1. Commit with `remediation_sha: TBD_backfill` and note in commit message "pass N SHA to be backfilled"
2. After push, read the commit SHA, update wave-state.yaml, commit `state: backfill Pass N remediation SHA`, push

Option B: Leave as `TBD_backfill` and immediately dispatch the second commit in the same burst. Do not leave it across sessions.

Never leave `TBD_this_burst` — that string is visually identical to a real entry and the adversary will not catch it until the next pass.

---

## STATE.md Bookkeeping

- [ ] **Frontmatter `adversary_pass_N_wave_integration_gate:`** — add new entry with `{passed, findings, remediated, timestamp}`
- [ ] **Frontmatter `convergence_status:`** — advance to `PHASE_3_WAVE_1_GATE_PASS_N_REMEDIATED_AWAITING_PASS_N+1`
- [ ] **Frontmatter `current_step:`** — update narrative to describe Pass N outcome and what was remediated
- [ ] **Frontmatter `awaiting:`** — update to outcome-neutral form ("if CLEAN...if BLOCKED...")
- [ ] **Frontmatter `convergence_window_progress:`** — update count
- [ ] **Body "Last Updated" table row** — update to describe Pass N
- [ ] **Body "Current Phase" table row** — update pass count and window
- [ ] **Body "Current Step" table row** — update
- [ ] **Body "Phase Progress" table — Wave 1 row** — add Pass N to finding progression
- [ ] **Body "Current Phase Steps" table** — append Pass N row to preserve audit trail (full history kept; header is unqualified `## Current Phase Steps — Wave 1`)
- [ ] **Session Resume Checkpoint** — replace with current checkpoint (outcome-neutral next-steps); archive old to session-checkpoints.md
- [ ] **Version bump** — minor for normal burst (2.X → 2.X+1)

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
# After each factory-artifacts commit, check that STATE.md + SESSION-HANDOFF.md SHAs are current
# Note on two-commit protocol: commit 2's SHA will always be one ahead of the SHA commit 2 cites
# (written during commit 1's context). This is expected — treat as a known exception, not a false positive.
ACTUAL_FA=$(git -C .factory rev-parse HEAD)
ACTUAL_DEV=$(git rev-parse develop)
CITED_FA_STATE=$(grep -oE 'factory-artifacts HEAD:? ?`?[0-9a-f]{8}' .factory/STATE.md | head -1 | grep -oE '[0-9a-f]{8}$')
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

**Pattern:** Every drift instance was caused by a remediation burst that fixed the adversary findings but did not sweep all 4 wave-state.yaml bookkeeping items. This checklist is the structural fix.

**Two-commit protocol exception:** When using the two-commit SHA backfill protocol, command #8 will report STALE on commit 2's own SHA (commit 2 cites the SHA of commit 1, which is one behind). This is expected behavior — the drift is intentional and resolves on the next read. Do not treat this as a protocol violation.
