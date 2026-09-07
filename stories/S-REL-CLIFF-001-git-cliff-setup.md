---
document_type: story
story_id: S-REL-CLIFF-001
title: "devops: git-cliff setup — cliff.toml at repo root + release-prep.yml Step 7 replacement (git log scaffold → git cliff --unreleased --prepend)"
wave: F-A
epic_id: E-REL-NOTES
priority: P0
status: ready
version: "1.7"
level: "L4"
producer: story-writer
timestamp: "2026-09-05T00:00:00Z"
tdd_mode: facade
# tdd_mode: facade — this story creates config/workflow files (cliff.toml TOML,
# release-prep.yml YAML) with no Rust production code. The deliverable is a working
# cliff.toml and a patched workflow step. Verification is via dry-run execution
# (git cliff --unreleased --output /dev/stdout on develop history), not TDD.
subsystems: [SS-22]
# Subsystem anchor justification:
#   SS-22 (Process Lifecycle) owns release toolchain infrastructure per ARCH-INDEX.
#   cliff.toml and the release-prep.yml change are release-engineering artifacts
#   within SS-22's scope.
crates_touched: []
target_module: devops
capabilities: []
behavioral_contracts: []
# BC status: N/A — changelog tooling setup; no subsystem behavioral contract governs
# CHANGELOG generation or CI workflow configuration.
verification_properties: []
depends_on: []
blocks: [S-REL-WRITER-001, S-REL-BETA1-NOTES-001, S-REL-VBUMP-001]
# Dependency anchor justifications:
#   blocks S-REL-WRITER-001: technical-writer dispatch step runs AFTER git-cliff
#     invocation in release-prep.yml; CLIFF-001 must establish the base cliff step.
#   blocks S-REL-BETA1-NOTES-001: first-release CHANGELOG requires cliff.toml to be
#     in place for the `git cliff --tag v1.0.0-beta.1` dry-run.
#   blocks S-REL-VBUMP-001: cargo-release pre-release-hook invokes git-cliff with
#     `--unreleased --tag ... --prepend`; cliff.toml must exist for the hook to work.
points: 5
estimated_days: 1
risk: LOW
acceptance_criteria_count: 6
red_gate_tests: 0
# red_gate_tests: 0 — facade mode. No Rust production code; no Red Gate.
# Quality gate: dry-run `git cliff --unreleased --output /dev/stdout` on develop
# history produces non-empty, correct-categorization output.
estimated_passes: "1-2 LOCAL adversary passes"
holdout_scenarios: []
# HOLDOUT-N/A: tdd_mode=facade; crates_touched=[]; no built prism binary; no MCP-visible
# surface. Verification is via git-cliff dry-run CLI execution that the implementer
# explicitly performs: AC-002 (skip-rules output check), AC-006 (breaking-changes
# ordering), and Tasks 3+6 both require running the dry-run and documenting the output
# snippet in the PR description. git-cliff is an external CLI tool — not the prism MCP
# binary. No info-asymmetry surface exists. Gate definitionally inapplicable per CLAUDE.md
# story-level holdout gate definition (requires built prism binary + MCP stdio wire
# assertions, scoped to the story's touched surface).
assumption_validations: []
risk_mitigations:
  - "git-cliff 2.14.1 must be pinned exactly. Future upgrades require bumping the
    pin in release-prep.yml and noting the version in the next ADR version per ADR-063 D1."
  - "PR link injection is token-free (ADR-063 v1.7 D3 Deviation-1). The
    `commit_preprocessors` regex rewrites `(#NNN)` in squash-merge commit subjects into
    markdown links BEFORE Tera rendering — no GITHUB_TOKEN or remote API call needed.
    The body template renders `{{ commit.message }}` (which already contains the link)
    and reads NO `commit.remote.*` fields (those require GITHUB_TOKEN and are empty in
    token-free dry-runs). The `[remote.github]` section is retained for possible future
    remote-API use but the body template ignores it."
  - "cliff.toml `[changelog] header` MUST NOT be empty (`header = \"\"`). ADR-063 v1.8 D3
    confirmed empirically (3 simulated release cycles on REAL CHANGELOG.md): `git cliff
    --prepend` with `header = \"\"` inserts the new `## [version]` section ABOVE the
    `# Changelog` masthead, burying it and orphaning `## [Unreleased]`. The `header` MUST
    carry the full masthead + keep-a-changelog preamble + `## [Unreleased]` line (use the
    exact multi-line block from the ADR-063 v1.8 D3 cliff.toml sketch). The 3-substep
    Step 7 in release-prep.yml — (a) PRE-STRIP, (b) git cliff, (c) LINK-REF UPDATE —
    handles idempotency across release cycles (ADR-063 v1.8 D3)."
  - "The git cliff invocation in release-prep.yml uses `--unreleased --tag` (NOT
    `--latest`). `--unreleased` renders all commits not yet in any tag; combined with
    `--tag` it writes the new section header. Do NOT add -o/--output alongside
    `--prepend` — that caused duplicate sections per ADR-063 D5 v1.1 fix. Per
    ADR-063 v1.2 D5 the canonical invocation is:
    `git cliff --tag \"${VERSION_TAG}\" --unreleased --prepend CHANGELOG.md`."
  - "cliff.toml skip rules suppress docs/ci/test/chore/style/build/revert commits.
    If the develop dry-run output is still noisy, additional skip rules may be added
    before the beta.1 tag (ADR-063 D6 noise-control gate)."
  - "Breaking Changes section must appear BEFORE Added in the output. The authoritative
    mechanism per ADR-063 v1.7 D3 is a Tera filter-based two-part body: (1) a manual
    `### Breaking Changes` section rendered FIRST via `commits | filter(attribute=\"breaking\",
    value=true)`; (2) a `commits | filter(attribute=\"breaking\", value=false) |
    group_by(attribute=\"group\")` loop for non-breaking commits. `{ breaking = true }`
    parser-level grouping is NOT effective in git-cliff 2.14.1 — a `feat!:` commit still
    lands in its message-derived group regardless of parser position — and MUST NOT be
    relied upon; no `{ breaking = true }` entry appears in cliff.toml commit_parsers.
    Non-breaking groups use the numbered-group-name-prefix idiom (`<!-- 1 -->Added` through
    `<!-- 5 -->Security`); the Tera body strips them via `{{ group | striptags | trim }}`
    (native git-cliff 2.14.1 mechanism; regex_replace is NOT a built-in Tera filter —
    Filter 'regex_replace' not found confirmed via dry-run). `[git] group_order` is NOT
    a valid key in git-cliff 2.14.1 and MUST NOT be added."
