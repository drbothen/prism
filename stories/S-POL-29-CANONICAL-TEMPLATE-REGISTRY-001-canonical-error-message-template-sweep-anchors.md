---
document_type: story
story_id: "S-POL-29-CANONICAL-TEMPLATE-REGISTRY-001"
title: "POL-29 Canonical Error-Message-Template Registry — Variant-Form Enumeration for Paraphrase Drift Detection"
wave: maintenance
epic_id: maintenance
priority: P2
status: planned
version: "0.2"
level: ops
producer: architect
timestamp: "2026-05-24"
created: "2026-05-24"
modified: "2026-05-24"  # v0.2: F-LP5-LOW-002 Suggestion field source-of-truth adjudication (Option B)
tdd_mode: strict
track: "Platform Engineering"
subsystems: []
crates_touched: []
target_module: ".factory/policies.yaml + .factory/specs/prd-supplements/error-taxonomy.md"
capabilities: []
behavioral_contracts: []
verification_properties: []
depends_on: []
blocks:
  - S-MAINT-POL29-HOOK-001  # The lint hook mechanization story must detect all variant
                             # forms defined by this registry, including paraphrase variants.
                             # Registry design produced here feeds S-MAINT-POL29-HOOK-001
                             # §Success Criteria (axis-class coverage).
points: 2
estimated_days: 0.5
risk: LOW
acceptance_criteria_count: 6
red_gate_tests: 0
estimated_passes: "2-3"
holdout_scenarios: []
assumption_validations: []
---

# S-POL-29-CANONICAL-TEMPLATE-REGISTRY-001: POL-29 Canonical Error-Message-Template Registry

## §Origin — [process-gap] with 4 cascade detections

This story resolves a [process-gap] class first detected in S-CONFIG-MULTI-TENANT-OVERRIDE-001 pass-4
(2026-05-24) and traceable to the same root axis across 3 prior cascades:

| Finding | Cascade | Root Cause |
|---------|---------|------------|
| F-LP2-MED-002 (S-CONFIG pass-2) | ci.yml EXPECTED=32→35 bump | Fix-burst-3 swept ci.yml + CLAUDE.md + scripts/check-non-exhaustive.sh but missed story body line 427+463 and PLUGIN-MIGRATION-001-E story body (4 sibling sites total) |
| F-LP3-MED-001 (S-CONFIG pass-3) | E-SPEC-023 trailing `Instance: '{instance_id}'` removal | Fix-burst-3 swept BC-2.06.016 v1.1 + code emission but missed error-taxonomy.md line 395 (description body vs message_template field — different occurrence forms in same file) |
| F-LP4-MED-001/002/003 (S-CONFIG pass-4) | BC-2.06.013/015 cite paraphrased templates for E-SPEC-021/022/023 | Fix-burst-4 swept canonical layer (taxonomy + BC-2.06.016) only. Missed BC-2.06.013 + BC-2.06.015 sibling BCs that paraphrase canonical templates |
| F-LP4-MED-004 (S-CONFIG pass-4) | S-CONFIG story body `EXPECTED=32` references | Fix-burst-3 missed sweep of story body (same citation-gap class as F-LP2-MED-002) |

**Root cause across all 4:** POL-29 step 3a sweep was scoped to the original TARGET STRING only.
Canonical error message templates exist in error-taxonomy.md. BC bodies and story task descriptions
paraphrase these templates with variant forms NOT caught by target-string grep alone:

- Separator drift: colon (`Error: `) vs em-dash (`Error — `) in BC prose
- Placeholder name drift: `{overlay_path}` vs canonical `{file}` in §Error Cases tables
- Capitalization drift: `Sensor` vs `sensor` in BC postcondition descriptions
- Omission drift: shortened form omits `overlay_fields` enumeration from canonical template

This is architecturally different from the existing POL-29 step 3a recidivist classes (version-pin
strings, catalog cites), which have stable, greppable canonical strings. Canonical error-message
templates require a VARIANT TAXONOMY — a classification schema for what constitutes a "paraphrase"
of each template. This design work is substantive enough to warrant its own story.

