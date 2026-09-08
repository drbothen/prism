---
document_type: story
story_id: S-REL-NIGHTLY-NOTES-001
title: "devops: nightly release notes — git-cliff categorized changelog replacing canned summary body (release.yml nightly path only)"
wave: F-A
epic_id: E-REL
priority: P2
# P2 justification: own independent track (human-directed context from D-2499); nightly
# notes quality improvement does not gate any product story or beta.2. It runs after
# P0/P1 work clears. OWN INDEPENDENT TRACK, NOT beta.2-blocking.
status: draft
# BC status: pending PO authorship (behavioral_contracts: [])
version: "1.0"
level: "L4"
producer: story-writer
timestamp: "2026-09-08T00:00:00Z"
tdd_mode: facade
# tdd_mode: facade — this story modifies a GitHub Actions workflow YAML file (release.yml)
# and installs a CLI tool (git-cliff) inside the nightly branch of the publish-release job.
# No Rust production code is modified. crates_touched: []. Verification is via structural
# grep/inspection checks + a live nightly run (AC-008 human-in-loop gate).
# Mutation testing at wave gate replaces Red Gate density check per BC-8.30.001.
subsystems: [SS-22]
# Subsystem anchor justification:
#   SS-22 (Process Lifecycle) governs the release build pipeline and binary distribution
#   channels per ARCH-INDEX Subsystem Registry. The nightly release notes improvement is
#   an enhancement to the nightly channel within that pipeline. No other subsystem owns CI
#   release note generation logic.
crates_touched: []
# crates_touched: [] — release.yml is a CI artifact only. No Rust crate source is modified.
target_module: devops
capabilities: []
behavioral_contracts: []
# BC status: N/A — nightly release notes generation is release-infrastructure governance.
# No subsystem behavioral contract governs CI changelog generation for nightly builds.
# Authority is RELEASE-CHANNELS.md §2/§3/§5 (human-approved strategy doc) and ADR-063
# §D1/§D3 (git-cliff version pin and categorization convention).
# POL-14 NO-OP (behavioral_contracts: []).
verification_properties: []
depends_on: [S-REL-NIGHTLY-001, S-REL-CLIFF-001]
# Dependency anchor justifications:
#   depends_on S-REL-NIGHTLY-001: nightly.yml (the lane that pushes the dated nightly tag
#     and triggers release.yml) must be operational before the release.yml nightly branch
#     can be exercised. Without the lane, no nightly tags land in release.yml.
#   depends_on S-REL-CLIFF-001: cliff.toml at repo root is the authoritative configuration
#     for git-cliff categorization (commit_parsers, tag_pattern, body template). This story
#     installs git-cliff in the nightly path and invokes it using cliff.toml — both the
#     tool and the config must exist before the nightly notes story can deliver.
blocks: []
# blocks: [] — nightly notes enhancement is an own independent track. No product stories
# depend on nightly release bodies being categorized. Nightly continues to publish via the
# canned fallback even if this story is not delivered.
points: 3
estimated_days: 1
risk: LOW
# Risk justification: the nightly path and the stable/beta/rc path are clearly separated
# by the nightly regex guard in the existing "Extract release notes" step. The change
# adds steps to the nightly branch ONLY; the stable/beta/rc hard-fail path is untouched.
# The highest-risk element is the regex anchor guard (must stay exact-form to prevent
# hybrid-tag bypass — AC-006 explicitly verifies this). The canned fallback ensures no
# nightly is ever blocked from publishing.
acceptance_criteria_count: 8
red_gate_tests: 0
# red_gate_tests: 0 — tdd_mode: facade. No Rust production code; no Red Gate.
# Quality gate: structural grep/inspection checks (AC-001..AC-007) + live nightly run (AC-008).
estimated_passes: "1-2 LOCAL adversary passes"
holdout_scenarios: []
# HOLDOUT-N/A: tdd_mode=facade; crates_touched=[]; no built prism binary; no MCP-visible
# surface. This story delivers modifications to .github/workflows/release.yml (a GitHub
# Actions YAML file). All verification surfaces are structural: AC-001 through AC-007 are
# grep/inspection checks on the workflow file. AC-008 (live verification) is a CI nightly
# run. No MCP stdio wire-level assertions are possible or applicable.
# Consistent with S-REL-NIGHTLY-001, S-REL-CLIFF-001, S-REL-DROP-INTEL-MAC-001
# holdout determinations.
assumption_validations: []
risk_mitigations:
  - "The nightly regex guard in the 'Extract release notes' step MUST remain exact-form:
    `^v[0-9]+\\.[0-9]+\\.[0-9]+-nightly\\.[0-9]{8}(\\.[0-9]+)?$`. This is the
    bypass-guard that prevents a hybrid tag like v1.0.0-alpha.1-nightly.X from entering
    the nightly branch and bypassing the curated-CHANGELOG gate. Any modification to this
    regex must preserve both start- and end-anchors. AC-006 verifies the pattern is intact."
  - "The stable/beta/rc hard-fail (RELEASE-NOTES-MISSING exit 1 when CHANGELOG section is
    absent) MUST remain completely untouched by this story. Both the awk extraction block
    and the empty-file guard (if [[ ! -s ]]) in the non-nightly branch are preserved
    exactly as committed. AC-005 verifies via grep that both landmarks are still present."
  - "Canned fallback is MANDATORY: any nightly must still publish even if git-cliff exits
    non-zero or produces empty/whitespace-only output. The fallback writes the existing
    canned '## Nightly build' body. This ensures no nightly release is blocked by a
    git-cliff failure. AC-004 verifies the fallback branch is present."
  - "Full commit history is REQUIRED for git-cliff --latest to work. The publish-release
    job checkout is shallow (fetch-depth: 1 by default). A conditional step must run
    `git fetch --unshallow --tags` for nightly tags before git-cliff is invoked. Without
    this, git-cliff cannot walk the tag range and will silently produce empty output,
    which triggers the fallback. AC-002 verifies the unshallow step is present and gated."
  - "git-cliff version pin: 2.14.1 MUST be pinned exactly, matching ADR-063 D1 and
    cliff.toml authority. The install mechanism (taiki-e/install-action) must also carry a
    SHA-pinned action reference per the project's CWE-494 pin discipline. The devops-engineer
    MUST look up and pin the commit SHA for taiki-e/install-action at the v2 tag during
    implementation. AC-001 verifies both pins."
  - "git-cliff --latest invocation: `--latest` renders commits between the two most recent
    tag_pattern-matching tags. In the nightly context, this is the window from the previous
    nightly (or previous release if first nightly of a cycle) to the current nightly tag.
    The cliff.toml tag_pattern 'v[0-9].*' matches nightly tags (they start with v1.0.0-nightly.),
    so nightly tags participate in the tag universe. If only one tag exists (very first nightly
    ever), --latest renders all commits to that tag; this may be long but is valid and does not
    trigger the empty-output fallback. Empty-output fallback triggers when: (a) all commits in
    the range are skip-rule commits (docs/ci/test/chore/style/build/revert only), or
    (b) git-cliff exits non-zero."
  - "Supply-chain note: both the action SHA pin (taiki-e/install-action) and the
    git-cliff version pin (2.14.1) must be explicitly verified during implementation. The
    devops-engineer must NOT use a floating @v2 tag without pinning to a commit SHA, per the
    project's actions hardening convention (all existing actions in release.yml use commit
    SHA pins, not tag pointers)."
