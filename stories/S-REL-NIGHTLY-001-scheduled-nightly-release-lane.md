---
document_type: story
story_id: S-REL-NIGHTLY-001
title: "devops: scheduled nightly release lane — cron + change-guard + vX.Y.Z-nightly.YYYYMMDD tag + 14-build retention + edge-nightly pointer (RELEASE-CHANNELS.md §2)"
wave: F-A
epic_id: E-REL
priority: P2
# P2 justification: nightly is a convenience/observability channel; fully independent
# of the beta.2 critical path (human-directed 2026-09-08: OWN INDEPENDENT TRACK,
# NOT beta.2-blocking). P2 signals it runs after P0/P1 work clears but does not gate any
# product story.
status: ready
version: "1.0"
level: "L4"
producer: story-writer
timestamp: "2026-09-08T00:00:00Z"
tdd_mode: facade
# tdd_mode: facade — this story creates a GitHub Actions workflow file (YAML) and updates
# a docs markdown file. No Rust production code is modified. Verification is via
# grep/structural assertions + a live workflow_dispatch run (AC-010 human-in-loop gate).
# Mutation testing at wave gate replaces Red Gate density check per BC-8.30.001.
subsystems: [SS-22]
# Subsystem anchor justification:
#   SS-22 (Process Lifecycle) governs the release build pipeline and binary distribution
#   channels. The nightly lane workflow and its retention/edge-pointer mechanics all fall
#   within SS-22's process lifecycle boundary. No other ARCH-INDEX subsystem owns the
#   CI/CD scheduled release lane.
crates_touched: []
# crates_touched: [] — nightly.yml and RELEASE-CHANNELS.md are CI/documentation artifacts
# only. No Rust crate source is modified.
target_module: devops
capabilities: []
behavioral_contracts: []
# BC status: N/A — scheduled nightly release lane is release-infrastructure governance.
# No subsystem behavioral contract governs the CI release channel schedule or retention.
# Authority is RELEASE-CHANNELS.md §2/§3/§5/§6/§7/§8 (human-approved strategy doc).
# POL-14 NO-OP (behavioral_contracts: []).
verification_properties: []
depends_on: [S-REL-001]
# Dependency anchor justification:
#   depends_on S-REL-001: release.yml (the tag-triggered 4-target build workflow) is the
#   downstream consumer of every tag pushed by nightly.yml. S-REL-001 repaired release.yml
#   to correctly handle pre-release hyphenated tags with --prerelease, which is the build
#   path that nightly tags trigger. Without an operational release.yml, the nightly tag
#   push fires nothing meaningful.
#   RELEASE_PROMOTE_TOKEN: same PAT mechanism as release-tag.yml (implemented alongside
#   S-REL-001 infra); no separate factory story for the secret setup.
blocks: []
# blocks: [] — nightly is an own independent track. No product stories depend on the
# nightly lane being operational. The human explicitly directed this as OWN TRACK
# (2026-09-08), not part of beta.2 scope.
points: 5
estimated_days: 1
risk: LOW
# Risk justification: pure new workflow file + documentation update — no changes to
# existing workflows. The TWO-LAYER safety filter in the retention step is the highest-risk
# element; it is directly verified by AC-007 grep inspection. The worst failure mode
# (deleting a non-nightly tag) is guarded by both the glob pre-filter and the regex
# double-check in the retention step body.
acceptance_criteria_count: 10
red_gate_tests: 0
# red_gate_tests: 0 — tdd_mode: facade. No Rust production code; no Red Gate.
# Quality gate: structural grep checks (AC-001..AC-009) + live workflow_dispatch run (AC-010).
estimated_passes: "1-2 LOCAL adversary passes"
holdout_scenarios: []
# HOLDOUT-N/A: tdd_mode=facade; crates_touched=[]; no built prism binary; no MCP-visible
# surface. This story delivers a GitHub Actions workflow file and a docs update. Verification
# is structural (grep/inspection) + a live CI run (AC-010). No MCP stdio wire-level
# assertions are possible. Gate definitionally inapplicable per CLAUDE.md story-level
# holdout gate definition (requires built prism binary + MCP stdio wire assertions scoped
# to the story's touched surface). Consistent with S-REL-CLIFF-001 and S-REL-DROP-INTEL-MAC-001
# holdout determinations.
assumption_validations: []
risk_mitigations:
  - "PAT (RELEASE_PROMOTE_TOKEN) is REQUIRED — the default GITHUB_TOKEN cannot trigger
    downstream on:push:tags workflows (release.yml) per GitHub's security model. The
    checkout step must use `token: RELEASE_PROMOTE_TOKEN` so the subsequent git push
    carries PAT credentials. The GH_TOKEN env var for gh CLI operations must also be
    set to RELEASE_PROMOTE_TOKEN (not GITHUB_TOKEN)."
  - "TWO-LAYER nightly-only safety filter in the retention step: (1) glob pre-filter
    `v*-nightly.*` limits the candidate set; (2) regex double-check anchored to
    `^v[0-9]+\.[0-9]+\.[0-9]+-nightly\.[0-9]{8}` skips any candidate that does not
    match the nightly pattern. Tags with alpha/beta/rc/stable/edge suffixes CANNOT
    be matched by the regex. Any non-matching candidate emits a ::warning:: and is
    skipped without deletion. This two-layer approach prevents accidental deletion of
    named channels under any YAML evaluation path."
  - "Change-guard uses `git rev-list -n 1` to dereference annotated tags to their
    underlying commit SHA, not the tag object SHA. This is required because nightly
    creates annotated tags (not lightweight tags) — comparing tag-object SHAs would
    always find a difference even when the tagged commit is identical to HEAD."
  - "edge-nightly tag does NOT start with 'v' — it therefore does NOT fire release.yml
    (release.yml trigger is `on: push: tags: v*`). This is intentional per
    RELEASE-CHANNELS.md §7: edge-nightly is a mutable convenience pointer only; dated
    nightly tags (v*) are what trigger release.yml builds."
  - "BASE-MATCH: prism-bin version channel suffix is stripped via `%%-*` shell parameter
    expansion (not a regex). `1.0.0-dev` → `1.0.0`. No version bump is needed between
    nightly builds on the same X.Y.Z development cycle per RELEASE-CHANNELS.md §3."
  - "Same-day collision guard: if `vX.Y.Z-nightly.YYYYMMDD` already exists on origin
    (develop advanced more than once in a UTC day), the tag computation step appends
    a monotonic suffix (.2, .3, ...) checked via `git ls-remote --tags origin`. The
    check-and-increment loop continues until a non-colliding tag is found."
  - "Concurrency group `nightly-build` with `cancel-in-progress: false` — a second
    scheduled run that arrives while a prior run is still in-progress is queued rather
    than cancelled, because a tagging workflow may have already pushed a tag mid-run."
