---
document_type: adr
adr_id: "ADR-063"
title: "CHANGELOG and Release Notes Architecture — git-cliff + Two-Layer Model + First-Release Handling"
status: ACCEPTED
date: "2026-09-05"
version: "1.8"
producer: architect
subsystems_affected: [SS-22]
supersedes: []
superseded_by: null
amends: null
anchor_stories:
  - S-REL-CLIFF-001        # D1/D3/D5 — cites ADR-063 D1/D3/D5 in §Authority
  - S-REL-WRITER-001       # D4/D5 — cites ADR-063 D4/D5 in §Authority
  - S-REL-BETA1-NOTES-001  # D6 — cites ADR-063 D6 in §Authority
  - S-REL-VBUMP-001        # D1 — cites ADR-063 git-cliff ownership in §Authority
related_adrs: [ADR-062]
related_bcs: []
locked_decisions: []
wiring_deferred_to: null
inputs:
  - .factory/research/release-notes-automation-2026.md
  - RELEASING.md
  - docs/RELEASE-CHANNELS.md
  - .github/workflows/release-prep.yml
  - .github/workflows/release.yml
input-hash: "360fc13"
---

# ADR-063: CHANGELOG and Release Notes Architecture — git-cliff + Two-Layer Model + First-Release Handling

## Status

ACCEPTED v1.8 (2026-09-07) — D3 `--prepend` mechanism canonicalized: empirically-verified
3-substep release-prep.yml Step 7 flow (git-cliff 2.14.1, verified idempotent over 3 simulated
release cycles on the REAL CHANGELOG.md).

`git cliff --prepend` with an **empty** `[changelog] header` corrupts CHANGELOG.md: the new
`## [version]` section is inserted ABOVE the `# Changelog` masthead (burying the masthead and
orphaning `## [Unreleased]`), and the existing `[version]: compare-link` reference definitions
are not updated. The v1.7 cliff.toml sketch `header = ""` is therefore INCORRECT and MUST NOT
be used.

Canonical 3-substep mechanism: (1) `[changelog] header` carries the masthead + preamble +
`## [Unreleased]` (NOT empty) — git-cliff writes `header + new-section + existing-content`,
placing the masthead at the top followed by the new version section (correct keepachangelog
order); (2) release-prep.yml Step 7 PRE-STRIP substep (Python, pre-git-cliff): strips prior
masthead and `## [Unreleased]` from CHANGELOG.md leaving the file at the first versioned
section — idempotency across repeated release cycles; (3) release-prep.yml Step 7 LINK-REF
UPDATE substep (Python, token-free, post-git-cliff): repoints `[Unreleased]` compare ref to
new version and inserts a `[VERSION]: compare-link` reference definition from the existing ref
text and the VERSION env var; no GITHUB_TOKEN required.

The core `git cliff --unreleased --tag ${VERSION_TAG} --prepend CHANGELOG.md` invocation (D5)
is UNCHANGED — it runs between the PRE-STRIP (removes prior masthead/Unreleased) and the
LINK-REF UPDATE (fixes compare-link refs). cliff.toml sketch: `header = ""` corrected to full
masthead + preamble + `## [Unreleased]`. §D3: `--prepend` mechanism section added documenting
all 3 substeps as REQUIRED. §Consequences two-layer bullet updated. No change to D1/D2/D4/D5/D6.

TD-VSDD-097: Dim-1 CLEAR (no ADR sibling twin — confirmed). Dim-2 HANDOFF to story-writer:
S-REL-CLIFF-001 Task 2 describes cliff.toml + release-prep.yml Step 7 mechanism — sweep to
reflect header-carries-masthead + pre-strip + link-ref-update substeps in same burst;
S-REL-WRITER-001 and S-REL-BETA1-NOTES-001 confirmed no `header=""` or incorrect direct-prepend
copy (CLEAR for both). Dim-3 CLEAR: prepend-correctness MUST anchored to S-REL-CLIFF-001
AC-003 (Step 7 replacement covers all 3 substeps) + AC-005 (`--prepend` without `--output`
correct invocation) — confirmed.

ACCEPTED v1.7 (2026-09-07) — D3 spec-accuracy update: two empirically-verified mechanism
deviations from the v1.6 §D3 cliff.toml sketch documented to match the shipped cliff.toml
(worktree feature/E-REL-NOTES-changelog, S-REL-CLIFF-001 implementation).

Deviation 1 — PR-link mechanism (token-free): v1.6 sketch used `commit.remote.pr_number` /
`commit.remote.pr_url` in the body template, requiring `GITHUB_TOKEN` + `[remote.github]` API
integration; those fields are empty in a token-free dry-run. Verified mechanism: a
`commit_preprocessors` regex rewrites `(#NNN)` in squash-merge commit subjects into markdown
links BEFORE Tera rendering — no `GITHUB_TOKEN` or remote API call needed. Body template renders
`{{ commit.message }}` (which already contains the link); `commit.remote.*` fields are NOT read.
`[remote.github]` block retained but body template ignores it. §D3 "GitHub PR link injection"
prose updated; cliff.toml sketch `commit_preprocessors` updated; `commit.remote.pr_number/pr_url`
clause removed from body template. §Consequences PR-numbers bullet updated.

