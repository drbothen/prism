# Releasing Prism

This document is the canonical procedure for cutting a Prism release. Follow it
exactly. Do not improvise.

---

## Table of Contents

1. [Versioning Philosophy](#1-versioning-philosophy)
2. [Branch Strategy and Release Flow](#2-branch-strategy-and-release-flow)
3. [Mandatory Invariants](#3-mandatory-invariants)
4. [Step-by-Step: Cutting a Release](#4-step-by-step-cutting-a-release)
5. [Release Notes Convention](#5-release-notes-convention)
6. [Recovery Procedures](#6-recovery-procedures)
7. [Appendix: What Is NOT Released in v1.0.0](#appendix-what-is-not-released-in-v100)

---

## 1. Versioning Philosophy

### Product version vs. library versions

The git tag `vX.Y.Z` is the **product / distribution version** — a snapshot of the
complete, compiled, packaged Prism as deployed to operators. It answers: "what
version of Prism is running on this machine?"

`prism-bin` is the only crate that carries the product version as its own `version`
field in `Cargo.toml`. When a release is cut, **bump `prism-bin` to match the tag**.
This makes `prism --version` report the correct product version.

All other workspace crates (prism-core, prism-query, prism-spec-engine, prism-sensors,
etc.) carry **independent semver versions on their own cadence**. They are all
`publish = false`. You never force-align library crate versions to the product tag.
A library crate version bumps when its public API or behavior changes, entirely
independent of when a product release is cut.

A product release is a point-in-time snapshot: it captures whatever version of each
library crate happened to be current at release time. The tag is the coordination
anchor, not a version decree over internal crates.

### Pre-1.0.0 note

The current workspace started at `prism-bin = "0.1.0"`. The first public release is
`v1.0.0`; `prism-bin` was bumped to `1.0.0` at that point. The `0.x`
workspace versions used during development do not correspond to distribution versions.

### Pre-release tags

A tag containing a hyphen (e.g., `v1.0.0-rc.1`, `v1.1.0-beta.2`) is automatically
marked as a pre-release by the release workflow. Tags without a hyphen are treated as
stable releases and are marked Latest on GitHub.

---

## 2. Branch Strategy and Release Flow

### Branch model

| Branch | Purpose |
|--------|---------|
| `develop` | Active development. All feature/fix PRs target `develop`. |
| `main` | Releases only. Every commit on `main` is a release point. |
| `feature/<story-id>` | Per-story work off `develop`. |
| `release/vX.Y.Z` | Short-lived release-prep branch off `develop`. |
| `factory-artifacts` | Orphan branch. Mounted at `.factory/` as a worktree. Never touched during release. |

### High-level flow

```
develop (release-ready HEAD)
    │
    ├─► release/vX.Y.Z branch
    │       version bump + CHANGELOG authored
    │       PR → develop (CI must pass)
    │
    ▼
develop (merge commit with bump + CHANGELOG)
    │
    ├─► PR develop → main (CI must pass — all 24 required checks)
    │       HUMAN MERGES — no auto-merge, no harness merge
    │
    ▼
main
    │
    ├─► git tag -a vX.Y.Z on main HEAD
    ├─► git push origin vX.Y.Z  ← triggers release.yml
    │
    ▼
GitHub Release (automated by release.yml)
    5-platform binaries + checksums.txt + attestations
```

### Why develop → main is human-only

The harness auto-mode classifier treats a develop→main merge as a deployment event
that crosses an environment boundary. The project git-safety protocol also prohibits
force-pushing main under any circumstances and requires explicit human approval for
all merges to main. No CI automation or agent may perform the develop→main merge.

---

## 3. Mandatory Invariants

Violating any of these breaks the release. All are enforced by CI, branch
protection, or project convention — not just policy.

| Invariant | Consequence of violation |
|-----------|--------------------------|
| Never force-push `main` | Branch protection blocks it; history corruption |
| Never skip git hooks (`--no-verify`) | TD-FACTORY-HOOK-BYPASS-001 P0 violation |
| No AI attribution in commits | Project convention; see CLAUDE.md §Git Workflow |
| Tag must live on `main`, not `develop` | release.yml triggers on `v*` tags; a tag on develop produces a release from the wrong base |
| Conventional commit for the release-prep commit | Enforced by lefthook pre-commit hook |
| All 24 required CI status checks must pass on develop before PR to main | Branch protection on both develop and main |
| crates.io / Chocolatey / Homebrew publishing is deferred post-v1 | All workspace crates carry `publish = false`; no tap exists; do NOT attempt registry publish for v1.0.0 |
| `prism-dtu-demo-server` is included in the release build | release.yml builds both `-p prism-bin -p prism-dtu-demo-server`; the demo-server archive is retained as a workflow artifact and is NOT uploaded as a GitHub Release asset |

### Required CI status checks (both develop and main)

These 24 checks must all pass before a PR merges to either branch. The names below
match the `name:` fields in the workflow files and the configured branch-protection
context strings. To retrieve the exact strings from the live branch protection rule:

```bash
gh api repos/drbothen/prism/branches/develop/protection \
  --jq '.required_status_checks.contexts[]'
```

- ADR-023 No-Hardcoded-Sensors compile-fail gate (AC-006 PLUGIN-MIGRATION-001-F)
- Cargo audit (RustSec)
- Cargo deny (license + advisory)
- Clippy (AD-008)
- Deep-recursion test stack-guard lint (OBS-002)
- E2E smoke
- Format check
- Fuzz smoke (vp021_parse_fuzz)
- Non-exhaustive violation compile-fail check (AC-5 S-PLUGIN-PREREQ-C)
- Perimeter compile-fail check (BC-2.11.006 v1.10)
- Perimeter symbols sync check (BC-2.11.006 OBS-001)
- Release gate (S-REL-001 AC-012)
- Semver compatibility
- Shellcheck demo scripts (S-DEMO-003 HIGH-2 / AC-014)
- Test (aarch64-apple-darwin)
- Test (no-default-features)
- Test (x86_64-apple-darwin)
- Test (x86_64-pc-windows-msvc)
- Test (x86_64-unknown-linux-gnu)
- Test (x86_64-unknown-linux-musl)
- ThreatIntel .prx staleness guard (F-MCPNULL-P2-OBS-002)
- Verify workflow structure (reachability + config invariants, 25 assertions)
- WASM32 compile check + .prx build (S-PLUGIN-CI-001 AC-001)
- Workspace crate layout (ADR-012)

---

## 4. Step-by-Step: Cutting a Release

### Prerequisites

- `gh` CLI authenticated to drbothen/prism with `contents: write` scope
- `git` with GPG or SSH signing configured (if project requires signed tags)
- Develop HEAD is confirmed release-ready (all CI green, no open blocking defects)
- Human approval to proceed

### Step 1 — Create the release-prep branch

```bash
git fetch origin
git checkout -b release/vX.Y.Z origin/develop
```

Replace `X.Y.Z` throughout with the actual version (e.g., `1.0.0`).

### Step 2 — Bump prism-bin version

Edit `crates/prism-bin/Cargo.toml`:

```toml
[package]
version = "X.Y.Z"   # was 0.1.0 or previous version
```

Regenerate both lockfiles that the version bump invalidates, then confirm the
build compiles. Both lockfiles must be committed — `release.yml` runs
`cargo build --locked` for every build leg.

```bash
# Regenerate both lockfiles that the version bump invalidates.
# Both must be committed — release.yml runs `cargo build --locked`.
cargo update -p prism-bin --precise X.Y.Z
cargo update --manifest-path tests/external/non-exhaustive-violation/Cargo.toml \
             -p prism-bin --precise X.Y.Z
cargo check -p prism-bin --locked   # now passes
```

### Step 3 — Author the CHANGELOG entry

If `CHANGELOG.md` does not yet exist, create it with this header:

```markdown
# Changelog

All notable changes to Prism are documented here.
Format: [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
```

Add the new release section immediately below `[Unreleased]` (and above any prior entries):

```markdown
## [X.Y.Z] - YYYY-MM-DD

### Added
- ...

### Fixed
- ...

### Changed
- ...

### Security
- ...
```

Pull the content from `git log` since the last tag (or since project inception for
the first release):

```bash
git log --oneline $(git describe --tags --abbrev=0 2>/dev/null || git rev-list --max-parents=0 HEAD)..HEAD
```

Group by type: feat commits → Added, fix commits → Fixed, etc. Include story IDs
and PR numbers where they exist. Be concise but specific enough that an operator can
understand what changed without reading the full diff.

### Step 4 — Commit and push the release-prep branch

```bash
git add crates/prism-bin/Cargo.toml \
        Cargo.lock \
        tests/external/non-exhaustive-violation/Cargo.lock \
        CHANGELOG.md
git commit -m "chore(release): bump prism-bin to vX.Y.Z, update Cargo.lock, add CHANGELOG entry"
git push -u origin release/vX.Y.Z
```

Lefthook runs fmt + clippy + crate-layout on pre-commit and the full `just check`
on pre-push. Do not skip them.

### Step 5 — Open the release-prep PR to develop

```bash
gh pr create \
  --base develop \
  --head release/vX.Y.Z \
  --title "chore(release): vX.Y.Z release prep" \
  --body "$(cat <<'EOF'
## Release prep: vX.Y.Z

- Bumps `prism-bin` to `X.Y.Z`
- Adds CHANGELOG entry

## Checklist
- [ ] All 24 required CI checks pass
- [ ] CHANGELOG entry reviewed for accuracy
- [ ] prism-bin version matches the intended tag
EOF
)"
```

Wait for all required CI checks to pass. Merge the PR to develop (squash or merge
commit — follow project convention for this PR type). Do not merge until CI is
fully green.

### Step 6 — Preview with /vsdd-factory:release --dry-run (optional)

Before the develop→main PR, run the release skill in dry-run mode to confirm the
release artifact list, version, and any automation steps:

```
/vsdd-factory:release --dry-run
```

Review its output. If it reports unexpected findings, resolve them before proceeding.

### Step 7 — Open the develop → main PR

This PR is human-reviewed and human-merged. No automation performs this merge.

```bash
git fetch origin
gh pr create \
  --base main \
  --head develop \
  --title "release: vX.Y.Z" \
  --body "$(cat <<'EOF'
## Release vX.Y.Z

Merges develop into main to cut the vX.Y.Z release.

All 24 required CI checks must pass before merge. Merging this PR is a
human action — do not use auto-merge.

After merge: create and push the annotated tag on main.
EOF
)"
```

All 24 required status checks must pass on this PR. Do not merge if any check is
red.

### Step 8 — Human merges develop → main

The human (Joshua) reviews and merges the PR in the GitHub UI. Merge strategy:
merge commit (preserves history lineage). Do not squash the develop→main merge.

After merge, pull the updated main locally:

```bash
git fetch origin
git checkout main
git pull origin main
```

Confirm the tip commit is the merge commit:

```bash
git log --oneline -3 main
```

### Step 9 — Create and push the annotated tag

The tag must be created on the `main` HEAD after the merge commit is confirmed.

```bash
git tag -a vX.Y.Z -m "$(cat <<'EOF'
Release vX.Y.Z

See CHANGELOG.md for the full change list.
EOF
)"
```

Verify the tag points to the correct commit:

```bash
git log --oneline -1 vX.Y.Z
```

Push the tag. This triggers `release.yml`:

```bash
git push origin vX.Y.Z
```

**Do not push any other commits to main at this moment.** The tag push alone is what
triggers the release workflow.

### Step 10 — Watch the release workflow

```bash
gh run watch --repo drbothen/prism
```

The workflow runs the 5-platform build matrix in parallel:

| Target | Runner | Archive |
|--------|--------|---------|
| `aarch64-apple-darwin` | `macos-latest` | `.tar.gz` |
| `x86_64-apple-darwin` | `macos-15-intel` | `.tar.gz` |
| `x86_64-unknown-linux-gnu` | `ubuntu-latest` | `.tar.gz` |
| `x86_64-unknown-linux-musl` | `ubuntu-latest` | `.tar.gz` |
| `x86_64-pc-windows-msvc` | `windows-latest` | `.zip` |

The musl leg uses `cargo-zigbuild` to avoid glibc symbol contamination
(DEFECT-REL001-MUSL-LIBSTDCXX-001). All legs build both `prism-bin` and
`prism-dtu-demo-server`.

Expected total wall-clock time: approximately 30–45 minutes (60-minute per-job
timeout). The publish-release job runs after all 5 build legs succeed.

### Step 11 — Verify the GitHub Release

```bash
gh release view vX.Y.Z --repo drbothen/prism
```

Verify all of the following before declaring the release complete:

1. **5 platform archives** are attached (`prism-vX.Y.Z-<target>.tar.gz` x4 +
   `prism-vX.Y.Z-x86_64-pc-windows-msvc.zip`).
2. **`checksums.txt`** is attached (merged SHA-256 checksums from all 5 legs).
3. **Build-provenance attestations** are present for each archive (created by
   `actions/attest-build-provenance` during the build step, visible in the
   workflow run's artifact attestations, verifiable via `gh attestation verify`).
4. Release is marked **Latest**, not Pre-release (for stable tags without a hyphen).
5. Generated release notes (from `--generate-notes`) are present.

### Step 12 — Update README version badge and install block

The README version badge and install block are authored in the release-prep commit
(Steps 3/4). For the first release, this was done as part of this release-prep PR.
For subsequent releases, update the version badge and install URLs in `README.md`
to reference the new tag version and commit those changes to the release-prep branch
(Step 4) alongside the CHANGELOG entry and version bump — not in a separate
follow-up PR.

Specifically, update:
- The `[![vX.Y.Z](...)]` shield badge URL in the README header to match the new tag
- All install URL paths (e.g., `prism-v1.0.0-` → `prism-vX.Y.Z-`) in the `## Install` section

Stage the updated `README.md` as part of the `git add` in Step 4.

**Note:** The install URLs in `README.md` reference an unpublished release from
the moment the release-prep PR merges to `develop` (when the badge/install block
lands in the public tree) until `release.yml` finishes publishing the GitHub
Release. The full window spans: release-prep PR merge → develop→main PR review
and merge → annotated tag → ~45-minute release.yml build. On a public repository
this is typically a few hours, but may extend to days if tag-cutting is delayed.
This is expected; the README is the intended install surface, not a live endpoint.

---

## 5. Release Notes Convention

### Structure

The GitHub Release body combines two layers:

**Layer 1 — Curated human narrative (prepend manually after release creation):**

```markdown
## Prism vX.Y.Z

[One-paragraph summary of what this release delivers — value, not just a feature list.]

### Install

**macOS (Apple Silicon):**
curl -LO https://github.com/drbothen/prism/releases/download/vX.Y.Z/prism-vX.Y.Z-aarch64-apple-darwin.tar.gz
tar xzf prism-vX.Y.Z-aarch64-apple-darwin.tar.gz
chmod +x prism
./prism --version

**macOS (Intel):**
curl -LO https://github.com/drbothen/prism/releases/download/vX.Y.Z/prism-vX.Y.Z-x86_64-apple-darwin.tar.gz
tar xzf prism-vX.Y.Z-x86_64-apple-darwin.tar.gz

**Linux (glibc — most distros):**
curl -LO https://github.com/drbothen/prism/releases/download/vX.Y.Z/prism-vX.Y.Z-x86_64-unknown-linux-gnu.tar.gz
tar xzf prism-vX.Y.Z-x86_64-unknown-linux-gnu.tar.gz

**Linux (musl — Alpine, static binary):**
curl -LO https://github.com/drbothen/prism/releases/download/vX.Y.Z/prism-vX.Y.Z-x86_64-unknown-linux-musl.tar.gz
tar xzf prism-vX.Y.Z-x86_64-unknown-linux-musl.tar.gz

**Windows (x86_64):**
Download prism-vX.Y.Z-x86_64-pc-windows-msvc.zip from the assets below.

### Verify checksums

sha256sum -c checksums.txt

### Verify build provenance

gh attestation verify prism-vX.Y.Z-<target>.tar.gz \
  --repo drbothen/prism \
  --signer-workflow drbothen/prism/.github/workflows/release.yml
```

**Layer 2 — Auto-generated notes** from `--generate-notes` (appended automatically
by the workflow): lists PRs merged since the previous tag, grouped by label.

### Editing the release body after workflow completes

The workflow creates the release with `--generate-notes` but without the curated
narrative above. Edit the body in the GitHub UI to prepend the install/verify block:

```bash
gh release edit vX.Y.Z --repo drbothen/prism --notes-file /tmp/release-notes-vX.Y.Z.md
```

Or use the GitHub UI release editor.

---

## 6. Recovery Procedures

### A build-release matrix leg failed

If one or more of the 5 build legs fails and the `publish-release` job never ran
(it `needs: build-release` — a partial matrix failure means no release was created):

1. Diagnose the failure: `gh run view <run-id> --repo drbothen/prism --log-failed`
2. Fix the root cause on `main` (commit to `main` via develop→main flow or a
   direct hotfix PR to main — do NOT cherry-pick to avoid divergence). If a
   direct hotfix to `main` is taken, immediately back-merge `main` → `develop`
   via a separate PR to keep the branches in sync.
3. Delete the tag: `git push origin :vX.Y.Z && git tag -d vX.Y.Z`
4. Re-create the annotated tag on the fixed `main` HEAD: `git tag -a vX.Y.Z -m "..."`
5. Re-push: `git push origin vX.Y.Z`

**Caution on tag re-push:** deleting and re-creating a tag after any assets were
partially uploaded can cause confusion in downstream tooling. Prefer to fix via a
new patch tag (`vX.Y.(Z+1)`) rather than re-using the original tag if the original
release was publicly visible.

### The publish-release job failed mid-upload (partial assets)

The workflow's `publish-release` job handles this case with an idempotent re-run
path. If `gh release view $TAG` succeeds (the release exists but assets are
incomplete), the job runs `gh release upload $TAG --clobber` instead of `gh release
create`. To re-run:

```bash
gh run rerun <failed-run-id> --repo drbothen/prism --failed
```

The `--clobber` flag safely overwrites any partially uploaded assets. Re-running
without deleting the tag or the release is safe.

### The checksums.txt is missing or incorrect

`checksums.txt` is merged from per-leg `checksums.txt` artifacts in the
`publish-release` job. If it is absent or contains only some legs, re-run the
workflow (see above). Do not manually construct or upload a `checksums.txt` — it
must come from the CI build.

### A required CI check is red on the develop → main PR

Do not merge. Fix the underlying issue on `develop` via the normal feature/fix PR
flow. Return to Step 7 once all checks are green. There is no bypass for a failing
required check — lefthook's `--no-verify` is forbidden, and branch protection
enforces all 24 checks.

### Anything else

**STOP. Surface to the human.** Do not improvise around an unexpected release failure.
Document the exact failure mode, the workflow run ID, and the current state of the
release (tag present? GitHub Release created? assets uploaded?). The human decides
the recovery path.

---

## Appendix: What Is NOT Released in v1.0.0

The following distribution channels are deferred post-v1.0.0. Do not attempt to
publish to any of them for this release.

| Channel | Status | Re-enable path |
|---------|--------|---------------|
| **crates.io** | Deferred (DEF-REL-004). All workspace crates carry `publish = false`. | Remove `publish = false`, add `CRATES_IO_TOKEN` secret, reinstate `crates-io-publish` job in release.yml. Story: S-REL-008. |
| **Homebrew tap** | Deferred (DEF-REL-003). `1898co/homebrew-tap` does not exist. | Create the tap repo, add `HOMEBREW_TAP_TOKEN` secret, reinstate `homebrew-update` job. Story: S-REL-008. |
| **Chocolatey** | Deferred (DEF-REL-002). `packaging/chocolatey/` does not exist. | Create the packaging directory, add `CHOCOLATEY_API_KEY` secret, reinstate `chocolatey-publish` job. Story: S-REL-008. |

These deferrals are intentional and tracked in release.yml comments (DEF-REL-002,
DEF-REL-003, DEF-REL-004). Do not attempt to unblock them in the same release cycle
without a dedicated story and CI validation.
