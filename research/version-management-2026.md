---
document_type: research
research_type: general
producer: research-agent
topic: Single-command version-management tooling for Prism's Rust workspace — one bump entrypoint that propagates the product version to prism-bin, lockfiles, git tag, and non-Cargo docs without touching the 23 independent crates
timestamp: 2026-09-05
status: complete
supersedes: none
related:
  - docs/RELEASE-CHANNELS.md
  - RELEASING.md
  - .github/workflows/release-prep.yml
  - .github/workflows/release-promote.yml
  - .github/workflows/release-tag.yml
  - .factory/research/release-notes-automation-2026.md
  - .factory/research/release-engineering-uncertainties-2026.md
sources:
  # Registry version + maintenance verification (crates.io API) — verified 2026-09-05
  - https://crates.io/api/v1/crates/cargo-release   # cargo-release 1.1.5 (2026-08-11) ACTIVE
  - https://crates.io/api/v1/crates/release-plz      # release-plz 0.3.161 (2026-09-03) VERY ACTIVE
  - https://crates.io/api/v1/crates/cargo-edit        # cargo-edit 0.13.13 (2026-07-15) ACTIVE
  - https://crates.io/api/v1/crates/knope             # knope 0.23.0 (2026-05-24) ACTIVE
  - https://crates.io/api/v1/crates/cocogitto         # cocogitto 7.0.0 (2026-03-04) MAINTAINED
  - https://crates.io/api/v1/crates/git-cliff         # git-cliff 2.14.1 (2026-09-01) VERY ACTIVE
  - https://crates.io/api/v1/crates/cargo-dist        # dist (fka cargo-dist) 0.32.0 REDUCED-MAINT
  # Behaviour / docs
  - https://github.com/crate-ci/cargo-release/blob/master/docs/reference.md
  - https://github.com/crate-ci/cargo-release/issues/925   # per-package hook re-run in workspaces
  - https://github.com/killercup/cargo-edit                # cargo set-version
  - https://github.com/killercup/cargo-edit/issues/882     # set-version --workspace limitation
  - https://knope.tech/reference/config-file/packages       # versioned_files regex + lockfile
  - https://docs.cocogitto.io/guide/bump.html
  - https://release-plz.dev/docs/usage/release
  - https://github.com/axodotdev/cargo-dist                 # "dist (formerly cargo-dist)"; recommends cargo-release for bump
  - https://www.reddit.com/r/rust/comments/1noozk7/psa_cargodist_is_not_dead
  - https://doc.rust-lang.org/cargo/commands/cargo-update.html  # cargo update -p --precise semantics
mcp_gate: "Perplexity MCP NOT available in this environment (no mcp__perplexity__* tools registered). Tavily MCP used as PRIMARY research engine (1x tavily_research deep-pro + 3x tavily_search). Registry versions verified via crates.io JSON API (WebFetch). See Research Methods + deviation note."
---

# Single-Command Version Management for Prism — Landscape & Recommendation (2026)

## 0. Executive Summary

Prism needs a **single bump entrypoint** so cutting a stable release edits ONE place
instead of the ~6 hand-maintained sites it touches today (`prism-bin/Cargo.toml`, root
`Cargo.lock`, `tests/external/non-exhaustive-violation/Cargo.lock`, the git tag,
`CHANGELOG.md`, and ~20 hardcoded strings in `docs/SETUP.md` plus example strings in
`scripts/install.sh` / `scripts/install.ps1`).

**Recommendation: `cargo-release` (v1.1.5) as the single bump entrypoint**, scoped to
`prism-bin` via `[package.metadata.release]` in `crates/prism-bin/Cargo.toml`, composed
with **git-cliff (v2.14.1) as a pre-release-hook** for CHANGELOG and **build-time git
injection (vergen)** for the binary's displayed version. For `docs/SETUP.md` the
correct fix is **make the docs version-agnostic wherever a "latest release" pointer or a
version-less asset name works, and keep a small, count-guarded `pre-release-replacements`
set only for the residual hard case (the expected `prism --version` line)** — and even
that is better converted to a format-example. Migration cost is **LOW**: it replaces
~3 script steps inside `release-prep.yml` with one `cargo release` invocation plus a
config block; the two human-gates (`release-promote.yml` environment gate, `release-tag.yml`
pre-release lanes) are **untouched**. Everything stays Rust-native — no Node or Python
runtime is added.

