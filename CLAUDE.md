# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

> **Toolchain:** Rust stable (per `rust-toolchain.toml`), edition 2024, resolver 2. Components: rustfmt, clippy, rust-src. Cross-compile targets: aarch64-apple-darwin, x86_64-apple-darwin, x86_64-unknown-linux-gnu, x86_64-unknown-linux-musl, x86_64-pc-windows-msvc. 24-crate workspace.

---

## Source-of-Truth Precedence

When two artifacts disagree, the **LATER, MORE-SPECIFIC artifact wins**. Apply this rule when adversary, consistency-validator, or spec-reviewer surfaces a conflict between two project documents:

1. **Story spec** (under `.factory/stories/`) supersedes the BC it traces to, when the conflict is about implementation scope. The BC supersedes when the conflict is about contract semantics.
2. **ADR** (under `.factory/specs/architecture/adr/` or numbered `ADR-NNN-*.md`) supersedes earlier ADRs that address the same decision; superseded ADRs are marked with explicit `Supersedes: ADR-NNN` and `Superseded by: ADR-MMM` frontmatter back-refs.
3. **PRD supplements** (`interface-definitions`, `error-taxonomy`, `nfr-catalog`, `test-vectors`) supersede the PRD prose for the same surface area.
4. **VP files** (`.factory/specs/verification-properties/`) supersede the prose verification narrative in PRD/architecture for the property they cover.
5. **Recent `.factory/STATE.md` decision rows (D-NNN)** supersede earlier-recorded but conflicting narrative in SESSION-HANDOFF.md.
6. **Recent adversary pass reports** supersede earlier pass reports for the same finding ID (cascade closure rationale tracks the chain).
7. **For code-vs-spec conflicts**: the SPEC wins (Standing Rule for VSDD). Code is brought into alignment via fix-burst or follow-up story, not the other way around. Only the human can authorize spec amendment to match code.

If two artifacts are at the same precedence level and disagree, surface to the orchestrator. The orchestrator routes to the artifact's owner-specialist (e.g., BC vs BC → product-owner; ADR vs ADR → architect) for adjudication.

---

## Pipeline Authority

The orchestrator (`vsdd-factory:orchestrator` agent) coordinates all phases. Specialist agents do the writing. **The orchestrator does NOT write files itself** — it delegates via the `Agent` tool with `subagent_type` set to the specialist (see Agent Routing Table in the Companion Principle section below). The single permitted exception is direct human-mandated edits to this CLAUDE.md or other project-root meta-docs (e.g., this paragraph itself).

Phase sequence for prism (brownfield mode):

- Phase 0: Codebase Ingestion (DONE 2026-04-14) — 9 reference repos analyzed
- Phase 1a/b/c/d: Spec Crystallization (DONE 2026-04-15..16) — domain spec / PRD / architecture / adversarial review
- Phase 2: Story Decomposition (DONE 2026-04-16) — 150 stories, dependency graph, wave schedule
- **Phase 3: TDD Implementation (CURRENT)** — Wave 3 multi-tenant + Wave 4 ops + Wave 0 plugin prereqs in flight
- Phase 4: Holdout Evaluation (gated on per-wave readiness)
- Phase 5: Adversarial Refinement (post-implementation cascade)
- Phase 6: Formal Hardening (Kani + cargo-fuzz + cargo-mutants + semgrep)
- Phase 7: Convergence — 7-dimensional convergence assessment

Per-story Phase 3 sub-workflow: stubs → failing tests → TDD green → LOCAL adversary 3-CLEAN → demo-recorder per-AC → push → pr-manager 9-step PR cycle → squash-merge → state-manager post-merge burst. BC-5.39.001 3-CLEAN protocol applies to every cascade.

---

## CANONICAL PRINCIPLE — Production-Grade Default

This principle binds every AI agent operating on this project. It overrides any default behavior in agent prompts, skills, or templates that conflicts with it. Mirrors the user's persistent directive recorded in `.factory/STATE.md` frontmatter (`user_directive_persistent: "No pragmatic convergence. Fix all issues before build."`) and Standing Orchestrator Rule 3 in `.factory/SESSION-HANDOFF.md`.

### Statement

**Default behavior is enterprise/production-grade correctness. Speed lives in feature *ordering*, not feature *completeness*.**

