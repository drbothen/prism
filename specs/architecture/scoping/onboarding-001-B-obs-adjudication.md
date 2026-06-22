---
document_type: adjudication-note
scope: LOCAL adversarial cascade — S-DEMO-PRISMQL-ONBOARDING-001-B
findings_adjudicated: [OBS-2, OBS-3]
status: final
producer: product-owner
date: 2026-06-22
---

# Adjudication Note — S-DEMO-PRISMQL-ONBOARDING-001-B OBS-2 and OBS-3

## Summary

Two LOW observations from the LOCAL adversarial cascade of story S-DEMO-PRISMQL-ONBOARDING-001-B
required product-owner adjudication because they touch spec artifacts (BCs and error taxonomy).
Both are resolved here. Neither expands the story's in-flight scope.

---

## OBS-2 — `explain_query` Parity (Scope Adjudication)

### Decision: OUT-OF-SCOPE for S-DEMO-PRISMQL-ONBOARDING-001-B

### Justification

The question has two sub-cases with different answers:

**Sub-case A — E-QUERY-038 (BC-2.11.016) on explain_query**

BC-2.11.016 §Postconditions do NOT mandate explain_query parity. The only reference to
explain_query in BC-2.11.016 is in §Related-BCs: "BC-2.11.010 — depends on: explain_query
performs the same plan-time validation; E-QUERY-038 fires on explain_query as well as query."
That sentence is a non-binding cross-reference annotation in the §Related-BCs section, not a
postcondition. It expresses intent — NOT a contractually binding requirement for this story.
BC-2.11.010 v1.6 §Error Cases lists only E-QUERY-001, E-ALIAS-001, and E-QUERY-003 — no
E-QUERY-037 or E-QUERY-038. The story's 6 ACs cover the `query`/`execute` MCP tool only.

Wiring explain_query to fire the E-QUERY-038 column gate is genuinely valuable (an LLM
pre-flighting with explain_query should get the same pedagogical feedback as query), but it
requires:
- Adding the E-QUERY-038 + E-QUERY-037 error path to BC-2.11.010 §Error Cases
- Adding a new AC to a follow-up story (or updating BC-2.11.010 and creating a dedicated story)
- Wiring `check_query_column_availability` and `check_operator_type_compatibility` from the
  `explain()` path in `prism-query/src/explain.rs`

This is a concrete future implementation task requiring a spec amendment that is outside the
currently-approved scope of S-DEMO-PRISMQL-ONBOARDING-001-B. It is NOT a convenience defer —
the BC-2.11.010 §Error Cases table is the authoritative spec for what explain_query returns
and it was never amended to include these gates.

**Sub-case B — E-QUERY-001/002/003/037 enrichments (BC-2.11.017) on explain_query**

BC-2.11.017 §Preconditions state: "A PQL query has been submitted to the `query` or
`explain_query` tool." This IS binding language — it places the enrichments on both tools.
BC-2.11.017 §Invariants confirms: "DI-004: All four error codes are emitted during `query`
or `explain_query` tool calls." This invariant is binding.

However, the story's ACs (AC-003/AC-004) only test the `query` path. The explain_query
application of BC-2.11.017 enrichments is a real spec gap in BC-2.11.010 (which does not
list E-QUERY-001/002/003 enrichment fields in its Error Cases table as binding for explain).
This is identical in structure to Sub-case A: BC-2.11.010 needs to be amended and a
follow-up story needs to wire the enrichments into explain.rs.

**Surface boundary rationale (why deferral is correct, not convenience)**

The surface boundary between `query`/`execute` (SS-11) and `explain_query` (also SS-11 but
a distinct code path in `prism-query/src/explain.rs`) is architecturally real. The plan-time
gates fire in `engine.rs`; explain_query in `explain.rs` does NOT go through the same
`check_availability_gate` code path. Wiring the gates into explain.rs is a distinct change
with distinct test coverage requirements. Folding it in-flight would expand story scope
without a spec amendment and without new ACs — exactly the "silent scope expansion without
spec backing" anti-pattern. The CORRECT path is: amend BC-2.11.010, create a follow-up
story, and deliver it properly.