Deviation 2 — Breaking Changes section ordering: v1.6 sketch used
`{ breaking = true, group = "<!-- 0 -->Breaking Changes" }` as the first `commit_parsers` entry,
expecting breaking commits to route to that group before type-based parsers applied. Verified in
git-cliff 2.14.1: `{ breaking = true }` in `commit_parsers` does NOT override message-based group
assignment — a `feat!:` commit still lands in `<!-- 1 -->Added` regardless of parser position.
Shipped cliff.toml uses a Tera two-part body: manual `### Breaking Changes` section via
`commits | filter(attribute="breaking", value=true)` rendered BEFORE the main
`commits | filter(attribute="breaking", value=false) | group_by(attribute="group")` loop.
Breaking entries render first; non-breaking entries group normally. `{ breaking = true }` commit_parser
entry removed from sketch. §D3 sketch body template and breaking-section-ordering prose updated.
`{ breaking = true }` parser-level grouping is NOT effective in git-cliff 2.14.1 and MUST NOT be
relied upon. Numbered-group-name-prefix idiom + `striptags | trim` preserved for non-breaking groups.
No change to D1/D2/D4/D5/D6.

ACCEPTED v1.6 (2026-09-07) — D3 internal consistency correction (F-4/F-8): (F-4) per-entry
`@author` attribution DROPPED — Prism is a single-author project; uniform `by @handle` on every
changelog entry is noise, not differentiation signal. PR links `(#NNN)` are retained. cliff.toml
sketch body template updated: `by @{{ commit.remote.username }}` removed from PR-link block.
§D3 subsection renamed "GitHub PR link injection". §Consequences attribution bullet removed.
(F-8) Commit body inclusion policy corrected — feat commit bodies are NOT rendered;
subject-line-only for all entry types. BREAKING CHANGE footer values ARE surfaced inline via
`commit.breaking_description` (appended as `— {description}` when present); cliff.toml sketch
updated with `breaking_description` render block. §D3 "Commit body inclusion policy" prose
corrected. No change to D1/D2/D4/D5/D6.

ACCEPTED v1.5 (2026-09-07) — D3 spec-accuracy correction: git-cliff 2.14.1 has NO built-in
`regex_replace` Tera filter (`Filter 'regex_replace' not found` confirmed via dry-run). The
native marker-stripping mechanism is `striptags | trim` (per the `git cliff --init` default
template). cliff.toml sketch Tera body template updated to `{{ group | striptags | trim }}`.
Numbered-group-name-prefix ordering intent is UNCHANGED — `<!-- 0 -->Breaking Changes` still
renders before `<!-- 1 -->Added` via ascending string-sort of `group_by(attribute="group")`.
`[git] group_order` non-existent key statement unchanged (still correct). No change to D1/D2/D4/D5/D6.

ACCEPTED v1.4 (2026-09-06) — D3 corrected: `[git] group_order` does not exist in git-cliff
2.14.1 and MUST NOT be used. Numbered-group-name-prefix idiom is the authoritative mechanism:
`commit_parsers` group values carry `<!-- N -->` sort prefixes; Tera body template strips them
via `regex_replace` (git-cliff Tera extension). `breaking = true` parser added to route breaking
commits into `<!-- 0 -->Breaking Changes` ahead of type-based parsers. Reference:
git-cliff.org/docs §Configuration → Template Context → regex_replace. No change to D1/D2/D4/D5/D6
decision content.

ACCEPTED v1.3 (2026-09-05) — v1.3 backfills `anchor_stories` from §Authority ground truth per
SAC-2: four E-REL-NOTES/E-REL-IDENTITY stories now exist on disk and cite ADR-063 in §Authority
(S-REL-CLIFF-001/S-REL-WRITER-001/S-REL-BETA1-NOTES-001/S-REL-VBUMP-001); SAC-2 VERIFIED-EMPTY
annotation removed. Frontmatter/traceability only — no decision content changed. v1.2 corrects
D5 Step 7 flag (`--latest` → `--unreleased --tag`, the documented pre-tag pattern) and D3 repo
owner/field (owner `drbothen`, `[remote.github]` `owner`/`repo` fields). v1.1 moved D4 Layer-1
inside `## [VERSION]` block and fixed D5 dual-flag. Informed by
`.factory/research/release-notes-automation-2026.md` (research-agent, 2026-09-05, Tavily two-pass
deep synthesis + 13 registry verifications).

---

## Context

Prism ships from a 25-crate Rust workspace where only `crates/prism-bin/Cargo.toml` carries the
product version (ADR-062). Releases follow a channel ladder defined in `docs/RELEASE-CHANNELS.md`:
nightly → dev → alpha → beta → rc → stable. Pre-releases tag `develop`; stable promotes to `main`
via `release-promote.yml`.

`RELEASING.md §5` defines a two-layer model in prose: Layer 1 is a manually-prepended curated
narrative block; Layer 2 is auto-generated notes. In practice, the Layer 2 generator in
`release-prep.yml` is a weak scaffold — it runs `git log --merges --oneline | head -100` and emits
an uncategorized seed list under a `### Changed/Added/Fixed (seed list — review + categorize)`
heading. The human must manually categorize, and there is no Keep-a-Changelog sectioning,
PR/author links, or channel-range awareness.

The current `release.yml` uses `--notes-file` to pass the extracted CHANGELOG section to
`gh release create`. This mechanism is correct and must not be changed.

Research findings (`release-notes-automation-2026.md`) evaluated 12 tools across 7 criteria for
Prism's constraints (single-product-version, `publish = false`, enforced Conventional Commits via
lefthook, AI-agent VSDD factory):

- **Rejected: release-plz** — per-crate version-bump + crates.io-publish model conflicts with
  Prism's single-version reality.