### Six rules

1. **No MVP-driven deferrals.** Phrases like "for now," "good enough," "we can fix later," "minimum viable," and "ship fast and iterate" are RATIONALIZATIONS, not engineering decisions. Treat them as defect-pattern smells. If a thing is worth doing in v1, it is worth doing correctly in v1.

2. **Feature order is the only acceptable speed lever.** It is acceptable to defer an entire feature (e.g., a future story or wave) to a later cycle. It is NOT acceptable to ship the current story partially or with shortcuts that need later cleanup. Each shipped feature must be production-grade on the cycle it ships.

3. **Tech debt register (`.factory/tech-debt-register.md`) is for HUMAN-DIRECTED deferrals ONLY.** AI agents must NOT add entries to it as a default catchment for issues found during review. If an agent discovers a defect, the default action is to FIX it in-scope. Adding to the register requires ALL of:
   - Explicit human direction to defer, AND
   - A concrete future dependency that makes the deferral necessary (e.g., "this depends on Wave 5 plugin SDK"), AND
   - Attachment to the specific future story or wave where it will be resolved (so it cannot get lost).

4. **AI-built defects are the AI's responsibility to fix.** Every artifact in `.factory/` and most code in `crates/` was written by AI (with human approval). When an AI agent finds an issue in another AI agent's output, the default is to fix it in the current scope — even if that means expanding scope. Surfacing the issue as a question, an "advisory," a "TODO for architect," or a "pending architect review" is the WRONG default. The correct default is to fix.

5. **`Suggest` is acceptable. `Default to cheap path` is not.** Agents may propose cheaper alternatives to the human, but the agent's DEFAULT action must be the correct path. "I noticed this would be faster if we skipped X — would you like to?" is fine. Skipping X without surfacing the option is not.

6. **"Pending architect review" / "TODO for architect" / "Placeholder for architect" in spec artifacts is forbidden when the question is answerable in current scope.** If the question requires architect adjudication only because the answer needs cross-component reasoning that hasn't happened yet, that's legitimate. If the question is mechanical (path migration, version pin selection, conventional clippy lint configuration), the AI handling the spec must answer it now.

### What this means in practice

| Anti-pattern | Production-grade replacement |
|--------------|------------------------------|
| "MVP: ship without test coverage on edge case X" | Write the edge case test. Cover it now. |
| "For now we'll hardcode this value; refactor later" | Read the value from config now. Write the config schema. |
| "We can add error handling in v2" | Add error handling now. Define the error taxonomy in scope. |
| "Architect TODO: confirm patch-version pinning policy" | Pick the production-grade default and write the rationale inline. |
| "Pending architect review: should we support 6 endpoints?" | Read the canonical contract, decide based on existing parity argument, document the decision. |
| "Phase 5 deferred: add this to tech-debt-register" | First ask: did the human direct this deferral? If no, fix it now. |
| "Good enough for v1" | "Production-grade for v1." If you can't say production-grade, you're not done. |
| Implementer claims "MVP scope" / "test-path-only" / "deferred to follow-up" | Adversary independently verifies the claim under fresh-context analysis (Standing Rule 3 §1). Implementer self-disclosure of risk severity is NOT authoritative. |
| Silent `Vec::new()` return where partial-failure data should propagate | Thread proper plumbing through; surface-and-defer-via-error is a SOUL.md #4 violation (Standing Rule 3 §2). |
| Doc comment claiming "this requires capability X" with no capability check | Either implement the gate or remove the docs (Standing Rule 3 §3). |
| Adding `Arc<dyn Foo>` plumbing to a constructor that didn't have it, to close a finding correctly | DO IT. "Wiring not redesign" (ADR-022 §C) means don't *replace* existing implementations; it does NOT mean don't *add* proper plumbing where it was missing (Standing Rule 3 §4). |
| File a P4 TD for cosmetic cleanup of 2 byte-identical types (~45 min total) | Fix the 2 cosmetic cleanups in-scope. P4 TDs that could have been a single inline edit are a defer-pattern smell. |

### Self-Audit Checklist (every agent, before declaring work done)

Run this checklist as the last act of every task. If any answer is "yes" or "I'm not sure," stop and remediate before declaring done.

