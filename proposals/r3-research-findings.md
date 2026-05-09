---
artifact_type: research-findings
target: r3 of vsdd-prevention-layers-2026-05-08.md
date: 2026-05-08
producer: general-purpose-research-agent
inputs:
  - .factory/proposals/vsdd-prevention-layers-2026-05-08.md
  - /Users/jmagady/Dev/vsdd-factory/ (read-only inspection)
  - https://code.claude.com/docs/en/plugins
  - https://code.claude.com/docs/en/plugins-reference
  - https://code.claude.com/docs/en/hooks
  - https://github.com/tree-sitter/tree-sitter/releases
  - https://docs.rs/tree-sitter/latest/tree_sitter/
  - https://crates.io/crates/wasmtime
  - https://docs.wasmtime.dev/examples-minimal.html
  - https://component-model.bytecodealliance.org/
  - https://www.assemblyscript.org/status.html
  - https://tinygo.org/docs/guides/webassembly/
status: ready-for-architect-r3-composition
---

# r3 Research Findings — VSDD Prevention Layers

This document is read-only research input for the architect composing r3. It contains:
1. **Local plugin grounding** — what already exists in `vsdd-factory` vs. what r2 proposes.
2. **Technology claim verification** — each r2 claim against current 2026 sources.
3. **Architectural taxonomy validation** — the proposed 4-substrate split for r2's "Layer 1".
4. **Open architectural questions** — decisions the architect must make in r3.

---

## Part 1 — vsdd-factory plugin grounding (from local inspection)

### Repository layout

`/Users/jmagady/Dev/vsdd-factory/` is **not just a plugin** — it is a Rust workspace whose primary product is the plugin sitting under `plugins/vsdd-factory/`. The repo also ships:

- `crates/factory-dispatcher/` — the **WASM hook dispatcher binary** (single-binary host that loads and runs all WASM hook plugins).
- `crates/hook-sdk/` + `crates/hook-sdk-macros/` — the host ABI for WASM hook plugins (`HOST_ABI` defined in `crates/hook-sdk/HOST_ABI.md`).
- `crates/hook-plugins/*` — 17 individual hook plugin crates that each compile to a `.wasm` file (e.g. `block-ai-attribution`, `validate-stable-anchors`, `regression-gate`, `lint-registry-async-invariant`, `validate-per-story-adversary-convergence`, `legacy-bash-adapter`).
- `crates/sink-{file,otel-grpc,honeycomb,datadog,http,core}/` — telemetry sinks for hook events (this plugin already has a full observability story).
- `tests/integration/hooks/` — integration tests for hook plugins.

**Implication for r3:** the plugin is FAR more architecturally mature than r2 assumes. r2 proposes "WASM substrate as Phase 3 build (~3 weeks)". The substrate **already exists in production** — every existing hook either runs as a native-WASM plugin or via the `legacy-bash-adapter.wasm` shim that execs an underlying bash script with declared capabilities. r3 should not propose building this; it should propose ADDING new hooks to the existing substrate.

### Existing plugin structure

Plugin root: `/Users/jmagady/Dev/vsdd-factory/plugins/vsdd-factory/`

```
plugins/vsdd-factory/
├── .claude-plugin/plugin.json        # name=vsdd-factory, version=1.0.0-rc.11, license=MIT
├── agents/                           # 36 agent .md files (state-manager, adversary, implementer, test-writer, …)
│   └── orchestrator/                 # nested orchestrator subagents
├── bin/                              # executables on $PATH while plugin is enabled
│   ├── compute-input-hash            # Rust binary (input-hash drift detection)
│   ├── factory-dashboard             # diagnostic dashboard
│   ├── factory-obs                   # observability stack manager
│   ├── factory-query                 # ?
│   ├── factory-replay                # ?
│   ├── factory-report                # ?
│   ├── factory-sla                   # ?
│   ├── lobster-parse                 # workflow parser
│   ├── multi-repo-scan               #
│   ├── research-cache                #
│   ├── wave-state                    # wave-state.yaml manipulator
│   └── emit-event                    # OTel event emitter (sourced by every hook script)
├── config/
│   └── artifact-path-registry.yaml   # validate-artifact-path's allow-list registry
├── docs/                             # plugin-internal docs
├── fixtures/                         # test fixtures
├── hook-plugins/                     # 25 .wasm files (compiled native WASM hooks)
│   ├── legacy-bash-adapter.wasm      # generic shim that runs hooks/*.sh under capabilities
│   ├── block-ai-attribution.wasm
│   ├── validate-stable-anchors.wasm
│   ├── regression-gate.wasm
│   ├── lint-registry-async-invariant.wasm
│   ├── validate-per-story-adversary-convergence.wasm
│   ├── validate-pr-review-posted.wasm
│   ├── handoff-validator.wasm
│   ├── pr-manager-completion-guard.wasm
│   ├── update-wave-state-on-merge.wasm
│   ├── session-{start,end}-telemetry.wasm
│   ├── capture-{commit,pr}-activity.wasm
│   ├── track-agent-{start,stop}.wasm
│   ├── tool-failure-hooks.wasm
│   ├── warn-pending-wave-gate.wasm
│   ├── worktree-hooks.wasm
│   ├── session-learning.wasm
│   ├── validate-artifact-path.wasm
│   └── hello-hook.wasm               # canonical example
├── hooks/                            # 33 .sh hook scripts (consumed by legacy-bash-adapter)
│   ├── dispatcher/                   # dispatched runner (?)
│   ├── lib/block.sh                  # canonical block_pre helper
│   ├── hooks.json.template           # rendered into per-platform hooks.json files
│   ├── hooks.json.{darwin,linux,windows}-{arm64,x64}    # platform-specific Claude Code manifest
│   └── *.sh                          # individual hook bash scripts
├── hooks-registry.toml               # 992-line single source of truth for ALL hook routes
├── rules/                            # 10 .md files (rust.md, bash.md, factory-protocol.md, …)
├── skills/                           # 121 skill directories, each containing SKILL.md
├── templates/                        # >100 templates (BC, VP, story, ADR, etc.)
├── tests/                            # 53 test directories
├── tools/                            # ?
└── workflows/                        # 9 .lobster workflow definitions
    ├── greenfield.lobster
    ├── brownfield.lobster
    ├── feature.lobster
    ├── maintenance.lobster
    ├── multi-repo.lobster
    ├── code-delivery.lobster
    ├── discovery.lobster
    ├── planning.lobster
    └── phases/
```