- **Rejected: Changesets/towncrier** — non-Rust runtimes (Node/Python); per-PR fragment system
  adds friction and drift risk that enforced Conventional Commits already prevents.
- **Rejected: standard-version** — deprecated/archived.
- **Chosen: git-cliff** — Rust-native static binary (`2.14.1`, 2026-09-01, crates.io), Tera
  templates, config-driven `commit_parsers` → Keep-a-Changelog sections, GitHub PR/author links.
  No new runtime added to the toolchain.

---

## Decision

### D1 — git-cliff as Canonical Changelog Assembler

**git-cliff `2.14.1`** (pinned) is the canonical tool for assembling Prism's CHANGELOG body.
It reads Conventional Commits from git history and emits categorized, linked prose via a
project-owned `cliff.toml`.

Version pin rationale: `2.14.1` was the current release at time of this ADR (verified 2026-09-01
against crates.io). Future upgrades are made by bumping the pin in `release-prep.yml` and noting
the version in the next ADR version. Breaking changes in git-cliff's Tera template API are
mitigated by pinning.

Installation in CI: `cargo install git-cliff --version 2.14.1 --locked` in `release-prep.yml`.
No binary cache is required for v1; a pre-built cache (GitHub Actions cache or a step that checks
for `git-cliff` before installing) may be added to optimize wall-clock time in a follow-up.

### D2 — Conventional Commits as the Sole Entry Source

Conventional Commits, already enforced by lefthook pre-commit and pre-push hooks, are the sole
source of CHANGELOG entries. No per-PR fragment file system (Changesets, towncrier, newsroom) is
introduced.

Rationale: lefthook enforces Conventional Commits unconditionally; every merged commit is already
a categorizable changelog entry. A fragment gate would layer noisy false-blocks (chore, ci, docs
PRs) on top of an already-enforced commit regime.

If a stronger per-release currency gate is ever needed, prefer `git cliff --unreleased` non-empty
assertion in CI rather than a per-PR fragment presence check.

### D3 — `cliff.toml` Categorization Convention

`cliff.toml` is placed at the repo root. It defines:

**Commit type → Keep-a-Changelog section mapping (normative):**

| Conventional Commit type | CHANGELOG section |
|--------------------------|-------------------|
| `feat` | Added |
| `fix` | Fixed |
| `perf` | Performance |
| `refactor` | Changed |
| `security` (unofficial but common) | Security |
| `docs`, `ci`, `test`, `chore`, `style`, `build`, `revert` | (skip — omit from body) |
| Breaking change footer (`BREAKING CHANGE:`) | Breaking Changes (top section) |

The Breaking Changes section is ordered first, before Added, to surface upgrade risk immediately.

**GitHub PR link injection (token-free via `commit_preprocessors`):** Prism uses squash-merge;
every merged PR commit subject already contains `(#NNN)`. A `commit_preprocessors` regex
(`{ pattern = "\\(#([0-9]+)\\)", replace = "([#${1}](https://github.com/drbothen/prism/pull/${1}))" }`)
rewrites `(#NNN)` into a markdown link BEFORE Tera rendering — no `GITHUB_TOKEN` or remote API
call needed. The body template renders `{{ commit.message }}`, which already contains the link.
The `commit.remote.pr_number` and `commit.remote.pr_url` fields (populated only when
`GITHUB_TOKEN` is available via the `[remote.github]` API integration) are NOT read by the body
template; the v1.6 approach using those fields renders nothing in a token-free dry-run. The
`[remote.github]` block (`owner = "drbothen"`, `repo = "prism"`, fields distinct — no combined
`repository` key) is retained for possible future remote-API use but the body template ignores it.
Per-entry `@author` attribution is NOT injected — Prism is a single-author project; uniform author
handles on every entry are noise rather than signal.

**Commit body inclusion policy:** Only the commit subject line (`{{ commit.message }}`) is
rendered for all entry types. BREAKING CHANGE footer values are surfaced inline via
`{{ commit.breaking_description }}` when present (appended after the subject as
`— {description}`). Full commit bodies are NOT rendered — feat bodies tend toward verbose prose
better suited to the Layer-1 curated top-block (D4); the BREAKING CHANGE footer value via
`breaking_description` surfaces the migration signal at lower verbosity without body noise.

**Sketch of `cliff.toml` (informative — authoritative at file creation time):**

```toml
[changelog]
header = """
# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).

## [Unreleased]

"""
body = """
{% if version %}## [{{ version | trim_start_matches(pat="v") }}] - {{ timestamp | date(format="%Y-%m-%d") }}
{% else %}## [Unreleased]
{% endif %}
{% set breaking_commits = commits | filter(attribute="breaking", value=true) %}
{% if breaking_commits | length > 0 %}
### Breaking Changes
{% for commit in breaking_commits %}
- **BREAKING** {{ commit.message }}{% if commit.breaking_description %} — {{ commit.breaking_description }}{% endif %}
{% endfor %}
{% endif %}
{% for group, group_commits in commits | filter(attribute="breaking", value=false) | group_by(attribute="group") %}
### {{ group | striptags | trim }}
{% for commit in group_commits %}
- {{ commit.message }}
{% endfor %}
{% endfor %}
"""
footer = ""
trim = true

[git]
conventional_commits = true
filter_unconventional = true
commit_preprocessors = [
  # Rewrite "(#NNN)" in squash-merge commit subjects to markdown PR links — token-free.
  { pattern = "\\(#([0-9]+)\\)", replace = "([#${1}](https://github.com/drbothen/prism/pull/${1}))" },
]
commit_parsers = [
  { message = "^feat", group = "<!-- 1 -->Added" },
  { message = "^fix", group = "<!-- 2 -->Fixed" },
  { message = "^perf", group = "<!-- 3 -->Performance" },
  { message = "^refactor", group = "<!-- 4 -->Changed" },
  { message = "^security", group = "<!-- 5 -->Security" },
  { message = "^docs", skip = true },
  { message = "^ci", skip = true },
  { message = "^test", skip = true },
  { message = "^chore", skip = true },
  { message = "^style", skip = true },
  { message = "^build", skip = true },
  { message = "^revert", skip = true },
]
protect_breaking_commits = true
filter_commits = false
tag_pattern = "v[0-9].*"
skip_tags = ""
ignore_tags = ""
topo_order = false
sort_commits = "oldest"

[remote.github]
owner = "drbothen"
repo = "prism"
```

