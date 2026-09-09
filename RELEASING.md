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
field in `Cargo.toml`. When a **stable** release is cut, **bump `prism-bin` to match the tag**.
This makes `prism --version` report the correct product version.

The `release-promote` workflow mechanically enforces this invariant: if the
dispatched `tag` input (e.g. `v1.0.0`) does not exactly match the `prism-bin`
Cargo.toml `version` field (the tag name minus the leading `v`) on the develop tree, the promotion
fails with a clear error before anything is written to `main`. This is the **stable** lane;
pre-release tags use `release-tag.yml` which applies BASE-MATCH (see Pre-release exception below).

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

A tag containing a hyphen (e.g., `vX.Y.Z-alpha.N`, `vX.Y.Z-beta.N`) is automatically
marked as a pre-release by the release workflow. Tags without a hyphen are treated as
stable releases and are marked Latest on GitHub.

### Pre-release exception (ADR-064 D2)

The `develop` branch carries `prism-bin` at version `1.0.0-dev`. **No `Cargo.toml`
version bump is needed between pre-releases on the same X.Y.Z cycle** (e.g., between
`vX.Y.Z-alpha.N` and `vX.Y.Z-beta.N`). Instead, the binary version is injected at build
time via `PRISM_VERSION`, resolved by `crates/prism-bin/build.rs` through the
following fallback chain:

1. `PRISM_BUILD_VERSION` env var (explicit override; must be semver-shaped —
   `X.Y.Z[-pre][+build]` — otherwise it is ignored and the chain falls through)
2. `GITHUB_REF_NAME` env var with a leading `v` stripped — **only on tag builds**
   (`GITHUB_REF_TYPE == "tag"`, or fallback: `GITHUB_REF` starts with `refs/tags/`).
   On non-tag runs (branch pushes, pull requests) this arm is skipped entirely and the
   resolver falls through to `CARGO_PKG_VERSION`, preventing branch names such as
   `develop` or `feature/S-3.01` from being baked into the binary.
3. `CARGO_PKG_VERSION` (local dev default; always `1.0.0-dev` on `develop`)

This means `prism --version` on `develop` reports `prism 1.0.0-dev` locally, and
**reports** `prism <tag-version>` on a tagged CI build — without requiring `prism-bin`
`Cargo.toml` to be updated for every pre-release tag.

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
    │       Bumps prism-bin + generates CHANGELOG
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
    4-platform binaries + checksums.txt + attestations
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
| Tag input to `release-promote` must equal `prism-bin` Cargo.toml version (stable lane only) | `release-promote` validate job fails; promotion is blocked. Mechanically enforced — see §1 note. Pre-release tags bypass this by using `release-tag.yml` directly (see §1 Pre-release exception). |
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

Replace `X.Y.Z` throughout with the actual version number.

**Pre-releases:** Do NOT use this procedure for pre-release tags. Use `release-tag.yml` directly (see `docs/RELEASE-CHANNELS.md §3`).

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
4. Generates a `## [X.Y.Z] - YYYY-MM-DD` CHANGELOG section via git-cliff (ADR-063 D3/D5),
   categorizing Conventional Commits into Added / Fixed / Performance / Changed / Security
   (docs, ci, test, chore, style, build, revert are skipped); PR links injected from
   `(#NNN)` in squash-merge commit subjects via `commit_preprocessors` (token-free)
5. Commits, pushes the branch, and opens a PR targeting `develop`

Monitor the run:

```bash
gh run list --repo drbothen/prism --workflow release-prep.yml --limit 3
gh run watch --repo drbothen/prism
```

### Step 2 — Review and merge the release-prep PR

The PR body contains a checklist. Before merging:

1. Review the git-cliff generated CHANGELOG entry (Layer 2): entries are auto-categorized
   under Added / Fixed / Performance / Changed / Security by `cliff.toml`. PR links appear
   inline from `(#NNN)` in commit subjects. No manual categorization needed.
2. Dispatch `vsdd-factory:technical-writer` to draft the Layer-1 top-block
   (`### Highlights` + `### Breaking Changes (narrative)` / `### Upgrade Notes` if
   applicable) inside the `## [VERSION]` block — this block is NOT present at PR open
   time and must be created before merge (ADR-063 D4). See §5 for the Layer-1 contract.
3. Review and curate the drafted Layer-1 block: current behavior only; remove
   aspirational bullets; omit `### Breaking Changes (narrative)` if there are no breaking changes.