inputs:
  - ".github/workflows/nightly.yml"
  - "docs/RELEASE-CHANNELS.md"
  - ".github/workflows/release-tag.yml"
input-hash: "[pending-recompute]"
traces_to: []
cycle: "v1.0.0-nightly-lane"
phase: "3"
---

# S-REL-NIGHTLY-001 — Scheduled Nightly Release Lane

**Story ID:** S-REL-NIGHTLY-001
**Status:** ready
**Version:** v1.0
**Wave:** F-A
**Priority:** P2
**Points:** 5
**Own-track note:** This story is an INDEPENDENT TRACK (human-directed 2026-09-08). The
nightly lane is a continuous observability/convenience channel that runs alongside the
main release cadence. It does NOT gate and is NOT gated by beta.2 or any product story.

---

## Origin

`docs/RELEASE-CHANNELS.md` §2 (human-approved 2026-09-04) defined the nightly channel
as the "current develop state — scheduled" tier of the release ladder. The channel model
requires: scheduled cron trigger, change-guard to skip no-op runs, BASE-MATCH tag
computation (RELEASE-CHANNELS.md §3), PAT-triggered release.yml build (§5), 14-build
retention (§6), and an `edge-nightly` mutable convenience pointer (§7). The channel is
explicitly ungated per §8.