inputs:
  - ".github/workflows/release-prep.yml"
  - ".factory/specs/architecture/decisions/ADR-063-changelog-release-notes-architecture.md"
input-hash: "[pending-recompute]"
traces_to: []
cycle: "v1.0.0-beta.1-release-identity"
phase: "3"
---

# S-REL-CLIFF-001 — git-cliff Setup: cliff.toml + release-prep.yml Step 7 Replacement

**Story ID:** S-REL-CLIFF-001
**Status:** ready
**Version:** v1.7
**Wave:** F-A
**Priority:** P0
**Points:** 5
**Beta.1-blocking:** YES — cliff.toml must be in place before S-REL-BETA1-NOTES-001 and
S-REL-WRITER-001 can execute.

---

## Origin

`release-prep.yml` Step 7 currently generates an uncategorized commit-seed list via
`git log --merges --oneline | head -100`. The human must manually categorize entries
into Added / Fixed / Changed / Security / Removed. ADR-063 v1.2 D5 replaces this step
with `git cliff --tag "${VERSION_TAG}" --unreleased --prepend CHANGELOG.md`.

ADR-063 D1 mandates `git-cliff 2.14.1` (pinned). ADR-063 D3 defines the `cliff.toml`
categorization convention. ADR-063 v1.2 D5 establishes the canonical release-prep
invocation: `git cliff --tag "${VERSION_TAG}" --unreleased --prepend CHANGELOG.md`.
This story creates the `cliff.toml` at the repo root and patches `release-prep.yml`.

---

## Narrative

As a release engineer, I want git-cliff to automatically generate categorized
CHANGELOG entries from Conventional Commits history, so that the manual categorization
step in release-prep is eliminated and every release section has consistent
Added/Fixed/Performance/Changed/Security sections with PR number links.

---

## Authority

