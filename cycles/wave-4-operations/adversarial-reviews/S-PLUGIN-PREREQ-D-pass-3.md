---
document_type: adversarial-pass-report
target_artifact: S-PLUGIN-PREREQ-D
pass_N: 3
target_sha: b8861027
base_sha: 95d46be2
verdict: BLOCKED-soft
streak: "0/3 → 0/3"
finding_summary:
  CRITICAL: 0
  HIGH: 0
  MEDIUM: 2
  LOW: 2
  OBS: 2
prior_passes: [pass-1, pass-2]
prior_fix_bursts: [fix-burst-1, fix-burst-2]
trajectory_note: "16 → 8 → 6 (descending; median LOW)"
producer: adversary (orchestrator-backfilled)
timestamp: 2026-05-13T09:00:00Z
---

# S-PLUGIN-PREREQ-D Adversary Pass 3 — BLOCKED-soft

**Target artifact:** S-PLUGIN-PREREQ-D story v1.2 at SHA b8861027  
**Base SHA:** 95d46be2  
**Verdict:** BLOCKED-soft (2 MEDIUM findings; streak does not advance)  
**Streak:** 0/3 → 0/3

## Pass-2 Closure Verification

| Finding | Verdict | Evidence |
|---|---|---|
| F-LP2-MED-001 | ✅ CONFIRMED CLEAN | Zero `test_BC_2_17_005` matches; AC-14 story-local; `test_hot_reload_*` in Red Gate |
| F-LP2-MED-002 | ✅ CONFIRMED CLEAN | BC-2.16.002 in frontmatter + body + AC-16 trace + Token Budget + inputs |
| F-LP2-MED-003 | ✅ CONFIRMED CLEAN | `red_gate_tests: 25` matches body (counted 25) |
| F-LP2-LOW-004 | ✅ CONFIRMED CLEAN | `anchor_capabilities [CAP-029, CAP-032, CAP-034]` |
| F-LP2-LOW-005 | ✅ CONFIRMED CLEAN | AC-1..AC-18 physical order matches numeric |
| F-LP2-LOW-006 | ✅ CONFIRMED CLEAN (sibling artifact) | VP-INDEX v1.34 trailing residue stripped |
| F-LP2-OBS-007 | ⚠️ PAPER-FIX RISK | BC-2.17.007 `introduced:` corrected, BUT POL-20 left 23 sibling BCs in violation → drives F-LP3-MED-002 |
| F-LP2-OBS-008 | ✅ CONFIRMED CLEAN | "non-crate root-repo deliverable" annotation present |

**Summary:** 7/8 CONFIRMED CLEAN + 1 PAPER-FIX-RISK (F-LP2-OBS-007 closure was locally correct but the policy-adoption commit did not sweep sibling BCs, generating F-LP3-MED-002 as cascade finding).

## Findings (6)

### F-LP3-MED-001 — Tasks list / Red Gate test name divergence (in-perimeter)

**Severity:** MEDIUM  
**Routing:** story-writer (in-perimeter)  
**Policy violation:** POL-8 (sibling-sweep gap) + POL-4 (semantic-anchoring)

Task 11 test names in the Tasks section diverge from §Red Gate Tests across 6 lines (truncated names + 2 BC mis-anchors):

- `test_BC_2_17_006_format_version_exceeded_rejected` should be `test_BC_2_17_007_format_version_exceeded_rejected` — manifest validation is BC-2.17.007 scope (not BC-2.17.006 WIT boundary scope).
- `test_BC_2_17_006_missing_allowed_urls_rejected` has the same mis-anchor; correct anchor is `test_BC_2_17_007_missing_allowed_urls_rejected`.

The §Red Gate Tests section has the correct BC-2.17.007 prefix; the Tasks list was not swept when fix-burst-1 hardened the Red Gate conventions. Two independent fresh-context passes (pass-1, pass-2) had correct Red Gate section but did not surface the Tasks-list mis-anchor until this pass-3 lens. Load-bearing: test framework dispatch uses the name as the canonical identity.

