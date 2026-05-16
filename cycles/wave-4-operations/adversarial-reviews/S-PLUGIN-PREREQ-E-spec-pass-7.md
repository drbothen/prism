---
document_type: adversarial-review
story: S-PLUGIN-PREREQ-E
pass: 7
scope: spec
verdict: BLOCKED
total_findings: 12
severity_breakdown:
  critical: 0
  high: 4
  medium: 4
  low: 0
  observation: 4
in_scope_findings: 8
observations_queued: 4
produced_by: adversary
reviewed_at: 2026-05-16
fix_burst: fix-burst-7
fix_burst_closed_at: pending
streak_after_pass: "0/3"
---

# S-PLUGIN-PREREQ-E Adversarial Spec Review — Pass 7

**Verdict: BLOCKED — 8 in-scope findings (4 HIGH + 4 MEDIUM) + 4 OBS. Streak resets to 0/3.**

Pass-7 fresh-context surfaces NOVEL classes that fix-burst-6 introduced or left unresolved:

1. **F-LP7-HIGH-001 (within-FB6 sibling-sweep asymmetry):** ADR-026 bumped v1.7→v1.8 in fix-burst-6. Architect's VP-156 edits in the same burst pinned all 4 live-narrative cites to `ADR-026 D7 v1.7` (the pre-bump version). BC-2.16.012 §Verification Properties was correctly swept to v1.8 in the same burst. The asymmetry (BC swept; VP not) is a new POL-23 defect class: within-burst version-pin-order gap. The orchestrator dispatch predicted this; pass-7 confirmed.

2. **F-LP7-HIGH-002 (TD-VSDD-059 paper-fix detection):** F-LP6-MED-004 adjudication updated EC-016-011-005 Resolution cell in BC-2.16.011 (bumping `deprecated_by: ADR-023 → ADR-027` and adding ADR-027 to removal_reason). But three LOAD-BEARING implementer-facing sites were NOT swept: (a) BC-2.16.011 §Postconditions removal_reason text; (b) Story §Tasks Task 8; (c) Story §Acceptance Criteria AC-6. An implementer following the story Task 8 + AC-6 verbatim would leave BC-2.16.004 `deprecated_by` as ADR-023 (contradicting the adjudication) and would write the stale removal_reason. SOUL.md #4 partial-fix anti-pattern. Story-writer + product-owner missed sibling-sweep.

