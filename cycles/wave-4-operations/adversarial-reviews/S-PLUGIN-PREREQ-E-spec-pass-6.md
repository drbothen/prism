---
document_type: adversarial-review
story: S-PLUGIN-PREREQ-E
pass: 6
scope: spec
verdict: BLOCKED
total_findings: 10
severity_breakdown:
  critical: 1
  high: 3
  medium: 4
  low: 2
  observation: 3
in_scope_findings: 10
observations_queued: 3
produced_by: adversary
reviewed_at: 2026-05-16
fix_burst: fix-burst-6
fix_burst_closed_at: pending
streak_after_pass: "0/3"
---

# S-PLUGIN-PREREQ-E Adversarial Spec Review — Pass 6

**Verdict: BLOCKED — 10 findings (1 CRIT + 3 HIGH + 4 MED + 2 LOW + 3 OBS); 10 in-scope + 3 OBS queued cycle-close (OBS are in-scope but classified as deferrable; CRIT/HIGH/MED/LOW total 10 in-scope must close in fix-burst-6)**

Pass-6 fresh-context surfaces novel anchor-correctness defects that pass 1-5 missed despite their 5-pass coverage:

1. **F-LP6-CRIT-001** — Intra-ADR-026 semantic contradiction: D2 mandates `auth_type_name() -> "cookie"` for ClarotyAuth; D3 enumerated set is `cookie_roundtrip`. Implementer following D2 verbatim breaks E-SPEC-014 structural-match rule. Story §File Structure Requirements propagates D2's stale value.

2. **F-LP6-HIGH-001** — VP-155 `source_bc: null` but BC-2.16.011 §VP Anchors explicitly lists VP-155 as the verification mechanism for INV-ADAPTER-RETIRE-002. Same defect class as F-LP1-CRIT-001 (VP-154 source_bc fix in fix-burst-1); sibling-sweep miss.

3. **F-LP6-HIGH-002** — STORY-INDEX PREREQ-E row tag shows `**draft** v1.5 prereq-e-fix-burst-4`; actual story is v1.6 / fix-burst-5. POL-11 + POL-23 (D-571 amendment) violation — story-version bumps not propagated to STORY-INDEX in fix-bursts 4 and 5.

4. **F-LP6-HIGH-003** — ADR-026 `runtime_deliverables:` lists "Add SensorAuth re-export to prism-sensors public API surface"; live `crates/prism-sensors/src/lib.rs` already re-exports `SensorAuth`. Phantom deliverable misleads implementer scope.

5. Four medium-severity intent-verification items (F-LP6-MED-001..004) and two low-severity items (F-LP6-LOW-001/002) covering stale ADR version pins, ADR-027 subsystem-scope drift, ADR-026 D2/D6 semver-stance consistency, and BC-2.16.011 EC `deprecated_by` adjudication.

6. Three OBS (process-gap candidates) — POL-22 Phase A extension to ADR runtime_deliverables; VP-156 ↔ BC-2.16.012 cross-citation symmetry; SS-17 story-subsystem intent.

CASCADE LENGTH NOTE: 6 passes deep. Trajectory 14→9→8→9→10→10. Count flat but **novel finding classes** at pass-6 (intra-ADR contradiction; runtime_deliverable phantom; STORY-INDEX row staleness; VP source_bc/BC VP-anchor asymmetry). BC-5.39.001 3-CLEAN protocol: streak resets 0/3, pass-7 NEXT.

---

## Finding Inventory

### F-LP6-CRIT-001 — `cookie` vs `cookie_roundtrip` semantic contradiction inside ADR-026

**Severity:** CRITICAL
**Type:** Intra-ADR semantic contradiction; cross-document anchor drift
**Anchor policies:** POL-4 (semantic_anchoring_integrity), POL-22 Phase C (named-entity-existence-verification)
**Routing:** architect (ADR-026 D2 narrative + downstream propagation)

