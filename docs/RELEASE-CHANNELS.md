# Release Channel Strategy

This document captures Prism's full release-channel strategy and maturity model.
It is the reference for understanding what each channel promises, how versions
flow through the ladder, and what criteria gate advancement.

**Operator-approved: 2026-09-04.**

> **Relationship to RELEASING.md:** `RELEASING.md` (repo root) is the operational
> stable-release runbook — step-by-step procedures for cutting a stable release using
> the implemented `release-prep.yml` and `release-promote.yml` workflows. This file is
> the channel strategy and reference. The pre-release lanes described here will be built
> as a dedicated epic after `v1.0.0-rc.1` ships.

---

## Table of Contents

1. [Purpose and Core Principle](#1-purpose-and-core-principle)
2. [Channel Model](#2-channel-model)
3. [Versioning Lifecycle](#3-versioning-lifecycle)
4. [Maturity and Advancement Rubric](#4-maturity-and-advancement-rubric)
5. [Build Path](#5-build-path)
6. [Retention Policy](#6-retention-policy)
7. [Edge Convenience Pointers](#7-edge-convenience-pointers)
8. [Gating Summary](#8-gating-summary)

---

## 1. Purpose and Core Principle

Prism releases continuously across a maturity ladder so a consumable, tagged build
always exists at every tier. Operators monitoring `develop` get a nightly snapshot.
Quality-conscious adopters get alpha/beta/rc builds. Production deployments consume
stable releases.

**Core principle:**

> Pre-release channels tag `develop` only — no `main` write, ungated.
> Only the stable channel promotes to `main`, and it is human-approval-gated via
> the `release-main` GitHub Environment.

This separates _release frequency_ (high — pre-release channels flow freely) from
_release authority_ (strictly controlled — only stable writes `main`).

---

## 2. Channel Model

| Channel | Promise | Trigger | Source branch | Human gate | Tag form | GitHub marking | Status |
|---------|---------|---------|---------------|------------|----------|----------------|--------|
| **nightly** | Current `develop` state — scheduled; only runs if `develop` changed since last nightly | Scheduled cron (nightly) | `develop` | None | `X.Y.Z-nightly.YYYYMMDD` | Pre-release | PLANNED |
| **dev** | Current `develop` state — ad-hoc snapshot for spot testing | `workflow_dispatch` | `develop` | None | `X.Y.Z-dev.<shortsha>` | Pre-release | PLANNED |
| **alpha** | First intentional pre-release of a new `X.Y.Z` line; may be feature-incomplete; API may move | `workflow_dispatch` | `develop` | None | `X.Y.Z-alpha.N` | Pre-release | PLANNED |
| **beta** | Feature-complete; feature freeze in effect; stabilization only | `workflow_dispatch` | `develop` | None | `X.Y.Z-beta.N` | Pre-release | PLANNED |
| **rc** | Code freeze; zero known release-blocking defects; LIVE-TENANT validated; CHANGELOG and docs final | `workflow_dispatch` | `develop` | None (light human review before dispatch) | `X.Y.Z-rc.N` | Pre-release | PLANNED |
| **stable** | Production-grade; fully soaked | `release-prep.yml` → `release-promote.yml` | `develop` → `main` | REQUIRED (`release-main` environment approval) | `X.Y.Z` | Latest | IMPLEMENTED |

### Implementation status notes

**STABLE (IMPLEMENTED):** The stable channel is fully operational via
`release-prep.yml` (creates the release-prep PR) and `release-promote.yml` (merges
`develop` → `main` and pushes the semver tag behind the `release-main` environment
gate). `release.yml` (tag-triggered) handles the 5-platform build and GitHub Release
creation. See `RELEASING.md` §4 for the complete operational runbook.

**All pre-release channels (PLANNED):** The nightly, dev, alpha, beta, and rc channels
are not yet implemented. They are designed to share the same `release.yml` build path
(tag-triggered) and require only lightweight dispatch workflows to be built. These will
be delivered as a dedicated release-channels epic after `v1.0.0-rc.1` ships.

---

## 3. Versioning Lifecycle

### Version carrier

`crates/prism-bin/Cargo.toml` is the **sole product-version crate**. All other
workspace crates carry independent semver versions on their own cadence and are not
touched during release preparation. See `RELEASING.md` §1 for the full versioning
philosophy.

### Base version convention

`develop` always carries a `X.Y.Z-dev` base version in `crates/prism-bin/Cargo.toml`
(e.g. `1.0.0-dev` during the v1.0.0 development cycle).

### Tag guard rules

Two distinct matching rules govern how the `release-promote.yml` version guard
validates the tag against the `prism-bin` version:

- **BASE-MATCH (pre-releases):** Compare only the `X.Y.Z` core; the channel suffix
  floats. A `prism-bin` version of `1.0.0-dev` satisfies any pre-release tag whose
  core is `1.0.0` — `v1.0.0-nightly.20260905`, `v1.0.0-alpha.1`, `v1.0.0-rc.2`, etc.
  This means pre-release tags require no version-bump churn between nightly builds.

- **EXACT-MATCH (stable):** The `release-prep.yml` workflow drops the `-dev` suffix
  before the stable release-prep PR: `1.0.0-dev` → `1.0.0`. The dispatched tag
  `v1.0.0` must equal this exactly. The `release-promote.yml` validate job enforces
  this with a strict string comparison.

### Worked example: v1.0.0 cycle

```
develop carries: prism-bin = "1.0.0-dev"

  → Dispatch: release-prepare-prerelease --channel nightly
      Tag: v1.0.0-nightly.20260905
      Guard: BASE-MATCH — "1.0.0" core matches prism-bin "1.0.0-dev"
      No version bump needed

  → Dispatch: release-prepare-prerelease --channel alpha --n 1
      Tag: v1.0.0-alpha.1
      Guard: BASE-MATCH — same core, no bump

  → Dispatch: release-prepare-prerelease --channel rc --n 1
      Tag: v1.0.0-rc.1
      Guard: BASE-MATCH — same core, no bump

  → Dispatch: release-prep.yml --field version=1.0.0
      release-prep PR: bumps prism-bin "1.0.0-dev" → "1.0.0"
      Tag after promote: v1.0.0
      Guard: EXACT-MATCH — "1.0.0" == "1.0.0" ✓

After stable ships, develop bumps to next cycle:
  prism-bin "1.0.0" → "1.1.0-dev"   (MINOR bump for next feature cycle)
```

Note: `release-prepare-prerelease` is the planned pre-release dispatch workflow
(PLANNED). Only `release-prep.yml` is currently IMPLEMENTED.

### Post-stable version bump

After a stable release merges to `main`, the next step on `develop` is a version
bump forward to the next development version (e.g. `1.0.0` → `1.1.0-dev`). This
resets the base for the next release cycle and ensures `develop` never carries a
version that looks stable.

---

## 4. Maturity and Advancement Rubric

Channel transitions are **deliberate human judgments**. The workflows only execute
the tag once a human decides to cut a channel build — advancement is never automated.
The criteria below define what a channel promises and what must be true before cutting
it.

| Channel | What it promises | Criteria to CUT / advance to |
|---------|-----------------|-------------------------------|
| **nightly / dev** | Current `develop` state. No promise beyond develop-is-always-green. May include incomplete features and unstable APIs. | `develop` CI is green. For nightly: `develop` has changed since the last nightly tag. For dev: operator wants an ad-hoc snapshot (no specific criteria). |
| **alpha** | First intentional pre-release of a new `X.Y.Z` line. Core features may be incomplete. API surface may move between alphas. Not suitable for production. | A significant chunk of planned features for the `X.Y.Z` cycle are merged and functional. Team judges that external pre-release feedback would be valuable. Develop CI green. Human decides to open the alpha. |
| **beta** | Feature-complete for the `X.Y.Z` line. Feature freeze in effect — no new features after beta.1; stabilization, bug fixes, and polish only. API surface is stable. | All planned features for `X.Y.Z` are merged and passing CI. Feature freeze committed by the team. Full CI green. Human declares feature-complete and cuts beta. |
| **rc** | Code freeze. Zero known release-blocking defects. Full CI green on all 24 required checks. LIVE-TENANT validation passed. CHANGELOG entries curated. Documentation final. Ready for production adoption. | Beta soak identified and fixed all known blocking defects. `develop` has zero open P0/P1 issues. LIVE-TENANT validation run completed successfully. CHANGELOG and docs reviewed and final. Full CI green. Human declares rc-ready. |
| **stable** | Production-grade. `rc.N` soaked for N days (operator-determined soak window) with no new blocking defects discovered. All release invariants in `RELEASING.md` §3 satisfied. | RC soak period complete. No new blocking defects in the soak window. Human approval via the `release-main` environment gate. |

---

## 5. Build Path

Every channel — including nightly — creates a tag. `release.yml` is triggered by any
`v*` tag push and performs the full build and GitHub Release creation.

**Auto-detection of pre-release status:** `release.yml` checks whether the tag
contains a hyphen (`v1.0.0-rc.1` has a hyphen; `v1.0.0` does not). Hyphenated tags
automatically receive `--prerelease` when creating the GitHub Release. Tags without a
hyphen are marked **Latest**.

**Full 5-platform matrix for every channel:** All five build targets are built for
every channel, including nightly. No reduced matrix for pre-release builds. This is
an operator decision — operators need pre-release builds on the same platforms they
deploy to.

| Target | Runner | Archive |
|--------|--------|---------|
| `aarch64-apple-darwin` | `macos-latest` | `.tar.gz` |
| `x86_64-apple-darwin` | `macos-15-intel` | `.tar.gz` |
| `x86_64-unknown-linux-gnu` | `ubuntu-latest` | `.tar.gz` |
| `x86_64-unknown-linux-musl` | `ubuntu-latest` | `.tar.gz` |
| `x86_64-pc-windows-msvc` | `windows-latest` | `.zip` |

The build path itself is shared and unchanged whether the channel is nightly or
stable. The only difference is how the tag is created (dispatch workflow vs.
`release-prep.yml` + `release-promote.yml`) and whether `main` is written.

---

## 6. Retention Policy

**Status: PLANNED.** Retention enforcement is not yet implemented. A cleanup step
will be built as part of the pre-release channels epic.

| Channel | Retention rule |
|---------|---------------|
| **nightly** | Keep the last 14 nightly Releases and their tags. Auto-delete older nightly Releases and tags. |
| **dev** | Keep approximately the last 10 dev builds, or delete builds older than a configurable number of days. Exact threshold to be determined during epic implementation. |
| **alpha** | Keep forever. |
| **beta** | Keep forever. |
| **rc** | Keep forever. |
| **stable** | Keep forever. |

Nightly and dev retention will be enforced by a cleanup step within the nightly/dev
dispatch workflow itself, or by a standalone scheduled cleanup workflow.

---

## 7. Edge Convenience Pointers

**Status: PLANNED.** Edge pointers are not yet implemented.

Immutable dated tags are the authoritative source of truth for every build — they
are auditable, permanent, and unambiguous. However, for fast-moving channels
(nightly and dev), operators often want a stable URL that always resolves to the
newest build without tracking tag names.

For the nightly and dev channels, a moving `edge` convenience pointer will be
maintained alongside the immutable dated tags:

- `edge-nightly` — a repointed tag and GitHub Release that always resolves to the
  newest nightly build.
- `edge-dev` — a repointed tag and GitHub Release that always resolves to the newest
  dev build.

**Mutability tradeoff:** Edge pointers are mutable — the tag is force-updated on each
new build. This means `git fetch --tags` will update the local ref, and a download URL
using the edge tag will resolve to a different binary over time. Operators who need
reproducibility must pin to the immutable dated tag (e.g. `v1.0.0-nightly.20260905`),
not the edge pointer. The edge pointer is a convenience for "give me the latest
nightly" use cases only.

---

## 8. Gating Summary

| Channel | Gated? | Gate mechanism |
|---------|--------|----------------|
| nightly | No | Flows freely; scheduled cron auto-triggers on develop change |
| dev | No | Flows freely; ad-hoc `workflow_dispatch` |
| alpha | No | Flows freely; ad-hoc `workflow_dispatch` |
| beta | No | Flows freely; ad-hoc `workflow_dispatch` |
| rc | No (light) | Flows freely; human makes the judgment call to dispatch |
| stable | YES | `release-main` GitHub Environment — required-reviewer approval in the GitHub Actions UI before `main` is written |

**Pre-release channels are deliberately ungated.** The design principle is
_release often at every tier_ — the cost of a bad nightly or alpha build is low
(it is automatically pre-release, never Latest), and the feedback value is high.
Gating pre-release channels would slow down the maturity ladder without meaningful
risk reduction.

**Only stable is gated.** The `release-main` environment gate plus the soak
requirement means a stable release requires both human approval and evidence of
stability from the rc soak. This is the only point where `main` is written and a
build is marked Latest.
