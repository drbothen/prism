# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

> **Toolchain:** Rust stable (per `rust-toolchain.toml`), edition 2024, resolver 2. Components: rustfmt, clippy, rust-src. Cross-compile targets: aarch64-apple-darwin, x86_64-apple-darwin, x86_64-unknown-linux-gnu, x86_64-unknown-linux-musl, x86_64-pc-windows-msvc. 26-crate workspace (25 once ADR-037 retires prism-customer-config; root Cargo.toml `members` is the source of truth).

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

Per-story Phase 3 sub-workflow: stubs → failing tests → TDD green → LOCAL adversary 3-CLEAN → **story-level holdout gate** → demo-recorder per-AC → push → pr-manager 9-step PR cycle → squash-merge → state-manager post-merge burst. BC-5.39.001 3-CLEAN protocol applies to every cascade.

**Story-level holdout gate (human-approved 2026-07-13):** product-owner authors 2–4 HIDDEN, SINGLE-USE holdout scenarios per story at story-materialization time (same touchpoint as the remove-uncertainty pass), stored under the holdout directory that test-writer/implementer never read. After LOCAL 3-CLEAN and BEFORE demo recording/push, the holdout-evaluator runs the scenarios against the story's built binary (real MCP stdio + DTU, wire-level assertions, scoped to the story's touched surface). The gate is BLOCKING: any unsatisfied scenario routes findings through the VSDD feedback loop as OBSERVED BEHAVIOR ONLY (never scenario text — contamination control) and resets the LOCAL streak per BC-5.39.001. Consumed scenarios are marked used and never reused. Wave-level and Phase-4 holdout pools are separate and unchanged. Origin: T13 live-audit triage (D-1715/D-1716) — end-to-end observed-output evaluation caught 3 defects that 5,483 tests (workspace test count at the D-1715 live-audit triage, 2026-07-13) and the adversarial cascades missed.

---

## Orchestrator Auto-Recovery Heartbeat (Standing Operating Procedure)

A durable scheduled heartbeat keeps autonomous runs self-healing — it auto-recovers from hard stalls, API errors, and dead background agents so a run never sits stuck. Background agents already auto-notify the orchestrator on completion **and** failure (per-agent failures are recoverable while the main loop is alive); the heartbeat covers the **hard-stall** case (main loop dead, or idle with no pending notification) that only an external durable scheduler can revive. Standing rule (human-directed 2026-08-30):

1. **Every session, as the FIRST orchestrator action on startup/resume** — before reading STATE.md for pipeline work — verify the heartbeat exists: call `CronList`. If it is **absent or expired**, RE-ARM immediately via `CronCreate` (`durable: true`, `recurring: true`) per the runbook `.factory/ops/vsdd-heartbeat-autorecovery.md`. This makes re-establishment automatic on every resume, independent of whether `.claude/scheduled_tasks.json` survived.
2. **Mechanism:** durable recurring cron (schedule `8,23,38,53 * * * *` — every 15 min, deliberately off the `:00`/`:30` fleet-collision marks), persisted to `.claude/scheduled_tasks.json`, fires only while the REPL is idle. Recurring crons auto-expire after 7 days; the heartbeat's own routine self-re-arms before expiry, and this resume-time check is the backstop. Use `CronCreate durable:true` (NOT `ScheduleWakeup`, which is in-session only and dies with the process).
3. **What each tick does (idempotent — never duplicate in-flight work):** orient (git rev-parse + `verify-sha-currency.sh` + STATE.md) → `TaskList` and re-dispatch any failed/stalled agent after re-verifying on-disk state → resume the next critical-path step if idle → no-op if healthy/in-flight → self-perpetuate → checkpoint STATE.md via state-manager if stale.
4. **Homes (one source of truth per layer):** this CLAUDE.md rule is the authoritative, permanent standing rule; the full reproducible/portable procedure (parameterized prompt template + per-project install steps) lives in `.factory/ops/vsdd-heartbeat-autorecovery.md`; STATE.md / SESSION-HANDOFF carry only a short resume *pointer* (STATE.md is compacted and is NOT a home for permanent procedure). Portable across projects; intended for eventual promotion into the vsdd-factory engine `HEARTBEAT.md`.