- ADR-063 D1 — git-cliff 2.14.1 as canonical changelog assembler
- ADR-063 D3 — cliff.toml categorization convention
- ADR-063 v1.2 D5 — release-prep.yml integration point; canonical invocation flag set

(No BC: changelog tooling; no subsystem behavioral contract.)

---

## Behavioral Contracts

This story has no subsystem behavioral contracts. Authority is ADR-063 D1/D3/D5.

| Architecture Source | Clause |
|---------------------|--------|
| ADR-063 D1 | `git-cliff 2.14.1` pinned; install via `cargo install git-cliff --version 2.14.1 --locked` |
| ADR-063 D3 | Commit type → CHANGELOG section mapping (feat→Added; fix→Fixed; perf→Performance; refactor→Changed; security→Security; docs/ci/test/chore/style/build/revert→skip) |
| ADR-063 v1.7 D3 | Breaking Changes section ordered BEFORE Added via Tera filter-based two-part body: `commits \| filter(attribute="breaking", value=true)` FIRST; `{ breaking = true }` parser-level grouping NOT effective in git-cliff 2.14.1 and MUST NOT be relied upon |
| ADR-063 v1.7 D3 | GitHub PR link injection token-free via `commit_preprocessors` regex — `(#NNN)` rewritten to markdown link BEFORE Tera rendering; no GITHUB_TOKEN needed; body template reads `{{ commit.message }}` only; `[remote.github]` retained but body template ignores it |
| ADR-063 v1.9 D3 | cliff.toml `[changelog] header` MUST carry full masthead + keep-a-changelog preamble + `## [Unreleased]` (NOT empty — `header = ""` corrupts CHANGELOG.md: new section inserted ABOVE masthead); body template MUST be `{% if version %}...{% endif %}` with NO `{% else %}` arm — `## [Unreleased]` comes solely from `[changelog] header`, never from the body template |
| ADR-063 v1.8 D3 | Step 7 is a 3-substep flow: (a) PRE-STRIP (Python, before git cliff) — strip prior masthead + `## [Unreleased]` from CHANGELOG.md; (b) `git cliff --tag "${VERSION_TAG}" --unreleased --prepend CHANGELOG.md` (core, unchanged); (c) LINK-REF UPDATE (Python, token-free, after git cliff) — repoint `[Unreleased]` ref + insert `[VERSION]` compare-link ref |
| ADR-063 v1.2 D5 | Do NOT add `--output CHANGELOG.md` alongside `--prepend` — dual flag caused duplicate sections |
| ADR-063 D4 | Technical-writer step runs AFTER git-cliff (S-REL-WRITER-001 is the separate follow-on story) |

---

## Token Budget Estimate

| Artifact | Estimated Tokens |
|----------|-----------------|
| This story spec | ~3,000 |
| `.github/workflows/release-prep.yml` (full) | ~5,000 |
| ADR-063 D1/D3/D5 sections | ~3,000 |
| cliff.toml sketch from ADR-063 D3 | ~1,000 |
| Total | ~12,000 |

Well within the 30% context window budget.

---

## Red Gate Test List (SAC-1)

**tdd_mode: facade — no Rust Red Gate tests.** Quality gate is a dry-run execution:
`git cliff --unreleased --output /dev/stdout` on develop history must produce
non-empty output with correct CHANGELOG section headers.

The implementer MUST run this dry-run and document the output snippet in the PR
description before the PR can be reviewed.

---

## Tasks

1. **Read `.github/workflows/release-prep.yml` in full** to understand Step 7 structure,
   surrounding steps (version bump, CHANGELOG insert, commit), and environment variable
   usage.

