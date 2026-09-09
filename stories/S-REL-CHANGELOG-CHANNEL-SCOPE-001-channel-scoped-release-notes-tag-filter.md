---
document_type: story
story_id: S-REL-CHANGELOG-CHANNEL-SCOPE-001
title: "devops: channel-scoped release notes — per-channel git-cliff tag-pattern filter for nightly/alpha/beta/rc/stable lanes"
wave: F-A
epic_id: E-REL
priority: P1
# P1 justification: MUST land BEFORE v1.0.0 stable is cut.
# The stable release-prep.yml git-cliff invocation currently uses tag_pattern = "v[0-9].*"
# (global; from cliff.toml), which includes nightly/alpha/beta tags in the tag universe.
# If nightly or pre-release tags sit between the last stable and the new stable tag, git-cliff
# treats those commits as "already released" and silently drops them from the stable CHANGELOG.
# Precedent: v1.0.0-beta.2 CHANGELOG required a manual --tag-pattern workaround for exactly this
# reason (S-REL-SPECS-TARBALL-001 release cascade, 2026-09-09 — see §Origin).
# This story generalises that workaround into automatic per-channel tag filtering on all lanes.
status: ready
version: "1.2"
level: "L4"
producer: story-writer
timestamp: "2026-09-09T00:00:00Z"
tdd_mode: facade
# tdd_mode: facade — this story modifies GitHub Actions workflow YAML files
# (release.yml, release-prep.yml, release-tag.yml) and documentation markdown files
# (RELEASE-CHANNELS.md, RELEASING.md). No Rust production code is modified.
# crates_touched: []. Verification is via structural grep/inspection checks
# (AC-001..AC-009) + per-channel live-run verification (AC-010).
# Mutation testing at wave gate replaces Red Gate density check per BC-8.30.001.
subsystems: [SS-22]
# Subsystem anchor justification:
#   SS-22 (Process Lifecycle) governs the release build pipeline and binary distribution
#   channels per ARCH-INDEX Subsystem Registry. Channel-scoped CHANGELOG generation is
#   an enhancement to the release pipeline within that subsystem.
crates_touched: []
# crates_touched: [] — all changes are CI workflow YAML and documentation. No Rust source modified.
target_module: devops
capabilities: []
behavioral_contracts: []
# BC status: N/A — release tooling / CI workflow governance.
# No subsystem behavioral contract governs channel-scoped git-cliff tag filtering.
# Authority is RELEASE-CHANNELS.md §3 (channel model + BASE-MATCH) and
# ADR-063 §D3/§D5/§D7 (git-cliff config, release-prep.yml integration, channel-scoped tag filtering).
# ADR-063 §D7 ACCEPTED v1.14 (2026-09-09): per-channel tag-pattern mapping, stable-floor rule,
# stable-exclusion invariant, and three workflow implementation sites are now normative.
# POL-14 NO-OP (behavioral_contracts: []).
verification_properties: []
depends_on: [S-REL-CLIFF-001, S-REL-NIGHTLY-NOTES-001]
# Dependency anchor justifications:
#   depends_on S-REL-CLIFF-001: cliff.toml at repo root (ADR-063 §D3) and the git-cliff
#     invocation in release-prep.yml Step 7 (ADR-063 §D5) must exist before this story
#     can extend them. S-REL-CLIFF-001 is merged (PR #264); dependency satisfied.
#   depends_on S-REL-NIGHTLY-NOTES-001: the git-cliff invocation in release.yml's nightly
#     path (`git cliff --latest`) must be established before this story adds --tag-pattern
#     to that invocation. S-REL-NIGHTLY-NOTES-001 is merged (PR #277); dependency satisfied.
blocks: []
# blocks: [] — no current story explicitly depends on this story's output.
# However, S-REL-VBUMP-001 (cargo-release stable bump entrypoint; draft) invokes
# git-cliff in its pre-release hook and will hit the same channel-scope bug unless this
# story lands first. The blocking relationship is noted in §V1.0-STABLE-DEPENDENCY.
points: 8
estimated_days: 2
risk: MEDIUM
# Risk justification: moderate — three workflow files are modified (release.yml,
# release-prep.yml, release-tag.yml), each in carefully guarded branches. The nightly
# bypass-guard (exact-form regex) must remain intact. The stable `--tag-pattern` must not
# accidentally exclude commits that should appear in the stable CHANGELOG. The
# per-channel patterns must be verified against real tag histories to avoid false-empty
# fallbacks. The highest-risk element is the stable-floor fallback design: if the
# pattern is wrong, the stable CHANGELOG silently omits commits between beta.N and stable.
acceptance_criteria_count: 10
red_gate_tests: 0
# red_gate_tests: 0 — tdd_mode: facade. No Rust production code; no Red Gate.
# Quality gate: structural grep/inspection checks (AC-001..AC-009) + per-channel live-run (AC-010).
estimated_passes: "2-3 LOCAL adversary passes"
holdout_scenarios: []
# HOLDOUT-N/A: tdd_mode=facade; crates_touched=[]; no built prism binary; no MCP-visible
# surface. All verification surfaces are structural: AC-001..AC-009 are grep/inspection
# checks on workflow files. AC-010 is a live verification per-channel run.
# Consistent with S-REL-NIGHTLY-NOTES-001, S-REL-CLIFF-001 holdout determinations.
assumption_validations: []
risk_mitigations:
  - "The exact-form nightly regex bypass-guard in release.yml ('Extract release notes' step)
    MUST remain intact: `^v[0-9]+\\.[0-9]+\\.[0-9]+-nightly\\.[0-9]{8}(\\.[0-9]+)?$`.
    Both `^` start-anchor and `$` end-anchor must be present. This guards against hybrid-tag
    bypass of the curated-CHANGELOG gate. The --tag-pattern modification to the nightly path
    must NOT alter the regex that selects which branch of the 'Extract release notes' step
    runs. AC-007 verifies the bypass-guard pattern is intact post-modification."
  - "The stable-floor fallback is implemented via pattern design: the per-channel
    --tag-pattern includes both the channel suffix AND the stable tag form (no suffix).
    Example for beta: `^v[0-9]+\\.[0-9]+\\.[0-9]+(-beta\\.[0-9]+)?$` matches both
    v1.0.0-beta.N and v1.0.0 (stable). When no prior beta tag exists, git-cliff falls
    back to the last stable tag as the delta floor. When no stable tag exists (new X.Y.Z
    line), git-cliff walks from the repo origin — correct per the requirement."
  - "The --tag-pattern flag must be verified to exist in git-cliff 2.14.1 and to override
    cliff.toml tag_pattern. The precedent (beta.2 CHANGELOG workaround) confirms the flag
    exists. However, the implementer MUST verify the flag semantics: does it replace
    cliff.toml tag_pattern entirely, or merge with it? The expected behavior is replacement.
    AC-001 verifies the correct flag form is present in each workflow invocation."
  - "The stable release-prep.yml path MUST use a stable-only pattern:
    `^v[0-9]+\\.[0-9]+\\.[0-9]+$` (no pre-release suffix). This ensures the stable CHANGELOG
    shows commits since the last stable release, not since the last nightly. If the pattern
    accidentally includes pre-release tags, commits between the last nightly/beta and stable
    would be dropped. AC-005 verifies the stable-only pattern."
  - "The release-tag.yml additions (git-cliff for alpha/beta/rc) MUST include the 3-substep
    flow (PRE-STRIP + git-cliff + LINK-REF-UPDATE) matching the release-prep.yml Step 7
    pattern (ADR-063 §D3 --prepend mechanism). Omitting PRE-STRIP causes masthead duplication
    on repeated CHANGELOG prepends. Omitting LINK-REF-UPDATE leaves stale compare-link refs.
    AC-003 verifies the 3-substep flow is present for release-tag.yml's pre-release CHANGELOG
    generation."
  - "ADR-063 §D7 amendment routing: DISCHARGED. ADR-063 §D7 was accepted (v1.14, 2026-09-09)
    before this story advanced to status: ready. The per-channel tag-pattern table, stable-floor
    fallback rule, and stable-exclusion invariant are now normative in §D7. Docs (AC-009) must
    reference 'ADR-063 §D7' (bare citation — no qualifier)."
  - "actionlint must pass on all three modified workflow files (release.yml, release-prep.yml,
    release-tag.yml). Any expression syntax issues from the added `--tag-pattern` arguments
    (bash variable interpolation inside YAML multi-line strings) must be resolved. AC-008
    verifies actionlint exits 0 on all three files."
