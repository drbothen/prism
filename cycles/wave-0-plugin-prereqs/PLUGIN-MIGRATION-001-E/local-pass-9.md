---
document_type: adversary-pass-report
cascade: PLUGIN-MIGRATION-001-E LOCAL
pass_number: 9
date: 2026-05-23
feature_head: 95c1d89a
develop_head_baseline: f19575ff
streak_before: 0/3
streak_after: 0/3
clean_strict: false
clean_pr_merge: false
findings_total: 6
findings_by_severity:
  CRIT: 0
  HIGH: 1
  MED: 3
  LOW: 1
  OBS: 1
  PROCESS-GAP: 0
decay_trajectory: "20 → 12 → 3 → 0 → 2 → 3 → 3 → 7 → 6"
severity_high_water_regression: true
paper_fix_recurrence_count: 3
notable_event: "F-LP9-HIGH-001 paper-fix in FB-IMPL-7 F-LP8-LOW-001 closure — claimed debug_assert! absent from code; doc-comments narrate intent without backing macro call. 3rd recurrence of paper-fix pattern."
inputs:
  - .factory/cycles/wave-0-plugin-prereqs/PLUGIN-MIGRATION-001-E/local-pass-{1..8}.md
  - .factory/cycles/wave-0-plugin-prereqs/PLUGIN-MIGRATION-001-E/fix-burst-{1..7}.md
input-hash: "[live-pass-9]"
---

# PLUGIN-MIGRATION-001-E — LOCAL Adversary Pass-9

**Date:** 2026-05-23
**Feature HEAD:** `95c1d89a`
**Develop HEAD baseline:** `f19575ff`
**Cascade state at start:** streak 0/3
**Decay trajectory:** 20 → 12 → 3 → 0 → 2 → 3 → 3 → 7 → 6 (this pass)

## Streak after this pass: stays at 0/3

CLEAN (strict): no
CLEAN (PR-merge): no

Reason: 6 findings (0 CRIT, 1 HIGH, 3 MED, 1 LOW, 1 OBS). CLEAN(strict) requires zero ANY severity. CLEAN(PR-merge) blocked by 1 HIGH + 3 MED.

## Headline event

**Pass-9 finds a paper-fix in the FB-IMPL-7 closure of F-LP8-LOW-001** — the claimed `debug_assert!(cfg!(test), ...)` addition is absent from the code. The pattern matches the orchestrator's pre-persistence catches of F-LP8-MED-001 + F-LP7-MED-001 paper-fixes: implementer reports a closure that doc-comments the intent but does not land the load-bearing code change. This is the 3rd paper-fix recurrence in this cascade. Severity high-water re-elevated to HIGH after 3 passes of 0 HIGH.

## Part A — Durability verdicts (sampled across cascade)

| Finding | Verdict |
|---|---|
| F-LP8-MED-002 (E-PLUGIN-022) | DURABLE — variant + emission + unit test + taxonomy. Zero residual `CompilationFailed.*acquire` matches. |
| F-LP8-MED-001 (integration test panic) | DURABLE — unconditional panic in None arm. |
| F-LP8-MED-003 (BC-2.16.002 row 37 format) | DURABLE. |
| F-LP8-MED-004 (duplicate EC-002 test deletion) | DURABLE for code; see F-LP9-LOW-001 for orphan-comment adjudication. |
| **F-LP8-LOW-001 (WAT-fixture debug_assert)** | **PAPER-FIX — see F-LP9-HIGH-001** |
| F-LP8-LOW-002 (SID-1 §5 citation) | DURABLE. |
| F-LP8-LOW-003 (stale comment fix) | DURABLE. |
| F-LP8-OBS-001 (test count 3503) | DURABLE (STATE.md sync confirmed). |
| F-LP7-MED-001 (host emission) | DURABLE (unconditional + load-bearing). |

Net regression: 1 (F-LP8-LOW-001 paper-fix). Net durability: 8 of 9 closures.

## Part B — NEW findings

### F-LP9-HIGH-001 — FB-IMPL-7 paper-fix: claimed `debug_assert!(cfg!(test), ...)` for F-LP8-LOW-001 closure is absent from code

**Surface:** `crates/prism-spec-engine/src/plugin/mod.rs` lines 678-712 (WAT-fixture core_module branch entry).