2. **Create `cliff.toml` at the repo root** with the normative ADR-063 D3 configuration.
   Use the informative sketch from ADR-063 D3 as the basis, ensuring:
   - `[changelog]` section with `header` carrying the full `# Changelog` masthead +
     keep-a-changelog preamble + `## [Unreleased]` line (ADR-063 v1.8 D3: `header = ""`
     is INCORRECT and MUST NOT be used — `git cliff --prepend` with an empty header
     inserts the new `## [version]` section ABOVE the masthead, burying it and orphaning
     `## [Unreleased]`; the v1.7 `header = ""` guidance is superseded by v1.8). The body
     template MUST be `{% if version %}...{% endif %}` with NO `{% else %}` arm —
     `## [Unreleased]` comes solely from `[changelog] header`, never from the body
     template; an `{% else %}## [Unreleased]` arm produces a duplicate heading on
     no-`--tag` dry-runs (ADR-063 v1.9 D3). Empty `footer` and `trim = true` are
     unchanged. Use the exact multi-line `header` and body template from the ADR-063
     v1.9 D3 cliff.toml sketch.
   - `[git]` section: `conventional_commits = true`, `filter_unconventional = true`,
     `tag_pattern = "v[0-9].*"`, `sort_commits = "oldest"`
   - `commit_preprocessors` entry for token-free PR link injection (ADR-063 v1.7 D3
     Deviation-1): `{ pattern = "\\(#([0-9]+)\\)", replace = "([#${1}](https://github.com/drbothen/prism/pull/${1}))" }`
     rewrites `(#NNN)` in squash-merge commit subjects to markdown links BEFORE Tera
     rendering — no GITHUB_TOKEN or remote API call needed. The body template renders
     `{{ commit.message }}` (which already contains the link). Do NOT add
     `commit.remote.pr_number` or `commit.remote.pr_url` to the body template; those
     fields require GITHUB_TOKEN and are empty in token-free dry-runs.
   - `commit_parsers` table using numbered-group-name-prefix idiom for non-breaking groups:
     feat→`<!-- 1 -->Added`, fix→`<!-- 2 -->Fixed`, perf→`<!-- 3 -->Performance`,
     refactor→`<!-- 4 -->Changed`, security→`<!-- 5 -->Security`,
     docs/ci/test/chore/style/build/revert each with `skip = true`. No
     `{ breaking = true }` entry — breaking commits are handled entirely by the Tera body
     filter (ADR-063 v1.7 D3 Deviation-2), not parser-level grouping.
   - `protect_breaking_commits = true`
   - `[remote.github]` section: `owner = "drbothen"`, `repo = "prism"` (retained for
     possible future remote-API use; the body template ignores it — `commit.remote.*`
     fields are NOT read)
   - Breaking Changes section must appear BEFORE Added in the output — use the Tera
     filter-based two-part body per ADR-063 v1.7 D3: (1) a manual `### Breaking Changes`
     section rendered FIRST via `commits | filter(attribute="breaking", value=true)`;
     (2) a `commits | filter(attribute="breaking", value=false) | group_by(attribute="group")`
     loop for non-breaking commits with `### {{ group | striptags | trim }}` headers.
     `{ breaking = true }` parser-level grouping is NOT effective in git-cliff 2.14.1 —
     a `feat!:` commit still lands in its message-derived group (`<!-- 1 -->Added`)
     regardless of parser position — and MUST NOT be relied upon. `[git] group_order`
     does NOT exist in git-cliff 2.14.1 and MUST NOT be added. Verify section ordering
     in dry-run output.

3. **Run dry-run on develop history:**
   ```bash
   git cliff --unreleased --output /dev/stdout
   ```
   Inspect output. If output is excessively noisy (many non-skip entries from
   chore/ci/docs), add additional skip rules per ADR-063 D6. Document the output
   snippet for the PR description.

4. **Patch `release-prep.yml` Step 7** with the canonical 3-substep flow (ADR-063 v1.8 D3):
   Replace the existing `Scaffold CHANGELOG entry` step (uses `git log --merges
   --oneline | head -100` + Python scaffold heredoc) with a 3-substep
   `Generate CHANGELOG body (git-cliff)` step. All three substeps are REQUIRED for
   idempotent, keepachangelog-compliant CHANGELOG.md production:

   **Substep (a) — PRE-STRIP (Python, runs BEFORE `git cliff`):** Strip the `# Changelog`
   masthead and any prior `## [Unreleased]` section from CHANGELOG.md, leaving the file
   starting at the first `## [X.Y.Z]` versioned section. Without this, each release cycle
   accumulates a duplicate masthead from the previous run's `header` injection
   (ADR-063 v1.8 D3 — idempotency gate).

   **Substep (b) — `git cliff` invocation (core, UNCHANGED from D5):**
   ```bash
   git cliff --tag "${VERSION_TAG}" --unreleased --prepend CHANGELOG.md
   ```
   Because `[changelog] header` carries the full masthead + preamble + `## [Unreleased]`,
   git-cliff writes `header + new-section + existing-content`. After this substep the file
   starts with the masthead, followed by the new `## [VERSION]` section, then existing
   history — correct keepachangelog order.
   Do NOT use `--latest` (unreliable for first release or pre-tag disambiguation).
   Do NOT add `-o/--output` alongside `--prepend` (dual flag causes duplicate sections,
   ADR-063 D5 v1.1). The `VERSION_TAG` env var is already set in the workflow context
   (verify the exact var name from the surrounding steps).

   **Substep (c) — LINK-REF UPDATE (Python, token-free, runs AFTER `git cliff`):** Repoint
   `[Unreleased]: .../compare/vPREV...HEAD` to the new version and insert a new
   `[VERSION_TAG]: .../compare/vPREV...vVERSION` compare-link reference definition using
   the existing ref text and the `VERSION_TAG` env var. No `GITHUB_TOKEN` required.
   Maintains the keepachangelog compare-link footer block (ADR-063 v1.8 D3).