4. Confirm `prism-bin` version in `Cargo.toml` matches the intended tag (`vX.Y.Z`).
5. Confirm the README reflects this release:
   - The `[![Latest Release](...)]` badge in the README header is a dynamic shields.io
     badge driven by the GitHub Releases API — it updates automatically when the release
     is published. No manual badge URL edit is required.
   - The `## Install` section uses `<version>` placeholders and references the GitHub
     Releases page; no URL updates are needed for the install instructions.
   - Review the `## Status` section text for any new or removed sensor support that
     should be noted in this release.
   Push any README changes to the release branch before merging.
6. Wait for all 24 required CI checks to pass.
7. Merge the PR to `develop` (squash or merge commit per project convention).

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

`release.yml` runs the 4-platform build matrix in parallel:

| Target | Runner | Archive |
|--------|--------|---------|
| `aarch64-apple-darwin` | `macos-latest` | `.tar.gz` |
| `x86_64-unknown-linux-gnu` | `ubuntu-latest` | `.tar.gz` |
| `x86_64-unknown-linux-musl` | `ubuntu-latest` | `.tar.gz` |
| `x86_64-pc-windows-msvc` | `windows-latest` | `.zip` |

The musl leg uses `cargo-zigbuild` to avoid glibc symbol contamination
(DEFECT-REL001-MUSL-LIBSTDCXX-001). All legs build both `prism-bin` and
`prism-dtu-demo-server`.

Expected total wall-clock time: approximately 30–45 minutes (60-minute per-job
timeout). The `publish-release` job runs after all 4 build legs succeed.

### Step 6 — Verify the GitHub Release

```bash
gh release view vX.Y.Z --repo drbothen/prism
```

Verify all of the following before declaring the release complete:

1. **4 platform archives** are attached (`prism-vX.Y.Z-<target>.tar.gz` x3 +
   `prism-vX.Y.Z-x86_64-pc-windows-msvc.zip`).
2. **`checksums.txt`** is attached (merged SHA-256 checksums from all 4 legs, plus an
   appended line for `prism-specs-vX.Y.Z.tar.gz`).
3. **`install.sh` and `install.ps1`** are attached (install scripts, uploaded per ADJ-002 / S-REL-003).
4. **`prism-specs-vX.Y.Z.tar.gz`** is attached (sensor spec tarball, uploaded per S-REL-SPECS-TARBALL-001).
5. **Build-provenance attestations** are present for each binary archive (created by
   `actions/attest-build-provenance` during the build step, visible in the workflow run's
   artifact attestations, verifiable via `gh attestation verify`). The specs tarball
   (`prism-specs-vX.Y.Z.tar.gz`) and install scripts do not carry SLSA attestations —
   their integrity is covered by `checksums.txt` SHA-256 verification only.
6. Release is marked **Latest**, not Pre-release (for stable tags without a hyphen).
7. **Release body** is present and contains the curated `## [VERSION]` CHANGELOG content
   (Layer-1 Highlights + Layer-2 git-cliff entries) extracted via `--notes-file` by
   the `publish-release` job. (`--generate-notes` is intentionally NOT used — the curated
   CHANGELOG section is the authoritative and complete release body.)
8. **Bundled specs** are present in each archive. Spot-check a tar.gz:
   ```bash
   curl -sL https://github.com/drbothen/prism/releases/download/vX.Y.Z/prism-vX.Y.Z-x86_64-unknown-linux-gnu.tar.gz \
     | tar tzf - | grep -E 'specs/|prism\.toml\.example'
   ```
   Expected: `specs/claroty.sensor.toml` and `prism.toml.example` — both at archive
   root level.

### Step 7 — Verify the GitHub Release body

The `publish-release` job sets the release body from the curated `## [VERSION]`
CHANGELOG section (Layer-1 Highlights + Layer-2 git-cliff entries) via `--notes-file`.
No routine body editing is required. If install or upgrade narrative that belongs in
the release body was not captured inside the `## [VERSION]` CHANGELOG block, add it
to the release body via the GitHub UI (Edit on the Releases page). Refer to §5 for
the canonical install and verification text.

---

## 5. Release Notes Convention

### Two-Layer CHANGELOG Model (ADR-063 D4)

Every release uses a two-layer CHANGELOG model. Layer 2 (git-cliff categorized body)
is auto-generated during the `release-prep.yml` workflow run. Layer 1 (Highlights /
Breaking Changes narrative / Upgrade Notes top-block) is MANUALLY drafted by
dispatching `vsdd-factory:technical-writer` and curated on the release-prep branch
BEFORE merge.