## §Relationship to S-MAINT-POL29-HOOK-001

S-MAINT-POL29-HOOK-001 mechanizes POL-29 step-8 as a deterministic lint hook. That story's
§Success Criteria requires the hook to detect 7 axis classes. This story defines an 8th axis class
(paraphrase-variant drift on canonical error-message templates) that feeds into S-MAINT-POL29-HOOK-001's
acceptance criteria. The sequencing is:

1. This story (S-POL-29-CANONICAL-TEMPLATE-REGISTRY-001) defines the registry schema + per-E-SPEC-NNN
   variant-form enumeration + sweep anchor set.
2. S-MAINT-POL29-HOOK-001 extends the lint hook to detect paraphrase drift using this registry.

This story does NOT implement tooling — it produces the policy amendment and registry definition
that enables tooling in S-MAINT-POL29-HOOK-001.

## §Adjudication Rationale (Option B — Defer to follow-up story)

The alternative options were evaluated against CLAUDE.md Canonical Principle:

**Option A (amend POL-29 in-scope) was rejected** because:
- POL-29 is already at v1.29 with 9 step-8 substeps and ~4,500 words. The session-review
  D-777 root-cause diagnosis is that the problem is NOT insufficient policy text but absent tooling.
  Adding a registry class to policy prose continues the over-codification anti-pattern.
- The registry design (variant taxonomy: separator drift / placeholder name drift / capitalization
  drift / omission drift) is a classification schema design problem, not a grep-pattern extension.
  Designing this in-scope of the S-CONFIG cascade would produce a half-designed policy entry.

**Option C (accept with rationale) was rejected** because:
- 4 cascade detections of the same [process-gap] class meets the "3+ recurrence" criterion in
  POL-29 step 3a itself for elevation to recidivist registry class.
- Accepting with a "human judgment" note violates CLAUDE.md Canonical Principle Rule 4.

**Option B was selected** per CLAUDE.md Rule 3:
- Concrete future dependency: S-MAINT-POL29-HOOK-001 lint hook REQUIRES the variant-form registry
  to implement axis-8 detection. This story feeds directly into S-MAINT-POL29-HOOK-001.
- Concrete story anchor: this file IS the anchor.
- Wave assignment: maintenance (same as S-MAINT-POL29-HOOK-001).
- The [process-gap] is bounded — the 4 originating findings (F-LP2-MED-002, F-LP3-MED-001,
  F-LP4-MED-001/002/003, F-LP4-MED-004) are all CLOSED. This story prevents recurrence, not
  retroactive repair.

## §Problem Statement

POL-29 step 3a defines a "variant-form registry" for recidivist value classes (3+ cascade recurrences).
Current registered classes:
- (a) error-taxonomy version pin — 3 variant forms (bare, with-md, backtick-quoted)
- (b) ADR-026 D7 pin — 4 variant forms (bare, embedded-section, parenthesized, prose-prefixed)
- (c) BC-2.16.002 catalog cite — 4 variant forms (canonical, no-parens, bare, close-paren-mid-row)

These are all VERSION-PIN or CITATION classes — stable, greppable canonical strings where the
variant is a formatting difference around an anchored version number.

The canonical-error-message-template class is structurally different:
- The "canonical string" is a full English sentence with interpolated placeholders
- Variants arise from SEMANTIC PARAPHRASE by BC authors and story writers, not formatting
- The paraphrase variants do NOT contain a version number or ID anchor that survives drift

Without sweep anchors (2-4 stable sub-phrases that survive paraphrase), step 3a greps cannot
catch these variants. The registry design must produce sweep anchors per E-SPEC-NNN.

## §Scope

This story is a SPEC-AND-POLICY story (Platform Engineering track). It produces:

1. **POL-29 step 3a canonical-error-message-template registry class** — formal definition of the
   class, variant taxonomy (separator drift / placeholder name drift / capitalization drift /
   omission drift), and registry entry schema.