5. **Verify the replacement step in context:** Read the surrounding steps to confirm:
   - The `checkout` step has sufficient depth (`fetch-depth: 0` or full history)
     for `git cliff` to walk the tag range. If only `fetch-depth: 1` (shallow),
     add or update the checkout step to `fetch-depth: 0`.
   - The commit step downstream still works (it stages `CHANGELOG.md`; this is
     unchanged because the file is still modified by the git-cliff step).
   - The PR-description in `release-prep.yml` checklist is updated to reference
     git-cliff instead of "categorize the seed list."

6. **Run dry-run with a tag argument (simulation):**
   ```bash
   git cliff --tag v1.0.0-beta.1 --unreleased --output /dev/stdout
   ```
   Confirm output shows a `## [1.0.0-beta.1]` section header and categorized commits.
   Do NOT use `--latest` here — `--unreleased` is the correct flag per ADR-063 v1.2 D5.

7. **Verify AC-001..AC-006.** Document dry-run output snippet for PR description.

---

## Acceptance Criteria

### AC-001: cliff.toml exists at repo root with required sections
`ls cliff.toml` exits 0. File contains `[changelog]`, `[git]`, `[remote.github]`,
and `commit_parsers` with at least six entries (feat, fix, perf, refactor, security,
docs-skip). (traces to ADR-063 D3 — cliff.toml categorization convention)

### AC-002: cliff.toml skips docs/ci/test/chore/style/build/revert
Dry-run output (`git cliff --unreleased --output /dev/stdout`) contains no entries
under these types. feat/fix/perf entries appear under their respective sections.
(traces to ADR-063 D3 — skip rules)

### AC-003: release-prep.yml Step 7 replaced with 3-substep git-cliff flow
The canonical ADR-063 v1.8 D3 3-substep Step 7 is present in `release-prep.yml`:
- `grep -n 'git cliff' .github/workflows/release-prep.yml` returns at least one match
  (Substep b present).
- `grep -n 'git log --merges' .github/workflows/release-prep.yml` returns no match
  (scaffold removed).
- Step 7 includes a PRE-STRIP substep (Python) that strips the prior masthead +
  `## [Unreleased]` from CHANGELOG.md before `git cliff` runs (Substep a — idempotency).
- Step 7 includes a LINK-REF UPDATE substep (Python, token-free) that repoints the
  `[Unreleased]` compare ref and inserts a `[VERSION]` compare-link reference definition
  after `git cliff` runs (Substep c — keepachangelog compare-link footer).
(traces to ADR-063 v1.8 D3 — canonical 3-substep `--prepend` mechanism; and ADR-063 D5 —
release-prep.yml integration point)

### AC-004: git-cliff version 2.14.1 pinned in release-prep.yml
`grep 'git-cliff --version' .github/workflows/release-prep.yml` shows `--version 2.14.1`.
(traces to ADR-063 D1 — version pin rationale)

### AC-005: --prepend used without --output in git-cliff invocation
`grep 'git cliff' .github/workflows/release-prep.yml` shows `--prepend CHANGELOG.md`
and does NOT show `-o CHANGELOG.md` or `--output CHANGELOG.md`.
(traces to ADR-063 D5 v1.1 fix — dual-flag caused duplicate sections)

