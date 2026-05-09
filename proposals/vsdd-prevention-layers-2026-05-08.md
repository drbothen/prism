---
artifact_type: proposal
target_repo: vsdd-factory
target_repo_url: https://github.com/drbothen/vsdd-factory
date: 2026-05-08
revision: 3
revision_history:
  - r1 (2026-05-08): initial — Plan 1 (3-layer model) + Plan 2 (YAML+ripgrep substrate)
  - r2 (2026-05-08): added Plan 3 (WASM extension substrate, day-1 commitment); updated build sequencing; added Q6-Q10
  - r3 (2026-05-08): ground-truth recalibration against actual vsdd-factory repo inspection. WASM substrate already shipped (factory-dispatcher + 17 native-WASM plugins + 33 bash shims in production); Component Model promoted to primary ABI; Layer 1 split into four substrates (1A CC hooks, 1B git hooks, 2 agent invariants, 3 scheduled audits); state-manager BC-promotion replaced by SubagentStop hook; POL IDs renumbered starting at POL-13 (POL-12 is taken by vsdd-factory's own bc_tv_emitter_consistency); vendor-on-init pattern specified for lefthook bash distribution; existing-analog references added for every deliverable; technology claims corrected from research findings.
status: draft-for-export
self_contained: true
prior_context_required: false
inputs:
  - .factory/proposals/vsdd-prevention-layers-2026-05-08.md  (r2, replaced by this document)
  - .factory/proposals/r3-research-findings.md               (ground truth; read this to understand all factual corrections)
  - .factory/proposals/vsdd-stub-merge-policy-2026-05-08.md  (companion schema proposal; reference, do not duplicate)
companion_proposal: vsdd-stub-merge-policy-2026-05-08.md
prism_audit_reference: ../cycles/wave-4-operations/workspace-audit-2026-05-08.md
---

# VSDD Prevention Layers — Stub-Merge / Status-Drift / Runtime-Gap Defense

**For:** A Claude Code session in the `vsdd-factory` plugin repo at
`/Users/jmagady/Dev/vsdd-factory/` (or equivalent), with **no prior context** about
the Prism project or the audit that motivated this proposal.

**Critical framing:** vsdd-factory is an architecturally mature plugin. It already ships
a WASM dispatcher (`crates/factory-dispatcher/`), 17 native-WASM hook plugins, 33
bash hook scripts running through `legacy-bash-adapter.wasm`, a 992-line
`hooks-registry.toml`, 36 agents, 121 skills, and 9 workflows at version `1.0.0-rc.11`.
**The WASM extension substrate r2 proposed "building in Phase 3" is already in
production.** This proposal adds new hooks, rules, and skills to the existing substrate.
It does not build new infrastructure.

**You are receiving:** three plans. Plan 1 is the four-substrate prevention model,
grounded against vsdd-factory's actual architecture, ready to land as v-next features.
Plan 2 is the YAML+ripgrep substrate for language-agnostic syntactic checks — your
contribution is new YAML rows, not a new substrate (the substrate already exists via the
33 bash hooks). Plan 3 describes how new WASM hook plugins are authored and landed in the
existing WASM extension substrate, including the Component Model path.

---

## TL;DR

A workspace audit found 53 findings (18 P0) in a VSDD-managed Rust project. Stories were
marked `merged` while production code paths were still `todo!()` panics. Tests were
silent-shallow. At least one test was inverted-polarity — green CI required production code
to stay broken.

The convergence ceremony (3-CLEAN adversarial passes) did not catch this. The adversary
converged on what was inside the perimeter shown to it. Nobody checked whether the perimeter
was right.

**Three plans, building on vsdd-factory's existing production substrate:**

- **Plan 1** adds the four-substrate prevention model to the existing plugin (~1 plugin week).
  The WASM dispatcher, hook registry, agent files, and skill system are already there.
  New deliverables are new entries, not new infrastructure.
- **Plan 2** adds YAML rows for the existing YAML+ripgrep substrate to cover stub-detection
  patterns. Adding a language = adding a YAML row; the hook runner is a new bash script.
- **Plan 3** describes how new WASM hook plugins are authored for the existing
  `factory-dispatcher` host, using the Component Model as the primary ABI.

Build order: **Plan 1 first** (proves the model, lands fast); **Plan 2 in parallel**
(cheap multi-language syntactic coverage); **Plan 3 as the deep-structural tier**
(tree-sitter AST checks via the existing WASM substrate).

---

## Background — The Failure Case

A VSDD-managed Rust workspace, 24 crates, ~145 verification properties, ~222 behavioral
contracts, ~50+ stories, late Phase 3.

**What the project's STORY-INDEX claimed:**
- S-3.02 (query engine): MERGED PR #129
- S-3.07 (write engine): MERGED PR #135
- S-1.11/12/14/15 (plugin/spec-engine, hot-reload, infusions): MERGED

**What a fresh-context audit found:**
- `QueryEngine::execute`, `run_materialization_pipeline`, `RocksDbTableProvider` — all
  `todo!()` panics. 5 sites in 3 files.
- `WriteExecutor::execute` Phase 3 fetch was hardcoded `vec![]`. Default `SensorAdapter::write()`
  returned `WriteNotImplemented`. SQL DML returned `NotImplemented`.
- `prism-mcp` (the entire MCP server crate, mapped to an accepted ADR claiming 35 tools):
  10 lines of placeholder code.
- 222/222 behavioral contracts were `status: draft`. Only 2/145 verification properties were
  `verified`. Promotion criteria undefined.
- One test in `prism-spec-engine/tests/hot_reload_tests.rs` actively codified the stub:
  `#[should_panic(expected = "not yet implemented")]`. That test would BREAK the day someone
  implemented the real function.
- 4 sensor TOML files were write-only (read-side `[[tables]]` never delivered). No production
  binary called `parse_spec_directory`, `ConfigManager::new`, `register_internal_tables`, or
  `WriteEndpointRegistry::new`. The runtime was unwired even though every individual story
  passed adversarial review.
- STORY-INDEX rows said MERGED while the same stories' frontmatter said `status: ready` or
  `status: draft`. Two systems of truth, no reconciliation hook.

**Root cause taxonomy (5 patterns):**
1. Stub-merge convention enforced inconsistently.
2. STORY-INDEX as a manual log instead of a status oracle.
3. No graduation contract on BC/VP status — 222/222 stayed draft.
4. Production wiring debt — no binary loaded the spec/config/registry entry points the
   architecture documented.
5. Doc-comment freeze after stub — `/// STUB — todo!()` doc-comments persisted long after code
   was real.

**The convergence ceremony missed all of it.** Adversarial review with 3-CLEAN passes
converged on the perimeter it was shown — internal consistency of a story's BCs and ACs.
Cross-cutting questions like "does any binary actually call this?" and "is the STORY-INDEX
status consistent with the story-file frontmatter?" were never asked.

---

## Plan 1 — Four-Substrate Prevention Model

r2 called this "Three-Layer Prevention Model" and left Layer 1 muddled. r3 splits Layer 1
into two distinct substrates with different latency, trust, and deployment models.

Each substrate catches a different latency class. Single-substrate defenses leak.
Defense-in-depth is mandatory.

### Substrate 1A: Claude Code Hooks (AI-action-time, ms latency)

**Fires on:** Claude tool calls (Edit, Write, Bash, Agent). Synchronous — can block. The
vsdd-factory plugin already has 40+ hooks in this substrate via `hooks-registry.toml`.

**How new hooks land:** add an entry to `hooks-registry.toml` (with capability block) plus
either a new `hooks/<name>.sh` (via `legacy-bash-adapter.wasm`) or a new Rust crate at
`crates/hook-plugins/<name>/` compiled to `.wasm`.

**Event types beyond PreToolUse/PostToolUse (from CC docs, all registered through
`factory-dispatcher`):**

| Event | Current vsdd-factory usage | r3 relevance |
|---|---|---|
| `PreToolUse` on Edit/Write | 8+ existing hooks | stub-residue at edit time, inverted-polarity at edit time |
| `PostToolUse` on Edit/Write | 10+ existing hooks | story-index coherence check |
| `SubagentStop` | 5 existing hooks (e.g. `validate-per-story-adversary-convergence.wasm`, `update-wave-state-on-merge.wasm`) | BC/VP promotion on pr-manager subagent completion |
| `SubagentStart` | 2 existing hooks | story-completion gates |
| `Stop` | `session-learning.wasm`, `warn-pending-wave-gate.wasm` | post-turn audits |
| `FileChanged` | Not currently used | stub-residue on manual out-of-Claude edits (future) |

**r3 deliverables for Substrate 1A:**

| Hook | Event | Tool matcher | Substrate | Behavior | Existing analog |
|---|---|---|---|---|---|
| `validate-stub-residue.sh` | `PreToolUse` | `Edit\|Write` on `crates/*/src/**` | legacy-bash-adapter | Greps for `todo!()`, `unimplemented!()`, `panic!("TODO")` in production file being written; non-zero match blocks edit | EXTEND pattern from `red-gate.sh` (different function; red-gate blocks premature implementation, this blocks stub commit); GREENFIELD implementation |
| `validate-story-frontmatter-index-coherence.sh` | `PostToolUse` | `Edit\|Write` on `stories/*.md` or `STORY-INDEX.md` | legacy-bash-adapter | Cross-checks story-file frontmatter `status:` against STORY-INDEX row; fail if they disagree | EXTEND (sibling to) `validate-state-index-status-coherence.sh` (which covers STATE.md ↔ cycles/INDEX.md; this covers story frontmatter ↔ STORY-INDEX) |
| `validate-inverted-polarity.sh` | `PreToolUse` | `Edit\|Write` on `tests/**/*.rs` | legacy-bash-adapter | Bans `#[should_panic(expected = ".*not yet implemented.*\|.*TODO.*\|.*stub.*")]` outside a declared RED-gate phase window | EXTEND pattern from `validate-red-ratio.sh` (both are red-discipline checks; that one enforces RED_RATIO frontmatter, this one bans polarity inversion) |
| `update-bc-status-on-story-merge.sh` | `SubagentStop` | match pr-manager agent | legacy-bash-adapter (or native WASM sibling to `update-wave-state-on-merge.wasm`) | When pr-manager subagent completes a merge, auto-promotes BCs in the story's `behavioral_contracts:` array from `draft → active` if story reaches `merged`; skips if `partial-merge` | EXTEND (sibling to) `update-wave-state-on-merge.wasm` (which updates wave state on merge; this updates BC status on the same event) |

**Note on `update-bc-status-on-story-merge.sh` and state-manager:** r2 placed BC
promotion in the state-manager agent prompt. `state-manager.md` explicitly says
"NEVER write specification documents or source code — state tracking only." Auto-promoting
BC `status: draft → active` is writing spec content. **r3 corrects this: BC promotion is a
hook-driven operation on `SubagentStop`, with no agent involvement.** The hook fires when
pr-manager completes, reads the merged story's `behavioral_contracts:` array, and writes
`status: active` to the relevant BC frontmatter. This matches the exact pattern of
`update-wave-state-on-merge.wasm`.

**Capability blocks for new hooks (required in `hooks-registry.toml`):**

```toml
[[hooks]]
name = "validate-stub-residue"
event = "PreToolUse"
tool = "Edit|Write"
if = "Edit(crates/*/src/**/*.rs) | Write(crates/*/src/**/*.rs)"
plugin = "hook-plugins/legacy-bash-adapter.wasm"
priority = 140
timeout_ms = 3000
on_error = "block"

[hooks.capabilities.exec_subprocess]
binary_allow = ["rg", "git", "bash", "jq"]
env_allow = ["PATH", "HOME", "CLAUDE_PROJECT_DIR", "CLAUDE_PLUGIN_ROOT", "TMPDIR"]

[hooks.capabilities.read_file]
path_allow = ["crates/**/*.rs"]
```

```toml
[[hooks]]
name = "validate-story-frontmatter-index-coherence"
event = "PostToolUse"
tool = "Edit|Write"
if = "Edit(.factory/stories/**) | Write(.factory/stories/**)"
plugin = "hook-plugins/legacy-bash-adapter.wasm"
priority = 130
timeout_ms = 5000
on_error = "block"

[hooks.capabilities.read_file]
path_allow = [".factory/stories/**/*.md"]
```

```toml
[[hooks]]
name = "validate-inverted-polarity"
event = "PreToolUse"
tool = "Edit|Write"
if = "Edit(tests/**/*.rs) | Write(tests/**/*.rs) | Edit(crates/*/tests/**/*.rs)"
plugin = "hook-plugins/legacy-bash-adapter.wasm"
priority = 145
timeout_ms = 3000
on_error = "block"

[hooks.capabilities.exec_subprocess]
binary_allow = ["rg", "bash"]
env_allow = ["PATH", "CLAUDE_PROJECT_DIR"]

[hooks.capabilities.read_file]
path_allow = ["tests/**/*.rs", "crates/*/tests/**/*.rs", "crates/*/src/**/*_test.rs"]
```

```toml
[[hooks]]
name = "update-bc-status-on-story-merge"
event = "SubagentStop"
plugin = "hook-plugins/legacy-bash-adapter.wasm"
priority = 90
timeout_ms = 10000
on_error = "continue"

[hooks.capabilities.read_file]
path_allow = [".factory/stories/**/*.md", ".factory/specs/behavioral-contracts/**/*.md"]

[hooks.capabilities.write_file]
path_allow = [".factory/specs/behavioral-contracts/**/*.md"]
max_bytes_per_call = 65536
```

### Substrate 1B: Git Hooks (commit/push-time, ms latency)

**Fires on:** `git commit` and `git push`, via lefthook (or raw git hooks for non-lefthook
projects). These run outside Claude Code — `${CLAUDE_PLUGIN_ROOT}` is NOT set in the git
commit shell.

**Distribution snag (addressed):** Plugin-shipped bash scripts at
`${CLAUDE_PLUGIN_ROOT}/hooks/<name>.sh` work correctly for Claude Code hooks (CC sets the
env var). They do NOT work for lefthook-invoked git hooks because `git commit` runs outside
Claude Code and `CLAUDE_PLUGIN_ROOT` is unset.

**Resolution — vendor-on-init pattern:** The plugin ships a `vsdd-factory:install-hooks`
skill (or a step in the existing `setup-env` skill) that copies the relevant hook scripts
from the plugin root into the project's `.factory/hooks/<name>.sh`. The project's
`lefthook.yml` then references the project-relative path. This matches the existing pattern
for `verify-sha-currency.sh` (per `state-manager.md`, which instantiates it from
`templates/verify-sha-currency.sh` into `.factory/hooks/verify-sha-currency.sh`).

**Prism's existing lefthook.yml pattern** (from `/Users/jmagady/Dev/prism/lefthook.yml`):
scripts reference project-local paths and `just` recipes — no plugin-path references. r3's
scripts must conform to this pattern after vendoring.

**r3 deliverables for Substrate 1B:**

| Hook | lefthook event | Behavior | Distribution path |
|---|---|---|---|
| `stub-residue-check.sh` | `pre-push` | Workspace-wide ripgrep scan of `crates/*/src/` for stub markers on changed files; non-zero exit blocks push | Vendored via `vsdd-factory:install-hooks` into `.factory/hooks/stub-residue-check.sh`; lefthook.yml entry added by install-hooks skill |
| `inverted-polarity-check.sh` | `pre-push` | Scans test files for `#[should_panic(...)]` matching stub idioms | Same vendor pattern |

Note: Substrate 1B catches the same syntactic patterns as Substrate 1A but is the fallback
layer for developer workflow outside Claude (e.g., direct `git push` from a terminal). Both
substrates are necessary; they are not redundant.

### Substrate 2: Agent-Prompt Invariants (per-dispatch, minute latency)

**Fires on:** every invocation of the affected agent. Cannot see topology (no workspace
scope). Sees semantics (meaning, not just syntax).

**Agents with r3 additions (all under `plugins/vsdd-factory/agents/`):**

**`implementer.md` — add Graduation Contract section:**

Every story completion requires a callgraph assertion: every public entry point named in the
story's `behavioral_contracts:` array must be reachable from the crate's public API without
hitting a `todo!()` panic. If any panic site exists in production code, the story status MUST
be `partial-merge`, never `merged`. This is not optional; it is the definition of "done".

Include specific check procedure:
```bash
# Per-file check the implementer runs before marking merged:
rg 'todo!\(\|unimplemented!\(' crates/*/src/ --glob '!tests/**'
# Zero output required for status: merged
```

**`adversary.md` — add production-stub-residue as a mandatory perimeter dimension:**

The adversary already has a Three-Perimeter Scope Contract (per ADR-017 in the
21KB adversary.md) covering per-story, per-wave, and phase-wide perimeters. Add a new
mandatory check to the workspace-wide (phase-wide) perimeter:

"For every story claiming `status: merged`, grep the story's delivery files for stub
markers. Production-path stub residue present = P0 finding regardless of convergence
status. This check cannot be skipped on convergence."

**`test-writer.md` — extend via POL-11 rather than adding a new check:**

`validate-consistency` Check 8 (POL-11 `no_test_tautologies`) and its fixture corpus
at `skills/validate-consistency/fixtures/tautology/` already cover the silent-shallow
test pattern. r3 adds a self-check reminder to `test-writer.md` that explicitly calls out
POL-11 as the governing policy, and adds the production-SUT-invocation requirement as
a self-check the test-writer runs before submitting test PRs.

**`state-manager.md` — NO BC promotion added here.**

As discussed in Substrate 1A: state-manager is strictly bookkeeper. BC promotion is
delegated to the `update-bc-status-on-story-merge.sh` hook (Substrate 1A, SubagentStop).

**`stub-architect.md` (already exists, 7.4KB) — reconcile before adding new rules:**

Before writing any new implementer/adversary rules, read `stub-architect.md`. This agent
is already focused on stubs. Any implementer Graduation Contract language should be
consistent with existing stub-architect rules; if there is overlap, consolidate in
stub-architect and reference from implementer.

### Substrate 3: Scheduled Audit Skills (cron-invoked, day/week latency)

**Scheduling clarification:** Claude Code skills do NOT have a `schedule:` field. Skills
are invoked by the model (context match on `description:`) or manually via
`/vsdd-factory:<name>`. The "scheduling" layer is external:
- GitHub Actions cron (`cron: '0 2 * * 0'` weekly)
- System cron running `claude --plugin-dir ... -p "/vsdd-factory:audit-stub-debt"`
- Wave-gate skill explicitly invoking audit skills as a gate step
- Manual human invocation pre-wave

These audit skills run at workspace scope, which is why they cannot be PR-gated.

**r3 deliverables for Substrate 3:**

| Skill | New vs. Existing | Behavior |
|---|---|---|
| `audit-stub-debt/` | GREENFIELD (no top-level existing analog) — but should be invocable as Sweep 4 from `maintenance-sweep` | Workspace-wide ripgrep scan for stub markers in all `status: merged` stories' delivery files + all `status: partial-merge` stories for graduation contract completeness. Emits structured findings report. Does NOT auto-file TDs (user must confirm) |
| `audit-runtime-wiring/` | GREENFIELD | Per accepted ADR with crate-level deliverables: verify at least one `bin/` target reaches the declared entry point. Uses `cargo metadata` for Rust; language-agnostic shims for others. Flags if no binary calls the entry point |
| `audit-orphan-configs/` | GREENFIELD | Every TOML/YAML/JSON config outside `Cargo.toml`/test fixtures must have at least one production loader. Reverse-grep approach |
| `audit-vp-promotion/` | GREENFIELD — use `compute-input-hash` binary pattern from `check-input-drift` skill | Flags any VP `status: draft` more than 30 days old where its anchor story has `status: merged` |

**Composition with `maintenance-sweep`:** Each audit skill is independently invokable.
`maintenance-sweep` (which already has Sweep 1/2/3 for deps, doc-drift, pattern-consistency)
should call these as Sweep 4/5/6/7. The skills remain top-level (per r2) for granular invocation;
maintenance-sweep composes them.

### Why all four substrates stacked

Each substrate's miss space is the next substrate's catch space:

- **1A misses** silent-shallow tests (compiles, imports production code, but exercises a mock
  that hard-codes the expected return). → Substrate 2 catches.
- **1B misses** "no binary calls this" because lefthook has no workspace topology view. → Substrate 3 catches.
- **Substrate 2 misses** "no binary calls this" because no individual agent has visibility
  outside its own story context. → Substrate 3 catches.
- **Substrate 3 misses** in-edit drift (runs on cadence, not on commit). → Substrates 1A/1B catch.

The Prism failure was a **4-of-4 substrate absence**. Adversarial review (a partial Substrate 2)
was the only defense, and it correctly converged on what it was shown.

---

## Per-Check Placement Table

| Check | Substrate(s) | Existing analog to extend | Greenfield? |
|---|---|---|---|
| `check-stub-residue` | 1A (PreToolUse on Edit/Write to `crates/*/src/**`) + 1B (lefthook pre-push, vendored) | Pattern from `red-gate.sh`; unrelated function but same gate pattern | GREENFIELD impl; EXTEND hook-registry pattern |
| `check-inverted-polarity` | 1A (PreToolUse on Edit/Write to `tests/**/*.rs`) + 1B (lefthook pre-push, vendored) | EXTEND pattern from `validate-red-ratio.sh` (sibling red-discipline check) | GREENFIELD impl |
| `check-story-index-consistency` | 1A (PostToolUse on Edit/Write to `stories/**`) | EXTEND (sibling to) `validate-state-index-status-coherence.sh` (that covers STATE.md ↔ cycles/INDEX.md; this covers story frontmatter ↔ STORY-INDEX) | GREENFIELD impl |
| `check-bc-promotion` | 1A (SubagentStop matching pr-manager subagent) | EXTEND (sibling to) `update-wave-state-on-merge.wasm` (same event, different state write target) | GREENFIELD impl |
| `audit-runtime-wiring` | 3 (scheduled skill, also wave-gate-invokable) | No existing analog found | GREENFIELD |
| `audit-stub-debt` | 3 (scheduled) + sub-sweep of maintenance-sweep | Compose with existing maintenance-sweep Sweep 1/2/3 | GREENFIELD |
| `audit-orphan-configs` | 3 (scheduled) | No existing analog found | GREENFIELD |
| `audit-vp-promotion` | 3 (scheduled) | Reuse `compute-input-hash` binary pattern from `check-input-drift` skill | GREENFIELD skill; EXTEND binary pattern |

---

## Concrete Deliverables for vsdd-factory Plugin

All paths are relative to `plugins/vsdd-factory/` unless noted.

### Hooks (Substrate 1A)

| File | Extend or Greenfield | Effort |
|---|---|---|
| `hooks/validate-stub-residue.sh` | GREENFIELD; register in `hooks-registry.toml` | 0.5d |
| `hooks/validate-story-frontmatter-index-coherence.sh` | GREENFIELD (sibling to existing `hooks/validate-state-index-status-coherence.sh`); register in registry | 0.5d |
| `hooks/validate-inverted-polarity.sh` | GREENFIELD; register in registry | 0.5d |
| `hooks/update-bc-status-on-story-merge.sh` | GREENFIELD (sibling to `hook-plugins/update-wave-state-on-merge.wasm`; can start as bash, migrate to native WASM later); register with SubagentStop event | 1d |
| `hooks-registry.toml` additions | EXTEND: add 4 new entries with capability blocks (spec'd above) | included in above |

### Vendor-On-Init Skill Patch (Substrate 1B)

| File | Extend or Greenfield | Effort |
|---|---|---|
| `skills/setup-env/SKILL.md` OR new `skills/install-hooks/SKILL.md` | EXTEND setup-env or GREENFIELD if install-hooks is too different in scope | 0.5d |
| `hooks/stub-residue-check.sh` (vendored version) | Copy of validate-stub-residue.sh adapted for lefthook context (no CC env vars, uses `git diff --name-only origin/HEAD`) | 0.25d |
| `hooks/inverted-polarity-check.sh` (vendored version) | Same pattern | 0.25d |
| Template lefthook.yml snippet for consuming projects | New in `templates/lefthook-stub-checks.yml.template` | 0.25d |

### Agent Patches (Substrate 2)

| File | Extend or Greenfield | Effort |
|---|---|---|
| `agents/implementer.md` | EXTEND: add Graduation Contract section | 0.5d |
| `agents/adversary.md` | EXTEND: add production-stub-residue check as mandatory Phase 5 workspace-wide perimeter dimension | 0.5d |
| `agents/test-writer.md` | EXTEND: add POL-11 self-check reminder + production-SUT-invocation requirement | 0.25d |
| `agents/stub-architect.md` | READ FIRST, then RECONCILE with implementer changes | 0.25d |

### Scheduled Audit Skills (Substrate 3)

| File | Extend or Greenfield | Effort |
|---|---|---|
| `skills/audit-stub-debt/SKILL.md` | GREENFIELD; also wire into `skills/maintenance-sweep/SKILL.md` as Sweep 4 | 0.5d |
| `skills/audit-runtime-wiring/SKILL.md` | GREENFIELD | 0.5d |
| `skills/audit-orphan-configs/SKILL.md` | GREENFIELD | 0.5d |
| `skills/audit-vp-promotion/SKILL.md` | GREENFIELD; reuse `compute-input-hash` binary pattern | 0.5d |
| `skills/maintenance-sweep/SKILL.md` | EXTEND: add Sweep 4-7 that invoke the new audit skills | 0.25d |

### Policy Registry Additions

| ID | Name | Severity | Notes |
|---|---|---|---|
| POL-13 | `production_stub_residue_blocks_merge` | HIGH | `status: merged` forbidden when production-path stub markers exist. Enforced by `validate-stub-residue.sh` (1A) + adversary mandatory check (2) + `audit-stub-debt` (3) |
| POL-14 | `story_frontmatter_index_consistency` | HIGH | Story frontmatter `status:` must match STORY-INDEX row. Enforced by `validate-story-frontmatter-index-coherence.sh` |
| POL-15 | `bc_vp_promotion_on_anchor_merge` | HIGH | BCs in a merged story auto-promote `draft → active` via SubagentStop hook. No manual promotion required; no manual blocking either |
| POL-16 | `runtime_wiring_required_for_accepted_adrs` | MEDIUM | Every accepted ADR with crate-level deliverables must be reachable from at least one `bin/` target. Enforced by `audit-runtime-wiring` skill |
| POL-17 | `no_inverted_polarity_tests_outside_red_gate` | HIGH | Ban `#[should_panic(...)]` matching stub idioms except within declared RED-gate phase windows. Enforced by `validate-inverted-polarity.sh` |

**POL numbering rationale:** vsdd-factory's own internal `.factory/policies.yaml` has
POL-12 = `bc_tv_emitter_consistency` (from validate-consistency Check 9). The plugin does
not currently ship a baseline policies.yaml for consuming projects (policies are
project-scoped). However, POL-12 as a policy name in a consuming project could still collide
with an internally-consistent policy numbering scheme at that project. **r3 proposes starting
at POL-13 for the five new policies**, treating this as the "next safe slot" for any project
that follows the vsdd-factory baseline numbering through POL-12.

**Implementation approach for the policy registry:** The plugin does not currently ship a
baseline `policies.yaml`. Two options:
- (A) Have `policy-registry init` write the new policies (POL-13..17) into the project's
  `policies.yaml` alongside the existing baseline (POL-1..12).
- (B) Ship a `templates/policies-baseline.yaml` that includes POL-1..17 as the new
  baseline; init copies it.

Option B is cleaner for new projects; Option A is safer for existing projects. r3 recommends
Option A with an explicit migration note for existing projects.

**Schema additions (companion proposal handles this):**
- `partial-merge` story status enum — see `vsdd-stub-merge-policy-2026-05-08.md`.
- BC/VP `draft → active → verified` transition definitions — see companion proposal.

**Total estimated effort:** 6-8 plugin-engineer-days.

---

## Plan 2 — Language-Agnostic YAML+ripgrep Substrate

### Goal

Plan 1 is Rust-anchored. Plan 2 makes the syntactic checks language-agnostic via a YAML
config consumed by one hook script. Adding Ruby = adding a YAML row, no plugin code change.

**The existing 33 bash hooks** at `hooks/*.sh` already establish the bash substrate. Plan 2's
contribution is adding stub-detection-specific YAML config and a new hook script that consumes
it — not building a new substrate.

### Observation: stub-residue is a small, finite pattern set per language

| Language | Stub idioms | Inverted-polarity test idioms |
|---|---|---|
| Rust | `todo!()`, `unimplemented!()`, `panic!("TODO...")` | `#[should_panic(expected = "not yet implemented")]` |
| Python | `raise NotImplementedError`, `...` ellipsis bodies | `pytest.raises(NotImplementedError)` |
| TypeScript / JavaScript | `throw new Error("not implemented")` | `expect(...).toThrow("not implemented")` |
| Go | `panic("not implemented")`, `panic("TODO")` | `assert.PanicsWithValue(t, ..., "not implemented")` |
| Java / Kotlin | `throw new UnsupportedOperationException()`, `TODO()` | `assertThrows(UnsupportedOperationException.class, ...)` |
| C# | `throw new NotImplementedException()` | `Assert.Throws<NotImplementedException>` |
| Ruby | `raise NotImplementedError` | `expect { ... }.to raise_error(NotImplementedError)` |
| Swift | `fatalError("not implemented")`, `preconditionFailure(...)` | `XCTAssertThrowsError(... "not implemented")` |
| Go | `panic("not implemented")`, `panic("TODO")` | `assert.PanicsWithValue(...)` |
| C / C++ | `assert(0 && "not implemented")`, `abort()` with stub comment | test-framework specific |

**This table IS the plugin.** Capture it in YAML; ship one hook script that consumes the YAML.

### Tier 1: Pattern-config approach (YAML + ripgrep)

A single shared YAML config, new file `templates/stub-detection.yaml`:

```yaml
# vsdd-factory/templates/stub-detection.yaml
# Consume with: bin/stub-residue-check --config stub-detection.yaml [--scan-prod|--scan-tests|--scan-doc-freeze]

schema_version: 1

languages:
  rust:
    detect_markers: [Cargo.toml, rust-toolchain.toml]
    src_globs: ["**/*.rs"]
    src_excludes: ["**/tests/**", "**/target/**", "**/proofs/**", "**/fuzz/**"]
    test_globs: ["**/tests/**/*.rs", "**/*_test.rs"]
    in_file_test_attr: '#\[cfg\(test\)\]'
    stub_patterns:
      - 'todo!\('
      - 'unimplemented!\('
      - 'panic!\(\s*"[^"]*(?:TODO|todo|not yet|stub|unimplemented|FIXME)'
    inverted_polarity:
      - '#\[should_panic\([^)]*expected\s*=\s*"[^"]*(?:not yet implemented|not implemented|TODO|stub)'
    doc_comment_stub_markers:
      - '///\s*STUB\b'
      - '///\s*todo!\(\)'
    binary_entry_points: '[[bin]]\s+name'

  python:
    detect_markers: [pyproject.toml, setup.py, requirements.txt]
    src_globs: ["**/*.py"]
    src_excludes: ["**/tests/**", "**/test_*.py", "**/*_test.py", "**/.venv/**"]
    test_globs: ["**/tests/**/*.py", "**/test_*.py"]
    stub_patterns:
      - 'raise\s+NotImplementedError'
      - '^\s*\.\.\.\s*$'
      - '^\s*pass\s*(?:#.*TODO)?'
    inverted_polarity:
      - 'pytest\.raises\(NotImplementedError'
      - 'self\.assertRaises\(NotImplementedError'
    doc_comment_stub_markers:
      - '"""\s*STUB\b'
      - '"""\s*TODO\b'

  typescript:
    detect_markers: [tsconfig.json]
    co_detect_markers: [package.json]
    src_globs: ["**/*.{ts,tsx}"]
    src_excludes: ["**/*.test.{ts,tsx}", "**/*.spec.{ts,tsx}", "**/node_modules/**", "**/dist/**"]
    test_globs: ["**/*.test.{ts,tsx}", "**/*.spec.{ts,tsx}", "**/__tests__/**/*.{ts,tsx}"]
    stub_patterns:
      - 'throw\s+new\s+Error\(\s*[\"''][^\"'']*(?:TODO|todo|not implemented|stub)'
      - 'throw\s+new\s+Error\(\s*[\"'']Not implemented'
    inverted_polarity:
      - 'expect\([^)]*\)\.toThrow\([^)]*[\"''][^\"'']*not implemented'

  go:
    detect_markers: [go.mod]
    src_globs: ["**/*.go"]
    src_excludes: ["**/*_test.go", "**/vendor/**"]
    test_globs: ["**/*_test.go"]
    stub_patterns:
      - 'panic\(\s*"[^"]*(?:TODO|not implemented|stub|unimplemented)'
    inverted_polarity:
      - 'assert\.PanicsWithValue\([^,]+,\s*[^,]+,\s*[\"''][^\"'']*not implemented'

  java:
    detect_markers: [pom.xml, build.gradle, build.gradle.kts]
    src_globs: ["**/src/main/**/*.{java,kt,kts}"]
    test_globs: ["**/src/test/**"]
    stub_patterns:
      - 'throw\s+new\s+UnsupportedOperationException'
      - 'throw\s+new\s+RuntimeException\(\s*"[^"]*(?:TODO|not implemented)'
      - 'TODO\(\s*"[^"]*"\s*\)'
      - 'throw\s+NotImplementedError'
    inverted_polarity:
      - 'assertThrows\(UnsupportedOperationException'
      - 'assertFailsWith<NotImplementedError>'

  csharp:
    detect_markers: ['*.csproj', '*.sln']
    src_globs: ["**/*.cs"]
    src_excludes: ["**/*.Tests/**", "**/bin/**", "**/obj/**"]
    test_globs: ["**/*.Tests/**/*.cs"]
    stub_patterns:
      - 'throw\s+new\s+NotImplementedException'
    inverted_polarity:
      - 'Assert\.Throws<NotImplementedException>'

  ruby:
    detect_markers: [Gemfile, '*.gemspec']
    src_globs: ["**/lib/**/*.rb", "**/app/**/*.rb"]
    src_excludes: ["**/spec/**", "**/test/**"]
    test_globs: ["**/spec/**/*.rb", "**/test/**/*.rb"]
    stub_patterns:
      - 'raise\s+NotImplementedError'
      - 'raise\s+[\"''][^\"'']*(?:TODO|not implemented|stub)'
    inverted_polarity:
      - 'to\s+raise_error\(NotImplementedError'
```

**The hook script** (`bin/stub-residue-check`, single bash file ~150-200 LoC):

```bash
#!/usr/bin/env bash
# vsdd-factory/bin/stub-residue-check
#
# Modes:
#   --scan-prod          Stub residue in production paths
#   --scan-tests         Inverted-polarity tests
#   --scan-doc-freeze    Stale stub doc-comments
#
# Output: TAP-like findings, one per line:
#   STUB <file>:<line> <pattern> "<matched text>"
#
# Exit: 0 = clean, 2 = findings present
#
# Detects project language by walking marker files.
# Multi-language monorepos: applies each language's rules to its subtree.
```

### Tier 2: WASM tree-sitter substrate (semantic checks)

ripgrep patterns catch ~90% of syntactic stubs. The remaining ~10% — silent-shallow tests,
doc/code coherence drift, BC-postcondition-vs-implementation drift — need an AST.

**The tree-sitter substrate does not need to be built; Plan 3 below describes how to add
new checks to the existing WASM extension infrastructure.**

### Per-language surface that legitimately remains per-language

| Check | Why per-language | Approach |
|---|---|---|
| **Binary entry-point reachability** | Each language's build system enumerates entry points differently | Rust: `cargo metadata --format-version 1`; Node: `package.json` `bin:`; Python: `[project.scripts]`; Go: `go list -json ./...`. Wrap behind uniform `find_entry_points()` interface (~30 LoC per language) |
| **Test-vs-production boundary** | Convention varies by language | Captured in YAML `src_globs`/`test_globs`. Already language-agnostic via config |
| **Build-system invocation** | Per-language toolchain | Out of scope; existing CI handles |

**Adding a new language:** PR a YAML row + test fixtures in
`test-fixtures/<lang>/{passing,failing}/`. Zero plugin code change.

**Effort:** 3-4 days for Rust+Python+TS coverage; 1-2 days per additional language.

---

## Plan 3 — WASM Extension Substrate (Already In Production; This Plan Extends It)

**Critical framing shift from r2:** r2 described Plan 3 as "building the WASM substrate
from day 1 — ~3 plugin weeks." That was written without inspecting the repo.

The substrate is **already shipped.** `crates/factory-dispatcher/` is a production WASM
dispatcher binary. `crates/hook-sdk/` + `crates/hook-sdk-macros/` define the host ABI.
`crates/hook-plugins/` contains 17 individual hook plugin crates, each compiled to `.wasm`.
33 bash hooks run through `legacy-bash-adapter.wasm`. The `hooks-registry.toml` is 992 lines
of per-hook capability declarations.

**r3's contribution in Plan 3:** author NEW hook plugins for the stub-residue checks
(compile-to-WASM Rust crates), leveraging the existing factory-dispatcher. The decision
also introduces the Component Model (WIT + `cargo-component`) as the primary ABI for new
hook plugins.

### The existing WASM extension architecture

```
vsdd-factory/
├── crates/
│   ├── factory-dispatcher/   # WASM dispatcher host binary (wasmtime)
│   ├── hook-sdk/             # HOST_ABI — raw exports ABI (existing)
│   ├── hook-sdk-macros/      # proc-macros for hook boilerplate
│   └── hook-plugins/         # 17 hook plugin crates → .wasm artifacts
├── plugins/vsdd-factory/
│   ├── hook-plugins/         # 25 compiled .wasm files (shipped with plugin)
│   ├── hooks/                # 33 .sh scripts (via legacy-bash-adapter)
│   └── hooks-registry.toml   # 992-line capability registry
```

### Component Model adoption for new hooks

r2 said "raw exports for v1; WIT/Component Model deferred to v2 — tooling still maturing."
**This is outdated.** In 2026, the Component Model is production-ready:

- `cargo-component` + `wit-bindgen` + `wasm32-wasip2` are the recommended path.
- Production adopters: American Express FaaS on wasmCloud, Fermyon's 75M RPS edge platform,
  Docker's 7-runtime Wasm support.
- Wasmtime (current: v40.x, released 2026-01) supports Component Model natively.
- The factory-dispatcher already embeds wasmtime; supporting Component Model alongside
  the existing raw-export ABI is a factory-dispatcher change, not a full rebuild.

**r3 recommendation:** new hook plugin crates for the stub-residue checks use the Component
Model path (`wasm32-wasip2`, `cargo-component`, WIT interface). Existing hook plugins keep
their current raw-export ABI (`crates/hook-sdk/HOST_ABI.md`). Both run on the same
factory-dispatcher.

**Open architect decision (Q2):** this requires factory-dispatcher to support both ABI
modes in the same registry. Evaluate whether that is a small `match abi_kind { Legacy, Component }` in the dispatcher or a larger refactor. If the cost is too high, the stub-residue
hooks can be authored as `legacy-bash-adapter` scripts initially and migrated later.

### Example WIT interface for a vsdd check

If the Component Model path is adopted, new checks define their interface via WIT:

```wit
// wit/vsdd-check.wit
package vsdd:check@0.1.0;

interface check {
    record finding {
        severity: string,
        rule: string,
        file: string,
        line: u32,
        column: u32,
        message: string,
        evidence: string,
    }

    record check-input {
        file-path: string,
        source-bytes: list<u8>,
        language: string,
    }

    metadata: func() -> string;   // JSON: name, version, applies-to-languages
    run: func(input: check-input) -> list<finding>;
}

world stub-residue-check {
    export check;
}
```

Generated bindings (`wit-bindgen`) eliminate the manual pointer/length marshaling of the
raw-export ABI. Authoring a new check becomes:

```bash
cargo component new vsdd-stub-residue --target wasm32-wasip2
# Edit src/lib.rs with impl Check for StubResidueCheck { ... }
cargo component build --release
# Copy .wasm to hook-plugins/ and add entry to hooks-registry.toml
```

### Tree-sitter WASM grammars (for Tier 2 structural checks)

When Tier 2 structural checks are added, tree-sitter WASM grammars provide the AST.
Relevant verified facts from research (correcting r2):

**Correct tree-sitter version claim:**
- Current latest: v0.26.8 (released March 2026).
- `WasmStore` is available in the current API behind the `wasm` feature flag:
  `tree-sitter = { version = "0.26", features = ["wasm"] }`.
- r2's "stable since v0.22 (2024)" was approximate. State it correctly: "available behind
  the `wasm` feature flag in v0.26.x (current as of May 2026)."

**Grammar build path (v0.26.1+):**
- `tree-sitter build --wasm` now uses **WASI SDK** instead of Emscripten (changed in
  v0.26.1). Plugin CI must have WASI SDK installed to build grammars locally.
- No official upstream pre-compiled WASM grammar registry exists. Options:
  - Local build from pinned source commits (recommended for provenance).
  - Community mirrors: Microsoft `vscode-tree-sitter-wasm`, `wasm-lsp/tree-sitter-wasm`,
    `kreuzberg/tree-sitter-language-pack-wasm` (305+ languages via jsdelivr CDN).
  - Hybrid: local build primary; community mirror as fallback for projects without WASI SDK.
- r3 recommendation: **build locally from pinned source** (option b from r2 Q8). Hash-pin
  artifacts in `extensions.lock.toml`. WASI SDK requirement documented in plugin CI setup.

**Wasmtime size (corrected):**
- r2 said "~10MB added to plugin binary." Verified: wasmtime minimal build is ~2-3MB.
  The cost is already absorbed by the existing factory-dispatcher binary.

### Secondary authoring toolchains

For non-Rust check authors (accurate claims, not r2's over-selling):

**AssemblyScript:** Alive and maintained in 2026. Targets WASM directly via Binaryen.
Notable limitations: no closures (waiting on Function References / GC proposals); limited
stdlib vs TypeScript. The no-closures limitation matters for tree-sitter query-result-callback
patterns. Position as viable secondary toolchain for TypeScript-fluent authors writing
simple checks; flag the closure limitation explicitly in check-authoring docs.

**TinyGo:** Actively maintained (guide last updated 2026-04-20). Supports `wasm-unknown`
target. Notable limitations: `net/http`, `encoding/json`, and reflection-heavy code are
partial or unsupported. Goroutines use a cooperative scheduler. Position as tertiary
toolchain for Go-fluent authors; expect friction from stdlib gaps.

**Toolchain priority:** Rust (primary, already proven in vsdd-factory) > AssemblyScript
(secondary, TypeScript-fluent) > TinyGo (tertiary, Go-fluent). Do not oversell
secondary/tertiary paths.

### The extension manifest (aligning with existing hooks-registry.toml schema)

r2 proposed a parallel manifest format. r3 aligns with the existing `hooks-registry.toml`
capability schema instead:

```toml
# hooks-registry.toml addition for a new WASM check plugin
[[hooks]]
name = "validate-stub-residue"
event = "PreToolUse"
tool = "Edit|Write"
if = "Edit(crates/*/src/**/*.rs) | Write(crates/*/src/**/*.rs)"
plugin = "hook-plugins/validate-stub-residue.wasm"  # new native WASM hook
priority = 140
timeout_ms = 3000
on_error = "block"
abi_kind = "component"         # new field: "legacy" | "component" — factory-dispatcher routes accordingly

[hooks.capabilities.read_file]
path_allow = ["crates/**/*.rs"]
```

The `abi_kind` field is the only addition needed to support both ABI modes in the same
registry. The factory-dispatcher reads it and selects the appropriate instantiation path.

### Capability model (unchanged from existing schema)

vsdd-factory's `hooks-registry.toml` already implements a production capability model.
r3 adds no parallel capability model — new hooks use the same schema fields:
`read_file.path_allow`, `write_file.path_allow`, `exec_subprocess.binary_allow`,
`exec_subprocess.env_allow`, `env_allow`.

Default-deny posture is already the baseline in the existing registry. New hooks declare
only the capabilities they need.

### What Plan 3 delivers in r3 (extending the existing substrate)

- `crates/hook-plugins/validate-stub-residue/` — new Rust crate, Component Model path,
  compiled to `hook-plugins/validate-stub-residue.wasm`.
- `crates/hook-plugins/validate-inverted-polarity/` — same pattern.
- `wit/vsdd-check.wit` — WIT interface definition for the new check type.
- `hooks-registry.toml` additions with `abi_kind = "component"` for new entries.
- factory-dispatcher change to support `abi_kind` dispatch.
- `docs/check-authoring.md` — WIT-first authoring guide (Rust primary, AS secondary).
- CI: validate new WASM artifacts against fixture sets; ABI compatibility check.

**Estimated effort:** 1-2 plugin weeks (factory-dispatcher abi_kind dispatch = 0.5w;
two new check crates = 0.5w; docs + CI = 0.5w). Compare to r2's "~3 plugin weeks for Phase
3.A-D" — the 3-week estimate was for building the substrate from scratch.

---

## Recommended Build Plan

### Phase 1 (immediate): Plan 1, Substrate 1A + 2 + 3, bash-first

Prove the four-substrate model on one language. Land in vsdd-factory plugin as v-next.

**Deliverables:**
- 4 hook scripts in `hooks/` registered in `hooks-registry.toml` (Substrate 1A)
- `update-bc-status-on-story-merge.sh` SubagentStop hook (replaces r2's state-manager patch)
- 4 agent-prompt patches: implementer (Graduation Contract), adversary (stub-residue perimeter
  dimension), test-writer (POL-11 self-check), stub-architect (reconcile before patching)
- 4 scheduled-audit skills (Substrate 3)
- POL-13 through POL-17 policy additions
- `partial-merge` story status enum (see companion proposal)

**Effort:** ~1 plugin week.

**Validation:** re-run the workspace audit on the source project after Phase 1 lands; new
findings count should drop to <5.

### Phase 2 (next cycle, parallel with Phase 1 where possible): Plan 2, YAML-driven substrate

Refactor Rust-anchored hook scripts into YAML-driven multi-language substrate.

**Deliverables:**
- `templates/stub-detection.yaml` schema + JSON Schema for editor autocomplete
- `bin/stub-residue-check` generic hook script
- Rust + Python + TypeScript + Go YAML rows
- Test fixtures per language in `test-fixtures/<lang>/{passing,failing}/`
- `CONTRIBUTING.md` "add a language" section
- Vendor-on-init skill patch (Substrate 1B lefthook integration)

**Effort:** ~1 plugin week.

### Phase 3 (after Phase 1 validates model): Plan 3, Component Model WASM hooks

Author new hooks as Component Model WASM plugins for the existing factory-dispatcher.

**Deliverables:**
- `wit/vsdd-check.wit` WIT interface
- factory-dispatcher `abi_kind` dispatch support
- 2 new WASM check plugin crates (stub-residue, inverted-polarity)
- Grammar build path via WASI SDK (for future Tier 2 tree-sitter checks)
- Check-authoring documentation (WIT-first)

**Effort:** ~1-2 plugin weeks.

---

## Policy Registry Additions (Summary)

These policies are added to the consuming project's `policies.yaml` via `policy-registry init`
or `policy-add`. They are NOT plugin-internal policies; they are methodology governance for
VSDD-managed projects.

```yaml
  - id: 13
    name: production_stub_residue_blocks_merge
    description: "status: merged is forbidden when rg finds todo!()/unimplemented!()/panic!(TODO) in production-path files. Stories with stub residue must use status: partial-merge."
    adopted: vsdd-prevention-layers-r3
    severity: HIGH
    enforced_by: [validate-stub-residue hook, adversary-prompt, audit-stub-debt skill]
    scope: [story]
    lint_hook: hooks/validate-stub-residue.sh
    verification_steps:
      - "Run: rg 'todo!\\(|unimplemented!\\(' crates/*/src/ --glob '!tests/**' against story delivery files"
      - "Zero matches required for status: merged"
      - "Non-zero matches → story must be partial-merge with graduation contract"

  - id: 14
    name: story_frontmatter_index_consistency
    description: "Story frontmatter status: must match the corresponding STORY-INDEX row. Two-system truth drift is a P0 finding."
    adopted: vsdd-prevention-layers-r3
    severity: HIGH
    enforced_by: [validate-story-frontmatter-index-coherence hook]
    scope: [story]
    lint_hook: hooks/validate-story-frontmatter-index-coherence.sh
    verification_steps:
      - "For each story in .factory/stories/*.md, read frontmatter status:"
      - "Find the story's row in STORY-INDEX.md, read Status column"
      - "If they differ → P0 finding with file:line citation for both locations"

  - id: 15
    name: bc_vp_promotion_on_anchor_merge
    description: "BCs listed in a story's behavioral_contracts: array auto-promote draft → active when story reaches merged. No manual promotion required."
    adopted: vsdd-prevention-layers-r3
    severity: HIGH
    enforced_by: [update-bc-status-on-story-merge SubagentStop hook]
    scope: [bc, story]
    lint_hook: hooks/update-bc-status-on-story-merge.sh
    verification_steps:
      - "For each merged story, read behavioral_contracts: array"
      - "For each BC ID, verify its status: is active (not draft)"
      - "If any BC is still draft after anchor story merged → P0 finding"

  - id: 16
    name: runtime_wiring_required_for_accepted_adrs
    description: "Every accepted ADR with crate-level deliverables must be reachable from at least one bin/ target. Unwired crates are architectural debt."
    adopted: vsdd-prevention-layers-r3
    severity: MEDIUM
    enforced_by: [audit-runtime-wiring scheduled skill]
    scope: [story, adr]
    lint_hook: null
    verification_steps:
      - "Run /vsdd-factory:audit-runtime-wiring before wave gate"
      - "For each accepted ADR: identify the primary entry point (e.g. MyService::new)"
      - "Verify cargo metadata or binary-grep shows at least one bin/ crate importing and calling it"
      - "Zero-call entry points → P0 finding with ADR citation"

  - id: 17
    name: no_inverted_polarity_tests_outside_red_gate
    description: "Tests using #[should_panic(expected = ...)] matching stub idioms (not yet implemented, TODO, stub) are forbidden outside declared RED-gate phase windows."
    adopted: vsdd-prevention-layers-r3
    severity: HIGH
    enforced_by: [validate-inverted-polarity hook, adversary-prompt]
    scope: [story]
    lint_hook: hooks/validate-inverted-polarity.sh
    verification_steps:
      - "Run: rg '#\\[should_panic.*expected.*not yet implemented|not implemented|TODO|stub' tests/ crates/*/tests/"
      - "Any match outside a file with #[red_gate_phase] attribute or red-gate-state.json entry → P0"
      - "Inverted-polarity tests codify broken state as green CI — treat as critical"
```

---

## Open Questions for the vsdd-factory Engineer

**Q1: Vendor-on-init details**
Should the vendor-on-init skill be a new `skills/install-hooks/SKILL.md` or a step added
to the existing `skills/setup-env/SKILL.md`? The `setup-env` skill already handles toolchain
provisioning; hooks vendoring is a natural adjacent step. If the skill is new, what triggers
it — explicit user invocation, or automatically on `SessionStart` for projects that have
the plugin enabled?

**Q2: Component Model adoption posture**
The factory-dispatcher currently runs raw-export ABI hooks exclusively (per `crates/hook-sdk/HOST_ABI.md`). Adopting `abi_kind = "component"` requires factory-dispatcher support for Component Model dispatch alongside the existing path. Is this a small conditional (`match abi_kind { Legacy => ..., Component => ... }`) or a larger refactor? If the refactor cost is >1 week, the Phase 3 stub-residue hooks should use `legacy-bash-adapter` initially and be migrated later. Architect should estimate before committing to the Component Model path in Phase 1.

**Q3: Tree-sitter grammar provenance strategy**
r3 recommends building WASM grammars locally from pinned upstream source commits (option b).
This requires WASI SDK in plugin CI. Is WASI SDK already available in the plugin's CI
environment (check `.github/workflows/`)? If not, is the CI setup cost acceptable, or is
a community mirror (e.g., `vscode-tree-sitter-wasm`) acceptable as primary source?
The WASI SDK migration happened in tree-sitter v0.26.1; the plugin should pin to v0.26.x.

**Q4: Policy shipping mechanism**
The plugin does not currently ship a baseline `policies.yaml`. For POL-13..17 to land in
consuming projects, either: (A) `policy-registry init` writes them, or (B) the plugin ships
a `templates/policies-baseline.yaml` that `init` copies. Which approach? Option A is safer
for existing projects already at POL-12; Option B is cleaner for new projects.

**Q5: state-manager exclusion from BC promotion — any objections?**
r3 moves BC promotion from state-manager agent patch (r2) to a SubagentStop hook on
pr-manager. `state-manager.md` says "NEVER write specification documents." This seems clear.
However: if there are BC files that need more nuanced promotion logic (e.g., partial-merge
stories should NOT promote BCs), does that logic fit cleanly in a bash hook, or does it
need agent judgment? If the latter, consider a `bc-promoter` agent (not state-manager)
with narrowly scoped write permissions.

**Q6: Audit-skill granularity — top-level vs maintenance-sweep sub-sweeps**
r3 proposes both (top-level skill AND sub-sweep in maintenance-sweep). Is there a reason
to prefer one over the other? Running maintenance-sweep calls all four audit skills at once
(good for wave-gate), but individual skills can be invoked on-demand (good for debugging a
specific audit failure). Composition appears correct; confirm there is no duplication concern.

**Q7: `update-bc-status-on-story-merge.sh` as bash vs native WASM**
The sibling hook `update-wave-state-on-merge.wasm` is a native WASM plugin. Should
`update-bc-status-on-story-merge` be bash (via legacy-bash-adapter, faster to write) or
native WASM (type-safe, matches the sibling pattern)? Given it needs to write BC frontmatter
files, bash is simpler for the initial implementation; migrate to WASM in a follow-up cycle.

**Q8: `#[should_panic]` allowlist for legitimate uses**
Not all `#[should_panic]` tests are inverted-polarity. Some correctly test error conditions
in production code (e.g., `#[should_panic(expected = "index out of bounds")]`). The
inverted-polarity check must distinguish "tests that require production code to be BROKEN"
from "tests that verify production error handling." The YAML pattern above is conservative
(matches only stub-idiom strings). Confirm the pattern set has acceptable false-positive
rate against the actual Prism test suite before shipping.

**Q9: STORY-INDEX drift with partial-merge status**
When r3's `validate-story-frontmatter-index-coherence.sh` runs, it will find drift for any
`partial-merge` story that doesn't yet have `partial-merge` as a valid STORY-INDEX status.
The STORY-INDEX schema must be updated to permit `partial-merge` before the hook ships.
This is covered in the companion proposal `vsdd-stub-merge-policy-2026-05-08.md` — confirm
that proposal lands first or simultaneously.

**Q10: stub-architect.md reconciliation**
`agents/stub-architect.md` (7.4KB) already addresses stub-related agent behavior. Before
patching `implementer.md` and `adversary.md`, read `stub-architect.md` to understand what is
already enforced. Some of r3's Graduation Contract language may already exist there.
If it does, the implementer.md patch should reference stub-architect rather than duplicating.

---

## Appendix A — Pattern Library Status

The Plan 2 YAML is a starting point. Production-grade pattern lists need:

- **False-positive corpus:** each language row should ship with "this looks like a stub but
  isn't" examples. Python's `...` ellipsis is legitimate in abstract declarations and protocol
  stubs — must scope by context (e.g., only flag `...` in non-abstract method bodies without
  a class inheriting `ABC`).
- **Idiom drift policy:** Rust nightly may add new panic macros; TypeScript evolves. The YAML
  should be versioned (`schema_version: 1`) and CI-gated against drift in upstream language
  specs.
- **Domain-expert pass per language:** the 9-language draft above is a 60-second starter.
  Each language needs a pass from someone who knows its idioms before shipping.
- **Project-level override:** some projects may legitimately permit certain panic idioms
  (e.g., `panic!("internal invariant: ...")` is not a stub). Provide a per-project
  `.factory/stub-detection-overrides.yaml` allowlist mechanism.

## Appendix B — Findings Reference

The source-project workspace audit that motivated this proposal:

- 53 findings: P0=18, P1=23, P2=12.
- 8 audit dimensions: production stub residue, story-vs-impl drift, silent-shallow tests,
  TOML/config orphans, BC postcondition gaps, ADR implementation status, VP proof status,
  documentation drift.
- Coverage by substrate (estimated):
  - Substrate 1A hooks would have caught: ~28 findings (production stub residue at
    edit time + story-index consistency + inverted-polarity + doc-freeze).
  - Substrate 1B lefthook would have caught: ~20 overlapping (second-line gate on push).
  - Substrate 2 agent invariants would have caught: ~14 findings (silent-shallow +
    BC promotion gaps + status enum drift + adversary mandatory checks).
  - Substrate 3 scheduled audits would have caught: ~7 findings (no-binary-loads-this +
    orphan TOMLs + VP graduation + workspace-wide drift).
- Substrate overlap is intentional. ~12 findings are caught by 2+ substrates.

Full audit report: `cycles/wave-4-operations/workspace-audit-2026-05-08.md` (Prism project).

## Appendix C — Companion Proposals

- `vsdd-stub-merge-policy-2026-05-08.md` — the schema fix: `partial-merge` status enum,
  graduation contract, adversary policy, `audit-stub-debt` skill. **Read this first if you
  haven't.** Schema changes (partial-merge enum, STORY-INDEX schema, STUB-DEBT-INDEX) are
  specified there; r3 references them and does not duplicate.
- `vsdd-prevention-layers-2026-05-08.md` — this document: the four-substrate enforcement
  infrastructure (Plan 1), language-agnostic YAML+ripgrep syntactic checks (Plan 2), and
  Component Model WASM hook authoring for the existing WASM extension substrate (Plan 3).

The two proposals are complementary, not redundant. The schema fix without the enforcement
infrastructure produces unenforced enums. The enforcement infrastructure without the schema
fix has nowhere to write the `partial-merge` state. Land them together or in immediate
sequence.

## Appendix D — vsdd-factory Repo Orientation for New Sessions

If you are a fresh Claude Code session reading this proposal in the vsdd-factory repo,
read these files first to understand the substrate you are extending:

| File | Purpose |
|---|---|
| `crates/factory-dispatcher/src/main.rs` | WASM dispatcher binary — understand the hook routing loop |
| `crates/hook-sdk/HOST_ABI.md` | Raw-export ABI spec for existing hook plugins |
| `plugins/vsdd-factory/hooks-registry.toml` | Full 992-line capability registry — this is where new hooks are registered |
| `plugins/vsdd-factory/hooks/validate-state-index-status-coherence.sh` | Canonical example of a PostToolUse bash hook |
| `plugins/vsdd-factory/hooks/validate-red-ratio.sh` | Canonical example of a red-discipline PreToolUse bash hook |
| `plugins/vsdd-factory/hook-plugins/update-wave-state-on-merge.wasm` | The SubagentStop hook your `update-bc-status-on-story-merge.sh` is modeled after — find its source in `crates/hook-plugins/update-wave-state-on-merge/` |
| `plugins/vsdd-factory/agents/adversary.md` | 21KB adversary prompt — find the Three-Perimeter Scope Contract section before patching |
| `plugins/vsdd-factory/agents/stub-architect.md` | 7.4KB — read before patching implementer.md |
| `plugins/vsdd-factory/skills/maintenance-sweep/SKILL.md` | Pattern for scheduled audit skills (Sweep 1/2/3 already exist) |
| `plugins/vsdd-factory/skills/check-input-drift/SKILL.md` | Pattern for binary-assisted audit skills (uses `compute-input-hash` Rust binary) |

---

## Revision History (full)

| Revision | Date | Author | Summary |
|---|---|---|---|
| r1 | 2026-05-08 | Architect | Initial — Plan 1 (3-layer model) + Plan 2 (YAML+ripgrep substrate). Rust-anchored. No WASM substrate design. |
| r2 | 2026-05-08 | Architect | Added Plan 3 (WASM extension substrate, "day-1 commitment"); updated build sequencing; added Q6-Q10. Written without inspecting the vsdd-factory repo — multiple claims were stale or wrong. |
| r3 | 2026-05-08 | Architect | Ground-truth recalibration from `r3-research-findings.md` (repo inspection + web research). Key changes: (1) WASM substrate already in production — Plan 3 extends it, does not build it; (2) Component Model production-ready, promoted to primary ABI for new hooks; (3) Layer 1 split into 1A (CC hooks) / 1B (git hooks / vendor-on-init); (4) state-manager excluded from BC promotion — replaced by SubagentStop hook modeled on `update-wave-state-on-merge.wasm`; (5) POL IDs start at POL-13 (POL-12 taken by vsdd-factory internal `bc_tv_emitter_consistency`); (6) vendor-on-init pattern specified for lefthook bash distribution; (7) all deliverables annotated with EXTEND vs GREENFIELD + existing-analog citations; (8) technology claims corrected (tree-sitter version, wasmtime size, Component Model maturity, AssemblyScript limitations, TinyGo limitations, grammar provenance); (9) added Appendix D (repo orientation for new sessions); (10) 10 open questions updated to reflect architectural decisions surfaced by research. |

End of proposal.
