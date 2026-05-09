---
document_type: adr
adr_id: "ADR-021"
title: "BC/VP Promotion Lifecycle — Draft → Active → Verified Transitions, Audit Cadence, and BC-INDEX Count Authority"
status: ACCEPTED
date: "2026-05-08"
version: "1.1"
producer: architect
subsystems_affected: []
supersedes: null
superseded_by: null
inputs:
  - .factory/cycles/wave-4-operations/workspace-audit-2026-05-08.md
  - .factory/proposals/vsdd-prevention-layers-2026-05-08.md
  - .factory/policies.yaml
  - .factory/specs/architecture/ARCH-INDEX.md
  - .factory/specs/behavioral-contracts/BC-INDEX.md
  - .factory/specs/verification-properties/VP-INDEX.md
anchor_stories: []
references_phase3_siblings: [ADR-020]
locked_decisions: []
runtime_deliverables: []  # Methodology/process decision — defines BC/VP status lifecycle and BC-INDEX count authority; no production code units
wiring_deferred_to: null  # No runtime wiring required; transitions enforced by audit cadence protocol and consistency-validator
---

# ADR-021: BC/VP Promotion Lifecycle — Draft → Active → Verified Transitions, Audit Cadence, and BC-INDEX Count Authority

## Status

ACCEPTED 2026-05-08, v1.0. Effective immediately; retrospective promotion of BCs and VPs
is a downstream concern triggered by Bundle A.2 story reconciliation.

## Context

### Problem: 222/222 BCs and 143/145 VPs are `draft` — promotion criteria undefined

The 2026-05-08 workspace audit findings F-AUD-D5-01 and F-AUD-D7-01 document that:

- All 222 active behavioral contracts in BC-INDEX are `status: draft`.
- 143 of 145 verification properties in VP-INDEX are `status: draft`.
- No promotion criteria exist in any VSDD methodology document.
- No tooling or agent rule triggers a BC or VP status transition.

The consequence: BC-INDEX and VP-INDEX provide zero signal about implementation reality.
A consumer reading "BC-2.11.001: draft" learns nothing about whether QueryEngine::execute
is implemented, tested, or proven. The status field is informational noise.

This connects to finding F-AUD-D5-02 and the audit's root-cause taxonomy item 3: "No
graduation contract on BC/VP status — promotion criteria undefined." The same gap that allows
stub-merge stories to persist also allows BCs to stay `draft` indefinitely.

### Problem: F-AUD-D7-04 — harness-only VP proofs

Some VP proof files exist but contain `unimplemented!()` bodies (e.g., the `plugin_linker.rs`
VP-040 proof body). These are not `draft` (the file exists) but they are not `proven`
(the proof cannot pass). There is no intermediate status to distinguish "harness scaffolded"
from "proof passes."

### Problem: F-AUD-D8-09 — BC-INDEX 222 vs. module-decomposition 200 arithmetic drift

The STORY-INDEX overview line reads "200 Wave 1-2 BCs" while BC-INDEX records 222 active BCs.
The 22-BC difference is the Wave 3 multi-tenant BCs (BC-3.1.001–004, BC-3.2.001–005,
BC-3.3.001–004, BC-3.4.001–004, BC-3.5.001–002, BC-3.6.001–002, BC-3.7.001) added in the
Wave 3 multi-tenant story registration burst (STORY-INDEX v1.55). The module-decomposition.md
reference to "200 active BCs" was never updated when Wave 3 BCs were registered.

**Authoritative figure: BC-INDEX.** BC-INDEX is the primary BC catalog (append-only,
version-tracked, adversary-validated). Module-decomposition.md is a derived reference that
describes crate-to-BC mappings for implementers; its "200 BC" reference is stale prose.
The correct count is 222 active BCs. The doc fix targeting module-decomposition.md goes
through Bundle D (doc cleanup sweep); this ADR establishes the authority.

## Decision

### 1. BC lifecycle: `draft → active → verified → retired`

**`draft`** (default at authorship):
BC declared in BC-INDEX and its file; not yet implemented. No anchor story has reached
`status: merged` with this BC's postconditions reachable.