### AC-006: Breaking Changes section precedes Added in cliff.toml output
Dry-run on a commit history that includes BREAKING CHANGE footers shows the
`### Breaking Changes` section before the `### Added` section in the generated
CHANGELOG body. The authoritative mechanism (ADR-063 v1.7 D3 Deviation-2) is a
Tera filter-based two-part body: `commits | filter(attribute="breaking", value=true)`
renders a manual `### Breaking Changes` section FIRST; then
`commits | filter(attribute="breaking", value=false) | group_by(attribute="group")`
renders non-breaking groups with `### {{ group | striptags | trim }}` headers.
`{ breaking = true }` parser-level grouping is NOT effective in git-cliff 2.14.1
(a `feat!:` commit still lands in `<!-- 1 -->Added` regardless of parser position);
no `{ breaking = true }` entry appears in cliff.toml `commit_parsers`. Non-breaking
groups use the numbered-prefix idiom (`<!-- 1 -->Added` through `<!-- 5 -->Security`);
`striptags | trim` is the native strip filter (per `git cliff --init` default template;
`regex_replace` is NOT a built-in Tera filter in git-cliff 2.14.1 —
`Filter 'regex_replace' not found` confirmed via dry-run).
`[git] group_order` MUST NOT appear in cliff.toml (it does not exist in git-cliff
2.14.1). Verify by: `grep 'group_order' cliff.toml` must return no match.
(traces to ADR-063 v1.7 D3 — filter-based two-part body; Breaking Changes first via
`commits | filter(attribute="breaking", value=true)`; `striptags | trim` for non-breaking)

---

## Previous Story Intelligence

N/A — first story in E-REL-NOTES. No predecessor stories exist for the CHANGELOG
tooling epic.

---

## Architecture Compliance Rules

| Rule | Source | Enforcement |
|------|--------|-------------|
| git-cliff 2.14.1 pinned | ADR-063 D1 | grep in release-prep.yml |
| `--unreleased --tag ... --prepend` only (not `--output`, not `--latest`) | ADR-063 v1.2 D5 | grep confirms no --output and no --latest flags |
| cliff.toml at repo root | ADR-063 D3 | `ls cliff.toml` |
| Full checkout depth in release-prep.yml | ADR-063 D5 (git cliff needs tag range history) | Verify fetch-depth: 0 in checkout step |
| Technical-writer step runs AFTER git-cliff (S-REL-WRITER-001) | ADR-063 D5 step order | S-REL-WRITER-001 is a separate story; not in scope here |
| release.yml `--notes-file` extraction UNCHANGED | ADR-063 D5 | release.yml is not modified by this story |

---

## Library & Framework Requirements

| Dependency | Version | Notes |
|------------|---------|-------|
| `git-cliff` | `2.14.1` | `cargo install git-cliff --version 2.14.1 --locked`; Rust-native static binary; no new runtime |
| `GITHUB_TOKEN` | not needed | Token-free PR links via `commit_preprocessors` regex (ADR-063 v1.7 D3 Deviation-1); no GITHUB_TOKEN required for cliff.toml operation; `[remote.github]` retained but body template ignores `commit.remote.*` fields |

---

## File Structure Requirements

| File | Action | Notes |
|------|--------|-------|
| `cliff.toml` | Create | At repo root; normative ADR-063 D3 configuration |
| `.github/workflows/release-prep.yml` | Modify | Replace Step 7 scaffold with git-cliff invocation |
| `.github/workflows/release.yml` | DO NOT modify | --notes-file mechanism is explicitly unchanged per ADR-063 D5 |
| `CHANGELOG.md` | Verify only | Dry-run output must not corrupt the existing file |

---

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| `cliff.toml` | repo root | Pure (Tera template config; no runtime behavior) |
| `release-prep.yml` Step 7 | `.github/workflows/` | Effectful (CI workflow; writes CHANGELOG.md) |

---

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| `cliff.toml` | pure-core | Static TOML configuration; no runtime I/O |
| `.github/workflows/release-prep.yml` | effectful-shell | CI workflow that writes files and opens PRs |

---

## Holdout Applicability

**Determination: N/A**

