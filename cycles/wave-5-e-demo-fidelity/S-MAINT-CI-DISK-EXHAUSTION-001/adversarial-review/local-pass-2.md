---
document_type: adversarial-review
scope: LOCAL
story_id: S-MAINT-CI-DISK-EXHAUSTION-001
passes: [2]
feature_head_at_review: 99f20d13
date: 2026-07-15
clean_strict: false
clean_pr_merge: false
finding_counts:
  total: 2
  crit: 0
  high: 0
  med: 1
  low: 1
  obs: 0
  process_gap: 1
streak_after: 0/3
convergence: NOT_CONVERGED
authored_by: orchestrator-relay
---

# LOCAL Adversary Pass 2 — S-MAINT-CI-DISK-EXHAUSTION-001

---

## Pass 2 (frozen 99f20d13; fresh-context adversary; CI disk-exhaustion hardening; streak 0/3)

**Pass result:** CLEAN(strict)=NO, CLEAN(PR-merge)=NO

**Findings:** 2 total (0 CRIT / 0 HIGH / 1 MED [process-gap] / 1 LOW / 0 OBS)

**STREAK: 0/3** — MED process-gap finding blocks merge.

**Code HEAD at review:** 99f20d13 (fix-burst-2: name-anchored AC-001 grep, df -P form, AVAIL_GB guard; LOCAL-ONLY on maintenance/ci-disk-hardening)

**CLEAN(strict):** NO — 1 MED (process-gap) + 1 LOW present

**CLEAN(PR-merge):** NO — 1 MED process-gap is merge-blocking

---

## Finding Register

### F-CIDISK-P2-MED-001 [MED][process-gap] AC-001/AC-002 assertions SELF-MATCH their own assertion line — systemic across 5 pre-existing assertions

**Severity:** MED

**Classification:** process-gap — assertion infrastructure defect enabling false-pass

**Description:**
The verify-workflow-structure assertions for AC-001 and AC-002 used `grep` patterns that could match the assertion line itself within the verify-workflow-structure job script. The verification step is embedded in ci.yml; `grep ... .github/workflows/ci.yml` therefore scans the file that contains the assertion. If the grep pattern literal appears verbatim in the assertion line (e.g., `grep -c "Report initial disk space" .github/workflows/ci.yml` in a step that itself contains the string `Report initial disk space`), the grep count includes the assertion line as a false match.

Investigation revealed this was systemic: 5 of the existing assertions in the verify-workflow-structure job across AC-001 through AC-005 shared the same pattern. The count-based forms (count ≥ N) were designed to tolerate exactly 1 self-match, but with 2 real occurrences required (linux-test + test-no-default-features), a self-match pushed the count to 3 for a 2-occurrence contract, producing a false-pass where only 1 real occurrence existed.

**Orchestrator adjudication (Canonical Principle Rule 4):** sweep all 7 in-scope verify-workflow-structure assertions to achieve self-match-proof form. No deferral permitted — the false-pass pattern is structurally identical across all assertions and fixing 2 while leaving 5 identical ones unfixed is a production-grade violation.

**Fix required:** For every `grep -cE` assertion in verify-workflow-structure:
1. Anchor the search string to a YAML structural prefix (`^\s+- name:` for step names, `^\s+uses:` for action uses) so the pattern cannot appear as-is in a bash assignment line
2. Where the structural prefix cannot be added (literal TOML content, non-YAML files), use filename scoping: the assertion searches a different file (e.g., `.cargo/config.toml`) rather than `ci.yml`, making self-match impossible
3. Add negative-verification proof comments adjacent to each assertion explaining why the pattern cannot self-match

---

### F-CIDISK-P2-LOW-001 [LOW] AVAIL_GB guard missing from AC-002 assertion

**Severity:** LOW

**Classification:** defensive coding gap — awk may return empty on df failure

**Description:**
The AC-002 post-reclaim gate at @99f20d13 included the `df -P` form but was missing the `${AVAIL_GB:-0}` default-value guard on the awk extraction. If `df -P /` produces no `NR==2` row (e.g., on a non-standard container or if df is not available), `AVAIL_GB` is empty and the `-ge 25` numeric test raises a bash error rather than triggering the diagnostic message.

**Fix required:** Add `AVAIL_GB=${AVAIL_GB:-0}` after the awk assignment, matching the guard pattern specified in the AC-002 spec.

---

## Fix-Burst 3 Closure Audit

Both findings above were closed in fix-burst-3. **Orchestrator adjudication applied Canonical Principle Rule 4**: all 7 in-scope assertions swept to self-match-proof form in a single burst rather than 2+5 phased approach.

**story-writer:** S-MAINT-CI-DISK-EXHAUSTION-001 v0.4→v0.5 — AC-001 through AC-007 spec updated with self-match-proof proof comments; AC-002 `${AVAIL_GB:-0}` guard added to spec; verification matrix expanded (positive / self-match-proof / negative form documented for each assertion).

**implementer @7df532d0:** ci.yml updated — 7 verify-workflow-structure assertions replaced with self-match-proof forms using structural YAML anchors (`^\s+- name:`, `^\s+uses:`) for workflow content and filename-scoped greps for `.cargo/config.toml` content; `${AVAIL_GB:-0}` guard added to AC-002 post-reclaim gate; negative-proof comments added inline.

**Result after FB-3:** HEAD @7df532d0 on maintenance/ci-disk-hardening (LOCAL-ONLY). Streak 0/3 on frozen @7df532d0.

---

## Standing Probe Results

**SAP-1:** N/A — `.github/workflows/ci.yml` only; no `event_type =` assignments.

**SAP-2:** N/A — no sensor TOML spec modifications.

**SID-1:** N/A — verify-workflow-structure bash assertions, not `#[ignore]`'d Rust tests.

---

## Convergence Assessment

**Pass 2 on frozen 99f20d13:** 0/3 (1 MED[PG] + 1 LOW; systematic self-match sweep dispatched; fix-burst-3 closed both)

**Cascade tally at FB-3 close:** 2 passes / 2 fix-bursts.

**New HEAD after FB-3:** @7df532d0 (LOCAL-ONLY).

**NEXT:** LOCAL pass 3 on frozen @7df532d0 (streak 0/3).