#### Layer 2 — git-cliff categorized body (auto-generated)

`release-prep.yml` Step 7 runs three sub-steps:

1. **Pre-strip:** a Python inline script removes the `# Changelog` masthead and any
   prior `## [Unreleased]` section from `CHANGELOG.md`, leaving the file starting at
   the first versioned `## [X.Y.Z]` section. This prevents masthead duplication on
   repeated release cycles (the masthead is re-emitted from the `cliff.toml` header
   on every run — see below).

   **First-release note (OBS-3):** The pre-strip removes the existing `# Changelog`
   masthead and any `## [Unreleased]` section before `--prepend`; git-cliff's
   `[changelog] header` re-emits them. The net result is idempotent across release
   cycles — not a no-op. The pre-strip mechanism is correct and expected on every
   release cycle.

   **Maintainer note (OBS-2):** `## [Unreleased]` is a git-cliff-managed placeholder.
   Do NOT accumulate hand-authored entries under it — the pre-strip step discards all
   content under `## [Unreleased]` on every release cycle. Pending release notes belong
   in the Layer-1 curation top-block inside the released `## [VERSION]` section, added
   after git-cliff runs in this step.

2. **git-cliff invocation:**

   ```bash
   git cliff --tag "v${VERSION}" --unreleased --prepend CHANGELOG.md
   ```

   `cliff.toml`'s `[changelog] header` carries the `# Changelog` masthead and the
   empty `## [Unreleased]` placeholder. `--prepend` inserts `header + new-section`
   before the existing file content, producing:
   `# Changelog ... ## [Unreleased] ## [X.Y.Z] ... existing sections`.

   Entries are categorized into Added / Fixed / Performance / Changed / Security.
   Skip types (docs, ci, test, chore, style, build, revert) are excluded.
   PR links are injected from commit message `(#NNN)` references (token-free).
   Do NOT add `--output` alongside `--prepend` (causes duplicate sections per
   ADR-063 §D5 v1.1 fix).

3. **Link-ref update:** a Python inline script updates the reference-style compare
   links at the bottom of `CHANGELOG.md` — the `[Unreleased]:` ref is pointed at
   `v{VERSION}...HEAD`, and a new `[{VERSION}]: .../compare/v{PREV}...v{VERSION}`
   entry is inserted (token-free, using the VERSION env var and the existing
   `[Unreleased]:` link to derive the previous tag).

Authority: `cliff.toml` at repo root (ADR-063 §D3); flag set `--unreleased --tag
... --prepend` (ADR-063 §D5). Masthead-in-header + pre-strip + link-ref-update
mechanism is canonical as of ADR-063 §D3 (empirically validated against
git-cliff 2.14.1 behavior).

#### Layer 1 — Curated top-block (human-authored, agent-assisted)

After git-cliff prepends the `## [VERSION]` block, the `vsdd-factory:technical-writer`
agent drafts a human-readable top-block consisting of:

- `### Highlights` — 5–8 bullets summarizing the platform-level value of the release
- `### Breaking Changes (narrative)` — prose description of any breaking changes;
  **omitted** if the release has no breaking changes
- `### Upgrade Notes` — migration steps for operators; omitted if not needed

**Placement invariant:** Layer-1 sections are placed INSIDE the `## [VERSION]` block,
AFTER git-cliff prepends the block, and BEFORE the first git-cliff commit-derived
`###` section. This placement ensures that `release.yml`'s awk extraction (from
`## [VERSION]` until the next `## [`) captures both layers without any modification
to `release.yml`.

**Authorship:** The `vsdd-factory:technical-writer` agent drafts this block from the
tag range, documenting **current behavior only** — not aspirational features. The
draft is a PR file edit in the release-prep branch.

**Human curation gate:** The draft is reviewed and curated in the release-prep PR
before merge. Empty sections (e.g., `### Breaking Changes (narrative)` when there are no breaking
changes) are removed. Aspirational bullets are removed. The human-curated text is
the final content; the technical-writer draft is an aid, not the final.

#### CHANGELOG.md structure (normative)

```
# Changelog

## [1.0.0-beta.1] - 2026-09-XX

### Highlights
[technical-writer draft, human-curated — inserted AFTER git-cliff prepends the block]

### Breaking Changes (narrative)
[technical-writer draft, human-curated — omitted if no breaking changes]

### Upgrade Notes
[technical-writer draft, human-curated — omitted if not needed]

### Breaking Changes
[git-cliff breaking commits — cliff.toml body Tera filter(attribute="breaking", value=true)]

### Added
[git-cliff feat commits — <!-- 1 --> parser]

### Fixed
[git-cliff fix commits — <!-- 2 --> parser]

...

## [previous-version] - YYYY-MM-DD
...
```