**Evidence:**
- ADR-026 D2 (`/Users/jmagady/Dev/prism/.factory/specs/architecture/decisions/ADR-026-sensorauth-unsealing.md`, §D2 narrative ClarotyAuth code block): `fn auth_type_name(&self) -> &'static str { "cookie" }`
- ADR-026 D3 (same file, §D3 enumerated set narrative): canonical enumerated set is `{oauth2_client_credentials, bearer_static, cookie_roundtrip, api_key, custom_via_plugin}` — `cookie_roundtrip`, NOT `cookie`
- Story §File Structure Requirements (`/Users/jmagady/Dev/prism/.factory/stories/S-PLUGIN-PREREQ-E-unseal-sensor-auth-deprecate-customadapter.md`, claroty.rs row): propagates D2 value `Add fn auth_type_name(&self) -> &'static str { "cookie" }`
- error-taxonomy.md E-SPEC-012 row (`/Users/jmagady/Dev/prism/.factory/specs/prd-supplements/error-taxonomy.md`): "Valid values: oauth2_client_credentials, bearer_static, **cookie_roundtrip**, api_key, custom_via_plugin"
- VP-153 §Property Statement Rule A (`/Users/jmagady/Dev/prism/.factory/specs/verification-properties/vp-153-sensorauth-runtime-cross-composition-prevention.md`): same `cookie_roundtrip` enumerated set

**Why this is a real defect, not a label nit:** If `ClarotyAuth::auth_type_name()` returns `"cookie"` and the spec-load validator enforces ADR-023 Rule 2 Rule C (E-SPEC-014, "resolved credential structural type must structurally match the declared `auth_type`"), then either (i) a `claroty.sensor.toml` with `auth_type = "cookie_roundtrip"` will introspect a credential whose `auth_type_name()` returns `"cookie"`, producing an E-SPEC-014 mismatch and breaking the four-initial-sensor behavioral-equivalence guarantee (INV-SPEC-PARSER-OPEN-002 + AC-8); or (ii) the validator only checks structural shape (not the string name), in which case ADR-026 D2 has chosen a Path B introspection name that is silently incorrect for audit logs — the exact failure mode D2 §Path B rationale was designed to prevent. The two cannot both be true.

**Fix sketch:** Architect picks one — recommend `"cookie_roundtrip"` to match D3 + E-SPEC-012 + VP-153 + canonical TOML. Story-writer propagates story line 335 + AC-2 example chain + any risk_mitigations line in same atomic burst.

---

### F-LP6-HIGH-001 — VP-155 `source_bc: null` contradicts BC-2.16.011 §VP Anchors claim

**Severity:** HIGH
**Type:** Bidirectional traceability drift; sibling-sweep miss (mirror of F-LP1-CRIT-001 VP-154 fix in FB1)
**Anchor policies:** POL-4 (semantic_anchoring_integrity), POL-9 (vp_index_is_vp_catalog_source_of_truth, named-alias semantic sync extension)
**Routing:** architect (set VP-155 source_bc + rewrite §Source Contract)

**Evidence:**
- VP-155 frontmatter `source_bc: null`; §Source Contract: "**BC:** None (pure enforcement property; no behavioral contract)"
- BC-2.16.011 §VP Anchors: "VP-155 (CustomAdapter Absent from prism-spec-engine Public API — compile-fail perimeter; two files asserting E0432 for CustomAdapter + CustomAdapterRegistry; P0; added in PLUGIN-MIGRATION-001-A scope after PREREQ-E merge)"
- BC-2.16.011 §Verification Properties table row: VP-155 explicitly listed
- BC-2.16.011 INV-ADAPTER-RETIRE-002: prism-spec-engine public API does NOT expose any type/trait/function from retired custom_adapter module

**Asymmetry:** BC-2.16.011 claims VP-155 as enforcing INV-ADAPTER-RETIRE-002; VP-155 claims no source BC. Identical defect class to F-LP1-CRIT-001 (VP-154 source_bc was null until FB1 set it to BC-2.16.011). FB1 closed for VP-154 but did not sibling-sweep VP-155.

**Fix sketch:** VP-155 `source_bc: BC-2.16.011`; rewrite §Source Contract to lead with BC-2.16.011 ownership of INV-ADAPTER-RETIRE-002; ADR-027 D3 becomes supporting reference. Bump VP-155 v0.3 → v0.4.

---

### F-LP6-HIGH-002 — STORY-INDEX row stale: shows story v1.5/fix-burst-4; actual story is v1.6/fix-burst-5

**Severity:** HIGH
**Type:** POL-11 + POL-23 (D-571 amendment) sibling-sweep gap
**Anchor policies:** POL-11 (index_bump_required_for_index_mutations), POL-23 (D-571 extension to story versions)
**Routing:** state-manager