- [ ] Did I rationalize any decision with "MVP," "for now," "good enough," or "we can fix later"?
- [ ] Did I add a new tech-debt-register entry without **all three** of: explicit human direction, concrete future dependency, and a specific future story/wave anchor?
- [ ] Did I leave any "pending architect review," "TODO for architect," or "Placeholder for architect" in a spec artifact for a question I could have answered in scope?
- [ ] Did I find a bug or gap in another AI's output and surface it as a question/advisory instead of fixing it in scope?
- [ ] Did I default to the cheapest mechanism instead of the correct mechanism?
- [ ] If I added an ADVISORY-severity finding to a report, did I evaluate whether it should be a BLOCKER under the production-grade lens? (Most "advisories" become blockers.)
- [ ] Did I paper-fix a finding by renaming, doc-commenting, or asserting-only when the real fix is structural? (TD-VSDD-059 paper-fix detection.)
- [ ] Did I sibling-sweep all callsites when I changed a function signature, constant, or canonical identifier? (TD-VSDD-060 sibling-site sweep.)

### Boundaries — what the principle does NOT mean

- **It does not mean "do everything before shipping anything."** Phasing waves (Wave 3 → Wave 4 → Wave 5) is correct. Within a wave, every shipped story must be production-grade.
- **It does not mean "no asks of the human."** Genuine human decisions — risk acceptance, business priorities, scope vs deadline tradeoffs, versioning policy — should be surfaced. The principle forbids deferring WORK that the AI can do; it does not forbid surfacing DECISIONS that only the human can make.
- **It does not mean "infinite scope expansion."** If you find an issue, fix it. If the fix requires expanding into a new domain that requires new specs or new architecture decisions, surface it cleanly and request scope expansion. The principle requires fixing, not infinite recursion.
- **It does not override security or correctness.** If a "production-grade fix" requires a security review, run the security review.

### Companion Principle — Correct Agent Routing

"Fix in scope" works ONLY when paired with correct agent routing. Otherwise it degrades into "every agent does everything," which destroys specialization and produces worse work than the defer-pattern it replaces.

#### Rules

1. **Agents own their domain.** A consistency-validator does NOT silently rewrite spec content. An implementer does NOT silently rewrite the spec. Each specialist agent has a defined scope (see Agent Routing Table below); work outside that scope is routed to the correct specialist via the orchestrator.
2. **The orchestrator owns routing.** When a specialist agent discovers a defect outside its own domain, it surfaces the finding to the orchestrator with the proposed routing. The orchestrator then dispatches the correct specialist. This is NOT a defer-pattern — it is correct-agent-pattern. The fix still happens in scope of the same work cycle.
3. **Surface vs defer — the critical distinction:**
   - **Surface (production-grade):** Agent A finds issue → routes to orchestrator → orchestrator dispatches specialist B → specialist B fixes in scope. **No human round-trip required for the routing.**
   - **Defer (forbidden):** Agent A finds issue → adds to tech-debt-register / advisory / "TODO for X" → original work declared done → issue persists. **Requires human to discover and re-prioritize.**
4. **When in doubt about routing, ask the orchestrator** — not the human. The orchestrator has the routing table loaded; let it route.
5. **The orchestrator NEVER does specialist work itself.** It coordinates, dispatches, and validates gates. If the orchestrator is tempted to write a file directly (other than this CLAUDE.md per direct human mandate), that is a routing failure — find the correct specialist and dispatch.

#### Agent Routing Table

Use this table to determine which specialist handles which kind of work. Authoritative reference; supersedes any conflicting routing in upstream skills until the upstream vsdd-factory canonicalization lands. Mirrors the routing table loaded by `.claude/agents/orchestrator.md`.

