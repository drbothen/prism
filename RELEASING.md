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

The `release-promote` workflow mechanically enforces this invariant: if the
dispatched `tag` input (e.g. `v1.0.0-rc.2`) does not exactly match the `prism-bin`
Cargo.toml `version` field (e.g. `1.0.0-rc.2`) on the develop tree, the promotion
fails with a clear error before anything is written to `main`.

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

### Default branch

`develop` is the default branch (changed 2026-09-04). This makes `workflow_dispatch`
workflows dispatchable against the active development branch without specifying
`--ref develop` each time in the GitHub UI, though the CLI commands in §4 include
the flag explicitly for clarity. `main` remains the release branch.

### High-level flow

```
develop (release-ready HEAD)
    │
    ├─► Dispatch release-prep.yml (--field version=X.Y.Z)
    │       Creates release/vX.Y.Z branch off develop
    │       Bumps prism-bin + scaffolds CHANGELOG
    │       Opens PR to develop — human reviews, curates, merges
    │
    ▼
develop (merged release-prep PR — CI all green)
    │
    ├─► Dispatch release-promote.yml (dry_run=true)  ← validates, nothing written
    │       merge + tree-identity + tag guard + version guard + push --dry-run
    │
    ├─► Dispatch release-promote.yml (dry_run=false)
    │       validate job (same guards, no approval)
    │       promote job → pauses at release-main environment gate
    │           required-reviewer approves in GitHub UI
    │       develop → main merge + annotated tag pushed to origin
    │
    ▼
main + vX.Y.Z tag
    │
    ├─► Tag push triggers release.yml
    │
    ▼
GitHub Release (automated by release.yml)
    5-platform binaries + checksums.txt + attestations
```

### Why develop → main goes through release-promote

The `release-promote` workflow handles the develop→main promotion with a
required-reviewer approval gate (the `release-main` GitHub Environment). No
manual PR to main is created; the environment gate IS the approval. The
`RELEASE_PROMOTE_TOKEN` (a PAT with Contents:write + Workflows:write) is required
because the default `GITHUB_TOKEN` cannot bypass main branch-protection rules such
as required status checks or required reviews.

### First-promotion note

The first promotion must handle the unrelated git histories between `main` (a
2-commit stub initialized independently) and `develop`. The `release-promote`
workflow always passes `--allow-unrelated-histories` to the merge; after the first
join it becomes a harmless no-op. The tree-identity safety gate confirms the merged
result equals `origin/develop` regardless.

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
| Tag input to `release-promote` must equal `prism-bin` Cargo.toml version | `release-promote` validate job fails; promotion is blocked. Mechanically enforced — see §1 note. |
| Conventional commit for the release-prep commit | Enforced by lefthook pre-commit hook (local) or by workflow convention (CI-generated commit) |
| All 24 required CI status checks must pass on develop | Branch protection on develop enforces this before the release-prep PR can merge |
| `RELEASE_PROMOTE_TOKEN` secret must be configured | `release-promote` cannot authenticate to push `main` or the semver tag (GITHUB_TOKEN cannot bypass main branch protection) |
| `release-main` GitHub Environment must exist with at least one required reviewer | The `promote` job runs without an approval gate (security regression); configure at Settings ▸ Environments ▸ release-main ▸ Required reviewers |
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

Follow this procedure exactly. Do not improvise. The two release workflows
(`release-prep.yml` and `release-promote.yml`) mechanize the steps that are
safe to automate; human decisions (CHANGELOG curation, final approval) remain
explicit gates.

Replace `X.Y.Z` throughout with the actual version (e.g. `1.0.0-rc.2`).

### Prerequisites

- `gh` CLI authenticated to drbothen/prism
- `RELEASE_PROMOTE_TOKEN` secret configured in the repository
  (Fine-Grained PAT: Contents:write + Workflows:write; or Classic PAT: `repo` + `workflow`)
- `release-main` GitHub Environment configured with at least one required reviewer
  (Settings ▸ Environments ▸ release-main ▸ Required reviewers)
- Develop HEAD is confirmed release-ready: all 24 required CI checks green, no open
  blocking defects
- Human approval to proceed

### Step 1 — Dispatch release-prep

```bash
gh workflow run release-prep.yml \
  --repo drbothen/prism \
  --ref develop \
  --field version=X.Y.Z
```

This workflow:
1. Creates branch `release/vX.Y.Z` off `develop`
2. Bumps `crates/prism-bin/Cargo.toml` `[package].version` to `X.Y.Z`
3. Updates both Cargo.lock files (root + `tests/external/non-exhaustive-violation/`)
4. Scaffolds a `## [X.Y.Z] - YYYY-MM-DD` CHANGELOG section seeded with merged-PR
   subjects since the previous tag
5. Commits, pushes the branch, and opens a PR targeting `develop`

Monitor the run:

```bash
gh run list --repo drbothen/prism --workflow release-prep.yml --limit 3
gh run watch --repo drbothen/prism
```

### Step 2 — Review and merge the release-prep PR

The PR body contains a checklist. Before merging:

1. Curate the CHANGELOG scaffold: categorize entries under Added / Fixed / Changed /
   Security / Removed; remove the `> **SCAFFOLD**` notice when done.
2. Confirm `prism-bin` version in `Cargo.toml` matches the intended tag (`vX.Y.Z`).
3. Update the README version badge and install URLs if this is not a pre-release:
   - The `[![vX.Y.Z](...)]` shield badge URL in the README header
   - All install URL paths (e.g. `prism-v1.0.0-rc.1-` → `prism-vX.Y.Z-`) in the `## Install` section
   Push these to the release branch before merging.
