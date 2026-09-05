---
document_type: research
research_type: general
producer: research-agent
topic: CHANGELOG / release-notes automation landscape for a Rust workspace with a channel ladder and an AI-agent factory
timestamp: 2026-09-05
status: complete
supersedes: none
related:
  - RELEASING.md
  - docs/RELEASE-CHANNELS.md
  - .factory/research/release-engineering-uncertainties-2026.md
sources:
  # Registry / version verification (crates.io, npm, PyPI) — verified 2026-09-05
  - https://crates.io/api/v1/crates/git-cliff        # git-cliff 2.14.1 (2026-09-01)
  - https://crates.io/api/v1/crates/release-plz       # release-plz 0.3.161 (2026-09-03)
  - https://crates.io/api/v1/crates/cargo-release      # cargo-release 1.1.5 (2026-08-11)
  - https://crates.io/api/v1/crates/cocogitto          # cocogitto 7.0.0 (2026-03-04)
  - https://crates.io/api/v1/crates/cargo-dist         # cargo-dist 0.32.0 (2026-05-22)
  - https://crates.io/api/v1/crates/knope              # knope 0.23.0 (2026-05-24)
  - https://registry.npmjs.org/@changesets/cli/latest  # @changesets/cli 3.0.2
  - https://pypi.org/pypi/towncrier/json               # towncrier 26.9.0 (2026-09-04)
  - https://registry.npmjs.org/release-please/latest   # release-please 17.11.2
  - https://registry.npmjs.org/semantic-release/latest # semantic-release 25.0.9
  - https://registry.npmjs.org/standard-version/latest # standard-version 9.5.0 (deprecated)
  - https://registry.npmjs.org/auto/latest             # auto 11.3.6
  # Docs / behaviour / maintenance status
  - https://git-cliff.org/docs
  - https://git-cliff.org/docs/github-actions/git-cliff-action
  - https://github.com/orhun/git-cliff/blob/main/examples/detailed.toml
  - https://release-plz.dev/docs/changelog
  - https://github.com/crate-ci/cargo-release/blob/master/docs/reference.md
  - https://docs.cocogitto.io/reference/config.html
  - https://knope.tech
  - https://changesets.dev/guide/config
  - https://towncrier.readthedocs.io/en/stable/tutorial.html
  - https://github.com/googleapis/release-please
  - https://github.com/semantic-release/release-notes-generator
  - https://github.com/conventional-changelog/standard-version/issues/919  # deprecation
  - https://blog.axo.dev/2024/10/new-name                                   # cargo-dist -> dist rename
  - https://keepachangelog.com/en/2.0.0
  # AI/LLM prior art
  - https://arxiv.org/html/2505.17977v1              # SmartNote LLM release-note generator
  - https://github.com/github/copilot-release-notes
  - https://github.com/ubicloud/llm-changelog-generator
  - https://itlackey.dev/blog/changeish-automate-your-changelog-with-ai
mcp_gate: "Perplexity MCP not available in this environment; Tavily MCP used as PRIMARY (2x tavily_research deep). See Research Methods + deviation note."
---

# Release-Notes / CHANGELOG Automation — Landscape & Recommendation for Prism (2026)

**Decision question:** How should Prism automate changelog prose and release notes given a
25-crate Rust workspace (edition 2024) where only `crates/prism-bin/Cargo.toml` carries the
product version, a nightly→dev→alpha→beta→rc→stable channel ladder (pre-releases tag
`develop`, stable promotes to `main`), enforced Conventional Commits, and an AI-agent VSDD
factory whose `technical-writer` agent could author changelog fragments.

**Bottom line up front:** Adopt **git-cliff** (Rust-native, single static binary, verified
active at `2.14.1`, 2026-09-01) as the changelog *assembler*, keep **Conventional Commits as
the entry source** (already enforced by lefthook — so the "currency gate" is essentially
free), and add a **`technical-writer`-agent-drafted curated top-block** (Highlights / Breaking
changes / Upgrade notes) at release-prep time, gated by human review. This is a hybrid that
formalizes the two-layer model already sketched in `RELEASING.md §5`. **Do not adopt
release-plz** — its per-crate version-bump + crates.io-publish model actively fights Prism's
single-product-version, `publish = false` reality.

