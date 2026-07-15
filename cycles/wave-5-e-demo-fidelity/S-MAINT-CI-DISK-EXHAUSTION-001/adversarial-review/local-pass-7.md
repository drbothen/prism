---
document_type: adversarial-review
scope: LOCAL
story_id: S-MAINT-CI-DISK-EXHAUSTION-001
passes: [7]
feature_head_at_review: 22cb83ad
date: 2026-07-15
clean_strict: false
clean_pr_merge: false
finding_counts:
  total: 3
  crit: 0
  high: 0
  med: 1
  low: 1
  obs: 1
  process_gap: 0
streak_after: 0/3
convergence: NOT_CONVERGED
authored_by: orchestrator-relay
---

# LOCAL Adversary Pass 7 — S-MAINT-CI-DISK-EXHAUSTION-001

---

## Pass 7 (frozen 22cb83ad; fresh-context adversary; CI disk-exhaustion hardening; streak 0/3 → 0/3)

**Pass result:** CLEAN(strict)=NO, CLEAN(PR-merge)=NO

**Findings:** 3 total (0 CRIT / 0 HIGH / 1 MED / 1 LOW / 1 OBS / 0 PROCESS-GAP)

**STREAK STAYS 0/3** — 1 MED finding; novelty MEDIUM (AC-003 assertion logic insufficient: presence-only check does not verify section scoping or value payload).

**Code HEAD at review:** 22cb83ad (NEW HEAD from fix-burst-5; DRIFT-ORCH-PRLEVEL-PUSH-001 clean — still LOCAL-ONLY, not pushed)

**CLEAN(strict):** NO — 1 MED + 1 LOW + 1 OBS findings present

**CLEAN(PR-merge):** NO — 1 MED merge-blocking

---

## Finding Register

### F-CIDISK-P7-MED-001 [MED] AC-003 config-invariant assertions are context-blind — presence check does not verify [profile.dev] scoping or debug=false payload

**Severity:** MED

**Classification:** test-coverage gap — assertions pass vacuously if the target lines appear anywhere in the file

**Description:**
The AC-003 Red Gate tests at @22cb83ad used substring/header presence checks of the form:

```bash
count=$(grep -c 'debug = ' .cargo/config.toml)
[ "$count" -ge 1 ]
```

This assertion would pass even if:
1. The `debug = ` line appeared in `[profile.release]` rather than `[profile.dev]`
2. The `debug = ` line appeared in a comment (`# debug = "line-tables-only"`)
3. The `[profile.dev.package."*"]` section existed without `debug = false` in it

The spec requires verification that:
- The `.cargo/config.toml` `[profile.dev]` section contains `debug = "line-tables-only"` (first-party crates)
- The `.cargo/config.toml` `[profile.dev.package."*"]` section exists and contains `debug = false` (dependency crates)

A presence-only grep on `debug =` is not equivalent to a section-scoped assertion. F-CIDISK-P4-HIGH-001 (pass-4) established that `.cargo/config.toml` already had both settings; AC-003's purpose is to guard against their silent removal. A context-blind assertion cannot distinguish "present in correct section" from "present anywhere."

**Fix required:** Redesign AC-003 assertions to use section-scoped `awk` matching:
1. Assert `[profile.dev]` block contains `debug = "line-tables-only"` (awk between section headers)
2. Assert `[profile.dev.package."*"]` block contains `debug = false` (awk between section headers)
Both positive (matching) and negative (non-matching mutations) verification required.

---

### F-CIDISK-P7-LOW-001 [LOW] AC-004 failure-annotation step snippet not synced to df -P

**Severity:** LOW

**Classification:** documentation accuracy — spec snippet uses `df -h` after fix-burst-5 corrected the prose but missed the inline snippet

**Description:**
Fix-burst-5 corrected the annotation step prose to reference `df -P`. However, the AC-004 spec block included an inline code snippet showing the `if: failure()` step shell commands, and this snippet still contained `df -h` rather than `df -P`. The prose and the example snippet were out of sync — a reader verifying the spec against the actual ci.yml would see the snippet still using the old flag.

**Fix required:** Update the AC-004 snippet to use `df -P`. Verify the snippet and the running ci.yml annotation step are identical character-for-character.

---

### F-CIDISK-P7-OBS-001 [OBS] Accounting label in verify-workflow-structure description uses imprecise phrasing

**Severity:** OBS

**Classification:** documentation clarity — minor phrasing imprecision in assertion description comment

**Description:**
The verify-workflow-structure assertion run-block used an accounting comment of the form "11 reach-assertions + 2 config-invariant assertions = 13 total" in one place and "11+2=13" in another. The "reach-assertions" label is non-standard — the established terminology from the story spec is "assertions" or "step assertions." Minor inconsistency; does not affect correctness.

**Fix required:** Normalize accounting label to "step assertions" + "config-invariant assertions" = total. Update both occurrences for consistency.

---

## Fix-Burst 6 Closure Audit

All 3 findings closed in fix-burst-6 via PO + implementer:

**PO adjudications — story v0.7→v0.8:**
- F-CIDISK-P7-MED-001: AC-003 assertions redesigned — section-scoped awk assertion pattern (awk between section markers); positive + negative verification matrix specified; "presence-only grep is insufficient" note added to §Architecture Compliance Rules
- F-CIDISK-P7-LOW-001: AC-004 snippet synced to `df -P`; inline example updated character-for-character with ci.yml annotation step
- F-CIDISK-P7-OBS-001: Accounting label normalized to "step assertions + config-invariant assertions" throughout

**implementer @e48033e4:**
- AC-003 awk assertions propagated to both Linux jobs in ci.yml: section-scoped match for `[profile.dev]`→`debug = "line-tables-only"` and `[profile.dev.package."*"]`→`debug = false`
- Positive AND negative verification: both assertion patterns pass on correct config, both fail on mutation
- Accounting labels normalized in run-block echo
- Full run-block: 13 assertions all exit 0 on @e48033e4

**Result after FB-6:** HEAD @e48033e4 on maintenance/ci-disk-hardening (LOCAL-ONLY; not pushed). Streak stays 0/3 (FB-6 commit advances HEAD). Pass-8 dispatched on frozen @e48033e4.

---

## Standing Probe Results

**SAP-1:** N/A — `.github/workflows/ci.yml` only; no `event_type =` assignments.

**SAP-2:** N/A — no sensor TOML spec modifications.

**SID-1:** N/A — verify-workflow-structure bash assertions, not `#[ignore]`'d Rust tests.

---

## Convergence Assessment

**Pass 7 on frozen 22cb83ad:** NOT CLEAN strict (1 MED + 1 LOW + 1 OBS); novelty MEDIUM (AC-003 context-blind assertion); streak STAYS 0/3.

**Cascade tally at FB-6 close:** 7 passes / 6 fix-bursts.

**New HEAD after FB-6:** @e48033e4 (LOCAL-ONLY; not pushed).

**NEXT:** LOCAL pass 8 on frozen @e48033e4 (streak 0/3; BC-5.39.001 requires 3 consecutive CLEAN(strict) passes).