2. **Per-E-SPEC-NNN sweep anchor sets** — for each E-SPEC-NNN currently in error-taxonomy.md,
   derive 2-4 sub-phrases that are stable across the known variant forms and suitable as grep
   anchors. Priority: E-SPEC-019 through E-SPEC-023 (the S-CONFIG error set where the
   [process-gap] was first detected).

3. **Retroactive application to existing BCs** — for each BC in behavioral-contracts/ that cites
   E-SPEC-NNN message templates, verify the cited form matches canonical template OR a registered
   variant-form. Flag and close any remaining drift.

4. **Lessons.md update** — entry 41 in wave-0-plugin-prereqs/lessons.md codifies the pattern
   (per pass-4 process-gap codification note in s-config-pass-4.md).

## §Acceptance Criteria

### AC-001 — POL-29 step 3a registry class definition

POL-29 step 3a in policies.yaml is amended (via proper PO dispatch) to include class (d):
`canonical-error-message-template sweep anchors`. The amendment includes:

- Variant taxonomy defining the 4 known paraphrase drift types (separator drift, placeholder name
  drift, capitalization drift, omission drift)
- Registry entry schema: `{ error_code: E-SPEC-NNN, canonical_template: "<verbatim string>",
  sweep_anchors: ["<stable sub-phrase-1>", "<stable sub-phrase-2>", ...] }`
- Trigger criterion: class (d) entry is created for EACH E-SPEC-NNN on its FIRST occurrence in
  a BC body or story task description (not deferred to 3rd recurrence, because the canonical
  template is explicitly defined and greppable from the moment it exists)
- Enforcement mechanism: adversary step 3a probe + (future) S-MAINT-POL29-HOOK-001 lint hook

**Red Gate Test:** Not applicable — this is a policy amendment. Adversarial pass-1 verifies
the amendment is internally consistent and covers the 4 variant drift types.

### AC-002 — Sweep anchor derivation for E-SPEC-019..E-SPEC-023

For each of E-SPEC-019, E-SPEC-020, E-SPEC-021, E-SPEC-022, E-SPEC-023 in error-taxonomy.md,
the POL-29 step 3a class (d) registry contains a populated entry with:

- Verbatim `canonical_template` matching the current error-taxonomy.md `message_template` field
- At least 2 sweep anchors that are (a) unique enough to not false-positive on unrelated text,
  (b) stable across all 4 known paraphrase drift types, (c) suitable for `rg` grep in .factory/

Example for E-SPEC-020 (illustrative — actual anchors derived from canonical template):
```yaml
- error_code: E-SPEC-020
  canonical_template: "Overlay file at '{file}' defines fields reserved for base sensor specs — overlay_fields: {overlay_fields}."
  sweep_anchors:
    - "Overlay file at '"
    - "fields reserved for base sensor"
    - "overlay_fields:"
```

### AC-003 — Retroactive BC sweep for E-SPEC-019..E-SPEC-023

For each BC in `.factory/specs/behavioral-contracts/` that contains any of E-SPEC-019, E-SPEC-020,
E-SPEC-021, E-SPEC-022, or E-SPEC-023:

- The cited error message form matches the canonical template OR is a documented variant-form
  that is registered in the class (d) sweep-anchor set
- Any remaining drift sites are closed (via PO dispatch) in the same burst as this story
- Post-fix: `rg` using each sweep anchor returns only canonical-form or registered-variant hits
  in `.factory/specs/behavioral-contracts/`

### AC-004 — lessons.md entry 41 codification

`.factory/cycles/wave-0-plugin-prereqs/lessons.md` entry 41 is authored with:

- [process-gap] label
- 4 originating findings (F-LP2-MED-002, F-LP3-MED-001, F-LP4-MED-001/002/003, F-LP4-MED-004)
- Root cause: POL-29 step 3a sweep scoped to target string only; no paraphrase variant registry
- Resolution: this story (S-POL-29-CANONICAL-TEMPLATE-REGISTRY-001) + downstream S-MAINT-POL29-HOOK-001
- Date: 2026-05-24

### AC-006 — Suggestion field source-of-truth adjudication documented in POL-29 step 3a class (d)

