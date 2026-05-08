---
artifact_type: proposal
target_repo: vsdd-factory
target_repo_url: TBD (Joshua's vsdd-factory plugin repo)
date: 2026-05-08
status: draft-for-export
self_contained: true
prior_context_required: false
companion_proposal: vsdd-stub-merge-policy-2026-05-08.md
prism_audit_reference: ../cycles/wave-4-operations/workspace-audit-2026-05-08.md
---

# VSDD Prevention Layers — Stub-Merge / Status-Drift / Runtime-Gap Defense

**For:** Claude Code session in the `vsdd-factory` plugin repo, with **no prior context** about the Prism project, the Wave 3 cascade, or the audits that motivated this proposal. This document is the entire input.

**You are receiving:** two plans. Plan 1 is the three-layer prevention model expressed in language-anchored (Rust) form, ready to land as a vsdd-factory v-next feature. Plan 2 is a refactor of Plan 1 into a language-agnostic substrate so a single plugin can serve Rust, Python, TypeScript, Go, Java, etc., without per-language forks.

---

## TL;DR

A workspace audit found 53 findings (18 P0) in a VSDD-managed Rust project. Stories were marked `merged` in the index while their production code paths were still `todo!()` panics. Tests were silent-shallow (asserting on mocks instead of production SUTs). At least one test was *inverted-polarity* — `#[should_panic(expected = "not yet implemented")]` — meaning green CI required production code to stay broken.

The convergence ceremony (3-CLEAN adversarial passes) **did not catch this**. The adversary correctly converged on what was inside the perimeter shown to it; nobody checked whether the perimeter was right.

This is a **defense-in-depth failure**. It needs three layers, not one.

- **Plan 1** delivers the three layers concretely for a Rust project, ~1 week of plugin work.
- **Plan 2** describes how to make Plan 1's hooks/rules/audits language-agnostic so the same plugin protects Python/TS/Go/Java projects without per-language plugins.

Recommended build order: **Plan 1 first** (proves the layered model on one language); **Plan 2 as a refactor cycle** (after the layered model is validated, generalize the substrate).

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

### Tier 2: Tree-sitter substrate (semantic checks)

Ripgrep patterns catch syntactic stubs (90% of cases). They miss:

- **Silent-shallow tests** — a test that imports production but never calls into it (calls a mock instead).
- **Doc/code coherence** — function declared `Returns: Vec<Foo>` but body is `vec![]` placeholder.
- **Specification ↔ implementation coherence** — BC says "produces side effect X" but code only logs.

These need an AST. **Tree-sitter is the right substrate**:

- One installation, ~200+ language grammars maintained upstream.
- Stable AST query language (S-expression matchers).
- Already heavily integrated into editors / GitHub semantic search.
- Queries are reusable across the same conceptual check.

**Example tree-sitter check (silent-shallow test):**

```scheme
; Find test functions that don't call the production crate's modules.
; Pseudo-query — actual queries vary per grammar.

(function_item
  name: (identifier) @test_name
  body: (block
    (call_expression
      function: (path) @callee))
  (#match? @test_name "^test_")
  (#not-match? @callee "<production_crate_name>::"))
```

**The Tier 2 plugin design:**
- Ship per-language tree-sitter queries in `vsdd-factory/queries/<lang>/`.
- A single Rust binary (`tree-sitter-vsdd-check`) loads the appropriate grammar at runtime, runs the queries, emits findings.
- Adding a new language = add grammar dependency + write per-check queries.

**Cost vs benefit:** Tier 2 is heavier — query authoring per language, grammar version-pinning, slower CI runs. But it catches the semantic class that ripgrep cannot. Recommend Tier 2 as a **cycle-after** project once Tier 1 is validated.

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

### Phase 3 (future cycle): Plan 2 Tier 2, tree-sitter semantic layer

Reason: catch silent-shallow tests and doc/code coherence drift. Heavy lift; defer until Tier 1 is in production for a quarter and the residual finding rate is measurable.

**Deliverables:**
- `tree-sitter-vsdd-check` Rust binary
- Per-language query files
- New skills: `audit-silent-shallow-tests`, `audit-doc-code-coherence`

**Effort:** ~2-3 plugin weeks.

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
- `vsdd-prevention-layers-2026-05-08.md` — this document: the three-layer enforcement infrastructure, language-anchored (Plan 1) and language-agnostic (Plan 2).

The two proposals are complementary, not redundant. The schema fix without the enforcement infrastructure produces unenforced enums; the enforcement infrastructure without the schema fix has nowhere to write the `partial-merge` state.

---

## Open Questions for the vsdd-factory Engineer

1. **Should Layer 3 audits run in CI or only on-demand?** Cron is a third option. Trade-off: CI burns minutes on every PR; on-demand requires a release-time discipline; cron is independent of PR velocity but can ship findings to a stale audience.
2. **Should the YAML schema be project-overridable?** A project might want to permit `panic!("internal invariant: ...")` even though the regex matches. Suggest a per-project `.factory/stub-detection-overrides.yaml` with an allowlist.
3. **How does the plugin handle multi-language monorepos?** Sketch above: detect each language by marker, apply each language's rules to its subtree. But path-disambiguation across overlapping subtrees needs a precedence rule.
4. **Does Tier 2 tree-sitter justify the maintenance burden?** Tree-sitter grammars need version-pinning per language; some languages (Rust, in particular) have grammar drift on edition bumps. Open question whether the semantic-check value justifies the maintenance.
5. **Should `partial-merge` be opt-in per project?** Some projects may not want the new enum value. Suggest the plugin ships it default-on but configurable.

End of proposal.