| If the work is... | Route to agent ID |
|-------------------|-------------------|
| Product brief, PRD, behavioral contracts (BCs), holdout scenarios | `vsdd-factory:product-owner` |
| Market analysis, L2 domain spec, ubiquitous language | `vsdd-factory:business-analyst` |
| Architecture, ADRs, DTU assessment, gene transfusion, dependency manifest | `vsdd-factory:architect` |
| UX spec, design system, wireframes, interaction design | `vsdd-factory:ux-designer` |
| Story decomposition, dependency graph, wave schedule | `vsdd-factory:story-writer` |
| Cross-document consistency (IDs, anchors, counts, naming) | `vsdd-factory:consistency-validator` |
| Adversarial fresh-context review (specs or implementation) | `vsdd-factory:adversary` |
| Constructive spec/story review (different-model cognitive diversity) | `vsdd-factory:spec-reviewer` |
| PR diff code review (different-model cognitive diversity) | `vsdd-factory:code-reviewer` |
| Deep codebase scanning, semantic analysis, brownfield ingest | `vsdd-factory:codebase-analyzer` |
| Brownfield extraction validation (catch hallucinated dependencies) | `vsdd-factory:validate-extraction` |
| TDD test stubs and failing tests | `vsdd-factory:test-writer` |
| TDD implementation (one failing test → minimum code → micro-commit) | `vsdd-factory:implementer` |
| E2E browser tests (Playwright/Cypress) | `vsdd-factory:e2e-tester` |
| Demo recordings (VHS terminal or Playwright browser) | `vsdd-factory:demo-recorder` |
| PR lifecycle (create, review dispatch, finding triage, merge) | `vsdd-factory:pr-manager` |
| Final fresh-eyes PR diff review before merge | `vsdd-factory:pr-reviewer` |
| Formal proofs (Kani), fuzzing, mutation testing, security scan | `vsdd-factory:formal-verifier` |
| Security review / triage (CWE/CVE, OWASP) | `vsdd-factory:security-reviewer` |
| Holdout scenario evaluation against implementation (strict info asymmetry) | `vsdd-factory:holdout-evaluator` |
| DTU clone validation against real third-party services | `vsdd-factory:dtu-validator` |
| Repo setup, worktrees, CI/CD, release, Cargo workspace init | `vsdd-factory:devops-engineer` |
| Toolchain preflight, env setup, dependency installation | `vsdd-factory:dx-engineer` |
| `.factory/STATE.md` updates, `.factory/` commits, cycle bookkeeping | `vsdd-factory:state-manager` |
| Spec governance, versioning, traceability audit | `vsdd-factory:spec-steward` |
| Documentation generation from code/specs (current behavior only) | `vsdd-factory:technical-writer` |
| External research (Perplexity, Context7, Tavily MCP access) | `vsdd-factory:research-agent` |
| GitHub CLI operations on behalf of agents without shell access | `vsdd-factory:github-ops` |
| Performance benchmarks, regression detection | `vsdd-factory:performance-engineer` |
| Data schemas, migrations, pure-core / effectful-I/O boundary | `vsdd-factory:data-engineer` |
| WCAG AA/AAA accessibility audit | `vsdd-factory:accessibility-auditor` |
| Visual regression, mockup fidelity comparison | `vsdd-factory:visual-reviewer` |
| Post-pipeline analysis, lessons capture, improvement proposals | `vsdd-factory:session-reviewer` |

#### Routing examples (from prism's recent history)

- **Cross-document consistency defect found by consistency-validator** during a phase gate: correct routing is `product-owner` (owner of BC/PRD content) OR `architect` (owner of ADR content), NOT consistency-validator-fixes-it. The orchestrator dispatches.
- **PR-LEVEL adversarial finding `pub type SensorId = String` shadow alias in prism-query::cache_key** (PREREQ-A pass 1): correct routing is `implementer` (the type is in code, not spec). The fix-burst dispatch happens via orchestrator-drives-cascade pattern (Standing Rule 2) because pr-manager lacks Agent tool access — that's a tooling routing constraint, not a defer-pattern.
- **TDD red-gate violation found by test-writer** where a Red Gate test does not align with the BC: route to `product-owner` (if the BC is the problem) or to the human (if the spec is genuinely contradictory). DO NOT have the test-writer modify the BC silently.
- **Security finding found by security-reviewer**: triage classification is security-reviewer's job. The FIX is implementer's job (with security-reviewer re-running to confirm). Use the `fix-pr-delivery` skill.
- **BC ↔ tracing-emission catalog drift discovered during implementation** (PREREQ-B PG-LP11-001): the implementer must amend BC-2.16.002 Structured Event Catalog in the SAME atomic commit. The implementer is editing the .factory/ artifact in-scope — this is correct-agent because the contract surface and the emission site are both implementer-owned at fix-burst time. Post-merge, state-manager + adversary verify.
- **Out-of-scope finding (legitimate scope-boundary defer)**: still route to orchestrator. Orchestrator records the deferral with explicit future-story attachment per Canonical Principle Rule 3. The deferral target must be a real story ID, not "Wave X" or "later."