This story materialized the nightly lane implementation as `.github/workflows/nightly.yml`
(committed on feature branch `feature/S-REL-NIGHTLY-001`, commits c799847d5 workflow +
a2c7a8bce docs) and updated `docs/RELEASE-CHANNELS.md` v0.1→v0.2 to mark §2/§6/§7
IMPLEMENTED.

This is a retroactive materialization story — the implementation exists on the feature
branch and is being registered for VSDD traceability.

---

## Narrative

As a release engineer, I want a scheduled nightly workflow that automatically creates a
dated pre-release tag on `develop` whenever the branch has advanced since the last nightly
build, so that consumers of the nightly channel always have a fresh, reproducible build
without manual intervention, and so that the release ladder (RELEASE-CHANNELS.md §2) is
fully operational at every tier.

---

## Authority

- RELEASE-CHANNELS.md §2 — Channel Model: nightly channel definition, trigger, tag form,
  source branch, GitHub marking (Pre-release)
- RELEASE-CHANNELS.md §3 — BASE-MATCH tag guard: X.Y.Z core read from prism-bin/Cargo.toml;
  channel suffix stripped; no version-bump churn needed
- RELEASE-CHANNELS.md §5 — Build Path: full 4-target matrix for every channel; release.yml
  triggered by any `v*` tag push
- RELEASE-CHANNELS.md §6 — Retention Policy: keep last 14 nightly builds; auto-delete older
  nightly Releases and tags
- RELEASE-CHANNELS.md §7 — Edge Convenience Pointers: edge-nightly mutable tag + GitHub
  Release updated on each nightly run
- RELEASE-CHANNELS.md §8 — Gating Summary: nightly is ungated; flows freely on schedule

(No BC: scheduled release lane; no subsystem behavioral contract. No ADR authored; authority
is RELEASE-CHANNELS.md. If an ADR is conventionally required for this channel, file a
follow-up to author one — do not invent an ADR anchor for a non-existent document.)

---

## Behavioral Contracts

This story has no subsystem behavioral contracts. Authority is RELEASE-CHANNELS.md §2/§3/§5/§6/§7/§8.

| Architecture Source | Clause |
|---------------------|--------|
| RELEASE-CHANNELS.md §2 | Nightly channel: scheduled cron trigger; source branch `develop`; no human gate; tag form `X.Y.Z-nightly.YYYYMMDD`; GitHub marking Pre-release |
| RELEASE-CHANNELS.md §3 | BASE-MATCH: compare only X.Y.Z core; channel suffix (`-dev`) stripped from prism-bin/Cargo.toml version; no version-bump churn between nightly builds |
| RELEASE-CHANNELS.md §3 | Same-day monotonic suffix: `X.Y.Z-nightly.YYYYMMDD.N` when develop advances more than once in a UTC day |
| RELEASE-CHANNELS.md §5 | Full 4-target build matrix for every channel including nightly; release.yml triggered by any `v*` tag push |
| RELEASE-CHANNELS.md §6 | Nightly retention: keep last 14; auto-delete older nightly Releases and tags; deletion filter cannot match alpha/beta/rc/stable/edge tags |
| RELEASE-CHANNELS.md §7 | edge-nightly mutable pointer: force-updated on each nightly run; delete-then-repush pattern; NOT a reproducibility pin |
| RELEASE-CHANNELS.md §8 | Nightly is ungated: flows freely; no human approval gate |
| RELEASE-CHANNELS.md §1 | Pre-release channels tag `develop` only — no `main` write; only stable channel writes `main` |

---

## Token Budget Estimate

| Artifact | Estimated Tokens |
|----------|-----------------|
| This story spec | ~3,500 |
| `.github/workflows/nightly.yml` (full) | ~4,000 |
| `docs/RELEASE-CHANNELS.md` (full) | ~3,500 |
| `.github/workflows/release-tag.yml` (structural reference for PAT mechanism) | ~2,000 |
| Total | ~13,000 |

Well within the 30% context window budget.

---