inputs:
  - ".github/workflows/release.yml"
  - "cliff.toml"
  - "docs/RELEASE-CHANNELS.md"
  - ".github/workflows/nightly.yml"
input-hash: "[pending-recompute]"
traces_to: []
cycle: "v1.0.0-nightly-lane"
phase: "3"
---

# S-REL-NIGHTLY-NOTES-001 — Nightly Release Notes: git-cliff Categorized Changelog

**Story ID:** S-REL-NIGHTLY-NOTES-001
**Status:** draft
**Version:** v1.0
**Wave:** F-A
**Priority:** P2
**Points:** 3
**Own-track note:** This story is an INDEPENDENT TRACK (own independent track per D-2499).
It enhances the nightly channel's release body quality without gating beta.2 or any product
story. Nightly continues to publish with the canned fallback body until this story merges.

---

## Origin

The nightly channel (`nightly.yml`, S-REL-NIGHTLY-001) publishes to GitHub Releases via
`release.yml`. The `publish-release` job's "Extract release notes" step currently emits a
canned body for nightly tags:

```
## Nightly build

Nightly pre-release build of `develop@${SHORT_SHA}` — ${DATE_UTC} (UTC).
...
```

This is intentional by design (MED-2 Option B comment in release.yml): `git-cliff` was not
installed in the publish-release job, and the checkout used `fetch-depth: 1` (shallow clone
with no tag history). The canned body was the correct declared unconditional output at that
time.