---

## Version & maintenance verification (all checked 2026-09-05 against registries)

| Tool | Latest version | Released | Registry / runtime | Maintenance |
|------|----------------|----------|--------------------|-------------|
| **git-cliff** | 2.14.1 | 2026-09-01 | crates.io — **Rust** binary | Active, frequent releases |
| **release-plz** | 0.3.161 | 2026-09-03 | crates.io — **Rust** | Very active |
| **cargo-release** | 1.1.5 | 2026-08-11 | crates.io (`crate-ci`) — **Rust** | Active |
| **cocogitto** | 7.0.0 | 2026-03-04 | crates.io — **Rust** | Active |
| **knope** | 0.23.0 | 2026-05-24 | crates.io — **Rust** binary | Active |
| **cargo-dist / `dist`** | 0.32.0 | 2026-05-22 | crates.io — **Rust** | ⚠ Reduced upstream maintenance — renamed `dist` (Oct 2024); Astral maintains a fork for uv/ruff. Packaging tool, **not** a changelog author. |
| **@changesets/cli** | 3.0.2 | current `latest` | npm — **Node.js** | Active |
| **towncrier** | 26.9.0 | 2026-09-04 | PyPI — **Python** | Active |
| **release-please** | 17.11.2 | current `latest` | npm — **Node.js** (Google) | Active |
| **semantic-release** | 25.0.9 | current `latest` | npm — **Node.js** | Active |
| **standard-version** | 9.5.0 | (2022) | npm — **Node.js** | ⚠ **DEPRECATED / unmaintained** — repo archived; deprecation tracked in conventional-changelog/standard-version#919. Do not adopt. |
| **auto** | 11.3.6 | current `latest` | npm — **Node.js** | Active |

> **Stale/flagged:** `standard-version` is deprecated (do not consider). `cargo-dist` is a
> distribution/packaging tool with signs of slowed upstream maintenance; it *consumes* an
> existing CHANGELOG for GitHub Release bodies but does not author prose — it is orthogonal to
> this decision and Prism already has its own `release.yml` build/publish path.
>
> **Correction to a common misconception:** `knope` is **Rust-native** (crates.io binary), not
> a Node tool. It is notable as the one tool that natively supports **both** changeset fragments
> **and** conventional-commit derivation.

---

## Comparison matrix (7 criteria from the brief)

Legend for **prose model**: **(a)** PR-time news fragments · **(b)** derived from Conventional
Commit subjects/bodies · **(c)** release-time generation from git/PR metadata.