## Red Gate Test List (SAC-1)

**tdd_mode: facade — Red Gate is N/A.** No Rust production code; no Red Gate tests.
Verification is via structural grep/inspection checks the implementer explicitly performs
before the PR:

- AC-001: `ls .github/workflows/nightly.yml` exits 0; `actionlint .github/workflows/nightly.yml` exits 0
- AC-002: `grep -c 'cron:' .github/workflows/nightly.yml` ≥ 1; `grep -c 'workflow_dispatch' .github/workflows/nightly.yml` ≥ 1
- AC-003: `grep 'nightly-build' .github/workflows/nightly.yml` returns match; `grep 'cancel-in-progress: false' .github/workflows/nightly.yml` returns match
- AC-004: `grep 'change_guard' .github/workflows/nightly.yml` returns match in step `id:` field; step exits 0 when HEAD unchanged
- AC-005: `grep 'prism-bin/Cargo.toml' .github/workflows/nightly.yml` returns match (BASE-MATCH source); `grep 'nightly\.' .github/workflows/nightly.yml` confirms tag form
- AC-006: `grep 'RELEASE_PROMOTE_TOKEN' .github/workflows/nightly.yml` ≥ 3 matches (checkout token + GH_TOKEN env); `grep 'GITHUB_TOKEN' .github/workflows/nightly.yml` returns NO match
- AC-007: `grep 'tail -n +15' .github/workflows/nightly.yml` ≥ 1 match; `grep 'nightly\.\[0-9\]' .github/workflows/nightly.yml` regex pattern present in retention step
- AC-008: `grep 'edge-nightly' .github/workflows/nightly.yml` ≥ 5 matches; `grep 'refs/tags/edge-nightly' .github/workflows/nightly.yml` present; no `git push origin HEAD` or branch write present
- AC-009: `grep 'IMPLEMENTED' docs/RELEASE-CHANNELS.md | grep -c nightly` ≥ 3 (§2 nightly row, §6 nightly row, §7 edge-nightly row)
- AC-010: Live verification step — human-in-loop gate at Phase-verify

---

## Tasks

The following tasks reflect the AS-BUILT implementation (retroactive materialization):

1. **Create `.github/workflows/nightly.yml`** with:
   - `name: Nightly`
   - Triggers: `schedule: cron '17 7 * * *'` (07:17 UTC daily, off-peak) + `workflow_dispatch`
   - Concurrency group `nightly-build` with `cancel-in-progress: false`
   - `permissions: contents: write`

2. **Configure checkout step** with `ref: develop`, `fetch-depth: 0` (full history required
   for change-guard tag walk and retention tag list), `token: RELEASE_PROMOTE_TOKEN`.
   Add `GH_TOKEN: RELEASE_PROMOTE_TOKEN` as a job-level env var so gh CLI uses the PAT
   for release create/delete operations.

3. **Configure git bot identity step**: `git config user.name "prism-release-bot"` +
   `git config user.email "prism-release-bot@users.noreply.github.com"` for annotated tag authorship.

4. **Implement change-guard step** (`id: change_guard`):
   - List nightly tags via `git tag --list "v*-nightly.*" --sort=-creatordate | head -1`
   - If no prior nightly tag: emit `proceed=true` to `$GITHUB_OUTPUT`; early-proceed
   - If prior nightly tag exists: dereference via `git rev-list -n 1 $LATEST_NIGHTLY`
     (handles annotated tags); compare to `git rev-parse HEAD`
   - If identical: emit `proceed=false`; print `::notice::No changes since last nightly`
   - If different: emit `proceed=true`

5. **Implement tag computation step** (`id: compute_tag`; gated on `proceed == 'true'`):
   - Read prism-bin version: `grep -m1 '^version = ' crates/prism-bin/Cargo.toml | sed 's/version = "//;s/"//'`
   - Strip channel suffix: `PRISM_BIN_CORE="${PRISM_BIN_VERSION%%-*}"` (BASE-MATCH per RELEASE-CHANNELS.md §3)
   - Build base: `BASE_TAG="v${PRISM_BIN_CORE}-nightly.$(date -u +%Y%m%d)"`
   - Same-day collision guard: while `git ls-remote --tags origin "${TAG}"` matches, append `.N`
   - Export `TAG`, `PRISM_BIN_CORE`, `TODAY` to `$GITHUB_ENV` and `$GITHUB_OUTPUT`