**There is no false dependency blocking the deferral.** The explain.rs wiring can be done
independently of 001-B's merge. The demo's pedagogical loop uses `query`, not `explain_query`,
as the failing path. Explain_query parity is valuable but not demo-blocking.

### Required BC Amendment (in-scope now — product-owner owns this)

BC-2.11.016 §Related-BCs contains the soft note: "explain_query performs the same plan-time
validation; E-QUERY-038 fires on explain_query as well as query." This is aspirational, not
currently contractually binding in §Postconditions. To prevent the adversary from re-flagging
this in future passes, the note should be tightened to be explicit about what is binding vs.
aspirational.

BC-2.11.016 §Related-BCs note is amended (see edit below) to replace the ambiguous statement
with: "explain_query parity for E-QUERY-038 is tracked via follow-up story STORY-EXPLAIN-PARITY-001
(anchor: BC-2.11.010 §Error Cases; BC-2.11.016 §explain_query_parity)."

### Follow-up Story Anchor

**Follow-up story:** `S-EXPLAIN-PARITY-001` (to be registered by orchestrator)
- Anchor: BC-2.11.010 §Error Cases (amend to add E-QUERY-037, E-QUERY-038, E-QUERY-002,
  E-QUERY-003 enrichment fields as mandatory error outputs for explain_query)
- Scope: Wire `check_query_column_availability` and `check_operator_type_compatibility`
  helpers from `explain.rs`; add enrichment fields to explain_query structured error responses
- BC anchor: BC-2.11.010 v1.7 (amendment), BC-2.11.016 §explain_query_parity,
  BC-2.11.017 §explain_query_parity
- Demo priority: P2 (valuable for production; NOT demo-blocking)
- DO-NOT-REFLAG for S-DEMO-PRISMQL-ONBOARDING-001-B cascade: YES

### Orchestrator routing required

- state-manager: record S-EXPLAIN-PARITY-001 as a pending story stub with the above anchor
- BC-2.11.016: amend §Related-BCs note (product-owner edits, in this burst)
- story-writer: create S-EXPLAIN-PARITY-001 when wave slot opens (after 001-B merges)

---

## OBS-3 — E-QUERY-002 Taxonomy Message Divergence

### Decision: (b) Update error-taxonomy.md E-QUERY-002 row

### Justification

The current taxonomy E-QUERY-002 Message Format is:
`"Type error: field '{field}' is {actual_type}, cannot use {operator}"`

The implementation (worktree S-DEMO-PRISMQL-ONBOARDING-001-B) has two live Display variants
that both emit E-QUERY-002:

1. **`PrismError::QueryPlanFailed`** (pre-existing, develop branch): `"E-QUERY-002: query planning failed: {detail}"`
2. **`PrismError::QueryTypeMismatch`** (new, S-DEMO-PRISMQL-ONBOARDING-001-B): `"E-QUERY-002: type mismatch — column '{column}' in table '{table}' has type '{actual_type:?}' which does not support operator '{operator}'"`

The taxonomy template (`"Type error: field '{field}' is {actual_type}..."`) matches NEITHER
live variant. Option (a) — aligning the `QueryTypeMismatch` Display to the taxonomy template
— would be wrong because:

1. The taxonomy template predates the `QueryTypeMismatch` variant. The new variant's Display
   is richer (includes `column`, `table`, `actual_type`, `operator`) and is more useful for LLM
   self-correction (it provides the full context the `valid_operators_for_type` enrichment
   complements).
2. The `valid_operators_for_type` field is the machine-readable contract (BC-2.11.017 §E-QUERY-002
   postconditions); the Display is human/LLM-facing prose. Choosing the Display that preserves
   the structured field names (`column`, `table`, `actual_type`, `operator`) is correct.