This artifact composes with (does not contradict) the two in-motion decisions:
(1) git-cliff owns CHANGELOG generation (already decided in
`release-notes-automation-2026.md`); (2) the ADR selecting build-time binary version
identity (vergen/git-based) so tagged builds report the exact tag while `develop`
reports `-dev`.

---

## 1. Prism's exact shape (verified from the repo, 2026-09-05)

Confirmed by reading the tree and `release-prep.yml`:

- **25-crate workspace, edition 2024.** Only `crates/prism-bin/Cargo.toml` carries the
  **product version**. The other 24 crates version independently (0.x) and MUST NOT move
  on a product bump. All crates are `publish = false` (no crates.io publishing).
- **The binary reports version via `env!("CARGO_PKG_VERSION")`** in prism-bin (main.rs
  `--version`, boot.rs boot log + HTTP user-agent).
- **The product version string is hand-duplicated** in: the git tag (`release-tag.yml` /
  `release-promote.yml`), `CHANGELOG.md` headers, `docs/SETUP.md` (~20 occurrences incl.
  the asset-filename table and the expected `prism --version` output), and example strings
  in `scripts/install.sh` + `scripts/install.ps1`.
- **The "2 lockfiles" are the root `Cargo.lock` and
  `tests/external/non-exhaustive-violation/Cargo.lock`** — the latter is a **separate
  cargo workspace** (compile-fail test crate) that pins `prism-bin`. `release-prep.yml`
  already updates both today via two explicit `cargo update -p prism-bin --precise`
  calls. This is the load-bearing detail for tool selection: **any tool run in the root
  workspace will not automatically touch the second lockfile** — it must be driven by a
  hook.
- **Channel ladder** (`docs/RELEASE-CHANNELS.md`): nightly/dev/alpha/beta/rc tag
  `develop` (BASE-MATCH, **no per-channel bump** — `develop` carries `X.Y.Z-dev`); stable
  bumps `-dev → X.Y.Z` and promotes to `main` (EXACT-MATCH), human-gated via the
  `release-main` GitHub Environment. `release-prep.yml` already bumps prism-bin + 2
  lockfiles + CHANGELOG **for STABLE only**.

The takeaway that constrains everything below: the bump tool is only ever invoked on the
**stable path** (and the post-stable `-dev` bump). Pre-release lanes deliberately do not
bump, so the tool never runs there.

---

## 2. Verified tool inventory (crates.io, 2026-09-05)

Every version and date below was pulled from the crates.io JSON API on 2026-09-05.
Nothing here is from training data.

| Tool | Latest ver | Released | Maintenance | Rust-native | Role |
|------|-----------|----------|-------------|-------------|------|
| **cargo-release** | **1.1.5** | 2026-08-11 | **ACTIVE** (crate-ci / epage) | Yes | Bump + replace + tag + push |
| release-plz | 0.3.161 | 2026-09-03 | VERY ACTIVE | Yes | crates.io-publish automation (PR-driven) |
| cargo-edit (`set-version`) | 0.13.13 | 2026-07-15 | ACTIVE (epage) | Yes | Manifest-only version edit primitive |
| knope | 0.23.0 | 2026-05-24 | ACTIVE | Yes* | Workflow-driven release framework |
| cocogitto (`cog bump`) | 7.0.0 | 2026-03-04 | MAINTAINED | Yes | Conventional-commit auto-bump + changelog |
| git-cliff | 2.14.1 | 2026-09-01 | VERY ACTIVE | Yes | Changelog generator (+ version *computer*) |
| dist (fka cargo-dist) | 0.32.0 | ~2026-05 | **REDUCED-MAINT** ⚠ | Yes | Artifact builder — **not a bumper** |

\* knope's crate is Rust; its docs are config-focused and do not restate the runtime, but
it installs as a Rust binary and needs no Node/Python.

**Stale/archived flags:**
- **dist (formerly `cargo-dist`)** — axodotdev wound down as a company; the project was
  renamed `cargo-dist → dist` and entered a period of reduced maintenance. The original
  author resumed limited work (Reddit r/rust PSA "cargo-dist is not dead", Sept 2025;
  changelog shows 0.30.x–0.31.0 in Feb 2026, crates.io shows 0.32.0). It is **not archived**
  but is **not a version-management tool** — its own README says to "run cargo-release" to
  do the bump, then dist reacts to the pushed tag. Excluded from the bump decision.