S-REL-CLIFF-001 has since landed `cliff.toml` at the repo root with Conventional Commits
categorization (Added/Fixed/Performance/Changed/Security) and PR link injection. The
nightly channel can now produce categorized output rather than the canned summary, without
any changes to the stable/beta/rc curated-CHANGELOG path.

This story modifies `release.yml` (nightly branch ONLY) to:
1. Obtain full commit history and all tags before git-cliff runs.
2. Install git-cliff 2.14.1 (pinned) in the publish-release job.
3. Replace the canned nightly body with `git cliff --latest` categorized output.
4. Fall back to the canned body on empty output or any error (robustness).

The stable/beta/rc curated-CHANGELOG path and the exact-form nightly regex bypass-guard
are preserved completely unchanged.

---

## Narrative

As a release engineer, I want nightly GitHub Release bodies to show a categorized changelog
of `feat:/fix:/perf:/refactor:/security:` commits in the nightly window (using `git cliff
--latest` with `cliff.toml`'s existing configuration), so that operators tracking `develop`
can see what changed at a glance rather than navigating to a separate CHANGELOG link.

---

## Authority

- RELEASE-CHANNELS.md §2 — Nightly channel definition; no CHANGELOG required; generated
  body is the declared output format for nightly builds.
- RELEASE-CHANNELS.md §5 — Build path: release.yml is the tag-triggered build and publish
  workflow; nightly tags trigger it unchanged.
- ADR-063 §D1 — git-cliff 2.14.1 version pin (canonical changelog assembler).
- ADR-063 §D3 — cliff.toml categorization convention; the existing cliff.toml is the
  configuration authority; this story does NOT modify cliff.toml.

(No BC: CI release notes generation; no subsystem behavioral contract. Authority is
RELEASE-CHANNELS.md and ADR-063 as above.)

---

## Behavioral Contracts

This story has no subsystem behavioral contracts. Authority is RELEASE-CHANNELS.md §2/§5
and ADR-063 §D1/§D3.

| Architecture Source | Clause |
|---------------------|--------|
| RELEASE-CHANNELS.md §2 | Nightly channel: no curated CHANGELOG required; machine-generated body is the declared, unconditional output format for nightly releases |
| RELEASE-CHANNELS.md §5 | Build path: full 4-target matrix for every channel; release.yml triggered by any `v*` tag push; nightly tags trigger it |
| ADR-063 §D1 | git-cliff 2.14.1 pinned — mandatory for any git-cliff invocation in CI |
| ADR-063 §D3 | cliff.toml at repo root is the authoritative config; tag_pattern `v[0-9].*` matches nightly tags; nightly tags participate in the tag universe; `--latest` gives the window between the two most recent matching tags |

---

## Token Budget Estimate

| Artifact | Estimated Tokens |
|----------|-----------------|
| This story spec | ~4,000 |
| `.github/workflows/release.yml` (full) | ~5,000 |
| `cliff.toml` (full) | ~1,000 |
| `docs/RELEASE-CHANNELS.md` §2/§3/§5 (relevant sections) | ~2,000 |
| `.github/workflows/nightly.yml` (structural reference for regex) | ~2,500 |
| Total | ~14,500 |

Well within the 30% context window budget.

---

## Red Gate Test List (SAC-1)

**tdd_mode: facade — Red Gate is N/A.** No Rust production code; no Red Gate tests.
Verification is via structural grep/inspection checks the implementer explicitly performs
before the PR:

- AC-001: taiki-e/install-action SHA-pinned; `tool: git-cliff@2.14.1` present; gated on nightly regex
- AC-002: Unshallow/full-history step present and gated on nightly regex
- AC-003: Nightly path invokes `git cliff --latest`; output written to `$NOTES_FILE`
- AC-004: Fallback to canned body on empty output or git-cliff error present
- AC-005: RELEASE-NOTES-MISSING hard-fail block and awk extraction unchanged in non-nightly branch
- AC-006: Exact-form nightly regex `^v[0-9]+\.[0-9]+\.[0-9]+-nightly\.[0-9]{8}(\.[0-9]+)?$` intact
- AC-007: `actionlint .github/workflows/release.yml` exits 0 on modified file
- AC-008: Live verification — human-in-loop gate at Phase-verify

---

## Tasks

1. **Read `.github/workflows/release.yml` in full** to understand the `publish-release` job
   structure, the existing checkout step, and the "Extract release notes" step's nightly and
   non-nightly branches. Note the exact bash variable names (`TAG`, `NOTES_FILE`, `VERSION`,
   `SHORT_SHA`, `DATE_UTC`) used in the step.

2. **Read `cliff.toml`** to confirm `tag_pattern = "v[0-9].*"` (nightly tags match),
   `sort_commits = "oldest"`, and the commit_parsers skip rules (docs/ci/test/chore/style/
   build/revert are all `skip = true`). No changes to cliff.toml are made by this story.

3. **Read `.github/workflows/nightly.yml`** to confirm the exact-form nightly regex used
   in the retention and "Extract release notes" steps. The new steps in release.yml must
   use the identical pattern: `^v[0-9]+\.[0-9]+\.[0-9]+-nightly\.[0-9]{8}(\.[0-9]+)?$`.

4. **Look up the current commit SHA for taiki-e/install-action at the v2 tag.** This is
   REQUIRED before writing any new `uses:` line. Use `gh api
   repos/taiki-e/install-action/git/ref/tags/v2 --jq .object.sha` or equivalent to get the
   current SHA. The SHA MUST be pinned; a floating `@v2` reference is not acceptable per the
   project's CWE-494 actions hardening discipline. All existing actions in release.yml use
   commit SHA pins. Pin git-cliff version to `2.14.1` in the `tool:` field.

5. **Add a "Fetch full history and tags (nightly path)" step** in the `publish-release` job,
   positioned BEFORE the "Extract release notes" step. Gate it with:
   ```yaml
   if: startsWith(env.TAG, 'v') && contains(env.TAG, '-nightly.')
   ```
   or via an `if: env.TAG =~ ...` bash expression inside the step body (verify which form
   actionlint accepts). The step body:
   ```bash
   set -euo pipefail
   git fetch --unshallow --tags 2>/dev/null || git fetch --tags
   ```
   The `|| git fetch --tags` fallback handles the case where the repo is already unshallowed
   (non-fatal: `--unshallow` on a full clone exits non-zero). For nightly builds, full history
   is required so `git cliff --latest` can walk the tag range.

6. **Add an "Install git-cliff (nightly path)" step** in the `publish-release` job, positioned
   AFTER the history-fetch step and BEFORE the "Extract release notes" step. Gate it with the
   same `if:` condition as step 5. Use:
   ```yaml
   uses: taiki-e/install-action@[SHA-FROM-STEP-4]  # v2
   with:
     tool: git-cliff@2.14.1
   ```
   This downloads a pre-built git-cliff binary (~10s) rather than compiling from source.
   The SHA-pinned action reference satisfies supply-chain requirements (CWE-494).

7. **Modify the nightly branch of the "Extract release notes" step.** The current nightly
   branch writes the canned body unconditionally. Replace it with:

   ```bash
   # ----- NIGHTLY CHANNEL: git-cliff categorized output (fallback: canned body) -----
   # git-cliff --latest renders commits between the two most recent tag_pattern-matching
   # tags (cliff.toml tag_pattern "v[0-9].*" matches nightly tags). This gives the
   # commit window from prev_tag to the current nightly tag.
   # Fallback: canned "## Nightly build" body when git-cliff exits non-zero or output
   # is empty/whitespace-only. A nightly MUST always publish.
   SHORT_SHA="$(git rev-parse --short HEAD)"
   DATE_UTC="$(date -u +%Y-%m-%d)"
   CLIFF_OUTPUT="$(git cliff --latest 2>/dev/null || true)"
   if [[ -n "$(echo "${CLIFF_OUTPUT}" | tr -d '[:space:]')" ]]; then
     echo "${CLIFF_OUTPUT}" > "$NOTES_FILE"
     echo "Nightly notes: git-cliff categorized output ($(wc -l < "$NOTES_FILE") lines) for ${TAG}"
   else
     # Fallback: git-cliff produced empty output (all skip-rule commits) or errored.
     {
       echo "## Nightly build"
       echo ""
       echo "Nightly pre-release build of \`develop@${SHORT_SHA}\` — ${DATE_UTC} (UTC)."
       echo ""
       echo "Unreleased changes; see [CHANGELOG \[Unreleased\]](../../blob/develop/CHANGELOG.md) for ongoing development notes."
     } > "$NOTES_FILE"
     echo "Nightly notes: canned fallback body (git-cliff empty or error) for ${TAG}"
   fi
   ```

   The `NOTES_FILE` variable is already set by the step's `mktemp` call earlier in the
   step body; reuse it for the nightly branch output. `SHORT_SHA` and `DATE_UTC` remain
   available as local variables for the fallback body.

   The non-nightly branch (stable/beta/rc awk extraction + RELEASE-NOTES-MISSING hard-fail)
   is NOT touched. The exact-form nightly `if:` regex guard at the top of the step is NOT
   touched.

8. **Verify AC-001..AC-007 explicitly.** For each AC, run the specified grep command and
   record the result. Document the verification results in the PR description. The nightly
   regex pattern verification (AC-006) is especially critical — compare character-by-character
   against the pattern in nightly.yml's retention step.

9. **Run `actionlint .github/workflows/release.yml`** (AC-007). Install actionlint if not
   present (`go install github.com/rhysd/actionlint/cmd/actionlint@latest` or via brew).
   Fix any actionlint findings before pushing. The modified release.yml must be actionlint-clean.

---

## Acceptance Criteria

### AC-001: git-cliff install step is present, SHA-pinned, and version-pinned to 2.14.1
An install step using `taiki-e/install-action` exists in the `publish-release` job with:
- `uses: taiki-e/install-action@<40-char-commit-SHA>` (NOT a floating `@v2` tag reference)
- `tool: git-cliff@2.14.1`
- An `if:` condition that gates the step to nightly tags only

`grep 'install-action' .github/workflows/release.yml` returns a line containing a 40-character
SHA. `grep 'git-cliff@2.14.1' .github/workflows/release.yml` returns a match.
(traces to ADR-063 §D1 — git-cliff 2.14.1 version pin; RELEASE-CHANNELS.md §5 — CWE-494
supply-chain discipline for all CI action references)

### AC-002: Full history fetch step is present and gated to nightly path
A step that runs `git fetch --unshallow --tags` (or equivalent) exists in the `publish-release`
job, positioned before the "Extract release notes" step, and is gated with an `if:` condition
matching the nightly tag pattern (contains `-nightly.`).
`grep 'unshallow' .github/workflows/release.yml` returns a match inside the `publish-release`
job section. The step does NOT run for stable/beta/rc tags.
(traces to RELEASE-CHANNELS.md §5 — git-cliff --latest requires full tag history that the
default shallow checkout cannot provide)

### AC-003: Nightly path invokes `git cliff --latest` and writes to `$NOTES_FILE`
The nightly branch of the "Extract release notes" step contains `git cliff --latest`. The
output is captured and written to `$NOTES_FILE` when non-empty.
`grep 'git cliff --latest' .github/workflows/release.yml` returns a match inside the
nightly branch of the "Extract release notes" step. The `$NOTES_FILE` variable (the same
mktemp-created file used for stable/beta notes) is used as the output destination.
(traces to ADR-063 §D3 — cliff.toml tag_pattern "v[0-9].*" includes nightly tags; --latest
gives the prev_tag..current_nightly_tag commit window)

### AC-004: Canned fallback is present for empty/error git-cliff output
The nightly branch contains a fallback that writes the existing canned "## Nightly build"
body to `$NOTES_FILE` when `git cliff --latest` either exits non-zero or produces empty
whitespace-only output.
`grep 'Nightly build' .github/workflows/release.yml` returns a match inside the nightly
branch fallback (not as the unconditional body it was before this story).
`grep 'git cliff --latest' .github/workflows/release.yml | grep -c 'fallback\|canned\|empty'`
OR a surrounding conditional block is present. The fallback body retains the original phrasing
(`develop@${SHORT_SHA}`, CHANGELOG Unreleased link) exactly.
(traces to RELEASE-CHANNELS.md §2 — canned summary body is the declared unconditional
output when git-cliff cannot produce categorized output; a nightly MUST always publish)

### AC-005: Stable/beta/rc curated-CHANGELOG path and RELEASE-NOTES-MISSING hard-fail are unchanged
The non-nightly branch of "Extract release notes" is identical to its pre-story form:
- The `awk` extraction block (extracts `## [VERSION] - DATE` section from CHANGELOG.md)
  is present and unmodified.
- The `if [[ ! -s "$NOTES_FILE" ]]` empty-file guard with `exit 1` and the
  `RELEASE-NOTES-MISSING:` error message is present and unmodified.
`grep 'RELEASE-NOTES-MISSING' .github/workflows/release.yml` returns exactly 1 match.
`grep 'awk.*ver.*CHANGELOG' .github/workflows/release.yml` returns a match (awk extraction).
`grep 'sed.*\\$!d' .github/workflows/release.yml` returns a match (leading-blank-line strip).
(traces to RELEASE-CHANNELS.md §2 — stable/beta/rc channels require curated CHANGELOG with
hard-fail; this story preserves that path unchanged)

### AC-006: Exact-form nightly regex bypass-guard is intact in the modified release.yml
The `if [[ "$TAG" =~ ... ]]` nightly regex at the top of the "Extract release notes" step
body is unchanged from its pre-story form:
`^v[0-9]+\.[0-9]+\.[0-9]+-nightly\.[0-9]{8}(\.[0-9]+)?$`
Both the `^` start-anchor and the `$` end-anchor are present. The optional same-day suffix
`(\.[0-9]+)?` group is present. The pattern matches `v1.0.0-nightly.20260908` and
`v1.0.0-nightly.20260908.2` but NOT `v1.0.0-alpha.1-nightly.X` (hybrid bypass guard).
`grep 'nightly\.\[0-9\]' .github/workflows/release.yml | grep -c '\\$'` ≥ 1 (end-anchor
present on the nightly regex line).
(traces to RELEASE-CHANNELS.md §2 — exact-form nightly regex prevents hybrid-tag bypass of
the curated-CHANGELOG gate; preserving it exactly is a correctness requirement)

### AC-007: actionlint exits 0 on the modified release.yml
`actionlint .github/workflows/release.yml` produces zero errors and exits 0. Any expression
syntax, step structure, or secret reference issues introduced by the new steps are resolved
before the PR is pushed.
(traces to RELEASE-CHANNELS.md §5 — release.yml is a production workflow; structural
correctness is required)

### AC-008: Live verification — next nightly release body shows categorized sections
After the PR merges to develop and the next nightly.yml run fires (scheduled or
`workflow_dispatch`), the resulting GitHub Release body for the dated nightly tag shows
at least one of the `### Added`, `### Fixed`, `### Performance`, `### Changed`, or
`### Security` section headers from git-cliff output (indicating categorized output, not
the canned summary). If all commits since the previous tag are skip-rule commits, the
canned fallback body is acceptable and the AC is satisfied by verifying the fallback path
works correctly.

This AC is satisfied at the Phase-verify step (human-in-loop gate). The implementer does
NOT need to satisfy this AC before pushing the PR.
(traces to RELEASE-CHANNELS.md §2 — nightly release body reflects develop state; git-cliff
output confirms the nightly path is functioning end-to-end)

---

## Previous Story Intelligence

| Story | Key Decisions | Patterns Established | Gotchas Discovered |
|-------|--------------|---------------------|-------------------|
| S-REL-NIGHTLY-001 | nightly.yml uses `fetch-depth: 0` for full history; publish-release job checkout is separate and shallow | PAT (RELEASE_PROMOTE_TOKEN) mechanism; nightly regex exact form `^v[0-9]+...nightly...$` established | publish-release checkout is independent from nightly.yml checkout — must unshallow separately in release.yml |
| S-REL-CLIFF-001 | cliff.toml at repo root; `--latest` invocation; ADR-063 §D1 version pin 2.14.1; `tag_pattern = "v[0-9].*"` matches nightly tags | git-cliff `--latest` = commits between two most recent tags; numbered group prefix + striptags+trim chain for section headers; canned fallback pattern for empty output | `regex_replace` is NOT a built-in Tera filter in git-cliff 2.14.1; `--output` alongside `--prepend` causes duplicate sections; `{ breaking = true }` parser-level group NOT effective |
| S-REL-BETA1-NOTES-001 | ADR-063 §D6 noise-control; skip rules in cliff.toml already suppress docs/ci/test/chore | cliff.toml skip rules are tuned; no further skip-rule changes needed for nightly | N/A — completed before this story |

Read `release.yml` §publish-release §"Extract release notes" in full before modifying it.
The `NOTES_FILE` variable is created via `mktemp` early in the step and reused throughout
both branches — do not re-create it in the nightly branch.

The `--latest` flag in git-cliff requires at least one prior tag to exist. On the very first
nightly tag ever (no prior tag in any channel), git-cliff's `--latest` behavior shows commits
from the initial commit to the first tag — potentially large output. This is NOT an error
condition; the fallback triggers only on empty output or exit non-zero.

---

## Architecture Compliance Rules

| Rule | Source | Enforcement |
|------|--------|-------------|
| git-cliff 2.14.1 pinned exactly | ADR-063 §D1 | AC-001 grep: `git-cliff@2.14.1` in install step |
| All CI action references use commit SHA pins (not tag pointers) | project CI hardening convention (release.yml existing actions) | AC-001: taiki-e/install-action uses 40-char SHA |
| Nightly regex bypass-guard intact (exact-form start+end anchors) | RELEASE-CHANNELS.md §2 | AC-006: grep for anchored regex on nightly branch if-condition |
| Stable/beta/rc hard-fail path (RELEASE-NOTES-MISSING) unchanged | RELEASE-CHANNELS.md §2 | AC-005: RELEASE-NOTES-MISSING grep exactly 1 match |
| Canned fallback ensures nightly always publishes | RELEASE-CHANNELS.md §2 | AC-004: fallback branch present with canned body |
| release.yml actionlint-clean | project CI quality standard | AC-007: actionlint exits 0 |
| cliff.toml is NOT modified by this story | ADR-063 §D3 — cliff.toml is authoritative; story does not adjust categorization rules | File structure: cliff.toml listed as DO NOT modify |
| Full tag history required before git-cliff --latest | ADR-063 §D3 — --latest walks tag range | AC-002: fetch --unshallow step present and gated |

---

## Library & Framework Requirements

| Dependency | Version | Notes |
|------------|---------|-------|
| `git-cliff` | `2.14.1` | Installed via `taiki-e/install-action`; exact version per ADR-063 §D1. Pre-built binary download (~10s vs 3-5 min cargo compile). |
| `taiki-e/install-action` | `@v2` commit SHA — MUST be pinned by devops-engineer during implementation | Pre-built binary installer for CI tools; used for fast git-cliff install. SHA determined via `gh api repos/taiki-e/install-action/git/ref/tags/v2 --jq .object.sha`. |
| `cliff.toml` | existing at repo root (S-REL-CLIFF-001) | No version; the existing file is the config authority. Not modified. |
| `git fetch --unshallow` | standard git (pre-installed on ubuntu-latest) | Required to expand the shallow checkout before git-cliff walks tag history. |

---

## File Structure Requirements

| File | Action | Notes |
|------|--------|-------|
| `.github/workflows/release.yml` | Modify | Add 2 new steps (unshallow + install) and modify nightly branch of "Extract release notes" step only |
| `cliff.toml` | DO NOT modify | cliff.toml is the authoritative ADR-063 §D3 config; categorization rules are already correct for nightly output |
| `.github/workflows/nightly.yml` | DO NOT modify | Nightly tag push mechanism is unchanged; the improvement is entirely in release.yml |
| `docs/RELEASE-CHANNELS.md` | Verify only | No status change required; §2 nightly row already reads IMPLEMENTED; release notes quality is a runtime behavior, not a channel-model status update |

---

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| `release.yml` §publish-release (nightly path steps) | `.github/workflows/` | Effectful (CI workflow; installs tools; writes release notes file; creates GitHub Release) |
| `cliff.toml` | repo root | Pure (static config; referenced but not modified) |

---

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| `.github/workflows/release.yml` (nightly path modifications) | effectful-shell | CI workflow that installs tools, fetches history, generates notes, and publishes a GitHub Release |
| `cliff.toml` | pure-core | Static TOML configuration; no runtime I/O; not modified by this story |

---

## Holdout Applicability

**Determination: N/A**

The story-level holdout gate (CLAUDE.md, human-approved 2026-07-13) requires a built
prism binary and an MCP-visible surface with wire-level assertions. This story has
`tdd_mode: facade` and `crates_touched: []` — it produces no prism binary. Its entire
deliverable is a modification to `.github/workflows/release.yml` (a GitHub Actions YAML
file).

All verification surfaces are structural: AC-001 through AC-007 are grep/inspection checks
on the workflow file. AC-008 (live verification) is a CI run on the next nightly that verifies
the categorized output appears in the GitHub Release body. The git-cliff output is rendered
by GitHub Actions on CI runners — not the prism MCP binary — so no MCP stdio wire-level
assertions are possible or applicable.

Consistent with prior infra-story determinations for S-REL-NIGHTLY-001, S-REL-CLIFF-001,
S-REL-DROP-INTEL-MAC-001, S-REL-WRITER-001 (all holdout_scenarios: []).

`holdout_scenarios: []` — no scenarios authored; HOLDOUT-INDEX unchanged.

---

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | All commits since the previous tag are skip-rule commits (docs/ci/chore/test/style/build/revert only) | git-cliff --latest produces empty output; fallback writes canned "## Nightly build" body; nightly publishes normally |
| EC-002 | git-cliff exits non-zero (binary missing, cliff.toml parse error, or unexpected CLI failure) | `git cliff --latest` runs with stderr discarded; the `true` appended on the non-zero exit path captures an empty string; the empty-output guard triggers the canned fallback body; nightly publishes normally |
| EC-003 | First-ever nightly tag (no prior tag in any channel) | git-cliff --latest shows commits from initial commit to first tag — potentially large output. Non-empty whitespace check passes; categorized output is written to NOTES_FILE; nightly publishes with full history categorized. No truncation is applied — this is acceptable for a first nightly. |
| EC-004 | Previous tag is a stable/beta/rc (not a nightly) | cliff.toml tag_pattern "v[0-9].*" matches all version tags including stable and pre-release. git-cliff --latest gives commits from prev_stable_or_beta..current_nightly. This is correct — shows everything new since the last formal release. |
| EC-005 | Same-day second nightly (vX.Y.Z-nightly.YYYYMMDD.2) | git-cliff --latest gives commits between the first nightly of the day (vX.Y.Z-nightly.YYYYMMDD) and the .2 nightly. This is correct behavior — shows only the delta since the morning nightly. |
| EC-006 | Shallow checkout was already unshallowed by a prior step | `git fetch --unshallow --tags` with stderr redirected to /dev/null; if it exits non-zero (already unshallowed), `git fetch --tags` runs as the error-path fallback; the step does not fail in either case |
| EC-007 | A hybrid tag like v1.0.0-alpha.1-nightly.X attempts to use the nightly path | The exact-form nightly regex `^v[0-9]+\.[0-9]+\.[0-9]+-nightly\.[0-9]{8}(\.[0-9]+)?$` does NOT match this pattern (end-anchor `$` rejects the alpha.1 prefix). The tag falls into the non-nightly branch where the awk CHANGELOG extraction runs; RELEASE-NOTES-MISSING fires if no CHANGELOG section exists. Bypass attempt is blocked. |

---

## Changelog

| Version | Date | Author | Change |
|---------|------|--------|--------|
| 1.0 | 2026-09-08 | story-writer | Initial materialization — RELEASE-CHANNELS.md §2/§5; ADR-063 §D1/§D3; git-cliff --latest design; taiki-e/install-action + unshallow pattern; canned fallback; bypass-guard preservation |