The heartbeat is a safety net, not a substitute for the orchestrator's normal completion-notification-driven loop.

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
- [ ] Did I discharge and report ALL THREE sweep dimensions — sibling pair, downstream copy target, and mandate anchor? (TD-VSDD-097 / POL-29.) A content-text grep plus a version-pin grep is NOT a discharge. If I wrote a `MUST`, did I name the story + AC + Red Gate test that executes it?

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
- **BC ↔ tracing-emission catalog drift discovered during implementation** (PREREQ-B PG-LP11-001): the implementer must amend the Canonical Structured Event Catalog in BC-2.16.002 §Postconditions in the SAME atomic commit. The implementer is editing the .factory/ artifact in-scope — this is correct-agent because the contract surface and the emission site are both implementer-owned at fix-burst time. Post-merge, state-manager + adversary verify.
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
- **TD-VSDD-097 — Three-dimension sweep checklist (amended 2026-07-27; operationalizes POL-29).** POL-29 (`within_fb_sibling_sweep_discipline`) has existed as a HIGH policy throughout, yet three sibling-sweep failures occurred in a single perimeter on 2026-07-27 (pass-66 CRIT-001, HIGH-002/HIGH-003, MED-001), meeting the 3-recurrence codification threshold. Diagnosis: POL-29 was phrased as a general exhortation to "sweep sibling sites," which agents satisfied by grepping for the *changed string*. The three misses were in dimensions a string-grep cannot reach. Every fix-burst MUST therefore explicitly discharge and report all THREE dimensions — a content-text grep and a version-pin grep alone are NOT a POL-29 discharge:
  1. **Sibling pair.** If the edited artifact has a twin — created by the same split, sharing a subsystem and capability, or documented as a pair — NAME the twin and sweep it, even when the changed string does not appear in it. Absence of the string is the failure mode, not evidence of cleanliness. _Precedent: BC-2.01.018 (Alerts) had cursor language removed while its ADR-053-D3 twin BC-2.01.006 (Assets) kept it, and the same burst wrote a NEW false cross-reference asserting Assets "retains" cursor pagination._
  2. **Downstream copy target.** If any section of the edited artifact is copied verbatim into a downstream artifact by a later agent leg (e.g. an ADR §D-section the product-owner transcribes into a BC), sweep that section and its copy in the SAME burst. A corrected artifact that retains a stale copy-source section propagates its own pre-correction text. _Precedent: ADR-056 v0.2 corrected §D3/§D4/§D5/§D9 but not §D8, the verbatim PO copy-source, which then carried pre-correction phrasing into BC-2.16.002._
  3. **Mandate anchor.** Any `MUST` written into a BC or spec MUST name the story + AC + Red Gate test that will execute it, or record a deferral against a REAL existing story ID per Canonical Principle Rule 3. An unanchored `MUST` cannot be tracked to completion and will be re-minted as a finding every pass. _Precedent: two §TOML Contract `MUST` blocks were authored with zero story anchors in the same amendment that correctly anchored EC-02-014 to a real story._

  The orchestrator MUST include these three clauses in every fix-burst dispatch, and the fixer MUST report a per-dimension verdict. The adversary independently verifies the discharge.