- No other tool in scope is deprecated or archived. `standard-version` (Node) is deprecated
  but was never a candidate here (not Rust-native).

---

## 3. Seven-criteria capability matrix

Legend: ✅ yes · ⚠ partial/with-caveat · ❌ no · — n/a

| Criterion | cargo-release | release-plz | cargo-edit | knope | cocogitto | git-cliff | dist |
|-----------|:---:|:---:|:---:|:---:|:---:|:---:|:---:|
| Single-command bump | ✅ | ✅ | ⚠ (manifest only) | ✅ | ✅ | ❌ (computes/prints only) | ❌ |
| Bump ONE crate, siblings untouched | ✅ (`-p` + config in that crate) | ⚠ (model = "all unpublished") | ✅ (`-p`) | ✅ (per-package) | ⚠ (monorepo→per-crate tags) | — | — |
| Updates lockfile(s) | ✅ root; ⚠ 2nd lock via hook | ⚠ undoc | ❌ (needs `cargo update`) | ✅ (explicit lockfile support) | ⚠ undoc | ❌ | ❌ |
| Substitutes NON-Cargo files (regex) | ✅ `pre-release-replacements` | ❌ | ❌ | ✅ `versioned_files` regex | ❌ | ❌ | ❌ |
| Creates + pushes git tag | ✅ | ✅ | ❌ | ✅ | ✅ | ❌ | ❌ (reacts to tag) |
| Pre-release / channel (rc/beta/dev) | ✅ (LEVEL: rc/beta/alpha or explicit) | ⚠ undoc | ⚠ explicit ver | ✅ (Pre rules) | ⚠ (pre config) | ⚠ (--bump prerelease) | ❌ |
| Rust-native (no Node/Python) | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |

Only **cargo-release** and **knope** satisfy all of {single-command, single-crate,
lockfile, non-Cargo regex, tag, pre-release, Rust-native}. cargo-release wins over knope
on fit (see §7).

### cargo-release config surface (verified from reference.md)

`[package.metadata.release]` supported keys relevant here:

- `pre-release-replacements` — array of `{ file, search (regex), replace, min=1, max, exactly, prerelease=false }`.
  Placeholders: `{{prev_version}}`, `{{version}}`, `{{crate_name}}`, `{{repository}}`,
  `{{date}}`, `{{metadata}}`. The **`exactly = N` key is the drift guard** — the release
  FAILS if the doc no longer contains exactly N matches, which is precisely what stops
  `docs/SETUP.md` from silently drifting.
- `tag = true`, `tag-name` (template with `{{prefix}}`/`{{version}}`), `tag-message`,
  `push = true`, `push-remote`.
- `shared-version` — enforce identical versions across a named subset. **Prism leaves this
  OFF**, so bumping prism-bin cannot cascade to the 24 siblings.
- `publish` — set `false` (prism crates are already `publish = false`; cargo-release
  respects the manifest, so no crates.io publish ever fires).