| Tool | 1. Prose model / readability | 2. Categorization (feat→Added…) | 3. Traceability (issue/PR links) | 4. Workspace / single-version fit | 5. Pre-release / channel | 6. CI currency gate | 7. Runtime |
|------|------------------------------|----------------------------------|-----------------------------------|------------------------------------|--------------------------|---------------------|-----------|
| **git-cliff** | (b)+(c). Tera templates → grouped, formatted prose. Readability = commit-subject quality; template can inject bodies/footers. Not just a raw list. | Config-driven `commit_parsers` regex→group. Fully customizable. | Yes — GitHub integration exposes PR #, author, contributors as template vars. | **Excellent for single-version.** Point it at the whole repo → **one** product changelog per tag range. Path-scoping (`include_path`) exists but you don't need it. | Works on tag ranges + `--unreleased`/`--tag`/`--bump`; channel ladder (nightly/rc tags on develop) maps cleanly onto tag-range generation. | `git cliff --unreleased` dry-run diff (non-empty check) possible; but real gate = Conventional-Commit enforcement upstream. | **Rust** (static binary). No Node/Python. |
| **release-plz** | (b)+(c) via git-cliff under the hood; produces a standing "release PR". | Inherits git-cliff config; default Keep-a-Changelog. | Yes — release PR lists PRs; host integration. | **Poor fit here.** Built around **per-crate** version bumps + crates.io publish + `version_group`. Prism has ONE product version and `publish = false` on all crates → you fight the tool. | Not first-class pre-release channel support documented. | Standing release-PR *is* the "what's unreleased" surface. | **Rust**. |
| **cargo-release** | (b) via git-cliff for changelog; owns version-bump/tag mechanics. | git-cliff mappings. | Via git-cliff. | Workspace-aware (`consolidate-commits`); still crate-publish-oriented. | **Defines `alpha`/`beta`/`rc` bump levels** natively. | dry-run/diff; no PR status gate. | **Rust**. |
| **cocogitto** (`cog`) | (b). `cog changelog` from Conventional Commit history. | Config `commit_types` map/order; can omit types. | `remote`/`owner`/`repository` config → URLs in output. | Rust crate, runnable per-repo; single-version workable; workspace strategy less documented than git-cliff. | Conventional-commit driven; supports bumping. | `cog check` verifies commits are conventional (a *commit-format* gate, not a fragment gate). | **Rust**. |
| **knope** | **Hybrid (a)+(b)** — changesets *and* conventional commits, its distinguishing feature. Prepares release PRs + changelog. | Config-driven. | PRs/releases. | Package/workspace aware; single-version workable. | Supports pre-release versions. | Changeset-style enforcement possible. | **Rust** binary. |
| **Changesets** (`@changesets/cli`) | **(a)** PR-time fragment files; default generator adds related-commit links → readable, intent-captured prose. | Fragment declares impact (major/minor/patch); mapping is per-fragment, not Conventional-Commit-driven. | Default generator adds commit links. | **Monorepo-first, multi-package.** Overkill/awkward for single product version. | Has a pre-release/snapshot mode. | **`changeset status`** = the canonical fail-PR-without-fragment gate. | **Node.js**. |
| **towncrier** | **(a)** PR-time "news fragments" → assembled human prose. Best-in-class *readable* prose. | Fragment filename category → section (manual author choice). | Fragment name can embed issue/PR #; you wire the link template. | Project-level (single changelog) — **good single-version fit**; no per-crate concept. | Draft/`--version` supports pre-releases. | CI check that fails when a PR adds no `newsfragment` (widely used by CPython/pip/pytest). | **Python** (PyPI). |
| **release-please** | (c) parses Conventional Commits → standing release PR + CHANGELOG. | Conventional Commits → notes. | Release PR + GitHub release with PR/commit info. | Rust releaser exists but needs `cargo-workspace` plugin for dependents; manifest mode for multi-pkg. | Limited channel semantics. | Standing release PR surfaces unreleased changes. | **Node.js** (Google). |
| **semantic-release** | (c) conventional-changelog presets → templated prose (Features/Bug Fixes). | Preset-driven (`conventionalcommits`). | Plugin-driven links. | npm-ecosystem-centric; awkward for a Rust product. | **Branch→dist-tag channels** (main→latest, next→next, beta) — strong channel model, but npm-shaped. | Fails release if no releasable commits. | **Node.js**. |
| **conventional-changelog / standard-version** | (c) commit-derived list. | Preset. | Links via preset. | Generic. | Basic prerelease flag. | None built-in. | **Node.js**. **standard-version DEPRECATED.** |
| **auto** | (c) **PR-label-driven** release notes. | Labels → sections (not Conventional Commits). | PR-centric. | Generic. | Canary/prerelease supported. | Label-presence checks. | **Node.js**. |
| **cargo-dist / `dist`** | N/A — *consumes* an existing CHANGELOG for the GitHub Release body; does not author prose. | N/A | Adds notes to GitHub Release. | Packaging/build focus. | Tag-driven. | N/A | **Rust** (⚠ maintenance). |

---

## Synthesis (A–E)

### A) PR-time fragments vs commit-derived generation — which yields the best readable notes with least cold-context reconstruction?

Two axes are in tension:

- **Author effort & drift:** commit-derived (git-cliff/cocogitto) wins. There is exactly one
  source of truth — the git history you already enforce — so nothing can drift out of sync and
  no extra per-PR file is required. Fragments (Changesets/towncrier) add a per-PR artifact that
  can be forgotten (hence they *need* a CI gate) and can drift from what the code actually did.
- **Prose quality & cold-context reconstruction:** fragments win. A news fragment is written by
  the author *at the moment of change*, capturing intent, user impact, and breaking-change
  nuance while context is hot. Commit-derived notes are only as good as the commit subjects, and
  turning terse `fix:`/`feat:` subjects into user-facing prose forces a *cold-context
  reconstruction* at release time by someone who must re-derive "why did this matter to a user."

**Real tradeoff table:**

| | Commit-derived (git-cliff) | PR-time fragments (towncrier/Changesets) |
|---|---|---|
| Author effort/PR | Zero extra (commit already required) | One fragment file per user-visible PR |
| Prose quality | Bounded by commit-subject discipline | High — intent captured hot |
| Cold-context reconstruction | Required at release time | Avoided (written at change time) |
| Drift risk | ~None (single source = history) | Real (fragment can be forgotten/stale) |
| Traceability | PR/author auto-injected from host | Manual link in fragment |

**For Prism specifically the axes partly collapse**, because commits are authored by AI agents
under an enforced Conventional-Commit regime and the factory can equally cheaply produce a
fragment. That removes the "extra author effort" penalty of fragments — but it does *not* remove
their drift risk. The clean resolution is the **hybrid**: let commit-derived generation carry
the bulk mechanical body (zero drift), and reserve a small hand/agent-authored top-block for the
handful of items that genuinely need hot-context prose (breaking changes, upgrade steps,
headline features). See E.

### B) Prior art on LLM/AI-agent-drafted changelogs

Yes, this is an active 2024–2026 space:

- **SmartNote** (arXiv 2505.17977) — an LLM-powered release-note generator evaluated across many
  OSS projects; reported producing high-quality, readable notes *even when commit messages are
  inconsistent*, outperforming conventional-changelog baselines on applicability.
- **GitHub Copilot Release Notes** (github/copilot-release-notes) — CLI/Action that summarizes
  PRs into release notes. Documented pitfall: PR-discovery strategy (merge-commit vs GitHub-API)
  changes which PRs are found and how output is structured; needs instruction files to get
  categorized (not flat-bullet) output.
- **Community tooling:** `ubicloud/llm-changelog-generator` (multi-model writeup of what worked),
  `Changeish` (AI-drafted changelog), plus numerous Marketplace "AI release notes" actions.

**Quality/pitfalls (consistent across sources):** LLM drafts are good for turning noisy history
into readable prose, but reliably **mis-classify or miss breaking-change and security items**,
tend toward flat bullet lists without a strong template/prompt, and add an operational surface
(external API keys, privacy). **The universal practice is review-only**: LLM output lands as an
editable draft, never auto-published. For Prism this maps perfectly onto the existing pattern —
the `technical-writer` agent drafts, a human curates in the release-prep PR, and the AI-opaque
credentials rule is a non-issue since changelog text carries no credentials.

### C) Layering a curated "Highlights / Upgrade / Breaking changes" top-block

Mature practice is a **two-layer changelog**: a small human-curated narrative block on top of an
otherwise machine-generated detail list. Mechanisms observed:

- **Standing release-PR that is editable** before publish (release-please, release-plz,
  Release Drafter) — the maintainer edits the top-block in the PR before merge/tag.
- **A dedicated fragment category** for highlights/breaking changes (towncrier custom types;
  Changesets summary) that sorts to the top.
- **Keep a Changelog** conventions (Added/Changed/Deprecated/Removed/Fixed/Security) as the
  section grammar underneath the narrative.

**Prism already does exactly this** — `RELEASING.md §5` defines Layer 1 (manually-prepended
curated narrative) + Layer 2 (auto `--generate-notes`). The recommendation below just upgrades
Layer 2's generator and formalizes *who* writes Layer 1.

