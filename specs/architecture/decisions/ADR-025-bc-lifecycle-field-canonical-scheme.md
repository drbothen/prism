---
document_type: adr
adr_id: "ADR-025"
title: "BC Lifecycle Field Canonical Scheme — Single status Field Governs; lifecycle Field Retired"
status: ACCEPTED
date: "2026-05-12"
version: "1.0"
producer: architect
subsystems_affected: []
supersedes: null
superseded_by: null
amends: ADR-021
anchor_stories: []
runtime_deliverables: []
wiring_deferred_to: null
---

# ADR-025: BC Lifecycle Field Canonical Scheme — Single status Field Governs; lifecycle Field Retired

## Context

ADR-021 (BC/VP Promotion Lifecycle, 2026-05-08) established a four-tier BC status
lifecycle: `draft → active → verified → retired`. That ADR defines `status` as the
field governing BC lifecycle state and specifies the promotion triggers (story merge,
VP proof passage, etc.).

During the STEP 2 maintenance sequence (D-321, D-449), the consistency-validator surfaced
a divergence in BC frontmatter: a cohort of newer BCs authored after ADR-021 carries two
lifecycle-related fields:

```yaml
status: draft
lifecycle: active
```

Spot-check of the BC catalog (2026-05-12) shows four BCs with this pattern:

| BC | status | lifecycle |
|----|--------|-----------|
| BC-2.05.012 | draft | active |
| BC-2.06.011 | draft | active |
| BC-2.21.001 | draft | active |
| BC-2.22.001 | accepted | active |

The `lifecycle: active` field is not defined in ADR-021, not enforced by any tooling or
policy, and not present in the BC-INDEX schema. Its presence creates ambiguity: does it
override `status: draft`? Is it a staging signal for the next promotion event? Is it
stale data from an earlier authoring convention that predates ADR-021?

A separate but related field, `amendment_lifecycle: pending`, appears in a different cohort
of BCs (e.g., BC-2.01.005, BC-2.02.003) and tracks pending ADR-023 amendments. This field
has distinct semantics (it refers to the amendment state of the BC's body text, not to the
BC's promotion state) and is NOT affected by this ADR.

## Decision

**The `status` field defined in ADR-021 is the sole canonical field governing BC promotion
lifecycle. The `lifecycle` field is retired as a BC frontmatter key.**

The four-tier ADR-021 lifecycle (`draft → active → verified → retired`) is fully expressed
through `status`. No second field is needed. The `lifecycle: active` entries in the four
affected BCs represent unintended authoring drift — they were written to signal "this BC is
conceptually active even though its anchor story has not merged," but that intent is already
correctly captured by the ADR-021 `status: draft` value (draft means implemented-but-not-merged
or not-yet-implemented; only a story merge transitions it to `active`).

Specifically:

1. **`lifecycle:` is retired as a BC frontmatter key.** State-manager will remove it from the
   four affected BCs in a follow-up sweep commit. No new BCs should include `lifecycle:`.

2. **`status:` is the sole governance field.** Its permitted values are: `draft`, `active`,
   `verified`, `retired` (per ADR-021 §1). Any BC with `status: accepted` is a protocol
   violation — the permitted values do not include `accepted`. BC-2.22.001 must be corrected
   to `status: draft` (since its anchor story S-WAVE5-PREP-01 has not reached `status: merged`).

3. **`amendment_lifecycle:` is preserved.** It tracks amendment state of the BC body text for
   BCs with pending ADR-023 amendments, not BC promotion state. Its semantics are orthogonal to
   this decision; it is governed by the ADR-023 amendment schedule.

4. **Sweep scope.** State-manager sweep must address:
   - Remove `lifecycle: active` from BC-2.05.012, BC-2.06.011, BC-2.21.001, BC-2.22.001
   - Correct BC-2.22.001 `status: accepted` → `status: draft` (invalid status value)
   - Verify no other BCs carry `lifecycle:` (full grep of `.factory/specs/behavioral-contracts/`)

## Rationale

**One field, one meaning.** Two fields governing the same concept (`status` and `lifecycle`)
with overlapping semantics is a consistency violation. When they disagree (as in `status: draft,
lifecycle: active`), any consumer — human or agent — must guess which field takes precedence.
The ADR-021 design is explicit: `status` is the authoritative field. The `lifecycle` additions
are unauthorized drift.

**ADR-021 already covers the intent.** The reason `lifecycle: active` was added to these four
BCs was to signal that the BC is "semantically live" even before its anchor story merges. ADR-021
already accommodates this: `status: draft` does not mean "ignored" — it means "authored but
anchor story not yet merged." The signal is already present; a second field to say "but we really
mean it's active" is redundant noise.

**`status: accepted` is not a valid ADR-021 value.** The four permitted values in ADR-021 §1
are `draft`, `active`, `verified`, `retired`. BC-2.22.001 carries `status: accepted`, which
was inherited from ADR authoring conventions (where `accepted` is a valid ADR status). For BCs,
`accepted` is not valid. The correct pre-merge status is `draft`.

**Consistency-validator and tooling can enforce one field.** With `lifecycle:` retired, POL-7
and the monthly audit cadence (ADR-021 §4) have a single field to validate. Adding `lifecycle:`
to BC authoring templates would require tooling updates across the board; retiring it requires
only a targeted sweep of four files.

## Consequences

### Positive

- BC frontmatter has one unambiguous lifecycle field: `status`
- BC-INDEX, tooling, and agents have a single field to read for promotion state
- ADR-021's promotion mechanics (`draft → active` on story merge) are not changed
- Authoring guidance is simplified: new BCs use `status: draft` only

### Negative / Trade-offs

- Four BC files require a frontmatter edit (one-line removal each) in a state-manager sweep
- BC-2.22.001 requires a `status: accepted` → `status: draft` correction

### Status as of 2026-05-12

ACCEPTED. ADR-021 amended by this decision. The state-manager sweep to remove `lifecycle:`
from the four affected BCs is a follow-up burst; it does not block story delivery.

## Alternatives Considered

- **Option A: Formalize both fields — status for ADR-021 lifecycle, lifecycle for "conceptual readiness."** Rejected. Formalizing a second field with fuzzy "conceptual readiness" semantics gives agents an escape hatch to leave BCs at `status: draft` while marking them `lifecycle: active`, defeating the ADR-021 promotion discipline. One field with explicit promotion triggers is sufficient.

- **Option B: Retire status, keep lifecycle as the canonical field.** Rejected. `status` is the ADR-021 canonical field, enforced by POL-14, referenced in BC-INDEX, and present in all 235 active BCs. Renaming the canonical field would require a full-catalog migration with no semantic benefit.

- **Option C: No change — document that lifecycle supersedes status when present.** Rejected. This legitimizes the drift and creates a two-tier schema where newer BCs follow different rules than older BCs. Consistency requires one scheme across the catalog.

## Source / Origin

- ADR-021 §1 (`draft → active → verified → retired` lifecycle definition)
- BC-INDEX.md v4.53 (`total_contracts: 235`, no `lifecycle:` column)
- D-321 deferred item #84 (BC frontmatter status/lifecycle divergence cleanup)
- 2026-05-12 spot-check: 4 BCs with `lifecycle: active` identified (BC-2.05.012, BC-2.06.011, BC-2.21.001, BC-2.22.001)

## Changelog

| Version | Date | Author | Change |
|---------|------|--------|--------|
| 1.0 | 2026-05-12 | architect | Initial decision — lifecycle field retired; status is sole canonical field; BC-2.22.001 status:accepted correction noted |