- **TD-VSDD-091 — Anti-volatile-pin (amended 2026-07-24).** Narrative spec content must cite function names + behavioral anchors, NOT `file.rs:NNN` line numbers (which decay on subsequent diffs). Justified citations (Red Gate test tables, AC source-of-truth tables) excepted. **Amendment 2026-07-24 — "pass-report changelogs" exception retired:** the original exception for changelog/ledger/ratification text is revoked. ALL record-tier text — adversary pass reports, changelog rows, ratification memos, STATE.md decision-log entries, burst-log rows — MUST use section/symbol/anchor cites ONLY. Rationale (evidence from CLIP email-notifications Stage-3 cascade, six consecutive passes): six adversary passes minted frame/off-by-one/arithmetic findings that existed SOLELY because prior record text cited volatile line numbers; every same-burst record insertion self-invalidated earlier cites written minutes before in the same pass. A record that contains a line number it cannot independently verify has already drifted by the time the commit lands. Enforced by the mechanical gate in TD-VSDD-092 (`scripts/records-lint.sh` check L9). _Cross-applied from the CLIP email-notifications Stage-3 cascade (trend-gate #4 structural intervention + S3-39..S3-42 evidence), human-directed 2026-07-24._
- **TD-VSDD-092 — Mechanical Records-Lint Gate.** A deterministic pre-commit script (`scripts/records-lint.sh`) the orchestrator runs before every state-manager/factory commit; exit 1 blocks. Four checks: **L1** — document's `version:` frontmatter must match the top (latest) changelog row version (prism convention: top = latest). **L7** — changelog version numbers must be in descending order top-to-bottom; a row that exceeds the row above it is a violation. **L9** — staged additions to `.factory/` record files must not contain volatile line-cite forms; pre-existing unchanged lines are grandfathered (ratchet scoping — see script header). `L9_CITE_PATTERN` covers five arms: (1) `filename.ext:NNN` — file-extension-anchored line cites; (2) backtick-quoted filename followed by a `line`/`lines` keyword; (3) `[Ll]ine(s) ~NNN` including en-dash ranges; (4) `DOCNAME vX.Y:NNN` doc-version cites; (5) `~L<NNN>` tilde+L form and bare `L<NNN>` positional cites — covers VP-INDEX changelog forms and burst-log/adversary-review forms; 2+ digits required to exclude `~L2`/`~L3` OSI layer refs and single-digit check names (`L1`/`L7`/`L9`). L9 queries **both** the main project index and the `.factory/` worktree index directly — because `.factory/` is a separate git worktree on the `factory-artifacts` orphan branch with its own index, and the main project index is blind to `.factory/` additions. **L10** — cross-document index↔artifact version consistency: for each pinned artifact row in `BC-INDEX.md`, `ARCH-INDEX.md`, and `VP-INDEX.md`, the row's version pin must equal the artifact's frontmatter `version:`. Detects STALE (index pin < artifact version) and PHANTOM (index pin > artifact version). Runs corpus-wide on every commit rather than staged-only, because index files may not themselves be staged. **L10 capability boundary (understated by design):** L10 detects the version-number half of index drift only. It cannot detect content falsification — a row describing a change that does not exist in the target artifact, which is the defect class that originally triggered this check. It also cannot verify rows carrying no structural version pin; 434 of 496 BC-INDEX rows use other formatting conventions and are counted as **unverifiable, not clean**. A "0 mismatches" result from L10 is NOT equivalent to "every index row verified." L1/L7 also run in ratchet mode by default (staged files only); `--full-scan` enables corpus audit. `--self-probe` verifies all 34 pass/fail cases; confirmed 34/34 (2026-07-25: 18 L1/L7/L9 + 16 L10 including range PASS/STALE). Config block at script top for the prism artifact map (BC, ADR, VP, prd-supplements, stories, architecture/flat directories). _Cross-applied from the CLIP email-notifications Stage-3 cascade (trend-gate #4 structural intervention + S3-39..S3-42 evidence), human-directed 2026-07-24._ **L9 operational gap disclosure (2026-07-24):** L9 was inoperative from its introduction until 2026-07-24. Because `.factory/` is a separate worktree with its own index, `git diff --cached` from the project root always returned empty, and L9 early-exited on every commit without scanning a single `.factory/` addition. Any L9 pass recorded before 2026-07-24 carries no evidentiary weight, and TD-VSDD-091 compliance in `.factory/` records committed before that date was never mechanically enforced. The gate's `--self-probe` passed 6/6 throughout that period — a green self-probe on synthetic temp repos coexisted with a production check that never fired. The distinction (probe-passes ≠ gate-fires) is the transferable lesson.
- **TD-VSDD-096 — Records-Only Micro-Burst.** When an adversarial or review pass returns ONLY records-tier LOW/OBS findings — no content/mechanism defects, no logic/correctness/security/API-contract issues — run a 2-step burst instead of the full cascade ceremony: (1) ownership-routed fixer dispatched for the records findings (state-manager for STATE.md/burst-log entries; spec-steward for BC/ADR/VP body text); (2) state-manager lint-gated commit (`scripts/records-lint.sh` must exit 0 before the commit lands). "Records-tier" findings are: changelog ordering violations (L7), volatile-line-cite violations (L9, TD-VSDD-091/092), frontmatter/changelog version mismatches (L1), stale version references in record prose, minor narrative-only prose inconsistencies with zero behavioral impact. Any finding that touches correctness, mechanism, algorithm, state machine, or API contract reverts to full ceremony. The fixer self-certifies that no content-mechanism changes are included; adversary independently verifies on the next pass. _Cross-applied from the CLIP email-notifications Stage-3 cascade (trend-gate #4 structural intervention + S3-39..S3-42 evidence), human-directed 2026-07-24._
- **BC-5.39.001 — 3-CLEAN convergence protocol.** Adversarial cascades require three consecutive clean passes for convergence; any finding resets the streak to 0/3. Applies to both LOCAL and PR-LEVEL cascades.

  **Strict vs PR-Merge Convergence Disambiguation** (amendment 2026-05-22, D-779):

  The CLEAN status reported by the adversary at the end of each pass has TWO INTERPRETATIONS:

  - **CLEAN (strict)** — ZERO findings of ANY severity (CRIT + HIGH + MED + LOW + OBS + PROCESS-GAP). This is the criterion required for **streak advancement** under BC-5.39.001 3-CLEAN. The 3-CLEAN streak advances only when 3 consecutive passes are CLEAN (strict).

  - **CLEAN (PR-merge)** — ZERO findings of CRIT + HIGH + MED severity (LOW/OBS/PROCESS-GAP findings present but non-blocking). This is a PR-merge-gate threshold ONLY; it does NOT advance the 3-CLEAN streak.

  **Adversary CLEAN reports MUST specify both criteria explicitly.** Recommended report format:

  ```
  CLEAN (strict): yes/no
  CLEAN (PR-merge): yes/no
  ```

  **Orchestrator dispatch decisions** for fix-bursts use the STRICT criterion. If CLEAN(strict)=no, orchestrator dispatches a fix-burst regardless of CLEAN(PR-merge) status.

  **Frozen-HEAD streak rule (DRIFT-ORCH-PRLEVEL-PUSH-001, 2026-06-08):** the 3-CLEAN streak only counts consecutive CLEAN(strict) passes taken against an UNCHANGED feature/PR HEAD. Pushing any new commit to the branch mid-cascade — a fix-burst, evidence refresh, or rebase — RESETS the streak to 0/3; the cascade must re-gate on the newly-pushed HEAD. Never count a pass taken before a push toward a streak completed after it. (Origin: S-DEMO-CLAROTY-PAGINATION-001 PR-LEVEL cascade, where a fix was pushed before re-gating.)

  **Rationale:** This disambiguation eliminates the cascade-internal mismatch where adversary CLEAN flag and orchestrator dispatch decision operated on different interpretations of the same flag. Evidence: PLUGIN-MIGRATION-001-D cascade passes 7-9; session-review-2026-05-22.md D-777 proposal B1; lessons 28, 31, 33 in cycles/wave-0-plugin-prereqs/lessons.md.

- **TD-FACTORY-HOOK-BYPASS-001 P0** — Use Edit/Write tools ONLY for `.factory/` mutations. NEVER use Python/sed/echo bypass. Enforced by POL-3.
- **POL-14 — Auto-promotion at merge.** When a story's PR merges, BCs in `behavioral_contracts` frontmatter auto-promote `draft → active`. State-manager runs this transition.

## Conventions (Code-Level)

Prism-specific coding patterns enforced by CI and/or adversarial review. These are non-negotiable under the production-grade default — violations are bugs, not style preferences.

### Highlights

- **`#[non_exhaustive]` discipline.** All public TOML-deserialized types and pub-API surface types require `#[non_exhaustive]`. Enforcement is via the compile-fail gate at `tests/external/non-exhaustive-violation/`; for the current enforced count run `python3 scripts/check-non-exhaustive-per-symbol.py --count` (AC-5 of S-PLUGIN-PREREQ-C, expanded by S-DEMO-DTU-LIVE-SCENARIO-001-A AC-014 and S-DEMO-DTU-LIVE-SCENARIO-001-B, S-DEMO-MULTI-TENANT-DTU-001 D-1075-API-GAP-001, S-5.02 HIGH-3 + followup (StructuredErrorFields, CapabilityEntry, ResolutionStep, CapabilityStatus), S-3.13 LOW-1 (TableNotAvailableDetails), S-3.13 CR-002 (TableRegistry), S-1.14-REDO burst-2 MED-1-RESIDUAL (Tier3CacheEntry), S-1.14-REDO fix-burst FIX-IN-SCOPE (InfusionUdfDescriptor, EnrichStageDescriptor), S-1.14-REDO adversarial OBS-1 FIX-IN-SCOPE (InfusionError), S-5.03 (ClientInventoryEntry, SensorConfigEntry, SensorHealthResult, RateLimitInfo, ResourcePressure, SensorHealthStructuredContent), S-DEMO-ENRICHMENT-PIVOT-002 (HttpLookupAuthType, HttpLookupCredentialConfig, HttpLookupConfig), S-DEMO-PRISMQL-ONBOARDING-001-A (PrismDescribeResponse, TableDescriptor, ColumnDescriptor), S-DEMO-PRISMQL-ONBOARDING-001-B (ColumnNotFoundDetails), S-5.04 F-S504-P5-002 (HealthSummary), S-DEMO-PRISMQL-GRAMMAR-REMEDIATION-001 (ExampleKind, SqlPipeQuery, UnknownSourceTableDetails), S-DEMO-FIDELITY-REMEDIATION-001 (EnrichUdfNotFoundDetails), S-PRISMQL-NATIVE-TEMPORAL-TYPING-001 (TemporalLiteralPosition), DEFECT-CSDEVICES-EMPTY-PIPELINE-001 F-CSD-P28-OBS-001 (prism_core VirtualField), DEFECT-CSDEVICES-EMPTY-PIPELINE-001 F-CSD-P31-OBS-002 (prism_query ast VirtualField), DEFECT-PQL-FNCALL-LHS-001 F-PQLFN-PR11-OBS-002 (ParseError)); `EXPECTED_SYMBOLS` in `scripts/check-non-exhaustive-per-symbol.py` is the **single source of truth** — when the gate grows, append the new symbol to that list ONLY. `EXPECTED_COUNT` in the same file is derived from the list length and needs no manual update; `scripts/check-non-exhaustive.sh` reads the count from the Python manifest automatically; ci.yml delegates to both scripts and carries no separate `EXPECTED` value. Do NOT restate the count in prose anywhere — including in this sentence. Layer 1 is an **equality** check, so BOTH a removed annotation and an unregistered new type fail CI with distinct diagnostics. (Before 2026-07-27 the count was duplicated across four locations and Layer 1 was a `-lt` floor, which let an unregistered new type pass CI silently; human-approved collapse to one source of truth.) External match arms must include a wildcard `_ => {}` arm. New public types added to `prism-core`, `prism-spec-engine`, or `prism-query` need `#[non_exhaustive]` added before the PR can merge.

- **Arc-DI plumbing.** Production runtime wires dependencies via `Arc<dyn ...>` constructors per ADR-022. The placeholder-construct anti-pattern (constructing a type without wiring real Arc dependencies "for now") is explicitly forbidden (Standing Rule 3 §4 in SESSION-HANDOFF.md). Adding `Arc<dyn Foo>` to a constructor that lacked it is "wiring, not redesign" and must be done in-scope.

- **Structured event catalog discipline.** Every `tracing::*!(event_type=…)` site must appear as a row in the Canonical Structured Event Catalog in BC-2.16.002 §Postconditions (BC-2.16.002's H1 title is *Multi-Step Fetch Pipeline Execution — Sequential Steps with Variable Interpolation*; the event catalog is a §Postconditions sub-section within that contract, not the contract's title) with full field schema, audit role, and recurrence policy (PG-LP11-001, established during S-PLUGIN-PREREQ-B cascade). New emission sites added without a corresponding BC-2.16.002 catalog row are a P1 finding in adversarial review.

- **Newtype + redacted `Debug` for credentials.** Sensitive types (`AuthToken`, `OrgSlug`, credential names) use newtypes with redacted `Debug` impls. `OrgSlug::new_unchecked` is a `pub` validation-bypass constructor guarded by a symbol-keyed allowlist audit test (`crates/prism-core/tests/new_unchecked_audit.rs`) rather than a Cargo feature gate — `#[cfg(test)]` does not propagate to downstream crates' test builds, so the audit-test-as-compensating-control mechanism is the ratified exception (human-approved 2026-06-10). It must never appear in production code paths; new call sites require an allowlist entry with justification. Credential values never transit AI context (AD-017; see project memory `project_ai_opaque_credentials.md`).

- **ColumnType canonical naming.** `prism_core::column::ColumnType` (variants `String / Integer / Float / Boolean / Datetime / Json`) is the canonical sensor schema API (ADR-024). The retired shadow enum `prism_spec_engine::types::ColumnType` must not be reintroduced. The distinct `prism_core::types::ColumnType` (variants `Text / Int64 / UInt64 / Float64 / Bool / Timestamp / Json / Bytes`) serves internal table schemas only — do not conflate the two.

- **Error taxonomy.** Use `SpecEngineError` for spec-engine failures, `E-QUERY-NNN` codes for query engine errors, `E-SENSOR-NNN` for sensor adapter errors — all defined in `.factory/specs/prd-supplements/error-taxonomy.md`. No `unwrap()` or `expect()` in critical code paths. No silent `Vec::new()` return where partial-failure data should propagate (Standing Rule 3 §2).

- **No `println!` in production code.** Use `tracing::*!` with structured fields only. `println!` is restricted to examples and CLI formatting helpers.

- **Perimeter-violation compile-fail gates.** `tests/external/perimeter-violation/` is the canonical pattern for enforcing pub-API surface invariants (E0432 from S-PLUGIN-PREREQ-A, E0639 from S-PLUGIN-PREREQ-C). New compile-fail gates for future API surface invariants use this crate as the template.

- **Single-workspace MSRV.** Toolchain is pinned via `rust-toolchain.toml`. No per-crate MSRV divergence. All crates build on the single pinned channel.

- **OCSF normalization.** Sensor adapters emit OCSF + protobuf shapes per the project vision (ephemeral federated query engine, see `project_core_architecture_insight.md` memory). Raw API responses are not forwarded; they are normalized at the adapter boundary.

- **HTTP client timeout.** Production `reqwest::Client` instances must use `.timeout(Duration::from_secs(30))`. The historical PipelineExecutor gap (TD-S-PLUGIN-PREREQ-B-005) was closed by PR #149 (plugin clients via `PLUGIN_HTTP_CLIENT_TIMEOUT_SECS`, boot.rs) and PR #166 (`build_http_client_with_timeout()` on the spec-driven adapter path); verified closed 2026-06-10. The rule remains binding for all new clients.

- **`reqwest` TLS backend — rustls-tls mandatory (ADR-050).** Every `reqwest` dependency entry in the workspace — `[dependencies]`, `[dev-dependencies]`, and optional/feature-gated entries — must declare `default-features = false, features = ["rustls-tls"]`. Omitting `default-features = false` silently enables `native-tls`, which causes ~65s macOS Keychain init overhead and opens a corporate MITM proxy interception path for outbound sensor API credentials. The `native-tls` feature and its aliases (`default-tls`, `native-tls-alpn`, `native-tls-vendored`) are forbidden workspace-wide. New workspace crates must declare `rustls-tls` at first write — there is no acceptable "fix in a follow-up" (ADR-050 D3).

- **Wire-shape assertion discipline (2026-07-13, human-approved).** Any test covering an MCP-visible surface (tool responses, resources) must include at least one assertion on the **serialized JSON output** — the exact envelope/row bytes the LLM agent consumes — not only pre-serialization Rust structures. NULL vs absent vs empty distinctions MUST be asserted at the wire level (BC-2.11.001 EC-11-079 row-shape null-not-absent). Origin: live-audit [C3]/[H20] escape — `arrow_json` `explicit_nulls` default silently omitted NULL keys; 5,483 tests (workspace test count at the D-1715 live-audit triage, 2026-07-13) missed it because none asserted serialized row shape.

### File size & module splitting

**No CI-enforced file-size gate exists in prism today.** Function-level complexity is gated via `clippy.toml` (`cognitive-complexity-threshold = 30`, `too-many-arguments-threshold = 8`). There is no file-level line-count check in CI, the Justfile, or anywhere in the workspace — no `xtask` crate, no `tokei` gate, no allowlist file. This absence is tracked debt, not a ratified exception (TD-DECOMP-RATCHET-001 in the tech-debt register).

**Oversized files are scheduled decomposition debt, not cohesion justifications.** Prism has 12 production files exceeding 2,000 lines (42 exceeding 1,000 lines), of which 4 exceed 5,000. Raw counts are inflated by inline `#[cfg(test)]` modules — `engine.rs` shows 17,041 total but carries ~12,100 lines of inline tests; its production logic is ~4,900 lines. Each file exceeding 2,000 lines is individually registered in the tech-debt register under TD-DECOMP-EPIC-001 with a decomposition story anchor. They are NOT silent exceptions; they are scheduled work.

**Soft authoring target: ~800 lines per production file.** When a new file or an existing file receiving additions approaches 800 lines, evaluate concern boundaries before continuing. A 900-line file that is genuinely single-concern is acceptable. A 900-line file that has accumulated two or three distinct responsibilities has started down the giant-file path and should be split now, not deferred.

**Ratchet rule — proposed gate, not yet enforced.** A follow-up story anchored under TD-DECOMP-EPIC-001 will implement `just check-file-sizes` — a `tokei`-backed Justfile recipe plus CI check — enforcing that no production `.rs` file exceeds 1,500 lines unless it appears in `.factory/file-size-allowlist.toml`. Adding a file to the allowlist requires a concrete decomposition story anchor; blanket "cohesion" rationales are not sufficient. The ratchet prevents new giants from accumulating; it does NOT retroactively fail the existing 12 registered >2,000-line files (which enter the allowlist on day 1 of the gate story and are removed as decompositions land). Until the gate ships, the rule operates as an authoring discipline: any PR growing a file past 1,500 lines must include a decomposition rationale in the PR description, citing a TD-DECOMP-EPIC-001 anchor story.

**`mod.rs` convention:** prism uses `mod.rs` for both re-exports and module-level logic (middleware, trait implementations, route builders). This is intentional — `routes/mod.rs` files own the router builder and shared middleware; `auth/mod.rs` owns the `SensorAuth` open trait. Do not enforce a re-export-only constraint; it contradicts the codebase. However, a `mod.rs` approaching 1,000 lines is a decomposition candidate even under this convention: `plugin/mod.rs` at 1,880 lines is registered under TD-DECOMP-EPIC-001.

**Cohesion guides where to split, not whether to split.** The split-by-concern heuristic still applies: additions that belong to the same single responsibility stay in the same file. But "this file owns the full query engine" names the starting point for a concern-boundary analysis (`engine/plan_gates.rs`, `engine/column_resolution.rs`, `engine/execution.rs`, `engine.rs` as orchestrator) — it does not justify a 17,000-line file. Citing cohesion to resist splitting a file already registered in the allowlist is an anti-pattern. **Over-fragmentation is also an anti-pattern** — do not split into many ~100-line files; a Rust file is a privacy boundary, and fragmentation creates grep-hostile `pub(crate)` noise without improving reviewability.

### Forbidden patterns

| Pattern | Reason |
|---------|--------|
| `prism_spec_engine::types::ColumnType::Int64` / `::Float64` / `::Timestamp` | Retired shadow enum variants (ADR-024); use `prism_core::column::ColumnType::Integer` / `::Float` / `::Datetime` |
| `lifecycle: active` in BC frontmatter | Retired field (ADR-025); use `lifecycle_status: active` + `status:` per ADR-021 |
| `OrgSlug::new_unchecked` in any production code path, or a new call site without a `new_unchecked_audit.rs` allowlist entry | Credential safety (AD-017); enforced by the symbol-keyed audit test, not a feature gate |
| `Arc::new(SomeThing::placeholder())` style stub construction in production boot path | ADR-022 wiring contract; placeholder-construct is Standing Rule 3 §4 violation |
| `reqwest::Client::new()` without `.timeout()` in production code | Must set 30s timeout (TD-S-PLUGIN-PREREQ-B-005 precedent, closed 2026-06-10) |
| `reqwest` dep without `default-features = false` or with `native-tls` / `default-tls` / `native-tls-alpn` / `native-tls-vendored` feature | ADR-050 D1/D2: native-tls causes ~65s macOS Keychain init in tests and allows MITM proxy interception of sensor API credentials; use `rustls-tls` |
| `unwrap()` / `expect()` on `Result` in non-test code paths | Error taxonomy rule; use `?` + structured `SpecEngineError` / `PrismError` variants |
| `tracing::*!(event_type=…)` without BC-2.16.002 catalog row | PG-LP11-001; structured event catalog must be kept in sync |

### Error handling

- Sensor adapter errors: return `SpecEngineError` variants from `.factory/specs/prd-supplements/error-taxonomy.md` `E-SENSOR-NNN` namespace.
- Query engine errors: `E-QUERY-NNN` variants from `prism_core::error::PrismError`.
- Partial failures in fan-out: propagate via `prism-query` partial-failure handling (BC-2.01.010); do not swallow and return empty `Vec`.
- Boot step failures: exit codes per ADR-022 §A table; `exit(4)` for audit init failure (BC-2.05.012), `exit(5)` for credential init failure (BC-2.03.013).

### Logging

- Use `tracing::info!` / `tracing::warn!` / `tracing::error!` / `tracing::debug!` with structured field syntax: `tracing::info!(sensor_id = %id, event_type = "fetch.started", "fetching sensor data")`.
- All `event_type` values must be registered in the Canonical Structured Event Catalog in BC-2.16.002 §Postconditions before the PR merges.
- Log target discipline: 18 diagnostic targets defined in `architecture/observability.md`; match the target to the subsystem.

### Channels / async

- Tokio multi-threaded runtime (AD-013). All sensor fan-out is async. Do not block the tokio thread pool with synchronous I/O.
- Arc-swap for config hot-reload (AD-007): read via `ArcSwap::load()`, not via Mutex. In-flight queries hold a snapshot reference across their lifetime.
- Concurrency permit limits — two distinct subsystems, do not conflate: (a) **query fan-out** uses `MAX_FANOUT_CONCURRENCY = 10` (`prism-sensors/src/fanout.rs`, BC-2.01.002) nested with the global `HTTP_SEMAPHORE_PERMITS = 200` (`http.rs`) — one fan-out permit + one HTTP permit per task is the intended nested pattern; (b) the **8/8 split** (ADR-022 §D, D-209) applies to the prism-operations scheduler/action-delivery subsystem only (see `architecture/concurrency-architecture.md`). Do not acquire pools across subsystems simultaneously without explicit justification.

### Conflict resolution

If this principle conflicts with a vsdd-factory agent prompt, skill, or rule, this principle wins for prism. Upstream changes to canonicalize these principles across all VSDD projects are tracked in the `drbothen/vsdd-factory` GitHub issue tracker.

### When in Doubt

If you are an AI agent and you are uncertain whether the production-grade default applies in a specific case, the answer is YES. The principle is the default. Ask only if you have a concrete reason to suspect this case is an exception.

If you are a human reviewing this file and you want to change the principle, edit this file and commit. The principle becomes whatever this file says.

---

## Standing Adversary Probes & Implementer Disciplines

These are project-local standing rules layered onto the upstream vsdd-factory agent prompts. Source: session-review-2026-05-22 D-777 codification of PLUGIN-MIGRATION-001-D Option B exit lessons (entries 16, 17, 19, 24 in `cycles/wave-0-plugin-prereqs/lessons.md`).

### SAP-1 — Adversary standing probe: tracing emission catalog completeness

For EVERY adversarial pass on stories or PRs touching `crates/**/*.rs`:

1. Grep `event_type =` across the entire `crates/` workspace (not just changed files): `rg 'event_type\s*=' crates/ --type rust`
2. For each `event_type` value found, verify a corresponding row exists in BC-2.16.002 §Postconditions (Canonical Structured Event Catalog) with full field schema, audit role, and recurrence policy
3. Tracing emission WITHOUT a catalog row = **P1 finding** per CLAUDE.md §Conventions structured event catalog discipline
4. Same-commit catalog row required for emissions added in branch
5. Removal of an emission (e.g., replaced by `?` propagation) does NOT require a new catalog row — `?` propagation provides audit trail without catalog overhead (D-765 precedent)

Source: PLUGIN-MIGRATION-001-D pass-2 FB-IMPL-1 + FB-IMPL-2 (2 recurrences); lessons 16, 19.

### SAP-2 — Adversary standing probe: DTU↔TOML schema parity (sensor-spec stories)

For ANY adversarial pass on stories or PRs touching `.prism/specs/sensors/*.toml` or equivalent sensor TOML specs:

1. For each TOML spec modified, read the corresponding DTU clone's source:
   - `crates/prism-dtu-<sensor>/src/types.rs` (response struct definitions)
   - `crates/prism-dtu-<sensor>/src/routes/<table>.rs` (route + response shape per table)
2. For EVERY column declared in the TOML `[[tables]]` blocks:
   - Verify the column name matches a field in the DTU types.rs response struct for that table
   - Verify the TOML column type matches the DTU Rust type (String↔String, Integer↔i64/u64, Float↔f64, Boolean↔bool). For datetimes, ALL THREE of these are valid pairings: (a) `Datetime ↔ chrono DateTime`; (b) `Datetime` ↔ an ISO-8601/epoch wire string carrying a **declared** `timestamp_formats` parse chain; (c) `Datetime` ↔ an ISO-8601 wire string with `timestamp_formats` **omitted entirely**, which resolves to the implicit `["iso8601"]` default supplied by `effective_formats` in `prism-spec-engine::pipeline` per ADR-028 §D8-B backward compatibility. JSON has no native datetime type, and the ratified normalization path is a wire string plus a `timestamp_formats` chain (e.g. `crates/prism-sensors/specs/cyberint.sensor.toml` `created_at` declares `["iso8601","unix_epoch_seconds"]` with E-SPEC-018 on total parse failure — this is the LIVE exemplar; `cyberint-alerts.sensor.toml` is its post-migration successor and does not exist yet). Treating chrono as the only valid pairing mints false findings — it would have produced five or more across the Wave-A sensor TOMLs alone. Arm (c) is equally load-bearing: `effective_formats` returns `vec!["iso8601"]` when the declared chain is empty, so an **absent** `timestamp_formats` key is a valid configuration, NOT a parity defect. Treating a declared chain as a precondition mints false CRITICALs against every sensor TOML that relies on the default — as of this amendment `claroty.sensor.toml`, `crowdstrike.sensor.toml`, and `armis.sensor.toml` declare no chains at all, so a literal reading of arms (a)+(b) alone would have produced at least seven. Verify the implicit default at `effective_formats` before filing any datetime-pairing finding.
3. **Column in TOML with no DTU equivalent → P1 CRITICAL** (runtime normalization will silently produce empty/wrong data)
4. **Field in DTU with no TOML column → MEDIUM** (missing coverage, not a runtime crash). A field deliberately excluded from TOML MUST have its exclusion documented in the owning BC; an undocumented exclusion causes this finding class to recur on every subsequent pass.
5. Adversary MUST read the DTU source directly — `crates/prism-dtu-{sensor}/src/types.rs` and `crates/prism-dtu-{sensor}/src/routes/{table}.rs` — and MUST NOT rely on story descriptions of the schema.
6. **Read the emission site, not just the type definition.** The wire-emission site — the `json!` envelope, serializer, or response literal inside the route handler — is AUTHORITATIVE over the struct definition. A field that exists on the response struct but is absent from the emitted envelope resolves to nothing at runtime and is a **P1 CRITICAL**, not a pass. A `# SAP-2 compliance: all columns have matching fields in .../types.rs` comment can be simultaneously true of the struct and false of the wire; such a comment is not evidence. Where a handler has both a static-fixture path and a generated-records path, verify EVERY path — a field present only on the generator path yields path-dependent behavior in which seeded demo scenarios pass while unseeded production runs silently return empty.

Source: PLUGIN-MIGRATION-001-D pass-3 FB-IMPL-3 (4 CRITICAL findings caught by this probe); lesson 24. Rules 2 (datetime pairing) and 6 (emission-site authority) added 2026-07-27 by human approval, from SAP-2 probe findings F-SAP2-OBS-002 (false-finding prevention) and F-SAP2-CRIT-001 (eight IOC columns resolved to nothing on the DTU static-fixture path while the `Alert` struct carried every field).

### SID-1 — Implementer discipline: no-ignored-test rationalization prohibition

When no failing test drives a spec-required behavior because integration tests are `#[ignore]`'d (e.g., DTU/external-service dependency):

1. This is NOT justification to defer the behavior
2. The correct response: add a unit test in the production module's `#[cfg(test)] mod tests` block that drives the behavior WITHOUT the external dependency (mock or stub at the dependency boundary)
3. The unit test must actually exercise the production code path
4. `#[ignore]`'d integration test must include a code comment citing the blocking dependency (e.g., `// DTU-EXT-001: requires DTU clone running; ungated in CI after 001-A deploys`)
5. "Deferred to non-ignored test" is ONLY valid if a SPECIFIC story ID and SPECIFIC test name are cited in the deferral
6. Implementer must self-check this before declaring a Red Gate test pass via a non-#[ignore]'d substitute

Source: PLUGIN-MIGRATION-001-D pass-1 FB-IMPL-1 D-764 orchestrator rejection + remediation cycle adding 7 unit tests; lesson 17.

### SAP-3 — Adversary standing probe: spec-arm reachability

For EVERY adversarial pass on stories or PRs touching prism-query grammar/plan gates or any BC postcondition arm / EC table:

1. For each BC postcondition arm / EC row claimed by the story, verify at least one test reaches the arm **end-to-end from the public surface** (parser input or MCP tool call) — not merely a unit test invoking the internal handler with a synthetic AST
2. Synthetic-AST / direct-handler tests count as defense-in-depth ONLY; an arm with ONLY synthetic coverage = **P2 finding** (the arm may be unreachable from the product surface)
3. Where an arm is intentionally defense-in-depth (unreachable by design), the covering test must carry a comment stating so with the reachability rationale
4. Precedent: BC-2.11.004 arm (4) NonColumnLhsComparison was grammar-unreachable from pipe mode until DEFECT-PQL-FNCALL-LHS-001 — synthetic-AST coverage masked the gap; the live audit ([H5c]) exposed it

Source: live-audit triage 2026-07-13 (D-1715/D-1716); human-approved codification.

### SAP-4 — Adversary standing probe: named-mechanism production-reachability

For EVERY adversarial pass where a BC or story names a specific function as the mechanism implementing an invariant/AC (e.g. "absent field → null cell produced via `ColumnMapper::map_record`"):

1. Grep `crates/**/src` EXCLUDING tests (`--glob '!**/tests/**'`, and disregard hits inside `#[cfg(test)]` blocks) for callers of the named mechanism function, and confirm it has **≥1 production caller**.
2. A named mechanism function with **zero production callers is dead in production**. Any test exercising it is defense-in-depth ONLY and does NOT satisfy reachability for the invariant/AC it claims to cover — that AC requires at least one test that drives the behavior through the ACTUAL production path. Finding severity: **P2 (MED)**, masked-coverage class.
3. This is distinct from SAP-3. SAP-3 distinguishes synthetic-AST / direct-handler tests from public-surface tests; SAP-4 catches the case where a test reaches a **real** function that is nonetheless never invoked on the production path — so the coverage looks legitimate while the production behavior is untested. A green test against a production-dead reference function is the failure mode SAP-3's synthetic-vs-surface framing does not reach.
4. The covering test set for a BC-named mechanism MUST include at least one assertion that reaches the behavior via the production entry point (e.g. `SpecDrivenSensorAdapter::fetch` / `QueryEngine::execute`), not only the named helper in isolation. A documented non-production "reference"/mirror function may be named as such, but MUST NOT be presented (in BC §Invariants, EC prose, or story ACs) as the production mechanism.

Precedent: S-CLAROTY-OT-EVENTS-001 LOCAL adversary re-gate — AC-007 (absent `event_id` → null `finding_info_uid`) and AC-008 (`detection_time` null passthrough) were covered only by tests invoking `ColumnMapper::map_record` directly; `map_record` has zero production callers (production materialization uses `build_column_array` / `pipeline_result_to_record_batch`), so the ACs had synthetic-only coverage against a dead path while the BC §Invariants mis-anchored the mechanism to `map_record`. Mirrored as POL-42 (`named_mechanism_production_reachability`) in `.factory/policies.yaml`.

Source: S-CLAROTY-OT-EVENTS-001 LOCAL adversary re-gate; human-approved codification 2026-08-31.

### SID-2 — Test-writer/implementer discipline: composed-output assertions

When a user/agent-visible string is composed from multiple fields (e.g., `message` + `suggestion`, category prefix + text):

1. At least one test MUST assert on the FULL composed string as emitted, not only its component fields
2. Where composed fields could overlap semantically, include a no-duplicated-phrase assertion (e.g., occurrence count of the shared phrase == 1)
3. Component-only assertions are insufficient to declare the surface covered
4. Precedent: [H8b] doubled "see audit log. See audit log for details." — message and suggestion were each individually asserted; the composition never was

Source: live-audit triage 2026-07-13 (D-1715/D-1716); human-approved codification.

### SAC-1 — Spec-authoring convention: enumerated Red Gate list on `tdd_mode: strict` stories

Every story with `tdd_mode: strict` MUST carry, before it reaches `status: ready`:

1. An **enumerated Red Gate test list** in the `RG-001..RG-NNN` format, with each entry naming the specific failing test to be written
2. A **BC-5.38.001 density check** paragraph stating the Red-Gate-test count and its relation to the acceptance-criteria count
3. **Red-then-green task ordering** — test-authoring tasks MUST precede implementation tasks. Embedding test-writing inline inside implementation tasks inverts the TDD ordering and is a defect, not a style choice
4. Rationale: the test-writer receives an explicit list of named failing tests to author before implementation begins. Without the enumerated list, the red-gate phase is silently skipped or inverted

Precedent: F-WASE-P64-MED-016 — six of seven Wave-A perimeter stories were `tdd_mode: strict` with no enumerated Red Gate list and no density check; only `S-WAVE-A-ENGINE-001` had the correct structure. FB61 applied corrective edits to all six. A structural validator gate is proposed in `S-MAINT-RG-LIST-GATE-001` (draft) so the gap cannot recur.

### SAC-2 — Spec-authoring convention: ADR `anchor_stories` frontmatter

Every ADR MUST carry an `anchor_stories` frontmatter key:

1. The key MUST be **present** — a missing key is a defect, distinct from an empty one
2. It is populated from ground-truth **`§Authority` citations in story files** — a story belongs in an ADR's `anchor_stories` when that story's `§Authority` section cites the ADR. Do NOT populate it from loose prose mentions: a story that names an ADR only to exclude itself from its scope is correctly absent
3. `anchor_stories: []` is legitimate ONLY when accompanied by a **verified-empty annotation**. A bare `[]` that contradicts stories demonstrably citing the ADR is stale and is a defect
4. Rationale: ADR→story traceability must be **bidirectional**. Without this key, stories cite ADRs but ADRs cite no stories, and an ADR change cannot be swept to its dependent stories — the mechanism behind several multi-cite propagation failures in the Wave-A cascade

Precedent: F-WASE-P64-OBS-001 — ADR-053 and ADR-054 carried no `anchor_stories` key at all; three of the four ADRs that had it populated it with a stale `[]`. FB62 populated all of ADR-050..056 from `§Authority` ground truth. A structural validator gate is proposed in `S-MAINT-ADR-ANCHOR-GATE-001` (draft).

Both SAC-1 and SAC-2 codified 2026-07-27 by human approval.

### Conflict with upstream agent prompts

If the upstream vsdd-factory adversary or implementer agent prompt defines a probe / discipline / convention that contradicts SAP-1, SAP-2, SAP-3, SAP-4, SID-1, SID-2, SAC-1, or SAC-2, the project-local rule wins for prism. Upstream canonicalization tracked in `drbothen/vsdd-factory` issue tracker.

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
- **Factory artifacts branch:** `factory-artifacts` (orphan branch mounted at `.factory/` via worktree). Pushed to `origin/factory-artifacts` under the standing authorization granted in D-1066; state-manager pushes it as part of each `.factory/` burst (no per-burst re-authorization needed). The branch is append-only in normal operation; a force-push of `factory-artifacts` still requires explicit human approval.

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