4. Wait for all 24 required CI checks to pass.
5. Merge the PR to `develop` (squash or merge commit per project convention).

### Step 3 — Dry-run release-promote (validate without writing)

```bash
gh workflow run release-promote.yml \
  --repo drbothen/prism \
  --ref develop \
  --field tag=vX.Y.Z \
  --field dry_run=true
```

The `validate` job runs the full gate sequence — merge, tree-identity check, tag
guard, version guard — then calls `git push --dry-run` to confirm auth and ref-update
validity. Nothing is written to origin. No approval required.

Review the job summary (Actions tab ▸ Release Promote ▸ latest run ▸ Summary).
All rows should show PASS / not "not reached". Confirm `prism-bin version` matches
the tag you dispatched.

### Step 4 — Real release-promote (promote develop → main + tag)

```bash
gh workflow run release-promote.yml \
  --repo drbothen/prism \
  --ref develop \
  --field tag=vX.Y.Z \
  --field dry_run=false
```

The `validate` job runs the same gates as the dry-run (no approval required). After
it succeeds, the `promote` job is queued and **pauses at the `release-main`
environment gate** for required-reviewer approval.

To approve:

1. Navigate to Actions tab ▸ Release Promote ▸ the new run.
2. Click **Review deployments**.
3. Select `release-main` and click **Approve and deploy**.

After approval the `promote` job:
- Deterministically re-performs the merge + version guard (idempotent: same
  develop HEAD, same `-X theirs` strategy → identical tree)
- Pushes `main` to origin
- Pushes tag `vX.Y.Z` to origin, which triggers `release.yml`

### Step 5 — Watch the release workflow

```bash
gh run watch --repo drbothen/prism
```

`release.yml` runs the 5-platform build matrix in parallel:

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
timeout). The `publish-release` job runs after all 5 build legs succeed.

### Step 6 — Verify the GitHub Release

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
6. **Bundled specs** are present in each archive. Spot-check a tar.gz:
   ```bash
   curl -sL https://github.com/drbothen/prism/releases/download/vX.Y.Z/prism-vX.Y.Z-x86_64-unknown-linux-gnu.tar.gz \
     | tar tzf - | grep -E 'specs/|infusions/|prism\.toml\.example'
   ```
   Expected: four `specs/*.sensor.toml` entries, two `infusions/*.infusion.toml`
   entries, and `prism.toml.example` — all at archive root level.

### Step 7 — Update the GitHub Release body

The workflow creates the release with `--generate-notes` but without the curated
install/verify narrative. Edit the body as described in §5 Release Notes Convention.

---

## 5. Release Notes Convention

### Structure

The GitHub Release body combines two layers:

**Layer 1 — Curated human narrative (prepend manually after release creation):**

```markdown
## Prism vX.Y.Z

[One-paragraph summary of what this release delivers — value, not just a feature list.]

### Install

Each release archive contains the `prism` binary plus bundled sensor and infusion
specs needed for a bootable installation — no source repository clone required.

**Archive contents:**
- `prism` (or `prism.exe` on Windows) — the compiled binary
- `prism.toml.example` — configuration template with inline instructions
- `specs/` — the four built-in sensor TOML specs (`armis`, `claroty`, `crowdstrike`, `cyberint`)
- `infusions/` — the two built-in infusion TOML specs (`threatintel`, `nvd`)

**Install steps (all platforms):**
1. Extract the archive (see platform commands below).
2. Copy `prism.toml.example` to `prism.toml` in your chosen config directory.
3. Copy `specs/` to the path you set as `spec_dir` in `prism.toml` (or leave it
   adjacent and set `spec_dir = "./specs"`).
4. Copy `infusions/` alongside `prism.toml` — it is auto-scanned from the config
   directory at boot (`{config_dir}/infusions/`).
5. Edit `prism.toml`: set `state_dir`, add `[[orgs]]` entries, configure credentials.
6. Run `prism start`.

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
Extract with Windows Explorer or: Expand-Archive prism-vX.Y.Z-x86_64-pc-windows-msvc.zip .

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
2. Fix the root cause on `develop` via the normal feature/fix PR flow, then
   back-merge `main` → `develop` via a direct hotfix PR if `main` has diverged.
   Do NOT cherry-pick to avoid divergence.
3. Delete the tag: `git push origin :vX.Y.Z && git tag -d vX.Y.Z`
4. Prefer a new patch tag (`vX.Y.(Z+1)`) if the original release was publicly visible
   to avoid confusion in downstream tooling. Otherwise re-dispatch `release-promote`
   with the original tag after deleting it from origin.

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

### A required CI check is red on the release-prep PR

Do not merge. Fix the underlying issue on `develop` via the normal feature/fix PR
flow. Rebase the release-prep branch onto develop and push before re-running CI.
Return to §4 Step 2 once all checks are green. There is no bypass for a failing
required check — `--no-verify` is forbidden (TD-FACTORY-HOOK-BYPASS-001) and branch
protection enforces all 24 checks.

### release-promote fails the version guard

If the `validate` job fails with `tag vX.Y.Z does not match prism-bin version Y.Z`:
- The release-prep PR's CHANGELOG curation was merged but the prism-bin version was
  not bumped, OR the wrong tag was dispatched.
- Fix: re-run release-prep with the correct version, or manually push a fix commit
  to `develop` bumping `crates/prism-bin/Cargo.toml` to the correct version, then
  re-dispatch `release-promote`.

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