#### When the routing is unclear

If a defect doesn't obviously map to a specialist:

1. **Ask the orchestrator first.** The orchestrator has the routing table loaded; let it route.
2. **If the orchestrator is uncertain, the orchestrator asks the human.** This is the legitimate use of human time — routing-table extensions, not domain-fixes-by-wrong-agent.
3. **Default fallback for unmapped work: research → architect.** Most truly novel work that doesn't fit a specialist needs external research first (`vsdd-factory:research-agent`), then architectural decision (`vsdd-factory:architect`).

#### Anti-patterns this principle blocks

- ❌ Adversary rewrites failing tests "to make them pass" (wrong: route to test-writer or implementer).
- ❌ State-manager writes spec content like BC bodies or ADR rationale (wrong: route to product-owner or architect; state-manager handles index rows, frontmatter syncs, decision logs, and cross-document version bumps).
- ❌ Consistency-validator silently edits brief frontmatter (wrong: route to product-owner).
- ❌ Implementer adds a new BC to fix a TDD red-gate (wrong: route to product-owner; implementer cannot author specs).
- ❌ Orchestrator writes the artifact itself when a specialist's output is unsatisfactory (wrong: re-dispatch the specialist with better instructions, or escalate to human).
- ❌ Any agent edits `.factory/STATE.md` directly (wrong: state-manager owns STATE.md).
- ❌ Filing a P4 "opportunistic cleanup" TD when the fix is ~45 minutes of in-scope work (wrong: fix in-scope per Canonical Principle Rule 3 + Rule 4).

#### Conflict with upstream

If a vsdd-factory agent prompt or skill defines a different routing than the table above, this table wins for prism. The upstream canonicalization issue (filed against `drbothen/vsdd-factory`) tracks bringing upstream into alignment.

### Operational Discipline TDs (prism-specific layering)

These project-specific operational rules layer onto the canonical principle. Recorded in `.factory/SESSION-HANDOFF.md` and enforced by the factory-dispatcher hook chain:

- **TD-VSDD-053 — Single-commit-per-burst.** Each logical burst → ONE commit in `.factory/`. Multi-commit chains (HEAD and HEAD^ both containing "backfill" / "Stage 1" / "Stage 2") trigger `MULTI_COMMIT_CHAIN_NOT_ALLOWED`. Recovery procedure documented in "Factory Hook Diagnostics" below.
- **TD-VSDD-059 — Paper-fix detection.** State-manager and adversary must verify every claimed closure has a load-bearing test or assertion, not just a doc-comment or rename. Implementer self-disclosure of risk severity is NOT authoritative — adversary independently verifies.
- **TD-VSDD-060 — Sibling-site sweep on value changes.** When changing a function signature, constant, or canonical identifier, grep for ALL callsites in the same crate (and adjacent crates if `pub`) before committing.
- **TD-VSDD-091 — Anti-volatile-pin.** Narrative spec content must cite function names + behavioral anchors, NOT `file.rs:NNN` line numbers (which decay on subsequent diffs). Justified citations (Red Gate test tables, AC source-of-truth tables, pass-report changelogs) excepted.
- **BC-5.39.001 — 3-CLEAN convergence protocol.** Adversarial cascades require three consecutive clean passes for convergence; any finding resets the streak to 0/3. Applies to both LOCAL and PR-LEVEL cascades.
- **TD-FACTORY-HOOK-BYPASS-001 P0** — Use Edit/Write tools ONLY for `.factory/` mutations. NEVER use Python/sed/echo bypass. Enforced by POL-3.
- **POL-14 — Auto-promotion at merge.** When a story's PR merges, BCs in `behavioral_contracts` frontmatter auto-promote `draft → active`. State-manager runs this transition.

### Conflict resolution

If this principle conflicts with a vsdd-factory agent prompt, skill, or rule, this principle wins for prism. Upstream changes to canonicalize these principles across all VSDD projects are tracked in the `drbothen/vsdd-factory` GitHub issue tracker.

### When in Doubt

If you are an AI agent and you are uncertain whether the production-grade default applies in a specific case, the answer is YES. The principle is the default. Ask only if you have a concrete reason to suspect this case is an exception.

