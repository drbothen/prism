> **CLIP-mounted copy.** This is the agnostic persona-storyboard runbook mounted into the CLIP multi-repo for execution here. Source of truth authored/enhanced in rivetry (`.factory/storyboard/PERSONA-STORYBOARD-PROCESS.md` @ 38a5c10); pending the deferred `vsdd-factory:persona-storyboard` engine-port that will make this a shared skill (no drift-reconcile until then). **Execution target = CLIP.** Storyboard artifacts live under `.factory-project/storyboard/<version>/`; the runbook's Rivetry EXAMPLE blocks remain illustration only — CLIP's own examples are its target-state domain-spec actors/roles, CAP-001..029, and the ADR-TS set.

# Persona-Storyboard Process — Repeatable Runbook

> **Type:** process/methodology runbook (not a spec, not an analysis artifact). **Status:** canonized,
> v1.0. **Scope:** project-agnostic PROCESS. Every concrete path, ID prefix, lifecycle-stage name, and
> content example drawn from Rivetry is explicitly marked `> **EXAMPLE (Rivetry):**` — a fresh reader
> (human or agent) applying this runbook to a different product should treat those blocks as
> illustration only, never as required structure.
>
> **Engine-port note.** This runbook is written **engine-port-ready**: it is intended to be lifted
> into a `vsdd-factory:persona-storyboard` skill so any VSDD project can run this method without
> hand-authoring it first. That lift is a **deferred follow-up**, not done by this document. One
> housekeeping note for whoever performs the lift: the task that requested this document referred to
> the lift as "tracked P-014" — but `P-014` is *already* Rivetry's own tracking ID for the
> responsive-evidence gap that **Stage 7 of this very document closes** (see
> `.factory/analysis/storyboard-process-research.md`'s Executive Summary). Rather than silently picking
> one meaning, both uses are named here explicitly: **P-014 = the evidence-discipline gap, now closed
> by Stage 7.** The engine-port itself has no ID yet; assign one (a fresh `P-NNN`/`D-NNN` entry) when
> the lift is actually scheduled, per Rivetry's own governance-owns-the-ID-space rule (`state-manager`).
>
> **Frontmatter convention check (this pass).** This corpus's own reference artifacts —
> `WORKFLOW-INVENTORY.md`, every `journeys/journey-*.md` file, and every `frames/*/README.md` — use a
> **blockquote status header**, not YAML frontmatter with an `input-hash` field. Only per-frame
> *narrative* files (e.g. `frame-01b-narrative.md`) carry a small YAML frontmatter block, and even
> those do not carry `input-hash`. No file in `.factory/storyboard/` carries an `input-hash` field
> today, so none is added here — this document follows the corpus's own dominant convention (a clean
> blockquote header) rather than inventing a field the corpus doesn't use.
>
> **Grounded in (read for this pass).** `.factory/analysis/storyboard-process-research.md` (external
> research + the 7 concrete enhancements + the P-014 evidence-discipline answer — all folded in below,
> cited inline at point of use); the existing Rivetry corpus as worked EXAMPLES:
> `.factory/storyboard/v0.1.0-greenfield/WORKFLOW-INVENTORY.md`,
> `journeys/journey-mechanical-designer.md`,
> `frames/frame-01b-modernized-net-classification/` (+ its `UX-MODERNIZATION-PRINCIPLES.md`),
> `frames/frame-12-direct-modeling-workbench/`, `design-language/direction-0{1..4}-*/`, and
> `.factory/ui-evidence/phase-b-responsive/` (the existing, pre-enhancement evidence set). **Additive
> only** — no existing corpus artifact was modified to produce this document; no commit was made
> (state-manager owns the closing burst).
>
> **Enhancements-v2 burst (2026-07-07, ux-designer, this pass).** A second, human-approved research round
> folds NET-NEW, additive-only content into this canonized v1.0 runbook, grounded in three fresh research
> memos: `.factory/analysis/storyboard-research-v2-persona-workflow.md` (Angle A — persona-enumeration
> completeness; Angle B — workflow-enumeration completeness, cross-persona discovery, coverage &
> prioritization), `.factory/analysis/storyboard-research-v2-hifi-agentic.md` (Angle C — hi-fi production
> discipline: design tokens, shared components, content, a11y annotation, AI-mock guardrails; Angle D —
> agentic/AI-UX design patterns), and `.factory/analysis/storyboard-research-v2-validation-structure.md`
> (Angle E — AI-runnable design-validation gates; Angle F — an agnostic, persona-traceable directory
> standard). Every insertion below is marked inline with this same dated, attributed note at its precise
> point of use, cites the source memo + its named authorities, and preserves every prior claim — nothing
> existing was deleted, rewritten, or renumbered.

---

## How to Run This End-to-End (Quick-Start)

Run the stages in this order for a **first pass** on a new product or a new major UX surface. Stages 1
and 2 are usually run once per project (or once per major scope-addition burst) and then extended
append-only; Stages 3–8 **iterate** — you run them again per persona, per workflow, or per frame as the
corpus grows.

| # | Stage | One-line purpose | Repeats? |
|---|-------|-------------------|----------|
| 0 | Overview | Orient — read this before doing anything | Once |
| 1 | Personas | Name the distinct actors, as proto-personas | Rarely (append-only when a new Role/Actor appears) |
| 2 | Master Workflow Inventory | Exhaustive task inventory + bidirectional coverage matrices | Refresh whenever the domain spec/BC/UX-spec surface changes |
| 3 | Per-persona journey maps | Walk each persona across the full inventory | Refresh per persona when their workflow set changes |
| 4 | Design-language direction exploration | Structured visual-direction decision | Once per major brand milestone |
| 5 | Divergence (fat-marker/thumbnail) | Cheap pre-hi-fi hole-finding | Per net-new frame |
| 6 | Hi-fi storyboard frames | Production-fidelity narrative frames, full state coverage | Per frame, per state added |
| 6.5 | Design validation *(Enhancements-v2)* | AI-runnable heuristic evaluation + cognitive walkthrough + usability-test-readiness prep, gating promotion | Per frame, before Stage 8 |
| 7 | Evidence & responsive validation | Deterministic per-breakpoint screenshots + manifest | Per frame, every time its HTML changes |
| 8 | UX-spec modernization + traceback | Promote storyboard findings into the formal UX-spec | Per stabilized frame / per closed gap |

> **Enhancements-v2 burst (2026-07-07, ux-designer, this pass).** Stage 6.5 is new — see its full
> definition between Stage 6 and Stage 7 below (`.factory/analysis/storyboard-research-v2-validation-structure.md`
> Angle E).

A **fresh agent picking this up cold** should: read Stage 0 in full, confirm Stages 1–2 already exist
(read them, don't re-derive), then find the specific stage matching the task at hand (new persona? new
workflow gap? new frame? evidence backfill? promote to spec?) and execute that stage's **Steps**
against that stage's **Template**, checking the stage's **Acceptance criteria** before declaring done.

---

## Artifact Map

```
<project-root>/storyboard/<version>/
├── WORKFLOW-INVENTORY.md              -- Stage 2 output (single file, append-only)
├── journeys/
│   └── journey-<persona-code>.md      -- Stage 1 (persona brief §1) + Stage 3, one file per persona
├── design-language/
│   ├── DESIGN-BRIEF.md                -- Stage 4 step 1
│   ├── DECISION-MATRIX.md             -- Stage 4 step 3-4
│   └── direction-NN-<slug>/           -- Stage 4 step 2, one dir per direction
│       ├── index.html / styles.css / README.md / preview.png
├── frames/
│   └── frame-<id>-<slug>/             -- Stage 5 + Stage 6, one dir per frame
│       ├── sketch/                    -- Stage 5 output (thumbnail/fat-marker + hole-finding notes)
│       ├── styles.css
│       ├── state-<letter>-<name>.html -- one file per rendered state/path (Stage 6)
│       ├── README.md                  -- why/files/moves/mobile/persona/deliberate-non-scope
│       └── frame-<id>-narrative.md    -- optional: Before→After + principle-by-principle audit
└── ui-evidence/<phase-tag>/
    └── frame-<id>-<state>-<W>x<H>.png -- Stage 7 output, one PNG per (state × breakpoint)
    (+ EVIDENCE-MANIFEST.md or a manifest table inside the frame README)

<project-root>/specs/ux/
├── UX-INDEX.md                        -- Stage 8 target
├── screens/SCR-*.md                   -- Stage 8 target
└── flows/FLOW-*.md                    -- Stage 8 target
```

> **EXAMPLE (Rivetry):** the live tree is `.factory/storyboard/v0.1.0-greenfield/...` and
> `.factory/specs/ux/...`; evidence lives at `.factory/ui-evidence/phase-b-responsive/`. Rivetry's
> existing evidence set predates this runbook's Stage 7 and uses a **4-row, width-only** naming
> (`frame-<id>__<state>__<W>.png`, only 320/390/768/1440) — flagged in Stage 7 as legacy, not a model
> to copy for new work.

> **Enhancements-v2 burst (2026-07-07, ux-designer, this pass).** The tree above is the v1.0 canonical
> structure and remains correct as far as it goes — every artifact class it names is still stored
> exactly once by stable ID. What it lacks is a **view/index layer** for opening one persona and seeing
> all of that persona's proof at a glance, a coverage cube proving "every persona × every workflow"
> completeness, and a few new artifact classes Stages 1, 4, and 6.5 now produce. The extended,
> project-AGNOSTIC tree below (`.factory/analysis/storyboard-research-v2-validation-structure.md`
> Angle F — docs-as-code single-source-of-truth + generated-view pattern: DITA/Antora transclusion,
> Sphinx/Antora/Docusaurus nav-decoupled-from-files, StrictDoc graph→matrix, Docusaurus versioned-docs,
> ADR immutability) is now the **agnostic default** going forward — it is additive over, not a
> replacement for, the tree above: nothing in the original tree moves or is renamed, `personas/` and
> `STORYBOARD-INDEX.md`/`VERSIONS.md` are pure additions, and `design-language/tokens/` and the new
> per-frame files (`HEURISTIC-EVAL.md`, `COG-WALKTHROUGH.md`, `USABILITY-TEST-PLAN.md`) slot into the
> existing `design-language/` and `frames/frame-<id>-<slug>/` directories named above.

### Extended Artifact Map (Enhancements v2 — agnostic default)

```
<project-root>/storyboard/<version>/          -- <version>/ is an IMMUTABLE snapshot; see Versioning Rule below
│
├── STORYBOARD-INDEX.md                        -- NEW: top-level nav + legend (CANONICAL vs VIEW dirs)
│                                                  + pointer to the coverage cube + this version's status
│
│   ══ CANONICAL STORES — each artifact stored ONCE, keyed by stable ID (no per-persona duplication) ══
│
├── WORKFLOW-INVENTORY.md                       -- Stage 2; canonical WF-*/CAP-* store (append-only)
│
├── frames/
│   └── frame-<id>-<slug>/                      -- canonical FRAME store: one dir per frame-id, NEVER per persona
│       ├── sketch/                             -- Stage 5
│       ├── state-<letter>-<name>.html          -- Stage 6 (one per state/path)
│       ├── styles.css                          -- Stage 6; composition-only, consumes design-language/tokens/tokens.css
│       ├── README.md                           -- frontmatter carries persona_ids:[…] + workflow_ids:[WF-…]
│       ├── HEURISTIC-EVAL.md                   -- NEW: Stage 6.5 (or a README section)
│       ├── COG-WALKTHROUGH.md                  -- NEW: Stage 6.5 (or a README section)
│       └── USABILITY-TEST-PLAN.md              -- NEW: Stage 6.5 (prepared, human-gated)
│
├── ui-evidence/<phase-tag>/                    -- canonical SCREENSHOT store, INSIDE the version root
│   ├── frame-<id>-<state>-<W>x<H>.png          -- one PNG per (frame × state × breakpoint), Stage 7
│   └── EVIDENCE-MANIFEST.md
│
├── design-language/                            -- canonical DIRECTION store
│   ├── DESIGN-BRIEF.md · DECISION-MATRIX.md    -- Stage 4
│   ├── direction-NN-<slug>/{index.html, styles.css, README.md, preview.png}
│   └── tokens/                                 -- NEW: Stage 4 step 6 (DTCG token SSoT)
│       ├── <name>.tokens.json                  -- DTCG 2025.10 format, primitive/semantic/component tiers
│       └── tokens.css                          -- generated via Style Dictionary `css/variables`
│
│   ══ VIEW / INDEX LAYER — links INTO the canonical stores; ZERO artifact duplication ══
│
├── personas/                                   -- NEW: the persona-navigation layer
│   ├── PERSONA-ROSTER.md                        -- stable HOME for the roster (name · code · role · JTBD 1-liner)
│   ├── PERSONA-WORKFLOW-MATRIX.md               -- the COVERAGE CUBE (persona × workflow × state, status + links)
│   └── persona-<code>.md                        -- per-persona INDEX: every WF this persona touches →
│                                                   link to frame dir + link to its evidence + status glyph
│
└── journeys/
    └── journey-<persona-code>.md                -- Stage 3 NARRATIVE (emotion curve, service blueprints) — unchanged

<project-root>/storyboard/VERSIONS.md            -- NEW: registry of every <version>/, marks exactly one `latest`
```

> **Reconciling the `ui-evidence/` location (Enhancements-v2 burst, additive note — does not delete the
> legacy EXAMPLE above).** Both the v1.0 tree and this extended tree place evidence INSIDE
> `<version>/ui-evidence/`. Rivetry's live tree diverges — its evidence currently lives OUTSIDE the
> version root at `.factory/ui-evidence/phase-b-responsive/`, a drift the v2 research flagged directly
> against this same Artifact Map. This is confirmed as a real reconciliation item (move the legacy set
> into the version root, or retain it read-only and start new phase-tags inside the root going forward)
> — attached to a concrete future housekeeping burst per this project's own governance, not left a bare
> "later." The legacy EXAMPLE block above remains as-is; this note does not alter it.

> **Status glyph vocabulary (Enhancements-v2 burst).** The coverage cube (`PERSONA-WORKFLOW-MATRIX.md`)
> and each `persona-<code>.md` use a glyph per cell, one glyph per pipeline stage that produces it — see
> the Naming & ID Conventions section below for the full lifecycle table.

```markdown
<!-- personas/PERSONA-WORKFLOW-MATRIX.md — EXAMPLE cells only -->
| WF-ID | Workflow | Stage | <persona-1> | <persona-2> | ... |
|-------|----------|-------|-------------|-------------|-----|
| WF-NNN | <name> | <lifecycle stage> | 🎨 | 🔍 | ... |
```

```markdown
<!-- personas/persona-<code>.md — a VIEW; links into canonical stores, duplicates nothing -->
# Persona Index — <Name> (<CODE>)   [VIEW — links into canonical stores; no duplicated artifacts]

Roster: ../personas/PERSONA-ROSTER.md · Narrative journey: ../journeys/journey-<code>.md

| WF-ID | Workflow | Hi-fi mock | Screenshots | Status |
|-------|----------|-----------|-------------|--------|
| WF-NNN | <name> | [frame-<id>](../frames/frame-<id>-…/README.md) | [N bp](../ui-evidence/…/) | <glyph> |

Coverage: N of M workflows evidenced · gaps: <list ⬜/🎨 rows>
```