**Evidence:**
- Story frontmatter `version: "1.6"` and §Changelog top row v1.6 prereq-e-fix-burst-5
- STORY-INDEX PREREQ-E row tag: `**draft** v1.5 prereq-e-fix-burst-4` (stale)
- STORY-INDEX §Changelog top row v2.109 covers v1.1 D-574 only; fix-burst-4 (v1.4→v1.5) and fix-burst-5 (v1.5→v1.6) both produced bumps not reflected in STORY-INDEX

**Fix sketch:** Update STORY-INDEX PREREQ-E row tag from `**draft** v1.5 prereq-e-fix-burst-4` to `**draft** v1.6 prereq-e-fix-burst-5`. Bump STORY-INDEX v2.109 → v2.110 with §Changelog row covering FB4 + FB5 backfill.

---

### F-LP6-HIGH-003 — ADR-026 `runtime_deliverable` "Add SensorAuth re-export" is already done

**Severity:** HIGH
**Type:** Phantom deliverable in ADR-026 frontmatter; misleads implementer scope
**Anchor policies:** POL-4 (semantic_anchoring_integrity), POL-22 Phase A (lexical-vs-semantic anchor-content verification, extension candidate to ADR runtime_deliverables)
**Routing:** architect

**Evidence:**
- ADR-026 frontmatter `runtime_deliverables:` field: includes `"Add SensorAuth re-export to prism-sensors public API surface"`
- Live `/Users/jmagady/Dev/prism/crates/prism-sensors/src/lib.rs` `pub use auth::{...}` line: `pub use auth::{ArmisAuth, ClarotyAuth, CrowdStrikeAuth, CyberintAuth, SensorAuth};` — SensorAuth already re-exported
- Story §Tasks 1–10: no task includes "add SensorAuth re-export"; Task 1 is sealed-marker removal only
- Story §File Structure Requirements row for `crates/prism-sensors/src/auth/mod.rs`: "Modify | Remove `private::Sealed` module, remove `: Sealed` supertrait" — no re-export addition

**Fix sketch:** Architect either (a) removes the entry, or (b) amends to "Confirm SensorAuth re-export remains in prism-sensors public API surface" if intent was verify-only. Bump ADR-026 v1.7 → v1.8; sibling-sweep ARCH-INDEX ADR registry row + STATE.md ADR-026 version pin.

---

### F-LP6-MED-001 — Stale `ADR-026 D7 v1.2` version pin in VP-156 §Source Contract body prose

**Severity:** MEDIUM
**Type:** POL-23 sibling-grep-gate violation
**Anchor policies:** POL-23 (bc_version_bump_sibling_grep_gate, extended to ADR cites by D7-bump pattern)
**Routing:** architect

**Evidence:**
- VP-156 §Source Contract: `... resolved to error-on-duplicate by ADR-026 D7 v1.2`
- BC-2.16.012 §Verification Properties VP-156 row: `... error-on-duplicate, per ADR-026 D7 v1.7`
- ADR-026 current version: v1.7

**Discrimination note:** v1.2 cite is in body prose (not §Changelog) so TD-VSDD-091 exception does not apply. Reads as present-tense "is anchored in D7 v1.2" — stale.

**Fix sketch:** Update VP-156 §Source Contract `ADR-026 D7 v1.2` → `ADR-026 D7 v1.7`. Bump VP-156 v0.4 → v0.5. Sibling-sweep workspace for other live-narrative `ADR-026 D7 v<1-6>` cites.

---

### F-LP6-MED-002 — ADR-027 §Consequences cites prism-query scope but `subsystems_affected:` excludes SS-07

**Severity:** MEDIUM
**Type:** Intent verification + structural metadata gap (mirror of F-LP5-HIGH-001 ADR-026 fix)
**Anchor policies:** POL-6 (architecture_is_subsystem_name_source_of_truth), POL-23 ADR-amendment sibling-sweep
**Routing:** architect

**Evidence:**
- ADR-027 frontmatter `subsystems_affected: [SS-16, SS-17]`
- ADR-027 §Consequences §Negative/Trade-offs bullet: "**`prism-query` is also touched in S-PLUGIN-PREREQ-E scope (TD-S-PLUGIN-PREREQ-A-003).**"
- ARCH-INDEX SS-07 row: "SS-07 | Adapter Pagination & Response Cache | query-engine.md | prism-query | Phase 1"

**Fix sketch:** Architect adjudicates intent. If ADR-027 CustomAdapter-only, prune §Consequences "prism-query is also touched" prose or move to footnote. If ADR-027 shares SS-07 scope, add SS-07 to `subsystems_affected:`.