**Plugin manifest (`plugins/vsdd-factory/.claude-plugin/plugin.json`)** is minimal — only `name`, `description`, `version`, `author`, `homepage`, `repository`, `license`, `keywords`. No explicit `hooks`/`agents`/`skills` paths, so Claude Code uses default discovery (which resolves `hooks/hooks.json`, `agents/`, `skills/`, etc., per the plugins-reference at https://code.claude.com/docs/en/plugins-reference).

### Existing hooks: the dispatcher pattern

The plugin uses a **single-dispatcher pattern** that is unusual relative to vanilla Claude Code documentation:

`hooks/hooks.json.template` registers a single command — `hooks/dispatcher/bin/{{PLATFORM}}/factory-dispatcher{{EXE_SUFFIX}}` — for every Claude Code event type. The dispatcher then internally consults `hooks-registry.toml` to decide which WASM modules (or `legacy-bash-adapter.wasm` shims) to run. Hook events covered by the manifest template:

| Event | How registered |
|---|---|
| `PreToolUse` | factory-dispatcher binary |
| `PostToolUse` | factory-dispatcher binary |
| `PermissionRequest` | factory-dispatcher binary |
| `Stop` | factory-dispatcher binary |
| `SubagentStop` | factory-dispatcher binary |
| `SessionStart` | factory-dispatcher binary (`once: true`) |
| `SessionEnd` | factory-dispatcher binary (`once: true`) |
| `WorktreeCreate` | factory-dispatcher binary |
| `WorktreeRemove` | factory-dispatcher binary |
| `PostToolUseFailure` | factory-dispatcher binary |

**The 992-line `hooks-registry.toml`** declares ~40 individual hook routes with rich capability metadata:

```toml
[[hooks]]
name = "validate-artifact-path"
event = "PreToolUse"
tool = "Write|Edit"
plugin = "hook-plugins/validate-artifact-path.wasm"
priority = 150
timeout_ms = 5000
on_error = "continue"

[hooks.capabilities.read_file]
path_allow = ["plugins/vsdd-factory/config/artifact-path-registry.yaml"]
```

Capability fields seen in registry: `read_file.path_allow`, `write_file.path_allow`, `write_file.max_bytes_per_call`, `exec_subprocess.binary_allow`, `exec_subprocess.env_allow`, `exec_subprocess.shell_bypass_acknowledged`, `env_allow`. This is a **production capability model**, already shipped.

**Concrete existing hook coverage relevant to r3:**

| Hook | Event | Tool matcher | Substrate | Behavior |
|---|---|---|---|---|
| `red-gate.sh` | PreToolUse | `Edit\|Write` | legacy-bash-adapter | Strict TDD red-before-green: blocks Edit on production source files unless declared in `.factory/red-gate-state.json`. Test files always allowed. **This is essentially a "no premature implementation" gate — r2 doesn't reference it.** |
| `validate-red-ratio.sh` | PostToolUse | `Edit\|Write` | legacy-bash-adapter | Enforces `RED_RATIO >= 0.5` in story `red_ratio:` frontmatter. P0 if missing exception path. |
| `regression-gate.wasm` | PostToolUse | (any) | native WASM | Reads/writes `.factory/regression-state.json`. |
| `validate-stable-anchors.wasm` | PreToolUse | `Write\|Edit` | native WASM | Blocks volatile `*.rs:NNN`-style line citations in `.factory/specs/**/*.md`. |
| `validate-state-index-status-coherence.sh` | PostToolUse | `Edit\|Write` | legacy-bash-adapter | Cross-checks `STATE.md` `convergence_status` against `cycles/*/INDEX.md` `Status:` headers. **Closely related to r2's proposed "story-frontmatter ↔ STORY-INDEX" check, but for cycle-level state, not story-level.** |
| `validate-story-bc-sync.sh` | PostToolUse | `Edit\|Write` | legacy-bash-adapter | Story `bcs:` frontmatter ↔ body BC table ↔ ACs sync. (Policy POL-8 `bc_array_changes_propagate_to_body_and_acs`.) |
| `validate-template-compliance.sh` | PostToolUse | `Edit\|Write` | legacy-bash-adapter | `on_error = "block"` — blocks edits that break template compliance. |
| `validate-input-hash.sh` | PostToolUse | `Edit\|Write` | legacy-bash-adapter | `on_error = "block"` — input-hash drift detection per file. |
| `validate-pr-merge-prerequisites.sh` | PreToolUse | `Agent` | legacy-bash-adapter | Blocks pr-manager Agent dispatch unless prerequisites met. |
| `validate-wave-gate-prerequisite.sh` | PreToolUse | `Agent` | legacy-bash-adapter | Blocks wave-gate Agent dispatch unless prerequisites met. |
| `validate-per-story-adversary-convergence.wasm` | SubagentStop | (any) | native WASM | Blocks wave-gate dispatch when any story lacks adversary convergence. |
| `validate-bc-title.sh`, `validate-vp-consistency.sh`, `validate-anchor-capabilities-union.sh`, `validate-finding-format.sh`, `validate-novelty-assessment.sh`, etc. | PostToolUse | `Edit\|Write` | legacy-bash-adapter | Already a comprehensive lint suite for VSDD artifacts. |

**Implication for r3:** the plugin already has the WASM-extension story r2 proposes for "Phase 3" — the architecture is shipped. New hooks become new entries in `hooks-registry.toml` plus either a new bash script (via legacy-bash-adapter) OR a new Rust crate at `crates/hook-plugins/<name>/` compiled to `.wasm`.

### Existing agents (`plugins/vsdd-factory/agents/`)

36 agent `.md` files. Frontmatter format (per Claude Code plugin spec) supports `name`, `description`, `model`, `effort`, `maxTurns`, `tools`, `disallowedTools`, `skills`, `memory`, `background`, `isolation: "worktree"`. Per Claude Code plugin spec, plugin agents **CANNOT** declare `hooks`, `mcpServers`, or `permissionMode` (security restriction).

Relevant agents for r3:
- `adversary.md` — 21KB. Has the **Three-Perimeter Scope Contract** (per-story / per-wave / phase-wide adversary scopes per ADR-017). r2's "production-stub-residue check as a mandatory dimension" must be added here.
- `implementer.md` — 16KB. Currently emphasizes UI Quality Loop (DF-037) and TDD. r2's "Graduation Contract" needs to be added.
- `test-writer.md` — 14KB. UI component state coverage focus (DF-037). r2's "silent-shallow scan" needs to be added.
- `state-manager.md` — 19KB. Bookkeeper, NEVER writes specs/code. Has STATE.md routing rules + Single-Commit Burst Protocol (TD-VSDD-053). r2's "BC/VP promotion-on-merge" needs to be added — but this is exactly the kind of write-action that the agent is currently bookkeeping-only for, so r3 must be careful about scope.
- `stub-architect.md` — 7.4KB. **Already exists** — this is an agent specifically focused on stubs, possibly already implementing some of r2's intent. r3 should reconcile r2's proposals against existing stub-architect rules.

### Existing skills (`plugins/vsdd-factory/skills/`)

121 skill directories. Format per Claude Code plugin spec: each is `<name>/SKILL.md` with frontmatter: `name`, `description`, `argument-hint`, optional `disable-model-invocation`. **No "scheduled" field** — skills are model-invoked (auto-dispatched by Claude when context matches `description`) or manually invoked via `/vsdd-factory:<name>` slash commands.

**Relevant existing skills:**
- `maintenance-sweep/SKILL.md` — already declares trigger types: "Scheduled (cron or GitHub Actions, recommended weekly)", "Manual", "Post-deploy". Already has `Sweep 1: Dependency Audit`, `Sweep 2: Documentation Drift`, `Sweep 3: Pattern Consistency`, etc. **r2's "audit-stub-debt" / "audit-runtime-wiring" / "audit-orphan-configs" / "audit-vp-promotion" can plug in as additional sweeps in this skill rather than greenfield new skills.**
- `check-input-drift/SKILL.md` — uses the `compute-input-hash` Rust binary at `bin/`. **Pattern to mimic for r3's audit skills**: ship a Rust binary that does the bulk scan, the skill orchestrates and reports.
- `track-debt/SKILL.md` — manages `.factory/tech-debt-register.md`. r2's findings can flow here.
- `policy-add/SKILL.md` + `policy-registry/SKILL.md` — already exist for policy registry management. r2's POL-12 through POL-16 plug into the existing registry.
- `validate-consistency/SKILL.md` — already has Check 8 (Test Tautology Detector, fixtures at `skills/validate-consistency/fixtures/tautology/`) and Check 9 (BC TV ↔ Emitter Field Consistency). **POL-11 (`no_test_tautologies`) and POL-12 (`bc_tv_emitter_consistency`) already exist.** r2 proposes POL-12 as `production_stub_residue_blocks_merge` — that name collides with existing POL-12. r3 must avoid the collision (next free policy IDs: 13+).
- `convergence-check/SKILL.md`, `convergence-tracking/SKILL.md`, `holdout-eval/SKILL.md`, `wave-gate/SKILL.md` — adjacent quality-gate skills.
- `factory-cycles-bootstrap/SKILL.md` — for migrating cycle structure.
- `state-burst/SKILL.md` — state-manager's atomic-burst protocol (per TD-VSDD-053).
- **No skill currently exists for** `audit-stub-debt`, `audit-runtime-wiring`, `audit-orphan-configs`, `audit-vp-promotion`. These are GREENFIELD per r2's deliverable list.

### Existing policies registry

The reference project (Prism) has its own `.factory/policies.yaml` at the **project level**, not the plugin level. **There is NO `policies.yaml` shipped with the plugin** — policies are project-scoped. The plugin ships SKILLS (`policy-add`, `policy-registry`) that create and manage a project's policies.yaml.

Prism's current policies.yaml (read at `/Users/jmagady/Dev/prism/...` not provided here, but the vsdd-factory mono-repo has its own at `/Users/jmagady/Dev/vsdd-factory/.factory/policies.yaml`):
- POL-1 through POL-9: baseline policies (from plugin documentation/baseline).
- POL-10: `demo_evidence_story_scoped` (lint hook: `hooks/validate-demo-evidence-story-scoped.sh`).
- POL-11: `no_test_tautologies` (validate-consistency Check 8).
- POL-12: `bc_tv_emitter_consistency` (validate-consistency Check 9). **HIGH severity, already taken.**

**Implication for r3 policy IDs:** r2's proposed POL-12 through POL-16 collide with vsdd-factory's POL-12. **r3 must use POL-13+ for the new policies, OR scope the policies as plugin-baseline-shipped rather than project-specific.**

Actually re-reading r2 more carefully: the proposal says "Policy registry additions (the plugin's baseline `policies.yaml`)". But based on inspection, **there is no plugin baseline policies.yaml at all** — each project owns its own. r3 has two options:

1. Have the plugin start shipping a baseline policies.yaml that gets copied into projects on initialization.
2. Have r2's policies be ADDED automatically when the plugin's `/vsdd-factory:policy-registry init` skill runs.

Either is a small architectural decision the architect should resolve.

### Existing CC hook integration evidence

Strong, mature, production. See "Existing hooks: the dispatcher pattern" above. The plugin uses a **dispatcher-routes-to-WASM** pattern, not a direct hooks.json-per-hook pattern. Adding a new hook means: (a) adding entry to `hooks-registry.toml`, (b) implementing as either a `hooks/<name>.sh` (legacy-bash-adapter route) or as a new Rust crate at `crates/hook-plugins/<name>/` compiled to `.wasm`.

### Build/release tooling

- **Cargo workspace** at repo root with `Cargo.toml` listing 12+ member crates.
- **`scripts/`** has bash CI helpers: `bump-version.sh`, `check-changelog-monotonicity.sh`, `check-platforms-drift.py`, `check-shakedown-window.sh`, `generate-hooks-json.sh`, `generate-registry-from-hooks-json.sh`.
- **`ci/`** directory exists.
- **`.github/workflows/`** exists (not enumerated in this scan — architect should confirm).
- **`Cross.toml`**, **`deny.toml`**, **`.semgrep.yml`** — multi-platform Rust build, dependency hygiene, secret scanning.
- Plugin version `1.0.0-rc.11` per `plugin.json`.

### Plugin distribution mechanism

Per `plugin.json`: `homepage = "https://github.com/drbothen/vsdd-factory"`, `repository = "https://github.com/drbothen/vsdd-factory"`. So the plugin is distributed via **git-hosted marketplace or direct git URL**. Per the plugins-reference docs, when `version` is set in `plugin.json` (1.0.0-rc.11), users only get updates when the maintainer bumps it. Users install via `claude plugin install vsdd-factory@<marketplace>` or via `--plugin-dir`/`--plugin-url` flags.

For this proposal, the relevant fact is: **bash scripts shipped at `plugins/vsdd-factory/hooks/*.sh` are reachable from CC hooks via `${CLAUDE_PLUGIN_ROOT}/hooks/<name>.sh`** (CLAUDE_PLUGIN_ROOT resolves to the install cache dir). This is exactly how all 33 existing bash scripts are wired today.

### What's already there vs. what r3 proposes

For each r3 deliverable in r2:

#### Hook scripts (r2 proposes 4, frame as "Layer 1: Hooks")

| r2 hook | r3 should propose | Why |
|---|---|---|
| `pre-push-stub-residue.sh` | **NEW** — but should be (a) a CC hook in `hooks-registry.toml` AND (b) a lefthook entry shipped via doc/config-template. r2 muddled this as a single "hook"; the substrates are different (see Part 3). | Greenfield. Closest existing analog: red-gate.sh (TDD red discipline) but different function. |
| `pre-burst-story-index-consistency.sh` | **PARTIAL** — `validate-state-index-status-coherence.sh` already exists for STATE.md ↔ cycles/INDEX.md sync. **r3 should REUSE/EXTEND** that hook to also cover story-file frontmatter ↔ STORY-INDEX.md row coherence, OR add a sibling `validate-story-frontmatter-index-coherence.sh`. | Sibling existing. |
| `pre-push-inverted-polarity.sh` | **NEW** — but pattern overlaps with `validate-red-ratio.sh` (both are red-discipline checks). r3 can add as a sibling lint hook in `hooks-registry.toml` and as a lefthook config-template entry. | Greenfield. |
| `pre-push-doc-freeze.sh` | **NEW** — no existing analog. | Greenfield. |

#### Policy registry additions (r2 proposes POL-12 through POL-16)

| r2 policy | r3 should propose | Why |
|---|---|---|
| POL-12 `production_stub_residue_blocks_merge` | **RENUMBER** to POL-13. POL-12 is taken (`bc_tv_emitter_consistency`). | Collision. |
| POL-13 `story_frontmatter_index_consistency` | **RENUMBER** to POL-14. | Cascading collision. |
| POL-14 `bc_vp_promotion_on_anchor_merge` | **RENUMBER** to POL-15. | Cascading. |
| POL-15 `runtime_wiring_required_for_accepted_adrs` | **RENUMBER** to POL-16. | Cascading. |
| POL-16 `no_inverted_polarity_tests_outside_red_gate` | **RENUMBER** to POL-17. | Cascading. |

Also: r2 says "the plugin's baseline `policies.yaml`" but the plugin **does not currently ship one**. r3 must decide whether to introduce one (with all baseline policies POL-1 through POL-9 + the new ones) or have these be shipped via the `policy-registry init` skill into project-level files.

#### Agent prompt patches (r2 proposes 4)

| r2 agent patch | Status | Notes |
|---|---|---|
| implementer.md: Graduation Contract | **NEW patch needed** | Existing agent is heavily UI/DF-037 focused. r2's BC-callgraph-reach requirement is genuinely new. |
| adversary.md: production-stub-residue check | **NEW patch needed** | Adversary already has 3-perimeter scope; this becomes a new mandatory perimeter dimension. r3 must specify which perimeter (per-story or workspace-wide). Likely workspace-wide (Phase 5). |
| test-writer.md: silent-shallow scan | **PARTIAL** | POL-11 `no_test_tautologies` (already gated via validate-consistency Check 8) is roughly equivalent to silent-shallow. r3 should **extend POL-11** rather than create a new check — existing fixture corpus exists at `skills/validate-consistency/fixtures/tautology/`. |
| state-manager.md: BC/VP promotion-on-merge | **NEW but architecturally fraught** | state-manager is currently strictly bookkeeper, NEVER writes spec content. Auto-promoting BC `status: draft → active` IS writing spec content. r3 must decide whether (a) state-manager gets a narrow scoped exception, or (b) the promotion happens in a different agent (adversary? consistency-validator?), or (c) the promotion is hook-driven (`update-bc-status-on-story-merge.sh`) with NO agent involvement. |

#### Scheduled skills (r2 proposes 4)

| r2 skill | r3 should propose | Why |
|---|---|---|
| `audit-stub-debt/` | **NEW** OR **MERGE into maintenance-sweep** as Sweep 4 | maintenance-sweep already declares it's the periodic-quality-sweep skill with weekly cron pattern. r3 should consider whether stub-debt is a sub-sweep there, vs. a separate top-level skill. |
| `audit-runtime-wiring/` | **NEW** | No analog exists. |
| `audit-orphan-configs/` | **NEW** | No analog. |
| `audit-vp-promotion/` | **NEW** but should reuse the `compute-input-hash` Rust-binary pattern | check-input-drift demonstrates the pattern. |

**Critical note on "scheduled":** Claude Code plugins do NOT have a built-in scheduling primitive. Per the docs, plugins ship `monitors/monitors.json` for background monitors, but those run for the session lifetime, not on cron. The actual scheduling mechanism is:
- **GitHub Actions cron** (recommended in maintenance-sweep — `cron: '0 2 * * 0'`)
- **External cron + `claude --plugin-dir ./vsdd-factory --print "/vsdd-factory:audit-stub-debt"`** invocation
- **Manual** human-triggered

r2 says "scheduled skills" as if they auto-run; in fact, the scheduling layer is OUTSIDE the plugin. r3 must clarify this distinction.

---

## Part 2 — Technology claim verification (from web research)

### Claim 1: "tree-sitter Rust WasmStore stable since v0.22 (2024)"

**r2 quote:** *"Loaded via tree-sitter's `WasmStore` (stable in the `tree-sitter` Rust crate since v0.22, 2024; production adopters include Zed editor and several language servers)."*

**Verified status:**
- Current latest tree-sitter version: **v0.26.8** (released March 31, 2026), per https://github.com/tree-sitter/tree-sitter/releases.
- v0.22.0 release (March 10, 2024) **does NOT introduce WasmStore**. Per the v0.22.0 release notes, the only wasm-related change was *"Remove vendored wasmtime headers"* — a refactor, not a feature introduction.
- WasmStore IS the current API in v0.26.8 docs (per https://docs.rs/tree-sitter/latest/tree_sitter/) and **is gated behind a `wasm` feature flag** ("Requires the feature `wasm` to be enabled").
- WasmStore was likely introduced earlier (v0.20.x or pre-v0.22). The v0.22 claim is approximate/wrong — the actual claim should be "available since at least v0.22 (2024-03), current as of v0.26.8 (2026-03)".

**Implication for r3:**
- **Fix the version claim** to: "tree-sitter Rust crate (v0.26.8 latest, May 2026), with WasmStore behind the `wasm` feature flag".
- **Mention the WASI SDK migration** (v0.26.1, 2025/2026): tree-sitter migrated WASM compilation from Emscripten to WASI SDK. This affects how grammars get compiled (`tree-sitter build --wasm` now uses WASI SDK by default). Important if the plugin's CI builds grammars from upstream sources.
- Pin a conservative version (e.g. v0.26.x) and watch for the next breaking change.

**Source:** https://github.com/tree-sitter/tree-sitter/releases (accessed 2026-05-08); https://docs.rs/tree-sitter/latest/tree_sitter/ (accessed 2026-05-08); https://github.com/tree-sitter/tree-sitter/releases/tag/v0.22.0 (accessed 2026-05-08).

### Claim 2: Tree-sitter grammar WASM availability

**r2 quote:** *"Sourced from upstream tree-sitter grammar releases or built locally via `tree-sitter build --wasm`."*

**Verified status:**
- **No official upstream pre-compiled WASM grammar repository.** Each grammar repo (e.g. `tree-sitter/tree-sitter-rust`) ships source, not pre-built `.wasm`.
- **Multiple community pre-build repositories exist** as of 2026:
  - **Microsoft `vscode-tree-sitter-wasm`** (https://github.com/microsoft/vscode-tree-sitter-wasm) — VS Code's pre-built grammars. Quasi-official.
  - **`wasm-lsp/tree-sitter-wasm`** (https://github.com/wasm-lsp/tree-sitter-wasm) — community-maintained.
  - **`Menci/tree-sitter-wasm-prebuilt`** (https://github.com/Menci/tree-sitter-wasm-prebuilt) — community.
  - **`kreuzberg/tree-sitter-language-pack-wasm`** — npm/jsdelivr CDN distribution covering 305+ languages.
- **`tree-sitter build --wasm`** is the canonical local-build path. Per v0.26.1 release notes, this now uses **WASI SDK** instead of Emscripten.

**Implication for r3:**
- For provenance discipline (r2 Q8), recommend **build-locally-from-pinned-source** (option b) as the architect leans. Prebuilt mirrors (option a) are a fallback for projects without WASI SDK toolchain.
- Note that "tree-sitter ships pre-compiled grammars" is **NOT TRUE** — that part of r2 should be tightened.

**Source:** https://github.com/microsoft/vscode-tree-sitter-wasm; https://github.com/wasm-lsp/tree-sitter-wasm; https://github.com/Menci/tree-sitter-wasm-prebuilt (all accessed 2026-05-08).

### Claim 3: wasmtime binary size (~10MB)

**r2 quote:** *"wasmtime adds ~10MB to the plugin binary."*

**Verified status:**
- **Wasmtime minimal-build C API library is ~2.1MB**, per https://docs.wasmtime.dev/examples-minimal.html. With `OPT_LEVEL=s` and `PANIC=abort`, drops to ~2.0MB.
- WASI implementation is the bulk (~1MB of remaining size).
- Wasmtime current version: **40.x** (40.0.1 released 2026-01-07; subsequent releases through 2026-04). Requires Rust 1.92.0+ to build.
- The `legacy-bash-adapter.wasm` and other WASM hook plugins in vsdd-factory currently embed wasmtime via the Rust SDK — if the binary size cost is real, the architect should know it's already paid (not new).

**Implication for r3:**
- **Fix the size claim** from ~10MB to ~2-3MB (closer to reality; depends on feature flags).
- Note that this cost is **already absorbed** by the existing factory-dispatcher binary — the WASM substrate is in production.
- If size is truly a concern, the architect can mention that Wasmtime's feature flags allow tuning (`disable-logging`, dropping WASI when not needed, etc.).

**Source:** https://docs.wasmtime.dev/examples-minimal.html (accessed 2026-05-08); https://crates.io/crates/wasmtime; https://github.com/bytecodealliance/wasmtime/releases.

### Claim 4: wasm-bindgen / wasm32-unknown-unknown for Rust → WASM

**r2 quote:** *"Rust → wasm32 is straightforward (`cargo build --target wasm32-unknown-unknown`)."*

**Verified status:**
- `wasm32-unknown-unknown` is **Tier 2 supported** in Rust stable; well-known and reliable.
- For tree-sitter ABIs / WASI integration, modern toolchain often prefers **`wasm32-wasi` / `wasm32-wasip1` / `wasm32-wasip2`** (the latter being the Component Model target).
- `wasm-bindgen` is a JavaScript-interop tool, **not generally needed** for Wasmtime-hosted plugins (it's for browser/Node integration).

**Implication for r3:**
- Statement is essentially correct but architecturally sloppy. r3 should clarify: **for Wasmtime-hosted plugins, target is either `wasm32-unknown-unknown` (raw exports / simple ABI) or `wasm32-wasip2` (Component Model)**. wasm-bindgen is for the browser path and not relevant here.
- The existing factory-dispatcher already runs Rust-to-WASM hook plugins; the toolchain is proven in this plugin.

**Source:** https://doc.rust-lang.org/rustc/platform-support.html (general Rust knowledge, not separately fetched).

### Claim 5: AssemblyScript current ecosystem state

**r2 quote:** *"AssemblyScript is approachable for non-Rust authors."*

**Verified status:**
- AssemblyScript is **alive and maintained** in 2026, but has **notable limitations vs Rust**:
  - **No closures** (waiting on Function References / GC proposals).
  - **Limited stdlib** vs TypeScript.
  - Targets WebAssembly directly via Binaryen.
- **WebAssembly 3.0 became a W3C standard September 2025** — broader feature set (WasmGC, exception handling, tail calls, 64-bit memory, 128-bit SIMD) standardized, which AssemblyScript benefits from incrementally.

**Implication for r3:**
- AssemblyScript IS a viable secondary toolchain for non-Rust check authors, but **call out the limitations** (no closures, stdlib gaps).
- The closure limitation matters for tree-sitter query-result-callback patterns. r3 should mention this as a known restriction.

**Source:** https://www.assemblyscript.org/status.html (accessed 2026-05-08); https://github.com/AssemblyScript/assemblyscript/wiki/Status-and-Roadmap.

### Claim 6: TinyGo for WASM target

**r2 quote:** *"TinyGo works for Go authors."*

**Verified status:**
- TinyGo's WebAssembly guide last updated 2026-04-20 — **actively maintained**.
- Supports `wasm-unknown` target (analogous to Rust's `wasm32-unknown-unknown`).
- **Notable limitations:**
  - `net/http`, `encoding/json`, anything reflection-heavy: partial or no support.
  - Reflection itself: limited.
  - Goroutines: cooperative scheduler, not Go's preemptive scheduler.
- Significant Go binaries shrink dramatically with TinyGo (Go's runtime + GC are heavy).

**Implication for r3:**
- TinyGo is a **viable third-tier toolchain** but with substantial caveats. Probably overstated as "approachable" in r2 — Go authors writing WASM hooks WILL hit reflection/stdlib gaps.
- **r3 recommendation:** position Rust as primary (already proven in vsdd-factory), AssemblyScript as secondary for TS-fluent authors, TinyGo as tertiary for Go-fluent authors. Don't oversell.

**Source:** https://tinygo.org/docs/guides/webassembly/ (last updated 2026-04-20).

### Claim 7: WebAssembly Component Model maturity (r2 punts to "v2 ABI")

**r2 quote:** *"WIT (WebAssembly Interface Types) and the Component Model would be more ergonomic than raw exports, but the tooling is still maturing as of 2026. Recommend starting with raw exports, then migrating to WIT once the Component Model stabilizes."*

**Verified status (most-changed-since-r2 claim):**
- **Component Model is production-ready in 2026 for server-side and edge workloads built on WASI 0.2.**
- WASI Preview 2 is **stable**.
- `wit-bindgen` reads WIT files and generates language-specific bindings. Per techbytes.app and javacodegeeks.com (both 2026): "wit-bindgen generates clean bindings, and the Component Model experience is polished."
- Rust support is best-in-class via `cargo-component` and direct `wasm32-wasip2` compilation.
- **Production adopters (2026):** American Express FaaS on wasmCloud, Fermyon's 75M RPS edge platform, Docker's 7-runtime Wasm support.
- **Caveats:** "general-purpose backend microservices" still constrained by unresolved threading in WASI; not relevant to this plugin's use case (single-shot hook checks, not multi-threaded servers).

**Implication for r3:**
- **r2's "raw exports as v1, WIT as v2" recommendation is OUTDATED.** The Component Model + `cargo-component` + `wit-bindgen` is the stable, recommended path in 2026.
- r3 should **lead with the Component Model + WIT** for the v1 ABI, not punt it to v2. Existing Rust hook plugins in vsdd-factory could already migrate to `wasm32-wasip2` if there's value.
- The factory-dispatcher would need to support component model embedding (Wasmtime supports it natively in current versions).
- **Open question for architect:** does this require any retrofit of existing hook-plugins crates, or can new hooks adopt Component Model alongside legacy raw-export hooks?

**Source:** https://component-model.bytecodealliance.org/ (accessed 2026-05-08); https://github.com/bytecodealliance/wit-bindgen; https://techbytes.app/posts/wasm-component-model-cheat-sheet/ (2026); https://www.javacodegeeks.com/2026/04/webassembly-in-2026-three-years-of-almost-ready.html (2026-04).

### Claim 8: Claude Code plugin hook system

**r2 quote (implied):** Layer 1 hooks live in projects' `lefthook.yml` invoking helpers shipped from the plugin.

**Verified status (extensive):**

Claude Code's hook system has **substantially more event types than r2 implies**. From https://code.claude.com/docs/en/hooks (and corroborated by https://code.claude.com/docs/en/plugins-reference), the full event list is:

| Event | Trigger | Relevant for r3? |
|---|---|---|
| `SessionStart` | Session begins/resumes | Maybe (init-time audit) |
| `Setup` | `--init-only` / `--init` / `--maintenance` | Likely YES — perfect place for `audit-runtime-wiring` precondition |
| `UserPromptSubmit` | User submits a prompt | No |
| `UserPromptExpansion` | Slash command expands | Possibly (for guardrails on `/vsdd-factory:*` invocations) |
| `PreToolUse` | Before any tool call | YES — primary substrate for r2's "stub residue" check on Edit |
| `PermissionRequest` | Permission dialog | No |
| `PermissionDenied` | Tool denied | No |
| `PostToolUse` | After tool succeeds | YES — primary substrate for "story-frontmatter ↔ STORY-INDEX" sync |
| `PostToolUseFailure` | After tool fails | No |
| `PostToolBatch` | After batch of parallel calls | Maybe (for batch-edit scenarios) |
| `Notification` | Notification sent | No |
| `SubagentStart` | Subagent spawned | YES — already used by `validate-pr-merge-prerequisites`/`validate-wave-gate-prerequisite` |
| `SubagentStop` | Subagent finishes | YES — already used by 5 existing hooks |
| `TaskCreated` | TaskCreate fired | No |
| `TaskCompleted` | Task marked done | No |
| `Stop` | Claude finishes turn | YES — already used by `session-learning`, `warn-pending-wave-gate` |
| `StopFailure` | Turn ended due to API error | No |
| `TeammateIdle` | Agent team teammate going idle | No |
| `InstructionsLoaded` | CLAUDE.md / .claude/rules/*.md loaded | Maybe |
| `ConfigChange` | Configuration file changes | Maybe |
| `CwdChanged` | cd executed | No |
| `FileChanged` | Watched file changed on disk | YES — strong candidate for stub-residue and STORY-INDEX coherence checks (no need to wait for edit-cycle) |
| `WorktreeCreate` / `WorktreeRemove` | Worktree lifecycle | YES — already used |
| `PreCompact` / `PostCompact` | Context compaction | No |
| `Elicitation` / `ElicitationResult` | MCP elicitation | No |
| `SessionEnd` | Session ends | YES — already used |

**Hook handler types (from same docs):**
- `command` (shell command)
- `http` (POST event JSON to URL)
- `mcp_tool` (call MCP server tool)
- `prompt` (evaluate a prompt with LLM, uses `$ARGUMENTS`)
- `agent` (run agentic verifier)

**The `prompt` and `agent` types are notable** — they suggest semantic checks could be hook-implemented via LLM, not just bash. r2 doesn't mention these. r3 might consider whether some "iron rules" could be implemented as `agent`-type hooks at PreToolUse instead of being baked into agent prompts.

**Plugin hooks live at:** `hooks/hooks.json` (relative to plugin root), per docs. **The vsdd-factory plugin uses a non-default pattern** — single `hooks.json.template` plus `hooks-registry.toml` and the factory-dispatcher binary that internally routes. This works because Claude Code's `command` hook can be any executable.

**The `if` field on hook handlers** supports permission-rule syntax for conditional firing (e.g. `"if": "Bash(git *)"`). r2 doesn't use this; r3 could leverage it for the stub-residue check (only fire on Edit to `crates/*/src/`-pattern paths).

**Matchers support** literal strings, `|`-separated lists, or full JS regex (any non-alphanumeric character triggers regex mode). r3 should specify matchers carefully.

**Implication for r3:**
- The taxonomy r3 proposes (CC hooks vs lefthook vs agent-prompt vs scheduled-skill) is **roughly correct** but should be expanded to acknowledge:
  - CC hooks have ~25 distinct event types, not just PreToolUse/PostToolUse.
  - CC hooks support 5 handler types (command/http/mcp_tool/prompt/agent), so "agent-prompt invariants" CAN be a hook (the `agent` type) in addition to being a baked-in agent rule.
  - The plugin already has the `SubagentStart`/`SubagentStop` infrastructure — perfect for graduation-contract-style checks.

**Source:** https://code.claude.com/docs/en/hooks (accessed 2026-05-08); https://code.claude.com/docs/en/plugins-reference (accessed 2026-05-08).

### Claim 9: Claude Code plugin distribution model

**r2 quote (implied):** plugin is shipped via marketplace.

**Verified status:**
- Plugin distribution channels (per plugins-reference):
  - **`claude plugin install <name>@<marketplace>`** — marketplace install.
  - **`claude --plugin-dir ./local-path`** — local development install (per-session).
  - **`claude --plugin-url https://example.com/plugin.zip`** — URL-fetch (per-session).
  - **Git source / git-subdir source / npm source** — marketplace mechanism.
- Per-version cache: `~/.claude/plugins/cache/<plugin>@<version>/`. Old versions retained 7 days.
- `${CLAUDE_PLUGIN_ROOT}` resolves to the cache install dir. **`${CLAUDE_PLUGIN_DATA}` resolves to `~/.claude/plugins/data/<id>/`** for persistent state.
- Manifest at `.claude-plugin/plugin.json`, version field optional (git SHA used as fallback).
- Path-traversal limitation: plugins **cannot reference files outside their cache directory** unless symlinked at packaging time.

**Implication for r3:**
- Bash scripts at `${CLAUDE_PLUGIN_ROOT}/hooks/<name>.sh` work correctly (vsdd-factory already does this).
- But **plugin scripts cannot read the consumer project's files arbitrarily** — they must take the file path via `tool_input.file_path` JSON input on stdin (standard CC hook contract). This is what existing scripts do (`INPUT=$(cat); FILE_PATH=$(echo "$INPUT" | jq -r '.tool_input.file_path // empty')`).
- For r2's "scan all crates/*/src/" stub-residue audit, the script reads from `$CLAUDE_PROJECT_DIR` (the consumer's project root), which is set in the environment by Claude Code when the hook is invoked. This is already a standard pattern in existing hooks.

**Source:** https://code.claude.com/docs/en/plugins-reference (accessed 2026-05-08).

### Claim 10: lefthook integration with bash scripts shipped from a plugin path

**r2 quote (implied):** "lefthook.yml (or git pre-push hook on non-lefthook projects), invoking helper binaries shipped by the vsdd-factory plugin."

**Verified status:**
- Lefthook **does support** referencing remote / shared configurations from a git repository in the local `lefthook.yml`. Per evilmartians/lefthook docs and community patterns: "centralize hook configurations in a shared repo and reference it from each project."
- Lefthook configs can shell out to **any** path the user's shell can reach. Referencing `${CLAUDE_PLUGIN_ROOT}/hooks/<script>.sh` is **viable** ONLY if `CLAUDE_PLUGIN_ROOT` is set in the developer's git environment, which it is NOT by default — `CLAUDE_PLUGIN_ROOT` is set by Claude Code at hook-invocation time, not by `git commit`.

**Implication for r3 (this is a real architectural snag):**
- **Plugin-shipped bash scripts work fine for CC hooks** (CLAUDE_PLUGIN_ROOT is set).
- **Plugin-shipped bash scripts do NOT trivially work for lefthook hooks** running outside Claude Code (CLAUDE_PLUGIN_ROOT is unset in `git commit` shell).
- Three resolution options for r3:
  1. **Vendor the script into the project** — `vsdd-factory:install-hooks` skill copies `hooks/<name>.sh` into the project's `.factory/hooks/<name>.sh`, and the lefthook.yml references the project-relative path. This is mechanically what already happens for `verify-sha-currency.sh` (per state-manager.md).
  2. **Ship a Rust binary on $PATH** — plugin's `bin/<tool>` is on PATH while plugin enabled (per plugin spec). Lefthook can invoke `<tool> --check-stub-residue` and the project doesn't need a copy. **But this only works while Claude Code is running the lefthook context — most `git commit`s happen outside Claude Code.**
  3. **Ship a standalone CLI** that's installed independently (e.g. `cargo install vsdd-check`) and lefthook references that binary. This is the cleanest pattern but requires a separate distribution path from the plugin.

The architect should pick option (1) or (3); option (2) is fragile.

**Source:** https://github.com/evilmartians/lefthook (accessed 2026-05-08); https://github.com/spellbookx/lefthook.

---

## Part 3 — Architectural restructure proposal (validation)

### Per-substrate validation

The proposed r3 4-substrate split:

#### Substrate 1: Claude Code hooks (PreToolUse/PostToolUse via plugin manifest)

**Verified.** Real, mature, in-production. Latency: ms. Fires on: Claude tool calls (Edit/Write/Bash/Agent/etc.). The plugin already has 40+ hooks running here.

**But**: the substrate is broader than just PreToolUse/PostToolUse — there are **~25 distinct event types** (see Claim 8). r3 should expand the taxonomy to acknowledge:
- `PreToolUse` / `PostToolUse` — primary
- `SubagentStart` / `SubagentStop` — for graduation/convergence checks (ALREADY USED in vsdd-factory for wave-gate and adversary-convergence)
- `FileChanged` — interesting alternative for STORY-INDEX coherence (fires on disk-change, not edit)
- `Setup` — perfect for one-time init-time audits

#### Substrate 2: Git hooks (lefthook)

**Verified.** Real but with a **distribution snag** (see Claim 10). Shipping bash scripts from the plugin to lefthook is non-trivial — the architect must pick a resolution.

Lefthook example from Prism's own `lefthook.yml` (`/Users/jmagady/Dev/prism/lefthook.yml`):
```yaml
pre-commit:
  parallel: true
  commands:
    fmt: { glob: "*.rs", run: cargo fmt --all --check }
    clippy: { glob: "*.rs", run: cargo clippy --all-features -- -D warnings }
    layout: { glob: "crates/**", run: scripts/check-crate-layout.sh }
pre-push:
  commands:
    check: { run: just check }
pre-tag:
  commands:
    semver-checks: { run: just semver-checks }
    audit: { run: just audit }
    deny: { run: just deny }
```
Note the pattern: **scripts are project-local** (`scripts/check-crate-layout.sh`) and `just` recipes encapsulate workspace-level checks. r3's plugin-shipped scripts must adapt to this pattern via Option 1 (vendor into project) or Option 3 (separate CLI).

#### Substrate 3: Agent-prompt invariants

**Verified.** Real, mature. The plugin already has 36 agent prompt files with detailed sub-rules.

But: as noted under Claim 8, **CC hooks support `agent` and `prompt` handler types**, meaning some "iron rules" could be implemented as agent-type CC hooks instead of (or in addition to) being baked into agent prompts. The architect should consider:
- **Pure agent-prompt invariants:** silent-shallow scan in test-writer.md (only the test-writer agent runs this).
- **Hook-driven semantic checks:** could fire on every Edit (not just test-writer's turn) via PreToolUse `agent`-type hook.

Latency: minute-scale per agent dispatch (matches r2).

#### Substrate 4: Scheduled skills

**PARTIALLY-INCORRECT.** Skills themselves do NOT have a "schedule" attribute. Skills are model-invoked or manually invoked. The "scheduling" comes from:
- **External cron** (GitHub Actions, system cron, etc.) running `claude --plugin-dir … -p "/vsdd-factory:audit-stub-debt"`.
- **Plugin `monitors/monitors.json`** — but those run for the session lifetime, not on cron.
- **Wave-gate trigger** — the wave-gate skill could explicitly invoke audit skills as part of its workflow.

r3 should **rename "scheduled skills" → "out-of-band audit skills"** or "periodic skills (invoked by external scheduler)" to be precise. Latency: as cron fires (day/week).

### Per-check placement validation

| Check | r3 proposes | Validated? | Notes |
|---|---|---|---|
| `check-stub-residue` | CC hook + lefthook (both) | **CONFIRM, with caveat.** CC hook (PreToolUse Edit) is ms-latency; lefthook pre-push is the second-line gate. Both make sense. Caveat: the lefthook script must be vendored into the project (see Substrate 2). |
| `check-inverted-polarity` | CC hook + lefthook (both) | **CONFIRM, with caveat.** Same pattern. Strong synergy with existing `validate-red-ratio.sh` — extend rather than parallel implement. |
| `check-story-index-consistency` | CC PostToolUse hook | **CONFIRM, but consider FileChanged alternative.** PostToolUse on Edit/Write fires on Claude-driven edits only. FileChanged fires on any disk change (including manual git operations). Either is valid; PostToolUse matches existing pattern. Existing `validate-state-index-status-coherence.sh` is the closest analog — r3 should EXTEND it rather than create a new sibling, OR be explicit about why a new sibling is preferred. |
| `check-bc-promotion` | agent-prompt rule (state-manager) | **REJECT placement; proposes architecturally fraught path.** state-manager is currently strict bookkeeper, NEVER writes specs. Auto-promoting BC `status: draft → active` IS writing spec content. r3 should consider:<br> - Option A: `update-bc-status-on-story-merge.sh` SubagentStop hook (fires when pr-manager finishes a merge subagent) — NO agent involvement. Cleanest.<br> - Option B: New `bc-promoter` agent (not state-manager) — clear scope.<br> - Option C: Incremental scope expansion of state-manager with explicit allowlist for promotion writes only. |
| `audit-runtime-wiring` | scheduled skill | **CONFIRM, with rename.** Should be "out-of-band audit skill (cron-invoked)" to be precise. Could also be a sub-sweep of `maintenance-sweep` instead of a top-level skill — open architect decision. |

### Substrate-cost-vs-latency matrix (corrected)

| Substrate | Latency | Trust | Existing in plugin? | Use for r3 |
|---|---|---|---|---|
| CC hook PreToolUse (Edit\|Write) | ms (synchronous) | Plugin-trust | YES (40+ hooks) | Stub-residue at edit time, inverted-polarity at edit time |
| CC hook PostToolUse (Edit\|Write) | ms | Plugin-trust | YES | Story-index coherence at write time |
| CC hook SubagentStop | ms | Plugin-trust | YES (5 hooks) | BC/VP promotion on pr-manager finish, story-completion gates |
| CC hook FileChanged | ms | Plugin-trust | NO (not currently used) | Out-of-Claude disk changes (e.g. user runs `git pull` and STORY-INDEX changes) |
| CC hook `agent`-type | minute | Plugin-trust | NO | Semantic checks (could replace some agent-prompt invariants) |
| Lefthook pre-commit | ms | Project-trust | YES in Prism, NO standardized via plugin | Stub-residue, inverted-polarity at git-commit time |
| Lefthook pre-push | ms-second | Project-trust | YES in Prism (`just check`) | Heavy checks before push |
| Agent prompt invariant | per-dispatch (minute) | Plugin-trust | YES (36 agents) | Per-agent context-aware rules |
| External cron + skill | day/week | Project-trust | YES (maintenance-sweep) | Workspace-wide audits |
| Wave-gate trigger | per-wave (~weekly) | Plugin-trust | YES | Composite audit at wave boundaries |

---

## Part 4 — Open architectural questions for r3

### Q1: Capability model for new hooks

vsdd-factory's `hooks-registry.toml` already declares per-hook capabilities (`read_file.path_allow`, `exec_subprocess.binary_allow`, etc.). The four new bash hooks need their capability blocks specified.

**For r3:**
- `pre-push-stub-residue.sh` needs: read-file glob (`crates/*/src/**`, or generic source-glob), exec_subprocess `rg` and `git`.
- `pre-burst-story-index-consistency.sh` needs: read-file `.factory/stories/**/*.md`, `.factory/stories/STORY-INDEX.md`.
- `pre-push-inverted-polarity.sh` needs: read-file `tests/**/*.rs` (and equivalents for other languages if multi-lang), exec `rg`.
- `pre-push-doc-freeze.sh` needs: read-file source globs, exec `rg`.

If implemented as native WASM (instead of via `legacy-bash-adapter`), the capability declarations stay similar but the hook is a Rust crate at `crates/hook-plugins/<name>/`.

**Default capability for plugin-shipped checks:** read_file granted for the file being scanned; exec_subprocess gated to specific binaries (`rg`, `git`, `jq`, `bash`); env_allow restricted to standard CC hook env (`PATH`, `HOME`, `TMPDIR`, `CLAUDE_PROJECT_DIR`, `CLAUDE_PLUGIN_ROOT`, `VSDD_SESSION_ID`).

### Q2: Component Model adoption (v1 vs v2 ABI)

**r2 says "raw exports v1, WIT v2".** Per Claim 7 verification, the Component Model is production-ready in 2026 and Rust support is best-in-class. r3 has a real choice:

- **Stay with raw exports v1** for compatibility with existing factory-dispatcher hooks (which use the `HOST_ABI` defined in `crates/hook-sdk/HOST_ABI.md`). New hooks fit alongside existing ones. **Lower risk; no retrofit cost.**
- **Adopt Component Model v2** for new hooks while maintaining v1 compatibility. Wasmtime supports both. New crate template uses `cargo-component`. **Better authoring ergonomics; some plugin-host work to support both.**

Architect decision required.

### Q3: Grammar provenance (mirror upstream vs build locally vs hybrid)

Plus the WASI SDK migration (tree-sitter v0.26.1+) — building grammars locally requires WASI SDK installed in plugin CI.

**r3 options (per r2 Q8):**
- (a) Mirror upstream prebuilt
- (b) Build locally from pinned commits
- (c) Both

r2's open recommendation is (b). With WASI SDK migration, (b) costs slightly more in CI setup but yields full provenance. Architect should confirm.

### Q4: Plugin discovery / how Claude Code finds vsdd-factory

Per docs, Claude Code finds plugins via:
- `claude plugin install <name>@<marketplace>` — needs a marketplace.
- `claude --plugin-dir ./vsdd-factory/plugins/vsdd-factory` — local dev.
- `claude --plugin-url https://...` — URL fetch.
- Configured in user/project/local `settings.json` `enabledPlugins`.

For r2's proposed scenario ("vsdd-factory plugin in v-next"), the plugin must be registered in a marketplace OR the consuming project enables it via `settings.json`. The Prism project's `settings.local.json` (in `.claude/`) presumably enables vsdd-factory. r3 doesn't need to design this — it's already wired.

### Q5: Bash script distribution path (Layer 1 lefthook concern)

Already covered in Claim 10. Architect picks Option 1 (vendor) or Option 3 (separate CLI).

**Recommendation embedded in research:** Option 1 (vendor) matches existing `verify-sha-currency.sh` pattern (state-manager.md mentions instantiating from `templates/verify-sha-currency.sh` into `.factory/hooks/verify-sha-currency.sh`). r3 can lean on this convention.

### Q6: state-manager scope expansion

r2 says state-manager auto-promotes BC `draft → active` on story merge. state-manager.md explicitly says: *"NEVER write specification documents or source code -- state tracking only."*

**Three options for r3:**
- (A) **Hook-driven** — `update-bc-status-on-story-merge.sh` runs on SubagentStop matching pr-manager, no agent involvement. Cleanest separation of concerns.
- (B) **New agent** — `bc-promoter` agent with narrowly scoped write permissions. Adds one agent to manage.
- (C) **Scoped state-manager exception** — state-manager gets explicit allowlist for BC frontmatter status field only. Risks scope creep.

Architect should pick (A) — hook-driven — based on the existing pattern (e.g. `update-wave-state-on-merge.wasm`).

### Q7: Policy ID renumbering

r2 proposes POL-12 through POL-16. POL-12 through POL-? may be used in vsdd-factory's own internal policies.yaml (POL-11 and POL-12 confirmed taken).

**r3 should:**
- Read the **consuming project's** current policies.yaml to find next free ID.
- Or scope these as **plugin-baseline** (POL-001 through POL-005 in a plugin-shipped baseline policies.yaml that gets merged into project-level via init).

Architect picks one approach.

### Q8: Schema for `partial-merge` enum

Companion proposal `vsdd-stub-merge-policy-2026-05-08.md` covers this. r3 should reference, not duplicate. **Not researched in this pass** — the architect should ensure r3 keeps the reference/duplication boundary clear.

### Q9: Maintenance-sweep merger vs new-skill creation

Existing `maintenance-sweep` skill already has Sweep 1/2/3 (deps, doc drift, pattern consistency). r2's audit-stub-debt and audit-runtime-wiring fit naturally as Sweep 4/5/6.

**r3 decision:** make them sub-sweeps of maintenance-sweep (less surface area, single periodic invocation), OR make them top-level skills (more granular, can be invoked individually).

**Recommendation:** create them as top-level skills (per r2) AND have maintenance-sweep call them (composition). Each is independently invokable; maintenance-sweep is the periodic-invocation gateway.

### Q10: Renaming "Layer 1: Hooks"

r2's "Layer 1: Hooks" is the muddled framing the user explicitly flagged. r3 must split it. Recommended naming for the 4 substrates (per the user's brief):

| Substrate | Latency | r3 name option |
|---|---|---|
| CC plugin hooks (PreToolUse/PostToolUse/SubagentStop/etc.) | ms | "Substrate 1A: Claude Code Hooks (synchronous, AI-action-time)" |
| Lefthook git hooks | ms | "Substrate 1B: Git Hooks (synchronous, commit/push-time)" |
| Agent-prompt invariants | minute | "Substrate 2: Agent-Prompt Invariants (per-dispatch, agent-time)" |
| External-scheduler skills | day/week | "Substrate 3: Scheduled Audit Skills (cron-invoked, workspace-time)" |

r2's three layers (Hooks, Iron Rules, Red Flags) become four substrates with clearer latency/scope distinctions. Layer 2 (Iron Rules) maps to Substrate 2. Layer 3 (Red Flags) maps to Substrate 3. Layer 1 (Hooks) splits into 1A and 1B.

**Per-check substrate placement (corrected from r3 proposal):**

| Check | Substrate placement |
|---|---|
| `check-stub-residue` | 1A (PreToolUse Edit) AND 1B (lefthook pre-commit/pre-push) |
| `check-inverted-polarity` | 1A AND 1B (likely extend existing `validate-red-ratio.sh` rather than parallel implement) |
| `check-story-index-consistency` | 1A (PostToolUse Edit/Write) — extend or sibling existing `validate-state-index-status-coherence.sh` |
| `check-bc-promotion` | **1A (SubagentStop matching pr-manager)** — NOT Substrate 2 (state-manager). Hook-driven, no agent involvement. Cleanest. |
| `audit-runtime-wiring` | 3 (cron-invoked or wave-gate-triggered audit skill) |
| `audit-stub-debt` | 3 |
| `audit-orphan-configs` | 3 |
| `audit-vp-promotion` | 3 |

---

## Summary of major r2 deltas r3 must address

1. **Layer 1 muddling.** Split into 1A (CC hooks) and 1B (lefthook). Both substrates host stub-residue and inverted-polarity checks (r2 already says this); the framing should make the distinction explicit.

2. **WASM substrate is shipped.** r2 reads as if it must be built (Phase 3, ~3 weeks). It's already in production via factory-dispatcher + 17 native-WASM hook plugins + 33 legacy-bash-adapter shims. r3 should propose ADDING new hooks to the existing substrate, not building substrate.

3. **Component Model is production-ready.** r2's "raw exports v1, WIT v2" recommendation is outdated. Architect should choose: (a) raw exports for compatibility with existing hook crates, or (b) Component Model for new hooks alongside.

4. **Tree-sitter version claim is approximate/wrong.** v0.22 (2024) ≠ "WasmStore introduction". WasmStore is current behind a `wasm` feature flag in v0.26.8 (March 2026). WASI SDK migration in v0.26.1 affects grammar build path.

5. **Wasmtime size claim is wrong.** ~2-3MB minimal, not ~10MB. And the cost is already paid by the existing factory-dispatcher.

6. **Policy ID collision.** r2's POL-12 collides with existing POL-12 (`bc_tv_emitter_consistency`). Renumber to POL-13+ OR introduce plugin-baseline policy file.

7. **state-manager BC promotion architecturally fraught.** state-manager is bookkeeper-only. r3 should propose hook-driven promotion (SubagentStop on pr-manager) rather than expanding state-manager scope.

8. **"Scheduled skills" terminology imprecise.** Skills don't have a schedule field. The scheduling layer is external (cron / GitHub Actions / wave-gate trigger). r3 should be explicit.

9. **Lefthook bash-script distribution snag.** Plugin-shipped bash scripts don't have CLAUDE_PLUGIN_ROOT in lefthook context. r3 must pick: vendor-on-init (matches existing `verify-sha-currency.sh` template pattern) or separate CLI distribution. Recommend vendor-on-init.

10. **Existing analogs** for r2 deliverables that r3 should explicitly extend rather than greenfield:
    - r2 stub-residue gate ↔ existing `red-gate.sh` (different but related)
    - r2 inverted-polarity ↔ existing `validate-red-ratio.sh`
    - r2 story-index-coherence ↔ existing `validate-state-index-status-coherence.sh`
    - r2 silent-shallow scan ↔ existing POL-11 `no_test_tautologies` + validate-consistency Check 8
    - r2 audit-stub-debt ↔ existing maintenance-sweep skill (compose, don't replace)

---

## End of findings

The architect composing r3 should treat this document as ground truth for plugin facts and for technology claim verification. Anywhere this doc says "open architectural question", the architect makes the call. Anywhere this doc cites a source, that's the authoritative basis (don't restate without citation).