3. **F-LP7-HIGH-003 (3-file changelog monotonic-order pattern within FB6):** ADR-026 v1.8 row, ADR-027 v1.4 row, and VP-155 v0.4 row all appear BEFORE their predecessor v1.7/v1.3/v0.3 rows in the §Changelog tables. POL-20 monotonic-strict-ascending violation across 3 distinct files all touched in the same burst (architect's three-file edit set). Same defect class as F-LP5-HIGH-003 renumber-repair-redo. Pattern flag: 3-site within-burst is a recurring failure mode of the multi-file architect commit pattern.

4. **F-LP7-HIGH-004 (phantom-prune-but-not-add pattern — POL-22 Phase C):** F-LP6-HIGH-003 correctly pruned the phantom "Add SensorAuth re-export" entry from ADR-026 `runtime_deliverables:` (it was pre-existing). But F-LP6-HIGH-003 did NOT add the genuinely-missing deliverables: ADR-026 D1 mandates adding `fn auth_type_name(&self) -> &'static str;` method to the SensorAuth trait, and D2 mandates four impl bodies in `crowdstrike.rs`, `cyberint.rs`, `claroty.rs`, `armis.rs`. The story §File Structure Requirements DOES enumerate these (claims to trace back to ADR-026 D1 Path B), but ADR-026's own `runtime_deliverables:` (source-of-truth for delivery scope) does not. The "phantom prune sometimes also needs to ADD what was rightly there but mis-phrased" class.

5. **F-LP7-MED-001/002/003/004 (sibling-sweep patterns):** 4 modified-field date drift across BC-2.16.011/BC-2.16.012/VP-155/VP-156; ADR-026 D5 phantom validation-activity-not-delivery; ARCH-INDEX rows for ADR-026 + ADR-027 diverge from H1 verbatim; BC-2.16.011 §Architecture Anchors + Story §References cite "ADR-027 §D5" for VP-154 but D5 covers spec_parser.rs scope (FB4 expanded D5 scope; sibling-sweep missed BC + Story narratives).

6. **OBS-LP7-001..004:** OBS-LP6-001 POL-7 surface-6 codification candidate elevated to HIGH-priority cycle-close per FB6 introducing 2 new mismatch sites; ADR-026 D7 RwLock `unwrap()` observation (acceptable per established exception); BC-2.16.012 v1.7 changelog cites F-LP6-CRIT-001 as driver for D7 sibling-sweep — correct attribution noted.

CASCADE LENGTH NOTE: 7 passes deep. Trajectory **14→9→8→9→10→10→FB6-CLOSED→8**. Pass-7 count is the LOWEST since pass-3, but novel finding classes remain. The within-FB6 sibling-sweep asymmetry (F-LP7-HIGH-001) and the paper-fix detection (F-LP7-HIGH-002) are the highest-priority closures. BC-5.39.001 3-CLEAN protocol: streak resets 0/3, pass-8 NEXT.

---

## Finding Inventory

### F-LP7-HIGH-001 — VP-156 ADR-026 D7 pin v1.7 stale; ADR-026 is v1.8 (within-FB6 sibling-sweep asymmetry)

**Severity:** HIGH
**Type:** POL-23 (D-571 amendment) within-burst version-pin-order gap; NEW defect class for FB6 sibling-sweep
**Anchor policies:** POL-23 (bc_version_bump_sibling_grep_gate, extended to ADR cites), TD-VSDD-060 (sibling-site sweep on value changes)
**Routing:** architect (POL-23 ADR-version-bump sibling sweep)

**Evidence:**
- `/Users/jmagady/Dev/prism/.factory/specs/verification-properties/vp-156-write-tool-registration-uniqueness.md` 4 live-narrative cites of `ADR-026 D7 v1.7`: §Property Statement, §Source Contract BC row, §Source Contract ADR row, proof-harness skeleton comment
- `/Users/jmagady/Dev/prism/.factory/specs/behavioral-contracts/BC-2.16.012-plugin-registry-dispatch-migration.md` §Verification Properties VP-156 row: `per ADR-026 D7 v1.8` (correct)
- `/Users/jmagady/Dev/prism/.factory/specs/architecture/decisions/ADR-026-sensorauth-unsealing.md` frontmatter `version: "1.8"` (current ADR version)

**Why this is the new POL-23 defect class:** Architect bumped ADR-026 v1.7→v1.8 (cookie_roundtrip + phantom-prune + semver-stance) and VP-156 v0.4→v0.5 (D7 pin sweep) in the SAME commit. The VP-156 sweep targeted "stale v1.2 → current v1.7" — but in the same burst the "current" became v1.8. The fix-burst execution order produced an in-progress-version snapshot in VP-156. The orchestrator dispatch predicted this; pass-7 confirmed.

**Fix sketch:** Architect updates VP-156 4 live-narrative cites to `ADR-026 D7 v1.8`. Bump VP-156 v0.5 → v0.6 with §Changelog row noting the within-FB6 asymmetry catch.

---

### F-LP7-HIGH-002 — BC-2.16.011 §Postconditions + Story Task 8 + AC-6 carry STALE removal_reason; F-LP6-MED-004 paper-fix

**Severity:** HIGH
**Type:** TD-VSDD-059 paper-fix detection; multi-site partial-fix (3 load-bearing sites)
**Anchor policies:** TD-VSDD-059, TD-VSDD-060
**Routing:** product-owner (BC body + story Task + AC are PO-domain narratives)

**Evidence:**
- BC-2.16.011 §Postconditions (`/Users/jmagady/Dev/prism/.factory/specs/behavioral-contracts/BC-2.16.011-customadapter-rust-trait-retirement.md` §Postconditions paragraph): `removal_reason: "PREREQ-E retirement per ADR-023 Rule 5"` — stale; missing ADR-027 §Decision citation
- Story Task 8 (`/Users/jmagady/Dev/prism/.factory/stories/S-PLUGIN-PREREQ-E-unseal-sensor-auth-deprecate-customadapter.md` §Tasks Task 8): `Add removal_reason: "PREREQ-E retirement per ADR-023 Rule 5"` — stale; no `deprecated_by: ADR-023 → ADR-027` instruction
- Story AC-6 (same file §Acceptance Criteria AC-6): same stale removal_reason text
- BC-2.16.011 EC-016-011-005 Resolution cell (same BC, EC table): correctly bumped to `deprecated_by: ADR-027` + ADR-027 §Decision in removal_reason (FB6 fix)

**Why this is HIGH paper-fix:** The FB6 changelog claim is "F-LP6-MED-004 EC-016-011-005 deprecated_by adjudicated to ADR-027." But the only edit was the EC table Resolution cell. The IMPLEMENTER-FACING sites (story Task 8, story AC-6, BC §Postconditions) were never swept. An implementer following the story Task 8 verbatim would (a) write the stale removal_reason and (b) leave deprecated_by as ADR-023 (since no instruction to bump it).

**Fix sketch:** Product-owner sweeps 3 narrative sites. Add `deprecated_by: ADR-023 → ADR-027` instruction to story Task 8. Update removal_reason at all 3 sites to `"PREREQ-E retirement per ADR-027 §Decision + ADR-023 Rule 5"`. Bump BC-2.16.011 v1.3 → v1.4 and story v1.7 → v1.8.

---

### F-LP7-HIGH-003 — 3-file changelog monotonic-order regression introduced by FB6

**Severity:** HIGH (3-site within-burst pattern)
**Type:** POL-20 monotonic-strict-ascending violation; same defect class as F-LP5-HIGH-003 renumber-repair-redo
**Anchor policies:** POL-20
**Routing:** state-manager (POL-20 catch class; previously routed to state-manager renumber-repair)

**Evidence:**
- ADR-026 §Changelog: v1.8 row appears BEFORE v1.7 row
- ADR-027 §Changelog: v1.4 row appears BEFORE v1.3 row
- VP-155 §Changelog: v0.4 row appears BEFORE v0.3 row

**Why this is HIGH not MED:** 3-site within-burst pattern flag (the "Story Frontmatter-Body Coherence" sibling-sweep heuristic: 3+ sites = HIGH).

**Fix sketch:** State-manager reorders changelog table rows in all 3 files to monotonic strict ascending by version. No semantic content changes; bumps not required (cosmetic ordering only).

---

### F-LP7-HIGH-004 — ADR-026 `runtime_deliverables:` missing trait method addition + 4 impl-body additions (the concrete PREREQ-E deltas)

**Severity:** HIGH
**Type:** POL-22 Phase C (anchor_content_lexical_vs_semantic — semantic completeness of runtime_deliverables); phantom-prune-but-not-add sibling class to F-LP6-HIGH-003
**Anchor policies:** POL-22 Phase C
**Routing:** architect (runtime_deliverables is architect-owned ADR frontmatter)

**Evidence:**
- ADR-026 §D1 (`/Users/jmagady/Dev/prism/.factory/specs/architecture/decisions/ADR-026-sensorauth-unsealing.md` §D1 narrative): mandates ADD `fn auth_type_name(&self) -> &'static str;` method to SensorAuth trait
- ADR-026 §D2 (same file §D2 narrative): mandates 4 impl bodies in `crates/prism-sensors/src/auth/{crowdstrike,cyberint,claroty,armis}.rs`
- ADR-026 frontmatter `runtime_deliverables:`: 8 items, NONE of which describe the trait method addition OR the 4 impl-body additions
- Story §File Structure Requirements: DOES enumerate the 4 impl files + the trait method addition, claims trace to "ADR-026 D1 Path B"
- Live code (`/Users/jmagady/Dev/prism/crates/prism-sensors/src/lib.rs` doc comment line 22): `SensorAuth is sealed` — confirms current code lacks both the unsealing AND the auth_type_name() addition. Both are real PREREQ-E deltas.

**Why this is HIGH:** F-LP6-HIGH-003 pruned a phantom (SensorAuth re-export pre-existing) but did NOT add the genuinely-missing deliverables. The story §File Structure Requirements traces back to ADR-026 D1 — but the ADR's own runtime_deliverables (source-of-truth) does not enumerate these. An implementer reading ADR-026 first (before story) would miss the trait-method addition + 4 impl bodies entirely.

**Fix sketch:** Architect appends two runtime_deliverables entries (or one consolidated entry covering all 5 file additions). ADR-026 v1.8 → v1.9.

---

### F-LP7-MED-001 — 4 frontmatter `modified:` field date drift across FB6-touched artifacts

**Severity:** MEDIUM (pattern flag — 4-site within-burst staleness; same class as F-LP4-LOW-002 promoted to MED)
**Type:** POL-27 (bc_modified_field_iso_date_format) within-burst sibling-sweep gap
**Anchor policies:** POL-27
**Routing:** state-manager (POL-27 frontmatter canonical sync)

**Evidence:**
- BC-2.16.011 frontmatter `modified: "2026-05-15"`; latest changelog v1.3 dated 2026-05-16
- BC-2.16.012 frontmatter `modified: "2026-05-15"`; latest changelog v1.7 dated 2026-05-16
- VP-155 frontmatter `modified: "2026-05-15"`; latest changelog v0.4 dated 2026-05-16
- VP-156 frontmatter `modified: "2026-05-15"`; latest changelog v0.5 dated 2026-05-16

**Fix sketch:** State-manager updates 4 frontmatter `modified:` fields to `"2026-05-16"`. No version bumps required (frontmatter-only edit).

---

### F-LP7-MED-002 — ADR-026 D5 runtime_deliverable phrased as validation activity, not code delivery

**Severity:** MEDIUM (phantom-runtime-deliverable sibling class)
**Type:** POL-22 Phase A + Phase C
**Anchor policies:** POL-22 Phase A, Phase C
**Routing:** architect (runtime_deliverables is architect-owned)

**Evidence:**
- ADR-026 frontmatter `runtime_deliverables:`: entry `"Validate PluginRuntime::load_plugin wiring path calls SensorAuth-implementing types"` is a verification activity verb (Validate), not a code delivery
- PluginRuntime::load_plugin already exists (PREREQ-D merged 2026-05-15, PR #149)

**Fix sketch:** Architect either prunes (if pre-existing wiring is sufficient) or re-phrases to concrete delivery (e.g., "Confirm Linker registration in PluginRuntime::load_plugin admits SensorAuth-implementing types per BC-2.17.* WIT validation"). Architect adjudicates.

---

### F-LP7-MED-003 — ARCH-INDEX ADR Registry rows for ADR-026 + ADR-027 diverge from H1 verbatim titles (POL-7 surface-6)

**Severity:** MEDIUM (2-site sibling pattern; OBS-LP6-001 candidate elevation per pass-7 evidence)
**Type:** POL-7 verbatim-H1 surface-6 candidate (pass-6 OBS class) promoted to MED per FB6 introducing 2 new mismatches
**Anchor policies:** POL-7 (D-571 amendment surface enumeration extension candidate)
**Routing:** architect

**Evidence:**
- ADR-026 H1: `ADR-026: SensorAuth Trait Un-Sealing — Remove private::Sealed, Enable Plugin Auth Implementations`
- ARCH-INDEX row Title: `SensorAuth Un-Sealing — Open Trait for Plugin-Implementable Auth; RwLock WriteToolInvalidationMap`
- ADR-027 H1: `ADR-027: CustomAdapter Rust Trait Deprecation and Wave 1/A Removal — Sole Escape Hatch is .prx WASM`
- ARCH-INDEX row Title: `CustomAdapter Deprecation and Removal — Trait Retirement, Registry Cleanup, prism-query Scope`

**Fix sketch:** Architect aligns ARCH-INDEX rows to verbatim H1 titles (or amends H1s to match the enriched descriptions if architect prefers). POL-7 D-571 sweep convention: H1 is canonical; downstream rows verbatim-quote.

---

### F-LP7-MED-004 — BC-2.16.011 §Architecture Anchors + Story §References cite "ADR-027 §D5" for VP-154; D5 covers spec_parser.rs scope, not VP-154

**Severity:** MEDIUM (semantic mis-anchor)
**Type:** POL-4 (semantic_anchoring_integrity); FB4 D5 scope expansion sibling-sweep miss
**Anchor policies:** POL-4
**Routing:** product-owner (BC + story §References narrative are PO-domain)

**Evidence:**
- BC-2.16.011 §Architecture Anchors line: `ADR-027 — CustomAdapter deprecation/removal architectural decision; §D3 defines compile-fail perimeter (VP-155) and §D5 defines PluginRuntime behavioral equivalence requirement (VP-154)`
- Story §References line: `[ADR-027] — CustomAdapter deprecation/removal; §D3 compile-fail perimeter (VP-155) + §D5 WASM equivalence (VP-154)`
- ADR-027 §D5 actual title: "Spec_parser.rs scope: verification clean-pass AND hardcoded-sensor-string dispatch audit" (FB4-expanded scope; VP-154 not cited in §D5)
- ADR-027 §Verification Property Anchors section: VP-154 IS cited here (correct location)

**Fix sketch:** Product-owner updates BC-2.16.011 §Architecture Anchors line + Story §References line to cite correct ADR-027 section for VP-154 (§Verification Property Anchors, not §D5). Bump BC-2.16.011 + story versions as needed.

---

### OBS-LP7-001 — `[process-gap]` POL-7 surface-6 OBS-LP6-001 cycle-close codification scope-expanded

OBS-LP6-001 (ADR runtime_deliverables-not-in-POL-7-enumeration) was queued for cycle-close codification at OBS tier. FB6 introduced 2 new ARCH-INDEX↔H1 mismatches (F-LP7-MED-003), and the surface-6 codification candidate now has 4+ historical sites. Recommend orchestrator elevate OBS-LP6-001 cycle-close priority from normal to HIGH.

### OBS-LP7-002 — Story §Token Budget Estimate is descriptive only, not load-bearing

§Token Budget Estimate totals ~17,300 tokens within ~40k budget. No finding; observation only.

### OBS-LP7-003 — ADR-026 D7 `RwLock::read().unwrap()` is acceptable per Rust convention

ADR-026 D7 states `.unwrap()` on `RwLock::read()` is infallible-if-no-writer-panics. This is an established exception to the "no unwrap in critical paths" convention; PoisonError is a hard contract violation. No finding.

### OBS-LP7-004 — BC-2.16.012 v1.7 changelog correctly attributes orthogonal-change-driven D7 pin sweep

The v1.7 changelog row cites F-LP6-CRIT-001+HIGH-003+MED-003 as drivers for the D7 sibling-sweep. None of these directly affected D7 semantics, but ADR-026's version bumped because of these orthogonal changes. POL-23 correctly handled the orthogonal-change-driven version bump. Observation only.

---

## Trajectory Summary

| Pass | Findings | In-Scope | OBS Queued | Delta | Note |
|------|----------|----------|------------|-------|------|
| 1 | 14 | 12 | 2 | — | Initial: 1C+4H+5M+2L+2OBS |
| 2 | 9 | 8 | 1 | -5 | 3 FB1 regressions caught |
| 3 | 8 | 8 | 0 | -1 | 5 FB2 sibling-sweep regressions |
| 4 | 9 | 9 | 0 | +1 | FLAT — VP-156 anchor-back gaps |
| 5 | 10 | 7 | 3 | +1 | REGRESSION — FB4 bookkeeping |
| 6 | 10 | 10 | 3 | 0 | FLAT count, NOVEL classes — intra-ADR contradiction |
| 7 | 12 | 8 | 4 | -2 | **DECREASING** — within-FB6 sibling-sweep asymmetry + paper-fix detection + 3-file changelog monotonic gap + phantom-prune-but-not-add |

Trajectory: **14→9→8→9→10→10→FB6-CLOSED→8** (lowest in-scope count since pass-3; cascade making progress).

---

## Artifact Versions After Pass-7 (Pre-Fix-Burst)

| Artifact | Pin | Expected FB7 Bump |
|----------|-----|-------|
| ADR-026 | v1.8 | v1.9 (F-LP7-HIGH-004 runtime_deliverables append + F-LP7-MED-002) |
| ADR-027 | v1.4 | (changelog reorder only — no version bump) |
| BC-2.16.011 | v1.3 | v1.4 (F-LP7-HIGH-002 + F-LP7-MED-004) |
| BC-2.16.012 | v1.7 | — |
| VP-155 | v0.4 | (changelog reorder only) |
| VP-156 | v0.5 | v0.6 (F-LP7-HIGH-001 D7 pin v1.7→v1.8) |
| Story | v1.7 | v1.8 (F-LP7-HIGH-002 + F-LP7-MED-004) |
| ARCH-INDEX | v2.49 | v2.50 (F-LP7-MED-003) |
| BC-INDEX | v4.83 | v4.84 (BC-2.16.011 v1.4 sibling) |
| VP-INDEX | v1.42 | v1.43 (VP-156 v0.6 sibling) |
| STORY-INDEX | v2.110 | v2.111 (story v1.7→v1.8 row tag) |

## Next Step

Fix-burst-7 dispatch: architect (4 findings) + product-owner (2 findings) parallel; state-manager closes (2 findings + STATE bump). Then adversary pass-8 dispatch. BC-5.39.001 3-CLEAN protocol — streak resets 0/3.

Pass-7 report: `cycles/wave-4-operations/adversarial-reviews/S-PLUGIN-PREREQ-E-spec-pass-7.md` (this file).