---

### F-LP6-MED-003 — ADR-026 D2 "no default impl" stance vs D6 semver-breaking-change rule consistency

**Severity:** MEDIUM (intent verification)
**Type:** Semantic awkwardness across D2 + D6
**Anchor policies:** POL-22 Phase A (lexical-vs-semantic anchor-content verification)
**Routing:** architect

**Evidence:**
- ADR-026 D2 §Path B rationale: "Each impl MUST declare its static name explicitly" (no default for the new methods)
- ADR-026 D6: "Adding required methods to this trait is a semver-breaking change for plugin consumers. Future method additions must provide a default impl or be gated by a new ADR + semver bump."

**Note:** D2's "no default" stance applies because at PREREQ-E merge there are no external plugin consumers (PLUGIN-AUDIT-001). Once trait is public ABI, D6 forbids the same "no default" pattern.

**Fix sketch:** Architect adds clarifying paragraph to D2: "This 'no default' stance applies ONLY to the four built-in impls authored in this same PREREQ-E commit. Once the trait is public ABI (post-PREREQ-E merge), per D6, any future method addition that lacks a default IS a semver-breaking change requiring a new ADR."

---

### F-LP6-MED-004 — BC-2.16.011 EC-016-011-005 `deprecated_by: ADR-023` vs ADR-027 supersession question

**Severity:** MEDIUM (intent verification)
**Type:** Anchor adjudication
**Anchor policies:** POL-22 Phase C (named-entity-existence-verification)
**Routing:** architect

**Evidence:**
- BC-2.16.011 EC-016-011-005: "`BC-2.16.004-rust-escape-hatch.md` frontmatter update conflicts with deprecation metadata | `deprecated_by` remains `ADR-023`; add `removed: \"2026-05-15\"`, `removal_reason: \"PREREQ-E retirement per ADR-023 Rule 5\"`, change `lifecycle_status: deprecated → removed`"
- ADR-027 §Decision narrative: explicit CustomAdapter deletion mandate with timeline + perimeter

**Question:** Does ADR-027 supersede ADR-023's Rule 5 mandate (architect intent), or does ADR-023 remain the deprecator-of-record because ADR-023 introduced the deprecation?

**Fix sketch:** Architect adjudicates `deprecated_by` value. Two options: (a) keep `ADR-023` (original deprecator), (b) bump to `ADR-027` (operational deletion mandate). Update BC-2.16.011 EC-016-011-005 to match.

---

### F-LP6-LOW-001 — Story `inputs:` cites ADR-023 via `decisions/` subfolder

**Severity:** LOW
**Type:** Path-convention observation (no action required)
**Routing:** none

**Evidence:**
- Story `inputs:` array: `.factory/specs/architecture/decisions/ADR-023-plugin-only-sensor-architecture.md`
- Actual file location: same path resolves

This is a briefing-format vs filesystem-reality observation. Path resolves cleanly. Logged for OBS-LP6 cycle-close documentation only.

---

### F-LP6-LOW-002 — VP-156 §Property Statement cites "ADR-026 D7" without version pin (vs sibling pins)

**Severity:** LOW (intent verification)
**Type:** Internal version-pin consistency
**Anchor policies:** POL-23
**Routing:** architect

**Evidence:**
- VP-156 §Property Statement: cites "ADR-026 D7" version-agnostic
- VP-156 §Source Contract: cites "ADR-026 D7 v1.2" (stale per F-LP6-MED-001)
- BC-2.16.012 §Verification Properties: cites "ADR-026 D7 v1.7"

**Fix sketch:** Architect either pins all four VP-156 ADR-026 D7 cites to v1.7 (consistent with BC-2.16.012) or strips the v1.2 pin from §Source Contract (version-agnostic family). Bundle with F-LP6-MED-001 fix.

---

### OBS-LP6-001 — `[process-gap]` POL-7 D-571 surface enumeration omits "ADR runtime_deliverables list"

**Severity:** OBSERVATION (process-gap codification candidate)
**Disposition:** QUEUED-CYCLE-CLOSE

POL-7 D-571 verbatim-H1 surface enumeration lists 5 citation surfaces (BC table Title cell; §References section; Architecture Compliance Rules rows; prose exclusion-note paragraphs; §References completeness). None cover ADR `runtime_deliverables:` frontmatter prose claims (F-LP6-HIGH-003). Codification candidate: extend POL-22 Phase A "lexical-vs-semantic anchor-content verification" to ADR `runtime_deliverables:` — for each item in the array, the adversary verifies the item is not already true in live codebase pre-merge.