#### Step order in release-prep.yml

1. **Step 7 sub-step A** — pre-strip: Python removes masthead + prior `[Unreleased]` section
   from `CHANGELOG.md` (leaving file starting at first versioned `## [X.Y.Z]` section)
2. **Step 7 sub-step B** — `git cliff --tag "v${VERSION}" --unreleased --prepend CHANGELOG.md`:
   prepends `cliff.toml header (masthead + ## [Unreleased])` + new `## [VERSION]` block
3. **Step 7 sub-step C** — link-ref update: Python updates `[Unreleased]:` and inserts
   `[VERSION]: .../compare/v{PREV}...v{VERSION}` reference at bottom of `CHANGELOG.md`
4. **Step 7a** — Workflow emits a `::notice::` reminder; release engineer MANUALLY
   dispatches `vsdd-factory:technical-writer`, which inserts Layer-1 `###` sections
   INSIDE the `## [VERSION]` block, before git-cliff's first commit-derived `###` section
5. **Step 2 in §4** — Human reviews and curates Layer-1 sections in the release-prep PR

### GitHub Release body

The `release.yml` `--notes-file` extraction captures the full `## [VERSION]` block from
`CHANGELOG.md` (both Layer-1 and Layer-2 content) via awk and passes it to
`gh release create --notes-file`. No modification to `release.yml` is required.

### Installing from a release

Each release archive contains the `prism` binary plus bundled sensor specs needed
for a bootable installation — no source repository clone required.

**Archive contents:**
- `prism` (or `prism.exe` on Windows) — the compiled binary
- `prism.toml.example` — configuration template with inline instructions
- `specs/claroty.sensor.toml` — the bundled Claroty xDome sensor TOML spec

**Install steps (all platforms):**
1. Extract the archive (see platform commands below).
2. Copy `prism.toml.example` to `prism.toml` in your chosen config directory.
3. Copy `specs/` to the path you set as `spec_dir` in `prism.toml` (or leave it
   adjacent and set `spec_dir = "./specs"`).
4. Edit `prism.toml`: set `state_dir`, add `[[orgs]]` entries, configure credentials.
5. Run `prism start`.

**macOS (Apple Silicon):**
curl -LO https://github.com/drbothen/prism/releases/download/vX.Y.Z/prism-vX.Y.Z-aarch64-apple-darwin.tar.gz
tar xzf prism-vX.Y.Z-aarch64-apple-darwin.tar.gz
chmod +x prism
./prism --version

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

---

## 6. Recovery Procedures

### A build-release matrix leg failed

If one or more of the 4 build legs fails and the `publish-release` job never ran
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
`publish-release` job; the `prism-specs-<tag>.tar.gz` checksum line is then
appended by the specs-upload step in the same job. If `checksums.txt` is absent,
contains only some platform legs, or is missing the specs-tarball line, re-run the
workflow (see above). Do not manually construct or upload a `checksums.txt` — it
must come from the CI build.

### git-cliff generates an empty section (no unreleased commits)

`git cliff --unreleased` exits 0 even when no qualifying commits exist — it produces
a `## [VERSION]` header with an empty body. Without a guard this would commit a
content-free CHANGELOG section. To prevent this, `release-prep.yml` Step 7 includes
an empty-output guard immediately after the git-cliff invocation: it checks whether
the newly-prepended section contains at least one `- ` bullet entry. If the section is
empty, the guard exits 1 with an explicit error message and the workflow fails at Step 7
before Step 8 (commit) or Step 9 (push) can run. The ephemeral runner checkout is
discarded with the modification.

**Cause:** every commit since the last release tag matches a skip pattern (docs, ci,
test, chore, style, build, revert), OR there are genuinely no commits since the last
tag (you dispatched the same version twice).

**Response:**
1. `gh run view <run-id> --log-failed` — inspect the Step 7 error message, which
   will state: "git-cliff produced an empty CHANGELOG section for vX.Y.Z — no
   qualifying commits found".
2. If all commits since the last tag are skip types: either add at least one
   non-skip commit to `develop` before re-dispatching, or write the CHANGELOG section
   manually (see §5 Release Notes Convention) and skip the `release-prep` workflow.
3. If the version was already released: do not re-create the same tag. Increment
   the patch version and dispatch with the correct version.
4. Never run `--no-verify` or bypass the step — fix the underlying cause.

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