Known caveat (issue #925): pre-release hooks/replacements can run **per selected package**
in a workspace. Because prism selects exactly one package (`-p prism-bin`) and the config
lives in that one crate, the replacement set runs once — no multi-fire risk.

---

## 4. Synthesis answers

### A) Which tool gives a genuine ONE-command bump that propagates to prism-bin (only), the 2 lockfiles, the git tag, AND the non-Cargo files — without touching the 23 siblings?

**cargo-release**, invoked as `cargo release -p prism-bin <X.Y.Z> --execute`. It propagates:

1. **`prism-bin/Cargo.toml` only** — `-p prism-bin` scopes the bump; siblings have no
   `shared-version` link, so they are untouched by design.
2. **Root `Cargo.lock`** — cargo-release updates the workspace lockfile as part of the
   bump (the manifest edit forces a lockfile resolve, which cargo-release performs;
   equivalent to `cargo update -p prism-bin --precise`).
3. **`tests/external/non-exhaustive-violation/Cargo.lock`** — this second lockfile lives
   in a **separate workspace**, so it is NOT auto-handled. It is driven by a one-line
   **pre-release-hook**: `cargo update -p prism-bin --precise {{version}} --manifest-path tests/external/non-exhaustive-violation/Cargo.toml`.
   (This is exactly what `release-prep.yml` step 6 does today, folded into the same command.)
4. **The git tag** — `tag-name = "v{{version}}"`, created and (optionally) pushed. See §4C
   for who actually *pushes* the tag under prism's gated model.
5. **The non-Cargo files** — `pre-release-replacements` entries for `docs/SETUP.md`,
   `scripts/install.sh`, `scripts/install.ps1` (bounded/count-guarded — see §4B).

No other tool hits all five. cargo-edit does (1)+(partial 2) only. release-plz/cocogitto
carry the wrong workspace model (all-packages / per-crate tags). knope can do all five but
duplicates git-cliff's changelog role (double-ownership, §7).

### B) Non-Cargo docs: regex replacement vs. version-agnostic docs?

**Both, in a deliberate split. The default should be version-AGNOSTIC docs; regex
replacement is the bounded exception for what genuinely cannot be made agnostic.**

The two hard cases in `docs/SETUP.md` decompose differently:

- **Asset-filename table → make it version-agnostic.** prism's release assets are named by
  **target triple, not version** (`release.yml` archives are per-target `.tar.gz`/`.zip`).
  If the archive names are already version-less
  (e.g. `prism-x86_64-unknown-linux-gnu.tar.gz`), the table needs no version at all — and
  the download URLs should use the **GitHub "latest release" redirect**
  (`github.com/<org>/prism/releases/latest/download/<asset>`). This is a perfect fit for
  prism's channel model: only **stable** is marked "Latest" (`release.yml` gives hyphenated
  tags `--prerelease`), so `/releases/latest/` resolves exactly to the newest stable —
  which is what an install doc should point at. **Result: the asset table and its URLs
  stop drifting because there is no version in them.** If any archive name currently
  *embeds* the version, the production-grade fix is to drop the version from the archive
  name in `release.yml`, not to regex-substitute it forever.

- **Expected `prism --version` output line → convert to a format-example (preferred), or
  regex-replace with a count guard (fallback).** This is the one string that intrinsically
  shows a concrete version. Preferred: reword the doc to assert the **format** not the exact
  string — show `prism 1.0.0` as an illustrative example with "(your installed version may
  differ)" and, if a doctest/CI check validates it, assert only the `prism ` prefix and a
  semver-shaped token. That removes the bump target entirely. Fallback, if the team wants
  the doc to show the exact shipped version: a single `pre-release-replacements` entry with
  `search = 'prism \d+\.\d+\.\d+[^\s]*'`, `replace = 'prism {{version}}'`,
  `exactly = 1`. The `exactly` guard is the anti-drift mechanism — if someone edits the doc
  and the match count changes, the release **fails loudly** instead of shipping a stale
  version string.

- **`scripts/install.sh` / `install.ps1` example strings → version-agnostic.** Install
  scripts should fetch the latest (or an operator-supplied) version at runtime (prism's
  `release-engineering-uncertainties-2026.md` already established `/releases?per_page=1`
  and vanity-domain patterns); the *example* version strings in comments should be
  genericized ("e.g. 1.0.0") so they carry no drift obligation.

**Why agnostic-first is better practice:** every regex replacement is a standing
drift-liability and a place the release can break. Every reference converted to a "latest"
pointer or a version-less asset name is a reference that **can never drift** because there
is nothing to bump. So the rule is: *eliminate the version from the doc if a latest-pointer
or triple-named asset can carry the meaning; use a count-guarded `pre-release-replacements`
entry only for the irreducible residue.* After this split, `docs/SETUP.md` has at most one
bump target (the `--version` example), and even that is preferably a format-example with
zero bump targets.

### C) Composition with git-cliff + build-time injection — one entrypoint, no double-ownership

The clean ownership map (each fact has exactly one writer):