If you are a human reviewing this file and you want to change the principle, edit this file and commit. The principle becomes whatever this file says.

---

## Build & Test

```bash
# TDD inner loop — single crate, fast iteration (~10-30 sec warm)
just iter <crate> [test_filter]
# Examples:
just iter prism-query                              # all prism-query tests
just iter prism-query test_BC_2_11_006             # filtered
# PROPTEST_CASES=32 (8× lower than default 256 for speed; full coverage runs in `just check`)

# Pre-push gate — full strict workspace check (5-8 min cold, ~1 min warm)
just check          # fmt + clippy + nextest + doctests + crate-layout
just check-fast     # clippy + layout only (no tests; for refactor sweeps)

# CI-equivalent local run — adds deny + audit + semver-checks
just check-ci

# Diagnostics
just timings        # cargo build --timings HTML report → target/cargo-timings/
just clippy         # workspace clippy with -D warnings
just fmt            # cargo fmt --all
just cov            # coverage via cargo-llvm-cov

# Specialty (require external toolchain installs)
just kani-local     # Kani formal verification proofs
just fuzz-local <crate> <target>   # cargo-fuzz
just mutants        # mutation testing
just udeps          # unused-dep detection (requires nightly)

# Setup (idempotent)
just setup          # install all dev toolchain extensions
```

**DO NOT** use `cargo test --workspace` directly during iteration — `just iter <crate>` is 5-10× faster.

### TDD Inner Loop Discipline

When iterating through a TDD fix-burst (closing multiple findings in sequence), use the cheapest verification that proves what you need. Match the tool to the question:

