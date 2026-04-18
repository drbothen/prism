# Task: Implement Lessons Learned from Prism Phase-3 Patch Cycle + Ship Policy 9 Integration

Drafted: 2026-04-18 (Prism Phase-3 patch cycle, post-pass-24)
Target plugin: vsdd-factory, installed version 0.24.1 at
`/Users/jmagady/.claude/plugins/marketplaces/vsdd-factory/plugins/vsdd-factory/`
Recipient: fresh Claude Code session rooted in the plugin source repo (not
the installed marketplace copy).

---

You are updating the `vsdd-factory` plugin. The target repo is the plugin
source (not a consuming project). The current installed version is 0.24.1 at:

  /Users/jmagady/.claude/plugins/marketplaces/vsdd-factory/plugins/vsdd-factory/

Confirm the source repo location at session start (likely a sibling clone;
the marketplaces/ path is the installed copy, not the dev source). If the
source clone is not obvious, ask the user — do not edit the installed copy
directly.

## Context

The Prism project (Rust-based MCP server, see /Users/jmagady/dev/prism/)
has been running a Phase-3 patch cycle in .factory/ for 24+ adversarial
passes. Recent session surfaced concrete bugs, gaps, and a new policy
(Policy 9 — vp_index_is_vp_catalog_source_of_truth) that need to be codified
into the plugin so every project benefits. Empirical evidence: Pass-24
finding P3P24-A-H-002 (prism-security fuzz column arithmetic drift) would
have been caught at edit-time by a Policy 9 lint hook.

## Baseline

Before any edit:
1. `git status` — ensure clean working tree, current branch noted.
2. Read `plugin.json` — note current version.
3. Read `skills/adversarial-review/SKILL.md` and
   `templates/adversarial-review-template.md` — these are the primary
   artifacts being modified.
4. List `hooks/` — note existing hooks (protect-vp.sh exists; we are adding
   validate-vp-consistency.sh).
5. Read `agents/adversary.md` and `agents/architect.md` — these may need
   prompt injection for Policy 9.
6. Announce baseline to the user before making any change.

## Work Units

Priorities are labeled P0 (ship this run), P1 (ship this run if time
permits), P2 (design + stub now, finish next run). Stop between priority
tiers and checkpoint with the user.

---

### P0.1 — Policy 9 lint hook

**File:** `hooks/validate-vp-consistency.sh` (NEW)

**Trigger:** PostToolUse on Write or Edit to any of:
- `specs/verification-properties/VP-INDEX.md`
- `specs/architecture/verification-architecture.md`
- `specs/architecture/verification-coverage-matrix.md`

**Behavior:**
1. Parse VP-INDEX.md — extract the VP catalog table (VP-NNN, module, tool,
   phase, status columns) and the per-tool summary (Kani/proptest/fuzz/
   mutation totals).
2. Parse verification-architecture.md — extract Provable Properties Catalog
   entries (VP-NNN → module, tool, phase) and the P0 list.
3. Parse verification-coverage-matrix.md — extract the VP-to-Module table
   (per-module columns: Kani, proptest, fuzz, VPs listed) and the Totals
   row.
4. Validate:
   (a) Every VP-NNN in VP-INDEX appears in verification-architecture.md
       Provable Properties Catalog with matching module/tool/phase.
   (b) P0 set in verification-architecture.md == VP-INDEX P0 subset.
   (c) Every VP in VP-INDEX is represented in verification-coverage-matrix.md
       (either in a VP-to-Module row cell or in the dedicated VPs column).
   (d) For each tool column (Kani, proptest, fuzz, mutation) in
       verification-coverage-matrix.md: sum of per-module counts ==
       Totals row == VP-INDEX summary total.
   (e) Every VP listed in a per-module row's "VPs" cell exists in VP-INDEX
       with matching module.
5. Output: on mismatch, emit a clear diagnostic listing which file(s) are
   inconsistent and the specific discrepancy. Exit non-zero to mark the hook
   as failed (harness surfaces this to the agent).
6. On all-pass, emit nothing and exit 0.