**Breaking Changes section ordering — authoritative mechanism (filter-based Tera two-part body;
`group_order` does not exist in git-cliff 2.14.1; `{ breaking = true }` parser-level grouping
NOT effective):**

`[git] group_order` is NOT a valid key in git-cliff 2.14.1's `cliff.toml` schema and MUST NOT be
added. `{ breaking = true }` in `commit_parsers` does NOT override message-based group assignment
in git-cliff 2.14.1 — a `feat!:` commit still lands in its message-derived group (e.g.,
`<!-- 1 -->Added`) regardless of parser position; this was verified empirically during
S-REL-CLIFF-001 implementation. `{ breaking = true }` parser-level grouping is NOT effective in
git-cliff 2.14.1 and MUST NOT be relied upon; the `{ breaking = true }` entry is therefore absent
from `commit_parsers` in the sketch above.

The authoritative mechanism is a **Tera two-part body**: (1) a manual `### Breaking Changes`
section rendered FIRST via `commits | filter(attribute="breaking", value=true)`, and (2) a
`commits | filter(attribute="breaking", value=false) | group_by(attribute="group")` loop for
non-breaking commits. Breaking entries render before Added/Fixed/etc.; non-breaking entries group
normally. The `striptags | trim` strip (git-cliff 2.14.1 has NO built-in `regex_replace` Tera
filter — `Filter 'regex_replace' not found` confirmed via `git cliff --unreleased --output /dev/stdout`
dry-run; `striptags | trim` is per the `git cliff --init` default template) is applied to group
names in the non-breaking `group_by` loop. The numbered-group-name-prefix idiom (`<!-- N -->` sort
prefixes on `commit_parsers` group values) is still used for non-breaking sections to maintain
consistent ordering.

The implementing story (S-REL-CLIFF-001) owns the canonical `cliff.toml` and validates the
filter-based breaking section against git-cliff 2.14.1; the filter-based approach is the REQUIRED
mechanism (anchored to S-REL-CLIFF-001 AC-006).

**`--prepend` mechanism — canonical 3-substep release-prep.yml Step 7 flow (empirically
verified: git-cliff 2.14.1, idempotent over 3 simulated release cycles on the REAL
CHANGELOG.md):**

`git cliff --prepend` with an **empty** `[changelog] header` corrupts CHANGELOG.md structure:
the new `## [version]` section is inserted ABOVE the `# Changelog` masthead (burying the
masthead and orphaning `## [Unreleased]`), and the existing `[version]: compare-link` reference
definitions at the bottom of the file are not updated. The `header = ""` form MUST NOT be used.

The canonical mechanism is a **3-substep** release-prep.yml Step 7 flow; all substeps are
REQUIRED for idempotent, keepachangelog-compliant CHANGELOG.md production:

**Substep 1 — PRE-STRIP (Python, runs BEFORE `git cliff`):** Strip the `# Changelog` masthead
and any prior `## [Unreleased]` section from CHANGELOG.md, leaving the file starting at the
first `## [X.Y.Z]` section. Without this substep, each release cycle accumulates a duplicate
masthead from the previous run's `header` injection.

**Substep 2 — `git cliff` invocation (core; UNCHANGED from D5):**
`git cliff --unreleased --tag ${VERSION_TAG} --prepend CHANGELOG.md`. Because `[changelog] header`
carries the full masthead + preamble + `## [Unreleased]`, git-cliff writes
`header + new-section + existing-content`. After this substep the file starts with the masthead,
followed by the new `## [VERSION]` section, followed by the existing versioned history —
correct keepachangelog order.

**Substep 3 — LINK-REF UPDATE (Python, token-free, runs AFTER `git cliff`):** Repoint
`[Unreleased]: .../compare/vPREV...HEAD` to the new version and insert a new
`[VERSION]: .../compare/vPREV...vVERSION` compare-link reference definition using the existing
ref text and the VERSION env var. No GITHUB_TOKEN required. Maintains the keepachangelog
compare-link footer block that the existing CHANGELOG.md uses to resolve version ranges.

The implementing story (S-REL-CLIFF-001) owns the cliff.toml `header` field and the
release-prep.yml Step 7 substep scripts; the 3-substep flow is the REQUIRED `--prepend`
mechanism (anchored to S-REL-CLIFF-001 AC-003 + AC-005).

### D4 — Two-Layer CHANGELOG Model

RELEASING.md §5 formalizes two layers; this ADR specifies the mechanisms for each layer:

**Layer 1 — Curated top-block (human-authored, agent-assisted):**