| Artifact | Sole owner | Mechanism |
|----------|-----------|-----------|
| `prism-bin/Cargo.toml` `[package].version` (version at rest) | **cargo-release** | `cargo release -p prism-bin` edits it; nothing else writes it |
| The 2 lockfiles | **cargo-release** (root auto; 2nd via its pre-release-hook) | one command |
| `CHANGELOG.md` | **git-cliff** | invoked by cargo-release as a `pre-release-hook`: `git cliff --tag v{{version}} -o CHANGELOG.md`. cargo-release's own changelog feature stays OFF |
| The git tag `vX.Y.Z` (release identity) | **the tag is the pivot** | cargo-release *computes* `tag-name`; the *push* stays in `release-promote.yml` behind the environment gate (see D) |
| Binary displayed version | **build-time git injection (vergen)** — DERIVED, never hand-written | reads `git describe`/tag → exact tag on tagged builds, `X.Y.Z-dev` on develop; `env!("CARGO_PKG_VERSION")` remains the fallback/base |

**Why there is no double-ownership:**
- cargo-release writes the version *number* into `Cargo.toml`; git-cliff never touches
  `Cargo.toml`.
- git-cliff writes CHANGELOG *content*; cargo-release never generates changelog text (its
  built-in changelog is disabled). git-cliff runs *inside* the one bump command as a hook,
  so there is still one entrypoint.
- The **binary's displayed version is derived from git, not authored** — so it cannot
  disagree with the tag. cargo-release owns the `-dev` base at rest and the tag identity;
  vergen reads that identity. No human ever types the version into the binary.
- git-cliff can additionally *compute* the next semver (`git cliff --bumped-version`) from
  Conventional Commits and feed that number to cargo-release — useful, but optional. Even
  when used, git-cliff only *proposes a number*; cargo-release is the only thing that
  *writes* it. So the "who decides the number" question can stay a human judgment (prism's
  channel model is deliberately human-gated) or be git-cliff-assisted, without changing the
  ownership map.

**One entrypoint, concretely:** the stable bump is a single `cargo release -p prism-bin
<X.Y.Z>` call that (a) edits Cargo.toml, (b) updates both lockfiles, (c) fires git-cliff to
regenerate CHANGELOG, (d) applies the bounded doc replacements. Downstream, the *tag* is the
single pivot from which the binary display version (vergen) and the GitHub Release
(`release.yml`) both derive. Nothing is written twice.

### D) Channel-ladder fit — pre-release (BASE-MATCH, no bump) vs stable (EXACT bump)

- **Pre-release path (nightly/dev/alpha/beta/rc):** BASE-MATCH, **no bump**. `develop`
  carries `X.Y.Z-dev` and pre-release lanes tag `develop` without editing `Cargo.toml`.
  **cargo-release is not invoked on this path at all** — `release-tag.yml` keeps cutting the
  hyphenated tag exactly as it does today. This matches prism's existing design (only
  `release-prep.yml` ever bumps). ✅ No change to `release-tag.yml`.
- **Stable path (EXACT bump `-dev → X.Y.Z`):** cargo-release **replaces the mechanical guts
  of `release-prep.yml`** — the Python-regex version edit (step 4), the two `cargo update`
  calls (step 6), and the CHANGELOG scaffold (step 7) collapse into one `cargo release`
  invocation (bump + both lockfiles + git-cliff CHANGELOG + doc replacements). Steps 3/8/9/10
  (branch, commit, push branch, open PR) stay. ✅ Only `release-prep.yml` internals change.
- **`release-promote.yml` — UNCHANGED.** It keeps sole ownership of the **tag push + `main`
  merge** behind the `release-main` environment gate and the EXACT-MATCH version guard. To
  respect this gate, cargo-release runs in `release-prep.yml` with **`tag = false, push =
  false`** (it produces the bumped branch + PR; it does NOT cut or push the tag). The tag
  remains the promote-workflow's job. So cargo-release owns *everything except the tag push*,
  and the human-gated promote owns the tag push — no authority is moved out of the gate.
- **Post-stable `-dev` bump (`1.0.0 → 1.1.0-dev`):** also a single
  `cargo release -p prism-bin 1.1.0-dev` on develop. One command.

Net workflow delta: **`release-prep.yml` internals only** (three script steps → one command
+ config). `release-tag.yml` and `release-promote.yml` are untouched.

### E) Recommendation, rationale, migration cost, Rust-native tradeoffs