**`active`**:
At least one anchor story has reached `status: merged` (NOT `status: partial-merge`).
The BC's postconditions are reachable from production code in the anchor crate. An
"active" BC does not imply formal verification — only that production code satisfies it.

Promotion trigger: state-manager auto-promotes `draft → active` in the same burst that
flips an anchor story to `status: merged`, for every BC in that story's
`behavioral_contracts:` frontmatter array. See POL-14.

If the story flips to `status: partial-merge` (not `merged`), no BC promotion occurs.
Partial-merge stories have stub residue; their BCs may not be reachable from production.

**`verified`**:
The BC has at least one VP with `status: proven`. A VP is `proven` when:
- Kani: the `cargo kani` proof runs and passes within declared bounds.
- Fuzz: the fuzz target compiles, a non-trivial corpus is committed, and a CI smoke run completes.
- Proptest: the proptest suite passes deterministically in CI.

A BC may be `verified` even if not all its VPs are proven — one proven VP is sufficient
for the promotion. Subsequent VP proofs may further increase confidence but do not change
the BC status beyond `verified`.

Promotion trigger: when a VP transitions to `status: proven`, state-manager promotes all
BCs cited in that VP's `traces_to` field from `active → verified` in the same burst.

**`retired`**:
BC superseded by another BC (the replacement BC must exist and be non-draft before
retirement is permitted). The `replaced_by` field in BC frontmatter must be populated.
Retired BCs remain in BC-INDEX per POL-1 (append-only numbering); they are listed with
`status: retired` and the `replaced_by` reference.

Reverse transitions (active → draft, verified → active) are permitted only with an explicit
ADR justifying the demotion. Ad-hoc manual reclassification without an ADR is a protocol
violation.

### 2. VP lifecycle: `draft → harness-only → proven → retired`

**`draft`** (default at registration in VP-INDEX):
VP declared in VP-INDEX but no proof file exists at the declared path.

**`harness-only`**:
A proof file exists at the VP's declared path but the proof body is not implemented —
it contains `unimplemented!()`, `todo!()`, or an empty body that cannot yield a proof result.
This status closes the observability gap identified in F-AUD-D7-04: a file that exists but
cannot prove anything is neither `draft` nor `proven`; it needs its own signal.

Promotion trigger: when the proof file is created (committed to the repo), state-manager
promotes `draft → harness-only`.

**`proven`**:
The proof file exists AND the proof passes (as defined per-tool above in the BC lifecycle
section). For Kani proofs, "passes" means `cargo kani` exits 0 with no failing checks within
the declared bounds. For fuzz targets, "passes" means a corpus is committed and the fuzz target
compiles and runs one smoke iteration without panicking. For proptests, "passes" means the
proptest suite is deterministically green in CI.

CI must capture proof passage: a `just kani-local` run, a fuzz smoke invocation, or the
existing `just check` proptest gate. State-manager promotes `harness-only → proven` when a
CI artifact (log or explicit promotion commit) confirms passage.

**`retired`**:
VP superseded or anchor BC retired. The `replaced_by` field in VP frontmatter must reference
the successor VP or contain `"bc-retired"` if the underlying BC is retired with no successor.
Retired VPs remain in VP-INDEX per POL-1.

### 3. Promotion mechanics

The state-manager agent is the sole authority for executing BC and VP status promotions.
Promotions are atomic: the BC/VP file frontmatter update, the BC-INDEX or VP-INDEX row update,
and the STORY-INDEX story-status update (where applicable) all land in a single commit burst.

Promotion is triggered by event, not by schedule:
- BC `draft → active`: triggered by story `status: merged` flip.
- BC `active → verified`: triggered by VP `status: proven` promotion.
- VP `draft → harness-only`: triggered by proof file creation commit.
- VP `harness-only → proven`: triggered by CI proof passage artifact.

Demotion (any reverse transition) requires an explicit ADR. The ADR must be authored before
state-manager executes the demotion.

### 4. Audit cadence