6. **Implement annotated tag creation and PAT push step** (gated on `proceed == 'true'`):
   - `git tag -a "${TAG}" -m "Nightly ${TAG} — tagged on develop (SHA: ${DEVELOP_SHA})"`
   - `git push origin "${TAG}"` — the PAT credential from the checkout step causes this
     push to fire `release.yml` (GITHUB_TOKEN push would not trigger downstream on:push:tags)

7. **Implement retention step** (gated on `proceed == 'true'`):
   - List: `git tag --list "v*-nightly.*" --sort=-creatordate`
   - Candidates to delete: `tail -n +15` (skip 14 newest)
   - TWO-LAYER safety filter per candidate: (1) glob `v*-nightly.*` pre-filtered the list
     already; (2) regex double-check `^v[0-9]+\.[0-9]+\.[0-9]+-nightly\.[0-9]{8}` — skip
     with `::warning::` if non-matching
   - For each confirmed candidate: `gh release delete "${old_tag}" --yes 2>/dev/null` (best-effort)
     then `git push origin ":refs/tags/${old_tag}" 2>/dev/null`

8. **Implement edge-nightly update step** (gated on `proceed == 'true'`):
   - Delete prior edge-nightly release: `gh release delete edge-nightly --yes 2>/dev/null` (best-effort)
   - Delete prior edge-nightly tag: `git push origin :refs/tags/edge-nightly 2>/dev/null` (best-effort)
   - Create new annotated tag: `git tag -f edge-nightly -a -m "edge-nightly — latest nightly is ${TAG}"`
   - Push: `git push origin edge-nightly`
   - Create GitHub Release: `gh release create edge-nightly --title "edge-nightly → ${TAG}" --notes-file "$EDGE_NOTES_FILE" --prerelease`
   - Release body references the dated nightly `${TAG}` release for actual build artifacts;
     includes reproducibility notice (pin to dated tag, not edge-nightly)

9. **Add job summary step** (`if: always()` — runs even on failure for observability):
   - Emits `$GITHUB_STEP_SUMMARY` table with trigger, tag computed, SHA tagged, latest prior nightly, prism-bin core
   - Three outcome branches: tag-pushed, no-op (no changes), workflow did not complete

10. **Update `docs/RELEASE-CHANNELS.md`** §2/§6/§7:
    - §2 nightly row: status column `PLANNED` → `IMPLEMENTED ('nightly.yml')`
    - §6 nightly row: status column → `IMPLEMENTED ('nightly.yml')`
    - §7 edge-nightly row: status column → `IMPLEMENTED in 'nightly.yml' step 8`
    - Bump document version and changelog row

11. **Verify AC-001..AC-009.** Document results in PR description.

---

## Acceptance Criteria

### AC-001: nightly.yml file exists and is actionlint-clean
`ls .github/workflows/nightly.yml` exits 0. `actionlint .github/workflows/nightly.yml`
exits 0 with no errors. (traces to RELEASE-CHANNELS.md §2 — nightly channel requires a
dedicated scheduled workflow)

### AC-002: Both triggers present — scheduled cron and workflow_dispatch
`grep -c 'cron:' .github/workflows/nightly.yml` ≥ 1; the cron expression is
`'17 7 * * *'` (07:17 UTC). `grep -c 'workflow_dispatch' .github/workflows/nightly.yml` ≥ 1.
(traces to RELEASE-CHANNELS.md §2 — "Scheduled cron (nightly)" trigger with workflow_dispatch
for manual verification runs)

### AC-003: Concurrency group nightly-build with cancel-in-progress: false
`grep 'nightly-build' .github/workflows/nightly.yml` returns a match inside the
`concurrency:` block. `grep 'cancel-in-progress: false' .github/workflows/nightly.yml`
returns a match. (traces to risk_mitigations "Concurrency group" — queuing rather than
cancelling protects in-progress tag pushes)