inputs:
  - ".github/workflows/release.yml"
  - ".github/workflows/release-prep.yml"
  - ".github/workflows/release-tag.yml"
  - "cliff.toml"
  - "docs/RELEASE-CHANNELS.md"
  - "RELEASING.md"
input-hash: "[pending-recompute]"
traces_to: []
cycle: "v1.0.0-stable-prereq"
phase: "3"
---

# S-REL-CHANGELOG-CHANNEL-SCOPE-001 — Channel-Scoped Release Notes: Per-Channel git-cliff Tag Filter

**Story ID:** S-REL-CHANGELOG-CHANNEL-SCOPE-001
**Status:** ready
**Version:** v1.2
**Wave:** F-A
**Priority:** P1
**Points:** 8

---

## ADR-063 §D7 Amendment Status

**ACCEPTED — blocker discharged.** ADR-063 §D7 "Channel-Scoped Tag Filtering" was authored
by the architect and accepted as ADR-063 v1.14 (2026-09-09). The per-channel `--tag-pattern`
mapping, stable-floor fallback rule, stable-exclusion invariant, and three normative workflow
implementation sites are now codified in §D7.

**Context (retained for history):** Per-channel tag-pattern scoping was identified as a new
design decision within ADR-063's domain (git-cliff + cliff.toml architecture, §D3/§D5).
§D7 documents:
- The five-channel `--tag-pattern` regex table (nightly/alpha/beta/rc/stable)
- The stable-floor fallback rule via optional-group pattern design
- The stable-exclusion invariant (stable pattern MUST NOT carry an optional pre-release suffix)
- The three normative workflow implementation sites: release.yml nightly branch, release-prep.yml
  Step 7, and the new release-tag.yml CHANGELOG generation step

This story's §Authority now cites §D7 directly (bare citation; no qualifier).

---

## Origin

**Precedent (v1.0.0-beta.2 CHANGELOG generation, 2026-09-09):**

During the S-REL-SPECS-TARBALL-001 release cascade, generating the v1.0.0-beta.2 CHANGELOG
required a manual `--tag-pattern "^v[0-9]+\.[0-9]+\.[0-9]+(-beta\.[0-9]+)?$"` workaround.

The root cause: nightly tags `v1.0.0-nightly.20260909` and `v1.0.0-nightly.20260909.2`
sat on commits from PRs #277 and #278. The global `tag_pattern = "v[0-9].*"` in cliff.toml
meant git-cliff treated those commits as "already released under the nightly tags" — so
`git cliff --unreleased --tag v1.0.0-beta.2` silently omitted them from the beta.2 CHANGELOG.

The manual `--tag-pattern` restricted the tag universe to beta + stable tags only, making
git-cliff see those commits as unreleased relative to the beta line. This produced a correct
beta.2 CHANGELOG with those commits included.

**The general principle:**

Each release channel should show the delta since the previous release of the SAME channel:
- nightly → previous nightly (or last stable if no prior nightly exists)
- alpha → previous alpha (or last stable if no prior alpha exists)
- beta → previous beta (or last stable if no prior beta exists)
- rc → previous rc (or last stable if no prior rc exists)
- stable → previous stable

The stable-floor fallback is automatically implemented by the pattern design: including
stable tags (no pre-release suffix) in each per-channel `--tag-pattern` regex. When no
prior same-channel tag exists, git-cliff's "previous tag" is the most recent stable tag
in the pattern-filtered universe.

**Consistency with S-REL-NIGHTLY-NOTES-001:**