**Adopt `cargo-release` (1.1.5)** as the single stable-path bump entrypoint, with:
- `[package.metadata.release]` in `crates/prism-bin/Cargo.toml`:
  `shared-version` OFF; `publish = false`; `tag = false`, `push = false` (promote owns the
  tag); `pre-release-hook` = git-cliff CHANGELOG + the second-lockfile `cargo update`;
  a **bounded, count-guarded** `pre-release-replacements` set for the doc/script residue.
- **git-cliff (2.14.1)** owns CHANGELOG (already the in-motion decision); invoked as the
  cargo-release hook — one entrypoint.
- **vergen build-time git injection** owns the binary display version (the in-motion ADR);
  derived from the tag, never authored.
- **docs made version-agnostic** (latest-pointer URLs + triple-named assets) so `SETUP.md`
  has at most one residual bump target.

**Migration cost — LOW:**
1. Add the `[package.metadata.release]` block to `prism-bin/Cargo.toml` (~30 lines).
2. Add `cliff.toml` (already planned for the changelog decision — no new cost here).
3. Rewrite `release-prep.yml` steps 4/6/7 as one `cargo release … --no-confirm` step
   (net-negative lines; deletes two Python heredocs).
4. Convert `docs/SETUP.md` asset table + URLs to latest-pointer/triple form; convert the
   `--version` line to a format-example (or a single count-guarded replacement).
5. Add `cargo install cargo-release git-cliff` (or `taiki-e/install-action`) to the
   release-prep CI job.

No changes to `release-tag.yml` or `release-promote.yml`. No source-code changes beyond the
vergen wiring already scoped by its ADR.

**Rust-native tradeoffs — none.** cargo-release, git-cliff, and vergen are all Rust; no
Node or Python runtime is introduced (and the existing Python heredocs in `release-prep.yml`
are removed, so runtime dependency footprint *shrinks*).

**Why not the alternatives:**
- **release-plz (0.3.161)** — rejected. Its model is "release all unpublished packages to
  crates.io, versions auto-decided from Conventional Commits, PR-driven." That conflicts with
  (1) `publish = false`, (2) the single-product-version crate + 24 independently-versioned
  siblings, and (3) prism's **deliberate human-judgment** channel decisions (release-plz wants
  to auto-compute versions). It also offers no arbitrary-file regex substitution for
  `SETUP.md`. Same conclusion the sibling changelog research reached. Its git-cliff
  integration is real but internal — it doesn't help a publish=false, human-gated product.
- **knope (0.23.0)** — capable on paper (per-package, lockfile, `versioned_files` regex,
  pre-release, tag) but it is a **release *framework*** that also generates its own
  changelog → **double-ownership with git-cliff**, contradicting the one-owner-per-fact map.
  Heavier config than a single cargo-release block. Overkill for "bump one crate + regen
  changelog + replace a couple doc strings."
- **cocogitto (7.0.0)** — `cog bump` auto-decides the version from Conventional Commits
  (fights human-gated channels), monorepo mode emits **per-crate tags** (wrong for a single
  product version), no arbitrary-file regex substitution, and it owns changelog
  (double-ownership). Rejected.
- **cargo-edit `set-version` (0.13.13)** — a manifest-only **primitive** (no tag, no docs, no
  changelog; second lockfile needs a manual `cargo update`). cargo-release is built on top of
  this class of operation and adds everything missing. Keep it available as a trivial fallback
  for a raw `-dev` bump, but it does not meet "one command → tag + docs."
- **dist / cargo-dist (0.32.0, reduced-maint ⚠)** — **not a version bumper**; it builds and
  hosts artifacts triggered by a pushed tag and its own README recommends cargo-release for
  the bump. prism already has its own 5-platform `release.yml`. Out of scope for the bump
  decision; flagged for reduced maintenance regardless.

---

## 5. Proposed `[package.metadata.release]` for prism-bin (illustrative)