### AC-004: Change-guard step present; skips when develop HEAD is unchanged
A step with `id: change_guard` exists. The step emits `proceed=false` to `$GITHUB_OUTPUT`
when `git rev-list -n 1 $LATEST_NIGHTLY` equals `git rev-parse HEAD` (develop unchanged).
`grep 'change_guard' .github/workflows/nightly.yml` ≥ 2 matches (step id + downstream
`if:` conditions). (traces to RELEASE-CHANNELS.md §2 — "only runs if develop changed since
last nightly")

### AC-005: Tag computation applies BASE-MATCH and same-day monotonic collision guard
The compute_tag step reads `crates/prism-bin/Cargo.toml` as the version source and uses
`%%-*` shell parameter expansion to strip the channel suffix (BASE-MATCH). The resulting
tag is `v${PRISM_BIN_CORE}-nightly.${TODAY}`. A while-loop checks `git ls-remote --tags
origin` and appends `.N` (N ≥ 2) when the base tag already exists.
`grep 'prism-bin/Cargo.toml' .github/workflows/nightly.yml` returns a match.
`grep '%%.*\*' .github/workflows/nightly.yml` returns a match (channel-suffix strip).
(traces to RELEASE-CHANNELS.md §3 — BASE-MATCH tag guard; §2 — tag form `X.Y.Z-nightly.YYYYMMDD`)

### AC-006: RELEASE_PROMOTE_TOKEN used for checkout token and GH_TOKEN; GITHUB_TOKEN absent
`grep 'RELEASE_PROMOTE_TOKEN' .github/workflows/nightly.yml` ≥ 3 matches (checkout token
field, GH_TOKEN env assignment, one other reference). `grep 'GITHUB_TOKEN'
.github/workflows/nightly.yml` returns NO match. (traces to RELEASE-CHANNELS.md §5 — PAT
required to trigger downstream release.yml; risk_mitigations "PAT is REQUIRED" bullet)

### AC-007: Retention keeps last 14; TWO-LAYER nightly-only safety filter
The retention step uses `tail -n +15` to select candidates beyond position 14 (14-build
retention). A regex double-check anchored to `^v[0-9]+\.[0-9]+\.[0-9]+-nightly\.[0-9]{8}`
is present in the retention step body — non-matching candidates emit `::warning::` and are
skipped. `grep 'tail -n +15' .github/workflows/nightly.yml` returns a match.
`grep 'nightly\.\[0-9\]' .github/workflows/nightly.yml` returns a match in the regex pattern.
(traces to RELEASE-CHANNELS.md §6 — "keep last 14 nightly Releases and their tags; auto-delete
older; deletion filter cannot match alpha/beta/rc/stable/edge tags")

### AC-008: edge-nightly force-update uses delete-then-repush on tag ref ONLY; no branch or main writes
The edge-nightly step deletes the prior GitHub Release (`gh release delete edge-nightly`)
and prior tag (`git push origin :refs/tags/edge-nightly`) before recreating them.
`grep 'refs/tags/edge-nightly' .github/workflows/nightly.yml` ≥ 1 match (tag delete).
`grep 'git push origin edge-nightly' .github/workflows/nightly.yml` ≥ 1 match (tag push).
`grep 'git push origin' .github/workflows/nightly.yml | grep -v 'refs/tags\|edge-nightly'`
returns NO match — no branch refs are pushed. (traces to RELEASE-CHANNELS.md §7 —
edge-nightly mutable pointer updated on each run; RELEASE-CHANNELS.md §1 — no main write)

### AC-009: RELEASE-CHANNELS.md §2/§6/§7 nightly rows marked IMPLEMENTED
`grep 'IMPLEMENTED' docs/RELEASE-CHANNELS.md | grep -c 'nightly'` ≥ 3. The §2 nightly
row status column reads `IMPLEMENTED ('nightly.yml')`. The §6 nightly row status column
reads `IMPLEMENTED`. The §7 edge-nightly row status column reads `IMPLEMENTED`.
(traces to RELEASE-CHANNELS.md §2/§6/§7 — implementation status tracking for the channel
strategy document)

### AC-010: Live verification — workflow_dispatch run succeeds end-to-end
A `workflow_dispatch` run on the feature branch (or post-merge develop): tags develop HEAD
with a `vX.Y.Z-nightly.YYYYMMDD` tag (matching BASE-MATCH pattern), fires `release.yml`
which builds the 4-target matrix and publishes a pre-release GitHub Release, and creates
or updates the `edge-nightly` tag and GitHub Release pointing at the new dated nightly.
The job summary step outputs the tag, SHA, and trigger fields correctly.

This AC is satisfied at the Phase-verify step (human-in-loop gate). The implementer does
NOT need to satisfy this AC before pushing the PR — it is verified by the release engineer
after the PR lands on develop.

---

## Previous Story Intelligence

| Story | Key Decisions | Patterns Established | Gotchas Discovered |
|-------|--------------|---------------------|-------------------|
| S-REL-001 | release.yml repaired for pre-release handling | release.yml handles `v*` tag push → 4-target build + GitHub Release; `--prerelease` on hyphenated tags | Full checkout depth (`fetch-depth: 0`) required for tag operations |
| S-REL-DROP-INTEL-MAC-001 | 4-target build matrix (dropped x86_64-apple-darwin) | release.yml 4-target: aarch64-apple-darwin + x86_64-unknown-linux-gnu + x86_64-unknown-linux-musl + x86_64-pc-windows-msvc | Intel Mac target is permanently dropped; nightly inherits this by firing release.yml |

The PAT mechanism (RELEASE_PROMOTE_TOKEN) was established alongside `release-tag.yml`
(the ad-hoc pre-release tagging companion workflow). Nightly mirrors the same checkout +
PAT push pattern from release-tag.yml. Read release-tag.yml before authoring nightly to
ensure PAT wiring consistency.

---

## Architecture Compliance Rules

| Rule | Source | Enforcement |
|------|--------|-------------|
| Pre-release channels tag `develop` only — no `main` write | RELEASE-CHANNELS.md §1 | AC-008 grep confirms no branch push; no `main` ref in any push command |
| BASE-MATCH: X.Y.Z core from prism-bin/Cargo.toml; channel suffix stripped | RELEASE-CHANNELS.md §3 | AC-005 grep for `prism-bin/Cargo.toml` and `%%-*` strip |
| Full 4-target build matrix for every channel | RELEASE-CHANNELS.md §5 | Downstream release.yml (unchanged) owns the matrix; nightly triggers it via tag push |
| Nightly retention: keep last 14; two-layer nightly-only filter | RELEASE-CHANNELS.md §6 | AC-007 grep for `tail -n +15` and anchored regex pattern |
| edge-nightly is NOT a v* tag — does NOT fire release.yml | RELEASE-CHANNELS.md §7 | Tag name `edge-nightly` lacks `v` prefix; confirmed by AC-008 |
| RELEASE_PROMOTE_TOKEN (PAT) required for tag push to trigger release.yml | risk_mitigations §1 | AC-006 grep confirms no GITHUB_TOKEN |
| Nightly is ungated — no human approval required | RELEASE-CHANNELS.md §8 | No GitHub Environment gate in nightly.yml |

---

## Library & Framework Requirements

| Dependency | Version | Notes |
|------------|---------|-------|
| `actions/checkout` | `de0fac2e4500dabe0009e67214ff5f5447ce83dd` (v6.0.2) | SHA-pinned per project CI hardening; `fetch-depth: 0` required for full tag history |
| `gh` CLI | pre-installed on ubuntu-latest | Used for `gh release create/delete`; authenticated via `GH_TOKEN: RELEASE_PROMOTE_TOKEN` |
| `RELEASE_PROMOTE_TOKEN` | repository secret | PAT with `contents: write` (fine-grained) or `repo` (classic); same secret as release-tag.yml |

---

## File Structure Requirements

| File | Action | Notes |
|------|--------|-------|
| `.github/workflows/nightly.yml` | Create | Full workflow: 9 steps + concurrency block |
| `docs/RELEASE-CHANNELS.md` | Modify | §2 nightly row + §6 nightly row + §7 edge-nightly row: PLANNED→IMPLEMENTED; document changelog row added |
| `.github/workflows/release.yml` | DO NOT modify | Tag-triggered 4-target build; nightly fires it unchanged |
| `.github/workflows/release-tag.yml` | DO NOT modify | Reference only — PAT mechanism established there |

---

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| `nightly.yml` | `.github/workflows/` | Effectful (CI workflow; pushes tags; deletes/creates GitHub Releases) |
| `docs/RELEASE-CHANNELS.md` | `docs/` | Pure (documentation; no runtime behavior) |

---

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| `.github/workflows/nightly.yml` | effectful-shell | CI workflow; tags origin; creates/deletes GitHub Releases |
| `docs/RELEASE-CHANNELS.md` | pure-core | Documentation file; no runtime behavior |

---

## Holdout Applicability

**Determination: N/A**

The story-level holdout gate (CLAUDE.md, human-approved 2026-07-13) requires a built
prism binary and an MCP-visible surface with wire-level assertions. This story has
`tdd_mode: facade` and `crates_touched: []` — it produces no prism binary. Its entire
deliverable is `.github/workflows/nightly.yml` (a GitHub Actions YAML file) and a status
update to `docs/RELEASE-CHANNELS.md`.

All verification surfaces are structural: AC-001 through AC-009 are grep/inspection checks
on the workflow file and documentation. AC-010 (live verification) is a CI run on
`workflow_dispatch` that verifies the workflow tags develop and triggers release.yml — the
prism binary built by release.yml is not MCP-scoped to nightly.yml's touched surface.

Consistent with prior infra-story determinations for S-REL-CLIFF-001, S-REL-WRITER-001,
S-REL-DROP-INTEL-MAC-001 (all holdout_scenarios: []).

`holdout_scenarios: []` — no scenarios authored; HOLDOUT-INDEX unchanged.

---

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | No prior nightly tag exists (first nightly run) | Change-guard emits `LATEST_NIGHTLY=""` and `proceed=true`; tag computation uses today's date for the base tag; no prior-tag dereference attempted |
| EC-002 | develop has not changed since last nightly (same-SHA run) | Change-guard emits `proceed=false`; all downstream steps are skipped (`if: steps.change_guard.outputs.proceed == 'true'`); job exits 0 with `::notice::No changes` |
| EC-003 | Same-day collision — develop advances twice in one UTC day | Tag computation loop finds `vX.Y.Z-nightly.YYYYMMDD` exists on origin; appends `.2`; if `.2` also exists, appends `.3`; continues until a unique tag is found |
| EC-004 | edge-nightly GitHub Release does not exist yet (first run) | `gh release delete edge-nightly --yes 2>/dev/null` is best-effort; on non-zero exit the error is suppressed and the step continues (failure-tolerant delete pattern); no failure propagated |
| EC-005 | release.yml build fails after nightly tag is pushed | The dated nightly tag is permanent; edge-nightly GitHub Release body references the dated release URL; release.yml failure is visible in Actions UI but does not affect the nightly.yml job (it has already completed) |
| EC-006 | Retention step finds fewer than 14 nightly tags | `tail -n +15` produces empty output; the empty-output guard sets `TO_DELETE` to an empty string; the "nothing to delete" branch logs and exits cleanly |
| EC-007 | A nightly tag exists that does not match the anchored regex (e.g. from a test) | The regex double-check emits `::warning::Skipping '${old_tag}'` and `continue`; the non-matching tag is NOT deleted |

---

## Changelog

| Version | Date | Author | Change |
|---------|------|--------|--------|
| 1.0 | 2026-09-08 | story-writer | Initial retroactive materialization — RELEASE-CHANNELS.md §2/§3/§5/§6/§7/§8; as-built nightly.yml with 9-step workflow |