- Content: Highlights / Breaking Changes narrative / Upgrade Notes
- Authorship: `vsdd-factory:technical-writer` agent drafts this block from the tag range at
  release-prep time, documenting current behavior only (not aspirational). The draft is a PR
  file edit in the release-prep branch. The technical-writer step runs AFTER git-cliff (D5) so
  the `## [VERSION]` block already exists; the draft is inserted INSIDE that block as `###`
  sections before the first git-cliff commit-derived `###` section.
- Gate: Human reviews and curates the draft in the release-prep PR before merge. The curated
  block is human-approved text; the technical-writer draft is an aid, not the final.
- Format: `### Highlights`, `### Breaking Changes (narrative)`, and (if relevant)
  `### Upgrade Notes` sections placed INSIDE the `## [vX.Y.Z]` block, before the first
  git-cliff commit-derived `###` section. Placement inside the block ensures that
  `release.yml`'s awk extraction (lines from `## [VERSION]` until the next `## [`) captures
  all Layer-1 content without any modification to `release.yml`.

**Layer 2 — git-cliff categorized body:**

- Content: Categorized commit entries per D3 (Added/Fixed/Performance/Changed/Security sections)
- Authorship: `git cliff --tag "${VERSION_TAG}" --unreleased --prepend CHANGELOG.md` invoked in
  `release-prep.yml` Step 7 (D5; replaces the `git log --merges --oneline` scaffold). Uses
  `--unreleased` (the documented pre-tag pattern) rather than `--latest`: `--unreleased` is robust
  for both first release (selects all commits when no prior tag exists) and recurring releases
  (selects commits since the last tag). `--latest` selects the range from the most recently created
  tag and is unreliable when the tag has not yet been created (release-prep runs pre-tag).
- Output: Written directly into `CHANGELOG.md` under the new version header
- No human modification of the body is required; it is treated as authoritative from git history

**CHANGELOG.md structure (top of file, normative):**

```
# Changelog

## [1.0.0-beta.1] - 2026-09-XX

### Highlights
[technical-writer draft, human-curated — inserted AFTER git-cliff prepends the ## [VERSION] block]

### Breaking Changes (narrative)
[technical-writer draft, human-curated — empty section omitted]

### Upgrade Notes
[technical-writer draft, human-curated — empty section omitted]

### Breaking Changes (from commits)
[git-cliff BREAKING CHANGE footers — from cliff.toml D3]

### Added
[git-cliff feat commits]

### Fixed
[git-cliff fix commits]

...

## [previous-version] - YYYY-MM-DD
...
```

Step order in `release-prep.yml`: (1) git-cliff prepends the `## [VERSION]` block to CHANGELOG.md
via `--unreleased --tag "${VERSION_TAG}" --prepend`; (2) technical-writer inserts `### Highlights`
/ `### Breaking Changes (narrative)` / `### Upgrade Notes` inside the `## [VERSION]` block, before
git-cliff's first `###` section; (3) human curates the Layer-1 sections in the release-prep PR
before merge. The awk extraction in `release.yml` captures the entire `## [VERSION]` block — both
Layer-1 `###` sections and Layer-2 commit sections — without modification.

### D5 — `release-prep.yml` Integration Point

The CHANGELOG scaffold step in `release-prep.yml` (currently step 7: `git log --merges --oneline
| head -100`) is REPLACED with a `git cliff` invocation:

```yaml
- name: Generate CHANGELOG body (git-cliff)
  run: |
    set -euo pipefail
    cargo install git-cliff --version 2.14.1 --locked --quiet
    # --unreleased --tag is the documented pre-tag pattern (git-cliff docs):
    # selects all commits not yet under a tag and renders them under VERSION_TAG.
    # Robust for both first release (no prior tag → all commits from root) and
    # recurring releases (commits since last tag). --latest is unreliable here
    # because release-prep runs BEFORE the tag is created.
    git cliff --tag "${VERSION_TAG}" --unreleased --prepend CHANGELOG.md
```

The `release.yml` `--notes-file` extraction step is UNCHANGED — it already extracts the
`## [VERSION]` section from CHANGELOG.md and passes it to `gh release create`. No modification
to `release.yml` is required. Because Layer-1 content is placed INSIDE the `## [VERSION]` block
(D4), both layers are captured by the existing awk extraction window.

The technical-writer dispatch for Layer 1 drafting is a NEW step in `release-prep.yml`, ordered
AFTER the git-cliff invocation. git-cliff creates the `## [VERSION]` block first via
`--unreleased --tag "${VERSION_TAG}" --prepend`; the technical-writer then edits CHANGELOG.md
to insert `### Highlights`, `### Breaking Changes (narrative)`, and `### Upgrade Notes` sections
inside that block, before git-cliff's first commit-derived `###` section. This ordering ensures
the awk extraction captures all content without any modification to `release.yml`.

### D6 — First-Release Handling for v1.0.0-beta.1

The first meaningful release (v1.0.0-beta.1) will cover the full development history back to the
initial commit — a long range with many chore/ci/refactor commits that are skipped by cliff.toml
(D3). The following constraints apply:

1. **Tag range:** The Step 7 invocation `git cliff --tag v1.0.0-beta.1 --unreleased --prepend
   CHANGELOG.md` (D5) handles the first release automatically. When no prior tag exists,
   `--unreleased` selects all commits from the repository root. When a prior tag exists (e.g.,
   `v1.0.0-rc.1`), `--unreleased` selects only commits since that tag. No special first-release
   invocation or explicit commit range is required.