**Canary test case (MUST include):** Replay the Prism Pass-24 finding —
a verification-coverage-matrix.md where prism-security fuzz column = 2
but only VP-038 is listed and VP-INDEX Fuzz total = 6 (row sum = 7). The
hook MUST exit non-zero with a diagnostic naming the prism-security row,
the "2 vs 1 listed" mismatch, AND the "7 row-sum vs 6 Totals" mismatch.

Include the canary as a fixture under `tests/policy-9/` with before/after
file pairs and a `test-validate-vp.sh` driver.

**Language:** Prefer bash + `yq`/`awk` if sufficient, or python3 with no
external deps if parsing gets hairy. Document in hook comments which was
chosen and why.

### P0.2 — Policy 9 agent prompt injection

**File:** `agents/architect.md` (EDIT — add section near the top of
"Operating Procedure" or equivalent)

**Content:** Add a "Source-of-Truth Invariants" subsection listing, at
minimum:
- VP-INDEX.md is authoritative for the VP catalog. Any VP add/retire/
  rename/reassignment MUST propagate SAME-BURST to
  verification-architecture.md (Provable Properties Catalog + P0 list) AND
  verification-coverage-matrix.md (VP-to-Module table + Totals row).
- Before editing any of {VP-INDEX, verification-architecture,
  verification-coverage-matrix}, read all three. After editing any,
  re-read and verify symmetry before committing.

Cross-link to `hooks/validate-vp-consistency.sh` as the automated enforcer.