A monthly automated audit (scheduled via the factory observability stack or a cron skill)
runs the following checks and reports findings to the human without auto-filing TDs:

1. Any BC `status: draft` where its anchor story has `status: merged` for more than 30 days.
   These are missed promotions — the state-manager promotion hook failed or was skipped.
2. Any VP `status: draft` where its anchor story has `status: merged` for more than 30 days.
3. Any VP `status: harness-only` for more than 30 days with no activity in the proof file
   git history.
4. Any BC with `status: active` but all its VPs are `status: draft` — the BC is reachable
   from production but has no proof coverage.

The audit is non-blocking: findings are reported and triaged by the human. Failing the audit
does not block story delivery; it informs the next cleanup cycle.

## Consequences

### Forward path for 222 draft BCs

Most of the 222 currently-draft BCs will auto-promote to `active` when the Bundle A.2 story
reconciliation reclassifies stories. Specifically:

- Stories reclassified from `merged` → `partial-merge` do NOT trigger BC promotion.
  Their BCs remain `draft` until the graduation contract closes and the story reaches `merged`.
- Stories confirmed as genuinely `merged` (after stub-residue verification) DO trigger
  retroactive BC promotion to `active` for all BCs in their `behavioral_contracts:` array.

The 8 stories in the Bundle A.2 reconciliation worklist (ADR-020 §Reconciliation Worklist)
include several that are confirmed fully-implemented (e.g., S-3.06 parser). Once those are
confirmed `merged`, their BCs auto-promote. Wave 1/Wave 2 stories already confirmed merged
(S-2.01 through S-2.08, S-6.07 through S-6.20 DTU clones, etc.) should trigger a bulk
retroactive BC promotion pass in Bundle A.2.

### Forward path for 143 draft VPs

143 of 145 VPs remaining at `draft` is a longer-term concern. Most VPs have no proof file
committed (they are in VP-INDEX but no corresponding `vp-NNN-*.rs` or equivalent exists).
The `harness-only → proven` promotion path for Kani proofs requires implementation effort
(VP-014 and VP-015 are the current proven examples). The monthly audit (Decision 4 above)
will track decay rate.

### BC-INDEX 222 vs. module-decomposition 200 (F-AUD-D8-09)

BC-INDEX is the source of truth. The correct active-BC count is 222. The stale "200 BCs"
reference in module-decomposition.md will be corrected in Bundle D (doc cleanup sweep).
This ADR documents the authority; no doc change is made here per bundle scope constraints.

### No schema change to BC-INDEX or VP-INDEX today

BC-INDEX v4.32 and VP-INDEX current version do not add a `status:` column today.
The status field is already present in individual BC and VP file frontmatter. Bundle A.2
will add a `status:` column to BC-INDEX and VP-INDEX summary tables to make the
aggregate promotion state visible at a glance.

## Alternatives Considered

### A: Single `status: active` flag (boolean promotion, no `verified` tier)

Rejected. Verification status is meaningfully distinct from implementation status. A BC that
is implemented (active) but not formally verified presents a different risk profile than one
that is implemented AND has a passing Kani proof. Conflating the two removes signal that the
formal-verifier and adversary both need.

### B: Percent-complete tracking per BC (n-of-M ACs with passing tests)

Rejected. Granular AC-level tracking adds authoring and tooling burden without proportional
benefit. The three-tier model (draft/active/verified) provides the signal stakeholders need
(is it real? is it proven?) with minimal machinery.

### C: On-demand promotion (human-initiated only, no automatic trigger)

Rejected. If promotion requires human action, the 222/222 draft situation will persist.
Auto-promotion on anchor-story merge is the only path to eventual convergence without
ongoing manual attention. The automation is safe because the trigger condition (story → merged)
already passes strict checks (POL-12 stub-residue gate).

### D: Count 200 as the canonical BC total and retire the 22 Wave 3 BCs as "proposed"

Rejected. BC-INDEX records 222 active BCs with full traceability. Downgrading 22 BCs to
"proposed" to match a stale prose reference in module-decomposition.md would corrupt the
BC catalog without technical justification. The doc fix is the correct path.
