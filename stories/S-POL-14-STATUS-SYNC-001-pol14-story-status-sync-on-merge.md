---
document_type: story
story_id: "S-POL-14-STATUS-SYNC-001"
title: "POL-14 Story-Status Sync on Merge — Paired BC-Promotion + story/sprint/index status flip"
wave: maintenance
epic_id: maintenance
priority: P2
status: draft
version: "1.0"
level: ops
producer: story-writer
timestamp: "2026-05-31"
created: "2026-05-31"
modified: "2026-05-31"
tdd_mode: strict
track: "Platform Engineering"
subsystems: []
crates_touched: []
target_module: ".factory/ hooks + state-manager post-merge burst"
capabilities: []
behavioral_contracts: []
# BC status: pending PO authorship
verification_properties: []
depends_on: []
blocks: []
points: 3
estimated_days: 0.5
risk: MEDIUM
acceptance_criteria_count: 3
red_gate_tests: 2
estimated_passes: "2-3"
holdout_scenarios: []
assumption_validations: []
---

# S-POL-14-STATUS-SYNC-001: POL-14 Story-Status Sync on Merge

## §Origin — [process-gap] DRIFT-D916-001 / decision D-917

**Source gap:** DRIFT-D916-001 (2026-05-31), escalated to decision D-917.

When a story's PR squash-merges to `develop`, POL-14 (auto-promote BCs draft→active) fires
inside the state-manager post-merge burst. However, there is **no paired mechanism** that
flips the story's own `status:` field to `merged` at the same time. The gap was first
observed when S-CONFIG-MULTI-TENANT-OVERRIDE-001 had its PR #155 squash-merged but its
story file continued to carry `status: ready` for 5 days. That stale status propagated
into a resume snapshot and nearly triggered a redundant re-delivery of an already-shipped
story.

**Root cause chain:**

| Step | What happens | What was missing |
|------|--------------|-----------------|
| PR #NNN squash-merged to develop | State-manager fires POL-14 → BCs flip draft→active | Paired story `status: ready → merged` flip |
| STATE.md decision row recorded (D-NNN MERGED) | State-manager records the merge SHA | Story frontmatter `status:` NOT updated |
| Next session resume | STORY-INDEX and sprint-state.yaml show BCs active | Story still shows `status: ready` — looks undelivered |
| Orchestrator loads sprint-state | `wave-N active_dispatch_set` still includes the story | Risk of redundant dispatch |

**Two complementary fixes are required:**

1. **Proactive (atomic):** State-manager post-merge burst MUST flip story `status: ready → merged`
   at the same time POL-14 promotes BCs. Paired writes to: story frontmatter, sprint-state.yaml
   active_dispatch_set, and STORY-INDEX Full Story List row.

2. **Detective (drift validator):** A factory hook / validator that detects the asymmetric state
   "BCs active but story status still draft/ready" and flags it as a DRIFT-class finding so the
   next dispatch is blocked until state-manager resolves it.

---

## Narrative

As a factory-process operator, I want the story `status:` field to be set to `merged`
atomically with POL-14's BC promotion when a story PR squash-merges, so that STORY-INDEX,
sprint-state.yaml, and session-resume snapshots never show a delivered story as still
`draft`/`ready`.

---

## Acceptance Criteria

### AC-001: Atomic merge-time status flip
**Given** a story whose PR has been squash-merged to `develop` and whose `behavioral_contracts:`
list is non-empty,
**When** the state-manager post-merge burst runs POL-14 BC promotion,
**Then** the story's frontmatter `status:` field MUST be updated from `draft` or `ready` to
`merged` in the SAME atomic commit that promotes the BCs — and the corresponding row in
`STORY-INDEX.md` and `sprint-state.yaml` `active_dispatch_set` MUST reflect the merged status
in that same commit.

Red Gate test: `test_POL_14_story_status_flip_is_atomic_with_bc_promotion`
(validates: pre-commit hook or state-manager protocol doc asserts single-commit; test reads
story frontmatter + STORY-INDEX + sprint-state from the commit being authored and asserts all
three are updated before the commit seals.)