The story-level holdout gate (CLAUDE.md, human-approved 2026-07-13) requires a built
prism binary and an MCP-visible surface with wire-level assertions. This story has
`tdd_mode: facade` and `crates_touched: []` — it produces no prism binary. Its entire
deliverable is `cliff.toml` (TOML config) and a `.github/workflows/release-prep.yml`
patch.

All verification surfaces are explicitly exercised by the implementer before PR:
AC-002 requires running `git cliff --unreleased --output /dev/stdout` and inspecting
skip-rule output; AC-006 requires verifying Breaking Changes precedes Added in dry-run
output; Tasks 3 and 6 both require running the dry-run and documenting the output
snippet in the PR description. git-cliff is an external CLI tool, not the prism MCP
binary, so no MCP stdio wire-level assertions are possible.

No machine-verifiable hidden surface with genuine info-asymmetry exists. Do NOT
fabricate a holdout for a gate that definitionally does not apply.

`holdout_scenarios: []` — no scenarios authored; HOLDOUT-INDEX unchanged.

---

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Shallow clone (`fetch-depth: 1`) in release-prep.yml checkout | `git cliff` cannot walk tag range; checkout must use `fetch-depth: 0`. Verify before AC-006 |
| EC-002 | No prior tag exists (first release) | Do NOT use `--latest` here: `--latest` processes commits belonging to the latest *existing* tag and is unreliable with zero prior tags. Per ADR-063 D6, the first release uses `git cliff --tag v1.0.0-beta.1` (no `--latest`); the git-cliff 2.14.1 documented full-history/first-release pattern is `git cliff --unreleased --tag <version>` (source: git-cliff.org/docs/usage/examples, verified 2026-09-05). The beta.1 CHANGELOG is produced by S-REL-BETA1-NOTES-001, which correctly omits `--latest`. |
| EC-003 | Prior tag is `v1.0.0-rc.1`; unreleased commits span `v1.0.0-rc.1..HEAD` | `--unreleased` automatically selects commits not yet in any tag; combined with `--tag v1.0.0-beta.1` the output section is headed `## [1.0.0-beta.1]`. No `--latest` needed |
| EC-004 | `GITHUB_TOKEN` not set locally for dry-run | PR links ARE injected even without GITHUB_TOKEN: the `commit_preprocessors` regex rewrites `(#NNN)` in the commit subject before Tera rendering, so no token is needed for the preprocessor path. The `[remote.github]` API integration is not used by the body template. Dry-run PR link output matches CI output. |
| EC-005 | Commit messages don't follow Conventional Commits (human hotfix commits) | `filter_unconventional = true` omits them; CHANGELOG quality is bounded by commit discipline per ADR-063 §Consequences |

---

## Changelog