Also add a parallel subsection for existing sources-of-truth if not
already present (match the Prism project's policies 6 and 7):
- ARCH-INDEX.md subsystem name is authoritative for all BC, story, PRD
  references.
- BC H1 title is authoritative for BC-INDEX and story references to that
  BC.

### P0.3 — Fix adversarial-review skill post-adversary persistence

**File:** `skills/adversarial-review/SKILL.md` (EDIT)

**Problem:** The adversary agent has only Read/Grep/Glob tools. The skill
currently tells the caller to "Write findings to
`.factory/cycles/<current>/adversarial-reviews/`" in prose, but provides
no orchestration step. If the orchestrator direct-spawns the adversary,
the findings come back as chat text and are lost.

**Fix options (pick one, document the decision):**
(a) Add an explicit "After the adversary returns, dispatch state-manager
    to persist the verbatim review to <target path>" step in the skill's
    procedure, with the target path computed from the cycle name
    convention (see P1.1).
(b) Grant the adversary a scoped Write tool allowing only writes under
    `.factory/cycles/*/adversarial-reviews/` (path-prefix guard). This
    preserves fresh-context asymmetry while making persistence atomic.

Recommendation: Option (a) for clarity; Option (b) only if the harness
supports path-scoped tool allowlists natively.

Update the SKILL.md with the chosen procedure in imperative steps, not
prose, so the orchestrator cannot miss it.

### P0.4 — Filename collision guard in adversarial-review skill

**File:** `skills/adversarial-review/SKILL.md` (EDIT)

**Problem:** Prism has 33 historical `specs/adversarial-review-pass-*.md`
files from an earlier Phase-1 convergence. Default skill behavior would
overwrite them on a fresh cycle. We dodged it manually this session but
it's a landmine for every long-lived project.

**Fix:**
Add a pre-flight step to the skill:

```
Before writing any review file:
1. Check if the target path exists. If yes AND content differs, refuse —
   the caller must either use a cycle subdirectory (see .factory/cycles/
   layout) or an explicit cycle prefix in the filename.
2. Emit a clear error identifying the collision and pointing to the
   cycle bootstrap skill (P1.1).
```

Implement this as skill logic the orchestrator executes, since skills
themselves don't run code — this is a procedural constraint documented
in SKILL.md that the caller must follow. Note in SKILL.md that a future
enhancement would be a preflight helper script.

### P0.5 — Bump plugin version and changelog

**Files:** `plugin.json`, `CHANGELOG.md`

Bump minor version (e.g., 0.24.1 → 0.25.0). Changelog entry summarizes:
- Policy 9 enforcement hook added
- Adversarial-review skill persistence procedure clarified
- Filename collision guard added

---

### P1.1 — Cycle layout bootstrap skill

**File:** `skills/factory-cycles-bootstrap/SKILL.md` (NEW)

**Purpose:** Migrate projects from the flat `.factory/specs/adversarial-
review-pass-N.md` layout (where all historical reviews pile up in one
directory and filename collisions are inevitable) to a cycle-keyed layout:

```
.factory/
  cycles/
    phase-1-convergence/
      adversarial-reviews/
        pass-1.md
        pass-2.md
        ...
      policies.yaml     (see P2.1)
      INDEX.md
    phase-3-patch/
      adversarial-reviews/
        pass-24.md
        ...
      policies.yaml
      INDEX.md
  specs/
    (unchanged — specs live here; reviews no longer do)
```

**Skill behavior:**
1. Prompt user for cycle name (phase-1-convergence, phase-3-patch, etc.).
2. Scan existing `.factory/specs/adversarial-review-pass-*.md` — if found,
   offer to archive them under a user-named cycle (default:
   `phase-1-convergence`).
3. Create the cycle directory structure.
4. Move (git mv) existing files into the new layout so history is
   preserved.
5. Emit a summary report listing new paths for follow-up pipeline work.

**Integration:** The adversarial-review skill's default output path
becomes `.factory/cycles/<current-cycle>/adversarial-reviews/pass-<N>.md`
where `<current-cycle>` is read from `.factory/current-cycle` (a one-line
pointer file) or explicitly passed.

### P1.2 — Finding-ID cycle prefix in templates

**Files:**
- `templates/adversarial-review-template.md` (EDIT)
- `templates/adversarial-finding-template.md` (EDIT)
- `templates/adversarial-review-index-template.md` (EDIT)

**Problem:** Templates use `ADV-P[N]-NNN`. Single-cycle assumption breaks
for multi-cycle projects where Phase-1 and Phase-3-patch findings coexist.

**Fix:** Replace `ADV-P[N]-NNN` with `<CYCLE-PREFIX>-P[N]-<SEV>-NNN` where
CYCLE-PREFIX is configured in `.factory/current-cycle` or a project-level
config. Examples:
- `PHASE1-P03-CRIT-001`
- `PHASE3PATCH-P24-HIGH-002`

Document the prefix convention in SKILL.md so users configure it once per
cycle.

### P1.3 — Scoped review parameter

**File:** `skills/adversarial-review/SKILL.md` (EDIT)

**Problem:** Default skill reads every spec doc every pass. For mid-cycle
verification (was the last burst's fix correct?), that's overkill and
dilutes adversary attention.

**Fix:** Add a `--scope` argument to the skill:
- `--scope=full` (default) — read all specs. Used for convergence
  candidates.
- `--scope=diff-from:<commit>` — focus on files changed since commit.
  Used between fix bursts.
- `--scope=paths:<pattern>` — focus on specific files (e.g.,
  `specs/architecture/`).

Update the target-determination section of the skill to honor the flag.

---

### P2.1 — Policy registry

**File:** `skills/policy-registry/SKILL.md` (NEW) and supporting template

**Purpose:** First-class project policy storage. Currently policies are
encoded in ad-hoc memory artifacts and re-copied into every adversary
dispatch prompt. Must become declarative.

**Schema proposal** (`.factory/policies.yaml`):

```yaml
policies:
  - id: 1
    name: append_only_numbering
    description: "Retired BC/VP/DI IDs never reused. Filename slugs may
                 lag H1 after semantic rename."
    adopted: burst-4
    severity: HIGH
    enforced_by: [adversary-prompt]
    scope: [bc, vp, di]
  - id: 9
    name: vp_index_is_vp_catalog_source_of_truth
    description: "VP-INDEX.md is authoritative. Any change must propagate
                 same-burst to verification-architecture.md and
                 verification-coverage-matrix.md."
    adopted: burst-24
    severity: HIGH
    enforced_by: [adversary-prompt, lint-hook]
    lint_hook: hooks/validate-vp-consistency.sh
```

**Behavior:**
1. adversarial-review skill auto-loads this file into the adversary's
   dispatch rubric. No more hand-pasting policies.
2. Lint hooks read this file to know which validators to run. A single
   master hook (`validate-policies.sh`) delegates to per-policy validators.
3. A `/vsdd-factory:policy-add` slash command helps users register new
   policies mid-cycle with the right schema.

**Migration:** For Prism, the 9 existing policies are extracted from
.factory/STATE.md § "Policy flags" into `.factory/policies.yaml` as a
one-time migration — do NOT automate this step in this run; document it
as a follow-up on the Prism side.

### P2.2 — disable-model-invocation flag clarification

**Files:** `skills/adversarial-review/SKILL.md`, `agents/orchestrator.md`
(or equivalent)

**Problem:** The skill frontmatter has `disable-model-invocation: true`.
Unclear if this blocks orchestrator-initiated invocation or only
auto-heuristic suggestions. If strict, orchestrator autonomy in VSDD
Phase 1d / 4 is broken.

**Fix:** Either
(a) Change to `disable-model-invocation: false` with a doc note that
    orchestrator is the intended caller.
(b) Leave flag as-is and add explicit guidance in orchestrator.md that
    the user must run `/vsdd-factory:adversarial-review` between every
    fix burst. (Less autonomous but more auditable.)

Recommendation: (a). Document the decision.

---

## Testing

For each P0 item, add at least one automated test:
- P0.1 hook — fixture under `tests/policy-9/` with the Prism Pass-24
  canary plus at least one all-green case and one all-missing-VP case.
- P0.3 skill procedure — a doc-test / lint that verifies SKILL.md
  contains the persistence step verbatim.
- P0.4 collision guard — a fixture under `tests/collision-guard/` with
  pre-existing pass-24.md and a fresh invocation that must refuse.

For P1 items, stubs are acceptable this run but must include doc tests
describing expected behavior.

## Commit strategy

One commit per Work Unit. Commit message format:

```
plugin(<scope>): <concise what> — <why>

<body: cite Prism Pass-24 finding or lesson number>
```

Example:

```
plugin(hooks): add Policy 9 validate-vp-consistency hook — prevents VP-INDEX
↔ verification-architecture ↔ coverage-matrix drift (Lesson 5, Prism
P3P24-A-H-002)

Replays Prism Pass-24 fuzz column arithmetic drift as canary fixture.
Hook triggers PostToolUse on edits to the three VP source files and exits
non-zero on row-sum/totals/per-module mismatch.
```

Do NOT squash work-units. Each must be independently revertible.

## Deliverable

At end of run, report:
1. Which priorities shipped (P0 / P1 / P2).
2. Commit hashes for each work unit.
3. New plugin version number.
4. Any deferred items with a brief "why deferred" explanation.
5. Any plugin-repo constraints discovered that blocked intended work
   (e.g., monorepo structure differences, tool limitations).

## Out of scope

- Modifying Prism's `.factory/` — that's a separate workstream.
- Migrating Prism's existing 9 policies into `.factory/policies.yaml` —
  document as a follow-up, don't do it.
- Archiving Prism's historical `adversarial-review-pass-1..33.md` —
  separate Prism housekeeping burst.
- Any project other than the plugin repo.

## Escalation

Stop and ask the user if:
- The plugin source repo location cannot be determined.
- Any work unit requires breaking API changes (e.g., renaming existing
  agent IDs) — bump major version and confirm first.
- The canary fixture (Prism Pass-24 P3P24-A-H-002) cannot be faithfully
  reproduced because it depends on non-trivial file content that's not
  available — ask for the three source files to be pulled from the Prism
  repo.

## Lesson indexing

This run closes the following lessons from the Prism Phase-3 patch cycle:
- Lesson 1 (adversary read-only persistence) → P0.3
- Lesson 2 (default path doesn't match) → P1.1
- Lesson 3 (finding-ID collision) → P1.2
- Lesson 4 (filename collision) → P0.4
- Lesson 5 (policy registry absent) → P2.1
- Lesson 6 (invocation flag ambiguity) → P2.2
- Lesson 7 (scope parameter) → P1.3
- Policy 9 (VP-INDEX source of truth) → P0.1, P0.2

Proceed to baseline first, then P0 tier, checkpoint, then P1, checkpoint,
then P2.