### AC-002: Drift detector flags asymmetric state
**Given** any story file under `.factory/stories/` where at least one BC in `behavioral_contracts:`
has `status: active` in `.factory/specs/behavioral-contracts/` (i.e., POL-14 has already
fired for it),
**When** the drift detector runs (invokable manually and as a pre-dispatch hook),
**Then** the detector MUST emit a DRIFT-class finding for every story where ANY of the following
is true:
- story `status:` is `draft` or `ready` AND at least one of its BCs is `status: active`
- story `status:` is `merged` but its STORY-INDEX row still shows `[draft]` or `[ready]`
- story `status:` is `merged` but it still appears in sprint-state.yaml `active_dispatch_set`

The finding MUST include: story ID, BC IDs that are active, current story status, and the
message "POL-14 BC promotion detected without paired story status flip — resolves DRIFT-D916-001."

Red Gate test: `test_drift_detector_catches_S_CONFIG_MULTI_TENANT_OVERRIDE_001_pattern`
(uses a fixture that replicates the S-CONFIG-MULTI-TENANT-OVERRIDE-001 state: BCs active,
story `status: ready`; asserts detector emits DRIFT finding with correct story ID and BC IDs.)

### AC-003: Regression — no false positives on correctly-merged stories
**Given** a story that has `status: merged` AND its BCs are `status: active` AND its
STORY-INDEX row shows `[merged PR #NNN ...]` AND it does NOT appear in sprint-state.yaml
`active_dispatch_set`,
**When** the drift detector runs,
**Then** the detector MUST NOT emit any finding for that story.

(Validates: the fix for DRIFT-D916-001 does not produce noise against the already-correctly
merged stories such as S-5.01-FOLLOWUP-MCP-BOOT, S-3.02-FOLLOWUP-RUNTIME, etc.)

---

## Architecture Mapping

| Component | Location | Pure/Effectful |
|-----------|----------|---------------|
| State-manager post-merge burst protocol | `.factory/SESSION-HANDOFF.md` §POL-14 + state-manager agent prompt | Pure (doc amendment) |
| Drift detector script / hook | `.factory/hooks/validate-pol14-story-status-sync.sh` (new file) | Pure (read-only scan, exit non-zero on drift) |
| STORY-INDEX row update | `.factory/stories/STORY-INDEX.md` (existing) | Effectful (state-manager write) |
| sprint-state.yaml active_dispatch_set update | `.factory/sprint-state.yaml` (existing) | Effectful (state-manager write) |
| Story frontmatter status flip | `.factory/stories/<story-id>-*.md` (per-story) | Effectful (state-manager write) |

---

## Tasks

- [ ] **Task 1 — Audit current post-merge burst protocol.** Read `.factory/SESSION-HANDOFF.md`
  §POL-14 and state-manager agent prompt. Identify the exact point where BC promotion is
  written and add "paired story status flip" as a mandatory step in the same paragraph.
  The protocol amendment must state: "In the same atomic commit: (a) promote BCs via POL-14,
  (b) set story frontmatter `status: merged`, (c) update STORY-INDEX Full Story List row from
  `[draft]`/`[ready]` to `[merged PR #NNN develop@SHA]`, (d) remove story from
  `sprint-state.yaml` `active_dispatch_set`."

