---
artifact_type: proposal
target_repo: vsdd-factory
target_repo_url: TBD (Joshua's vsdd-factory plugin repo)
date: 2026-05-08
revision: 2
revision_history:
  - r1 (2026-05-08): initial — Plan 1 (3-layer model) + Plan 2 (YAML+ripgrep substrate)
  - r2 (2026-05-08): added Plan 3 (WASM extension substrate, day-1 commitment); updated build sequencing; added Q6-Q10
status: draft-for-export
self_contained: true
prior_context_required: false
companion_proposal: vsdd-stub-merge-policy-2026-05-08.md
prism_audit_reference: ../cycles/wave-4-operations/workspace-audit-2026-05-08.md
---

# VSDD Prevention Layers — Stub-Merge / Status-Drift / Runtime-Gap Defense

**For:** Claude Code session in the `vsdd-factory` plugin repo, with **no prior context** about the Prism project, the Wave 3 cascade, or the audits that motivated this proposal. This document is the entire input.

**You are receiving:** three plans. Plan 1 is the three-layer prevention model expressed in language-anchored (Rust) form, ready to land as a vsdd-factory v-next feature. Plan 2 is a YAML+ripgrep substrate that makes Plan 1's syntactic checks language-agnostic at low cost. Plan 3 is the WASM extension substrate the plugin is committed to from day 1 — tree-sitter grammars and custom checks both ship as WASM modules, providing one extensibility model for everything compiled.

---

## TL;DR

A workspace audit found 53 findings (18 P0) in a VSDD-managed Rust project. Stories were marked `merged` in the index while their production code paths were still `todo!()` panics. Tests were silent-shallow (asserting on mocks instead of production SUTs). At least one test was *inverted-polarity* — `#[should_panic(expected = "not yet implemented")]` — meaning green CI required production code to stay broken.

The convergence ceremony (3-CLEAN adversarial passes) **did not catch this**. The adversary correctly converged on what was inside the perimeter shown to it; nobody checked whether the perimeter was right.

This is a **defense-in-depth failure**. It needs three layers, not one.

- **Plan 1** delivers the three layers concretely for a Rust project, ~1 week of plugin work.
- **Plan 2** describes how to make Plan 1's syntactic hooks/rules/audits language-agnostic via a YAML+ripgrep substrate that handles ~90% of the value at low cost.
- **Plan 3** describes the WASM extension substrate the plugin is built on from day 1 — tree-sitter grammars and custom checks both shipped as WASM modules, one extensibility model for everything compiled. WASM is committed-to from day 1 of the plugin, not migrated-to later.

Recommended build order: **Plan 1 first** (proves the layered model on one language); **Plan 2 as a parallel cycle** (cheap multi-language coverage for syntactic checks); **Plan 3 in parallel with Plan 2 once Plan 1 ratifies the model** (WASM substrate + tree-sitter for the deeper structural checks). No native-then-migrate dance.

---

## Background — The Failure Case

A VSDD-managed Rust workspace, 24 crates, ~145 verification properties, ~222 behavioral contracts, ~50+ stories, late Phase 3.

**What the project's STORY-INDEX claimed:**
- S-3.02 (query engine): MERGED PR #129
- S-3.07 (write engine): MERGED PR #135
- S-1.11/12/14/15 (plugin/spec-engine, hot-reload, infusions): MERGED

**What a fresh-context audit found:**
- `QueryEngine::execute`, `run_materialization_pipeline`, `RocksDbTableProvider` — all `todo!()` panics. 5 sites in 3 files.
- `WriteExecutor::execute` Phase 3 fetch was hardcoded `vec![]`. No concrete `SensorAdapter::write()` override existed; default returned `WriteNotImplemented`. SQL DML returned `NotImplemented`.
- `prism-mcp` (the entire MCP server crate, mapped to an accepted ADR claiming 35 tools): **10 lines of placeholder code**.
- 222/222 behavioral contracts were `status: draft`. Only 2/145 verification properties were `verified`. Promotion criteria undefined.
- One test in `prism-spec-engine/tests/hot_reload_tests.rs` actively codified the stub: `#[should_panic(expected = "not yet implemented")]`. The test would BREAK the day someone implemented the real function.
- 4 sensor TOML files were write-only (read-side `[[tables]]` never delivered). No production binary called `parse_spec_directory`, `ConfigManager::new`, `register_internal_tables`, or `WriteEndpointRegistry::new`. The runtime was unwired even though every individual story passed adversarial review.
- STORY-INDEX rows said MERGED while the same stories' frontmatter said `status: ready` or `status: draft`. Two systems of truth, no reconciliation hook.

**Root cause taxonomy** (5 patterns):
1. Stub-merge convention enforced inconsistently (some stories properly tagged deferrals via `#[ignore = "TD-..."]`; others merged with the panic in place).
2. STORY-INDEX as a manual log instead of a status oracle.
3. No graduation contract on BC/VP status — promotion criteria undefined, so 222/222 stayed draft.
4. Production wiring debt — no binary loaded the spec/config/registry entry points the architecture documented.
5. Doc-comment freeze after stub — many crates carried `/// STUB — todo!() pending` doc-comments long after the code was real, never cleaned post-implementation.

**The convergence ceremony missed all of it.** Adversarial review with 3-CLEAN passes converged on the perimeter it was shown — internal consistency of a story's BCs and ACs. It did not check the perimeter. Cross-cutting questions like "does any binary actually call this?" or "is the STORY-INDEX status field consistent with the story-file frontmatter?" were never asked.

---

## Plan 1 — Three-Layer Prevention Model

Each layer catches a different latency class. Single-layer defenses leak. Defense-in-depth is mandatory.

### Layer 1: Hooks (mechanical, runs at PR/commit time, ms latency)

| Hook | What it does | Catches |
|---|---|---|
| **Stub-residue gate** | `rg 'todo!\(\|unimplemented!\(' crates/*/src/ \| grep -v test` on PR-changed file set; non-zero exit blocks the PR | Pure stub-merge: production code with literal panic-on-call |
| **Story-frontmatter ↔ STORY-INDEX consistency** | Diff `status:` in each `.factory/stories/*.md` frontmatter against the STORY-INDEX row; fail if they disagree | Two-systems-of-truth drift |
| **Inverted-polarity test linter** | Ban `#[should_panic(expected = ".*not yet implemented.*\|.*TODO.*\|.*stub.*")]` outside an explicit RED-gate phase window | Tests that require prod-broken to stay green |
| **Doc-comment freeze detector** | Flag `/// STUB` or `/// todo!()` doc-comments in files where no `todo!()` body actually exists | Stale stub-architect doc-residue |

**Where these live:** the project's `lefthook.yml` (or git pre-push hook on non-lefthook projects), invoking helper binaries shipped by the vsdd-factory plugin.

**Why hooks first:** they catch syntactic drift instantly. Free CPU, no agent dispatch. Closes ~60% of the audit's findings at commit time.

### Layer 2: Iron Rules (agent-prompt invariants, minute latency)

These are baked into agent prompts and enforced during dispatch. The hook can't see semantics — agents must.

| Rule | Lives in | Catches |
|---|---|---|
| **Implementer's Graduation Contract** | implementer agent prompt | "Story complete" requires a callgraph proof: every public entry point in the story's BC array must reach NO `todo!()` panic. Failure → story stays `partial-merge`, not `merged` |
| **Adversary's mandatory production-stub-residue check** | adversary agent prompt | Every adversarial pass MUST grep production paths for stubs and emit a P0 for each. Cannot be skipped on "convergence" — silence is not consent |
| **Test-writer's silent-shallow scan** | test-writer agent prompt | When generating tests, must verify the test actually invokes a production module from the BC's anchor crate, not a mock that hard-codes the expected return |
| **State-manager's BC/VP promotion gate** | state-manager agent prompt + lint hook | When story merges, BCs in its `behavioral_contracts:` array auto-promote `draft → active`. If the story is `partial-merge`, no promotion. Eliminates the 222/222-draft decay |
| **Status enum closed-set** | adversary + lint hook | Story status is restricted to a fixed set (e.g. `{draft, ready, in-progress, partial-merge, merged, retired}`). Free-form values like `delivered` cause silent skip in downstream checks |

**Where these live:** agent prompt files in `vsdd-factory/agents/*.md` and skill prompts in `vsdd-factory/skills/*/SKILL.md`. The status enum is enforced via a shared schema referenced from the consistency-validator agent.

**Why iron rules:** hooks see syntax. Iron rules see meaning. Together they close another ~25% of findings.

### Layer 3: Red Flags (workspace-wide, day-to-week latency)

These cannot be PR-gated because they're topological invariants — only visible at the workspace scope. They run on a schedule (cron) or at wave gates.

| Audit | Trigger | Catches |
|---|---|---|
| **No-binary-loads-this audit** | scheduled `/vsdd-factory:audit-runtime-wiring` skill | Every accepted ADR with crate-level deliverables must be reachable from at least one `bin/` target. If `prism-mcp::Server::new` is never called from any binary, fail the audit |
| **Orphan-config detector** | scheduled `/vsdd-factory:audit-orphan-configs` | Every TOML/YAML/JSON config outside `Cargo.toml`/test fixtures must have at least one production loader. Reverse-grep |
| **VP graduation watch** | scheduled `/vsdd-factory:audit-vp-promotion` | Flag any VP `draft` >30 days where its anchor story has merged |
| **Periodic stub-debt sweep** | scheduled `/vsdd-factory:audit-stub-debt` | Wave-gate or weekly cron: workspace-wide audit identical in shape to what produced the 53-finding report. If new findings appear, the wave does not gate |

**Where these live:** scheduled skills in `vsdd-factory/skills/`, optionally registered via a wave-gate hook in the orchestrator agent prompt.

**Why red flags:** the previous two layers are local. The runtime-wiring failure and the orphan-TOML failure require a topology view. Without Layer 3, the slowest signal becomes the canary, and you find out months later when the audit you just ran shows 53 findings.

### Why all three layers stacked

Each layer's miss space is the next layer's catch space:

- **Hooks miss** silent-shallow tests (the test compiles and imports production code, but actually exercises a mock). → Iron rules catch.
- **Iron rules miss** "no binary calls this" because no individual story has visibility outside its own crate. → Red flags catch.
- **Red flags miss** in-PR drift because they run on a cadence, not a commit. → Hooks catch.

The Prism failure was a **3-of-3 layer absence**. Adversarial review (a partial Layer 2) was the only defense, and it correctly converged on what it was shown.

### Concrete deliverables for the vsdd-factory plugin

Land these in a single Bundle-A-equivalent cycle (~1 plugin week):

**Hook scripts** (`vsdd-factory/hooks/`):
- `pre-push-stub-residue.sh`
- `pre-burst-story-index-consistency.sh`
- `pre-push-inverted-polarity.sh`
- `pre-push-doc-freeze.sh`

**Policy registry additions** (the plugin's baseline `policies.yaml`):
- POL-12: `production_stub_residue_blocks_merge` (HIGH)
- POL-13: `story_frontmatter_index_consistency` (HIGH)
- POL-14: `bc_vp_promotion_on_anchor_merge` (HIGH)
- POL-15: `runtime_wiring_required_for_accepted_adrs` (MEDIUM)
- POL-16: `no_inverted_polarity_tests_outside_red_gate` (HIGH)

**Agent prompt patches:**
- implementer.md: add the Graduation Contract section
- adversary.md: add the production-stub-residue check as a mandatory dimension
- test-writer.md: add the silent-shallow scan as a self-check
- state-manager.md: add BC/VP promotion-on-merge

**Schema additions:**
- Story status enum: add `partial-merge` to the closed set (companion proposal `vsdd-stub-merge-policy-2026-05-08.md` covers this in detail; reference, don't duplicate)
- BC/VP promotion criteria: define `draft → active → verified` transitions

**New scheduled skills** (`vsdd-factory/skills/`):
- `audit-stub-debt/` — workspace-wide stub-residue sweep
- `audit-runtime-wiring/` — ADR ↔ binary reachability
- `audit-orphan-configs/` — TOML/YAML/JSON loader-coverage
- `audit-vp-promotion/` — VP draft-decay detector

**Estimated effort:** 5-7 plugin-engineer-days for the Rust-anchored implementation.

---

## Plan 2 — Language-Agnostic Implementation

### Goal

Plan 1 as written assumes Rust patterns: `todo!()`, `unimplemented!()`, `#[should_panic]`, `crates/*/src/`, `tests/`. A naive port to Python or TypeScript would mean forking the plugin per language. **That doesn't scale.**

The goal of Plan 2: **one vsdd-factory plugin, language-agnostic substrate, language-specific config.** Adding Ruby = adding a YAML row, no new plugin code.

### Observation: stub-residue is a small, finite pattern set per language

Every common language has 1-3 idiomatic ways to express "this function is unfinished":

| Language | Stub idioms | Inverted-polarity test idioms |
|---|---|---|
| Rust | `todo!()`, `unimplemented!()`, `panic!("TODO...")` | `#[should_panic(expected = "not yet implemented")]` |
| Python | `raise NotImplementedError`, `pass` (in declared-behavior fns), `...` ellipsis bodies | `pytest.raises(NotImplementedError)` |
| TypeScript / JavaScript | `throw new Error("not implemented")`, function bodies with only `// TODO` | `expect(...).toThrow("not implemented")` |
| Go | `panic("not implemented")`, `panic("TODO")` | `assert.PanicsWithValue(t, ..., "not implemented")` |
| Java | `throw new UnsupportedOperationException()`, `throw new RuntimeException("TODO")` | `assertThrows(UnsupportedOperationException.class, ...)` |
| Kotlin | `TODO()` (stdlib function), `throw NotImplementedError()` | `assertFailsWith<NotImplementedError>` |
| C# | `throw new NotImplementedException()` | `Assert.Throws<NotImplementedException>` |
| Swift | `fatalError("not implemented")`, `preconditionFailure(...)` | `XCTAssertThrowsError(... "not implemented")` |
| Ruby | `raise NotImplementedError`, `raise "TODO"` | `expect { ... }.to raise_error(NotImplementedError)` |
| C / C++ | `assert(0 && "not implemented")`, `abort()` with stub comment | (test framework specific) |

**This table IS the plugin.** Capture it as a YAML config; ship one hook script that consumes the YAML.

### Tier 1: Pattern-config approach (ripgrep + YAML)

A single shared YAML config, loaded by a single hook script:

```yaml
# vsdd-factory/templates/stub-detection.yaml

languages:
  rust:
    detect_markers: [Cargo.toml, rust-toolchain.toml]
    src_globs:
      - "**/*.rs"
    src_excludes:
      - "**/tests/**"
      - "**/target/**"
      - "**/proofs/**"
      - "**/fuzz/**"
    test_globs:
      - "**/tests/**/*.rs"
      - "**/*_test.rs"
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
    detect_markers: [pyproject.toml, setup.py, setup.cfg, requirements.txt]
    src_globs:
      - "**/*.py"
    src_excludes:
      - "**/tests/**"
      - "**/test_*.py"
      - "**/*_test.py"
      - "**/conftest.py"
      - "**/.venv/**"
      - "**/venv/**"
    test_globs:
      - "**/tests/**/*.py"
      - "**/test_*.py"
      - "**/*_test.py"
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
    src_globs:
      - "**/*.{ts,tsx}"
    src_excludes:
      - "**/*.test.{ts,tsx}"
      - "**/*.spec.{ts,tsx}"
      - "**/__tests__/**"
      - "**/node_modules/**"
      - "**/dist/**"
    test_globs:
      - "**/*.test.{ts,tsx}"
      - "**/*.spec.{ts,tsx}"
      - "**/__tests__/**/*.{ts,tsx}"
    stub_patterns:
      - 'throw\s+new\s+Error\(\s*[\"''][^\"'']*(?:TODO|todo|not implemented|stub)'
      - 'throw\s+new\s+Error\(\s*[\"'']Not implemented'
    inverted_polarity:
      - 'expect\([^)]*\)\.toThrow\([^)]*[\"''][^\"'']*not implemented'

  go:
    detect_markers: [go.mod]
    src_globs:
      - "**/*.go"
    src_excludes:
      - "**/*_test.go"
      - "**/vendor/**"
    test_globs:
      - "**/*_test.go"
    stub_patterns:
      - 'panic\(\s*"[^"]*(?:TODO|not implemented|stub|unimplemented)'
    inverted_polarity:
      - 'assert\.PanicsWithValue\([^,]+,\s*[^,]+,\s*[\"''][^\"'']*not implemented'

  java:
    detect_markers: [pom.xml, build.gradle, build.gradle.kts]
    src_globs:
      - "**/src/main/**/*.{java,kt,kts}"
    src_excludes: []
    test_globs:
      - "**/src/test/**"
    stub_patterns:
      - 'throw\s+new\s+UnsupportedOperationException'
      - 'throw\s+new\s+RuntimeException\(\s*"[^"]*(?:TODO|not implemented)'
      - 'TODO\(\s*"[^"]*"\s*\)'   # Kotlin stdlib
      - 'throw\s+NotImplementedError'   # Kotlin
    inverted_polarity:
      - 'assertThrows\(UnsupportedOperationException'
      - 'assertFailsWith<NotImplementedError>'

  csharp:
    detect_markers: ['*.csproj', '*.sln']
    src_globs:
      - "**/*.cs"
    src_excludes:
      - "**/*.Tests/**"
      - "**/*.Test/**"
      - "**/bin/**"
      - "**/obj/**"
    test_globs:
      - "**/*.Tests/**/*.cs"
    stub_patterns:
      - 'throw\s+new\s+NotImplementedException'
    inverted_polarity:
      - 'Assert\.Throws<NotImplementedException>'

  ruby:
    detect_markers: [Gemfile, '*.gemspec']
    src_globs:
      - "**/lib/**/*.rb"
      - "**/app/**/*.rb"
    src_excludes:
      - "**/spec/**"
      - "**/test/**"
    test_globs:
      - "**/spec/**/*.rb"
      - "**/test/**/*.rb"
    stub_patterns:
      - 'raise\s+NotImplementedError'
      - 'raise\s+[\"''][^\"'']*(?:TODO|not implemented|stub)'
    inverted_polarity:
      - 'to\s+raise_error\(NotImplementedError'
```

**The hook script** (single bash file, ~150-200 LoC):

```bash
#!/usr/bin/env bash
# vsdd-factory/bin/stub-residue-check
#
# Modes:
#   --scan-prod          Stub residue in production paths
#   --scan-tests         Inverted-polarity tests
#   --scan-doc-freeze    Stale stub doc-comments
#   --scan-orphan-configs (Tier 1.5; uses a separate config-loader-pattern YAML)
#
# Output: TAP-like findings, one per line:
#   STUB <file>:<line> <pattern> "<matched text>"
#
# Exit:  0 = clean, 2 = findings present
```

The hook detects the project's language(s) by walking marker files (multi-language monorepos are valid — apply each language's rules to its respective subtree). All language rules live in YAML; no per-language code.

**Adding a new language:** PR a YAML row, optionally with a small test fixture in `vsdd-factory/test-fixtures/<lang>/`. Zero plugin code change.

### Tier 2: WASM tree-sitter substrate (semantic checks)

Ripgrep patterns catch syntactic stubs (~90% of cases). The remaining ~10% are *structural* — silent-shallow tests, doc/code coherence drift, BC-postcondition-vs-implementation drift — and need an AST.

The full Tier 2 architecture is **Plan 3** below. In short: tree-sitter is the right substrate, shipped as **WASM grammars from day 1** alongside custom-check WASM extensions. The plugin commits to WASM as the universal compiled-extension mechanism — same loader, same trust model, same distribution path serves grammars and user-authored checks. See Plan 3 for the full architecture.

### Per-language surface that legitimately remains

Some checks are inherently per-language because they depend on the language's build system or runtime model:

| Check | Why per-language | Approach |
|---|---|---|
| **Binary entry-point reachability** ("does any binary load this?") | Each language's build system enumerates entry points differently | A 30-line shim per language: Rust = `cargo metadata --format-version 1`; Node = `package.json` `bin:`/`main:`; Python = `[project.scripts]`/`entry_points`; Go = `go list -json ./...`; Java = jar manifest `Main-Class`. Wrap each behind a uniform `find_entry_points()` interface |
| **Test-vs-production boundary** | Convention varies (Rust `tests/` + `#[cfg(test)]`; Python `test_*.py`; Go `_test.go`; Java `src/test/`) | Captured in the YAML's `src_globs`/`test_globs`/`in_file_test_attr` fields. Already language-agnostic via config |
| **Build-system invocation** ("does the project build?") | Each language has its own toolchain | Out of scope for stub-detection. Existing CI handles this |

**Verdict:** the language-specific surface is small (entry-point detection ≈ 30 LoC per language, in a uniform shim) and isolated (test boundary is YAML config). Everything else is universal.

### Recommended Tier 1 build order

1. **Schema first.** Lock the YAML schema for `stub-detection.yaml`. Document each field. Ship a JSON Schema for editor autocomplete.
2. **Rust + Python rows first.** Two languages with very different idioms. If the schema serves both cleanly, it'll serve TypeScript and Go.
3. **Test fixtures per language.** A `vsdd-factory/test-fixtures/<lang>/{passing,failing}/` set so the hook can be regression-tested.
4. **Hook script.** Single bash file consuming the YAML.
5. **CI smoke test for the hook itself.** The plugin's own CI runs the hook against the fixture set on every plugin PR.
6. **Document onboarding for new languages.** A `CONTRIBUTING.md` section: "How to add language X — 4 steps, no code changes."

**Effort:** 3-5 days for Rust+Python+TS coverage; 1-2 days per additional language thereafter.

---

## Plan 3 — WASM Extension Substrate (Day-1 Commitment)

Plan 1 ships the three-layer prevention model in language-anchored form. Plan 2 Tier 1 ships the YAML+ripgrep substrate for the syntactic class of checks across languages. **Plan 3 is the plugin's compiled-extension architecture** — the substrate for everything that needs more than data-driven regex: tree-sitter grammars, semantic checks, project-specific custom checks, future cross-cutting analyses.

WASM is the substrate **from day 1**. No native-then-migrate. No two-mechanism split where grammars live as Cargo deps and custom checks ship as Lua scripts and language shims live as bash. One ABI, one loader, one trust model, one distribution path.

### Why WASM-first, not WASM-eventually

| Property | WASM-first | Native-then-migrate |
|---|---|---|
| **Adding a language at runtime** | Drop a `.wasm` file | Rebuild plugin binary |
| **Trust model** | Sandboxed by default | Process-level trust |
| **Cross-platform** | One artifact, all OS/arch | Build matrix per platform |
| **Auditability** | Hashable, signable artifact | Per-platform binary signing |
| **Extension authorship** | Same toolchain for all extension types | Different paths for grammars vs scripts vs shims |
| **Operational cost of migration** | None — committed up front | Refactor cost when languages 3+ arrive |

The native-then-migrate path looks cheaper in week 1 and gets more expensive every week thereafter. Native grammars become Cargo deps that the plugin binary owns; migrating to WASM later means rewriting the loader, retesting all language paths, and repinning grammar versions. Pay the cost upfront, once.

The strongest argument: **WASM is also how custom checks ship.** A user-authored check (project-specific pattern, organization-wide policy) is a WASM module. If grammars are native and checks are WASM, the plugin has two extension mechanisms forever. If both are WASM, the plugin has one extension mechanism — for grammars, for built-in checks, for user-authored checks, for future analysis types not yet imagined.

### The two extension classes

**Class 1: Grammar extensions** (consumed by the tree-sitter runtime)
- One `.wasm` per language.
- Sourced from upstream tree-sitter grammar releases or built locally via `tree-sitter build --wasm`.
- Loaded via tree-sitter's `WasmStore` (stable in the `tree-sitter` Rust crate since v0.22, 2024; production adopters include Zed editor and several language servers).
- Per-language query files (`.scm`) live alongside as plain text — they're queries, not code, no need for WASM.

**Class 2: Check extensions** (consumed by the plugin's check dispatcher)
- WASM modules implementing the `vsdd-check` ABI.
- Built-in checks ship in `extensions/checks/builtin/` as part of the plugin distribution.
- User-authored checks live in `<project>/.factory/extensions/checks/` and are loaded at scan time.
- Authored in any language that compiles to WASM — Rust → wasm32 is the primary path; AssemblyScript and TinyGo work for non-Rust authors.

### Plugin core architecture

```
vsdd-factory/
├── crates/
│   └── vsdd-check/                  # the binary
│       └── src/
│           ├── runtime.rs           # wasmtime + tree-sitter WasmStore wiring
│           ├── manifest.rs          # extension manifest schema
│           ├── dispatcher.rs        # check dispatch loop
│           └── findings.rs          # finding emit + serialization
├── extensions/
│   ├── grammars/                    # tree-sitter WASM grammars
│   │   ├── rust.wasm
│   │   ├── python.wasm
│   │   └── typescript.wasm
│   ├── checks/
│   │   └── builtin/                 # ships with the plugin
│   │       ├── silent-shallow.wasm
│   │       ├── doc-code-coherence.wasm
│   │       └── stub-residue-deep.wasm
│   └── manifests/
│       ├── rust.toml
│       ├── python.toml
│       └── ...
├── queries/                         # tree-sitter S-expression queries
│   ├── rust/
│   │   ├── silent-shallow.scm
│   │   ├── doc-code-coherence.scm
│   │   └── stub-residue-deep.scm
│   ├── python/
│   │   └── ...
│   └── typescript/
│       └── ...
└── test-fixtures/
    ├── rust/{passing,failing}/
    ├── python/{passing,failing}/
    └── typescript/{passing,failing}/
```

The plugin core is a thin Rust binary (~1500-2500 LoC) that:
1. Discovers grammars in `extensions/grammars/`.
2. Discovers built-in checks in `extensions/checks/builtin/`.
3. Discovers project-local custom checks in `<project>/.factory/extensions/checks/`.
4. Detects file language (extension + marker file).
5. For each file, dispatches matching checks against the parsed AST.
6. Emits findings to stdout/JSON/SARIF.

### Extension manifest format

Each extension declares its metadata, ABI version, and capabilities:

```toml
# extensions/manifests/silent-shallow.toml
name = "silent-shallow"
version = "1.0.0"
abi_version = 1                    # plugin core supports range [1, N]
type = "check"                     # "check" | "grammar" | "shim"
artifact = "extensions/checks/builtin/silent-shallow.wasm"

applies_to_languages = ["rust", "python", "typescript"]
requires_grammar = true            # this check needs an AST

[capabilities]
read_target_file = true            # default
read_arbitrary_files = false       # sandboxed by default
network = false                    # no
```

For grammars:

```toml
# extensions/manifests/rust-grammar.toml
name = "rust"
version = "0.21.2"                 # mirrors upstream tree-sitter-rust
abi_version = 1
type = "grammar"
artifact = "extensions/grammars/rust.wasm"
upstream_source = "https://github.com/tree-sitter/tree-sitter-rust"
upstream_commit = "<sha>"

file_extensions = [".rs"]
marker_files = ["Cargo.toml", "rust-toolchain.toml"]
```

### The `vsdd-check` ABI (sketch)

A check is a WASM module exporting two functions (raw exports for v1; WIT migration documented as v2):

```
;; metadata: returns JSON describing what this check looks for
(func $check_metadata (result i32 i32))   ; pointer + length to JSON string

;; check: runs against a parsed file
(func $check
    (param i32 i32)                       ; ptr+len to source bytes
    (param i32)                           ; tree-sitter tree handle
    (param i32 i32)                       ; ptr+len to file path
    (result i32 i32))                     ; ptr+len to findings JSON array
```

Findings are serialized JSON:
```json
[
  {
    "severity": "P0",
    "rule": "silent-shallow",
    "file": "crates/foo/tests/integration.rs",
    "line": 142,
    "column": 5,
    "message": "Test asserts on mock return value but never invokes production module 'foo::engine'",
    "evidence": "let result = mock_engine.execute(); assert_eq!(result, 42)"
  }
]
```

**Future-proofing:** WIT (WebAssembly Interface Types) and the Component Model would be more ergonomic than raw exports, but the tooling is still maturing as of 2026. Recommend starting with raw exports, then migrating to WIT once the Component Model stabilizes — the migration is mechanical (interface generators handle the marshaling).

### Capability model

Extensions are sandboxed by default. The plugin core grants capabilities per-extension based on the manifest:

| Capability | Default | Used for |
|---|---|---|
| `read_target_file` | granted | The file being scanned |
| `read_arbitrary_files` | denied | Cross-file analysis (must be explicit) |
| `read_factory_artifacts` | denied | BC/VP index lookup |
| `network` | denied | Currently no use case; keep denied |
| `wall_clock_time` | granted | Logging, timing |

This matters because user-authored extensions are untrusted code. A check that reads `~/.aws/credentials` and exfiltrates findings is a real attack vector. Capability-by-default-deny is the right starting posture.

### Build and CI flow

**Grammar build:**
```bash
# In a per-grammar build script
cd third-party/tree-sitter-rust
tree-sitter build --wasm
cp tree-sitter-rust.wasm $PLUGIN_ROOT/extensions/grammars/rust.wasm
```

The tree-sitter CLI compiles the grammar's C source to WASM via emscripten or docker. Plugin CI runs this for each grammar on each release.

**Check build (Rust → wasm32):**
```bash
# In a check crate
cargo build --target wasm32-unknown-unknown --release
cp target/wasm32-unknown-unknown/release/silent_shallow.wasm \
   $PLUGIN_ROOT/extensions/checks/builtin/silent-shallow.wasm
```

**Plugin CI:**
1. Build all built-in WASM artifacts.
2. Validate manifest schema against JSON Schema.
3. Run plugin against `test-fixtures/<lang>/{passing,failing}/` — passing dirs must produce zero findings; failing dirs must produce expected findings.
4. ABI compatibility check: each extension's `abi_version` must be within plugin core's supported range.
5. Hash-pin all artifacts in an `extensions.lock.toml` for reproducibility.

### Distribution and versioning

The plugin core declares a supported ABI range:

```toml
# vsdd-factory's plugin metadata
abi_versions_supported = { min = 1, max = 1 }
```

Each extension declares its required ABI (`abi_version = 1`). When ABI version 2 lands, plugin core supports `{min=1, max=2}` for at least one minor release cycle, giving extension authors time to migrate. After that, plugin core's `min` bumps to 2.

User-authored extensions are versioned independently. A project pins its extensions in `<project>/.factory/extensions.lock.toml`:

```toml
[grammars.rust]
version = "0.21.2"
sha256 = "..."

[checks.org_specific_db_layer]
version = "1.0.3"
sha256 = "..."
source = "https://github.com/<org>/vsdd-checks/releases/download/v1.0.3/db-layer.wasm"
```

### Honest costs of WASM-first

1. **WASM tree-sitter parsing is ~2-5x slower than native.** For PR-time hooks running on changed files only (10-100 files), this is millisecond-scale either way. For workspace audits (thousands of files), it adds tens of seconds. Acceptable.
2. **wasmtime adds ~10MB to the plugin binary.** Modern dependency, not painful but real. `wasmer` is an alternative if size is critical.
3. **WASM toolchain learning curve for check authors.** Rust → wasm32 is straightforward (`cargo build --target wasm32-unknown-unknown`). AssemblyScript is approachable for non-Rust authors. TinyGo works for Go authors. Less polished than native compilation, but production-grade.
4. **Debugging WASM is rougher than native.** Stack traces work; gdb-style debugging is limited. Extension authors are expected to test extensively in fixtures before shipping.
5. **Tree-sitter Rust WasmStore is newer than the native path.** Stable since `tree-sitter` v0.22 (2024). Production adopters include Zed editor and several language servers. Pin a conservative `tree-sitter` version and watch for ecosystem stabilization.
6. **Grammar provenance discipline.** WASM grammars must be compiled from trusted sources. Plugin CI builds grammars from upstream commits pinned in manifests, not from prebuilt binaries scraped off random sites.

### What WASM does NOT replace

Plan 2 Tier 1 (YAML + ripgrep) stays as-is. WASM is for compiled, structural, non-trivial extensions. Cheap data-driven checks remain YAML — there's no value in WASM-ifying a regex pattern set. The plugin runs both substrates: ripgrep for fast syntactic checks, WASM for structural checks. They emit findings in the same format and merge in the dispatcher.

### Phase 3.A–D sequencing (the WASM substrate build)

**Phase 3.A — substrate (1 plugin week):**
- `vsdd-check` core binary skeleton: wasmtime + tree-sitter WasmStore.
- Extension manifest schema + JSON Schema.
- Capability model implementation (default-deny).
- One grammar end-to-end: Rust WASM grammar built from upstream + loaded + parsed.
- One check end-to-end: `silent-shallow` for Rust as a built-in WASM check.
- CLI: `vsdd-check --check silent-shallow <file>`.
- Test fixtures for Rust passing and failing cases.

**Phase 3.B — multi-language proof (~0.5 plugin week):**
- Add Python WASM grammar.
- Author `queries/python/silent-shallow.scm`.
- Add Python fixtures.
- Verify language detection picks the right grammar.
- Verify the same check WASM module dispatches against multiple grammars (the check accesses the AST via tree-sitter's API regardless of which grammar parsed it).

**Phase 3.C — custom-check ABI + reference (~1 plugin week):**
- Finalize the `vsdd-check` ABI (raw exports for v1).
- Reference custom check: a project-specific pattern, e.g., "no `unwrap()` in production code outside test modules" — built as a separate Rust crate compiled to wasm32.
- Document the ABI in `vsdd-factory/docs/check-authoring.md`.
- Plugin CI: validate the reference check against fixtures.

**Phase 3.D — authoring docs + secondary toolchain (~0.5 plugin week):**
- AssemblyScript example custom check.
- "Adding a language" doc: end-to-end recipe (find grammar → build WASM → write queries → add manifest → ship fixtures).
- "Authoring a custom check" doc: ABI walkthrough, fixture pattern, capability declaration.

**Total Phase 3 effort: ~3 plugin weeks.**

### What ships at the end of Phase 3

- A plugin with the full prevention layer (Plan 1) + cheap multi-language syntactic coverage (Plan 2 Tier 1) + structural multi-language coverage via WASM (Plan 3).
- Extensible: new languages added by dropping a WASM grammar + query files.
- Extensible: new checks added by writing a WASM module + manifest.
- Sandboxed: untrusted user-authored checks can ship safely.
- Reproducible: lockfile-pinned grammars and checks.

### What this enables that nothing else does

Once the WASM substrate is in place, the plugin becomes a host for organizational policy enforcement that goes well beyond stub-merge defense:

- An organization can ship internal checks ("no direct DB queries outside the repo layer", "all log statements include a request ID") as WASM modules without forking the plugin.
- Project-specific lints ("our domain model forbids using floating-point for currency") can ship as repo-local extensions.
- Cross-cutting policies ("every public API must have a docstring referencing a BC") become one custom check per organization.

These are out of scope for the prevention proposal — but they're the natural growth path the WASM substrate enables. **The plugin stops being a stub-merge defense tool and becomes a programmable policy engine.** That's the deeper reason WASM-first matters from day 1.

---

## Recommended Build Plan

### Phase 1 (immediate): Plan 1, language-anchored, Rust-only

Reason: prove the three-layer model works end-to-end on one language. Land in the source project (Prism) as the cleanup-Bundle-A deliverable; land in the vsdd-factory plugin as v-next features.

**Deliverables:**
- 4 hook scripts (Rust-flavored regex)
- 5 policies (POL-12 through POL-16)
- 4 agent-prompt patches (implementer, adversary, test-writer, state-manager)
- 4 scheduled-audit skills
- `partial-merge` story status enum

**Effort:** ~1 plugin week.

**Validation:** re-run the workspace audit on the source project after Phase 1 lands; new findings count should drop to <5.

### Phase 2 (next cycle): Plan 2 Tier 1, YAML-driven refactor

Reason: now that the layered model is validated, refactor the Rust-flavored hook script into a YAML-config substrate. Ship Rust + Python + TypeScript + Go rows.

**Deliverables:**
- `stub-detection.yaml` schema + JSON Schema
- Generic hook script (`bin/stub-residue-check`)
- 4 language YAML rows
- Test fixtures for 4 languages
- `CONTRIBUTING.md` "add a language" section

**Effort:** ~1 plugin week.

**Validation:** apply the plugin to a Python project and a TypeScript project; both should pass the hook on green code and fail on injected stubs.

### Phase 3 (parallel with Phase 2 once Phase 1 lands): Plan 3, WASM extension substrate

Reason: deeper checks where regex cannot reach (silent-shallow tests, doc/code coherence, BC-postcondition-vs-implementation drift), AND establish the plugin's compiled-extension architecture. WASM is the substrate from day 1 — no native-then-migrate dance, no two-mechanism split between grammars and custom checks. See Plan 3 above for the full architecture.

**Deliverables:**
- `vsdd-check` Rust binary embedding wasmtime + tree-sitter WasmStore
- WASM tree-sitter grammars: Rust (Phase 3.A) + Python (Phase 3.B)
- `vsdd-check` ABI specification + JSON Schema for extension manifests
- Capability-based sandbox model (default-deny)
- Reference custom check (Rust → wasm32) demonstrating the authoring path
- Per-language query files (`queries/<lang>/<check>.scm`)
- New skills: `audit-silent-shallow-tests`, `audit-doc-code-coherence`
- Authoring documentation (Rust → wasm32 primary, AssemblyScript secondary)

**Effort:** ~3 plugin weeks (3.A 1wk + 3.B 0.5wk + 3.C 1wk + 3.D 0.5wk).

Phase 3 can begin **in parallel with Phase 2** once Phase 1 has validated the layered prevention model. Phase 2 (YAML+ripgrep) and Phase 3 (WASM tree-sitter) emit findings into the same format and dispatcher — they're complementary substrates, not competing ones.

---

## Appendix A: Pattern Library Status

The Plan 2 Tier 1 YAML is a starting point. Production-grade pattern lists per language need:

- Vetting against language-specific style guides (e.g. Python's `...` ellipsis is sometimes legitimate in protocol/abstract declarations — must scope by context).
- Idiom drift: Rust nightly may add new panic macros; Python's PEP-695 introduces new syntax. The YAML should be versioned (`schema_version: 2`) and CI-gated against drift in upstream language specs.
- False-positive corpus: each language's row should ship with example "this looks like a stub but isn't" cases.

The 9-language draft above is a 60-second starter; harden each language with a domain expert pass before shipping.

## Appendix B: Findings Reference

The source-project workspace audit that motivated this proposal:

- 53 findings: P0=18, P1=23, P2=12.
- 8 audit dimensions: production stub residue, story-vs-impl drift, silent-shallow tests, TOML/config orphans, BC postcondition gaps, ADR implementation status, VP proof status, documentation drift.
- Coverage by layer (estimated):
  - Layer 1 hooks would have caught: ~32 findings (production stub residue + STORY-INDEX consistency + inverted-polarity + doc-freeze).
  - Layer 2 iron rules would have caught: ~14 findings (silent-shallow + BC promotion gaps + status enum drift + adversary mandatory checks).
  - Layer 3 red flags would have caught: ~7 findings (no-binary-loads-this + orphan TOMLs + VP graduation + workspace-wide drift).
- Layer overlap is intentional. ~6 findings are caught by 2+ layers; this is desired redundancy.

Full audit report: see `companion_proposal: vsdd-stub-merge-policy-2026-05-08.md` and the source project's `cycles/wave-4-operations/workspace-audit-2026-05-08.md`.

## Appendix C: Companion proposals

- `vsdd-stub-merge-policy-2026-05-08.md` — the schema fix: `partial-merge` status enum, graduation contract, adversary policy, `audit-stub-debt` skill. **Read this first.**
- `vsdd-prevention-layers-2026-05-08.md` — this document: the three-layer enforcement infrastructure, language-anchored (Plan 1), syntactic-multi-language via YAML+ripgrep (Plan 2), and structural-multi-language via WASM extension substrate (Plan 3).

The two proposals are complementary, not redundant. The schema fix without the enforcement infrastructure produces unenforced enums; the enforcement infrastructure without the schema fix has nowhere to write the `partial-merge` state.

---

## Open Questions for the vsdd-factory Engineer

1. **Should Layer 3 audits run in CI or only on-demand?** Cron is a third option. Trade-off: CI burns minutes on every PR; on-demand requires a release-time discipline; cron is independent of PR velocity but can ship findings to a stale audience.
2. **Should the YAML schema be project-overridable?** A project might want to permit `panic!("internal invariant: ...")` even though the regex matches. Suggest a per-project `.factory/stub-detection-overrides.yaml` with an allowlist.
3. **How does the plugin handle multi-language monorepos?** Sketch above: detect each language by marker, apply each language's rules to its subtree. But path-disambiguation across overlapping subtrees needs a precedence rule.
4. **Does Tier 2 tree-sitter justify the maintenance burden?** Tree-sitter grammars need version-pinning per language; some languages (Rust, in particular) have grammar drift on edition bumps. Open question whether the semantic-check value justifies the maintenance.
5. **Should `partial-merge` be opt-in per project?** Some projects may not want the new enum value. Suggest the plugin ships it default-on but configurable.
6. **Capability-based security for WASM extensions — what file/network access do extensions get?** Sketched in Plan 3 as default-deny (only `read_target_file` granted by default). Open: should `read_factory_artifacts` (BC/VP index lookup) be a common opt-in capability or remain rare? Cross-file analysis checks need it.
7. **WIT (WebAssembly Interface Types) for the check ABI vs raw exports?** WIT future-proofs and adds ergonomics but the Component Model tooling is still maturing. Plan 3 starts with raw exports (v1 ABI) and documents WIT migration as v2. Open: when does the Component Model stabilize enough to make WIT the default?
8. **Where do WASM grammars come from?** Three options: (a) mirror upstream tree-sitter grammar releases as binary artifacts; (b) build locally in plugin CI from grammar source repos at pinned commits; (c) both (mirror as a fallback, prefer locally-built). Local builds give provenance; mirrors give speed. Open recommendation: option (b) with hash-pinning.
9. **Tree-sitter Rust WasmStore version policy.** Stable since v0.22 (2024); production adopters include Zed editor. Pin a conservative version (e.g. v0.22.6) and bump intentionally, or follow latest? Open: define a tree-sitter version-pin policy.
10. **What capabilities should built-in checks ship with vs user-authored checks?** A built-in `silent-shallow` check arguably needs `read_arbitrary_files` to walk the project's module graph; a user-authored check probably should not get that capability by default. Open: define a "trusted built-in" tier vs a "user-authored sandbox" tier in the manifest schema.

End of proposal.