2. **Noise control:** The `cliff.toml` skip rules (D3) suppress docs/ci/test/chore/style/build
   commits. The implementer of E-REL-NOTES must run a dry-run pass (`git cliff --unreleased
   --output /dev/stdout`) on the current develop history and inspect the output before beta.1.
   If the output is still noisy, additional skip rules may be added to `cliff.toml` via a
   follow-up commit before the tag.

3. **Curation gate:** For beta.1 the human curation gate (D4 Layer 1) is MANDATORY. The
   technical-writer agent drafts an intentionally short Highlights block (5–8 bullets maximum)
   summarizing the platform-level value, not a commit-by-commit recap. The human removes any
   bullets that describe incomplete or aspirational features.

4. **No retroactive entry authoring:** Commits already in history are not re-worded retroactively.
   The first-release CHANGELOG reflects the actual commit history; any quality gap in commit
   subjects is absorbed by the curation gate, not by manual history rewriting.

---

## Rationale

git-cliff is the correct choice for Prism for five reasons, all verified in
`release-notes-automation-2026.md`:

1. **No new runtime.** Single Rust static binary; `cargo install` in CI. Lefthook already pulls in
   Rust tooling. The Node.js (Changesets, towncrier, release-please) and Python (towncrier) tools
   add runtimes the workspace doesn't have.

2. **Single-version workspace fit.** git-cliff is pointed at the whole repo with a tag range; it
   emits one product-level changelog per range. This maps exactly onto Prism's model (one product
   version, one tag).

3. **Zero per-PR overhead.** The entry source is the Conventional Commit subject already required
   by lefthook. No fragment files, no extra PR step, no CI gate to tune.

4. **Channel ladder compatibility.** Each channel tag (beta, rc, stable) defines a git-cliff range.
   `--unreleased --tag vX.Y.Z` selects all commits not yet captured in a tag and renders them
   under the new version header. Pre-releases using BASE-MATCH (docs/RELEASE-CHANNELS.md §3)
   generate their own range since develop carries `1.0.0-dev` and each pre-release tag is a
   distinct version marker. The `--unreleased` flag works correctly for all positions in the
   channel ladder, including the first release where no prior tag exists.

5. **Release-plz is not fit for purpose here.** release-plz is the highest-quality Rust release
   automation tool but is built around per-crate version coordination and crates.io publishing.
   Prism has neither requirement; adopting release-plz would mean permanently suppressing its
   core behaviors.

---

## Consequences

### Positive

- CHANGELOG body generation is automated; no manual categorization step in release-prep
- Keep-a-Changelog structure (Added/Fixed/Performance/Changed/Security) applied consistently
- PR numbers appear in every entry automatically (token-free via `commit_preprocessors` regex —
  Prism's squash-merge commit subjects already contain `(#NNN)`; no `GITHUB_TOKEN` or remote API
  call needed); no per-entry author attribution (single-author project — uniform handles are noise,
  not signal)
- BREAKING CHANGE footer values are surfaced inline via `commit.breaking_description`
- The two-layer model in RELEASING.md §5 is operationalized: Layer-2 (git-cliff body) is prepended
  first as a `## [VERSION]` block via the canonical 3-substep release-prep.yml Step 7 flow (D3
  `--prepend` mechanism: PRE-STRIP substep removes prior masthead/`## [Unreleased]` before git-cliff
  runs; git-cliff injects the new section via the `[changelog] header`-carries-masthead mechanism;
  LINK-REF UPDATE substep maintains keepachangelog compare-link footer refs); Layer-1
  (technical-writer `### Highlights / Breaking Changes / Upgrade Notes` sections) is inserted inside
  that block afterward. The `release.yml` awk extraction captures both layers without any modification
  to `release.yml`.
- No new runtime dependency introduced to the workspace
- release.yml `--notes-file` mechanism unchanged; existing release publication path unaffected

### Negative / Trade-offs

- One new CLI tool (`git-cliff`) added to `release-prep.yml`. Contributors running release-prep
  locally need `cargo install git-cliff --version 2.14.1 --locked`.
- CHANGELOG quality is bounded by Conventional Commit subject quality. AI-agent commits already
  follow the convention, so this is low risk for factory-authored commits; human hotfix commits
  may need discipline reminders.
- The `cliff.toml` skip rules suppress chore/ci/docs commits entirely. If an operator wants to
  audit CI changes, they must use `git log`, not the CHANGELOG.
- First-release noise control (D6) requires a manual dry-run pass before beta.1.

### Status as of v1.2

ACCEPTED. Implementation pending E-REL-NOTES story authoring by product-owner. The immediate
next action is: product-owner authors S-REL-CLIFF-001 (cliff.toml creation — D3 owner is
`drbothen`; release-prep.yml Step 7 uses `--unreleased --tag "${VERSION_TAG}" --prepend` per D5)
and S-REL-WRITER-001 (technical-writer dispatch — Layer-1 inserted INSIDE the `## [VERSION]`
block per D4). See story breakdown in §Source / Origin below.

---

## Alternatives Considered