| Question | Command | Time (warm) |
|---|---|---|
| Did my single fix make its target test pass? | `cargo nextest run -p <crate> -E 'test(<test_name>)'` | < 1s after build |
| Did my fix break anything in this crate? | `just iter <crate>` | 10-30s |
| See ALL failing tests at once (don't stop at first) | `cargo nextest run -p <crate> --no-fail-fast` | 30-60s |
| Final pre-push gate (workspace canonical) | `just check` | 1min warm / 5-8min cold |

**Common anti-pattern:** running `just check` (full workspace) between every TDD fix in a multi-finding burst. For a 10-fix burst this burns 10-50 minutes that adds nothing the per-crate run wouldn't already have caught. Reserve `just check` for ONCE at end of fix-burst before declaring done.

**Auto-iteration:** `cargo watch -x 'nextest run -p <crate> --no-fail-fast'` re-runs on save — useful for tight feedback when iterating on a single module.

**In-process vs subprocess tests:** Integration tests under `crates/<crate>/tests/` that spawn `prism start` as a subprocess each cost 200-800ms (subprocess overhead + RocksDB open). Unit tests inside `src/*.rs` `#[cfg(test)] mod tests` blocks run in-process at ~5ms. For tight inner-loop iteration on logic, prefer unit tests; reserve subprocess integration tests for behavior that genuinely needs the full binary.

**Deep recursion tests** (depth ≥ 50) MUST wrap with `crates/prism-query/src/tests/util.rs::run_with_deep_stack` to avoid SIGBUS on macOS aarch64's 2MB default test thread stack. See SIGBUS triage in `.factory/STATE.md` D-242 / pass-9.

## Formal Verification (Kani)

Verification properties VP-014 (size limit) and VP-015 (depth limit) have Kani proofs in `crates/prism-query/src/proofs/`. Run them locally with:

```bash
just kani-local            # all crate proofs
cargo kani -p prism-query  # prism-query proofs only
```

**Platform support:** Kani is **Linux/macOS only** (upstream Kani uses CBMC as its backend; Windows is not supported by the Kani project). The `kani-verifier` dev-dependency is gated to non-Windows in `crates/prism-query/Cargo.toml`. Windows contributors should rely on concrete unit tests + CI's Linux/macOS proof job — proof validity is platform-agnostic (Rust code is the same on all platforms; one proof = truth for all).

VP coverage layers:
- **Kani proof** (formal, exhaustive within bounds) — Linux/macOS only
- **Concrete unit tests** (specific points, deterministic) — all platforms
- **Fuzz target `vp021_parse_fuzz`** (random exploration) — Linux CI smoke + nightly long-run

## Git Workflow

### Branch model
- **Default branch:** `main` (release branch, infrequent commits)
- **Active development:** `develop` (PRs target `develop`)
- **Feature branches:** `feature/<story-id>` (e.g., `feature/S-3.01`)
- **Maintenance branches:** `maintenance/<scope>` (e.g., `maintenance/rename-crowdstrike-session`)
- **Worktree pattern:** per-story worktrees in `.worktrees/<story-id>/` for parallel work
- **Factory artifacts branch:** `factory-artifacts` (orphan branch mounted at `.factory/` via worktree). Local-only by default — orchestrator does NOT push factory-artifacts to remote without explicit user authorization.

### Commit conventions
- **Conventional Commits** enforced by `lefthook.yml`:
  - `pre-commit`: fmt + clippy + layout
  - `pre-push`: `just check`
  - `pre-tag`: semver-checks + audit + deny
- **Factory hook chain** (`.factory/` commits): single-commit-per-burst per TD-VSDD-053; MULTI_COMMIT_CHAIN_NOT_ALLOWED detector blocks two consecutive commits with "backfill" / "Stage 1" / "Stage 2" in their subjects. See "Factory Hook Diagnostics" section below for the full recovery procedure.

### Non-negotiable git rules
- **NEVER skip hooks** (`--no-verify`, `--no-gpg-sign`). If a hook fails, investigate and fix the underlying issue. Bypassing is a TD-FACTORY-HOOK-BYPASS-001 P0 violation.
- **NEVER add AI attribution to commits** — no `Co-Authored-By: Claude`, no robot emojis. The user has explicitly directed this for prism.
- **NEVER force-push to `main`.** Force-push to `develop` requires explicit human approval. Force-push to feature/maintenance branches is acceptable when the work is local-only (no collaborators); `--force-with-lease` preferred over raw `--force`.
- **NEVER use destructive operations as a first-line response.** `git reset --hard`, `git clean -f`, `git checkout --` should be the last option after exhausting safer alternatives (`git stash`, `git reset --soft`, worktree-based isolation).

### Operational tips
- **Heredoc workaround:** large commit-message heredocs are sometimes blocked by hook payload limits. When `git commit -m "$(cat <<'EOF' ... EOF)"` fails, write the message to `/tmp/<file>` and use `git commit -F /tmp/<file>`. The Factory Hook Diagnostics section enumerates the specific hook validators that may trigger this.
- **Soft reset for recovery, never `--hard`.** Per the multi-commit-chain recovery procedure: `git -C .factory reset --soft HEAD~N` preserves the working tree state; re-author as a single combined commit.
- **`git stash` for in-progress work** when context-switching between worktrees — preserves uncommitted changes without losing them to a reset.

## Factory Hook Diagnostics

When `Agent` tool dispatches fail with errors like:

```
PreToolUse:Agent hook error: [...factory-dispatcher]: factory-dispatcher trace=<UUID> event=PreToolUse tool=Agent host_abi=1 matched_tiers=N plugins_run=N total_ms=N block_intent=true exit_code=2
```

— the factory-dispatcher hook chain (52 plugins, see `~/.claude/plugins/cache/claude-mp/vsdd-factory/1.0.0-rc.11/hooks-registry.toml`) blocked the dispatch. The error message itself carries NO human-readable reason — only the trace UUID. To diagnose, follow this procedure.

### Step 1 — Locate the dispatcher log

Internal logs live at:

```
.factory/logs/dispatcher-internal-YYYY-MM-DD.jsonl
```

(One file per day, JSONL format, one event per line.)

### Step 2 — Find the block reason

Search the day's log for the trace UUID:

```bash
grep '<TRACE-UUID>' .factory/logs/dispatcher-internal-$(date +%Y-%m-%d).jsonl
```

Look for `plugin.log` entries with `level: warn` — those carry the human-readable block reason as an embedded multi-line `message` field. Example payload from a real block:

```
"FAIL: MULTI_COMMIT_CHAIN_NOT_ALLOWED — HEAD and HEAD^ both contain 'backfill'.
 The single-commit protocol (TD-VSDD-053) does not use backfill commits.
 ...
 Recover with: git -C .factory reset --soft HEAD~2 then re-author as a single commit"
```

The `plugin_name` field on the same record (e.g., `validate-wave-gate-prerequisite`, `validate-pr-merge-prerequisites`, `regression-gate`) tells you which guard fired.

### Step 3 — Common blockers and recovery procedures

| Blocker | Detection | Recovery |
|---------|-----------|----------|
| **Multi-commit chain (TD-VSDD-053)** | HEAD and HEAD^ both have `backfill` / `Stage 1` / `Stage 2` in their commit messages | `git -C .factory reset --soft HEAD~N` (preserves working tree); re-author as one combined commit; force-push with `--force-with-lease` (requires explicit user approval) |
| **SHA drift** | STATE.md or SESSION-HANDOFF.md cite a develop SHA that doesn't match `git rev-parse origin/develop` | Update narrative via state-manager dispatch; STATE.md `develop_head` and SESSION-HANDOFF cited SHAs must match `c98a38b0` (or current `git -C . log -1 --format=%H develop`) |
| **In-progress narrative** | STATE.md decision log has an open phase without closure | Add closure row via state-manager; bump version |
| **factory-artifacts dirty** | `git -C .factory status --porcelain` is non-empty | Commit/discard pending changes via state-manager |

### Step 4 — Re-run the validator before re-dispatching

```bash
bash .factory/hooks/verify-sha-currency.sh
```

Expected: exit 0 with `PASS` lines and no `FAIL` lines. If it still fails, repeat Step 2 with the new dispatch's trace.

### Step 5 — Going-forward discipline (orchestrator)

To avoid the multi-commit-chain block:

- **Bundle backfills.** When state-manager performs multi-document backfills (e.g., adversary pass-N report + fix-pass-N closure report), stage all files THEN commit ONCE. Never two state-manager dispatches in a row both producing "backfill" commits.
- **Single-commit-per-burst.** Each logical burst (one adversary cascade step, one fix-pass cycle, one phase transition) → one commit in `.factory/`. Multiple consecutive commits with the same theme word (`backfill`, `Stage`) trigger the chain detector.
- **Soft-reset for recovery, never `--hard`.** The working tree state is what we want to preserve.
- **Force-push always needs user approval.** Per project git-safety protocol; orchestrator must request it from the human.

### Hook source locations (read-only reference)

- Dispatcher binary: `~/.claude/plugins/cache/claude-mp/vsdd-factory/<version>/hooks/dispatcher/bin/<platform>/factory-dispatcher`
- Hook registry config: `~/.claude/plugins/cache/claude-mp/vsdd-factory/<version>/hooks-registry.toml`
- Hook plugins (WASM): `~/.claude/plugins/cache/claude-mp/vsdd-factory/<version>/hook-plugins/*.wasm`
- Project-side validator scripts: `.factory/hooks/*.sh` (e.g., `verify-sha-currency.sh`)

## Project References

| Path | Description |
|------|-------------|
| `.factory/STATE.md` | Live pipeline state (current phase, decisions log, session resume checkpoint) |
| `.factory/SESSION-HANDOFF.md` | Resume-ready handoff for new sessions |
| `.factory/specs/architecture/` | Architecture docs + ADRs + ARCH-INDEX.md (subsystem registry) |
| `.factory/specs/behavioral-contracts/` | BC files + BC-INDEX.md |
| `.factory/specs/verification-properties/` | VP files + VP-INDEX.md (Kani proofs + fuzz targets) |
| `.factory/specs/domain-spec/` | L2 domain spec (entities, invariants, capabilities, edge cases) |
| `.factory/stories/` | Per-story implementation specs + STORY-INDEX.md |
| `.factory/research/` | Cited research artifacts (e.g., build-optimization-2026.md) |
| `.factory/policies.yaml` | Project governance policy registry (10 baseline + project-specific) |
| `docs/dev-setup.md` | Dev environment setup + build performance notes |
| `crates/` | 24-crate Rust workspace (parser, sensors, DTU clones, MCP, etc.) |
| `tests/external/perimeter-violation/` | Compile-fail test crate enforcing prism-query security perimeter |
| `fuzz/` | cargo-fuzz targets (vp021_parse_fuzz, etc.) |
| `Justfile` | Task runner — `just --list` for current recipes |
| `lefthook.yml` | Pre-commit/push/tag git hook config |
| `rust-toolchain.toml` | Pinned Rust toolchain channel + components + targets |