### F-LP3-MED-002 — POL-20 sibling-sweep gap: 23 BCs still in violation (system-level, out-of-perimeter)

**Severity:** MEDIUM  
**Routing:** state-manager workspace POL-20 sweep (separate burst — NOT a PREREQ-D blocker per Companion Routing rule 3)  
**Policy violation:** POL-20 (bc_introduced_field_canonical_format)

POL-20 adopted in fix-burst-2 closure (policies.yaml v1.9) targets the canonical `introduced: YYYY-MM-DD` format for BC frontmatter. However the policy-adoption commit targeted only BC-2.17.007 (the BC authored in this story's scope). A workspace-wide grep shows 23 BCs still carry non-canonical `introduced:` values (`introduced: wave-3`, `introduced: v3.0.0`, `introduced: v1.0.0-greenfield`).

Adversary categorizes as system-level deferred finding per BC-5.39.002 PC2 — **NOT a PREREQ-D blocker** if categorized correctly in the next fix-burst. The state-manager should execute a POL-20 sibling sweep as a **separate legitimate burst** (not a defer per CLAUDE.md Companion Routing rule 3 surface-vs-defer; the finding is real, the routing is system-level not story-level).

### F-LP3-LOW-003 — AC-10 fixture path ambiguity (.wat vs .prx)

**Severity:** LOW  
**Routing:** story-writer

Story line 375, AC-10 acceptance criterion: fixture path extension is ambiguous between `.wat` (WASM text format used in toolchain documentation) and `.prx` (prism-plugin format canonicalized by ADR-023). The criterion reads as if either extension is acceptable, but ADR-023 §C2 establishes `.prx` as the only runtime-recognized extension. Clarification: fixture generation tests should use `.prx` exclusively; `.wat` is compile-time intermediate only.

### F-LP3-LOW-004 — TODO(S-4.08) tag collision: 4 sites, 1 closed by this story, 3 out-of-scope

**Severity:** LOW  
**Routing:** story-writer (annotation refinement)

Four `TODO(S-4.08)` tags exist in the story; this story is S-PLUGIN-PREREQ-D. One tag was legitimately closed by this story's scope (plugin manifest timeout config wiring). Three tags reference S-4.08 (a different story number — plugin authoring SDK, Wave 5+) and are not closed by this story. The tags are not greppable without disambiguation. Recommend: rename closed tag to `RESOLVED(S-PLUGIN-PREREQ-D)` and rename the out-of-scope three to `TODO(S-4.08-PLUGIN-SDK)` for disambiguation.

### F-LP3-OBS-005 — Changelog v1.2 "6/8 closed in-scope" scope note

**Severity:** OBS  
**Routing:** story-writer

Changelog entry for v1.2 reads "6/8 closed in-scope." This count is true for findings closed within the story file only. Two additional closures landed in sibling artifacts: VP-INDEX (fix-burst-1, architect commit 4218e72a) and BC-2.17.007 (fix-burst-2, this story's frontmatter via architect commit). The changelog entry undercounts total fix-burst-2 scope. Suggest clarifier: "6/8 story-local + 2 sibling-artifact" or similar.

### F-LP3-OBS-006 — Architecture Compliance Rules row cites BC-2.17.005 (post fix-burst-1 removal)

**Severity:** OBS  
**Routing:** story-writer

The Architecture Compliance Rules section contains one row still referencing BC-2.17.005. Fix-burst-1 removed BC-2.17.005 from the PRD and BC-INDEX (dropped as superseded by BC-2.17.007 manifest validation). The Architecture Compliance Rules row was not swept as part of that removal. Minor inconsistency; non-blocking but should be cleaned.

## KUDOs (4)

1. **AC-5 manifest validation specification** — best-spec discipline: per-field error mapping, no-partial-registration guarantee, sibling-plugin isolation semantics. Serves as reference pattern for future manifest-heavy BCs.
2. **AC-9 reqwest::Client construction-site / distribution-site split** — unambiguous: construction in plugin loader, distribution via Arc<dyn HttpClient>. No ambiguity about ownership or clone semantics.
3. **F-LP2-LOW-006 closure routing through VP-INDEX** — emergent policy-aware routing: adversary correctly identified that VP-INDEX trailing-residue was a sibling-artifact finding, and fix-burst-2 routed it to architect context (4218e72a) rather than bundling with story-writer. Demonstrates growing process maturity.
4. **AC-18 precedence rule defensive-spec** — `PRISM_DISABLE` + stale `plugin_dir` preemption rule explicitly specified with failure-mode annotation. Prevents runtime ambiguity.

## Process-Gaps

### PG-LP3-001 (HIGH process-gap) — Policy-adoption SOP: no sibling-scope grep before merge

POL-20 (and earlier POL-9 step 6) was adopted via fix-burst closure targeting single BCs without scanning the broader sibling scope. This is the second recurrence of this pattern (POL-9 step 6 in fix-burst-1 similarly targeted only the anchor artifact). Recommend: policy-adoption commits should include a documented grep-based sibling-scope check before merge. Consider amending the state-manager's commit-message template for policy-add events to include a mandatory sweep log.

**Recurrence count:** 2 (POL-9 fix-burst-1 + POL-20 fix-burst-2).

### PG-LP3-002 (MEDIUM process-gap) — Tasks-list test names not swept against Red Gate Tests

The Tasks list (§Tasks, line ~110) enumerates test names for task 11. These names diverged from §Red Gate Tests across two fix-bursts. Fix-burst-1 hardened the BC/VP prefix convention in §Red Gate Tests; the Tasks list was not swept. Story templates should either DRY the test list (single source) or include a lint-hook checking Tasks-name ⊆ Red Gate-name. Pass-3 fresh-context lens caught this; two prior passes' adversaries did not audit the Tasks section independently from Red Gate.

**Pattern:** cross-section name divergence; requires explicit cross-section consistency audit step in fix-burst SOP.

## Novelty Assessment

| Field | Value |
|-------|-------|
| **Pass** | 3 |
| **New findings** | 6 |
| **Duplicate/variant findings** | 0 |
| **Novelty score** | 6 / (6 + 0) = 1.00 |
| **Median severity** | LOW (2 MED + 2 LOW + 2 OBS; median LOW) |
| **Trajectory** | 16 → 8 → 6 |
| **Verdict** | FINDINGS_REMAIN |

F-LP3-MED-001 is a load-bearing semantic gap (2 BC mis-anchors in test names) that survived 2 prior fresh-context passes — the pass-3 lens targeted the Tasks section independently from Red Gate and caught the divergence. F-LP3-MED-002 is genuinely novel because POL-20 didn't exist until fix-burst-2; this is the first pass where the policy was in scope. Trajectory 16 → 8 → 6: convergence in count, but quality of remaining findings is non-trivial (2 MEDIUM in-scope items require action before CLEAN).

## Convergence Position

**Verdict:** BLOCKED-soft (2 MEDIUM findings present; streak cannot advance)  
**Streak:** 0/3 → 0/3

**Two-track fix recommendation:**

- **Track 1 (in-perimeter):** F-LP3-MED-001 + F-LP3-LOW-003 + F-LP3-LOW-004 + F-LP3-OBS-005 + F-LP3-OBS-006 → story-writer fix-burst-3 (story v1.2 → v1.3)
- **Track 2 (system-level):** F-LP3-MED-002 → state-manager POL-20 workspace sweep, separate legitimate burst per CLAUDE.md Companion Routing rule 3 (surface finding is real; routing is system-level, not a defer)

If both tracks close cleanly: pass-4 expected CLEAN → streak 0/3 → 1/3. Two additional CLEAN passes (pass-5, pass-6) required for 3/3 CONVERGED under BC-5.39.001 protocol.