S-REL-NIGHTLY-NOTES-001 (merged PR #277) uses `git cliff --latest` in release.yml's
nightly path. `--latest` gives the range between the two most recent tags matching
`tag_pattern`. Since cliff.toml `tag_pattern = "v[0-9].*"` includes ALL version tags,
the "previous tag" for a nightly might be another nightly, or a beta, or an rc — whichever
was most recently pushed. Adding `--tag-pattern` to the nightly invocation ensures the
previous tag is always the previous nightly (or last stable if first nightly).

---

## V1.0-STABLE-DEPENDENCY

**This story MUST land before v1.0.0 stable is cut.**

The stable release-prep.yml git-cliff invocation at Step 7 currently uses the global
`tag_pattern = "v[0-9].*"`. When nightly tags and beta/rc tags exist between the last
stable and the new stable, commits under those nightly/beta tags are "claimed" by them and
silently dropped from the stable CHANGELOG. This would produce an INCOMPLETE stable
CHANGELOG for v1.0.0 — omitting all commits from the develop branch since beta.1 that
happened to be captured under nightly tags.

**Downstream dependency note:** S-REL-VBUMP-001 (cargo-release stable bump entrypoint;
draft) invokes git-cliff in its pre-release hook without `--tag-pattern`. It should depend
on this story (or include the channel-scoped pattern in its hook). This dependency must be
reconciled when S-REL-VBUMP-001 advances to `status: ready`.

---

## Narrative

As a release engineer, I want every release channel to automatically show the delta since
the previous release of the same channel — nightly-to-nightly, beta-to-beta, stable-to-stable
— so that release notes never silently omit commits that were captured under tags from other
channels, and the first release in any channel line correctly floors to the last stable release.

---

## Authority

- RELEASE-CHANNELS.md §3 — Channel model + BASE-MATCH semantics; nightly/alpha/beta/rc/stable
  each form a distinct tag series.
- RELEASE-CHANNELS.md §5 — Build path: release.yml is the tag-triggered build and publish
  workflow; all channels share it.
- ADR-063 §D1 — git-cliff 2.14.1 version pin (canonical changelog assembler).
- ADR-063 §D3 — cliff.toml at repo root is the authoritative config; `tag_pattern` is set
  globally; this story adds per-invocation `--tag-pattern` overrides per channel.
- ADR-063 §D5 — release-prep.yml integration point; this story extends it with
  per-channel `--tag-pattern` detection.
- **ADR-063 §D7** (v1.14, ACCEPTED 2026-09-09) — per-channel tag-scoping model: five-channel
  `--tag-pattern` regex table, stable-floor fallback rule, stable-exclusion invariant, and
  three normative workflow implementation sites.

(No BC: CI release tooling governance; no subsystem behavioral contract. POL-14 NO-OP.)

---

## Behavioral Contracts

This story has no subsystem behavioral contracts. Authority is RELEASE-CHANNELS.md §3/§5
and ADR-063 §D1/§D3/§D5/§D7 (§D7 ACCEPTED v1.14, 2026-09-09).

| Architecture Source | Clause |
|---------------------|--------|
| RELEASE-CHANNELS.md §3 | Each channel (nightly/alpha/beta/rc/stable) is a distinct tag series; versioning uses BASE-MATCH for pre-releases |
| RELEASE-CHANNELS.md §5 | release.yml triggered by any `v*` tag; nightly path (canned/git-cliff body); stable/beta/rc/alpha path (curated CHANGELOG awk extraction) |
| ADR-063 §D1 | git-cliff 2.14.1 pinned — mandatory for any git-cliff invocation in CI |
| ADR-063 §D3 | cliff.toml `tag_pattern = "v[0-9].*"` is the default; `--tag-pattern` CLI flag overrides it per invocation |
| ADR-063 §D5 | release-prep.yml Step 7 git-cliff invocation: `git cliff --tag "v${VERSION}" --unreleased --prepend CHANGELOG.md` — this story adds `--tag-pattern` detection |
| ADR-063 §D7 (v1.14) | Per-channel tag-pattern table (nightly/alpha/beta/rc/stable), stable-floor fallback rule, stable-exclusion invariant, and three normative workflow implementation sites; Site 1 no `2>/dev/null`; Site 3 uses taiki-e SHA pin `d438492...` + ordering MUST |

---

## Token Budget Estimate

| Artifact | Estimated Tokens |
|----------|-----------------|
| This story spec | ~8,000 |
| `.github/workflows/release.yml` (full) | ~5,500 |
| `.github/workflows/release-prep.yml` (full) | ~5,000 |
| `.github/workflows/release-tag.yml` (full) | ~3,500 |
| `cliff.toml` (full) | ~1,000 |
| `docs/RELEASE-CHANNELS.md` §3/§5 (relevant) | ~2,500 |
| `RELEASING.md` §5 (relevant) | ~2,000 |
| **Total** | **~27,500** |

Well within the 30% context window budget.

---

## Red Gate Test List (SAC-1)

**tdd_mode: facade — Red Gate is N/A.** No Rust production code; no Red Gate tests.
Verification is via structural grep/inspection checks the implementer explicitly performs
before the PR:

- AC-001: `--tag-pattern` flag present in nightly path (release.yml), stable path
  (release-prep.yml), and pre-release path (release-tag.yml)
- AC-002: nightly pattern covers nightly + stable; excludes alpha/beta/rc
- AC-003: alpha/beta/rc pattern (release-tag.yml) covers channel + stable; excludes nightly
  and other pre-release channels
- AC-004: stable pattern (release-prep.yml) covers stable-only; excludes all pre-releases
- AC-005: stable-floor fallback implicit in pattern design — verified by dry-run on
  "first beta in line" scenario (no prior beta tag; previous is stable)
- AC-006: nightly bypass-guard regex in release.yml unchanged
- AC-007: curated-CHANGELOG hard-fail (RELEASE-NOTES-MISSING) unchanged in release.yml
- AC-008: actionlint exits 0 on all three modified workflow files
- AC-009: docs updated (RELEASE-CHANNELS.md §5, RELEASING.md §5)
- AC-010: live verification per-channel (human-in-loop gate at Phase-verify)

---

## Tasks

1. **Read release.yml in full** — understand the `publish-release` job's "Extract release
   notes" step, both the nightly path (`git cliff --latest` from S-REL-NIGHTLY-NOTES-001)
   and the non-nightly path (awk CHANGELOG extraction). Note the `TAG` variable form and
   the `if [[ "$TAG" =~ ... ]]` nightly regex guard structure.

2. **Read release-prep.yml in full** — understand Step 7 "Generate CHANGELOG section
   (git-cliff)" including the current `git cliff --tag "v${VERSION}" --unreleased --prepend
   CHANGELOG.md` invocation in the "Generate CHANGELOG section (git-cliff)" step. Note the `VERSION` variable set earlier in the
   workflow (the `x.y.z` form without the `v` prefix). Note the 3-substep structure
   (PRE-STRIP + git-cliff + LINK-REF-UPDATE) per ADR-063 §D3.

3. **Read release-tag.yml in full** — understand the current workflow structure: version
   guard, tag creation, tag push. Identify where to INSERT the CHANGELOG generation step
   (before or after tag push), and which variable holds the full version tag (e.g.,
   `v1.0.0-beta.2`).

4. **Read cliff.toml** — verify `tag_pattern = "v[0-9].*"` (global default) and
   `ignore_tags = ""`. Confirm `--tag-pattern` CLI flag overrides `tag_pattern`. No
   changes to cliff.toml are required by this story.

5. **Design per-channel patterns.** The patterns must satisfy:
   - Each channel includes same-channel tags AND stable tags (stable = no pre-release suffix)
   - Each channel excludes tags from other pre-release channels
   - The `?` optional group provides the stable-floor fallback automatically

   Canonical patterns (verify against git-cliff 2.14.1 behavior before committing):

   | Channel | Pattern |
   |---------|---------|
   | nightly | `^v[0-9]+\.[0-9]+\.[0-9]+(-nightly\.[0-9]{8}(\.[0-9]+)?)?$` |
   | alpha | `^v[0-9]+\.[0-9]+\.[0-9]+(-alpha\.[0-9]+)?$` |
   | beta | `^v[0-9]+\.[0-9]+\.[0-9]+(-beta\.[0-9]+)?$` |
   | rc | `^v[0-9]+\.[0-9]+\.[0-9]+(-rc\.[0-9]+)?$` |
   | stable | `^v[0-9]+\.[0-9]+\.[0-9]+$` |

   **Verify the fallback:** For beta pattern `^v[0-9]+\.[0-9]+\.[0-9]+(-beta\.[0-9]+)?$`,
   test against a tag set `[v1.0.0-nightly.20260909, v1.0.0-nightly.20260909.2]` — none
   should match. Test against `[v1.0.0-beta.1, v1.0.0-nightly.20260909]` — only `v1.0.0-beta.1`
   should match. Test against `[v1.0.0]` (stable, no beta exists) — `v1.0.0` should match
   (stable-floor).

6. **Modify release.yml — nightly path only.** In the "Extract release notes (channel-aware)"
   step's nightly branch, the **only prescribed change** is inserting the `--tag-pattern`
   argument. The live invocation already has `--strip=header` and does NOT have `2>/dev/null`:
   ```bash
   # LIVE before:
   CLIFF_OUTPUT="$(git cliff --latest --strip=header || true)"
   ```
   Change to (§D7 Site 1 target):
   ```bash
   CLIFF_OUTPUT="$(git cliff --latest \
     --tag-pattern '^v[0-9]+\.[0-9]+\.[0-9]+(-nightly\.[0-9]{8}(\.[0-9]+)?)?$' \
     --strip=header || true)"
   ```
   Do NOT add `2>/dev/null` — the live step deliberately surfaces stderr for observability.
   The nightly regex bypass-guard at the top of the step (gating which branch runs) is
   NOT touched. The `--tag-pattern` change is inside the nightly branch body only.

7. **Modify release-prep.yml — Step 7 git-cliff invocation.** Add channel detection and
   per-channel `--tag-pattern`. The `VERSION` variable in release-prep.yml holds the semver
   without the `v` prefix (e.g., `1.0.0` or `1.0.0-beta.2`). Detect the channel suffix:

   ```bash
   # Detect channel for per-channel tag scoping (ADR-063 §D7)
   VERSION_TAG="v${VERSION}"
   if [[ "${VERSION}" =~ -nightly\. ]]; then
     TAG_PATTERN='^v[0-9]+\.[0-9]+\.[0-9]+(-nightly\.[0-9]{8}(\.[0-9]+)?)?$'
   elif [[ "${VERSION}" =~ -alpha\. ]]; then
     TAG_PATTERN='^v[0-9]+\.[0-9]+\.[0-9]+(-alpha\.[0-9]+)?$'
   elif [[ "${VERSION}" =~ -beta\. ]]; then
     TAG_PATTERN='^v[0-9]+\.[0-9]+\.[0-9]+(-beta\.[0-9]+)?$'
   elif [[ "${VERSION}" =~ -rc\. ]]; then
     TAG_PATTERN='^v[0-9]+\.[0-9]+\.[0-9]+(-rc\.[0-9]+)?$'
   else
     # stable: no pre-release suffix
     TAG_PATTERN='^v[0-9]+\.[0-9]+\.[0-9]+$'
   fi
   git cliff --tag "${VERSION_TAG}" --unreleased \
     --tag-pattern "${TAG_PATTERN}" \
     --prepend CHANGELOG.md
   ```

   The 3-substep structure (PRE-STRIP before, LINK-REF-UPDATE after) is unchanged.
   Only the git-cliff invocation line gains `--tag-pattern "${TAG_PATTERN}"`.

8. **Modify release-tag.yml — insert two new steps BEFORE the existing step 6 "Create
   annotated tag on develop HEAD" (§D7 Site 3).** The workflow currently creates the
   annotated tag at step 6 and pushes it at step 7, triggering release.yml which extracts
   the CHANGELOG section via awk. Insert steps 5a and 5b immediately before step 6.

   **Step 5a — Install git-cliff (MUST):**
   ```yaml
   - name: Install git-cliff (release-tag path)
     uses: taiki-e/install-action@d438492cf8a250514fa2d34b30bc3c0dc37c65ff # v2
     with:
       tool: git-cliff@2.14.1
   ```
   Use the exact SHA `d438492cf8a250514fa2d34b30bc3c0dc37c65ff` — the same pin already
   used by release.yml's "Install git-cliff (nightly path)" step. Do NOT use
   `cargo install git-cliff` here: release-tag.yml has no Rust toolchain setup step
   and `cargo install` would require one (~3–5 min compile). `taiki-e/install-action`
   downloads a pre-built binary without a toolchain dependency (~10 s).

   **Step 5b — Generate CHANGELOG, commit, and push develop (MUST):**
   Use the `TAG` workflow input variable (includes `v` prefix, e.g., `v1.0.0-beta.2`).
   Detect the channel suffix with the same if/elif/else structure as Task 7 but matching
   against `TAG` instead of `v${VERSION}`:
   ```bash
   if [[ "${TAG}" =~ -nightly\. ]]; then
     TAG_PATTERN='^v[0-9]+\.[0-9]+\.[0-9]+(-nightly\.[0-9]{8}(\.[0-9]+)?)?$'
   elif [[ "${TAG}" =~ -alpha\. ]]; then
     TAG_PATTERN='^v[0-9]+\.[0-9]+\.[0-9]+(-alpha\.[0-9]+)?$'
   elif [[ "${TAG}" =~ -beta\. ]]; then
     TAG_PATTERN='^v[0-9]+\.[0-9]+\.[0-9]+(-beta\.[0-9]+)?$'
   elif [[ "${TAG}" =~ -rc\. ]]; then
     TAG_PATTERN='^v[0-9]+\.[0-9]+\.[0-9]+(-rc\.[0-9]+)?$'
   else
     TAG_PATTERN='^v[0-9]+\.[0-9]+\.[0-9]+$'
   fi
   ```
   Then run the full 3-substep §D3 mechanism:
   - **PRE-STRIP:** strip prior masthead + `## [Unreleased]` from CHANGELOG.md
   - `git cliff --tag "${TAG}" --unreleased --tag-pattern "${TAG_PATTERN}" --prepend CHANGELOG.md`
   - **LINK-REF-UPDATE:** repoint `[Unreleased]` compare ref + insert version ref

   After the 3-substep flow, commit and push:
   ```bash
   git commit -m "chore: update CHANGELOG for ${TAG}"
   git push origin develop
   ```

   **Auth — no new permission needed:** The existing `RELEASE_PROMOTE_TOKEN` PAT (used for
   checkout in step 1) carries "Repository contents: Read and write" scope.
   `permissions: contents: write` is already declared at the workflow-level `permissions`
   block. No additional scope addition is required.

   **Tag-ordering invariant (MUST):** Step 5b's `git push origin develop` MUST complete
   before step 6 ("Create annotated tag on develop HEAD") runs. `git tag -a "${TAG}"` in
   step 6 targets the local HEAD; the annotated tag therefore points at the post-CHANGELOG
   commit. When step 7 pushes the tag and triggers release.yml, release.yml's awk extraction
   reads CHANGELOG.md from its checkout at TAG_SHA — the correctly-scoped `## [VERSION]`
   section is already present because the tag was created on the post-CHANGELOG HEAD.

9. **Update docs.** Update the following:
   - `docs/RELEASE-CHANNELS.md` §5 "Build Path": add a note explaining that each channel
     uses a channel-scoped `--tag-pattern` for git-cliff, so release notes show the delta
     since the previous same-channel release (with stable as floor). Reference "ADR-063 §D7".
   - `RELEASING.md §5` "Two-Layer CHANGELOG Model": add a note under Layer 2 that
     release-prep.yml detects the channel from VERSION and passes the appropriate
     `--tag-pattern` to git-cliff. Reference "ADR-063 §D7".

10. **Verify AC-001..AC-009 explicitly.** For each AC, run the specified grep command or
    dry-run and record the result. Document in PR description.

11. **Run `actionlint`** on all three modified workflow files:
    ```bash
    actionlint .github/workflows/release.yml
    actionlint .github/workflows/release-prep.yml
    actionlint .github/workflows/release-tag.yml
    ```
    Fix any findings before pushing.

---

## Acceptance Criteria

### AC-001: `--tag-pattern` flag present in all three modified workflow invocations
Each of the three git-cliff invocations that this story modifies includes a `--tag-pattern`
argument:
- `release.yml` nightly path: `grep 'tag-pattern' .github/workflows/release.yml` returns a
  match inside the `git cliff --latest` nightly branch.
- `release-prep.yml` Step 7: `grep 'TAG_PATTERN' .github/workflows/release-prep.yml`
  returns a match inside the git-cliff invocation.
- `release-tag.yml` CHANGELOG generation: `grep 'tag-pattern' .github/workflows/release-tag.yml`
  returns a match inside the new CHANGELOG generation step.
(traces to ADR-063 §D3 — `--tag-pattern` CLI override of cliff.toml `tag_pattern`; the
per-channel pattern design is the authoritative fix for the beta.2 CHANGELOG masking issue)

### AC-002: Nightly pattern covers nightly + stable; excludes all other pre-releases
The nightly channel `--tag-pattern` used in release.yml satisfies:
- `echo "v1.0.0-nightly.20260909" | grep -E '^v[0-9]+\.[0-9]+\.[0-9]+(-nightly\.[0-9]{8}(\.[0-9]+)?)?$'` matches
- `echo "v1.0.0-nightly.20260909.2" | grep -E '...'` matches (same-day suffix)
- `echo "v1.0.0" | grep -E '...'` matches (stable-floor)
- `echo "v1.0.0-beta.1" | grep -E '...'` does NOT match (excluded)
- `echo "v1.0.0-rc.1" | grep -E '...'` does NOT match (excluded)
`grep 'nightly' .github/workflows/release.yml | grep 'tag-pattern'` returns a line
containing the nightly-specific regex.
(traces to RELEASE-CHANNELS.md §3 — nightly channel is a distinct tag series; beta/rc
commits must not be claimed as "already released" in the nightly universe)

### AC-003: Pre-release pattern (release-tag.yml) covers channel + stable; excludes nightly and other channels
For each pre-release channel (alpha, beta, rc), the `--tag-pattern` used in release-tag.yml satisfies:
- Beta: `echo "v1.0.0-beta.2" | grep -E '^v[0-9]+\.[0-9]+\.[0-9]+(-beta\.[0-9]+)?$'` matches
- Beta floor: `echo "v1.0.0" | grep -E '...'` matches (stable-floor when no prior beta)
- Beta excludes nightly: `echo "v1.0.0-nightly.20260909" | grep -E '...'` does NOT match
- Beta excludes rc: `echo "v1.0.0-rc.1" | grep -E '...'` does NOT match
Same verification applies to alpha and rc patterns with their respective regexes.
`grep 'TAG_PATTERN\|tag-pattern' .github/workflows/release-tag.yml` returns a multi-line
block showing channel detection (if/elif/else) and per-channel pattern assignment.
(traces to RELEASE-CHANNELS.md §3 — each pre-release channel is a distinct tag series;
the beta.2 precedent demonstrates the bug that cross-channel masking causes)

### AC-004: Stable pattern (release-prep.yml) covers stable-only; excludes all pre-releases
The stable branch of the channel detection in release-prep.yml uses a stable-only pattern:
`^v[0-9]+\.[0-9]+\.[0-9]+$`. This regex:
- `echo "v1.0.0" | grep -E '^v[0-9]+\.[0-9]+\.[0-9]+$'` matches
- `echo "v1.0.0-nightly.20260909" | grep -E '...'` does NOT match
- `echo "v1.0.0-beta.1" | grep -E '...'` does NOT match
`grep 'stable\|TAG_PATTERN' .github/workflows/release-prep.yml | grep -v '#'` shows a
pattern assignment where the stable branch assigns a regex without any `(-...)?` suffix group.
(traces to RELEASE-CHANNELS.md §3 — stable CHANGELOG must show all commits since the last
stable release, regardless of which nightly/beta tags captured those commits en route)

### AC-005: Stable-floor fallback is implicit in pattern design
When the beta pattern `^v[0-9]+\.[0-9]+\.[0-9]+(-beta\.[0-9]+)?$` is used and no prior
beta tag exists in the repository, git-cliff's "previous tag" resolves to the last matching
stable tag (because stable `v1.0.0` matches the optional-group pattern). Verified by dry-run:
```bash
git cliff --unreleased --tag v1.0.0-beta.1 \
  --tag-pattern '^v[0-9]+\.[0-9]+\.[0-9]+(-beta\.[0-9]+)?$' \
  --output /dev/stdout
```
on a repo that has stable `v1.0.0` but no prior `v1.0.0-beta.*` tags — output should include
commits since `v1.0.0`, not from the repo origin (which would include all history).
Document the dry-run result in the PR description.
(traces to RELEASE-CHANNELS.md §3 — "first release of a channel within an X.Y.Z line falls
back to the last stable tag as the delta floor")

### AC-006: Nightly bypass-guard regex in release.yml is unchanged
The `if [[ "$TAG" =~ ... ]]` nightly regex at the top of the "Extract release notes" step
body in release.yml is unchanged from its post-S-REL-NIGHTLY-NOTES-001 form:
`^v[0-9]+\.[0-9]+\.[0-9]+-nightly\.[0-9]{8}(\.[0-9]+)?$`
Both the `^` start-anchor and the `$` end-anchor must be present. The optional same-day
suffix group `(\.[0-9]+)?` must be present.
`grep 'nightly\.\[0-9\]' .github/workflows/release.yml | grep -c '\\$'` ≥ 1.
The `--tag-pattern` addition is in the nightly BODY only; it does not alter the branch
selector.
(traces to RELEASE-CHANNELS.md §2 — exact-form nightly bypass-guard prevents hybrid-tag
masking of the curated-CHANGELOG gate; must be preserved)

### AC-007: Curated-CHANGELOG hard-fail (RELEASE-NOTES-MISSING) unchanged in release.yml
The non-nightly branch of "Extract release notes" in release.yml is identical to its
post-S-REL-NIGHTLY-NOTES-001 form:
- `grep 'RELEASE-NOTES-MISSING' .github/workflows/release.yml` returns exactly 1 match.
- `grep 'awk.*CHANGELOG' .github/workflows/release.yml` returns a match (awk extraction).
This story does NOT modify the non-nightly branch of release.yml's release notes step.
(traces to RELEASE-CHANNELS.md §2 — stable/beta/rc channels require curated CHANGELOG
with hard-fail; this story does not alter that gate)

### AC-008: `actionlint` exits 0 on all three modified workflow files
```bash
actionlint .github/workflows/release.yml
actionlint .github/workflows/release-prep.yml
actionlint .github/workflows/release-tag.yml
```
All three commands produce zero errors and exit 0. Any expression syntax issues from
the new `--tag-pattern` arguments or channel-detection if/elif/else blocks are resolved
before the PR is pushed.
(traces to RELEASE-CHANNELS.md §5 — all release lane workflows are production CI;
structural correctness is required)

### AC-009: Docs updated to describe channel-scoped model
Both documentation files are updated:
- `docs/RELEASE-CHANNELS.md` §5 "Build Path": contains a new paragraph or table row
  describing that each channel uses a channel-scoped `--tag-pattern`, with reference to
  "ADR-063 §D7" (bare citation — §D7 is ACCEPTED v1.14; no qualifier).
  `grep 'channel.scope\|tag.pattern\|D7' docs/RELEASE-CHANNELS.md` returns a match.
- `RELEASING.md §5` "Two-Layer CHANGELOG Model" Layer 2 section: contains a note that
  release-prep.yml detects the channel from VERSION and passes `--tag-pattern` accordingly.
  `grep 'channel.scope\|tag.pattern\|D7' RELEASING.md` returns a match.
(traces to RELEASE-CHANNELS.md §3 — channel model documentation must reflect actual
git-cliff behavior so operators and future implementers understand the delta scope)

### AC-010: release-tag.yml structural checks + live per-channel verification
Two-part verification:

**Part A — Structural checks (pre-merge, via grep/inspection):**
These checks verify the §D7 Site 3 MUSTs are implemented correctly before the PR merges
(anchored: ADR-063 §D7 install MUST + ordering MUST):

1. **Install SHA match:**
   `grep 'taiki-e/install-action@d438492cf8a250514fa2d34b30bc3c0dc37c65ff' .github/workflows/release-tag.yml`
   returns a match (step 5a uses the correct SHA pin — same as release.yml).

2. **Tool spec:**
   `grep 'git-cliff@2.14.1' .github/workflows/release-tag.yml`
   returns a match inside the new install step.

3. **Tag-ordering invariant:** Inspect the step order in release-tag.yml's job:
   The `git push origin develop` command in step 5b appears BEFORE the "Create annotated
   tag on develop HEAD" step (step 6). Verify by reading the workflow and confirming step
   numbers are sequential: 5a (install) → 5b (CHANGELOG+commit+push) → 6 (git tag -a).

4. **No `cargo install` for git-cliff in release-tag.yml:**
   `grep 'cargo install.*git-cliff' .github/workflows/release-tag.yml` returns no match.

**Part B — Live verification (post-merge, human-in-loop gate at Phase-verify):**
After the PR merges, verify with dry-runs for at least two channels:

1. **Nightly**: Run `git cliff --latest \
   --tag-pattern '^v[0-9]+\.[0-9]+\.[0-9]+(-nightly\.[0-9]{8}(\.[0-9]+)?)?$' \
   --strip=header --output /dev/stdout` on develop. Verify output does NOT include commits
   already under a beta/rc/stable tag.
2. **Stable (simulated)**: Run `git cliff --unreleased --tag v1.0.0 \
   --tag-pattern '^v[0-9]+\.[0-9]+\.[0-9]+$' --output /dev/stdout`.
   Verify output includes commits from all channels (nightly, beta, rc) that contributed
   new feat/fix/perf/refactor/security commits since the last stable tag.

Document Part B results in the post-merge live-verify comment.
(traces to RELEASE-CHANNELS.md §3 — channel-scoped CHANGELOG is the core invariant;
structural correctness (Part A) and end-to-end verification (Part B) together confirm
the install mechanism, step ordering, and per-channel pattern design are all correct)

---

## Previous Story Intelligence

| Story | Key Decisions | Patterns Established | Gotchas Discovered |
|-------|--------------|---------------------|-------------------|
| S-REL-CLIFF-001 | cliff.toml at repo root; `--unreleased --tag` is the pre-tag pattern; 3-substep PRE-STRIP+git-cliff+LINK-REF-UPDATE for release-prep.yml Step 7; `tag_pattern = "v[0-9].*"` global | git-cliff `--tag-pattern` CLI arg overrides cliff.toml `tag_pattern` for that invocation; catch-all `{ message = ".*", skip = true }` as final commit_parser | `--output` with `--prepend` causes duplicate sections; `regex_replace` not in Tera; `{ breaking = true }` parser-level group NOT effective in 2.14.1 |
| S-REL-NIGHTLY-NOTES-001 | git-cliff `--latest` in release.yml nightly path; SHA-pinned taiki-e/install-action; `git fetch --unshallow --tags` before git-cliff; canned fallback on empty/error output | git-cliff installed in publish-release job via taiki-e/install-action; `--latest` gives prev_tag..current_tag window per cliff.toml tag_pattern | publish-release checkout is shallow by default; nightly bypass-guard `if [[ "$TAG" =~ ... ]]` must remain exact-form |
| S-REL-BETA1-NOTES-001 (precedes beta.2 issue) | `--unreleased --tag vX.Y.Z-beta.N --prepend` for pre-release CHANGELOG; first-release handling via `--unreleased` selecting from root when no prior tag | curated-CHANGELOG curation model; technical-writer Layer-1 top-block | N/A — completed before channel-scope problem manifested |
| beta.2 CHANGELOG (ad-hoc precedent, 2026-09-09) | Manual `--tag-pattern "^v[0-9]+\.[0-9]+\.[0-9]+(-beta\.[0-9]+)?$"` workaround | This IS the channel-scoped pattern applied once; this story generalises it | v1.0.0-nightly.20260909 and .2 sat on commits PRs #277/#278; `--unreleased` without `--tag-pattern` treated them as released; commits were silently dropped |

Read each workflow file IN FULL before modifying. The multi-branch structure of release.yml's
"Extract release notes" step is complex; the nightly regex guard and the curated-CHANGELOG
hard-fail must survive the modifications untouched.

The `--tag-pattern` flag in git-cliff 2.14.1 replaces (not merges with) the cliff.toml
`tag_pattern` for that invocation. Verify this behavior with a quick dry-run on develop
before committing the patterns.

---

## Architecture Compliance Rules

| Rule | Source | Enforcement |
|------|--------|-------------|
| git-cliff 2.14.1 pinned exactly (where installed) | ADR-063 §D1 | release-tag.yml step 5a MUST use `taiki-e/install-action@d438492cf8a250514fa2d34b30bc3c0dc37c65ff # v2` with `tool: git-cliff@2.14.1`; `cargo install` MUST NOT be used (no Rust toolchain step in release-tag.yml) |
| cliff.toml NOT modified | ADR-063 §D3 — cliff.toml is authoritative config; per-channel scoping via CLI flags | AC-001 file structure: cliff.toml listed as DO NOT MODIFY |
| Nightly regex bypass-guard intact (exact-form start+end anchors) | RELEASE-CHANNELS.md §2 | AC-006: grep for anchored regex on nightly branch if-condition |
| Curated-CHANGELOG hard-fail (RELEASE-NOTES-MISSING) unchanged | RELEASE-CHANNELS.md §2 | AC-007: RELEASE-NOTES-MISSING grep exactly 1 match |
| 3-substep PRE-STRIP+git-cliff+LINK-REF-UPDATE pattern for CHANGELOG generation | ADR-063 §D3 `--prepend` mechanism | release-tag.yml CHANGELOG generation step must include all 3 substeps |
| All CI action references use commit SHA pins (not tag pointers) | project CI hardening convention | release-tag.yml step 5a uses exact SHA `d438492cf8a250514fa2d34b30bc3c0dc37c65ff`; verified match with release.yml pin |
| `--tag-pattern` CLI override replaces cliff.toml tag_pattern for that invocation | ADR-063 §D3 | Task 4 verification note; confirmed by beta.2 precedent |
| Stable-floor fallback implemented via optional-group pattern design | RELEASE-CHANNELS.md §3; ADR-063 §D7 | AC-005 dry-run verification |
| All modified workflows must be actionlint-clean | project CI quality standard | AC-008: actionlint exits 0 on all three files |

---

## Library & Framework Requirements

| Dependency | Version | Notes |
|------------|---------|-------|
| `git-cliff` | `2.14.1` | Same pin as ADR-063 §D1; all three sites (release.yml nightly, release-prep.yml Step 7, release-tag.yml step 5a) must use this exact version. |
| `taiki-e/install-action` | `d438492cf8a250514fa2d34b30bc3c0dc37c65ff # v2` | Required for release-tag.yml step 5a (`tool: git-cliff@2.14.1`). Mirrors the SHA pin already used in release.yml's "Install git-cliff (nightly path)" step. Do NOT re-resolve the SHA — use the exact value here to avoid a third divergent pin. `cargo install` MUST NOT be used in release-tag.yml (no Rust toolchain step present). |
| `cliff.toml` | existing at repo root | Not modified. `--tag-pattern` CLI flag overrides `tag_pattern` for the invocation. |
| Bash regex (`[[ =~ ]]`) | standard (bash 5+) | Used for channel detection in release-prep.yml and release-tag.yml. `ubuntu-latest` provides bash 5+. |

---

## File Structure Requirements

| File | Action | Notes |
|------|--------|-------|
| `.github/workflows/release.yml` | Modify (nightly path only) | Insert `--tag-pattern` into existing `git cliff --latest --strip=header` in nightly branch of "Extract release notes (channel-aware)" step; no `2>/dev/null`; bypass-guard and non-nightly path UNCHANGED |
| `.github/workflows/release-prep.yml` | Modify (Step 7 only) | Add channel-detection block + `--tag-pattern` to git-cliff invocation; 3-substep PRE-STRIP/LINK-REF-UPDATE unchanged |
| `.github/workflows/release-tag.yml` | Modify (add CHANGELOG generation step) | Add CHANGELOG generation step (channel-detect + 3-substep + git-cliff `--tag-pattern`) before tag push; add commit step for CHANGELOG.md |
| `cliff.toml` | DO NOT modify | cliff.toml is authoritative (ADR-063 §D3); per-channel scoping is via CLI flags only |
| `docs/RELEASE-CHANNELS.md` | Modify (§5 only) | Add channel-scoped model paragraph with ADR-063 §D7 reference |
| `RELEASING.md` | Modify (§5 only) | Add channel-scoped Layer 2 note with ADR-063 §D7 reference |

---

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| `release.yml` §publish-release nightly path | `.github/workflows/` | Effectful (CI workflow; invokes git-cliff; writes release notes file; creates GitHub Release) |
| `release-prep.yml` Step 7 | `.github/workflows/` | Effectful (CI workflow; installs git-cliff; modifies CHANGELOG.md; commits) |
| `release-tag.yml` CHANGELOG step (new) | `.github/workflows/` | Effectful (CI workflow; installs git-cliff; modifies CHANGELOG.md; commits; pushes tag) |
| `cliff.toml` | repo root | Pure (static config; referenced but not modified) |

---

## Purity Classification

All deliverables in this story are **Effectful** CI workflow artifacts. No pure-core Rust
code is modified. Classification per ADR-022 pure/effectful taxonomy:

| Artifact | Classification | Rationale |
|----------|---------------|-----------|
| `.github/workflows/release.yml` (nightly path modification) | Effectful | CI workflow step: invokes git-cliff process, writes files, triggers GitHub API calls |
| `.github/workflows/release-prep.yml` (Step 7 modification) | Effectful | CI workflow step: reads git history, writes CHANGELOG.md, commits to develop branch |
| `.github/workflows/release-tag.yml` (new CHANGELOG step) | Effectful | CI workflow step: reads git history, writes CHANGELOG.md, commits, pushes tag via git |
| `cliff.toml` | Pure (not modified) | Static config file; changes via CLI --tag-pattern only |
| `docs/RELEASE-CHANNELS.md` (doc update) | Pure (documentation) | Markdown documentation; no side effects |
| `RELEASING.md` (doc update) | Pure (documentation) | Markdown documentation; no side effects |

No Rust production code is added or modified. The pure/effectful boundary in the prism
workspace (`prism-core`, `prism-query`, etc.) is unaffected.

---

## Holdout Applicability

**Determination: N/A**

This story has `tdd_mode: facade` and `crates_touched: []` — it produces no prism binary.
Its deliverables are modifications to GitHub Actions YAML workflows and documentation files.
All verification surfaces are structural: AC-001..AC-009 are grep/inspection/dry-run checks.
AC-010 is a live CI run verification.

Consistent with prior infra-story determinations for S-REL-NIGHTLY-NOTES-001, S-REL-CLIFF-001,
S-REL-DROP-INTEL-MAC-001, S-REL-WRITER-001 (all `holdout_scenarios: []`).

`holdout_scenarios: []` — no scenarios authored; HOLDOUT-INDEX unchanged.

---

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | First nightly ever in the X.Y.Z line (no prior nightly tag; last tag is a stable or no tag at all) | Nightly `--tag-pattern` includes stable tags (optional-group pattern). If a stable exists, git-cliff uses it as floor. If no tag at all, git-cliff renders from repo origin. No error; potentially large output (categorized commits from origin). Canned fallback fires only on empty output or git-cliff error. |
| EC-002 | First beta in an X.Y.Z line (no prior beta.N tag; last matching tag is a stable) | Beta `--tag-pattern` `(-beta\.[0-9]+)?$` matches stable too. git-cliff uses last stable as floor. Commits since stable (including those under nightly tags) appear in the beta CHANGELOG. |
| EC-003 | Stable release after a long nightly+beta+rc sequence | Stable `--tag-pattern` `^v[0-9]+\.[0-9]+\.[0-9]+$` excludes ALL pre-release tags. git-cliff sees only the previous stable as "prior released". All commits since the previous stable — regardless of how many nightly/beta/rc tags they were "under" — appear in the stable CHANGELOG. |
| EC-004 | Same-day second nightly (vX.Y.Z-nightly.YYYYMMDD.2) | Nightly pattern matches both `.YYYYMMDD` and `.YYYYMMDD.N` forms (two optional groups). git-cliff `--latest` gives commits between `.YYYYMMDD` and `.YYYYMMDD.2`. |
| EC-005 | release-tag.yml runs for a dev tag (vX.Y.Z-dev.<shortsha>) | Dev channel is not explicitly handled in the per-channel if/elif/else. Falls through to the stable branch (no `-{channel}\.` pattern match). This is intentional: dev builds are ad-hoc snapshots with no specific CHANGELOG scope requirement. Or add an explicit dev branch if required. |
| EC-006 | git-cliff produces empty output with per-channel `--tag-pattern` | For release-tag.yml and release-prep.yml: git-cliff empty output means no categorized commits in the range (all skip-rule types). The CHANGELOG section would be empty under the version header — the RELEASE-NOTES-MISSING hard-fail in release.yml would trigger for stable/beta/rc. Implementer must decide: add a fallback for release-tag.yml similar to nightly path? Or fail-fast to signal that a release with zero categorized commits is unusual? |
| EC-007 | `--tag-pattern` semantics differ between git-cliff versions | The workaround used in the beta.2 CHANGELOG confirmed `--tag-pattern` works in git-cliff 2.14.1. If a future git-cliff upgrade changes the flag name or semantics, the patterns in these workflow files must be updated. The version pin (2.14.1) protects against accidental breakage. |

---

## Changelog

| Version | Date | Author | Change |
|---------|------|--------|--------|
| 1.2 | 2026-09-09 | story-writer | ADR-063 §D7 v1.14 spec-accuracy corrections: Fix 1 — Task 6 before-image corrected to live step name "Extract release notes (channel-aware)" and live invocation (no 2>/dev/null; --strip=header already present); Fix 2 — Library & Framework Requirements taiki-e row states exact SHA d438492... + tool spec git-cliff@2.14.1, removes re-resolution language, adds cargo install MUST NOT prohibition; Fix 3 — Task 8 replaced with concrete §D7 Site 3 contract (steps 5a+5b before step 6, auth already in place, tag-ordering MUST); Architecture Compliance Rules updated with exact SHA; AC-010 expanded with Part A structural checks for install SHA + ordering (install MUST + ordering MUST anchored from §D7 v1.14 Dim-3); all v1.13 ADR version pins updated to v1.14 |
| 1.1 | 2026-09-09 | story-writer | ADR-063 §D7 ACCEPTED (v1.13, 2026-09-09): swept all [pending amendment]/[pending] qualifiers from §Authority, Behavioral Contracts table, Task 9, Architecture Compliance Rules, and risk_mitigations; updated Amendment Routing Flag section to reflect live status; added --strip=header to Task 6 nightly git-cliff invocation per §D7 Site 1; behavioral_contracts comment updated to include §D7; status draft → ready |
| 1.0 | 2026-09-09 | story-writer | Initial materialization — channel-scoped git-cliff tag filter; per-channel patterns; stable-floor fallback; release-tag.yml CHANGELOG generation addition; ADR-063 amendment routing flag; v1.0-stable dependency; 10 ACs; facade tdd_mode |
