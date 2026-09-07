---
document_type: story
story_id: S-REL-CLIFF-001
title: "devops: git-cliff setup — cliff.toml at repo root + release-prep.yml Step 7 replacement (git log scaffold → git cliff --unreleased --prepend)"
wave: F-A
epic_id: E-REL-NOTES
priority: P0
status: ready
version: "1.3"
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
  - "The GITHUB_TOKEN secret is available by default in GitHub Actions. The [remote.github]
    section in cliff.toml uses it for PR link injection. No PAT required."
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
    mechanism is the numbered-group-name-prefix idiom per ADR-063 v1.5 D3: commit_parsers
    group values carry <!-- N --> HTML comment sort prefixes (<!-- 0 -->Breaking Changes
    before <!-- 1 -->Added); the Tera body template strips them via
    {{ group | striptags | trim }} (native git-cliff 2.14.1 mechanism per `git cliff --init`
    default template; regex_replace is NOT a built-in Tera filter in git-cliff 2.14.1 —
    Filter 'regex_replace' not found confirmed via dry-run).
    [git] group_order is NOT a valid key in git-cliff 2.14.1 and MUST NOT be added."
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
**Version:** v1.3
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
Added/Fixed/Performance/Changed/Security sections with PR numbers and author attribution.

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
| ADR-063 D3 | Breaking Changes section ordered BEFORE Added |
| ADR-063 D3 | GitHub PR link injection via [remote.github] section (owner/repo) |
| ADR-063 v1.2 D5 | Step 7 invocation: `git cliff --tag "${VERSION_TAG}" --unreleased --prepend CHANGELOG.md` |
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
   - `[changelog]` section with empty `header`, the Tera body template, empty `footer`,
     and `trim = true`
   - `[git]` section: `conventional_commits = true`, `filter_unconventional = true`,
     `tag_pattern = "v[0-9].*"`, `sort_commits = "oldest"`
   - `commit_parsers` table: feat→Added, fix→Fixed, perf→Performance, refactor→Changed,
     security→Security, docs/ci/test/chore/style/build/revert each with `skip = true`
   - `protect_breaking_commits = true`
   - `[remote.github]` section: `owner = "jmagady"`, `repo = "prism"`
   - Breaking Changes section must appear BEFORE Added in the output — use the
     numbered-group-name-prefix idiom per ADR-063 v1.5 D3: `commit_parsers` group
     values carry `<!-- N -->` HTML comment sort prefixes (`<!-- 0 -->Breaking
     Changes` before `<!-- 1 -->Added`); the Tera body template strips them via
     `{{ group | striptags | trim }}` (native git-cliff 2.14.1 mechanism per
     `git cliff --init` default template; `regex_replace` is NOT a built-in Tera
     filter in git-cliff 2.14.1 — `Filter 'regex_replace' not found` confirmed via
     dry-run). `[git] group_order` does NOT exist in git-cliff 2.14.1 and MUST NOT
     be added. Verify section ordering in dry-run output.

3. **Run dry-run on develop history:**
   ```bash
   git cliff --unreleased --output /dev/stdout
   ```
   Inspect output. If output is excessively noisy (many non-skip entries from
   chore/ci/docs), add additional skip rules per ADR-063 D6. Document the output
   snippet for the PR description.

4. **Patch `release-prep.yml` Step 7:**
   Replace the existing `Scaffold CHANGELOG entry` step (uses `git log --merges
   --oneline | head -100` + Python scaffold heredoc) with:
   ```yaml
   - name: Generate CHANGELOG body (git-cliff)
     run: |
       set -euo pipefail
       cargo install git-cliff --version 2.14.1 --locked --quiet
       git cliff --tag "${VERSION_TAG}" --unreleased --prepend CHANGELOG.md
   ```
   Per ADR-063 v1.2 D5: `--unreleased` selects all commits not yet in any tag;
   `--tag` sets the new section header; `--prepend` writes ONLY to the top of
   CHANGELOG.md. Do NOT use `--latest` — that flag is for existing-tag ranges
   and is unreliable for first-release or tag-boundary disambiguation.
   The `VERSION_TAG` environment variable is already set in the workflow context
   (verify the exact var name from the surrounding steps).

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

### AC-003: release-prep.yml Step 7 replaced with git-cliff invocation
`grep -n 'git cliff' .github/workflows/release-prep.yml` returns at least one match.
`grep -n 'git log --merges' .github/workflows/release-prep.yml` returns no match.
(traces to ADR-063 D5 — release-prep.yml integration point)

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
CHANGELOG body. The numbered-group-name-prefix idiom (ADR-063 v1.5 D3) must be
in place: `<!-- 0 -->Breaking Changes` appears before `<!-- 1 -->Added` in
cliff.toml `commit_parsers`; the Tera body template strips the `<!-- N -->` prefix
via `{{ group | striptags | trim }}` (native git-cliff 2.14.1 mechanism per
`git cliff --init` default template; `regex_replace` is NOT a built-in Tera filter
in git-cliff 2.14.1 — `Filter 'regex_replace' not found` confirmed via dry-run);
`[git] group_order` MUST NOT appear in cliff.toml (it does not exist in git-cliff
2.14.1). Verify by: `grep 'group_order' cliff.toml` must return no match.
(traces to ADR-063 v1.5 D3 — numbered-group-name-prefix idiom; Breaking Changes
ordered first via <!-- 0 --> sort prefix; `striptags | trim` is the native strip filter)

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
| `GITHUB_TOKEN` | N/A | Built-in GitHub Actions secret; required for PR link injection via [remote.github] |

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
| EC-004 | `GITHUB_TOKEN` not set locally for dry-run | PR links will not be injected in local dry-run; that is acceptable (CI has the token) |
| EC-005 | Commit messages don't follow Conventional Commits (human hotfix commits) | `filter_unconventional = true` omits them; CHANGELOG quality is bounded by commit discipline per ADR-063 §Consequences |

---

## Changelog

| Version | Date | Author | Change |
|---------|------|--------|--------|
| 1.3 | 2026-09-07 | story-writer | TD-VSDD-097 Dim-2 downstream sweep (ADR-063 v1.4→v1.5 §D3 spec-accuracy correction): replace `regex_replace(pattern="<!-- \\d+ -->", replacement="")` with `striptags \| trim` in Task 2 Tera body template snippet, AC-006 marker-strip criterion, and risk_mitigations Breaking Changes ordering bullet. State explicitly that `regex_replace` is NOT a built-in Tera filter in git-cliff 2.14.1 (`Filter 'regex_replace' not found` confirmed via dry-run); `striptags \| trim` is the native mechanism per `git cliff --init` default template. Numbered-group-name-prefix ordering language and `[git] group_order` MUST-NOT statement unchanged. ADR-063 version refs updated from v1.4 D3 → v1.5 D3 throughout. Status: ready unchanged. |
| 1.2 | 2026-09-06 | story-writer | Sweep #14 (group_order): replace group_order language in Task 2, AC-006, and risk_mitigations with ADR-063 v1.4 D3 numbered-group-name-prefix idiom; state that [git] group_order does NOT exist in git-cliff 2.14.1 and MUST NOT be added; status draft→ready |
| 1.1 | 2026-09-05 | story-writer | Sync to ADR-063 v1.2 D5 — replace `--latest` with `--unreleased --tag ... --prepend` throughout; Task 4 YAML, Task 6 dry-run, Behavioral Contracts table, Architecture Compliance Rules, risk_mitigations, title, and blocks comment all updated to use `--unreleased` |
| 1.0 | 2026-09-05 | story-writer | Initial — ADR-063 D1/D3/D5 materialization |