- **release-plz:** Rejected (see §Rationale #5 and `release-notes-automation-2026.md` §E).
  release-plz wraps git-cliff; the recommendation is to use git-cliff directly and keep Prism's
  existing release workflow.

- **towncrier (Python, fragment-based):** Rejected. Non-Rust runtime; per-PR fragment discipline
  adds overhead on top of an already-enforced Conventional Commit regime. Excellent prose quality
  is achievable via the technical-writer top-block at lower friction.

- **cocogitto:** A viable Rust-native alternative. Rejected in favour of git-cliff because (a)
  git-cliff has richer Tera templating, (b) git-cliff is more widely adopted in the Rust
  ecosystem, (c) git-cliff is the underlying engine used by release-plz, so expertise transfers.

- **semantic-release:** Node.js ecosystem; npm-shaped channel model. Not a fit for a Rust-native
  workspace without adding Node to CI.

- **Status quo (git log scaffold):** The current release-prep.yml scaffold step produces an
  uncategorized bullet list that the human must manually sort. This is acceptable for infrequent
  releases but does not scale and produces inconsistent CHANGELOG quality across releases.

---

## Source / Origin

- `.factory/research/release-notes-automation-2026.md` — 12-tool comparison matrix, AI changelog
  prior art, two-layer pattern, Prism-specific recommendation §E
- `RELEASING.md §5` — existing two-layer model (operationalized by this ADR)
- `docs/RELEASE-CHANNELS.md §3` — channel ladder and BASE-MATCH semantics
- `.github/workflows/release-prep.yml` — current scaffold step being replaced (D5)
- `.github/workflows/release.yml` — `--notes-file` mechanism confirmed unchanged

**Proposed story breakdown (hand to product-owner for E-REL-NOTES epic):**

| Story ID (proposed) | Title | ADR decisions | Rough scope |
|---------------------|-------|---------------|-------------|
| S-REL-CLIFF-001 | git-cliff setup: cliff.toml + release-prep.yml step replacement | D1, D3, D5 | Create cliff.toml; replace scaffold step in release-prep.yml; verify dry-run on develop history; add cliff binary version pin |
| S-REL-WRITER-001 | Technical-writer dispatch: Layer-1 top-block at release-prep | D4 | Add technical-writer agent invocation to release-prep.yml; define top-block template contract; document curation gate in RELEASING.md |
| S-REL-BETA1-NOTES-001 | First-release CHANGELOG: beta.1 dry-run, noise control, curation | D6 | Execute dry-run; tune skip rules; human curation gate; produce CHANGELOG.md beta.1 section |

---

## Changelog

| Version | Date | Author | Change |
|---------|------|--------|--------|
| 1.8 | 2026-09-07 | architect | D3 `--prepend` mechanism canonicalized: DEVIATION-3 — `git cliff --prepend` with empty `[changelog] header` corrupts CHANGELOG.md (buries `# Changelog` masthead; orphans `## [Unreleased]`; compare-link ref-defs not updated). Verified empirically: git-cliff 2.14.1. Canonical 3-substep release-prep.yml Step 7 flow, verified idempotent over 3 simulated release cycles: (1) cliff.toml `[changelog] header` carries masthead + preamble + `## [Unreleased]` (NOT empty) — git-cliff writes `header + new-section + existing-content` → correct keepachangelog order; (2) PRE-STRIP substep (Python, pre-git-cliff): strips prior masthead + `## [Unreleased]` from CHANGELOG.md leaving file at first versioned section — idempotency across release cycles; (3) LINK-REF UPDATE substep (Python, post-git-cliff, token-free): repoints Unreleased compare ref + inserts VERSION compare ref from existing ref + VERSION env var; no GITHUB_TOKEN. Core `git cliff --unreleased --tag ${VERSION_TAG} --prepend CHANGELOG.md` invocation UNCHANGED. cliff.toml sketch: `header = ""` corrected to full masthead + preamble + `## [Unreleased]`. §D3: `--prepend` mechanism section added (3 substeps documented as REQUIRED). §Consequences two-layer bullet updated. TD-VSDD-097: Dim-1 CLEAR (no ADR sibling twin — confirmed). Dim-2 HANDOFF to story-writer: S-REL-CLIFF-001 Task 2 sweep to header-carries-masthead + pre-strip + link-ref-update substeps; S-REL-WRITER-001 and S-REL-BETA1-NOTES-001 confirmed no `header=""` or incorrect direct-prepend copy (CLEAR for both). Dim-3 CLEAR: prepend-correctness MUST anchored to S-REL-CLIFF-001 AC-003 (Step 7 replacement) + AC-005 (`--prepend` without `--output`). No change to D1/D2/D4/D5/D6. |
| 1.7 | 2026-09-07 | architect | D3 spec-accuracy update: two empirically-verified mechanism deviations from the v1.6 §D3 cliff.toml sketch documented to match shipped cliff.toml (S-REL-CLIFF-001 implementation). DEVIATION-1 — PR-link mechanism (token-free): `commit_preprocessors` regex rewrites `(#NNN)` in squash-merge commit subjects into markdown links BEFORE Tera rendering — no `GITHUB_TOKEN` required; `commit.remote.pr_number/pr_url` NOT read by body template (empty in token-free dry-run); `[remote.github]` block retained but body template ignores it. cliff.toml sketch: `commit_preprocessors` updated with regex; `commit.remote.pr_number/pr_url` clause removed from body template; §D3 "GitHub PR link injection" prose updated; §Consequences PR-numbers bullet updated. DEVIATION-2 — Breaking Changes section ordering: `{ breaking = true }` in `commit_parsers` does NOT override message-based group assignment in git-cliff 2.14.1 (verified empirically: `feat!:` still lands in `<!-- 1 -->Added` regardless of parser position); single `group_by` loop with `{ breaking = true }` parser does not produce a separate Breaking Changes group. Shipped mechanism: Tera two-part body — manual `### Breaking Changes` section via `commits \| filter(attribute="breaking", value=true)` rendered BEFORE `commits \| filter(attribute="breaking", value=false) \| group_by(attribute="group")` loop; `{ breaking = true }` commit_parser entry removed from sketch; numbered-prefix idiom + `striptags \| trim` preserved for non-breaking groups. §D3 sketch body template and breaking-section-ordering prose updated. `{ breaking = true }` parser-level grouping MUST NOT be relied upon in git-cliff 2.14.1. No change to D1/D2/D4/D5/D6. TD-VSDD-097: Dim-1 CLEAR (no ADR sibling twin — confirmed). Dim-2 HANDOFF to story-writer: S-REL-CLIFF-001 Task-2 describes cliff.toml template (commit_parsers + body) referencing v1.6 mechanisms — sweep to v1.7 filter-based breaking section + preprocessor PR-link in same burst. Dim-3 CLEAR: ordering MUST anchored to S-REL-CLIFF-001 AC-006 — confirmed. |
| 1.6 | 2026-09-07 | architect | D3 internal consistency correction (F-4/F-8): (F-4) per-entry `@author` attribution DROPPED — single-author project; uniform `by @handle` on every entry is noise not signal; `by @{{ commit.remote.username }}` removed from cliff.toml sketch PR-link block; §D3 "GitHub PR/author injection" section renamed to "GitHub PR link injection"; §Consequences attribution bullet removed. (F-8) commit body inclusion policy corrected — subject-line-only for all entry types; feat commit bodies NOT rendered; BREAKING CHANGE footer value surfaced inline via `commit.breaking_description` (appended as `— {description}` when present); cliff.toml sketch updated with `breaking_description` render block; §D3 "Commit body inclusion policy" prose corrected; §Consequences updated. TD-VSDD-097: Dim-1 CLEAR (no ADR sibling twin). Dim-2 HANDOFF to story-writer: S-REL-CLIFF-001 §Narrative mentions "author attribution" — that phrase must be removed. Dim-3 CLEAR: no new MUSTs; existing CLIFF-001 AC anchors unchanged. |
| 1.5 | 2026-09-07 | architect | D3 spec-accuracy correction: git-cliff 2.14.1 has NO built-in `regex_replace` Tera filter (`Filter 'regex_replace' not found` confirmed via dry-run). `striptags \| trim` is the native marker-stripping mechanism (per `git cliff --init` default template). cliff.toml sketch body template updated from `regex_replace(pattern="<!-- \\d+ -->", replacement="")` to `striptags \| trim`. Numbered-group-name-prefix ordering intent unchanged. `[git] group_order` non-existent key statement unchanged (still correct). TD-VSDD-097: Dim-1 CLEAR (no ADR twin affected by Tera-filter-name fix). Dim-2 HANDOFF: `regex_replace` snippet copied into S-REL-CLIFF-001 (Task 2, AC-006, risk_mitigations) — story-writer must sweep those three locations to `striptags \| trim` in the same burst. Dim-3 CLEAR: marker-strip MUST anchored to S-REL-CLIFF-001 AC-006 grep + ordering gate. |
| 1.4 | 2026-09-06 | architect | D3 corrected: `[git] group_order` is not a valid git-cliff 2.14.1 cliff.toml key and MUST NOT be used. Numbered-group-name-prefix idiom is the authoritative mechanism for Breaking Changes section ordering: `commit_parsers` group values updated with `<!-- N -->` sort prefixes (`<!-- 0 -->Breaking Changes` first via `breaking = true` parser); Tera body template updated with `regex_replace` strip expression. Reference: git-cliff.org/docs §Configuration → Template Context → regex_replace. |
| 1.3 | 2026-09-05 | state-manager | MED-1 SAC-2 anchor_stories backfilled: four E-REL-NOTES/E-REL-IDENTITY stories verified on disk and citing ADR-063 in §Authority — S-REL-CLIFF-001 (D1/D3/D5), S-REL-WRITER-001 (D4/D5), S-REL-BETA1-NOTES-001 (D6), S-REL-VBUMP-001 (D1 git-cliff ownership). SAC-2 VERIFIED-EMPTY annotation removed. Frontmatter/traceability only — no decision content changed. |
| 1.2 | 2026-09-05 | architect | C2: D5 Step 7 `--latest` → `--unreleased --tag` (documented pre-tag pattern; robust for first and recurring releases; `--latest` unreliable when no prior tag exists). D4 Layer-2 description + step-order note updated to match. D6 item 1 updated (--unreleased handles first-release automatically). Rationale #4 updated. C3: D3 prose corrected — `[remote.github]` `owner`/`repo` fields replace `repository` combined key; owner corrected `jmagady` → `drbothen`; cliff.toml sketch updated. |
| 1.1 | 2026-09-05 | architect | BLOCKING-2: D5 git-cliff invocation fixed — removed `--output CHANGELOG.md`, keeps `--latest --prepend CHANGELOG.md` only (dual-flag caused duplicate sections). BLOCKING-4/5: D4 Layer-1 placement moved INSIDE the `## [VERSION]` block as `###` sections, not above it — ensures release.yml awk extraction captures Layer-1 without changes to release.yml; D5 technical-writer step reordered to run AFTER git-cliff; CHANGELOG structure example updated; Consequences updated. |
| 1.0 | 2026-09-05 | architect | Initial. D1 git-cliff 2.14.1; D2 Conventional Commits as sole entry source; D3 cliff.toml categorization convention; D4 two-layer model with technical-writer agent for Layer 1; D5 release-prep.yml integration (replaces git log scaffold, keeps release.yml --notes-file unchanged); D6 first-release handling for beta.1. |