| Version | Date | Author | Change |
|---------|------|--------|--------|
| 1.7 | 2026-09-07 | story-writer | TD-VSDD-097 Dim-2 downstream sweep (ADR-063 v1.9 §D3). Task 2 body-template: "UNCHANGED" reference → explicit no-`{% else %}`-arm constraint: body MUST be `{% if version %}...{% endif %}` with NO `{% else %}` arm; `## [Unreleased]` comes solely from `[changelog] header`, never from body template (else-arm produces duplicate heading on no-`--tag` dry-runs, ADR-063 v1.9 D3). ADR-063 v1.8 D3 ref updated to v1.9 D3 in Task 2 and §Behavioral Contracts header-row. |
| 1.6 | 2026-09-07 | story-writer | TD-VSDD-097 Dim-2 downstream sweep (ADR-063 v1.8 §D3 `--prepend` mechanism). Task 2 `[changelog] header` bullet: "empty header" → "header carries full masthead+preamble+`## [Unreleased]`" (ADR-063 v1.8 D3: `header = ""` is INCORRECT — corrupts CHANGELOG.md by inserting new section ABOVE masthead). Task 4 (Step 7): single git-cliff invocation → enumerated 3-substep flow: (a) PRE-STRIP Python pre-git-cliff idempotency strip of masthead+`## [Unreleased]`, (b) `git cliff --unreleased --tag ${VERSION_TAG} --prepend CHANGELOG.md` (unchanged core), (c) LINK-REF UPDATE Python token-free post-git-cliff repoint `[Unreleased]`+insert `[VERSION]` compare-link ref. AC-003: updated to verify all 3 substeps present (PRE-STRIP + git cliff + LINK-REF UPDATE). Behavioral Contracts table: old Step 7 invocation row → 3-substep description; new ADR-063 v1.8 D3 `header` carries masthead row added. risk_mitigations: new `header = ""` corruption + 3-substep bullet. AC-005 (`--prepend` without `--output`) UNCHANGED. AC-006 (breaking ordering/no group_order) UNCHANGED. |
| 1.5 | 2026-09-07 | story-writer | TD-VSDD-097 Dim-2 downstream sweep (ADR-063 v1.7 §D3). DEVIATION-1 (PR-link token-free): Task 2 `commit_preprocessors` bullet added — `(#NNN)` rewritten to markdown link BEFORE Tera rendering via regex preprocessor; no GITHUB_TOKEN needed; body template reads `{{ commit.message }}` only; `commit.remote.pr_number`/`commit.remote.pr_url` NOT read. `[remote.github]` note updated (retained, body template ignores it). Behavioral Contracts PR-link row updated. Library & Framework GITHUB_TOKEN row updated (not needed). risk_mitigations GITHUB_TOKEN bullet updated. DEVIATION-2 (Breaking Changes filter-based): `{ breaking = true, group = "<!-- 0 -->Breaking Changes" }` commit_parser guidance removed — that parser-level grouping NOT effective in git-cliff 2.14.1 and MUST NOT be relied upon. Task 2 commit_parsers bullet updated (numbered prefix starts at `<!-- 1 -->Added`; no `<!-- 0 -->`). Task 2 Breaking Changes bullet replaced with Tera filter-based two-part body: `commits \| filter(attribute="breaking", value=true)` FIRST, then `commits \| filter(attribute="breaking", value=false) \| group_by(attribute="group")`. Behavioral Contracts Breaking Changes row updated. risk_mitigations Breaking Changes bullet updated. AC-006 mechanism updated (filter-based; `group_order` MUST NOT and `striptags \| trim` preserved). No-group_order, striptags\|trim, numbered-prefix for non-breaking groups (`<!-- 1 -->Added` … `<!-- 5 -->Security`) preserved throughout. |
| 1.4 | 2026-09-07 | story-writer | F-7/Dim-2 (ADR-063 v1.6 §D3 reconciliation): Task 2 [remote.github] owner corrected jmagady→drbothen (ADR-063 v1.2 D3 C3 — owner=drbothen is the single source of truth per §D3 and shipped cliff.toml). §Narrative "and author attribution" removed — ADR-063 v1.6 D3 F-4 drops per-entry @author attribution for single-author project; PR-number links retained. |
| 1.3 | 2026-09-07 | story-writer | TD-VSDD-097 Dim-2 downstream sweep (ADR-063 v1.4→v1.5 §D3 spec-accuracy correction): replace `regex_replace(pattern="<!-- \\d+ -->", replacement="")` with `striptags \| trim` in Task 2 Tera body template snippet, AC-006 marker-strip criterion, and risk_mitigations Breaking Changes ordering bullet. State explicitly that `regex_replace` is NOT a built-in Tera filter in git-cliff 2.14.1 (`Filter 'regex_replace' not found` confirmed via dry-run); `striptags \| trim` is the native mechanism per `git cliff --init` default template. Numbered-group-name-prefix ordering language and `[git] group_order` MUST-NOT statement unchanged. ADR-063 version refs updated from v1.4 D3 → v1.5 D3 throughout. Status: ready unchanged. |
| 1.2 | 2026-09-06 | story-writer | Sweep #14 (group_order): replace group_order language in Task 2, AC-006, and risk_mitigations with ADR-063 v1.4 D3 numbered-group-name-prefix idiom; state that [git] group_order does NOT exist in git-cliff 2.14.1 and MUST NOT be added; status draft→ready |
| 1.1 | 2026-09-05 | story-writer | Sync to ADR-063 v1.2 D5 — replace `--latest` with `--unreleased --tag ... --prepend` throughout; Task 4 YAML, Task 6 dry-run, Behavioral Contracts table, Architecture Compliance Rules, risk_mitigations, title, and blocks comment all updated to use `--unreleased` |
| 1.0 | 2026-09-05 | story-writer | Initial — ADR-063 D1/D3/D5 materialization |