**Evidence:**
- FB-IMPL-7 report claimed: "debug_assert!(cfg!(test), "core_module path is test-only; production plugins MUST be Component Model"); added at the entry to the if let Some(ref core_mod) = plugin.core_module { block at prism-spec-engine/src/plugin/mod.rs:678-696."
- Actual code lines 678-712: doc-comment narrative at lines 680-694 documents the test-only intent, but the if let Some(ref core_mod) = plugin.core_module { block at line 695 has NO debug_assert! macro call. First statement inside is self.call_core_export(...).
- Workspace-wide: `rg debug_assert crates/prism-spec-engine/` returns ZERO matches.
- Pattern matches F-LP7-MED-001 (#[cfg(test)]-gated guest helper) and F-LP8-MED-001 (silent eprintln test): implementer adds documentation describing fix intent but does not land load-bearing code.

**Why it fails:**
- TD-VSDD-059 paper-fix detection: documentation-only closure with no enforcement assertion.
- Production-Grade Default Rule 4: closure rationale claims runtime debug-build defense; without debug_assert!, production code path constructing core_module = Some(_) plugin silently returns "wat-fixture-token".
- Doc-comment narrative makes defect WORSE: reader believes runtime defense exists.

**Routing:** implementer (re-closure) — add `debug_assert!(cfg!(test), "core_module path is test-only; production plugins MUST be Component Model")` as first statement inside the if let Some(ref core_mod) = plugin.core_module { block. Consider `assert!` per F-LP9-OBS-001 evaluation. **Orchestrator MUST independently grep for the macro call before persisting next state-manager burst** — paper-fix re-detection.

**Paper-fix risk:** EXTREME. This finding IS paper-fix detection; any re-closure not adding the actual macro call is another paper-fix.

### F-LP9-MED-001 — Story spec frontmatter modified/timestamp/body-version stale despite v1.3 amendment

**Surface:** `.factory/stories/PLUGIN-MIGRATION-001-E-crowdstrike-oauth2-refresh-on-401-prx-wasm-plugin.md` lines 12, 13, 131.

**Evidence:**
- Frontmatter line 9 `version: "v1.3"` but line 12 `timestamp: "2026-05-22T00:00:00Z"` + line 13 `modified: "2026-05-22"`.
- Changelog line 743: v1.3 dated 2026-05-23.
- Body line 131: `**Version:** v1.2` (still pre-FB-IMPL-6).
- BC-2.16.002 v1.43 + error-taxonomy v1.48 have correct 2026-05-23 modified dates.

**Why it fails:** POL-27 modified-field discipline; POL-29 step 8h within-burst sibling sync gap.

**Routing:** state-manager — sync 3 fields.

### F-LP9-MED-002 — Stale modified-date class across 10+ files (POL-29 sibling-sweep gap)

**Surface:** 10+ artifacts with `modified: 2026-05-22` per `rg '^modified:\s*"?2026-05-22"?' .factory/`.

**Evidence:** Includes 4 stories + 6 BC files. F-LP9-MED-001 covers the PLUGIN-MIGRATION-001-E story instance; the 9 sibling files require intent verification (may be correctly dated for their actual last-modification).

**Routing:** orchestrator adjudicate. Pending intent verification — sibling files likely correctly dated.

`[pending intent verification]`

### F-LP9-MED-003 — E-PLUGIN-022 error message conflates two distinct operator-triage signals

**Surface:** `crates/prism-core/src/error.rs:1140-1144`; `crates/prism-spec-engine/src/plugin/mod.rs:1137-1138`; `error-taxonomy.md:473`.

**Evidence:** Error message "or" clause groups (1) guest AuthError::ResponseParse and (2) missing kv_set call into one logged event. Both trigger same event_type / E-code / message. Host has no information channel to distinguish.

**Why it fails:** Operator audit observability accuracy — exact problem F-LP8-MED-002 fixed (E-PLUGIN-008 mixing) reproduced within E-PLUGIN-022 boundary.

**Routing:** product-owner adjudicates options: (a) narrow message dropping "or missing kv_set call" clause, or (b) add guest-side audit emission for distinction. Pending intent verification.

`[pending intent verification]`

### F-LP9-LOW-001 — Orphan deletion-marker comment still names deleted test

**Surface:** `crates/prism-spec-engine/plugins/crowdstrike-oauth2/src/lib.rs:1076-1081`.

**Evidence:** FB-IMPL-7 report claimed comment block at lines 1076-1078 removed; actually relocated/restructured at lines 1076-1081 with deletion-marker narrative.

**Routing:** orchestrator note inaccuracy in FB-IMPL-7 report OR implementer delete the comment block. Pending intent verification (the comment is useful documentation).

`[pending intent verification]`

### F-LP9-OBS-001 — debug_assert! vs assert! choice for WAT-fixture defense-in-depth

**Surface:** F-LP9-HIGH-001 re-closure surface at `plugin/mod.rs:695+`.

**Evidence:** debug_assert! compiles out in release builds — production code reaching the branch silently takes it. Production-grade default suggests assert! (panics in release too) or full #[cfg(test)] branch gating.

**Routing:** orchestrator + implementer evaluate during F-LP9-HIGH-001 closure.

## Decay trajectory

| Pass | Findings | Severity high-water | New axis surfaced |
|---|---|---|---|
| 1 | 20 | 4 CRIT, 7 HIGH | code-level review |
| 2 | 12 | 2 CRIT, 5 HIGH | wire-up verification |
| 3 | 3 | 0 CRIT, 1 HIGH | wit-bindgen exports |
| 4 | 0 | — | false CLEAN |
| 5 | 2 | 0 CRIT, 2 HIGH | structural-coverage verification |
| 6 | 3 | 0 CRIT, 0 HIGH, 2 MED, 1 LOW | EC-test-vs-spec fidelity (test-body) |
| 7 | 3 | 0 CRIT, 0 HIGH, 2 MED, 1 LOW | fidelity sub-dims: spec-emission/deferral/sibling |
| 8 | 7 | 0 CRIT, 0 HIGH, 4 MED, 2 LOW, 1 OBS | test-as-paper-fix; error-variant fidelity; format-specifier; orphan sweep; emission reachability |
| 9 | **6** | **0 CRIT, 1 HIGH, 3 MED, 1 LOW, 1 OBS** | **paper-fix axis applied to FB report claims — structural-coverage of code-claim verification; modified-date staleness class; operator-triage conflation recurrence** |

Severity high-water REGRESSED to HIGH (was 0 HIGH for 3 consecutive passes) — true regression because FB-IMPL-7's F-LP8-LOW-001 closure didn't land.

## Recommended next action

Dispatch FB-IMPL-8 to close 6 findings. Critical: orchestrator MUST independently grep for the actual macro call after implementer reports F-LP9-HIGH-001 closure (paper-fix re-detection discipline).

## Total counts

| Severity | Count |
|---|---|
| HIGH | 1 |
| MED | 3 |
| LOW | 1 |
| OBS | 1 |
| **TOTAL** | **6** |