The POL-29 step 3a class (d) registry entry schema includes a `suggestion_authority` field
that explicitly records: BC-2.06.016 `Suggestion` rows are canonical for operator-facing
remediation guidance; taxonomy description-prose sub-clauses are informative-only and do not
constitute a competing `Suggestion` authority. This prevents future adversary passes from
re-raising this as a [pending intent verification] finding.

**Verification:** adversary pass-1 reads POL-29 class (d) definition and confirms
`suggestion_authority` field exists with the correct attribution. BC-2.06.016 Suggestion rows
are used as the authoritative form in all AC-003 retroactive sweep comparisons (not taxonomy
description-prose).

**Originating finding:** F-LP5-LOW-002 (S-CONFIG pass-5; architect adjudication 2026-05-24 Option B).

### AC-005 — S-MAINT-POL29-HOOK-001 blocking dependency registered

`S-MAINT-POL29-HOOK-001` frontmatter `depends_on:` list is updated to include
`S-POL-29-CANONICAL-TEMPLATE-REGISTRY-001`, with the justification:
"The lint hook's paraphrase-drift detection (axis class 8) requires the variant-form registry
defined here as its grep-anchor input."

STORY-INDEX.md is updated with the new `depends_on` relationship.

## §File List

| File | Action | Agent | Notes |
|------|--------|-------|-------|
| `.factory/policies.yaml` | Modify (POL-29 step 3a) | product-owner | Add class (d) canonical-error-message-template registry. Bump policies.yaml version. |
| `.factory/specs/prd-supplements/error-taxonomy.md` | Read-only audit | product-owner | Source of canonical templates for E-SPEC-019..E-SPEC-023; derive sweep anchors |
| `.factory/specs/behavioral-contracts/BC-2.06.013.md` | Verify / modify if drift | product-owner | Already swept in fix-burst-5 v1.1; verify with new sweep anchors |
| `.factory/specs/behavioral-contracts/BC-2.06.015.md` | Verify / modify if drift | product-owner | Already swept in fix-burst-5 v1.1; verify with new sweep anchors |
| `.factory/specs/behavioral-contracts/` (all) | Sweep with new anchors | product-owner | Retroactive sweep for E-SPEC-019..E-SPEC-023 paraphrase drift |
| `.factory/cycles/wave-0-plugin-prereqs/lessons.md` | Add entry 41 | state-manager | [process-gap] codification (per s-config-pass-4.md process-gap note) |
| `.factory/stories/S-MAINT-POL29-HOOK-001-validate-cite-pin-completeness-lint-hook.md` | Modify frontmatter | story-writer | Add `depends_on: [S-POL-29-CANONICAL-TEMPLATE-REGISTRY-001]` |
| `.factory/stories/STORY-INDEX.md` | Add row + update dependency | story-writer | Register S-POL-29-CANONICAL-TEMPLATE-REGISTRY-001; update S-MAINT-POL29-HOOK-001 depends_on |

## §Token Budget Estimate

| File | Lines | Budget |
|------|-------|--------|
| policies.yaml POL-29 step 3a amendment | ~40 lines | 400 tokens |
| error-taxonomy.md audit (E-SPEC-019..023) | read-only | 200 tokens |
| BC sweep (5 BCs, verify-only or minor fixes) | ~10 lines/BC | 250 tokens |
| lessons.md entry 41 | ~20 lines | 150 tokens |
| STORY-INDEX.md row + depends_on | ~5 lines | 60 tokens |
| S-MAINT-POL29-HOOK-001 frontmatter update | ~2 lines | 30 tokens |
| **Total** | | **~1,090 tokens** |

## §Previous Story Intelligence

- S-MAINT-POL29-HOOK-001 (2026-05-22): mechanization story for POL-29 step-8 cite-pin sweep.
  Session-review D-777 identified POL-29 over-codification asymptote. This story feeds into
  S-MAINT-POL29-HOOK-001's axis-8 detection requirement.

## §Process Note — Agent Dispatch Sequence

This story is a spec/policy amendment. Recommended dispatch sequence:

1. **Product-owner** — read error-taxonomy.md E-SPEC-019..023 canonical templates, derive sweep
   anchors, author POL-29 step 3a class (d) amendment, run retroactive BC sweep
2. **State-manager** — commit policies.yaml + BC updates + lessons.md entry 41 as single burst
3. **Story-writer** — update S-MAINT-POL29-HOOK-001 depends_on + STORY-INDEX.md
4. **Adversary (pass-1)** — verify registry internal consistency + BC sweep completeness
5. **State-manager** — close + archive

No TDD cycle needed (no code changes). Adversary pass-1 is the convergence gate.

## §Source of Truth Precedence

- `error-taxonomy.md` E-SPEC-NNN `message_template` fields are canonical. BC bodies and story
  task descriptions that cite these templates are DOWNSTREAM. If conflict: taxonomy wins.
- `policies.yaml` POL-29 step 3a class (d) registry is the sweep anchor source of truth.
  S-MAINT-POL29-HOOK-001 hook implementation consumes this registry.
- **Suggestion field authority: BC-2.06.016 is canonical.** The taxonomy description column
  embeds suggestion guidance as prose sub-clauses inside a free-form description field; it
  does not have a dedicated Suggestion column. BC-2.06.016 has a first-class, structured
  `Suggestion` row per error code containing fuller, operator-facing remediation detail. These
  are not competing representations of the same field — they are different fields in different
  schemas. CLAUDE.md Rule #3 (PRD supplements supersede PRD prose "for the same surface area")
  applies to message_template, severity, category, exit code, and retryable flag — fields the
  taxonomy explicitly columns out. It does NOT extend to Suggestion text, which has no
  counterpart column in the taxonomy. AC-003 retroactive BC sweep should verify that
  BC-2.06.013 and BC-2.06.015 Suggestion citations are consistent with BC-2.06.016 (not with
  taxonomy description-prose). Adjudicated from F-LP5-LOW-002 (2026-05-24).

## §Originating Findings Cross-Reference

| Finding ID | Pass | Cascade | Closed by | Pattern |
|------------|------|---------|-----------|---------|
| F-LP2-MED-002 | S-CONFIG pass-2 | S-CONFIG-MULTI-TENANT-OVERRIDE-001 | fix-burst-3 story-writer | EXPECTED=32 citation in story body not swept |
| F-LP3-MED-001 | S-CONFIG pass-3 | S-CONFIG-MULTI-TENANT-OVERRIDE-001 | fix-burst-4 PO bd9ef119 | E-SPEC-023 description body (line 395) not swept by message_template-scoped grep |
| F-LP4-MED-001 | S-CONFIG pass-4 | S-CONFIG-MULTI-TENANT-OVERRIDE-001 | fix-burst-5 PO 6585f846 | BC-2.06.013 §Postconditions separator drift (colon vs em-dash) for E-SPEC-020 |
| F-LP4-MED-002 | S-CONFIG pass-4 | S-CONFIG-MULTI-TENANT-OVERRIDE-001 | fix-burst-5 PO 6585f846 | BC-2.06.013 §Error Cases placeholder name drift (`{overlay_path}` vs `{file}`) |
| F-LP4-MED-003 | S-CONFIG pass-4 | S-CONFIG-MULTI-TENANT-OVERRIDE-001 | fix-burst-5 PO 6585f846 | BC-2.06.015 E-SPEC-022 omitted `sensor_id` field + capitalization drift |
| F-LP4-MED-004 | S-CONFIG pass-4 | S-CONFIG-MULTI-TENANT-OVERRIDE-001 | fix-burst-5 story-writer 872f5a63 | S-CONFIG story body E-SPEC-020 shortened form (omission drift) |
| F-LP5-LOW-002 | S-CONFIG pass-5 | S-CONFIG-MULTI-TENANT-OVERRIDE-001 | architect adjudication 2026-05-24 (Option B) | BC-2.06.016 Suggestion rows vs taxonomy description-prose suggestion clauses — source-of-truth adjudicated: BC-2.06.016 canonical for Suggestion field; taxonomy description-prose does not constitute a competing Suggestion authority |