- [ ] **Task 2 — Backfill S-CONFIG-MULTI-TENANT-OVERRIDE-001.** Story
  S-CONFIG-MULTI-TENANT-OVERRIDE-001 (PR #155) is the canonical DRIFT-D916-001 instance.
  Its story file currently has `status: ready`. Flip it to `status: merged` as part of this
  story's delivery commit. (No separate story needed — this is a 1-line fix.)

- [ ] **Task 3 — Write drift detector script.** Create
  `.factory/hooks/validate-pol14-story-status-sync.sh`. Algorithm:
  1. For each `.factory/stories/S-*.md` file: read `behavioral_contracts:` frontmatter array
     and `status:` field.
  2. For each BC in the array: check if `.factory/specs/behavioral-contracts/<BC-ID>.md`
     contains `status: active`.
  3. If ANY BC is active AND story `status:` is `draft` or `ready`: emit DRIFT finding to
     stdout and set exit code 1.
  4. If story `status:` is `merged`: check STORY-INDEX for `[draft]`/`[ready]` row (grep).
     If found: emit DRIFT finding. Check sprint-state.yaml `active_dispatch_set` for story
     ID. If found: emit DRIFT finding.
  Exit code 0 only when zero drift findings.

- [ ] **Task 4 — Register detector as a pre-dispatch hook.** Add
  `validate-pol14-story-status-sync` as an entry in the hooks registry so it runs before
  any `deliver-story` or `start-work` dispatch. Hook must run in read-only mode (no writes).

- [ ] **Task 5 — Write Red Gate tests.** Two tests per AC-001 and AC-002 (see AC red_gate_test
  names above). Tests use YAML fixture files under `.factory/hooks/tests/fixtures/` to
  replicate real-world states (no live `.factory/` mutation in tests).

- [ ] **Task 6 — Verify no false positives.** Run detector against current `.factory/`
  state after Task 2 backfill. Assert zero findings. Document result in AC-003 closure note.

---

## Previous Story Intelligence

No predecessor self-improvement story covers the POL-14 + story-status pairing specifically.

Related stories and their lessons:

- **S-POL-29-CANONICAL-TEMPLATE-REGISTRY-001** (maintenance wave, P2): Established the pattern
  of writing a registry/protocol doc amendment + a mechanized validator as paired outputs.
  This story follows the same dual-output pattern: (a) protocol amendment in SESSION-HANDOFF.md,
  (b) drift detector script.

- **S-MAINT-POL29-HOOK-001**: Downstream hook story for S-POL-29. This story does NOT need a
  separate downstream hook story because the drift detector IS the hook — it is both the
  specification and the implementation.

- **TD-VSDD-053 single-commit-per-burst**: The atomicity requirement in AC-001 mirrors the
  same discipline. The state-manager must NOT split the BC promotion and the story status flip
  across two commits.

---

## Architecture Compliance Rules

Extracted from `.factory/specs/architecture/` and ADRs; enforced by adversarial review:

1. **TD-VSDD-053 (single-commit-per-burst):** The protocol amendment (Task 1) mandates that
   BC promotion + story status flip + STORY-INDEX update + sprint-state update occur in ONE
   commit. Two consecutive commits with "backfill" in the subject are blocked by the
   MULTI_COMMIT_CHAIN_NOT_ALLOWED detector. This is a hard constraint, not a preference.

2. **TD-FACTORY-HOOK-BYPASS-001 (no bypass of Write tool):** The drift detector script must be
   written via the Edit/Write tool path only. Python/sed/echo bypass is P0 forbidden.

3. **Read-only hook invariant:** The detector runs as a pre-dispatch read-only gate. It MUST NOT
   write any `.factory/` file. It only reads and emits findings to stdout + exit code.

4. **POL-14 scope:** POL-14 applies only to stories whose `behavioral_contracts:` is non-empty.
   Stories with `behavioral_contracts: []` (e.g., infrastructure/devops stories) are EXEMPT
   from the drift check. The detector must skip them without emitting false positives (AC-003).

5. **State-manager ownership:** Only state-manager may write story frontmatter `status:` fields.
   This story does NOT grant implementer or any other agent permission to flip story statuses
   directly. The protocol amendment (Task 1) is a doc change to state-manager's operating
   procedure, not a code change to any agent's tool access.

---

## Library & Framework Requirements

This story touches only shell scripts and YAML/Markdown factory artifacts. No Rust crates,
no external library versions required.

- Shell: bash 5.x (macOS ships bash 3.x; use `/usr/bin/env bash` with `set -euo pipefail`).
  Prefer POSIX-compatible constructs to avoid bash-version drift between macOS and Linux CI.
- YAML parsing in the hook script: use `grep` + `sed` for frontmatter extraction (no `yq`
  dependency; not guaranteed in all environments). A Python one-liner (`python3 -c`) is
  acceptable as a fallback if grep-based YAML parsing is insufficient for nested arrays.
- No new Cargo dependencies introduced.

---

## File Structure Requirements

| File | Action | Notes |
|------|--------|-------|
| `.factory/hooks/validate-pol14-story-status-sync.sh` | CREATE | Drift detector script; chmod +x |
| `.factory/hooks/tests/fixtures/drift-story-bcs-active-status-ready.yaml` | CREATE | Fixture for AC-002 red gate test |
| `.factory/hooks/tests/fixtures/drift-story-merged-no-index-update.yaml` | CREATE | Fixture for AC-002 second case |
| `.factory/hooks/tests/test_pol14_drift_detector.sh` | CREATE | Test harness for AC-001 + AC-002 + AC-003 |
| `.factory/SESSION-HANDOFF.md` | MODIFY | Add "paired story status flip" to POL-14 post-merge burst protocol |
| `.factory/stories/S-CONFIG-MULTI-TENANT-OVERRIDE-001-*.md` | MODIFY | Backfill: flip `status: ready` → `status: merged` (Task 2) |
| `.factory/stories/STORY-INDEX.md` | MODIFY (via state-manager) | Register this story row; state-manager handles in separate burst |
| `.factory/sprint-state.yaml` | MODIFY (via state-manager) | Add to maintenance wave; state-manager handles in separate burst |

---

## Token Budget Estimate

| Artifact | Estimated Tokens |
|----------|-----------------|
| This story spec | ~3,500 |
| SESSION-HANDOFF.md (read for Task 1) | ~8,000 |
| S-CONFIG-MULTI-TENANT-OVERRIDE-001 story (read for Task 2 backfill) | ~5,000 |
| BC files for active-status check examples (2-3 BCs) | ~1,500 |
| Hook script to write (.factory/hooks/validate-pol14-story-status-sync.sh) | ~800 |
| Test fixture YAMLs (2 files) | ~400 |
| Test harness script (.factory/hooks/tests/test_pol14_drift_detector.sh) | ~800 |
| sprint-state.yaml read (Task 6 verification) | ~2,000 |
| **Total** | **~22,000 tokens** |

Well within the 20-30% agent context budget (200k context → 40k-60k budget).
This story is appropriate for a single dispatch.

---

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|------------------|
| EC-001 | Story has `behavioral_contracts: []` (e.g., S-MAINT-001) and `status: draft` | Detector skips — no BCs means POL-14 never fires; NOT a drift finding |
| EC-002 | Story has one BC active and one BC still `status: draft` | Drift detected: at least one BC active → story must be `merged` already or the BC was promoted incorrectly |
| EC-003 | STORY-INDEX row shows `[merged PR #NNN]` but story frontmatter still `status: ready` | Drift detected: STORY-INDEX and story frontmatter are out of sync (reverse of the canonical case) |
| EC-004 | Story `status: merged` and all BCs active AND story absent from sprint-state active_dispatch_set AND STORY-INDEX row correct | No finding — clean state, AC-003 passes |
| EC-005 | Story has `bcs:` alias instead of `behavioral_contracts:` | Detector must recognize both field names per BC Array Propagation Policy alias rule |
| EC-006 | Detector run on a repo where `.factory/hooks/` does not yet exist | Script creates no files; exits 1 with "hooks directory not found" message; does not silently succeed |

---

## Dependencies

- `depends_on: []` — No product story dependencies. This is a pure factory-process story.
- `blocks: []` — No downstream stories blocked. The STORY-INDEX registration and sprint-state
  entry are handled by state-manager in its post-registration burst (separate from this story
  file creation).

---

## Complexity & Points

**3 story points.** Rationale:

| Component | Sub-estimate |
|-----------|-------------|
| SESSION-HANDOFF.md protocol amendment (Task 1) | 0.5 pts |
| S-CONFIG backfill (Task 2) | 0.5 pts |
| Drift detector script (Task 3) | 1 pt |
| Hook registration (Task 4) | 0.5 pts |
| Red Gate tests + fixture files (Task 5 + Task 6) | 0.5 pts |
| **Total** | **3 pts** |

No algorithm complexity (shell scripting + YAML grep). Scope is well-bounded. Points are
calibrated to the same scale as S-POL-29-CANONICAL-TEMPLATE-REGISTRY-001 (2 pts for doc
amendment only). This story adds a working script + tests, justifying +1 pt.