3. ADR-035 canonical-row convention says "Message Format is the verbatim shipped Display." The
   shipped Display is `"E-QUERY-002: type mismatch — column '{column}'..."` — so the taxonomy
   must match the code, not the other way around, when the code is the implementation-of-record
   and the taxonomy row is stale.
4. `QueryPlanFailed` also diverges and represents a genuine two-display-format collision on
   E-QUERY-002 — analogous to the E-AUTH-001/002 collision documented in the taxonomy. The
   correct response is to document the collision, not paper over it.

The taxonomy update (option b) is production-grade: it documents the actual state of the code,
notes the `QueryPlanFailed` divergence, preserves the structured-field contract in the row,
and bumps the version per POL-26/POL-32.

**Exact new Message Format for E-QUERY-002:**
```
COLLIDED — two Display formats: (a) PrismError::QueryPlanFailed: "E-QUERY-002: query planning failed: {detail}"; (b) PrismError::QueryTypeMismatch (ADR-041 L4): "E-QUERY-002: type mismatch — column '{column}' in table '{table}' has type '{actual_type:?}' which does not support operator '{operator}'"
```

The structured-error response for `QueryTypeMismatch` adds `valid_operators_for_type`
per BC-2.11.017 §E-QUERY-002 postconditions — the machine-readable contract is unambiguous
regardless of the Display string.

### Implementer constraint confirmed (no Display change needed)

Because we chose option (b), the implementer does NOT need to change `QueryTypeMismatch`'s
`#[error(...)]` attribute. The Display string is production-grade as implemented. The only
change is to the taxonomy row (product-owner edit, in this burst).

### error-taxonomy.md edits (product-owner executes, in this burst)

1. Update E-QUERY-002 row Message Format and Description to document both Display formats
   and the collision.
2. Add a note about `QueryPlanFailed` divergence consistent with the E-AUTH-001/002 collision
   documentation style (two-format collision, blast-radius renumbering would be needed to
   resolve, defer to maintenance story).
3. Bump taxonomy version v1.91 → v1.92.
4. Add changelog row.

---

## Artifacts Edited in This Burst

| Artifact | Change | Owned by |
|----------|--------|----------|
| `.factory/specs/architecture/scoping/onboarding-001-B-obs-adjudication.md` | Created (this document) | product-owner |
| `.factory/specs/prd-supplements/error-taxonomy.md` | E-QUERY-002 row updated; version bumped v1.91→v1.92; changelog row added | product-owner |
| `.factory/specs/behavioral-contracts/BC-2.11.016-e-query-038-column-not-found.md` | §Related-BCs note tightened (explain_query parity deferral explicit) | product-owner |

## Downstream Routing Required (Orchestrator)

| Routing target | Action | Priority |
|----------------|--------|----------|
| state-manager | Record S-EXPLAIN-PARITY-001 as pending story stub; attach to BC-2.11.010 + BC-2.11.016 + BC-2.11.017; mark as P2, NOT demo-blocking | After 001-B merges |
| story-writer | Create S-EXPLAIN-PARITY-001 story spec when wave slot opens | After 001-B merges |
| implementer | No Display change required for `QueryTypeMismatch` (OBS-3 decision b) | — |
| adversary | OBS-2 and OBS-3 are CLOSED. S-EXPLAIN-PARITY-001 is the explicit follow-up anchor — DO-NOT-REFLAG OBS-2 for this cascade | Immediately |

## DO-NOT-REFLAG (this cascade)

- OBS-2 (`explain_query` E-QUERY-038 parity): closed out-of-scope; follow-up story
  S-EXPLAIN-PARITY-001 registered; BC-2.11.016 §Related-BCs clarified.
- OBS-3 (E-QUERY-002 Display divergence): closed by taxonomy update v1.92; implementer
  Display unchanged.