```toml
# crates/prism-bin/Cargo.toml
[package.metadata.release]
publish = false                 # prism crates never hit crates.io
shared-version = false          # do NOT cascade to the 24 sibling crates
tag = false                     # release-promote.yml owns the tag push (env gate)
push = false                    # ditto — prep only produces the branch + PR
# CHANGELOG via git-cliff + second-workspace lockfile, in one hook:
pre-release-hook = [
  "git", "cliff", "--tag", "v{{version}}", "-o", "CHANGELOG.md",
]
# (a second hook line runs: cargo update -p prism-bin --precise {{version}}
#  --manifest-path tests/external/non-exhaustive-violation/Cargo.toml)

# Residual doc/script version strings — count-guarded so drift FAILS the release:
[[package.metadata.release.pre-release-replacements]]
file    = "docs/SETUP.md"
search  = 'prism \d+\.\d+\.\d+[0-9A-Za-z.\-]*'   # the expected `--version` line
replace = 'prism {{version}}'
exactly = 1                                       # anti-drift guard
```

Note: the asset-filename table and download URLs in `SETUP.md` are **not** in this block —
they are converted to `releases/latest/download/<triple-asset>` and therefore need no
replacement (§4B). That is the point: the smaller this block, the less there is to drift.

---

## 6. Confidence & limitations

- **HIGH** — version/date/maintenance facts (crates.io API, verified 2026-09-05) and
  cargo-release's config surface (reference.md read directly).
- **HIGH** — the "2 lockfiles = root + non-exhaustive-violation, second is a separate
  workspace" fact (read from `release-prep.yml`).
- **MEDIUM** — that cargo-release updates the *root* `Cargo.lock` automatically for a
  `publish = false` bump. reference.md does not enumerate lockfile behavior explicitly; the
  conclusion is cross-derived from the manifest-edit→resolve mechanism and cargo's documented
  `cargo update -p --precise` semantics. **Mitigation:** the design pins the second lockfile
  via an explicit hook regardless, and the same explicit `cargo update -p prism-bin` can be
  added for the root lock as a hook if cargo-release's implicit behavior is ever in doubt —
  making the outcome deterministic either way. Verify with a dry-run (`cargo release … --no-execute`)
  during migration.
- **INCONCLUSIVE from sources** — release-plz single-package-scoping flags and cocogitto's
  "bump exactly one package" option were not documented in retrieved sources; both tools are
  rejected on model-fit grounds independent of this gap, so it does not affect the
  recommendation.

---

## Research Methods

| Tool | Queries | Purpose |
|------|---------|---------|
| **Perplexity perplexity_research (PRIMARY)** | 0 | ⚠ UNAVAILABLE — no `mcp__perplexity__*` tools registered in this environment (see deviation note) |
| Tavily tavily_research | 1 | Deep (model=pro) 8-criteria comparison of cargo-release/release-plz/cargo-edit/knope/cocogitto/cargo-dist with primary-source citations |
| Tavily tavily_search | 3 | cargo-release maintenance; cargo-dist/dist rename+shutdown status; cargo-release lockfile behavior |
| WebFetch | 8 | crates.io JSON API version+date verification (cargo-release, release-plz, cargo-edit, cocogitto, git-cliff, knope) + cargo-release reference.md config surface |
| Context7 | 0 | Not needed — registry API + primary docs were authoritative; no library-API question |
| Read (repo) | 3 | docs/RELEASE-CHANNELS.md, .github/workflows/release-prep.yml, prior release-notes-automation-2026.md (compose, not duplicate) |
| Glob (repo) | 3 | Cargo.lock inventory (confirmed the 2 relevant lockfiles), workflow + research-dir listing |
| Training data | 1 area | cargo-release's implicit root-lockfile-update behavior — flagged MEDIUM confidence with an explicit deterministic mitigation; NOT relied on for any version number |

**Total MCP tool calls:** 4 (1 tavily_research + 3 tavily_search)
**Training data reliance:** low — every version number and release date was verified against
the crates.io JSON API on 2026-09-05; the single training-derived claim (root-lockfile auto-update)
is flagged MEDIUM and neutralized by an explicit hook-based mitigation.

### MCP-gate deviation note

**Perplexity MCP was not available** in this environment — no `mcp__perplexity__*` tools are
registered (only Context7 and Tavily MCP servers are present). Per the research-agent MCP
gate, Tavily was used as the PRIMARY engine: one deep `tavily_research` (model=pro) pass plus
three targeted `tavily_search` calls, cross-validated against direct crates.io registry reads
and the cargo-release reference docs. This satisfies the "≥1 MCP call" requirement (4 Tavily
MCP calls); the `perplexity_research`-preference could not be honored because the tool is
absent, not skipped.