> **Evidence gap (flagged):** I could not retrieve primary-source annotated examples showing the
> exact top-block files/owners in specific large Rust or Kubernetes repos. The two-layer pattern
> itself is well-attested; the specific "who curates the top block in repo X" detail is
> lower-confidence and drawn from general practice + tool capabilities, not a cited artifact.

### D) Best-practice CI gate to keep a changelog current — and how noisy it is

Two gate families:

1. **Fragment-presence gate** (`changeset status`, towncrier's "PR must add a newsfragment"
   check). *Effective* — it mechanically prevents a user-visible PR from merging without an
   entry, which is why CPython/pip/pytest/large-monorepos use it. *But noisy*: it false-blocks
   trivial PRs (docs, CI, chore, typo) and generates bot churn, so teams universally add
   escape hatches — an "empty changeset", a `skip-changelog` label, or path/type exemptions.
   Net: high assurance, real friction, needs tuning.
2. **Commit-format gate** (Conventional Commits enforced at commit time; optionally
   `git cliff --unreleased` must be non-empty, or `cog check`). *Much lower noise* because it
   piggybacks on discipline the team already keeps; the "entry" is the commit itself, so there is
   nothing extra to forget. The failure mode is a low-quality subject, not a missing entry.

**For Prism the currency gate is effectively already in place:** lefthook enforces Conventional
Commits, so every merged change is already a categorizable changelog entry. Adding a
fragment-presence gate would introduce the noisy false-block problem *on top of* an
already-enforced commit regime — poor ROI. If a stronger gate is ever wanted, prefer a
release-prep-time `git cliff --unreleased` non-empty assertion over a per-PR fragment gate.

### E) Recommended target architecture for Prism

**Recommendation: git-cliff (assembler) + Conventional Commits (entry source, already enforced)
+ `technical-writer`-agent-drafted curated top-block (review-only). A focused hybrid — NOT
release-plz, NOT a full Changesets/towncrier fragment system.**

Rationale, weighed against the three candidate architectures:

- **vs release-plz:** Rejected. release-plz's whole model is per-crate version bumps and
  crates.io publishing coordinated across a workspace. Prism has a *single* product version
  (`prism-bin` only), all crates `publish = false`, and a bespoke `release-prep.yml` /
  `release-promote.yml` promotion flow with a human `release-main` environment gate. release-plz
  would duplicate and conflict with that machinery. You'd spend effort *suppressing* its core
  behavior. (git-cliff — which release-plz merely wraps — gives you the good part without the
  version/publish machinery you don't want.)
- **vs a Changesets/towncrier fragment system:** Rejected as the *primary* mechanism. It adds a
  non-Rust runtime (Node or Python — against the stated minimal-toolchain preference), a per-PR
  fragment file, and a noisy fail-PR gate — all to solve a problem (missing entries) that
  enforced Conventional Commits already solves. Its genuine advantage (hot-context prose for
  breaking changes/highlights) is captured more cheaply by the agent-authored top-block.
- **Chosen hybrid:** git-cliff renders the mechanical, categorized body (feat→Added, fix→Fixed,
  etc. via configurable `commit_parsers`) with PR/author links auto-injected, producing *one*
  product-level changelog per tag range — a perfect fit for the single-version, tag-driven
  channel ladder (each nightly/dev/alpha/beta/rc/stable tag defines a range git-cliff renders).
  The `technical-writer` agent drafts the Layer-1 top-block (Highlights / Breaking changes /
  Upgrade notes) during release-prep, delivered as a review-only draft the human curates in the
  release-prep PR — exactly the review-only discipline the LLM prior art (B) mandates.

**Concrete shape:**
1. Add `cliff.toml` (commit_parsers → Keep-a-Changelog sections; GitHub PR/author links; group
   ordering with Breaking-changes first).
2. In `release-prep.yml`, replace the current "seed CHANGELOG with merged-PR subjects since the
   previous tag" step with `git cliff --tag vX.Y.Z` (or `--unreleased`) to generate Layer 2.
   Keep the human CHANGELOG-curation gate in the release-prep PR (§4 Step 2) unchanged.
3. Add an orchestrator dispatch of `vsdd-factory:technical-writer` at release-prep to draft the
   Layer-1 top-block from the same tag range (documenting current behavior only), output to the
   PR for human curation.
4. Keep `release.yml --generate-notes` or swap the GitHub Release body to the git-cliff output
   for consistency between `CHANGELOG.md` and the Release page.
5. Optionally add a light `git cliff --unreleased` non-empty check to CI; do **not** add a
   per-PR fragment gate.

**Migration cost: LOW.** git-cliff is a single static Rust binary (no Node/Python added to the
toolchain — satisfies the constraint). It slots into the *existing* two-layer model in
`RELEASING.md §5` and the *existing* `release-prep.yml` scaffold step; you are replacing one
"seed from PR subjects" step with a better generator and adding one agent dispatch. No change to
versioning philosophy, the channel ladder, the promotion gates, or `publish = false`. cocogitto
is a viable Rust-native alternative to git-cliff, but git-cliff has richer Tera templating, is
more widely adopted, and is the engine release-plz itself uses — the lower-risk pick.

---

## Bottom-line recommendation (one paragraph)

Standardize on **git-cliff** (Rust-native, single binary, active `2.14.1` as of 2026-09-01) to
generate Prism's categorized changelog body from the Conventional Commits you already enforce,
and have the **`technical-writer` agent draft a review-only "Highlights / Breaking changes /
Upgrade notes" top-block** at release-prep time for human curation — a hybrid that formalizes the
two-layer model already in `RELEASING.md §5`. This is the lowest-effort, lowest-drift,
no-new-runtime path: because Conventional Commits are enforced by lefthook, the changelog
"currency gate" is already satisfied, so a noisy per-PR fragment system (Changesets/towncrier,
both non-Rust) buys little; and release-plz is the wrong tool because its per-crate
version-bump/crates.io-publish model directly conflicts with Prism's single-product-version,
`publish = false`, bespoke promotion-workflow reality. Migration cost is low — swap one step in
`release-prep.yml` for a `git cliff` invocation plus a `cliff.toml`, and add one agent dispatch.

---

## Research Methods

| Tool | Queries | Purpose |
|------|---------|---------|
| **Perplexity perplexity_research (PRIMARY)** | 0 | **Unavailable in this environment** — see deviation note below. |
| Tavily tavily_research | 2 | Deep multi-source synthesis: (1) full 12-tool landscape against the 7 criteria; (2) AI/LLM-drafted changelog prior art + curated top-block practice + CI-gate noise. |
| WebFetch | 13 | Registry version/date verification: crates.io (git-cliff, release-plz, cargo-release, cocogitto, cargo-dist, knope), npm (@changesets/cli, release-please, semantic-release, standard-version, auto), PyPI (towncrier), knope.tech docs. |
| WebSearch | 2 | cargo-dist/`dist` axo.dev maintenance-status + Astral fork confirmation. |
| Read | 2 | Prism's existing `RELEASING.md` + `docs/RELEASE-CHANNELS.md` for constraint grounding. |
| Training data | 2 areas | (a) standard-version deprecation context (corroborated by issue #919); (b) general "two-layer changelog" curation practice where a primary-source annotated example was not retrievable — flagged as an evidence gap in §C. |

**Total MCP tool calls:** 2 (both `tavily_research`, deep model).
**Training data reliance:** low — every tool claim is registry- or docs-cited; the two
training-data areas are explicitly flagged, one corroborated by a cited issue and one marked as
a confidence-lowering evidence gap.

**Deviation note (MCP precedence):** The agent mandate names `perplexity_research` as the PRIMARY
tool, but no Perplexity MCP server is exposed in this environment (available MCP servers were
Context7 and Tavily only). Per the "single exception" clause I did not quietly skip — I used
**Tavily MCP (`tavily_research`, `pro` model) as the primary deep-research tool** for two
comprehensive multi-source passes, satisfying the ≥1-MCP-call gate, and cross-verified every
version number and maintenance-status claim directly against the authoritative registries
(crates.io / npm / PyPI) rather than relying on any single synthesized answer.