---

### OBS-LP6-002 — VP-156 status:draft already cited in BC-2.16.012 — symmetry note

**Severity:** OBSERVATION
**Disposition:** QUEUED-CYCLE-CLOSE

VP-156 is `status: draft`. BC-2.16.012 §Verification Properties cites VP-156 with full version pin. Cross-citation pattern correct. Symmetry holds for VP-156↔BC-2.16.012 but does NOT yet hold for VP-155↔BC-2.16.011 (F-LP6-HIGH-001).

---

### OBS-LP6-003 — `[process-gap]` Story `subsystems:` does not include SS-17 (WASM Plugin Runtime)

**Severity:** OBSERVATION (process-gap codification candidate)
**Disposition:** QUEUED-CYCLE-CLOSE

Story narrative explicitly says SensorAuth trait is opened for plugin implementation; SS-17 is plugin-runtime subsystem. ADR-026 `subsystems_affected:` includes SS-17. Story frontmatter `subsystems: [SS-01, SS-07, SS-16]` excludes SS-17. Intent-pending (story may scope to trait-surface side only). Codification candidate: when an ADR adds a subsystem to its `subsystems_affected:`, the anchor story's `subsystems:` must be evaluated in the same atomic burst (sibling-sweep extension).

---

## Trajectory Summary

| Pass | Findings | In-Scope | OBS Queued | Delta | Note |
|------|----------|----------|------------|-------|------|
| 1 | 14 | 12 | 2 | — | Initial: 1C+4H+5M+2L+2OBS |
| 2 | 9 | 8 | 1 | -5 | 3 FB1 regressions caught |
| 3 | 8 | 8 | 0 | -1 | 5 FB2 sibling-sweep regressions |
| 4 | 9 | 9 | 0 | +1 | FLAT — VP-156 anchor-back gaps (FB1 residue) |
| 5 | 10 | 7 | 3 | +1 | REGRESSION — FB4 bookkeeping + POL-7 surface 2 |
| 6 | 10 | 10 | 3 | 0 | FLAT count, NOVEL classes — intra-ADR contradiction + phantom runtime_deliverable + STORY-INDEX row staleness + VP source_bc/BC VP-anchor asymmetry |

Trajectory: **14→9→8→9→10→10**. Flat in count, but pass-6 surfaces NEW finding classes (intra-ADR contradiction; runtime_deliverable phantom; STORY-INDEX row staleness; VP↔BC asymmetric anchor — second instance after FB1 VP-154 fix).

---

## Artifact Versions After Pass-6 (Pre-Fix-Burst)

| Artifact | Pin |
|----------|-----|
| ADR-026 | v1.7 (will bump v1.8 in FB6 per F-LP6-CRIT-001 + F-LP6-HIGH-003) |
| ADR-027 | v1.3 (intent verification only; may stay v1.3 or bump v1.4 per F-LP6-MED-002 architect choice) |
| BC-2.16.011 | v1.2 (may bump v1.3 in FB6 per F-LP6-MED-004 outcome) |
| BC-2.16.012 | v1.6 |
| VP-155 | v0.3 (will bump v0.4 in FB6 per F-LP6-HIGH-001) |
| VP-156 | v0.4 (will bump v0.5 in FB6 per F-LP6-MED-001) |
| Story | v1.6 (will bump v1.7 in FB6 per F-LP6-CRIT-001 propagation) |
| STORY-INDEX | v2.109 (will bump v2.110 in FB6 per F-LP6-HIGH-002) |
| ARCH-INDEX | v2.48 (will bump v2.49 in FB6 per ADR-026 v1.8 sibling-sweep) |
| VP-INDEX | v1.41 |
| BC-INDEX | v4.82 |

## Next Step

Fix-burst-6 dispatch: architect (multi-finding) + state-manager (STORY-INDEX). All 10 in-scope findings (1 CRIT + 3 HIGH + 4 MED + 2 LOW) must close. 3 OBS queue for cycle-close codification. Then adversary pass-7 dispatch (fresh-context). BC-5.39.001 3-CLEAN protocol — streak resets 0/3.

Pass-6 report: `cycles/wave-4-operations/adversarial-reviews/S-PLUGIN-PREREQ-E-spec-pass-6.md` (this file).
