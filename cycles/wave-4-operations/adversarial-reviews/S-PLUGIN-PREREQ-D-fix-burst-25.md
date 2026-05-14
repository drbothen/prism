---
document_type: fix-burst-closure-report
target_artifact: S-PLUGIN-PREREQ-D
fix_burst_N: 25
target_pass: 27
findings_closed: 4 (3 MEDIUM F-LP27-MED-001 subsystems SS-16 anchor + F-LP27-MED-002 PluginError #[non_exhaustive] MVP-hedge + F-LP27-MED-003 References POL-7 paraphrase 7/8; 1 LOW F-LP27-LOW-001 inputs BC-2.17.005 gap)
findings_closed_burst_a: 0
findings_deferred: 1 OBS (F-LP27-OBS-001 [process-gap] POL-7 cross-table sweep — codification candidate #13; routed to cycle-close session-reviewer adjudication)
producer: state-manager (orchestrator-coordinated; story-writer Stage 1 + state-manager Stage 2 — single commit per TD-VSDD-053)
story_v_before: 1.24
story_v_after: 1.25
factory_shas: [9a18c2bd, 19236864, "TBD (see STATE.md D-518 row)"]
trajectory: "16 → 8 → 6 → 4 → 0 → 4 → 7 → 4 → 2 → 2 → 2 → 1 → 1 → 1 → 3 → 6 → 4 → 4 → 4 → 1 → 1 → 1 → 1 → 0 → 4 → 1 → 4 → CLOSED"
next_action: "Adversary pass-28 dispatch — target streak 0/3 → 1/3 if CLEAN; apply codifications #11 (open-and-grep) + #12 (BC body-table titles verbatim) + #13 (POL-7 cross-table sweep — verify BC title verbatim in §References + Architecture Compliance Rules + frontmatter comments + Architecture Mapping + Match-Site Inventory descriptions). Standard POL-22 Phase A 30+anchor + Phase B 5-chain + Phase C carry-forward + Phase D novelty."
codification_candidate_13: "POL-7 cross-table sweep — for each BC title citation in a story (body BC table, §References, Architecture Compliance Rules, prose, frontmatter comments), verify the citation matches BC H1 verbatim. Pass-27 surfaced this as sibling pattern to codification #12 (which swept body BC table only). Adversary discovered: fix-burst-24 closed body BC table 8/8 verbatim; §References section still had 7/8 paraphrased."
---

# S-PLUGIN-PREREQ-D Fix-Burst-25 Closure Report

**Fix-burst-25 CLOSED: 4/4 in-scope findings (3 MED + 1 LOW); 1 OBS routed to cycle-close session-reviewer**
**Dispatch: story-writer (Stage 1 @ story v1.24 → v1.25) + state-manager (Stage 2 — this commit)**
**23rd consecutive single-commit-with-TBD-pin (TD-VSDD-053; F-LP10-OBS-001 DECISIVELY STABLE)**

---

## Closures Table

| Finding | Severity | Closed By | Stage | Method |
|---------|----------|-----------|-------|--------|
| F-LP27-MED-001 | MEDIUM | story-writer | 1 | `subsystems:` frontmatter updated `[SS-22, SS-17]` → `[SS-22, SS-17, SS-16]`; YAML comment block updated with SS-16 justification (BC-2.16.002 anchors SS-16 plugin-runtime subsystem per PREREQ-B precedent `[SS-16, SS-01]`; AC-16 implements in prism-spec-engine which maps to SS-16). Symmetric `anchor_subsystem:` field also updated `[SS-22, SS-17]` → `[SS-22, SS-17, SS-16]` per TD-VSDD-060 sibling-site discipline. |
| F-LP27-MED-002 | MEDIUM | story-writer | 1 | `PluginError` `#[non_exhaustive]` conditional MVP-hedge language replaced with direct prescription — story §non_exhaustive Requirements section now lists PluginError enum-level `#[non_exhaustive]` unconditionally, aligning with PrismError pattern at `error.rs:15-17` and the 30+-type compile-fail perimeter audit (EXPECTED=30 gate). MVP-conditional language ("if the production-grade principle applies") stripped entirely; direct requirement substituted. |
| F-LP27-MED-003 | MEDIUM | story-writer | 1 | §References section lines 1008-1015 rewritten: 7/8 BC titles updated from paraphrased sub-scope labels to verbatim BC H1 titles per POL-7. BC-2.17.007 parenthetical annotation preserved (annotation adds story-context, does not replace title). Sibling pattern to codification #12 (body BC table sweep) — codification #13 addresses the wider POL-7 cross-table scope. |
| F-LP27-LOW-001 | LOW | story-writer | 1 | `inputs:` frontmatter array: BC-2.17.005-plugin-hot-reload-atomic-swap.md inserted between BC-2.17.004 and BC-2.17.006. Finding root: fix-burst-23 re-anchored spawn_blocking to BC-2.17.005 §Invariants, but did not add BC-2.17.005 to `inputs:` — sibling-site sweep miss (fix-burst-23 swept 3/3 axes but missed frontmatter as a citation site for this BC). |

## Deferred Findings

| Finding | Severity | Routing | Rationale |
|---------|----------|---------|-----------|
| F-LP27-OBS-001 | OBS | cycle-close session-reviewer adjudication | [process-gap] codification candidate #13 (POL-7 cross-table sweep — verify BC title verbatim in §References + Architecture Compliance Rules + frontmatter comments + Architecture Mapping + Match-Site Inventory + prose). Not a content defect in the story — a gap in the adversary verification protocol. Session-reviewer decides whether to (a) amend POL-7 with explicit scope language, (b) extend POL-22 Phase B further, or (c) retire as subsumed by codification #12's intent. Does NOT enter tech-debt-register (no human direction to defer; no concrete future dependency anchor). |

---

## Story-Writer Stage 1 Detail

**Factory SHAs (prior commits in cascade):** 9a18c2bd (fix-burst-23 closure), 19236864 (D-517 pass-27 BLOCKED report)
**Story transition:** v1.24 → v1.25

### F-LP27-MED-001 — subsystems: SS-16 Anchor Fix + Symmetric anchor_subsystem: Update

**Root cause:** `subsystems:` frontmatter listed `[SS-22, SS-17]`. BC-2.16.002 (Multi-Step Fetch Pipeline Execution — Sequential Steps with Variable Interpolation) is anchored to SS-16 (WASM Plugin Runtime) in the BC-INDEX — the same BC is one of the 8 primary BCs for PREREQ-D. AC-16 (PluginRuntime integration into PipelineExecutor) implements directly in prism-spec-engine, which maps to SS-16. The PREREQ-B story established the precedent pattern: `subsystems: [SS-16, SS-01]` because BC-2.16.002 governs the plugin-runtime step.

**Before/After (subsystems frontmatter):**

| Field | Before | After |
|-------|--------|-------|
| `subsystems:` | `[SS-22, SS-17]` | `[SS-22, SS-17, SS-16]` |
| `anchor_subsystem:` | `[SS-22, SS-17]` | `[SS-22, SS-17, SS-16]` |
| YAML comment block | SS-22 + SS-17 only | SS-22 + SS-17 + SS-16 with BC-2.16.002 anchor justification |

**Sibling-site discipline (TD-VSDD-060):** `anchor_subsystem:` is the symmetric field — both updated atomically in the same story-writer Stage 1 edit. No asymmetric state created.

**PREREQ-B precedent:** The PREREQ-B story used `subsystems: [SS-16, SS-01]` because BC-2.16.002's plugin-runtime anchor was established there. PREREQ-D inherits the same BC and the same SS-16 mapping — the subsystems field must reflect this structural dependency.

---

### F-LP27-MED-002 — PluginError #[non_exhaustive] MVP-Hedge Stripped

**Root cause:** The story's §non_exhaustive Requirements section contained conditional language: "PluginError SHOULD carry `#[non_exhaustive]` if the production-grade principle applies to this enum." This is an MVP-hedge — it makes a mandatory requirement conditional on a principle that is never NOT in effect (per CLAUDE.md, production-grade is always the default). Additionally, `PrismError` at `error.rs:15-17` unconditionally carries `#[non_exhaustive]` as established precedent; PluginError is a sibling error type adding 4 new variants in this story and must match the pattern.

**Before/After (§non_exhaustive Requirements):**

| Site | Before | After |
|------|--------|-------|
| §non_exhaustive Requirements — PluginError row | "SHOULD carry `#[non_exhaustive]` if production-grade principle applies (conditional)" | "`#[non_exhaustive]` required — unconditional. Matches PrismError pattern at `error.rs:15-17`. Compile-fail gate EXPECTED=30 perimeter audit governs coverage count." |
| §non_exhaustive Requirements — enum-level subsection | absent | Added: PluginError enum-level `#[non_exhaustive]` listed as unconditional requirement |

**Connection to F-LP22-OBS-001 (prior phase-5 deferral):** F-LP22-OBS-001 was deferred at D-488 to phase-5 because it required architect evaluation of compile-fail gate EXPECTED=30 impact. The present finding (F-LP27-MED-002) is a DIFFERENT defect: the story's prescription language was conditional/hedged, not the implementation. Fix-burst-25 corrects the story prescription. The EXPECTED=30 gate impact (F-LP22-OBS-001) remains in the phase-5 deferred register — implementer will see the unconditional prescription in v1.25 and handle the gate count per the compile-fail gate pattern.

---

### F-LP27-MED-003 — §References POL-7 Paraphrase 7/8 BC Titles

**Root cause:** After fix-burst-24 canonicalized the BC body-table Title cells (codification #12), pass-27 fresh-context adversary applied codification #12 discipline to the §References section and found 7/8 BC entries still used paraphrased sub-scope labels rather than verbatim BC H1 titles.

**Before/After (representative samples — all 7 corrected):**

| BC ID | Before (paraphrased) | After (verbatim BC H1) |
|-------|---------------------|------------------------|
| BC-2.16.002 | "Multi-Step Fetch Pipeline — Event Catalog Integration" | "Multi-Step Fetch Pipeline Execution — Sequential Steps with Variable Interpolation" |
| BC-2.17.001 | "Plugin Load Sequence — Unsigned Plugin Boot" | "Plugin Load Pipeline — Unsigned Plugin Validation and Boot Sequence" |
| BC-2.17.002 | "WASM Plugin Runtime — URL Allowlist" | "WASM Plugin Runtime — HTTP URL Allowlist Enforcement" |
| BC-2.17.003 | "Plugin Signature — Future Reserved" | "Plugin Signature Verification — Reserved for Phase 2 (Signed Plugin Support)" |
| BC-2.17.004 | "Plugin Boot Warning — Audit Log" | "Plugin Boot Warning Emission — Audit Log Entry on Unsigned Plugin Load" |
| BC-2.17.006 | "PluginRuntime — Boot Integration" | "PluginRuntime Boot Integration — prism-bin Initialization Sequence Wiring" |
| BC-2.22.001 | "Plugin TOML Spec — Sensor Definition Format" | "Plugin TOML Specification — Sensor Definition Schema and Validation Contract" |

**BC-2.17.007 status:** BC-2.17.007 already had a verbatim title with a parenthetical annotation that adds story context. The parenthetical annotation pattern is preserved — annotations that follow the verbatim title are permissible per the codification #12 precedent established at fix-burst-24. Only paraphrase-replacement (substituting a different phrase for the H1) is a POL-7 violation.

---

### F-LP27-LOW-001 — inputs: BC-2.17.005 Gap (Sibling-Site Sweep Miss from fix-burst-23)

**Root cause:** Fix-burst-23 re-anchored the spawn_blocking prescription from the fabricated `ADR-023 §C4` to the canonical `BC-2.17.005 §Invariants`. The story body now cites BC-2.17.005 at two sites (lines 980 and 1012). However, the `inputs:` frontmatter array was not updated to include BC-2.17.005 — the story's inputs array listed BCs 2.17.001/002/003/004/006/007 but skipped 2.17.005. This means the story declares a dependency (in body) that its frontmatter does not register.

**Fix:** BC-2.17.005-plugin-hot-reload-atomic-swap.md inserted at the correct alphabetical position between BC-2.17.004 and BC-2.17.006 in the `inputs:` array.

**Sibling-site sweep miss analysis:** Fix-burst-23 swept 3/3 declared axes (body prescription sites, AC-9 trace header, architecture compliance rules row). The `inputs:` array is a 4th axis that was not included in the fix-burst-23 sweep scope. This is a process gap: when a body citation is added or changed, the `inputs:` array must be swept as a sibling site. Codification #11 (open-and-grep) and #12 (BC body-table verbatim) both apply to body content; neither explicitly covers frontmatter inputs as a citation-site axis. Pass-27 surfaced this implicitly as part of the §References sweep scope expansion.

---

## Frontmatter Update

| Field | Before | After |
|-------|--------|-------|
| `version` | `"1.24"` | `"1.25"` |
| `timestamp` | `"2026-05-13T14:00:00Z"` | `"2026-05-13T17:00:00Z"` |
| `subsystems:` | `[SS-22, SS-17]` | `[SS-22, SS-17, SS-16]` |
| `anchor_subsystem:` | `[SS-22, SS-17]` | `[SS-22, SS-17, SS-16]` |
| `inputs:` | BC-2.17.005 absent | BC-2.17.005-plugin-hot-reload-atomic-swap.md inserted |
| Changelog | — | v1.25 row inserted above v1.24 row |

---

## Sibling-Site Sweep (TD-VSDD-060) — 5/5 CLEAN

| Sweep Axis | Target | Result |
|------------|--------|--------|
| subsystems YAML comment block | SS-16 justification added | CLEAN — comment updated symmetrically |
| anchor_subsystem: field | `[SS-22, SS-17, SS-16]` | CLEAN — symmetric with subsystems: |
| BC-2.17.005 inputs: entry | Inserted between BC-2.17.004 and BC-2.17.006 | CLEAN — alphabetical position correct |
| §References BC titles (8/8 verbatim check) | 7/8 corrected + BC-2.17.007 annotation preserved | CLEAN — all 8 now verbatim or annotation-pattern |
| PluginError #[non_exhaustive] prescription | Unconditional in §non_exhaustive Requirements | CLEAN — no conditional language remains |

---

## F-LP27-OBS-001 Cross-Reference (Routed Cycle-Close)

F-LP27-OBS-001 is a process-gap observation, not a story content defect. It identifies that codification #12 (BC body-table title verbatim) was scoped to the body BC table only, while the §References section uses the same title-citation pattern and is governed by the same POL-7 requirement. Fix-burst-25 closed the §References gap as F-LP27-MED-003. The process observation is: codification #12's verification discipline should have included §References as an equal-weight citation site.

The session-reviewer will adjudicate at cycle-close whether to:
- Amend POL-7 with explicit enumeration of all citation-site scopes
- Extend POL-22 Phase B to cover all citation sites (body BC table + §References + Architecture Compliance Rules + frontmatter comments + prose)
- Retire codification candidate #13 as naturally subsumed by codification #12's intent (the "verbatim BC title" rule applies wherever a BC is cited, regardless of section)

This is the 13th process-gap codification candidate. Count at cycle-close: 13 active candidates.

---

## Process-Gap Codification Cascade Insight

**Codification #12 → #13 scope-widening pattern:**

Codification #12 was authored at fix-burst-24 with a narrow scope: "verify each BC body-table Title cell against BC H1 verbatim." Pass-27 adversary applied #12 to the body BC table (8/8 CLEAN per fix-burst-24) and then independently verified §References using the same principle. The adversary found 7/8 paraphrased titles in §References — a structural sibling to the body BC table finding.

This is the same scope-widening dynamic that produced codification #11 from codification #8 (Phase A anchor-content verification from Phase A anchor-existence verification). The pattern: an adversary protocol rule is stated narrowly, catches one class of site, and a later fresh-context adversary discovers a wider scope class that the narrow rule didn't cover.

The policy implication: POL-7 already states "BC titles must be cited verbatim." The codification candidates (#12, #13) are not contradicting POL-7 — they are making explicit which citation sites fall under POL-7 scope. Session-reviewer's adjudication should probably be: amend POL-7 §Scope to enumerate all citation sites, so that future adversary passes have a single authoritative list rather than an accumulating stack of codification candidates.

---

## Convergence Status

- **Pass-27:** BLOCKED (4 findings: 0C/0H/3M/1L/1OBS) — 3 new finding classes surfaced after 26 prior passes
- **Fix-burst-25:** CLOSED — 4/4 in-scope (3M+1L); 1 OBS routed to cycle-close session-reviewer
- **Streak:** 0/3 HOLD — fix-burst-25 does not advance streak; pass-28 next
- **Trajectory:** `16 → 8 → 6 → 4 → 0 → 4 → 7 → 4 → 2 → 2 → 2 → 1 → 1 → 1 → 3 → 6 → 4 → 4 → 4 → 1 → 1 → 1 → 1 → 0 → 4 → 1 → 4 → CLOSED`
- **Next action:** Adversary pass-28 at story v1.25. Apply codifications #11 + #12 + #13. Standard POL-22 Phase A 30+anchor + Phase B 5-chain + Phase C carry-forward + Phase D novelty.

**Pass-28 verification checklist:**
- MUST apply codification #11 (open-and-grep cited target documents; story-body substring match is NOT sufficient)
- MUST apply codification #12 (verify all 8 BC body-table Title cells against canonical BC H1 verbatim — confirmed CLEAN after fix-burst-24; regression-check only)
- MUST apply codification #13 (verify BC title verbatim in §References + Architecture Compliance Rules + frontmatter comments + Architecture Mapping + Match-Site Inventory descriptions)
- MUST verify `subsystems: [SS-22, SS-17, SS-16]` and `anchor_subsystem: [SS-22, SS-17, SS-16]` — symmetric
- MUST verify PluginError `#[non_exhaustive]` prescription is unconditional (no MVP-hedge language)
- MUST verify `inputs:` includes BC-2.17.005-plugin-hot-reload-atomic-swap.md
- Standard carry-forward: 5 phase-5 deferred findings unchanged; 13 codification candidates active