The matrix and per-persona indexes are **generated views**, pivoted from frame-README frontmatter
(`persona_ids`, `workflow_ids`) + the evidence manifest + Stage 2's Persona(s) column — never
hand-maintained divergently from the canonical stores (Angle F, StrictDoc's graph→matrix model).

> **EXAMPLE (Rivetry):** applying this tree, `personas/PERSONA-ROSTER.md` would list EE/MD/TA/BAI/SR/
> CMR/GDPR; `personas/PERSONA-WORKFLOW-MATRIX.md` would be a 52-row (WF-001..WF-052) × 7-persona grid;
> `personas/persona-MD.md` would link WF-022/WF-052/etc. straight into `frames/frame-01b-…/README.md`
> and `frames/frame-12-…/README.md` plus their evidence sets — no content copied, only linked.

---

## Naming & ID Conventions

- Every artifact class gets a **stable, append-only, never-reused ID prefix** chosen once per project
  (e.g. `WF-`, `CAP-`, `SCR-`, `FLOW-`, `BC-`, `frame-`, `direction-`). IDs are assigned in the order
  a pass first mints them; a later refresh pass **inserts** a new ID into its correct structural
  location (e.g. the correct lifecycle-stage table row) without renumbering anything that already
  exists.
- Every new artifact **traces to at least one upstream ID** (a capability, a BC, an ADR, a prior
  workflow) — an untraceable artifact is a defect, not a stylistic choice (see Stage 2's
  zero-orphan discipline).
- Every burst that adds, corrects, or supersedes prior content documents itself **inline, in place**,
  using a dated, attributed note (`> **<Burst name> (<date>, <agent>, this pass).** ...`) rather than
  silently rewriting prior claims — this is what makes the corpus's own history auditable and is a
  hard requirement, not a style preference.

> **EXAMPLE (Rivetry):** `WF-001..WF-052`, `CAP-001..CAP-023`, `SCR-001..SCR-020`, `FLOW-001..FLOW-018`,
> `frame-01..frame-12` (with `01b`/`01c` sibling suffixes for same-workflow variant explorations),
> `direction-01..direction-04`. See `WORKFLOW-INVENTORY.md`'s own numbering note for the append-only
> discipline stated in the corpus's own words.

> **Enhancements-v2 burst (2026-07-07, ux-designer, this pass) — Status Glyph Lifecycle.** The coverage
> cube and per-persona indexes (Artifact Map, Angle F) status every (persona × workflow) cell with a
> glyph drawn from the pipeline stage that produced it — never a hand-typed word, so the vocabulary is
> fixed and machine-checkable:
>
> | Glyph | Meaning | Produced by |
> |---|---|---|
> | `—` | N/A — this persona is not an actor for this workflow | Stage 2 Persona(s) column |
> | `⬜` | not started | — |
> | `✏️` | sketched | Stage 5 |
> | `🎨` | hi-fi built | Stage 6 |
> | `🔍` | design-validated | Stage 6.5 |
> | `📸` | evidenced (breakpoint manifest green) | Stage 7 |
> | `✅` | promoted to SCR-*/FLOW-* | Stage 8 |
>
> A cell only advances glyphs in this order; a glyph regression (e.g. `📸` → `✏️`) signals a real rebuild,
> not a typo, and should carry its own dated note explaining why.

> **Enhancements-v2 burst (2026-07-07, ux-designer, this pass) — Anti-Persona ID Class.** Anti-personas
> (Stage 1) get their own ID prefix (e.g. `AP-`) and their own roster, kept **structurally separate** from
> the designed-for persona roster and its `orphan check` — an anti-persona deliberately maps to no
> designed-for workflow and would fail (and pollute) that check if lumped in. Anti-personas instead trace
> to the **abuse vectors** they are defended against, annotated on the relevant Stage 3 service-blueprint
> backstage/line-of-interaction. See Stage 1 for the full definition
> (`.factory/analysis/storyboard-research-v2-persona-workflow.md` Angle A, NN-A2, and its CONFLICT-if-lumped
> flag).

> **Enhancements-v2 burst (2026-07-07, ux-designer, this pass) — Versioning Rule.** `<version>/` (the
> Artifact Map's root) is an **immutable snapshot**. When the version rolls, copy-on-write a NEW version
> directory (the Docusaurus `versioned_docs` model) rather than editing the prior one in place; the prior
> directory is retained **read-only** and gets a superseded banner (the same ADR `Supersedes:` discipline
> this runbook already uses for in-place dated annotations — see Stage 8). A root
> `storyboard/VERSIONS.md` registry (an "index of indices," à la Docusaurus's `versions.json`) lists every
> version, marks exactly one `latest`, and links each; views/matrices reference artifacts by
> version-relative path so links survive the roll. Keep the live set small (fewer than ~10 versions;
> archive older ones). Canonical stores (`frames/`, `ui-evidence/`, `design-language/`) are frozen within
> a version; the `personas/` view layer and `journeys/` are regenerated/updated within the *current*
> version only. (`.factory/analysis/storyboard-research-v2-validation-structure.md` Angle F, Pattern 3.)

---

## Dual-Adversary Cascade — Record Conventions

> **Burst 323 codification (2026-07-31, ux-designer, this pass).** Both conventions in this section are
> DESCRIPTIVE of practice first demonstrated in the email-notifications Stage-3 dual-adversary cascade
> (workstream 3). They were practiced before being named here. The check being added as Lever A/B item
> B+9 (check L19) will enforce the dual-adversary cascade-ledger-row schema mechanically, using this
> section as its ratified source of truth. The check's scope is defined by this text; the check
> enforces the text, not the reverse.

This section defines the two conventions governing **dual-adversary cascade ledger rows** — the rows
recorded in `STORYBOARD-INDEX.md`'s adversarial cascade table for passes run by two independent
`vsdd-factory:adversary` reviewers against the same frozen HEAD in one pass. Both conventions belong in
the Findings cell of every such row.

### Convention: `mechanism-core` re-derivation attestation

**What it means.** Each adversary reviewer independently re-derives the feature's mechanism core from
the authoritative source worktrees during the pass and attests whether the re-derived mechanism is
CLEAN (consistent with the spec and free of structural defect). A consecutive-CLEAN count is tracked
across passes and reported in the row.

**When it applies.** Every dual-adversary cascade ledger row in a feature-focused cascade where a
mechanism-core host set is defined.

**Authoritative host list.** The host list is read from the STATE.md canon, never from a procedure
note or UI-surface reference. For the email-notifications cascade, the canonical hosts are
`kafka-connector-jirasm` and `clip-connector`. The UI-surface hosts `clipfe` and `clip-admin` are NOT
mechanism-core hosts — they contain zero mechanism-core symbols and must not be substituted.
The two host classes are distinct and non-overlapping: UI-surface hosts remain correct and
load-bearing for theme and component-identity claims, and those claims are a separate concern from
mechanism-core re-derivation. Dispatches mandating mechanism-core re-derivation MUST name
`kafka-connector-jirasm` and `clip-connector` and MUST NOT name `clipfe` or `clip-admin`.
(Codified: lessons.md lesson-61.)

**Practiced forms.** Three forms appear in the corpus; all attest the same underlying truth:

- Compact consecutive-count form (first established with pass S3-51):
  `mechanism-core CLEAN THIRD consecutive dual`

- Expanded form naming the re-derivation explicitly (passes S3-53 and S3-54):
  `mechanism core re-derived CLEAN by both (FIFTH consecutive dual mechanism-core-CLEAN)`

- Full attestation with host-compliance clause (pass S3-69, after the Burst-313 host-list misdirection
  incident documented as finding UL8 in pass S3-70):
  `Mechanism core CLEAN on BOTH independent source re-derivations; both S3-69 reviewers were dispatched
  against the correct mechanism-core hosts and both executed source reads in those trees`

**Documented variant — hyphenation.** The term appears both hyphenated (`mechanism-core`) and
unhyphenated (`mechanism core`) across corpus rows. The hyphenated form is the canonical spelling in
lessons.md lesson-61 and in STATE.md; either form is accepted in ledger row prose.

**Boundary not determined by current practice.** Whether mechanism-core re-derivation is required for
all future dual-adversary cascades on any product feature, or only for features with an explicitly
designated mechanism-core host set, is not settled by current practice. All attested occurrences are
in the email-notifications Stage-3 cascade; generalization to other features has not been established.

### Convention: per-reviewer split of the finding mass

**What it means.** A dual-adversary cascade ledger row records each reviewer's finding mass
separately, before the deduped union. "Finding mass" denotes the count of each reviewer's findings by
severity tier, written as `<count>C/<count>H/<count>M/<count>L/<count>OBS` (Critical / High / Medium /
Low / Observation), or in an abbreviated additive form showing only the non-zero tiers separated by
`+`.

**When it applies.** Every dual-adversary cascade ledger row. Recording only the union without the
per-reviewer breakdown is a defect: the S3-69 ledger row was initially authored without the
per-reviewer split and was retroactively corrected as finding UL10 in pass S3-70 (fix-burst 114).

**Dominant practiced form.** The Findings cell records both per-reviewer masses before the union:

`Reviewer A = 0C/2H/0M/3L/1OBS (refuted ×13); Reviewer B = 0C/0H/2M/4L/1OBS (refuted ×14)`

This full CHMLO form — including the `(refuted ×count)` refutation annotation — is the dominant form
used across the majority of email-notifications Stage-3 dual passes.

**Documented variants.** Three forms appear in the corpus:

1. **Full CHMLO with refuted count** (dominant; from S3-48 onward through the middle of the cascade):
   `Reviewer A = 0C/2H/0M/3L/1OBS (refuted ×13); Reviewer B = 0C/0H/2M/4L/1OBS (refuted ×14)`
   Sub-variant (pass S3-49): the `=` sign is dropped:
   `Reviewer A 0C/3H/2M/1L/1OBS (refuted ×20); Reviewer B 0C/2H/5M/2L/2OBS (refuted ×18)`
   Both sub-variants are within the dominant-form family.

2. **CHMLO without OBS tier and without refuted count** (single occurrence, pass S3-70):
   `Reviewer A 0C/1H/3M/2L, Reviewer B 0C/0H/4M/2L`

3. **Abbreviated additive form — non-zero tiers only** (single occurrence, applied retroactively to
   pass S3-69 as fix-burst 114):
   `reviewer A 3M+2L, reviewer B 3M+2L`
   Used when the full CHMLO breakdown was not available for reconstruction at correction time.

**Boundary not determined by current practice.** Whether the refuted-findings count annotation
(the `refuted ×13` style) is required or optional for CLIP: S3-70 omits it while most earlier passes
include it. Whether the OBS tier must be shown explicitly when zero or may be omitted when the
reviewer raised no observations. Both questions are open.

---

## Stage 0 — Overview

### Purpose

Persona-storyboarding is a **narrative-first UX exploration method**: before (or alongside) writing the
formal, implementation-ready UX spec, build a parallel exploration corpus — proto-personas, an
exhaustive workflow inventory, per-persona journey maps, a structured visual-direction decision, and
hi-fi narrative "storyboard frames" with deterministic visual evidence — that surfaces gaps,
contradictions, and design opportunities the spec alone would not, and that produces content mature
enough to **promote into the formal UX spec** once stabilized (Stage 8).

It is not a replacement for `SCR-*`/`FLOW-*` UX-spec files. It is the exploration layer that feeds them,
the same way a story map or a design sprint feeds a backlog rather than replacing it.

### When to Run It in the Pipeline

Run persona-storyboarding **during the UX design phase** — after the domain spec (capabilities,
entities, Actors/Roles) and the PRD/behavioral-contracts exist (so there is a real capability/BC surface
to trace to), and either before or in parallel with the formal `UX-INDEX.md`/`SCR-*`/`FLOW-*` authoring.
It is equally valid to run it:
- **Greenfield, exploratory-first:** storyboard before the formal spec, then promote (Stage 8) into the
  spec once directions and flows stabilize.
- **Greenfield, spec-first:** author the formal spec first, then run this process as a **modernization
  pass** against an existing screen (exactly how Rivetry's own `frame-01` → `frame-01b` redesign
  happened — see Stage 6).
- **Brownfield / redesign:** run Stage 2 against the *existing* product's capability surface to find
  screens with no clean workflow story, then target Stages 3–7 at the highest-friction workflows first.

### Full Artifact Map

See **Artifact Map** above.

### The Trace Chain

Every artifact this process produces must be placeable on one chain:

```
user-need / JTBD  →  workflow (WF-*)  →  capability (CAP-*)  →  screen/flow (SCR-*/FLOW-*)  →  verification (BC-*/VP-*/test)
```

This is the RTM-literature "canonical need-to-test chain," lifted explicitly above the capability layer
per Enhancement #6 (`storyboard-process-research.md` §3, §9.2) — previously the chain in Rivetry's own
corpus started at `capability`; this runbook requires the `need/JTBD` layer to sit above it from the
first pass, not be retrofitted later. Concretely:
- Each **persona** (Stage 1) carries a JTBD statement — the top of the chain.
- Each **workflow row** (Stage 2) cites the JTBD/need it serves, the capability it maps to, and — new
  per this runbook — a **verification column** naming the BC/VP/test that proves it (closing the chain
  at the bottom, not leaving it dangling at "has a screen").
- Each **frame** (Stage 6) traces back up through its workflow to its capability and BC.

A workflow, screen, or frame that cannot be placed on this chain is either mis-scoped (doesn't belong in
this pass) or reveals a genuine spec gap (route it — see Stage 8).

---

## Stage 1 — Personas

### Purpose

Establish the **proto-persona** cast: the distinct behavioral clusters of actor who will walk the
workflow inventory in Stage 3. "Proto-persona" is the honest, deliberately-chosen typing — see
Rationale below.

### Inputs

- Product brief (goals, ICP)
- Domain spec: Actors/Roles, entity ownership
- Behavioral contracts: any BC that gates an action by Role/authorization level
- PRD: any persona language already present

### Steps

1. **Enumerate every distinct Actor/Role** named in the domain spec and BCs — including
   non-authenticating actors (e.g. an external recipient of a shared artifact) and actors who never
   directly operate the product (e.g. a compliance requester whose request is executed on their
   behalf by another actor).
2. **Apply the "distinct behavioral cluster" inclusion test** to decide the final persona count — the
   literature gives no numeric standard here (`storyboard-process-research.md` §1, §10), so this test
   is the operative rule: *two candidate personas merge into one if they hold identical decision
   authority and goals for every workflow in scope; one candidate splits into two if a single named
   Role actually contains two materially different authorization/goal sets.* Demographics never
   justify a split; distinct decision authority, distinct devices/context, or a distinct never-logs-in
   posture do.
3. **Draft a persona brief** per the template below for each surviving cluster, including:
   - a **provenance + validation-cadence header** — Enhancement #2
     (`storyboard-process-research.md` §1, §9.1): state plainly that this is a **proto-persona**
     (assumption/spec-derived, not user-research-derived) unless it genuinely is research-grounded,
     and name the trigger that would prompt validation (e.g. "validate against real usage once N
     paying tenants exist" / "validate at first customer interview round").
   - a **JTBD statement** — Enhancement #6 (`storyboard-process-research.md` §1, §9.1): `"When
     <situation>, I want to <motivation>, so I can <expected outcome>"`, plus explicit functional and
     emotional success criteria.
   - a **scenario "expectations" line** — what this persona expects going in (speed, rigor, trust
     posture) — this is what shapes the journey's emotion curve in Stage 3.

> **Enhancements-v2 burst (2026-07-07, ux-designer, this pass) — Steps 3a–3d.** Four net-new steps,
> grounded in `.factory/analysis/storyboard-research-v2-persona-workflow.md` Angle A. They run after
> Step 3 (persona briefs exist) and before Step 4 (the orphan check) — Step 3a is Step 4's necessary
> precondition, so Step 4 is now **Gate 2 of 2** (see the refinement note under Step 4 below).

3a. **Ecosystem/onion map + role-permission/RACI completeness pass (Gate 1 of 2).** Before trusting the
    spec's own Actor/Role list, build a **stakeholder/onion map** (primary → secondary → tertiary →
    served → negative layers) and a **role-permission matrix + RACI** (rows = tasks, cols = roles) over
    the product's real ecosystem — including behind-the-scenes systems, partners, regulators, and data
    intermediaries, not only spec-listed actors. Annotate every node/role
    `persona-covered | shared-persona | system-component-only | anti-persona | deliberately-out-of-scope`
    — no unannotated node. A role holding distinct create/read/update/delete/**moderate/approve**
    permissions, or an accountable (RACI "A") role that never appears as a designed-for persona, is a
    **missing persona**, caught here before it ever reaches the orphan check. This is *necessary but not
    sufficient* completeness — the literature is explicit that no formal completeness proof exists for a
    persona cast; both this gate and Step 4 are **risk-aware coverage arguments**, not guarantees. (Angle
    A, "Completeness test beyond the orphan check"; NN-A4.)
3b. **Persona Spectrum coverage pass.** For each core interaction demand drawn from the workflow
    inventory (Stage 2), run it through the Microsoft Inclusive Design **Persona Spectrum** lens —
    **permanent / temporary / situational** variants across the relevant ability and cognitive domains
    (vision, hearing, motor, cognitive: e.g. permanent ADHD/dyslexia, temporary acute stress/fatigue,
    situational multitasking/crisis) — and record the result as an **ability × mismatch-type matrix** in
    the roster. **This refines, it does not violate, Step 2's "demographics never justify a split" rule:**
    Persona Spectrum variants attach to an **existing** persona as ability/context **dimensions** (e.g.
    "MD-in-bright-fab-light," "reviewer-under-time-pressure"), they never become new demographic
    personas. (Angle A, NN-A1; Microsoft *Inclusive 101* / cognition toolkit.)
3c. **Anti-persona roster — a separate, first-class artifact class.** Build a *negative-persona* track
    (own ID prefix, e.g. `AP-`; see Naming & ID Conventions) using the NN/g anti-persona template — **name
    + face · goal (the threat) · motivations · actions · tools · needs (the absences that let the threat
    succeed) · consequences** — built around **behaviors and goals, not identities**. Trigger it wherever
    the product touches tenant isolation, a safety-load-bearing confirmation, credentials, or erasure;
    low-probability/high-consequence threats still warrant one. **Anti-personas are exempt from the
    Step-4 orphan check** — they deliberately map to no designed-for workflow — and instead trace to the
    *abuse vectors* they must be defended against, annotated on the relevant Stage 3 service-blueprint
    backstage/line-of-interaction. (Angle A, NN-A2; NN/g anti-personas.)
3d. **First-class system/agent personas for load-bearing agents.** Promote any load-bearing AI
    agent/automation/external system from *touchpoint-only* (Stage 3's existing "Agentic touchpoint"
    field) to *actor*: give it a short **system-persona brief** — capabilities, decision criteria, hard
    boundaries, escalation/hand-back triggers, known failure modes — honestly typed as a **design spec,
    not a research-grounded model** (no authority comparable to the Persona Spectrum or the NN/g
    anti-persona template exists yet for this class; keep the human cast central, do not anthropomorphize
    past its stated boundaries). This refines, not replaces, the Agentic-touchpoint field: the field
    records *where* an agent appears; the brief records *what it is and where it fails*. (Angle A, NN-A3.)

4. **Orphan check (Gate 2 of 2):** confirm every Actor/Role named in the spec corpus maps to exactly one
   persona (never zero, never split silently across two personas without the split being the deliberate
   Step-2 outcome).

> **Enhancements-v2 burst (2026-07-07, ux-designer, this pass) — refinement note.** This orphan check
> proves coverage of *listed* actors only; it is now explicitly **Gate 2 of 2**. Step 3a is **Gate 1**
> (the ecosystem/RACI proof that the spec's own actor list omitted nothing). A green orphan check without
> Step 3a gives false confidence — run both. (Angle A, "REFINES Stage 1 orphan check.")

### Output Artifact(s) + Path Convention

- A **persona roster table** (name, code, one-line role) — lives at the top of `WORKFLOW-INVENTORY.md`
  (Stage 2) or in its own `journeys/PERSONA-ROSTER.md` for a large cast.
- The **full persona brief** — embedded as `journeys/journey-<persona-code>.md` §1 (co-located with
  that persona's journey, Stage 3's convention) for small-to-medium casts, or as a standalone
  `journeys/personas/PERSONA-<CODE>.md` for a large cast reused across multiple journey/service-blueprint
  artifacts. Pick one convention per project and apply it consistently.

> **Enhancements-v2 burst (2026-07-07, ux-designer, this pass).** Where a project has adopted the
> Extended Artifact Map's `personas/` view layer (Angle F), the roster's **one stable home** is
> `personas/PERSONA-ROSTER.md`, co-located with the coverage cube and the per-persona indexes — this
> supersedes the "top of WORKFLOW-INVENTORY.md vs. own file" either/or above for projects on that layer;
> projects not yet on the view layer keep the original latitude. The anti-persona roster (Step 3c) lives
> at `personas/ANTI-PERSONA-ROSTER.md` (or `journeys/ANTI-PERSONA-ROSTER.md` absent the view layer) — a
> **separate file from the designed-for roster**, never merged into it. System/agent-persona briefs
> (Step 3d) live alongside the persona briefs they parallel, one per load-bearing agent.

### Template

```markdown
## Persona Brief — <Full Name> (<CODE>)

**Role:** <product Role/Actor name>, <auth posture — e.g. "authenticated Tenant member" |
  "never authenticates">

**evidence_basis:** proto-persona (spec/BC-derived, unvalidated) | research-based (cite study)
**validate_by:** <concrete trigger — e.g. "first N customer interviews" | "at GA">

**Goals:**
- <goal 1, tied to a capability/BC>
- <goal 2>

**JTBD:** "When <situation>, I want to <motivation>, so I can <expected outcome>."
  Functional success: <criterion>. Emotional success: <criterion>.

**Scenario expectations:** <what this persona expects re: speed / rigor / trust before they start>

**Context:** <primary work context — device mix, environment, frequency of use>

**Devices:** <primary / secondary / tertiary, with an honest note on which workflows are full-fidelity
  vs. reduced on each>

**Success feeling:** *"<first-person quote describing the ideal outcome>"*
```

### Acceptance Criteria (Quality Gate)

- [ ] Every Actor/Role in the domain spec + BCs maps to exactly one persona (orphan check passes).
- [ ] Every persona passed the distinct-behavioral-cluster test (no demographic-only split; no
      merge that loses a real authorization/goal difference).
- [ ] Every persona brief carries the `evidence_basis` + `validate_by` header — no untyped persona.
- [ ] Every persona brief carries a JTBD statement with both functional and emotional success
      criteria, and a scenario-expectations line.

> **Enhancements-v2 burst (2026-07-07, ux-designer, this pass) — new criteria (Steps 3a–3d).**
> - [ ] An ecosystem/onion map + role-permission/RACI matrix exists this pass (Gate 1); every node/role
>       is annotated persona-covered | shared-persona | system-component-only | anti-persona |
>       deliberately-out-of-scope — no unannotated node.
> - [ ] Every core interaction demand has at least one identified permanent, temporary, *and* situational
>       Persona Spectrum variant, or an explicit "N/A for this product" note; no Spectrum variant was
>       added as a new demographic-split persona.
> - [ ] Every product surface handling sensitive data or a safety invariant has ≥1 anti-persona (built
>       from behaviors/goals, not identities), or a stated risk-accepted rationale; the anti-persona
>       roster is a separate artifact from the designed-for roster and was **not** run through the Step-4
>       orphan check.
> - [ ] Every load-bearing agent/automation/external system named in the spec corpus that drives design
>       decisions has a system-persona brief, honestly typed as a design-spec (not research-grounded).

> **EXAMPLE (Rivetry):** four core personas (EE, MD, TA, BAI) plus three proposed additions (SR, CMR,
> GDPR) — none merged/split on demographics, all justified by distinct authorization posture (SR =
> DI-015's dedicated confirm-privilege Role) or a distinct never-logs-in posture (CMR, and the GDPR
> requester who never directly operates the product). See `WORKFLOW-INVENTORY.md`'s own "Proposed
> Persona Additions" section for the full justification, and `journeys/journey-mechanical-designer.md`
> §1 for a worked persona-brief example (Devon Cole).

---

## Stage 2 — Master Workflow Inventory

### Purpose

Produce the single, exhaustive, append-only inventory of every user-facing (and load-bearing
backend-invariant) workflow implied by the spec corpus, with bidirectional traceability matrices proving
zero orphans in both directions. This is the project's Requirements Traceability Matrix fused with a
Jeff Patton story map (`storyboard-process-research.md` §3) — the single strongest asset the method
produces, per the research; do not dilute it.

### Inputs

- Domain spec: capabilities (CAP-*), entities, Actors
- BC-INDEX (or equivalent behavioral-contract index)
- Architecture: API/Operation surface, subsystem registry
- UX-INDEX + existing SCR-*/FLOW-* (if any exist yet — empty on a true first pass)
- Product brief's phased-scope framing (IN/OUT), if one exists

### Steps

1. **Define the lifecycle-stage spine** — a small number (4–8) of ordered stages that reflect the
   product's actual usage arc (e.g. onboarding → managing → creating → the core payoff moment → any
   agent/automation layer → the full composed end-to-end arc). This spine is project-specific; pick
   stages that will hold every workflow without forcing an awkward fit.
2. **Walk every capability** and enumerate the workflow(s) it implies. For a capability whose task is
   genuinely complex, apply **Hierarchical Task Analysis** (decompose goal → subgoal → task → subtask
   until each step is inspectable for omissions) rather than writing one under-specified row
   (`storyboard-process-research.md` §3).

> **Enhancements-v2 burst (2026-07-07, ux-designer, this pass) — Steps 2a–2b.** Grounded in
> `.factory/analysis/storyboard-research-v2-persona-workflow.md` Angle B. HTA (Step 2) *decomposes*; it
> does not *generate* the exception/failure set. These two steps supply the missing generative methods.

2a. **Cockburn extension-condition brainstorm (generative method behind the Path-coverage column).** For
    each workflow's 3–9-step main success scenario, *exhaustively* brainstorm every condition the system
    can detect at each step, then write the extension-handling step for each — every extension ends back
    in the main scenario, at a separate success exit, or in failure. Promote any extension that has its
    own screen/state into its **own WF row** (not a note in an existing row). This is the generative
    engine behind Step 3's Path-coverage column below — see the refinement note on that column. (Angle B,
    NN-B1; Cockburn use-case extension conditions.)
2b. **Lightweight FMEA pass on safety/tenancy workflows.** For any workflow carrying a safety or tenancy
    invariant, run a lightweight FMEA in user terms: function → failure mode → effect → severity → cause
    → detection/control, with RPN = Severity × Occurrence × Detection as an optional risk rank. Latent
    recovery/verification/escalation workflows this surfaces become new WF rows; the severity/RPN feeds
    the Storyboard-priority column (Step 7 below). (Angle B, NN-B2; FMEA/FMECA + ISO 14971
    reasonably-foreseeable-misuse.)

3. **Assign each workflow a stable ID**, inserted into its correct lifecycle-stage table, with columns:
   `WF-ID | Workflow | Persona(s) | CAP-NNN | Key BCs/Operations | Need/JTBD | UX Coverage | Path
   coverage | Verification | Storyboard priority | MVP Scope | Notes`.
   - **Need/JTBD column** (Enhancement #6): cite the persona JTBD (Stage 1) this workflow serves —
     this is what lifts the chain above CAP per the Stage-0 trace chain.
   - **UX Coverage column:** the SCR-*/FLOW-* IDs that cover it, or an explicit gap marker
     (e.g. `⛔ NO SCREEN`) with a one-line reason — never a silent blank cell.
   - **Path coverage column** (Enhancement #6c): does the corpus's current coverage include the
     *exception/recovery* path, not just the happy path? Mark `happy-only`, `happy+recovery`, or
     `N/A` (no meaningful exception path for this workflow) — never leave this column implicit.
     > **Enhancements-v2 burst (2026-07-07, ux-designer, this pass) — refinement.** #6c originally
     > *marked* this column's status; Steps 2a/2b/3a now *generate* its content (Cockburn extensions,
     > FMEA, the state machine) — read this as "generate the column's content, then audit," not "audit
     > an empty column." (Angle B, refinement flag on Enhancement #6c.)
   - **Verification column** (Enhancement #6b): the BC/VP/test ID(s) that prove this workflow's
     behavior — the chain's terminal link.
   - **Storyboard priority column** *(Enhancements-v2, new)*: a defensible, recomputable sequencing rank
     for "which workflow gets storyboarded first" — see Step 7 below for the formula. Replaces an ad-hoc
     "highest-risk/highest-density/Aha!-moment" pick with a stated score.

> **Enhancements-v2 burst (2026-07-07, ux-designer, this pass) — Step 3a.** Grounded in
> `.factory/analysis/storyboard-research-v2-persona-workflow.md` Angle B, NN-B3.

3a. **State-machine + error/recovery-state enumeration.** For any workflow complex enough to warrant it,
    model it as a finite state machine — e.g. draft → validated → submitted → under-review →
    approved/rejected/cancelled, **plus explicit error and recovery states** (awaiting-retry,
    awaiting-support, partial-rollback) that task decomposition (HTA) alone tends to miss. Derive
    state-coverage, transition-coverage, and *invalid*-transition (error) coverage from the machine. This
    state set becomes the **authoritative source** for Stage 6's "enumerate the full state set" step and
    Stage 7's per-state evidence rows — closing the loop between this column and the rendered-state
    coverage the runbook already requires.

4. **Build the coverage matrices** — at minimum: capability → workflow(s), subsystem → workflow(s), and
   (once screens/flows exist) screen/flow → workflow(s). Each matrix's finding line must state
   explicitly whether it found **zero orphans** or list every orphan found.
5. **Run the exception/recovery path-coverage audit** as its own pass across the finished inventory:
   scan the Path-coverage column for `happy-only` rows and produce a short list of candidates for a
   dedicated recovery-path frame (Stage 6) or journey narration (Stage 3).

> **Enhancements-v2 burst (2026-07-07, ux-designer, this pass) — Step 5a.** Grounded in
> `.factory/analysis/storyboard-research-v2-persona-workflow.md` Angle B, NN-B4.

5a. **Cross-persona workflow discovery pass (feeding the Stage 3 service blueprint).** Single-persona
    journey walks (Stage 3) structurally miss workflows that exist only **between** actors — handoffs,
    approvals, escalations, delegations, backstage corrections. Run an explicit inter-actor discovery
    step over the actor cast using **swimlane / cross-functional flowcharts** (lanes = roles/systems;
    every cross-lane arrow is a candidate handoff/approval/escalation) plus a **RACI matrix**
    (multiple R/C on one task ⇒ collaborative workflow; A shifting across roles ⇒ escalation/delegation;
    I-only ⇒ a notification workflow to design). The lane-identification step itself can surface hidden
    roles (e.g. risk officers, compliance reviewers) — route any newly surfaced role back through Stage 1
    Step 3a. Each discovered cross-lane arrow/accountability-shift becomes a candidate WF row. **This
    composes with, and feeds, the already-folded-in Stage 3 service blueprint (Stage 3 Step 6) rather
    than duplicating it** — this step *discovers* the inter-actor workflow; the blueprint *renders* it.

6. **Run the "silently-missed-capability" audit** every time the inventory is refreshed: re-diff the
   coverage matrix's capability list against the domain spec's **current, live** capability list —
   never trust that the matrix "still ends where it always ended." A matrix that silently stops at an
   old max capability ID after new capabilities have landed is a real defect class, not a hypothetical
   one (see the EXAMPLE below).

> **Enhancements-v2 burst (2026-07-07, ux-designer, this pass) — Step 7.** Grounded in
> `.factory/analysis/storyboard-research-v2-persona-workflow.md` Angle B, NN-B5.

7. **Build the coverage cube and compute the Storyboard-priority column.**
   (a) **Coverage cube.** Extend the 2-D coverage matrices (Step 4) to an explicit **persona × workflow ×
   state** cube view — with breakpoint (Stage 7) as an optional 4th axis — so "every persona's every
   workflow's every state is covered" is provable at a glance, not narrated in prose. This lives in
   `personas/PERSONA-WORKFLOW-MATRIX.md` (see the Extended Artifact Map) and is a **generated view**,
   pivoted from frame-README frontmatter + the evidence manifest + this stage's own Persona(s) column —
   never hand-maintained divergently. Flag the cube honestly as an **emerging view, not a codified
   standard** (the 2-D persona × workflow matrix is a clean RTM extension; the explicit 3-D/4-D cube is
   this runbook's own synthesis).
   (b) **Storyboard-priority column.** Compute each workflow's `Storyboard priority` (Step 3's new
   column) using an adapted **WSJF**: Cost of Delay (user/business value + time criticality +
   risk-reduction, with the FMEA severity from Step 2b as the risk-reduction input) ÷ design-effort as
   Job Size. **WSJF = CoD / Job-Size.** This is the recommended default for *design/storyboard*
   sequencing specifically because it explicitly weights risk reduction and time criticality — the axes
   that matter for a safety-load-bearing flow FMEA flags as high-risk. Note **RICE** = (Reach × Impact ×
   Confidence) / Effort and the **JTBD opportunity score** = Importance + (Importance − Satisfaction) as
   valid alternative lenses (RICE for reach/impact framing, JTBD-opportunity for
   user-dissatisfaction framing), and **MoSCoW**/**Kano** as categorical pre-filters — but WSJF is this
   runbook's default because prioritization frameworks are natively backlog/implementation-sequencing
   tools, and WSJF adapts most directly to *design* sequencing's own risk/time axes. This formula
   **replaces** Stage 5's prior ad-hoc "highest-risk/highest-density/Aha!-moment" pick with a defensible,
   recomputable ordering — Stage 5 Step 1 now points here for its ordering rule.

### Output Artifact(s) + Path Convention

- `storyboard/<version>/WORKFLOW-INVENTORY.md` — one file, sectioned by lifecycle stage, append-only.

### Template

> **Enhancements-v2 burst (2026-07-07, ux-designer, this pass).** `Storyboard priority` column added
> below (Step 7b, WSJF default); see the Extended Artifact Map's `personas/PERSONA-WORKFLOW-MATRIX.md`
> for the coverage-cube view (Step 7a) that pivots this same table by persona × workflow × state.

```markdown
| WF-ID | Workflow | Persona(s) | CAP-NNN | Key BCs / Operations | Need/JTBD | UX Coverage | Path coverage | Verification | Storyboard priority | MVP Scope | Notes |
|-------|----------|-----------|---------|----------------------|-----------|-------------|----------------|---------------|----------------------|-----------|-------|
| WF-NNN | <name> | <codes> | <CAP-NNN> | <BC-ids> | <persona code>'s JTBD, 1 clause | ✅/⛔ SCR-*/FLOW-* | happy-only / happy+recovery / N/A | <BC/VP/test id> | <WSJF score, or RICE/JTBD-opp if elected> | IN/OUT | <notes> |

### Completeness Cross-Check

<CAP → WF(s), covered? table>
<Subsystem → WF(s), covered? table>
<SCR/FLOW → WF(s), covered? table — once screens exist>

**Finding: <zero orphans | N orphans, listed>.**

### Workflows with NO screen (gap table)

| WF-ID | Workflow | Why it's a gap |
```

### Acceptance Criteria (Quality Gate)

- [ ] Every capability maps to ≥1 workflow (zero-orphan finding stated explicitly, not implied).
- [ ] Every subsystem (or equivalent architectural grouping) is represented in ≥1 workflow.
- [ ] Every existing screen/flow maps to ≥1 workflow (once screens exist).
- [ ] Every workflow row has a non-blank Path-coverage cell.
- [ ] Every workflow row has a non-blank Verification cell (or an explicit "no BC yet — flagged" note,
      routed per Stage 8).
- [ ] The capability-coverage matrix was re-diffed against the domain spec's **current** capability
      list this pass — not assumed unchanged.

> **Enhancements-v2 burst (2026-07-07, ux-designer, this pass) — new criteria (Steps 2a–2b, 3a, 5a, 7).**
> - [ ] Every workflow marked `happy-only` was run through the Cockburn extension-condition brainstorm
>       (Step 2a) at least once; any recovery task with its own screen/state was promoted to its own WF
>       row, not left as a note.
> - [ ] Every safety/tenancy-invariant workflow has a lightweight FMEA pass (Step 2b) on file, with its
>       severity/RPN feeding the Storyboard-priority column.
> - [ ] Every complex workflow has an enumerated state machine (Step 3a) including error and recovery
>       states, ready to seed Stage 6's state-set enumeration.
> - [ ] A cross-persona swimlane + RACI discovery pass (Step 5a) has run at least once over the current
>       actor cast; every discovered cross-lane handoff/escalation is either a WF row or explicitly
>       deferred with an owner.
> - [ ] Every WF row has a non-blank Storyboard-priority score (Step 7b), computed by the project's
>       chosen formula (WSJF default), not left blank or "TBD."
> - [ ] `personas/PERSONA-WORKFLOW-MATRIX.md` (the coverage cube, Step 7a) exists and reflects the
>       current inventory — regenerated, not hand-diverged.

> **EXAMPLE (Rivetry) — the silently-missed-capability defect, caught and fixed.** The
> "storyboard-gap closure burst" in `WORKFLOW-INVENTORY.md` found that its own capability-coverage
> matrix had silently stopped at `CAP-022`, missing `CAP-023` (which already had full UX-spec coverage
> from a prior pass) purely because the matrix hadn't been re-run since. It was closed by appending
> `WF-052` and extending the matrix — exactly the audit Step 6 above requires as standing practice, not
> a one-time fix.

---

## Stage 3 — Per-Persona Journey Maps

### Purpose

Walk each persona across the full workflow inventory, in lifecycle-stage order, producing the
emotional/trust narrative and surfacing gaps a bare traceability matrix cannot show — and, for
multi-actor or safety/tenancy-critical workflows, the operational (frontstage/backstage) view a
user-only journey map cannot show either.

### Inputs

- `WORKFLOW-INVENTORY.md` (Stage 2)
- The persona brief (Stage 1)
- UX-INDEX + SCR-*/FLOW-* where they exist
- Design-system responsive/mobile-first constraints, if defined

### Steps

1. **One file per persona** (the "one point of view per journey map" discipline —
   `storyboard-process-research.md` §1).
2. **Persona brief** as §1 (from Stage 1, or a reference to the standalone persona file).
3. **Walk every workflow where this persona is a named actor**, in lifecycle-stage order, one
   subsection per workflow, using the seven-field format: **Intent · Screen/Flow (or ⛔ NO-SCREEN) ·
   Agentic touchpoint (or "N/A" stated, not omitted) · Decisions/branches · Trust/emotional beat ·
   Mobile posture (or device-context posture, project-appropriate) · Handoffs.**

   > **Enhancements-v2 burst (2026-07-07, ux-designer, this pass) — Agentic touchpoint field, rubric
   > -checked.** For any workflow whose Agentic-touchpoint field is non-N/A, add one line per **A1–A4**
   > dimension of `AGENTIC-UX-RUBRIC.md` (defined immediately below Stage 3), stating how that dimension
   > is — or is deliberately not — served at this beat; a deliberately-omitted dimension is a stated
   > decision, never a blank. This makes the field load-bearing rather than a bare presence/absence
   > marker, surfacing agent-UX gaps in the journey layer where they are cheapest to catch — exactly as
   > the emotion curve (Step 5) already surfaces trust valleys. This refines the field; it does not
   > replace it (`.factory/analysis/storyboard-research-v2-hifi-agentic.md` Angle D, D2).
4. **Add the scenario "expectations" line** at the top of each lifecycle-stage section (Enhancement,
   `storyboard-process-research.md` §1) — what does this persona expect going into this stage
   specifically, and how does that shape the emotion curve below.
5. **Add an emotion curve** — a compact table or ASCII sparkline plotting the Trust/emotional beat
   across every stage at a glance (Enhancement, `storyboard-process-research.md` §1, §9.1) — this
   surfaces peaks/valleys the per-workflow prose alone doesn't make visible.
6. **Add a service-blueprint layer** for any workflow with **≥2 actors or a safety/tenancy invariant**
   (Enhancement #3, `storyboard-process-research.md` §1, §9.3): frontstage (what the actor(s) see) /
   backstage (the operational machinery — queues, workers, external subprocess calls, isolation
   resets) / line of visibility / support processes — one blueprint per qualifying workflow, not one
   per persona file.
7. **Close with a Gaps section** — every gap surfaced in the walk, each with **explicit ownership**
   (which artifact/specialist would close it) — never left as an unrouted observation (this composes
   directly with Stage 8).

### Output Artifact(s) + Path Convention

- `storyboard/<version>/journeys/journey-<persona-code>.md`

### Template

```markdown
# End-to-End Journey Map — <Persona Name> (<CODE>)

## 1. Persona Brief
<Stage 1 template>

## 2. End-to-End Journey Across the N Lifecycle Stages

### Stage <k> — <name>
> **Expectations:** <what this persona expects entering this stage>

#### WF-NNN — <name>
- **Intent:**
- **Screen/Flow:**
- **Agentic touchpoint:**
- **Decisions/branches:**
- **Trust/emotional beat:**
- **Mobile posture:**
- **Handoffs:**

[optional, if ≥2 actors or a safety/tenancy invariant]
> **Service blueprint — WF-NNN**
> | Layer | Content |
> |---|---|
> | Frontstage | <what the actor(s) directly see/do> |
> | Backstage | <the operational machinery behind it> |
> | Line of visibility | <what marks the boundary> |
> | Support processes | <anything else that must succeed for this to work> |

## 3. Emotion Curve

| Stage | 1 | 2 | 3 | 4 | 5 | 6 |
|---|---|---|---|---|---|---|
| Beat | <low/neutral/high or short phrase> | ... |

## 4. Gaps

| Gap | Surfaced at | Owner/route |
```

### Acceptance Criteria (Quality Gate)

- [ ] One file per persona; no journey file covers two personas' points of view.
- [ ] Every workflow this persona is a named actor for (per Stage 2's Persona(s) column) appears,
      in the correct lifecycle-stage section.
- [ ] Every workflow subsection has all seven fields, none blank/omitted.
- [ ] An emotion curve exists and is legible at a glance (not buried only in prose).
- [ ] Every workflow meeting the ≥2-actor-or-safety/tenancy-invariant test has a service-blueprint
      subsection.
- [ ] Every gap in the Gaps section carries an explicit owner/route.

> **Enhancements-v2 burst (2026-07-07, ux-designer, this pass) — new criterion.**
> - [ ] Every non-N/A Agentic-touchpoint beat carries an A1–A4 note (per `AGENTIC-UX-RUBRIC.md`, below);
>       a deliberately-omitted dimension is stated, not blank.

> **Fix-burst 45A (2026-07-19, ux-designer, ADV-J3-NDP-023 / ADV-J3-INF-014) — new criterion.**
> - [ ] Every WF subsection heading matches the §3 Workflow cell for its WF-ID and cites that row's
>       Req. Anchor; every EN-FMEA-N citation matches the §4 row's workflow, S/O/D/RPN, and tier —
>       mechanically verifiable.

> **Fix-burst 51 (2026-07-19, ux-designer, ADV-S3P7-001 — lineage: ADV-S3P1-002 → ADV-S3P7-001,
> second occurrence) — new criterion.**
> - [ ] Any requirements amendment propagation pass (a fix-burst that updates requirements,
>       WORKFLOW-INVENTORY, or journey files to reflect a new v2.x amendment) MUST include the
>       session persona briefs (`personas/PERSONA-SYS-NDP.md`, `personas/PERSONA-SYS-INF.md`, and
>       any future session-scoped persona brief) in its touched-artifact population. Persona brief
>       decision criteria are a separate implementer-facing site from journey maps and must receive
>       the same amendment — a stale criterion list is a load-bearing defect (an implementer
>       consuming the decision criteria list in isolation would build the superseded behavior).

> **Fix-burst 52 (2026-07-19, ux-designer, ADV-S3P8-001/-004 — lineage: ADV-S3P1-002 →
> ADV-S3P7-001 → ADV-S3P8-001/-004, third occurrence; new journey-originated vector) —
> companion criterion widening the fix-burst-51 scope.**
> - [ ] Any fix-burst that introduces or amends NORMATIVE MECHANICS in ANY artifact the session
>       persona briefs mirror — not only requirements (v2.x amendments), but also
>       WORKFLOW-INVENTORY (§3 workflow cells, §4 FMEA, §8 state machines) and JOURNEY FILES
>       (per-workflow Intent, Agentic A1–A4, Decisions/branches, blueprints) — MUST include the
>       session persona briefs in its touched-artifact population and reconcile the persona briefs'
>       decision criteria with the amended mechanics, OR record an explicit n/a-with-reason for
>       each criterion that was evaluated and found not to be impacted.
>       Rationale: ADV-S3P8-001 demonstrated that a normative gate (`error_count == 0`) first
>       stated in a journey file (journey-sys-inf.md WF-EN-002 A4) was never reconciled into the
>       persona brief's decision criteria, producing a weaker gate that would misdirect
>       implementers. The fix-burst-51 criterion's "requirements amendment propagation pass" trigger
>       did not cover this vector because the mechanic originated in the journey, not in a v2.x
>       requirements amendment. This companion criterion closes that gap.
>       Lineage cited for audit: ADV-S3P1-002 (first occurrence — OI-1/OI-3 resolution propagation
>       missed persona briefs) → ADV-S3P7-001 (second occurrence — v2.5 PROD-2/PROD-3 amendment
>       missed SYS-NDP persona brief; fix-burst 51 criterion added) → ADV-S3P8-001/-004 (third
>       occurrence, new journey-originated vector; this companion criterion added).

> **Fix-burst 54A (2026-07-19, ux-designer, ADV-S3P10-002 — lineage: ADV-S3P1-002 →
> ADV-S3P7-001 → ADV-S3P8-001 → ADV-S3P10-001, fourth occurrence; new STORYBOARD-INDEX
> roster/rationale-cell vector) — companion criterion widening the fix-burst-52 scope.**
> - [ ] Any fix-burst that introduces or amends NORMATIVE MECHANICS in any artifact that the
>       STORYBOARD-INDEX "Frame scope rationale" section cells restate — including requirements
>       (v2.x amendments), WORKFLOW-INVENTORY (§3 workflow mechanics, §8 state machines), and
>       JOURNEY FILES (per-workflow Intent, Decisions/branches, corrected mechanism annotations)
>       — MUST include the STORYBOARD-INDEX "Frame scope rationale" cells (and any other
>       STORYBOARD-INDEX prose cells that restate normative mechanics: frame-roster state lists,
>       frame-03 PROD-4 self-sufficiency constraint, frame-04 arrival mechanism) in its
>       touched-artifact sweep population and reconcile those cells with the amended mechanics,
>       OR record an explicit n/a-with-reason for each cell evaluated and found not to be impacted.
>       Rationale: ADV-S3P10-001 demonstrated that the v2.2 mechanism correction (ADV-J3-CBU-001)
>       — closed in fix-bursts 45B/45C across requirements PROD-6, journey-cbu WF-EN-014,
>       WORKFLOW-INVENTORY §3 WF-EN-014 row, and §8 WF-EN-014 state machine — did not reach the
>       STORYBOARD-INDEX frame-04 rationale cell, which restates the same normative arrival
>       mechanism as prose. The fix-burst-52 companion criterion's declared sweep population
>       (requirements, WORKFLOW-INVENTORY, journey files) did not include STORYBOARD-INDEX
>       roster/rationale cells; ADV-S3P10-001 is the exact residual this gap produces.
>       Alternative considered: demote STORYBOARD-INDEX roster/rationale cells to pure pointers
>       (ADV-EN6-001 pattern, applied to persona-table cells in fix-burst 6). That pattern was
>       not applied to the Frame scope rationale section and roster/rationale cells provide
>       navigational value; the population widening is the production-grade fix now.
>       Lineage cited for audit: ADV-S3P1-002 (first occurrence — OI-1/OI-3 resolution
>       propagation missed persona briefs) → ADV-S3P7-001 (second occurrence — v2.5 PROD-2/
>       PROD-3 amendment missed SYS-NDP persona brief; fix-burst 51 criterion added) →
>       ADV-S3P8-001/-004 (third occurrence, new journey-originated vector; fix-burst 52
>       companion added) → ADV-S3P10-001 (fourth occurrence, STORYBOARD-INDEX roster/
>       rationale-cell vector; this companion added).

> **Fix-burst 56 (2026-07-20, ux-designer, ADV-S3P12-002/-003 — lineage: ADV-S3P1-002 →
> ADV-S3P7-001 → ADV-S3P8-001/-004 → ADV-S3P10-001/-002 → ADV-S3P12-002/-003, fifth
> occurrence; new journey Gaps-table route-cell vector) — companion criterion widening the
> fix-burst-54A scope.**
> - [ ] Any fix-burst that introduces or amends NORMATIVE MECHANICS in any artifact that
>       journey §4 Gaps-table route/owner cells restate — including requirements (v2.x
>       amendments), WORKFLOW-INVENTORY (§3 workflow mechanics, §8 state machines), journey
>       files (per-workflow Intent, Decisions/branches, corrected mechanism annotations), and
>       persona briefs (decision-criteria lists) — MUST include journey §4 Gaps-table
>       route/owner cells that restate those mechanics (BC-scope enumerations, gate definitions,
>       mechanism descriptions) in its touched-artifact sweep population and reconcile those
>       cells with the amended mechanics, OR record an explicit n/a-with-reason for each cell
>       evaluated and found not to be impacted.
>       Rationale: ADV-S3P12-002 demonstrated that the v2.6 startup corpus-seed pass
>       (ADV-S3P11-001, fix-burst 55) — propagated across requirements PROD-2/PROD-3,
>       journey-sys-ndp WF-EN-015 blueprint/Support processes/A4, and the SYS-NDP persona brief
>       — did not reach the journey-sys-ndp §4 Gaps-table WF-EN-015 route cell, which restates
>       the WF-EN-015 BC scope as a BC authoring guide. The fix-burst-54A companion criterion's
>       declared sweep population (requirements, WORKFLOW-INVENTORY, journey files,
>       STORYBOARD-INDEX roster/rationale cells) did not include journey Gaps-table route/owner
>       cells; ADV-S3P12-002 is the exact residual this gap produces.
>       Lineage cited for audit: ADV-S3P1-002 (first occurrence — OI-1/OI-3 resolution
>       propagation missed persona briefs) → ADV-S3P7-001 (second occurrence — v2.5 PROD-2/
>       PROD-3 amendment missed SYS-NDP persona brief; fix-burst 51 criterion added) →
>       ADV-S3P8-001/-004 (third occurrence, new journey-originated vector; fix-burst 52
>       companion added) → ADV-S3P10-001/-002 (fourth occurrence, STORYBOARD-INDEX roster/
>       rationale-cell vector; fix-burst 54A companion added) → ADV-S3P12-002/-003 (fifth
>       occurrence, journey Gaps-table route-cell vector; this companion added).

> **Fix-burst 57 (2026-07-20, ux-designer, ADV-S3P13-004 — lineage: ADV-S3P1-002 →
> ADV-S3P7-001 → ADV-S3P8-001/-004 → ADV-S3P10-001/-002 → ADV-S3P12-002/-003 →
> ADV-S3P13-004, sixth occurrence; new interaction-vector class — first instance) —
> companion criterion requiring interaction analysis for new startup/lifecycle mechanics.**
> - [ ] Any fix-burst that introduces or amends a NEW STARTUP OR LIFECYCLE MECHANIC — including
>       conditional triggers, stream-position capture, resume-token handling, or other startup
>       ordering changes in the notification producer (WF-EN-015) or any subscriber workflow —
>       MUST include an explicit derivation of how that mechanic interacts with EXISTING behaviors:
>       stream-resume-token replay, per-event silent PROD-2 maintenance (criterion 1), and
>       subscriber-registration fan-out behavior. The derivation MUST be recorded in the fix-burst
>       narrative (requirements amendment entry, inventory §3 Notes, journey blueprint, or persona
>       brief) as a separate interaction-analysis sentence — not merely propagated prose from the
>       requirements. An explicit n/a-with-reason is acceptable when the interaction is genuinely
>       orthogonal (e.g. the mechanic operates on a different code path with no shared state).
>       Rationale: ADV-S3P13-001 demonstrated that the v2.6 unconditional startup corpus-seed
>       pass — propagated soundly across all five sweep-population classes (requirements,
>       inventory, journey, persona brief, Gaps-table route cells) — nonetheless introduced two
>       regression vectors (RESTART-SWALLOW, SEED-WINDOW EVENT LOSS) whose root cause was
>       failure to analyze the new pass mechanic's interaction with the EXISTING resume-token
>       replay path. The v2.6 propagation passes were mechanically correct for the stated v2.6
>       mechanic but did not derive whether that mechanic was safe in the context of existing
>       stream-resume behavior. This criterion closes that gap: interaction analysis is a
>       REQUIRED artifact for any startup/lifecycle mechanic amendment, not just propagation.
>       Lineage cited for audit: ADV-S3P1-002 (first occurrence — OI-1/OI-3 resolution
>       propagation missed persona briefs) → ADV-S3P7-001 (second occurrence — v2.5 PROD-2/
>       PROD-3 amendment missed SYS-NDP persona brief; fix-burst 51 criterion added) →
>       ADV-S3P8-001/-004 (third occurrence, new journey-originated vector; fix-burst 52
>       companion added) → ADV-S3P10-001/-002 (fourth occurrence, STORYBOARD-INDEX roster/
>       rationale-cell vector; fix-burst 54A companion added) → ADV-S3P12-002/-003 (fifth
>       occurrence, journey Gaps-table route-cell vector; fix-burst 56 companion added) →
>       ADV-S3P13-001/-004 (sixth occurrence, first interaction-vector instance; this criterion
>       added).

> **Fix-burst 67 (2026-07-21, ux-designer, ADV-S3P23-001 — S3-23-001 class) — companion
> criterion requiring target-state composition check before ratifying source-verified design rules.**
> - [ ] **TARGET-STATE COMPOSITION CHECK:** Any design rule justified by "source-verified current-code
>       behavior" MUST ALSO be validated against the code changes that the same requirements package
>       mandates. A rule that is source-true pre-package but becomes false under the package's own
>       target-state changes (e.g., DD-8 r2 `is_public=True` hardcode: source-true because jirasm
>       filters to jsdPublic=True in current code; false under PROD-5 C4 item 1 which removes that
>       filter) is not valid for inclusion in requirements ratified together with that package. Before
>       ratifying any design rule citing source-verified behavior: (a) identify every target-state
>       change in the same requirements package that touches the cited code path; (b) verify the rule
>       holds under the post-change target state; (c) if not, correct the rule to the target-state-
>       coherent form before ratification. Explicit n/a-with-reason is acceptable only when the
>       requirements package contains no target-state changes that touch the cited code path.
>       Rationale: ADV-S3P23-001 demonstrated that DD-8 r2 sourced `is_public=True` as a hardcoded
>       constant, justified by source-verified jirasm jsdPublic=True filter behavior — a valid
>       source-truth claim for current code. However, the SAME requirements package also mandates
>       PROD-5 C4 item 1 (remove the jsdPublic=True filter to allow internal comments). Under that
>       target state, jsdPublic=False comments reach jirasm and `is_public=True` hardcode is incorrect.
>       The r2 rule was source-coherent but target-state-incoherent; r3 corrects it to source
>       `is_public` from the echo's jsdPublic-derived value. This criterion prevents recurrence.
>

> - [ ] **Fix-burst 68 (ADV-S3P24-OBS2, 2026-07-22) — LEXICAL VARIANTS criterion:** When a numeric class-kill sweep is declared (e.g., "five+ write paths" or "5+ write paths"), the sweep MUST search ALL lexical variants of the target expression — digit form ("5+"), word form ("five"), symbol vs. no-symbol ("five+", "five plus", "five+ write"), and plural variants — to catch all instances regardless of how the value was written across different authors and fix-bursts. Failure to sweep all variants leaves live instances uncorrected and breaks the "class-kill" guarantee. Correct procedure: (1) declare the canonical target value; (2) enumerate all lexical variants; (3) run a separate grep per variant; (4) report N and F/P per variant in the sweep section of the fix report. [ADV-S3P24-OBS2, fix-burst 68, 2026-07-22; promoted to own checkpoint [ADV-S3P25-011, fix-burst 69, 2026-07-22]]

> - [ ] **Fix-burst 69 (ADV-S3P25-OBS3, 2026-07-22) — INVERSE-VECTOR criterion:** When a finding scopes an existing endpoint to a NEW trigger surface (e.g., `PATCH .../comments/{id}/visibility` for `is_public` writes, where `update_comment` was the prior surface), treat the new trigger surface as a fresh vector for all quality checks already applied to the original: EN-FMEA coverage, gate sub-tests, ACC-2 rows, and UX affordance. Do NOT rely on original trigger-surface coverage to subsume the new one. Correct procedure: (1) identify the original endpoint + its existing coverage; (2) for the new trigger surface, enumerate which checks are reused vs. require a new entry (FMEA row, gate sub-test, ACC-2 scenario, UX affordance); (3) either extend the existing checks or record n/a-with-reason for each. **CARRIER-DOCUMENT WRITERS clause (extended [ADV-S3P26-OBS, fix-burst 70, 2026-07-22]):** The trigger-surface sweep unit is every writer of the anchored document or subdocument (grep the full collection's `$set`/`$push`/replace population), not just the named endpoint. Step (4): for each new trigger surface, grep ALL write-path functions that touch the same MongoDB document or subdocument collection — even functions that are not endpoint-type-specific. Example: `update_ticket` was missed in a C9 write-path sweep (ADV-S3P26-002 finding class) because it is not a comment endpoint, but it writes the ticket document that contains the `comments` subdocument array — making it a carrier-document writer that belongs in the sweep. Consequence: any is_visible recompute check scoped to "comment endpoints" must be broadened to "all writers of the ticket document or comments subdocuments." [ADV-S3P25-OBS3, fix-burst 69, 2026-07-22; CARRIER-DOCUMENT WRITERS appended ADV-S3P26-OBS, fix-burst 70, 2026-07-22]

> - [ ] **Fix-burst 73 (ADV-S3P29-trend-gate-3b, 2026-07-22) — CARRIER-WAR-GAME RECOMPOSITION criterion:** Every war-game verdict (design §6 W*) must carry an annotation naming the DD-revision stack it was last validated against. Whenever ANY DD mechanism is revised, every W-verdict whose correctness logic composes with the revised mechanism MUST be recomposed in the same fix-burst: either verdict HOLDS (append `*[recomposed against rN stack, fix-burst NNN, date: verdict holds. <brief rationale>]*`) or REWRITTEN (add supersession annotation + corrected verdict text). An adversary finding a W-verdict with a revision annotation older than the DD it composes with is a defect. Correct procedure: (1) identify which W-verdicts compose with the revised DD(s) — grep the war-game section (§6) for each revised DD number; (2) for each composing W-verdict, evaluate whether the revision changes the verdict's correctness claim; (3) if unchanged: append the HOLDS annotation; if changed: add supersession annotation + corrected verdict; (4) record all recomposed verdicts in the fix-burst narrative. Explicit n/a-with-reason is acceptable only when the DD revision is to a code path the W-verdict's logic does not traverse. Lineage: ADV-S3P29-002 (W11 held an r2-era dedup-safe verdict five DD revisions stale — DD-3 r5 rewrote the pre-create PATCH path, invalidating W11's correctness claim; W11 was never recomposed against r3/r4/r5/r6/r7/r8). [trend-gate #3 (b), fix-burst 73, 2026-07-22]

> - [ ] **Fix-burst 73 (ADV-S3P29-trend-gate-3b, 2026-07-22) — REVISION-ONLY PIN policy criterion:** Live instructional/normative text MUST cite artifact revisions (e.g., design r9, requirements v2.22, CHANGE-MANIFEST v1.4), NEVER line counts. Line counts are legitimate only in: ledger scalar records, header/currency blocks stating a count alongside the revision in the schematic form `design rN (NNNNL)` (e.g., ~~`design r9 (2543L)`~~ *[struck ADV-S3P30-008, fix-burst 74, 2026-07-22: this criterion's own illustrative example carried a FALSE live scalar — design r9 was 2514L at its ratification commit (Burst 264), not 2543L; the criterion that exists to kill volatile-scalar defects self-spawned one. Superseded by the schematic placeholder `design rN (NNNNL)`, which demonstrates the currency-block form while carrying NO live scalar that can go false or stale]*), and changelog rows. Class-kill sweeps for pin staleness MUST include ALL lexical variants of the line-count pattern: digit-only (e.g., `2293L`), long-form (`"2293 lines"`), parenthetical (`"read in full (2293L)"`), and word-form (`"2293-line"`). Correct procedure: (1) declare the target pin class; (2) enumerate all lexical variants; (3) grep each variant separately; (4) replace stale line-count citations in normative text with the current revision pin; (5) report N-found and F/P per variant in the fix-burst sweep section. Lineage: ADV-S3P28-003 (volatile line-count pin introduced in normative text of a fix-burst); ADV-S3P29-006 (pin-staleness class recurred inside its own class-kill fix — the same fix-burst that added pin-policy text introduced a stale-pinned artifact citation in the new policy wording). [trend-gate #3 (b), fix-burst 73, 2026-07-22]

> - [ ] **Fix-burst 74 (ADV-S3P30-OBS-4, 2026-07-22) — TWIN-BLOCK LINKAGE criterion:** Any block that repeats normative content "for traceability" (a TWIN) MUST carry a reciprocal anchor to its twin(s): each twin names the other twin's section/anchor (anchor or heading names, NEVER line numbers), so the full twin set is discoverable from any member. Superseding, striking, or rewriting EITHER twin FORCES recomposition of ALL its declared twins in the same fix-burst — a fixer's sweep population for ANY supersession MUST include the superseded block's declared twin set. An adversary finding a superseded block whose declared twin still carries the refuted claim is a defect. Correct procedure: (1) when authoring a repeated-for-traceability block, declare the twin linkage reciprocally in ALL members at creation time; (2) when superseding, striking, or rewriting any block, read its twin declarations and enumerate the twin set BEFORE editing; (3) apply the same supersession (strike/rewrite/annotate) to every declared twin in the same fix-burst, or record explicit n/a-with-reason per twin; (4) the fix-burst report MUST enumerate the twin set consulted for every superseded block (or state "no twins declared"). Lineage: ADV-S3P30-001 — the fix-burst 73 W11 verdict rewrite never reached 5 live "repeated-for-traceability" TWIN blocks across two artifacts; the twins carried the refuted claim for a full pass. [ADV-S3P30-OBS-4, fix-burst 74, 2026-07-22]

> - [ ] **Fix-burst 76 (ADV-S3P32-001 pattern, 2026-07-23) — STORYBOARD-TIER LIVE-MIRROR criterion:** Every fix-burst whose normative-tier changes (requirements / design / CHANGE-MANIFEST) alter a COUNT or ENUMERATION — gate condition sets, gate sub-test sets, ACC-2 totals, deploy-step counts, mechanism-set lists — MUST include the storyboard tier's live mirrors in its sweep population; verifying that each individual normative fix landed does NOT discharge this obligation. The known live-mirror carriers are: (a) WORKFLOW-INVENTORY §8 gate condition/sub-test blocks; (b) WORKFLOW-INVENTORY ACC-2 total + derivation chain; (c) WORKFLOW-INVENTORY terminal Stage-3 ratification enumeration; (d) journey/brief BC-scoping rows (gate-range + ACC-2-count citations); (e) journey/brief failure-mode + metric-enumeration carriers; (f) journey RESIDUAL deploy-ordering rows. Correct procedure: (1) at fix-burst close, list every count/enumeration the burst's normative changes altered; (2) for each, grep the storyboard tier (inventory + journeys/ + personas/) for the OLD value and ALL its lexical variants (per the LEXICAL VARIANTS criterion) plus the carrier list above; (3) recompose each live mirror to the new canonical form (history/ledger rows retained struck or dated); (4) the fix-burst report enumerates the mirror population with LIVE-FIXED / HISTORY-RETAINED classification per hit, or states "no storyboard-tier mirrors carry this value". Lineage: ADV-S3P32-001/-003/-004 — the storyboard tier's live count-bearing mirrors (gate (i-a)–(i-o), ACC-2 43, five-step deploy rows, missing (i-q)/`writeback_failure` failure-mode family) froze at the FB69/FB70 state across FIVE normative cycles (v2.19→v2.25, r5→r12, v1.0→v1.7) while every individual normative fix was verified landed — the class survived because no burst's sweep population included the storyboard tier. [ADV-S3P32-001 pattern, fix-burst 76, 2026-07-23]

> - [ ] **Fix-burst 77 (ADV-S3P33-001, 2026-07-23) — TERMINAL PIN-RECONCILIATION criterion:** (1) Every fix-burst that bumps ANY artifact version or changes ANY artifact line count MUST end with ONE terminal reconciliation pass, executed AFTER the last content edit of the last fixer and BEFORE the state-manager commit. (2) The terminal pass greps every live pin — precedence lines, inputs/derived_from blocks, §-Source sentences, Groundwork rows, anchor cells — that cites a version or count changed this burst, and advances stale ones against the FINAL values. (3) Mid-burst pin sweeps do NOT discharge this obligation — they run against pre-bump values by construction. (4) The terminal pass's population and per-hit disposition appear in the burst report. (5) Ordering note: fixers whose artifacts are cited BY others' pins should run before the artifacts carrying those pins where feasible (content-first, pins-last). Lineage: ADV-S3P32-002 → ADV-S3P33-001 — the identical stale-pin class regenerated in two consecutive bursts, each time by the fixing burst's OWN trailing version/count bumps: FB76 fixed the pins S3-32 flagged (v1.6/v2.24-era), then FB76's own terminal bumps (requirements v2.25, design r12, manifest v1.7) re-created the identical staleness because the pin sweep ran before the bumps landed. The class regenerates STRUCTURALLY whenever pin sweeps execute before the same burst's final version/count bumps; only a terminal, post-bump reconciliation pass kills it. [ADV-S3P33-001, fix-burst 77, 2026-07-23] **⚠ EXTENSION [ADV-S3P35-002/-003/-004 lineage, fix-burst 79, 2026-07-23]:** A SCOPED AMENDMENT (post-ratification same-burst change to any artifact) is itself a version/count-changing event: it RE-TRIGGERS the terminal reconciliation obligation over every record, enumeration, and pin written earlier in the same burst that cites the amended artifact's count, revision, test ranges, or mechanism enumerations — including the amending artifact's OWN internal pointers. (Regeneration path this extension kills: FB78's scoped amendment changed the design's line count and test enumeration AFTER the burst's records/enumerations were written — requirements carried the pre-amendment 3001L, and the DD-9 Tests pointer and manifest test row stayed stale — because no terminal reconciliation treated the amendment as a trigger; the FB77 criterion body above keyed the obligation to the burst's version/count bumps, and a same-burst post-ratification amendment slipped between the terminal pass and the commit.) **⚠ REPORT-ONLY BURST RULE (PO-adjudicated, fix-burst 81, 2026-07-23):** report-only bursts do NOT require a PO pin-currency micro-step; requirements live pins advance at the next PO-edited burst; the report-only burst's design status-field ledger row MUST carry an explicit 'requirements pins intentionally stale at rN' disclosure. A terminal pin pass in any PO-edited burst MUST include the requirements artifact's live pins in its sweep population. **⚠ MANIFEST REQUIREMENTS-PINS EXTENSION [ADV-S3P38-003, fix-burst 82, 2026-07-23]:** Any burst that advances the requirements version REQUIRES a terminal ARCHITECT-RESUME — running AFTER the PO's requirements bump and BEFORE the state-manager commit — to advance the CHANGE-MANIFEST's requirements-pins (precedence `inputs.requirements`, `derived_from`, §5b Source) to the final value. Rationale: under content-first-pins-last (ordering note (5) above), the PO bumps requirements AFTER the architect completes the manifest step, creating a window where the manifest's requirements-pins cite the pre-bump version; an explicit ARCHITECT-RESUME is the only mechanism that advances them; the state-manager commit MUST NOT land until that step completes. Lineage: ADV-S3P32-002 → ADV-S3P33-001 → ADV-S3P37-005 → ADV-S3P38-003, 4th recurrence. **⚠ PO SCOPED-RATIFICATION SAME-BURST COUNT-SWEEP [ADV-S3P39-003, fix-burst 83, 2026-07-24]:** When the terminal architect-resume (or ANY post-ratification same-burst event) changes an artifact's line count, the closing PO scoped ratification MUST sweep all same-burst records IN OTHER ARTIFACTS that cite that count (grep the count value + artifact version, plus ALL lexical variants per the LEXICAL VARIANTS criterion) and route corrections to the owning agent. Correct procedure: (1) after the terminal architect-resume completes, identify every line count changed relative to the pre-resume state; (2) grep each changed count (and its lexical variants) across all other artifacts that carry same-burst records; (3) for each hit citing the pre-resume count, route a correction to the owning agent in the same closing step; (4) the closing PO ratification report states "zero cross-artifact count hits" or enumerates each correction routed. Lineage: ADV-S3P35-002 (r13 3001L/3016L — the SCOPED AMENDMENT EXTENSION above catches count-changes WITHIN the same artifact; this ⚠ catches count-citations IN OTHER ARTIFACTS) → ADV-S3P39-003 (v1.13 509L/510L — the terminal architect-resume's own +1-line manifest growth from the lesson-50 annotation invalidated the PO's earlier-written v2.29 record in email-notifications-requirements.md (the v2.29 amendment record) citing "v1.13 (509L)"; the third ⚠ directed the architect-resume but did not instruct the PO to sweep cross-artifact count-citations). [ADV-S3P39-003, fix-burst 83, 2026-07-24]

> - [ ] **Fix-burst 78 (ADV-S3P34-OBS-1, 2026-07-23) — MIRROR-SOURCE CONTRADICTION criterion:** When a mirror-fidelity fixer's own verification rationale contradicts the normative source text it is recomposing to, the fixer MUST NOT recompose-and-proceed. Correct procedure: (1) STOP the recomposition of the affected mirror element; (2) record the contradiction as a routed source defect — to the owning fixer via the orchestrator, in the SAME burst (a mid-burst routing, not a deferral); (3) recompose the mirror ONLY AFTER the source is corrected. A mirror faithfully reproducing a defective source is not fidelity, it is propagation. The fixer's contrary evidence is itself review signal — it must surface as a routed finding against the source, never be silently overridden by the recompose-to-source instruction, and never be silently trusted over the source either (the fixer does not self-adjudicate; the owning fixer corrects the source). The fix-burst report either states "zero mirror-source contradictions encountered" or enumerates each STOP with its routing disposition. Lineage: ADV-S3P33-003 fix → ADV-S3P34-001(b) — the FB77 mirror fixer derived contrary evidence against the source it was recomposing to ("Step E is never retried" for outage-era comments vs. the source's "write-back resumes" recovery text) and recomposed anyway, ratifying the contradiction into the WORKFLOW-INVENTORY (i-p) confirm (d) mirror; S3-34 found the source defect one pass later. [ADV-S3P34-OBS-1, fix-burst 78, 2026-07-23]

> - [ ] **Fix-burst 84 (trend-gate #4, 2026-07-24) — MECHANICAL RECORDS-LINT criterion:** (Adjudication: this is a NEW top-level Stage-3 quality-gate criterion — a SIBLING of TERMINAL PIN-RECONCILIATION, not a fifth ⚠ under it; it introduces a new enforcement mechanism rather than a refinement of an existing one.) (1) The orchestrator MUST run `bash tools/records-lint.sh` BEFORE every state-manager commit in this workstream; exit 1 BLOCKS the commit. (2) Each lint violation routes to its owning agent by check ID: **L1** (H1/frontmatter version mismatch in requirements or manifest) → architect; **L2** (DD-9 Tests-pointer range max inconsistency across design §8 and manifest §1.3) → architect; **L3** (manifest frontmatter `inputs.requirements` version/count or `inputs.design` revision out of sync with actual artifact state) *(extended: also checks `derived_from` design revision pin, `derived_from` requirements version pin, and §5b Source terminal requirements version pin [ADV-S3P41-003, lint hardening, fix-burst 85, 2026-07-24])* — L3 failures mid-burst are EXPECTED until the terminal architect-resume lands the final version pins, but MUST be ZERO at commit time; **L4** (ADV-IDs cited in the latest pending block of the manifest status field are absent from that version's §7 findings column) *(extended: compound slash-form ADV-IDs (e.g. ADV-S3P40-001/-004) expanded iteratively; OBS forms (e.g. ADV-S3P39-OBS-1) supported [ADV-S3P41-004, lint hardening, fix-burst 85, 2026-07-24])* → architect; **L5** (known-bad literals in live text: "design §7" phantom-section anchor, or live `vN.NN (NNNL)` line-count citation whose count does not match actual `wc -l`) *(extended: L5b checks ~~four~~ five citation families [ADV-S3P45A-002/ADV-S3P45B-002, fix-burst 89, 2026-07-25 — S3-44/S3-45 midnight-span] — (a) requirements cites in manifest, (b) manifest self-cites, (c1)–(c3) manifest/design/self cites in requirements file; struck spans stripped; §7 historical rows excluded [ADV-S3P41-002, lint hardening, fix-burst 85, 2026-07-24]; L5a denial-phrase filter [ADV-S3P41-OBS-1, lint hardening, fix-burst 85, 2026-07-24] *(clause split [ADV-S3P42-005, fix-burst 86, 2026-07-24]: denial-phrase filter separated from [ADV-S3P41-002] L5b bundle; L5b now covers: ~~four~~ five citation families [ADV-S3P45A-002/ADV-S3P45B-002, fix-burst 89, 2026-07-25 — S3-44/S3-45 midnight-span] + struck-span strip + §7-row exclusion)*)* → owning agent by artifact (architect for manifest/design; PO for requirements); **L6** (precedence terminal line versions in manifest do not match artifact frontmatter versions) → architect; **L7** (§7 monotonic version order — version entries in the manifest §7 ~~findings column~~ **VERSION column (first column)** are not in ~~monotonically non-decreasing~~ **strictly ascending** order — adjacent equal versions FAIL; duplicates are structural errors) [ADV-S3P43-003, fix-burst 87, 2026-07-24] → architect; **L8** (STORYBOARD-INDEX Stage-Status live-terminal package scalars — any live-terminal package scalar in the STORYBOARD-INDEX Stage-Status block does not match the corresponding artifact frontmatter version; legitimately FAILS mid-burst until the state-manager currency append; must be ZERO at commit) → state-manager [ADV-S3P42-001/-003 lint extension, fix-burst 86, 2026-07-24]. **L8b** (Stage-Status currency parity: on the LIVE (struck-stripped) Stage-3 row, the live-text stripper is nesting-blind (the `sed '~~[^~]*~~'` removal mis-pairs on nested strikes); therefore strike spans MUST NOT nest and MUST NOT contain inner '~~' characters; requires (a) exactly ONE '[state-manager currency, Burst NNN...]' tag surviving strike-stripping — VIOLATION message when count ≠ 1: "predecessor blocks not struck: pre-convention historical entries and/or chimera append-in-place"; (b) when exactly one tag survives, its burst number must equal the current burst derived from the highest 'BURST NNN DONE' marker in STATE.md; (c) at most one distinct 'NEXT = S3-NN' token in the live text (self-contradiction guard); SEGMENTED-STRIKE REPAIR CONTRACT: when conformance repair must strike a block that already contains inner struck spans, use SEGMENTED strikes — strike each unstruck segment around the existing inner struck spans separately so that every '~~...~~' span is tilde-free inside; never nest a strike inside another strike) → state-manager [S3-59 AE4, fix-burst 103, 2026-07-27]. **L8c** (header Status chain NEXT-token consistency: extracts the single blockquote line matching '^>.*NEXT = S3-' in STORYBOARD-INDEX [anchor verified unique on the live corpus — table/ledger rows start with '|' not '>'], strike-strips it with the same nesting-blind stripper [segmented-strike contract applies to this surface too], ~~counts distinct live 'NEXT = S3-NN' tokens, and fires a VIOLATION when the count exceeds 1~~ counts distinct live NEXT targets under the extended pattern 'NEXT = (pass |dispatch )?S3-NN', each match normalized to its S3-NN core before the distinct-count (strict + variant same-target = ONE), and fires a VIOLATION when the count exceeds 1; residuals: colon-form 'NEXT: S3-NN' not covered, and the '^>.*NEXT = S3-' anchor misses a line whose raw NEXT tokens are all variant-form [S3-64 AJ3, fix-burst 108, 2026-07-27] — the stale-NEXT/self-contradiction class extended from the Stage-3 row to the header Status chain, which is the codified 4th live mirror; all extraction pipelines || true-guarded, counts single-emission) → state-manager [orchestrator catch, micro-burst, Burst 301, 2026-07-27]. **Check L8d — cross-surface NEXT-parity:** The STORYBOARD-INDEX carries TWO independent NEXT pointers — the header Status chain and the Stage-Status Stage-3 row — and until this check existed nothing compared them. Burst 308 advanced the Stage-3 row to the next pass while omitting the header-chain carrier advance entirely, leaving a stale live pointer naming the previous pass; check L8c did not fire because it only detects more than one live token on the anchor line itself, and check L8b inspects only the Stage-3 row, so the lint reported a full pass. Check L8d fires when the live NEXT pointer on the header chain disagrees with the live NEXT pointer on the Stage-Status row, and the violation names both values. Liveness is determined strike-aware, so struck predecessors are ignored. Diagnosis: identify which of the two surfaces was not advanced. Fix: strike the stale pointer on whichever surface was not advanced and set it to match the other surface — check L8d fires symmetrically, so the header chain is not always the stale side; when the Stage-Status Stage-3 row is the stale side, the remedy is a strike-and-replace of its NEXT token, NOT appending a chain entry. [S3-69 UL4, fix-burst 113, 2026-07-28] Routing owner: **state-manager** — both surfaces are record surfaces. Note this check LEGITIMATELY FAILS mid-burst, between the point where one surface is advanced and the point where the records step advances the other. [S3-68 UL1, fix-burst 112, 2026-07-28] **L9** (line-position cites in newly-authored record text — the RECORD LINE-CITE BAN's mechanical guard) → authoring agent. **L10** (stale design-revision pin tokens: scans ~~requirements §Existing Groundwork section and journeys/*.md~~ *(superseded [ADV-S3P49B-002+ADV-S3P49B-OBS-2, fix-burst 93; documented fix-burst 94 per ADV-S3P50A-003=ADV-S3P50B-005]: 2-tier population expanded to 4 tiers)* requirements §Existing Groundwork section, journeys/*.md, personas/*.md, and WORKFLOW-INVENTORY.md (four tiers) against the design frontmatter revision; struck spans stripped; journey annotation spans deliberately NOT stripped; fail-closed [heading absent = VIOLATION; journeys dir absent = SKIP with stderr notice; personas dir absent = SKIP with stderr notice; WORKFLOW-INVENTORY.md absent = SKIP with stderr notice]; provenance: ADD-CHECK-FIRST [A-OBS-1, fix-burst 90, 2026-07-25]) → routing: requirements §Existing Groundwork hits → product-owner; journey hits → ux-designer; persona hits → ux-designer; inventory hits → ux-designer; script defects → architect [ADV-S3P47A-002/ADV-S3P47B-001, fix-burst 91, 2026-07-25; population expanded ADV-S3P49B-002+ADV-S3P49B-OBS-2, fix-burst 93; documented fix-burst 94 per ADV-S3P50A-003=ADV-S3P50B-005]. **L11** (ratification-block parity: every manifest version asserted RATIFIED in the STORYBOARD-INDEX Stage-3 live terminal (struck spans stripped from Stage-3 raw) must appear in a co-occurring block in the manifest status field (struck spans also stripped from status) — AND-conjunct: both 'PO RATIFICATION COMPLETE' and 'CHANGE-MANIFEST vX.YY RATIFIED' must co-occur within the SAME bracket-bounded block (bracket-depth scan to block boundary) in the stripped status; struck version markers FAIL; quoted markers without a co-occurring 'PO RATIFICATION COMPLETE' FAIL; absent STORYBOARD-INDEX is a VIOLATION; fail-closed: zero version tokens extracted → VIOLATION; LINT-L11: N version token(s) checked emitted to stderr) → routing: status-field block omissions → product-owner; script defects → architect [W3=ADV-S3P51A-004=ADV-S3P51B-003, fix-burst 95, 2026-07-26; ADV-S3P50A-OBS-1, fix-burst 94, 2026-07-25; X6=S3-52, fix-burst 96, 2026-07-26: (a) fail-closed: zero version tokens extracted → VIOLATION; (b) fixed 1000-char window replaced with bracket-depth block-boundary scan]. **L11b** (design ratification-block parity: every design rNN asserted RATIFIED in the STORYBOARD-INDEX Stage-3 live terminal (struck spans stripped) must co-occur with a 'PO RATIFICATION COMPLETE' block (bracket-depth bounded, same block-boundary scan as L11) that co-contains rNN as a space-bounded token in the design's own status field (struck spans stripped); fail-closed: absent STORYBOARD-INDEX → VIOLATION; zero rNN RATIFIED tokens extracted → VIOLATION; LINT-L11b: N design revision token(s) checked emitted to stderr; founding omission class: FB92 design r22 ratification-block absent from design status field; ADD-CHECK-FIRST [X9=S3-52, fix-burst 96, 2026-07-26]: PASSES on r26 today; WOULD HAVE FIRED on FB92 r22 omission) → routing: design status field omissions → architect; ratification block omissions → PO; script defects → architect [X9=S3-52, fix-burst 96, 2026-07-26]. **L12** (cascade-ledger row-shape parity — every STORYBOARD-INDEX Adversarial Cascade Ledger data row ~~(prefix '| S3-')~~ *(widened [AB1=ADV-S3P56A-001=ADV-S3P56B-001, fix-burst 100, 2026-07-26]: population is now prefix-agnostic — awk from the '| Pass |' header through the contiguous '|'-prefixed block, separators skipped; ~~truncation guard: every file-wide '| S3-' row must fall inside the bounded region — table splits/blank-line breaks → VIOLATION, fail-closed~~ ~~truncation guard checks that the file-wide data-row count after the | Pass | header (any "| "-prefixed line, separator rows excluded) equals the bounded row count; any excess indicates a table split in any tier (not only S3-); precondition: all other tables precede the ledger header [AC1=ADV-S3P57A-001=ADV-S3P57B-002, fix-burst 101, 2026-07-26]~~ The file-wide pipe-initial (^|) row count after the | Pass | header must equal the bounded row count; any excess indicates a table split or orphaned rows outside the contiguous block; any deficit indicates a predicate/masking anomaly (no-space row inflating bounded count). Both conditions fire as violations. [AD1=ADV-S3P58A-001=ADV-S3P58B-001, fix-burst 102, 2026-07-26])* must have exactly the header's 6 cells (Pass | Stage | Date | Agent | Findings | Streak); cell count = pipe chars after GFM-escaped-pipe (\|) substitution − 1; struck spans NOT stripped (struck pipes are structural); unescaped raw '|' inside cell prose = violation (escape as \| per the S3-52 X5 precedent); fail-closed: absent index → VIOLATION, zero rows extracted → VIOLATION; 'LINT-L12: N ledger row(s) checked' to stderr; founding incident: FB98 row-insertion splice of the S3-fix-97 row (+11 pre-existing malformed rows repaired same-burst); routing: ledger-row shape defects → state-manager (never ux); script defects → architect. [AA2=ADV-S3P55A-002=ADV-S3P55B-002, fix-burst 99, 2026-07-26; AB1=ADV-S3P56A-001=ADV-S3P56B-001, fix-burst 100, 2026-07-26] **L13** (STATE.md structural invariants: (1) '## Decisions Log' heading must occur line-initial (fusion class: archive rotation removing the separating newline causes the heading to be fused into a preceding table row's tail); fails-noisy when the string appears only mid-line or is absent entirely; (2) every '| BURST NNN DONE' row in the Current Phase Steps table must end with '|' (table-row fence invariant; secondary detection of the same fusion class); absent STATE.md: silent skip; all pipelines || true-guarded, counts single-emission) → routing: STATE.md structural repairs (heading line-initiality, BURST-row fencing) → state-manager; defects in check script → architect [S3-64 AJ2, fix-burst 108, 2026-07-27]. **L14** (cell-count / row-shape parity (escape-aware, header-derived) for three scan populations: (a) STATE.md '| BURST NNN DONE' rows vs '| Step | Agent | Status | Output |' 4-column header — each BURST row must have exactly 4 cells; (b) burst-log.md '| BURST NNN DONE' rows — expected count from the first '| Step |' header in the file; (c) lessons.md '| Lesson | Proposed Policy | Scope | Status |' policy-table data rows — expected count derived from each table's own header row; escape-aware: '\|' substituted to ESCPIPE placeholder before pipe counting (ESCPIPE idiom, shared with check L12); cell count = (pipe chars after substitution) − 1; absent file → SKIP with stderr notice, no violation; absent '| Step |' header (populations a, b) → SKIP with stderr notice; the header-absent skip is SCOPED to the cell-count and header-pipe-count predicates only — the five-row population-count invariant was hoisted out of the header-present branch and now runs unconditionally whenever STATE.md is present, because it depends solely on a ~~'^| BURST '~~ '^| BURST [0-9][0-9]* DONE' (DONE-qualified; closes the masking hole where a non-DONE BURST row could substitute for a deleted DONE row at count five; check L13 fence-check and check L14(a) cell-check row iterations retain the unqualified '^| BURST [0-9]' form — do not conflate [S3-70 UL12, fix-burst 114, 2026-07-30]) row population and has no '| Step |' header dependency, so a damaged window is still caught with the header missing and the verdict note reads "BURST row cell-count did not run; population-count invariant ran independently" [S3-69 UL5, fix-burst 113, 2026-07-28]; zero data rows: 0 checked, no violation; per-population row counts emitted to stderr; ADD-CHECK-FIRST fires on 11 pre-fix violations — all routed to state-manager Step 3) → routing: row-shape/cell-count repairs → state-manager; script defects → architect [S3-65 AK2 + AK4, fix-burst 109, 2026-07-27]. **Check L14 sub-check AL4 — column-zero cross-check:** ~~Detects a BURST table row that has lost its leading pipe. Such a row exits the scan populations of check L13, check L14(a) and check L14(b) entirely (all three select on a line-initial pipe), so the damage would otherwise be invisible to every check. Two distinct predicates, one per file; both fire on a SINGLE damaged row.~~ Detects a BURST table row displaced from the line-initial pipe form, in either of two shapes: column-zero (the leading pipe and its following space both removed) or leading-space (the pipe removed, the following space retained). Such a row exits the scan populations of check L13, check L14(a) and check L14(b) entirely, because all three select on a line-initial pipe. Because a row can also leave those populations by outright deletion or by window-size drift, check L14 additionally asserts the STATE.md five-row Current Phase Steps window count. That assertion is an inequality against the expected window size, so it fires whenever the population moves away from five in EITHER direction — shrink from damage or deletion, and growth from window-size drift. All predicates fire on a SINGLE damaged row. [S3-67 UL1, fix-burst 111, 2026-07-27] *STATE.md — threshold-zero, no allow-list:* any line-start occurrence of 'BURST' followed by digits followed by ' DONE' (with no leading pipe) fires a LINT-L14 violation immediately; expected baseline is zero such lines; violation names the offending burst number(s). Diagnosis: find the line in STATE.md beginning with 'BURST' and a burst number rather than a pipe. Fix: restore the leading pipe so the row begins with '| BURST ' and its number. Routing owner: **state-manager** (STATE.md and all record surfaces are state-manager-owned). *burst-log.md — allow-list of four intentional plaintext archive entries (bursts 159, 160, 162 and 166):* any line-start occurrence whose burst number is NOT one of those four fires a LINT-L14 violation immediately; violation names the offending burst number(s). Diagnosis: for each named burst number, determine which of two cases applies — (a) a damaged table row, or (b) a new legitimate plaintext archive entry. Fix, case (a): restore the leading pipe. Fix, case (b): the burst number ~~MUST be added to the allow-list in the check L14 burst-log branch of 'tools/records-lint.sh'~~ MUST be added to the _AL4_BLOG_ALLOWLIST array in the check L14 burst-log branch of 'tools/records-lint.sh' — that array is the single edit site, and the membership test, the grep pattern and the expected occurrence count are all derived from it at runtime [S3-67 UL5 + DE3, fix-burst 111, 2026-07-27], or every subsequent lint run fires a false violation. Routing owners: **state-manager** for damaged rows; **architect** for extending the allow-list (the allow-list lives in tool code, and tool code is architect-owned — do NOT route allow-list edits to spec-steward). Separately from the not-in-allow-list predicate, check L14 also asserts that the TOTAL number of allow-listed line-start occurrences does not exceed the size of the _AL4_BLOG_ALLOWLIST array — one per allow-listed burst. A count above that size fires an "allow-list count anomaly" violation, which means either a duplicate plaintext archive entry or a damaged BURST table row whose number happens to match an allow-listed burst. An allow-listed burst number is therefore NOT automatically safe. Diagnosis: identify which allow-listed number appears more than once, then determine whether it is a duplicate archive entry or a damaged row; the routing owners stated above apply unchanged. [S3-68 UL5, fix-burst 112, 2026-07-28] *Maintenance obligation to state plainly:* the allow-list is a static hand-maintained set; it is the only ongoing manual maintenance obligation introduced by this check, and it must be extended whenever a new legitimate plaintext archive entry is added to burst-log.md in that format [fix-burst 110, AL4, 2026-07-27]. A grep hard failure on STATE.md or on burst-log.md (for example an unreadable file, grep exit code 2 or above) now fires a LINT-L14 violation stating that the check did not run, rather than treating an empty count as clean; diagnosis is to verify file permissions. [S3-67 UL6, fix-burst 111, 2026-07-27] ~~Fail-closed behaviour is per-call, not uniform across the sub-check: the STATE.md column-zero predicate and the burst-log.md total-count predicate fire a LINT-L14 violation when their grep exits 2 or above, whereas the STATE.md leading-space predicate and the burst-log.md allow-list count-sensitivity predicate silently skip on the same condition. One hard-failure violation per file is sufficient to flag that file as unreadable, so the silent skips cannot produce a false clean for a file whose first predicate already fired.~~ All five predicates in the sub-check now fail closed with explicit hard-failure violations; the STATE.md leading-space predicate and the burst-log.md allow-list count-sensitivity predicate each gained an explicit hard-failure else branch (exit code ≥ 2 → independent LINT-L14 hard-failure violation) — no predicate silently skips on grep exit ≥ 2. When STATE.md is unreadable, three independent LINT-L14 hard-failure violations fire: population-count, column-zero cross-check, and leading-space cross-check. Unreadability is reported per-predicate; the silent-skip rationale is obsolete. [S3-70 UL2, fix-burst 114, 2026-07-30] The STATE.md population-count invariant is a fifth predicate in the sub-check, now running unconditionally (hoisted from the header-present branch in this burst); ~~on grep exit 2 or above its || echo 0 fallback sets the count to zero, and zero ≠ five fires a violation with a population-mismatch message rather than an explicit hard-failure message — a file-unreadability incident and genuine row damage produce indistinguishable violation text on this predicate; the column-zero predicate's explicit hard-failure message on the same file remains the authoritative unreadability signal.~~ on grep exit 2 or above, the explicit return-code capture fires an explicit hard-failure violation ('STATE.md: population-count — grep hard failure (exit code N; file unreadable; check did not run)') — a file-unreadability incident and genuine row damage are distinguishable on this predicate: unreadability fires the hard-failure message, row damage fires the population-mismatch message. [S3-69 UL5, fix-burst 113, 2026-07-28] [S3-70 UL1, fix-burst 114, 2026-07-30] **L15** (STATE.md workstream-(3) carrier parity: the 'current_step:' frontmatter key and the '**Current Step**' metadata-table row (extraction anchored to '^| **Current Step**' — a '| **Current Step**' substring appearing inside prose or narrative text is not matched and does not interfere with the check) each independently carry the workstream (3) burst number; checks that the '(3) BURST NNN' burst number extracted from each carrier matches; fires when they diverge; scoped exclusively to the '(3)' component — workstreams (1) and (2) not inspected; absent STATE.md: silent skip; ~~absent '(3) BURST NNN DONE' component in either carrier → SKIP with stderr notice~~ absent '(3) BURST NNN' component in either carrier → SKIP with stderr notice (burst number extracted from a '(3) BURST <digits>' token — no suffix such as 'DONE' is required by the extractor; any text after the digits is ignored; step descriptions using 'SUB-BURST', 'PARKED', or 'MICRO-BURST' notation are not matched and produce a SKIP); ADD-CHECK-FIRST fires on AK1 divergence — frontmatter 'current_step:' carried '(3) BURST 304 DONE' while '**Current Step**' metadata table carried '(3) BURST 303 DONE') → routing: carrier-currency repairs → state-manager; script defects → architect [S3-65 AK1, fix-burst 109, 2026-07-27; step-1b (AG1+AJ3 class): DONE-suffix false requirement struck; anchoring of '**Current Step**' carrier to '^| **Current Step**' now stated]. *Check L15 known gap — 'Last Updated' carrier is outside all mechanical coverage:* the AK1 incident that created check L15 was a two-of-three carrier advance defect; the three carriers are the 'current_step:' frontmatter line, the Project Metadata '**Current Step**' cell, and the Project Metadata '**Last Updated**' cell; check L15 compares only the first two. The '**Last Updated**' cell carries a burst number with no workstream prefix and is legitimately advanced by workstream-(1) commits, so it is deliberately outside check L15's workstream-(3) scope. The consequence to state honestly: a divergence between the '**Last Updated**' burst number and the '**Current Step**' burst number is detected by NO check and requires manual inspection; do not imply the AK1 class is fully closed by mechanical means — it is not. *Multi-token carrier behavior:* if a carrier ever held two workstream-(3) burst tokens, the comparison degrades fail-noisy, never false-green; an adversary probed this and refuted it as a defect; it is documented because it was previously undocumented, not because it is a problem [fix-burst 110, 2026-07-27]. (3) The lint SUPPLEMENTS — does NOT replace — the four ⚠ disciplines (STORYBOARD-TIER LIVE-MIRROR, TERMINAL PIN-RECONCILIATION, MIRROR-SOURCE CONTRADICTION, TWIN-BLOCK LINKAGE) and their ⚠ extensions. (4) Lineage: seven consecutive passes (S3-34..S3-40) minted attribution findings in fix-burst record text despite strengthened self-audit instructions; prose self-audits are demonstrably insufficient — ADV-S3P40-003 (attribution class) survived its own strengthened self-audit in the burst that adjudicated it; mechanical enforcement is the class-kill. (5) Lexical class sweeps MUST enumerate MORPHOLOGICAL VARIANTS of the target phrase, not one surface form. At minimum: space-separated, hyphenated and compound-adjective forms of every term in the pattern. A sweep whose recorded pattern cannot match a form of its own target class is not a sweep, and its attestation is unsupported. Every sweep record MUST state the pattern actually run and a per-hit disposition for every match, so the attestation is reproducible from its own stated basis. [S3-67 UL4, fix-burst 111, 2026-07-27] [trend-gate #4 structural intervention, fix-burst 84, 2026-07-24; tool: `tools/records-lint.sh` checks L1 L2 L3 L4 L5 L6 L7 L8 L8b L8c L8d L9 L10 L11 L11b L12 L13 L14 L15]

> - [ ] **Fix-burst 87 (lever 1, ADV-S3P43-OBS-3, 2026-07-24) — RECORD LINE-CITE BAN criterion:** (Adjudication: this is a NEW top-level Stage-3 quality-gate criterion — a SIBLING of MECHANICAL RECORDS-LINT; it introduces a new lexical restriction on newly-authored record text and supersedes the prior frame-declaration mitigation for line-position cites.) (1) All newly-authored record text — ratification blocks, changelog rows (§7 rows), ledger rows, pin-currency chains, lesson entries, amendment records — MUST cite section anchors, heading names, and symbol names ONLY. Bare line-position cites (`:NNN` form or `filename:NNN` form) are FORBIDDEN in newly-authored record text. Pure `NNNL` wc-count scalars (e.g., `807L`) are permitted; they do not constitute a line-position cite. (2) The former frame-declaration mitigation (which allowed a line-position cite inside a named-section framing) is SUPERSEDED; section-or-anchor citation is the complete replacement form, not an additional wrapper. (3) History predating Burst 279 is grandfathered — this ban applies only to record text authored at Burst 279 onward. (4) Mechanically enforced by lint L9 (see MECHANICAL RECORDS-LINT criterion, L9 check); violations route to the authoring agent. When git is unavailable or a git diff fails for every scanned file, check L9 is reported as SKIPPED in the terminating PASS verdict and is omitted from the list of clean checks, so a check that did not run can no longer be read as having passed. The scan population was also extended this burst from six files to eighteen, adding 'tools/records-lint.sh', WORKFLOW-INVENTORY.md, the seven journey files and the three persona files. [S3-67 UL7 + UL8, fix-burst 111, 2026-07-27] This exclusion is no longer specific to check L9. The terminating PASS verdict now omits any check that was wholly skipped — check L8c, check L8d, check L13 and check L15 among them — so a check that did not run can never be read as having passed. Checks that ran on some inputs and skipped others, such as check L8b, check L10 and check L14, remain listed with a per-sub-path SKIPPED notice, because they did execute. [S3-68 UL7, fix-burst 112, 2026-07-28] (5) Lineage + provenance: root CLAUDE.md anti-volatile-pin discipline applied; the explicit exception for record text in newly-authored blocks retired 2026-07-24 (root CLAUDE.md anti-volatile-pin exception retired same date; cross-applied to prism + ferrochain). Root cause this criterion kills: ADV-S3P40-004/-008 → ADV-S3P42-002 → ADV-S3P43-001/-002 — three consecutive intra-burst recurrences of bare line-position cites in newly-authored record text surviving prose self-audit; mechanical L9 guard is the class-kill. [lever 1, ADV-S3P43-OBS-3, fix-burst 87, 2026-07-24]

> - [ ] **Fix-burst 87 (lever 3, 2026-07-24) — RECORDS-ONLY MICRO-BURST criterion:** (Adjudication: this is a NEW top-level Stage-3 quality-gate criterion establishing an acceleration path for fix-bursts where all findings are at the records tier — a speed lever on the burst class, not a reduction in lint-gate rigor.) (1) When an adversarial pass returns findings that are EXCLUSIVELY records-tier LOW or OBS (zero CRIT, zero HIGH, zero MED; zero content-mechanism defects of any severity), the fix-burst executes as a TWO-STEP MICRO-BURST; the same two-step shape applies to records-tier defects surfaced outside a formal adversarial pass (orchestrator catch, human report), with the initiating catch recorded in the ledger row [ADV-S3P47A-OBS-1, fix-burst 91, 2026-07-25]: Step 1 — ownership-routed fixer step (routed by record type to the authoring agent) ~~+ state-manager commit~~ *(Step 1 is the fixer step only; the state-manager commit is the terminal act of the micro-burst, executed after Step 2 returns [ADV-S3P46B-003, fix-burst 90, 2026-07-25])*; Step 2 — PO step that is REPORT-ONLY-OR-SKIPPED per the REPORT-ONLY BURST RULE when no requirements content changes (the PO step may produce a brief ratification note confirming no requirements text was authored or amended, but does not itself author new requirements content). (2) The MECHANICAL RECORDS-LINT gate remains MANDATORY at commit — lint must EXIT 0 before the state-manager commit proceeds; the micro-burst path does not waive any lint check. (3) The terminal architect-resume (TERMINAL PIN-RECONCILIATION criterion, ⚠ MANIFEST REQUIREMENTS-PINS EXTENSION) still applies and is TRIGGERED if the fix-burst advances the requirements version; lesson-50's architect-resume obligation is governed by version advancement, not by content depth, and is unaffected by this criterion. The PO's optional Step-2 note is REPORT-ONLY — it lands in the burst ledger row via state-manager ~~(appended to the Step-1 commit)~~ *(superseded: the commit does not exist at Step 1; state-manager incorporates the PO note into the ledger row and authors the single commit as the terminal act after Step 2 returns [ADV-S3P46B-003, fix-burst 90, 2026-07-25])* and is never authored as a second commit, preserving the single-commit-per-burst rule. [ADV-S3P44B-OBS-1, fix-burst 88, 2026-07-24] More precisely: state-manager authors the single commit AFTER the PO's Step-2 report returns — the commit is the terminal act of the micro-burst, incorporating the PO note into the ledger row before the commit is first made; amending a previously completed commit SHA is never the mechanism. [ADV-S3P45A-OBS-1, fix-burst 89, 2026-07-25] [lever 3, fix-burst 87, 2026-07-24]

> **EXAMPLE (Rivetry):** `journeys/journey-mechanical-designer.md` (persona: Devon Cole, MD) is the
> fullest worked example of the seven-field format across all six lifecycle stages, including the
> honest "focused-touch — the honest hard case" mobile-posture framing for WF-018/WF-023 and the
> explicit `<768px` hard floor for WF-052. `WORKFLOW-INVENTORY.md`'s net-classification workflow
> (WF-022) plus its multi-actor, tenant-isolation-reset backstage is the strongest Rivetry candidate
> for a service-blueprint layer per Step 6 (EE-proposer → agent-advisor → SR-confirmer frontstage over
> worker-fleet/`kicad-cli`/job-envelope/tenant-reset backstage).

---

## Artifact — `AGENTIC-UX-RUBRIC.md` *(Enhancements v2, new)*

> **Enhancements-v2 burst (2026-07-07, ux-designer, this pass).** Grounded in
> `.factory/analysis/storyboard-research-v2-hifi-agentic.md` Angle D. This is a **per-project artifact**,
> created once a product has an agent layer (cite it from Stage 3, above, and Stage 6, below) — sibling to
> the project's own UX-modernization-principles rubric (Stage 6). It makes the runbook's existing
> "Agentic touchpoint" field load-bearing: the field marks *where* an agent acts; this rubric checks
> *how well* that touchpoint is designed, against four dimensions synthesized from the most empirically
> validated human-AI-interaction guideline sets available.

| # | Rubric dimension | Design question every agent-touched frame must answer | Grounded in |
|---|------------------|-------------------------------------------------------|-------------|
| A1 | **Trust calibration** | Does the frame state what the agent can do, how well it performs, and its limitations/uncertainty — so users neither over- nor under-trust it? | Microsoft 18 Guidelines for Human-AI Interaction (CHI 2019, HAX Toolkit) G1/G2; Google PAIR mental-models + explainability; NN/g on AI hallucinations; Salesforce disclosure pattern; Apple HIG for Generative AI; Frontiers in Computer Science (uncertainty-visualization trust study) |
| A2 | **Explainability / "show your work"** | Does the frame give an understandable rationale for the agent's behavior/output, with progressive access to deeper reasoning and verifiable references? | Microsoft "make clear why the system did what it did"; Google PAIR explainability; NN/g segment-level verification/progressive disclosure; UXmag agentic-UX framework |
| A3 | **Human-in-the-loop confirmation** | Before the agent takes an impactful action, can the user approve / reject / refine the proposal, and easily correct/override its output? | Google PAIR feedback + control; Microsoft correction/dismissal guidelines; Salesforce conversation structure + escalation path; Apple HIG undo/correct |
| A4 | **Error / uncertainty / refusal handling** | Are error, uncertainty, and **refusal** states explicitly represented, with visible confidence cues, helpful error messages, refusal *explanations*, and clear recovery/fallback paths? | Google PAIR errors + graceful failure; Microsoft "when wrong"/"over time" guidelines; NN/g; agentic-design.ai confidence-visualization patterns; Frontiers in Computer Science |

**Emerging-vs-verified honesty note:** Microsoft's 18 Guidelines and Google PAIR are the most
empirically validated sources here; the rubric's specific four-dimension synthesis (A1–A4) — and the
practice of applying it mechanically per-frame — is this runbook's own adaptation, not a codified
industry standard. Treat A1–A4 as **adopt-as-view**, the same honesty standing this runbook already gives
proto-personas (Stage 1) and the coverage cube (Stage 2 Step 7a).

> **EXAMPLE (Rivetry).** `frame-12-direct-modeling-workbench`'s `state-c-disambiguation.html` ("refuse,
> never guess" on an ambiguous safety-load-bearing selection) is already a textbook **A4 refusal state**
> — the corpus's existing instincts already match the guideline literature. The net-classification confirm
> flow (WF-022, EE-proposer → agent-advisor → SR-confirmer) is the strongest **A3 human-in-the-loop**
> candidate — the SR-confirmer *is* the loop. The rubric's job is to make that a *checked* requirement for
> every agent-touched frame, not an ad-hoc strength of one.

---

## Stage 4 — Design-Language Direction Exploration

### Purpose

Produce a small number of genuinely distinct visual directions and select one through a **structured,
criteria-based decision** — not an implicit founder/stakeholder taste pick — framed as Double Diamond
*Develop → Deliver* (Enhancement #7, `storyboard-process-research.md` §5, §9.6).

### Inputs

- Product brief brand attributes (elicit if not already named)
- ICP / target ecosystem (what "premium" or "trustworthy" means for this audience)
- Any existing brand assets
- Accessibility constraints (contrast, motion, color-independence)

### Steps

1. **Author a 1–2 page design brief** (`DESIGN-BRIEF.md`): objectives, audience, 3–5 named brand
   attributes, voice/tone, constraints, success metrics. This brief **becomes the evaluation criteria**
   for Step 3 — do not write it after the directions exist.
2. **Produce 3 (default) to 4 directions** (`direction-NN-<slug>/`), each a genuinely distinct
   interpretation of the brief's brand attributes — state "the bet" explicitly for each, plus a token
   sketch (palette, type, spacing, motion, one signature element), a rationale, and one reference
   screenshot at a single representative resolution (this is a direction-level sketch, not the frame's
   full Stage 7 evidence set). More than 4 directions fragments stakeholder discussion
   (`storyboard-process-research.md` §5) — do not exceed 4 without a specific reason.
3. **Build a weighted decision matrix** (`DECISION-MATRIX.md`): the brief's criteria as rows, each
   direction as a column, each criterion weighted, each direction scored, totals computed. Include
   accessibility fit and implementation feasibility as criteria even if not named explicitly in the
   brief.
4. **Document the rationale** for the chosen direction, tied to the matrix's scores against named
   criteria — never "the team liked it." Name the runner-up and why it lost.
5. **Human decision gate:** present the brief + directions + matrix to the human sponsor. If a
   provisional adoption is needed before formal ratification (e.g. to unblock downstream frame work),
   say so explicitly and name what ratification is still pending — never silently treat a provisional
   pick as final.

> **Enhancements-v2 burst (2026-07-07, ux-designer, this pass) — Step 6.** Grounded in
> `.factory/analysis/storyboard-research-v2-hifi-agentic.md` Angle C, C1. The ratified direction's Step-2
> "token sketch" (palette, type, spacing, motion, one signature element) has, until now, stayed a prose
> sketch — leaving every per-frame `styles.css` (Stage 6) to re-declare the same design values
> independently, the exact drift surface DTCG-era tooling targets.

6. **Crystallize the ratified direction's token sketch into a real token file.** Once a direction is
   ratified (Step 5), author `design-language/tokens/<name>.tokens.json` conforming to the **W3C Design
   Tokens Community Group (DTCG) Design Tokens Format Module, version 2025.10** — the first stable,
   vendor-neutral token specification (`$value`/`$type`/`$description` fields; supports modern color
   spaces such as Oklch/Display P3). Tier the tokens **primitive/global** (raw brand scales, e.g.
   `color.brand.primary.500`, `space.4`) → **semantic/alias** (purpose-driven roles, e.g.
   `color.text.primary`, `color.surface.warning`) → **component-specific** (used sparingly, e.g.
   `button.primary.background`) — this three-tier architecture is strong community consensus, not a
   spec mandate, so expect refinement over time. Run a **Style Dictionary** `css/variables` build to emit
   `design-language/tokens/tokens.css` (CSS custom properties, e.g. `--color-brand-primary-500`) — the
   single source of truth Stage 6 frames consume via `var(--…)`. No ratified direction remains a
   prose-only "sketch" past this step.

### Output Artifact(s) + Path Convention

- `storyboard/<version>/design-language/DESIGN-BRIEF.md`
- `storyboard/<version>/design-language/DECISION-MATRIX.md`
- `storyboard/<version>/design-language/direction-NN-<slug>/{index.html, styles.css, README.md, preview.png}`
- *(Enhancements v2, new)* `storyboard/<version>/design-language/tokens/<name>.tokens.json` +
  generated `tokens.css` — Step 6, for the ratified direction only

### Template

```markdown
# Design Brief

**Objectives:** ...
**Audience:** ...
**Brand attributes:** 1. <attr> 2. <attr> 3. <attr> ...
**Voice/tone:** ...
**Constraints:** ...
**Success metrics:** ...
```

```markdown
# Decision Matrix

| Criterion (weight) | Direction 1 | Direction 2 | Direction 3 | Direction 4 |
|---|---|---|---|---|
| <brand attribute 1> (w=<n>) | score | score | score | score |
| Accessibility fit (w=<n>) | | | | |
| Implementation feasibility (w=<n>) | | | | |
| **Weighted total** | | | | |

**Chosen:** Direction <N> — <one paragraph rationale tied to the scores above>.
**Runner-up:** Direction <M> — <why it lost>.
**Ratification status:** ratified <date> | provisionally adopted, pending <what>.
```

### Acceptance Criteria (Quality Gate)

- [ ] The design brief exists and predates the directions (not written retroactively to justify a
      pick already made).
- [ ] 3–4 directions exist, each with an explicit "bet" distinguishing it from the others.
- [ ] A weighted decision matrix exists, scored against the brief's own criteria.
- [ ] The chosen direction's rationale cites the matrix, not preference language.
- [ ] Ratification status is stated explicitly (ratified, or provisional-pending-X — never silent).

> **Enhancements-v2 burst (2026-07-07, ux-designer, this pass) — new criterion (Step 6).**
> - [ ] The ratified direction has a committed `.tokens.json` (DTCG 2025.10, tiered
>       primitive/semantic/component) + generated `tokens.css` — no ratified direction remains
>       prose-only.

> **EXAMPLE (Rivetry):** four directions exist (`direction-01-forge-precision-instrument` /
> `direction-02-atelier-editorial-clarity` / `direction-03-aurora-signature-hybrid` /
> `direction-04-forge-aurora-hybrid`), each with a named "bet" and full token sketch — this is
> already textbook stylescape practice per the research. What Rivetry's corpus did **not** yet have at
> the time of the research pass was the structured brief → weighted matrix → documented rationale this
> stage now requires going forward (the directions existed "ahead of P-004 brand ratification" as an
> implicit founder pick) — a genuine ENHANCE item this stage closes for future direction work.

---

## Stage 5 — Divergence Stage (Fat-Marker / Thumbnail)

### Purpose

Insert cheap, low-fidelity exploration **before** hi-fi HTML/CSS for any genuinely new frame, so holes
in the interaction design get found while they're still cheap to fix (Enhancement #5,
`storyboard-process-research.md` §2, §9.4).

### Inputs

- The target workflow(s) from `WORKFLOW-INVENTORY.md` (usually the highest-risk, highest-density, or
  Aha!-moment workflow in a given lifecycle stage first)

  > **Enhancements-v2 burst (2026-07-07, ux-designer, this pass).** Where Stage 2 Step 7b's
  > `Storyboard priority` column (WSJF default) has been computed, use it as the defensible, recomputable
  > ordering rule in place of this ad-hoc "highest-risk/highest-density/Aha!-moment" heuristic — the
  > heuristic remains valid latitude for a project that hasn't run Step 7b yet.
  > (`.factory/analysis/storyboard-research-v2-persona-workflow.md` Angle B, NN-B5.)
- The chosen design direction's rough token sketch (Stage 4) — colors/type as a loose guide only, not
  final values
- The state set this workflow/screen will need to cover (draft it now; Stage 6 finalizes it)

### Steps

1. For **every net-new frame** (a workflow/screen with no existing hi-fi frame), produce a rough
   sketch — hand-drawn photo, ASCII/ad-hoc-markdown wireframe, or bare unstyled HTML — covering the
   primary state, with at least a one-line note per other state in the draft state set.
2. **Stress-test for holes**: walk the sketch asking "what next / what could go wrong" per step
   (the story-mapping technique) and annotate directly on the sketch — this is where a missing
   exception path or an unaddressed ambiguous-state gets caught before it's expensive to fix.
3. **Graduate to Stage 6 (hi-fi)** only when: the sketch has been walked for hole-finding, the full
   state set is enumerated (not just sketched for the primary state), and — if the workflow's fidelity
   depends on it — the design direction (Stage 4) has at least a provisional pick. It is acceptable to
   **skip this stage** only for a frame that is a direct, low-risk variant of an already-hi-fi'd sibling
   frame in the same corpus (e.g. a mobile-detail variant of an existing desktop frame) — log the skip
   and the reason inline in that frame's README rather than silently omitting the sketch directory.

### Output Artifact(s) + Path Convention

- `storyboard/<version>/frames/frame-<id>-<slug>/sketch/` — thumbnail image(s) or a `sketch.md` with
  ASCII/markdown wireframes and inline hole-finding annotations.

### Template

```markdown
# Sketch — frame-<id>-<slug>

## Primary state (rough)
<ASCII wireframe or embedded thumbnail>

## Other states (one-liner each)
- <state>: <what's different>
- ...

## Hole-finding pass ("what next / what could go wrong")
- <step> → <what could go wrong here> → <resolved by X | flagged as open question>
```

### Acceptance Criteria (Quality Gate)

- [ ] Every net-new frame has a `sketch/` artifact, or an explicit, logged skip-rationale in its
      README citing the sibling frame it's a low-risk variant of.
- [ ] The hole-finding annotations exist and reference at least one concrete "what could go wrong"
      per major decision point.
- [ ] The full state set is enumerated before hi-fi work (Stage 6) begins.

> **EXAMPLE (Rivetry):** the corpus as it stands jumps directly to hi-fi HTML/CSS for every frame — this
> is the one enhancement (`storyboard-process-research.md` §2's explicit caution for
> "teams with strong build capability") that has **no existing worked example in the corpus yet**.
> Apply Stage 5 starting with the next net-new frame; do not retroactively sketch the frames already
> built hi-fi.

---

## Stage 6 — Hi-Fi Storyboard Frames

### Purpose

Produce production-fidelity HTML/CSS narrative frames that prove the interaction design end to end —
covering **every state in the enumerated state set, including failure/edge/empty states**, applying the
project's UX-modernization rubric, and linked to the formal spec's flow steps via a wireflow.

### Inputs

- The Stage 5 sketch (or its logged skip)
- The chosen design direction's full token set (Stage 4)
- The relevant SCR-*/FLOW-*/BC-* references (or, if the formal spec doesn't cover this workflow yet,
  the WF-*/CAP-* references from Stage 2)
- The project's UX-modernization principles rubric (see Rivetry's `UX-MODERNIZATION-PRINCIPLES.md` as
  the worked example) — or, on a first pass with no rubric yet, this stage's own output is what seeds
  one (see the note at the end of this stage)

### Steps

1. **Structure the frame directory:**
   `frame-<id>-<slug>/{styles.css, state-<letter>-<name>.html (one per state/path), README.md,
   frame-<id>-narrative.md (optional, for redesigns of a prior baseline)}`.

   > **Enhancements-v2 burst (2026-07-07, ux-designer, this pass) — token + vocabulary preconditions.**
   > Grounded in `.factory/analysis/storyboard-research-v2-hifi-agentic.md` Angle C, C1–C2. Two
   > preconditions apply **before** this step's `styles.css` is authored:
   > - **Tokens (C1).** `styles.css` must `@import` / reference the Stage 4-generated
   >   `design-language/tokens/tokens.css` and use `var(--token-name)` for every color/spacing/type/
   >   radius value a token exists for — **no literal hex/px where a token exists.** Per-frame
   >   `styles.css` is then limited to *layout/composition* unique to that frame, never re-declared
   >   design values (tokens above, composition below — both layers, not either/or).
   > - **Shared component vocabulary (C2).** Compose the frame from an enumerated **shared component
   >   vocabulary** (buttons, cards, inputs, banners, the confirm-dialog, the agent-reasoning panel,
   >   etc.) — same DOM structure + class names + token-backed CSS across every frame. A genuinely new
   >   pattern must be **added to the shared vocabulary first**, then used — never invented inline in
   >   one frame. This is the composition-discipline analog of Stage 2's zero-orphan discipline. (This
   >   is distinct from Stage 7's Storybook-as-VRT-baseline graduation path — that captures *evidence*;
   >   this is about *authoring vocabulary* — keep both, do not collapse them.)
2. **Apply the modernization-principles rubric selectively** — cite which principles apply to this
   frame and why; none are mandatory-per-screen checkboxes, but every choice to apply or skip a
   principle should be a stated decision, not a silent omission.
3. **Enumerate the full state set explicitly in the README before building**, then render **one file
   (or one clearly delineated block) per state in that set — including failure/edge/empty states, not
   only the primary/happy path** (Enhancement #4, `storyboard-process-research.md` §2, §9.4). This is
   the concrete, rendered form of "state coverage as a first-class deliverable" — naming the states in
   prose is necessary but not sufficient; each named state needs its own rendered evidence. Where
   Stage 2 Step 3a's state machine exists for this workflow, it is the **authoritative source** for this
   enumeration, not a re-derivation.

   > **Enhancements-v2 burst (2026-07-07, ux-designer, this pass) — content + AI-specific state
   > extensions.** Two refinements to this step's state set, neither duplicating the generic
   > failure/edge/empty coverage above — both are *additional dimensions*, not substitutes.
   > - **Realistic content, no lorem ipsum (Angle C, C3).** Every rendered state carries real/
   >   representative domain content and microcopy — never lorem ipsum — drawn from a small shared
   >   **content-fixture set** (e.g. `frames/_fixtures/` or documented inline constants) reused across
   >   frames so recurring entities/values stay consistent and reviewers can follow a journey across
   >   frames. Descriptive link/button text ("Learn more about X," not "Click here") and specific error
   >   copy ("Password must be at least 8 characters," not "Error in field") are accessibility matters,
   >   not just polish. The enumerated state set must include at least one **content-extreme** state
   >   (longest label, multi-line error, densest table) alongside the existing failure/edge/empty states
   >   — a content-extreme state is not an empty state; both are required where applicable.
   >   (`.factory/analysis/storyboard-research-v2-hifi-agentic.md` Angle C, C3.)
   > - **AI-specific states for agent-touched frames (Angle D, D3) — a specialization, not a duplicate,
   >   of the failure/edge/empty taxonomy above.** For any frame depicting an agent touchpoint, the state
   >   set additionally includes, where meaningful: an **agent-reasoning / "show-your-work"** state, a
   >   **low-confidence/uncertainty** state, a **human-in-the-loop confirmation (approve/reject/refine)**
   >   state, and a **refusal / "I don't know" / limitation-disclosure** state (a specialization of the
   >   generic failure state, per `AGENTIC-UX-RUBRIC.md`'s A4). Each renders, each closes on a resolution
   >   state (Step 4 below), and the frame is checked against `AGENTIC-UX-RUBRIC.md`'s A1–A4 dimensions
   >   at build time — a Stage-6 checklist item for agent-touched frames, parallel to the
   >   modernization-principles rubric (Step 2). (`.factory/analysis/storyboard-research-v2-hifi-agentic.md`
   >   Angle D, D3.)
4. **Add a resolution/"satisfaction" state** closing each path — goal attained, or explicitly refused —
   so no rendered path dead-ends without a clear terminal state.
5. **Build a wireflow** — at minimum a table in the README mapping `State → FLOW-* step(s) → BC(s)`; for
   a genuinely complex multi-path frame, an actual diagram (e.g. via the project's diagramming skill) is
   worth the extra effort (Enhancement #4, `storyboard-process-research.md` §2, §9.4). This is what
   makes narrative coverage checkable against interaction coverage.
6. **Cross-reference every claim** in the frame's README/narrative to a concrete BC/CAP/ADR/component
   contract file — a frame that invents behavior not grounded in the spec corpus is out of scope for
   this stage (route the invented behavior to the appropriate spec owner instead, per Stage 8).
7. **Document contextual variants explicitly**, as a required checklist inside the README, not left
   implicit: dark mode (or the project's default canvas), reduced-motion, touch-target sizing,
   color-independence for any safety/status signal, and — flagged as a real design problem to solve
   deliberately if not yet prioritized, never silently solved by omission — high-contrast mode and
   slow-network/optimistic-update behavior.

> **Enhancements-v2 burst (2026-07-07, ux-designer, this pass) — Steps 7a–8.** Grounded in
> `.factory/analysis/storyboard-research-v2-hifi-agentic.md` Angle C, C4–C5.

7a. **Author an accessibility-annotation layer** — a per-frame **`a11y-annotations`** block (a README
    section, or a dedicated `frame-<id>-a11y.md`) documenting, as **design intent** (not an after-the-fact
    audit): intended **focus/tab order**; **semantic role + ARIA** per interactive element; **landmark
    regions**; **heading-level** assignments; **alt text / icon labels**; and **acceptable contrast
    pairings, cited by token name** (e.g. `color.text.primary`, not a literal hex) so token changes never
    silently break the annotation. Author this *with* the frame; it feeds implementation handoff and
    references the Step 1 shared component vocabulary, which carries its own baseline a11y annotations.
    **This is distinct from Stage 7's WCAG screenshot matrix**, which is *evidence that reflow/layout
    works* (an audit artifact) — this layer is *authored intent* a screenshot cannot express; the two are
    complementary, and Stage 7's manifest gains one cross-check row confirming this layer is present and
    internally consistent, without turning Stage 7 into an authoring stage.
8. **Apply the AI-generation guardrail box** — because this runbook is itself executed by AI agents, any
    AI-generated frame markup/CSS is **not accepted** until it (a) references the Step 1 shared tokens
    (`var(--…)`, no literal values where a token exists), (b) composes only from the Step 1 shared
    component vocabulary (no invented components), (c) uses Step 3's realistic content, and (d) carries
    the Step 7a a11y annotations — **on top of** this stage's existing acceptance gate, not instead of it.
    This converts the general "don't hallucinate a component or a literal value" caution into this
    runbook's own mechanical, machine-checkable gate, consistent with its AI-executable design philosophy
    (v1 §6): AI-authored frames are held to the same token + vocabulary + content + a11y gates as
    hand-authored ones — the gate *is* the guardrail.

### Output Artifact(s) + Path Convention

- `storyboard/<version>/frames/frame-<id>-<slug>/{styles.css, state-*.html, README.md,
  frame-<id>-narrative.md?}`
- *(Enhancements v2, new)* `frame-<id>-a11y.md` (or a README `a11y-annotations` section) — Step 7a

### Template

`README.md`:
```markdown
# Frame <id> — <name> (<CAP-NNN>, `<SCR-*>` + `<FLOW-*>`)

> **Status:** storyboard exploration artifact, hi-fi. **Read-only against spec.** Every claim below
> traces to <list files>.

## Why this frame exists
<the gap it closes / the workflow it proves>

## Files
<tree>

## State set (enumerated before building)
| State | What it proves | Rendered as |
|---|---|---|
| <primary> | | state-a-*.html |
| <edge/failure> | | state-b-*.html |
| <empty> | | state-c-*.html |
| <content-extreme> [Enhancements-v2] | | state-*.html |
| <agent: reasoning/uncertainty/HITL/refusal, if agent-touched> [Enhancements-v2] | | state-*.html |
| <resolution> | | (final state in the primary path) |

## Wireflow
| State | FLOW-* step(s) | BC(s) |
|---|---|---|

## Principles applied
<cite the subset of the rubric that applies, and why>

## Agentic-UX rubric (if agent-touched) [Enhancements-v2]
<A1 trust / A2 explainability / A3 HITL / A4 error-uncertainty-refusal — addressed | deliberately N/A, per
`AGENTIC-UX-RUBRIC.md`>

## Contextual variants
- Dark mode / default canvas: ...
- Reduced motion: ...
- Touch targets: ...
- Color-independence: ...
- High-contrast: <addressed | flagged as open, not silently solved>
- Slow-network / optimistic-update: <addressed | flagged as open, not silently solved>

## Accessibility annotations [Enhancements-v2]
<focus/tab order · semantic role + ARIA per interactive element · landmark regions · heading levels ·
alt text/icon labels · contrast pairings cited by token name — see `frame-<id>-a11y.md` if split out>

## Mobile-first / responsive posture
<full / focused-touch / unavailable-below-Xpx, and why — an honest floor is acceptable, an unstated one is not>

## Persona framing
<which persona(s), why, and which personas this frame deliberately does NOT depict>

## What this frame deliberately does NOT do
<explicit non-scope list, so absence reads as a decision, not an oversight>

## AI-generation guardrail check [Enhancements-v2, if AI-authored]
<tokens (var(--...), no literal values) | vocabulary (no invented components) | content (no lorem ipsum)
| a11y (annotations present) — each ✅/❌>
```

### Acceptance Criteria (Quality Gate)

- [ ] The **golden reference artifact** for this project has been named (see note below) and this
      frame was checked against it.
- [ ] The state set is enumerated in the README *before* the HTML files, and every enumerated state
      (including at least one failure/edge state and one empty state where applicable) has a rendered
      file or block.
- [ ] A resolution/satisfaction state closes every rendered path.
- [ ] A wireflow (table minimum) maps every state to its FLOW-* step(s).
- [ ] Every behavioral claim traces to a concrete spec file — no invented behavior.
- [ ] The contextual-variants checklist is present and every item is either addressed or explicitly
      flagged as open (never silently omitted).

> **Enhancements-v2 burst (2026-07-07, ux-designer, this pass) — new criteria (Steps 1, 3, 7a–8).**
> - [ ] `styles.css` references the Stage-4 `tokens.css` and uses `var(--…)` for every value a token
>       exists for; every interactive element maps to a named entry in the shared component vocabulary,
>       or is explicitly logged as a net-new pattern promoted into that vocabulary this pass.
> - [ ] No frame contains placeholder/lorem text; a content-extreme state is rendered; recurring
>       entities use the shared content-fixture values.
> - [ ] For an agent-touched frame: the applicable AI-specific states (reasoning/uncertainty/HITL/
>       refusal) are enumerated and rendered, and the frame is checked against `AGENTIC-UX-RUBRIC.md`'s
>       A1–A4, with any skipped dimension a stated decision.
> - [ ] Every interactive element and every image/icon carries a role/label a11y annotation; focus order
>       is stated; contrast pairings cite token names.
> - [ ] Any AI-authored frame markup/CSS passes the guardrail check (tokens + vocabulary + content +
>       a11y) in addition to this stage's other criteria.

> **Golden reference artifact (Enhancement #7, `storyboard-process-research.md` §6, §9.7):** once a
> project has run Stage 6 at least once, **name the strongest resulting frame as the project's golden
> reference** and point every subsequent new frame at it for structure/rigor comparison.
>
> **EXAMPLE (Rivetry):** `frame-01b-modernized-net-classification` is the corpus's own golden reference
> — its `UX-MODERNIZATION-PRINCIPLES.md` rubric (11 principles, itself derived from redesigning this one
> frame) is exactly Stage 6's "this stage's own output is what seeds a rubric" case in action: Principle
> 11 (fluid-breakpointed layout, not a fixed-canvas mockup) was added *after* a Playwright-diagnosed
> regression in this same frame, and immediately generalized to every other multi-pane workspace frame
> in the corpus (see `frame-12-direct-modeling-workbench/README.md`'s own citation of Principle 11).
> `frame-12-direct-modeling-workbench` is a second strong worked example: its three states (drafting →
> applied → disambiguation-refused) are a clean instance of Step 3's "including failure/edge states,"
> its `state-c-disambiguation.html` is exactly the "refuse, never guess" failure-state render Step 3
> requires for a safety-load-bearing frame, and its README's own "What this frame deliberately does NOT
> do" section is the worked example for this template's matching field.

---

## Stage 6.5 — Design Validation *(Enhancements v2, new stage)*

> **Enhancements-v2 burst (2026-07-07, ux-designer, this pass).** Entire stage is new, grounded in
> `.factory/analysis/storyboard-research-v2-validation-structure.md` Angle E. Inserted between Stage 6
> (rendered hi-fi frames must exist first) and Stage 7 (may cite Stage 7 screenshots once they exist,
> but does not require them) — and is a **mandatory pre-gate to Stage 8 promotion**: no frame promotes to
> `SCR-*`/`FLOW-*` until it has passed this stage.

### Purpose

Ask "is this design any good?" — a question Stage 7's evidence matrix mechanically verifies *reflow and
layout at each breakpoint* but never asks. Two **inspection methods** (expert judgment against an
interface; no users required) are AI-runnable directly against static HTML/CSS + screenshots:
**heuristic evaluation** (Nielsen's 10, 0–4 severity) and **cognitive walkthrough** (a 4-question form
per task step). A third — **usability-test readiness** (first-click / tree-testing / task-success) — is
**empirical**: it needs real users, so an AI can only PREPARE it and hand it to a human-decision gate.
This inspection-vs-empirical split is drawn straight from Nielsen's discount-usability framing,
corroborated by NN/g and the Interaction Design Foundation (IxDF).

**Honest caveat (mandatory framing for this whole stage):** no authoritative source — not NN/g, not
ISO 9241, not IxDF — codifies AI-run heuristic evaluation or cognitive walkthrough; all three assume
human evaluators. AI inspection output here is **advisory and human-review-pending, never
authoritative** — exactly the honesty standing this runbook already gives Stage 1's proto-personas and
Stage 4's provisional direction picks. Mechanical fact-checks (Stage 2's zero-orphan matrix, Stage 7's
evidence manifest) stay authoritative; this stage is a third gate class — machine-*assisted* but
human-*authoritative*.

### Inputs

- The Stage 6 hi-fi frame's rendered states (all of them, including AI-specific and content-extreme
  states where applicable)
- The Stage 6 **wireflow** (`State → FLOW-* step(s) → BC(s)`) — the task-step sequence a cognitive
  walkthrough runs against
- The frame's persona(s) (Stage 1/3) — an inspection primed context-free is not valid per ISO 9241-11's
  usability definition (effectiveness/efficiency/satisfaction *for specified users in a specified
  context*)
- Stage 7 evidence screenshots, if they already exist (optional input; not required to start)

### Steps

1. **Heuristic evaluation pass.** Run **Nielsen's 10 heuristics** — (1) visibility of system status;
   (2) match between system and the real world; (3) user control and freedom; (4) consistency and
   standards; (5) error prevention; (6) recognition rather than recall; (7) flexibility and efficiency of
   use; (8) aesthetic and minimalist design; (9) help users recognize/diagnose/recover from errors;
   (10) help and documentation — against the frame's rendered states + screenshots. Record each finding
   with the heuristic cited and a **0–4 severity** (0 = not a problem, 1 = cosmetic, 2 = minor, 3 = major,
   4 = catastrophe/must-fix). Because a single evaluator's severity rating is unreliable, run **≥3
   independent evaluator passes** (three separate AI evaluations, or 2 AI + 1 human) and take the **mean
   severity**. Note the honest limit: dynamic heuristics (1, 5) are only partly inspectable on a static
   mockup — infer post-action behavior from microcopy/patterns, don't invent it. Emit `HEURISTIC-EVAL.md`
   per frame (or a README section).
2. **Cognitive walkthrough pass.** For each core task in the frame, take the step sequence straight from
   the Stage 6 wireflow, primed with the frame's persona. Answer the canonical **4 questions** per step:
   (1) will the user try to achieve the right effect/subgoal? (2) will they notice the correct action is
   available? (3) will they associate the action with the effect they want? (4) after the action, will
   they see that progress was made toward the goal? Any "No" is a fail-step, recorded with a stated cause.
   Emit `COG-WALKTHROUGH.md` per frame (or a README section). Same caveat as Step 1: static mockups
   underexpose dynamic feedback (Question 4) — infer honestly, flag where inference is thin.
3. **Usability-test-readiness prep (NOT run here — a human-decision gate).** Produce a
   `USABILITY-TEST-PLAN.md` the AI CAN author: task scenarios, success criteria, the expected correct
   first-click target, the expected navigation path, and — for navigation-heavy frames — a candidate
   tree-test tree. Tag it `evidence_basis: prepared, unvalidated` and a `validate_by:` trigger (e.g.
   "first N moderated sessions at GA") — the same honesty-header pattern Stage 1 already uses for
   proto-personas. This artifact is a **prepared input to a human decision**, not an AI-runnable gate.
4. **Honesty header (mandatory on every AI-produced report).** Every `HEURISTIC-EVAL.md` and
   `COG-WALKTHROUGH.md` carries, verbatim or equivalent:
   `> AI-run inspection — advisory, human-review pending. Not authoritative per NN/g/ISO/IxDF (which
   assume human evaluators).`

### Output Artifact(s) + Path Convention

- `storyboard/<version>/frames/frame-<id>-<slug>/HEURISTIC-EVAL.md` (or a README section)
- `storyboard/<version>/frames/frame-<id>-<slug>/COG-WALKTHROUGH.md` (or a README section)
- `storyboard/<version>/frames/frame-<id>-<slug>/USABILITY-TEST-PLAN.md`

### Template

```markdown
## Heuristic Evaluation — frame-<id>-<slug>

> AI-run inspection — advisory, human-review pending. Not authoritative per NN/g/ISO/IxDF (which assume
> human evaluators).

Evaluator passes: <N ≥ 3>. Mean severity per finding shown below.

| # | Heuristic | Finding | Severity (0-4, mean of N) | State(s) affected | Fixed / Owner + re-verify trigger |
|---|-----------|---------|----------------------------|--------------------|-------------------------------------|
```

```markdown
## Cognitive Walkthrough — frame-<id>-<slug>

> AI-run inspection — advisory, human-review pending. Not authoritative per NN/g/ISO/IxDF (which assume
> human evaluators).

Persona: <code>. Task: <task name, from the Stage 6 wireflow>.

| Step | Q1 right subgoal? | Q2 notice action? | Q3 associate action→effect? | Q4 see progress? | Fail cause (if any) |
|------|--------------------|--------------------|-------------------------------|--------------------|------------------------|
```

```markdown
## Usability-Test-Readiness Plan — frame-<id>-<slug>

evidence_basis: prepared, unvalidated
validate_by: <concrete trigger — e.g. "first N moderated sessions at GA">

**Task scenarios:** ...
**Success criteria:** ...
**Expected first-click target:** ...
**Expected navigation path:** ...
**Candidate tree-test tree** (if navigation-heavy): ...
```

### Acceptance Criteria (Quality Gate — mandatory pre-gate to Stage 8)

- [ ] Heuristic evaluation exists with ≥3 evaluator passes; **no unresolved severity-4** (catastrophe);
      every **severity-3** either fixed or carried with an explicit owner + re-verification trigger
      (never a silent "later" — same discipline Stage 8 already applies to its own unresolved gaps).
- [ ] Cognitive walkthrough exists for every core task in the frame; fail-steps are listed with causes
      and each is fixed or routed (no unrouted fail-step past this pass).
- [ ] A usability-test-readiness plan exists, provenance-typed (`prepared, unvalidated`) with a
      `validate_by` trigger — flagged as human-gated, not silently skipped.
- [ ] Every AI-run report (heuristic eval + cognitive walkthrough) carries the advisory/non-authoritative
      honesty header.

> **Closing-checklist note.** The Closing Acceptance Checklist below gains a **Design validation** row
> between Hi-fi frames and Evidence, and the coverage cube's `🔍` glyph is produced by this stage.

---

## Stage 7 — Evidence & Responsive Validation *(closes P-014)*

### Purpose

Replace single-reference-resolution screenshots with **deterministic, breakpoint-complete, WCAG-inclusive
visual evidence** per rendered frame state — evidence a reviewer or an agent can verify mechanically
against a manifest, not eyeball once and trust.

### Inputs

- The hi-fi frame's rendered HTML files (Stage 6)
- A pinned Playwright/browser environment
- The frame's own enumerated state-coverage table (Stage 6 README)
- The project's own responsive-layout rubric (e.g. Rivetry's Principle 11 collapse boundaries) if one
  exists yet — if not, this stage's manifest is what starts building one

### Steps

1. **Adopt the standard six-row breakpoint matrix** (Enhancement #1/#15/#16,
   `storyboard-process-research.md` Executive Summary, §4, §9.5):

   | Row | Viewport (W×H) | Tier | Why this row |
   |-----|----------------|------|--------------|
   | 1 | **320×640** | phone (min) | **WCAG 2.1/2.2 SC 1.4.10 Reflow** conformance floor — content must reflow at 320 CSS px with no 2-D scroll; non-optional at AA. |
   | 2 | **390×844** | phone (primary) | Representative modern phone; real mobile traffic clusters 360–430. |
   | 3 | **768×1024** | tablet portrait | Canonical tablet; framework tablet breakpoint. |
   | 4 | **1024×768** | tablet landscape / small laptop | Tailwind `lg`. |
   | 5 | **1280×800** | laptop | Tailwind `xl`. |
   | 6 | **1440×900** | large laptop/desktop | The historical single-reference resolution — kept as **one row, no longer the only row**. |
   | (opt) | 1920×1080 | large desktop | Common in analytics; optional, add if the project's real traffic data supports it. |

   > **EXAMPLE (Rivetry):** rows 4 and 5 are not arbitrary here — they are *exactly* Rivetry's own
   > Principle 11 collapse boundaries (`<1024` collapses, `>=1280` is full-workspace), so this matrix
   > is the row set that actually proves Principle 11's own claim, not a generic set layered on top of
   > it. A different project's rubric will produce different row-4/row-5 justifications; keep rows 1/2/3
   > (WCAG + real-traffic clustering) as non-negotiable regardless of project.

2. **Deterministic capture discipline** — every capture run must: pin the browser/Chromium version; run
   in a fixed environment; disable animations (respect or force `prefers-reduced-motion`); mock any
   dynamic data/timestamps; wait for network-idle before shooting. The goal: the same frame yields
   byte-identical screenshots unless the UI actually changed.
3. **Naming convention:** `frame-<id>-<state>-<W>x<H>.png` — one file per (state × breakpoint row).
4. **Tooling:** for static HTML/CSS frames, a **Playwright viewport-matrix script** iterating the six
   (or seven) rows across every state file in a frame is the minimal repeatable mechanism. Graduate to
   **Storybook + Chromatic/Percy/BackstopJS** baseline-diff VRT in CI once frames become live
   components.
5. **Produce a per-frame evidence manifest** — one table listing every captured breakpoint against a
   pass/fail for the project's own layout expectation at that breakpoint (e.g. Rivetry's Principle 11
   expectation: full workspace ≥1280 / compress 1024–1279 / collapse <1024, primary safety action
   reachable within one short scroll) — so a reviewer *or an agent* can verify coverage mechanically,
   without re-deriving the expectation from prose each time.
6. **Reject single-resolution evidence explicitly.** A frame whose evidence set has only one captured
   resolution does not pass this stage's gate — name this as a documented anti-pattern
   (`storyboard-process-research.md` §4: NN/g, BrowserStack, and WCAG all require ranges; a single
   desktop width cannot even evaluate SC 1.4.10), not an acceptable minimum.

### Output Artifact(s) + Path Convention

- `storyboard/<version>/ui-evidence/<phase-tag>/frame-<id>-<state>-<W>x<H>.png` (6–7 files per state)
- `EVIDENCE-MANIFEST.md` (either a standalone file per phase-tag, or a manifest section appended to
  each frame's own README)

### Template

```markdown
## Evidence Manifest — frame-<id>-<slug>

Captured: <date>, <Playwright/Chromium version pinned>, animations disabled,
`prefers-reduced-motion` <forced/respected>, network-idle wait <Y/N>.

| State | 320×640 | 390×844 | 768×1024 | 1024×768 | 1280×800 | 1440×900 | 1920×1080 (opt) |
|---|---|---|---|---|---|---|---|
| state-a-<name> | ✅/❌ | ✅/❌ | ✅/❌ | ✅/❌ | ✅/❌ | ✅/❌ | ✅/❌/N-A |
| state-b-<name> | ... |
```

### Acceptance Criteria (Quality Gate)

- [ ] Every rendered state (Stage 6) has all six required breakpoints captured (seven if the optional
      row is elected for this project).
- [ ] Capture is deterministic — the pinned browser/Chromium version and the animation/network-idle
      discipline are documented alongside the PNGs, not assumed.
- [ ] A manifest exists with an explicit pass/fail per (state × breakpoint) cell, not just raw PNGs
      with no verdict.
- [ ] No frame with only one captured resolution passes this gate.

> **This closes P-014.** The gap named in `storyboard-process-research.md`'s framing was "evidence at
> one resolution"; this stage's fix is "evidence at a standard, WCAG-inclusive, deterministically
> captured matrix, committed as artifacts with a pass/fail manifest." Rivetry's own Principle 11
> regression — a fixed 1440×900 canvas that clipped and horizontally scrolled at 1366×768 — is precisely
> the class of defect this matrix, applied from the start, would have caught automatically.
>
> **EXAMPLE (Rivetry) — legacy evidence, not a model to copy.** `.factory/ui-evidence/phase-b-responsive/`
> already holds a 4-row, width-only evidence set (`frame-<id>__<state>__<W>.png`, only 320/390/768/1440 —
> note the double-underscore separator and the missing `x<H>` suffix) for `frame-01` through `frame-11`.
> This predates this runbook and is **flagged as legacy**, not deprecated-and-deleted: new frame work
> (starting with `frame-12` and beyond) adopts this stage's six-row `WxH` convention going forward.
> Backfilling the legacy set with the two missing rows (1024×768, 1280×800) and renaming to the `WxH`
> convention is a real, valid housekeeping item — but it is not a blocker on new work, and per this
> project's own governance it must be attached to a concrete future story/burst if deferred, not left as
> a bare "later."

---

## Stage 8 — UX-Spec Modernization + Traceback

### Purpose

Close the loop: feed stabilized storyboard findings back into the formal, implementation-ready UX spec
(`UX-INDEX.md`/`SCR-*`/`FLOW-*`), so the exploration layer this whole process produces actually reaches
the artifact that implementers, test-writers, and e2e-testers consume.

### Inputs

- All Stage 2–7 artifacts for the workflow/frame being promoted
- The current `UX-INDEX.md` and its `screens/`/`flows/` shards
- The project's agent-routing table (who owns BCs vs. screens vs. component contracts)

### Steps

1. **Route every unrouted gap** (⛔ NO-SCREEN rows from Stage 2, UNRESOLVED questions surfaced in a
   Stage 6 frame narrative) to the correct spec owner per the project's routing table — a BC gap goes
   to whoever owns behavioral contracts; a screen gap goes to the UX spec owner; a component-contract
   gap (e.g. a confirmation surface's presentation-variant axis) goes to whichever pair of specialists
   jointly own that contract layer. Never leave a flagged gap to accumulate silently past the pass that
   raised it.
2. **Promote stabilized frame content** into the formal spec: for any frame whose design direction is
   ratified (Stage 4), whose evidence manifest is green (Stage 7), whose principles/state-coverage
   checklist (Stage 6) is complete, **and whose design-validation gate (Stage 6.5) has passed
   (Enhancements-v2 burst, 2026-07-07, ux-designer, this pass)** — no unresolved severity-4 heuristic
   finding, no unrouted cognitive-walkthrough fail-step, a prepared usability-test plan on file — author
   or update the corresponding `SCR-*.md`/`FLOW-*.md` using the project's standard UX-spec templates,
   citing the storyboard frame as the design-exploration source.
3. **Update `UX-INDEX.md`'s** screen/flow inventory and any coverage tables to reflect the promotion.
4. **Re-run Stage 2's coverage matrices** to confirm the promotion introduced zero new orphans (a
   promoted screen with no corresponding workflow row, or a workflow row whose UX-coverage cell wasn't
   updated to point at the new SCR-*/FLOW-*, are both defects this re-run catches).
5. **Update design-system component contracts** if the frame surfaced a new pattern or a contract gap —
   route to the correct joint owners; a genuinely unresolved contract question (e.g. does a
   confirmation component need a new presentation variant?) must either be resolved this pass or
   explicitly carried forward with an owner and a re-verification trigger named — never left as an
   implicit "someone will notice."
6. **Record the traceback bidirectionally:** the promoted `SCR-*`/`FLOW-*` file cites the storyboard
   frame as its design-exploration source (frontmatter or a References section); the source storyboard
   artifact gets an in-place, dated annotation ("Resolved — promoted to SCR-NNN") rather than being
   rewritten or deleted — preserving the corpus's own audit trail.

### Output Artifact(s) + Path Convention

- Updates to `specs/ux/UX-INDEX.md`, `specs/ux/screens/SCR-*.md`, `specs/ux/flows/FLOW-*.md`
- In-place, dated annotations on the source storyboard artifacts (no rewrites)

### Template

Uses the project's existing UX-spec screen/flow templates for the SCR-*/FLOW-* output. The
traceback annotation on the source artifact:

```markdown
> **Resolved (<date>, <agent>, this pass).** Promoted to `SCR-NNN`/`FLOW-NNN` — see that file for the
> now-authoritative spec. This storyboard artifact remains as the design-exploration record; claims
> here that predate the promotion are historical, not live spec claims, unless the promoted file says
> otherwise.
```

### Acceptance Criteria (Quality Gate)

- [ ] Zero flagged gaps (from any earlier stage) are left unrouted at the end of this pass.
- [ ] Every promoted `SCR-*`/`FLOW-*` file cites its storyboard source.
- [ ] `WORKFLOW-INVENTORY.md`'s coverage matrices were re-run after promotion and show zero new
      orphans.
- [ ] No design-system component-contract question raised by a Stage 6 frame remains UNRESOLVED past
      this pass without an explicit owner and re-verification trigger.

> **EXAMPLE (Rivetry):** this is exactly the "INTEGRATE burst" pattern already used repeatedly in
> `WORKFLOW-INVENTORY.md` — e.g. the burst that landed `BC-2.11.014`–`.016`, promoted
> `frames/frame-10-gdpr-erasure-intake/` and `frame-11-erasure-admin-queue/` into `SCR-018`/`SCR-019`/
> `FLOW-016`, and updated every previously-`🟡 storyboard-only` row to cite the new official coverage —
> and the "RESOLVED pass-35/D-086" annotation on `frame-01b-narrative.md`'s own gap #1 (the
> `risk-confirm-dialog.yaml` `presentation: overlay | anchored-panel` axis question), which stayed
> visibly UNRESOLVED across several passes with a named owner before finally being closed in place —
> the correct behavior per Step 5, not a silent gap.

---

## Closing Acceptance Checklist — Complete Storyboard Package

Use this checklist to certify a storyboard corpus (or a newly added slice of one) as complete and ready
for Stage 8 promotion / human review:

- [ ] **Personas:** every Actor/Role has a persona; every persona is provenance-typed with a
      validation trigger and carries a JTBD statement.
- [ ] **Inventory:** zero-orphan capability/subsystem/screen-flow coverage; every row has a
      Path-coverage and Verification cell; the capability list was re-diffed against the current spec
      this pass.
- [ ] **Journeys:** one file per persona, full seven-field coverage per workflow, an emotion curve,
      service-blueprint layers where required, every gap routed.
- [ ] **Design language:** a design brief predating the directions; 3–4 directions with named bets; a
      weighted decision matrix; a stated ratification status.
- [ ] **Divergence:** every net-new frame has a sketch or a logged skip-rationale.
- [ ] **Hi-fi frames:** every enumerated state (including failure/edge/empty + a resolution state)
      rendered; a wireflow; every claim spec-traced; a contextual-variants checklist addressed or
      flagged.
- [ ] **Evidence:** every frame state captured at all six (or seven) required breakpoints,
      deterministically, with a pass/fail manifest — no single-resolution evidence anywhere.
- [ ] **Traceback:** zero unrouted gaps; every promoted frame cited in its SCR-*/FLOW-*; coverage
      matrices re-clean after promotion.

> **Enhancements-v2 burst (2026-07-07, ux-designer, this pass) — new checklist rows.**
> - [ ] **Persona completeness:** an ecosystem/onion + role-permission/RACI pass (Gate 1) exists in
>       addition to the orphan check (Gate 2); every core interaction demand has a Persona Spectrum
>       coverage note (permanent/temporary/situational, or "N/A"); every sensitive/safety surface has an
>       anti-persona or a stated risk-accepted rationale, kept as a separate roster; every load-bearing
>       agent has a system-persona brief.
> - [ ] **Coverage cube:** `personas/PERSONA-WORKFLOW-MATRIX.md` exists, is generated (not
>       hand-diverged), and every persona × workflow cell carries a status glyph.
> - [ ] **Storyboard priority:** every WF row has a non-blank `Storyboard priority` score (WSJF default
>       or a stated alternative).
> - [ ] **Token SSoT:** the ratified direction has a committed DTCG `.tokens.json` + generated
>       `tokens.css`; no frame hard-codes a value a token exists for.
> - [ ] **Accessibility annotation:** every hi-fi frame's authored a11y-annotation layer (focus order,
>       roles, ARIA, alt text, contrast-by-token) is present and distinct from Stage 7's screenshot audit.
> - [ ] **Agentic-UX rubric:** every agent-touched frame is checked against `AGENTIC-UX-RUBRIC.md`'s
>       A1–A4, with AI-specific states rendered where applicable.
> - [ ] **Design validation (Stage 6.5):** every promoted frame passed heuristic evaluation (no
>       unresolved severity-4) and cognitive walkthrough (no unrouted fail-step), and has a prepared,
>       human-gated usability-test plan on file. *(This row sits between Hi-fi frames and Evidence in
>       promotion order.)*
> - [ ] **Persona-view traceability:** every persona has a `personas/persona-<code>.md` index linking to
>       every workflow it touches, its canonical frame, and its evidence — with zero artifact
>       duplication against the canonical stores.

---

## Enhancements Folded In — Confirmation

Every enhancement identified in `.factory/analysis/storyboard-process-research.md` is folded into this
runbook, at the stage noted:

| # | Enhancement | Folded in at |
|---|-------------|---------------|
| 1 | Standard breakpoint matrix + deterministic per-breakpoint evidence (closes P-014) | Stage 7, all steps |
| 2 | Proto-persona typing + validation cadence | Stage 1, Step 3 + template |
| 3 | Service-blueprint layer (frontstage/backstage/line-of-visibility) | Stage 3, Step 6 + template |
| 4 | Failure/edge/empty-state frames + wireflow linking frame → FLOW-* | Stage 6, Steps 3–5 |
| 5 | Cheap divergence stage (fat-marker/thumbnail) before hi-fi | Stage 5, entire stage |
| 6 | Need/JTBD layer above CAP + verification column + exception/recovery path-coverage audit | Stage 0 (trace chain), Stage 2 Steps 2–5, Stage 1 Step 3 |
| 7 | Structured design-direction decision (brief → weighted matrix → rationale, Double Diamond) | Stage 4, entire stage |
| — | AI-executable process design (explicit stages, DoD checklists, golden reference artifact, human-decision gates) | Whole-document structure; golden reference named in Stage 6; human-decision gates named explicitly in Stage 4 Step 5 and Stage 1's validation-cadence header |

**P-014 evidence discipline** (the six-row breakpoint matrix, deterministic capture, `WxH` naming, and
the per-frame evidence manifest) is fully folded in at **Stage 7**, including the explicit anti-pattern
naming of single-reference-resolution evidence and the reconciliation note against Rivetry's own
pre-existing legacy evidence set.

> **Enhancements-v2 burst (2026-07-07, ux-designer, this pass).** A second, human-approved research round
> (`.factory/analysis/storyboard-research-v2-persona-workflow.md`,
> `.factory/analysis/storyboard-research-v2-hifi-agentic.md`,
> `.factory/analysis/storyboard-research-v2-validation-structure.md`) adds 8 further enhancements,
> numbered continuing from #7 above — additive only, nothing above renumbered or removed:

| # | Enhancement | Folded in at |
|---|-------------|---------------|
| 8 | Persona-enumeration completeness: ecosystem/onion + RACI Gate 1, Persona Spectrum coverage pass, anti-persona roster (separate class, exempt from orphan check), first-class system/agent personas | Stage 1, Steps 3a–3d + template + acceptance criteria |
| 9 | Workflow-enumeration completeness: Cockburn extension-condition brainstorm, lightweight FMEA, state-machine + error/recovery-state enumeration, cross-persona swimlane/RACI discovery feeding the Stage 3 blueprint | Stage 2, Steps 2a–2b, 3a, 5a |
| 10 | Persona × workflow × state coverage cube + WSJF/RICE `Storyboard priority` column (replaces ad-hoc Stage 5 ordering) | Stage 2, Step 7 + template; Stage 5 Inputs note |
| 11 | Agentic-touchpoint field made rubric-checked against a new `AGENTIC-UX-RUBRIC.md` (A1 trust / A2 explainability / A3 HITL / A4 error-refusal) | Stage 3, Step 3 note; standalone `AGENTIC-UX-RUBRIC.md` section; Stage 6, Step 3 |
| 12 | DTCG `.tokens.json` token SSoT (primitive/semantic/component tiers) + Style Dictionary → `tokens.css`, crystallized from the ratified direction | Stage 4, Step 6 + output artifacts |
| 13 | Hi-fi production discipline: shared component-vocabulary precondition, authored a11y-annotation layer (distinct from Stage 7's WCAG screenshots), realistic content + content-extreme state, AI-specific agent states, AI-generation guardrail box | Stage 6, Step 1 note, Step 3 note, Steps 7a–8 + template + acceptance criteria |
| 14 | New Stage 6.5 — Design Validation: AI-runnable heuristic evaluation + cognitive walkthrough (advisory, human-review-pending) and human-gated usability-test-readiness prep, as a mandatory pre-gate to Stage 8 | New Stage 6.5, entire stage; Stage 8 Step 2 note |
| 15 | Agnostic, persona-traceable directory standard: `STORYBOARD-INDEX.md`, `personas/` view layer (roster + coverage cube + per-persona index, zero duplication), `design-language/tokens/`, status-glyph lifecycle, `<version>/` immutability + `VERSIONS.md` registry | Extended Artifact Map (new subsection); Naming & ID Conventions (glyph lifecycle, anti-persona ID class, versioning rule) |

**Engine-port note (unchanged).** The `vsdd-factory:persona-storyboard` skill engine-port named in this
document's own header remains the **deferred follow-up** — this Enhancements-v2 burst adds runbook
content only and does not perform or schedule that lift.
