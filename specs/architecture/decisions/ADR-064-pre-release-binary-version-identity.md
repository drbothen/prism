---
document_type: adr
adr_id: "ADR-064"
title: "Pre-Release Binary Version Identity — build.rs Tag Injection; Develop Carries 1.0.0-dev; Single-Command Version Bump via cargo-release"
status: ACCEPTED
date: "2026-09-05"
version: "1.3"
producer: architect
subsystems_affected: [SS-22]
supersedes: []
superseded_by: null
amends: [ADR-062]
anchor_stories:
  # anchor_stories: [] — SAC-2 VERIFIED-EMPTY. Implementation stories for this ADR have not yet
  # been authored. Stories will be anchored when product-owner authors S-REL-DEV-RESET-001 (D1),
  # S-REL-BVERSION-INJECT-001 (D2), S-REL-VBUMP-001 (D3), and S-REL-DOCS-AGNOSTIC-001 (D3 docs).
  # None exist on disk at ADR creation time.
related_adrs: [ADR-062, ADR-063]
related_bcs: []
locked_decisions: []
wiring_deferred_to: null
inputs:
  - crates/prism-bin/Cargo.toml
  - crates/prism-bin/src/main.rs
  - crates/prism-bin/src/boot.rs
  - crates/prism-bin/src/cli.rs
  - crates/prism-bin/src/spec_driven_adapter.rs
  - docs/RELEASE-CHANNELS.md
  - .github/workflows/release-tag.yml
  - .github/workflows/release.yml
  - .factory/specs/architecture/decisions/ADR-062-product-version-alignment.md
  - .factory/research/version-management-2026.md
input-hash: "f985d45"
---

# ADR-064: Pre-Release Binary Version Identity — build.rs Tag Injection; Develop Carries 1.0.0-dev; Single-Command Version Bump via cargo-release

## Status

ACCEPTED v1.3 (2026-09-05) — v1.3 fixes sole remaining stale count in D2 intro sentence ("four"
→ "six"). v1.2 corrected D2 version-report sites table (4 → 6 prism-bin sites, cli.rs clap
attribution, vergen rationale, GITHUB_REF_NAME disambiguation) and D3 pre-release-hook
(array-of-arrays, `--latest --prepend`). Amends ADR-062 D2 for the pre-release path. Informed by
`.factory/research/version-management-2026.md` (research-agent, 2026-09-05, Tavily deep-pro + 8
registry verifications). `anchor_stories` is SAC-2 VERIFIED-EMPTY; stories to be authored in
proposed epic E-REL-IDENTITY.

---

## Context

### The Bug

`crates/prism-bin/Cargo.toml` currently contains `version = "1.0.0-rc.1"`. Develop has carried
this value since S-REL-002 set it for the rc.1 release.

`docs/RELEASE-CHANNELS.md §3` specifies that develop always carries `X.Y.Z-dev` between tags.
Under BASE-MATCH (RELEASE-CHANNELS.md §3), the `release-tag.yml` version guard compares only the
`X.Y.Z` core of the tag against the `X.Y.Z` core of `prism-bin/Cargo.toml`. This means:

- A `v1.0.0-beta.1` tag would pass the BASE-MATCH guard (both cores = `1.0.0`)
- But the built binary would report `prism 1.0.0-rc.1` from `env!("CARGO_PKG_VERSION")` baked at
  compile time

The result is that `prism --version`, the boot log, and HTTP user-agent headers all report the
wrong channel identifier for any pre-release build that isn't exactly `v1.0.0-rc.1`.

### The Six Version-Report Sites in prism-bin

Six distinct version-report sites in `prism-bin` use `env!("CARGO_PKG_VERSION")` (either
explicitly or via clap derivation):

| File | Site (function / macro) | Description |
|------|------------------------|-------------|
| `crates/prism-bin/src/main.rs` | `println!` in `Commands::Version` arm (2 call sites) | `prism version` subcommand output |
| `crates/prism-bin/src/cli.rs` | `#[command(version)]` on `CliArgs` (clap derivation) | `prism --version` output; fix = `#[command(version = env!("PRISM_VERSION"))]` |
| `crates/prism-bin/src/boot.rs` | `tracing::info!("Prism v{}", ...)` in boot log sequence | First boot log line |
| `crates/prism-bin/src/boot.rs` | `let version = env!(...)` feeding `BootAuditEmitter` | Boot-time audit record per BC-2.05.012 — would log `1.0.0-dev` on tagged CI builds without D2 |
| `crates/prism-bin/src/boot.rs` | `.user_agent(concat!("prism/", ...))` in `build_http_client_with_timeout` (2 call sites) | HTTP client user-agent |
| `crates/prism-bin/src/spec_driven_adapter.rs` | `.user_agent(concat!("prism/", ...))` in spec-driven fetch builder | Spec-driven adapter fetch path user-agent |

**Out-of-scope site (prism-spec-engine):** `crates/prism-spec-engine/src/pipeline.rs` also
contains `.user_agent(concat!("prism/", env!("CARGO_PKG_VERSION")))` in its pipeline HTTP client
builder. This site is NOT in `prism-bin`; a `build.rs` in `prism-bin` can only set `PRISM_VERSION`
for that crate — it cannot affect `prism-spec-engine`'s build. Consequence: the spec-engine
adapter path emits its own crate version (from `prism-spec-engine/Cargo.toml`), not the product
version. This is acceptable because (a) the primary user-agent for boot-initiated sensor calls is
the `prism-bin/src/boot.rs` site, which IS updated; (b) aligning the spec-engine user-agent with
the product version would require coupling `prism-spec-engine` to `prism-bin`'s version identity.
A follow-up story outside E-REL-IDENTITY may address this if user-agent consistency becomes a
requirement.

All six prism-bin sites must be consistent. Changing only `--version` while any other site
remains on `CARGO_PKG_VERSION` would produce inconsistent version strings across the product
surface.

### Why ADR-062 Option B Was Not a Precedent Against This Approach

ADR-062 rejected "Option B: Add a `BUILD_VERSION` env var override" in the context of *stable
releases*, where the correct action is to bump `prism-bin/Cargo.toml` before tagging (D2). That
rejection is sound for stable. For pre-release channels where BASE-MATCH allows tagging without
bumping `Cargo.toml`, a build-time injection mechanism is the only way to get accurate version
strings without requiring a `Cargo.toml` bump (and thus a Cargo.lock commit) for every pre-release.
This ADR does not reopen ADR-062 D2 for stable; it extends D2 with a pre-release-specific path.

### The Scattered Version String Problem (D3 scope)

`prism-bin/Cargo.toml` `version` field is the binary source of truth, but the product version
string also appears in:

- `Cargo.lock` (two entries for `prism-bin` and a path dep)
- `CHANGELOG.md` headers
- `docs/SETUP.md` (~21 occurrences including asset filenames, expected `--version` output, and curl
  download commands)
- `scripts/install.sh` and `scripts/install.ps1` (USAGE comment examples — script logic is
  version-agnostic via GitHub API)

Each release requires updating all these locations consistently. Today this is done manually.
`version-management-2026.md` (research-agent, 2026-09-05) evaluated single-command tooling and
recommended cargo-release. D3 (below) specifies the finalized decision.

---

## Decision

### D1 — Develop Carries `1.0.0-dev` (Immediate Interim)

`crates/prism-bin/Cargo.toml` `version` is reset from `1.0.0-rc.1` to **`1.0.0-dev`**
immediately. This brings develop into compliance with `docs/RELEASE-CHANNELS.md §3`
("develop always carries X.Y.Z-dev") and prevents any future pre-release tag from baking a stale
channel suffix into the binary.

**Immediate implementation obligation:** S-REL-DEV-RESET-001 (story to be authored by
product-owner) executes this one-line change:

```toml
# crates/prism-bin/Cargo.toml — before
version = "1.0.0-rc.1"

# crates/prism-bin/Cargo.toml — after
version = "1.0.0-dev"
```

The story also updates both `Cargo.lock` entries (root + path dep), and bumps `docs/SETUP.md`
occurrences of `v1.0.0-rc.1` to `v1.0.0-dev` as a stop-gap (D3 will later replace the manual
SETUP.md update with a single-command bump). Story must land on `develop` before the `v1.0.0-beta.1`
tag is cut.

**BASE-MATCH compatibility:** `1.0.0-dev` core is `1.0.0`; `v1.0.0-beta.1` core is `1.0.0`.
BASE-MATCH guard in `release-tag.yml` passes.

### D2 — Build-Time Version Injection via `build.rs` (Full Fix — Pre-Req for beta.1)

A new file `crates/prism-bin/build.rs` is introduced. It reads `GITHUB_REF_NAME` (set by GitHub
Actions on every tag-triggered run to the tag name, e.g., `v1.0.0-beta.1`) and emits a
`PRISM_VERSION` compile-time env var that overrides `CARGO_PKG_VERSION` at all six version-report
sites in `prism-bin`.

**Fallback chain (normative):**

1. `PRISM_BUILD_VERSION` env var — explicit override for tooling/testing
2. `GITHUB_REF_NAME` env var, stripped of leading `v` — CI tag build path
3. `CARGO_PKG_VERSION` — local development fallback (resolves to `1.0.0-dev` on develop, which
   is correct for local dev)

**`build.rs` (normative contract):**

```rust
fn main() {
    println!("cargo:rerun-if-env-changed=PRISM_BUILD_VERSION");
    println!("cargo:rerun-if-env-changed=GITHUB_REF_NAME");

    let cargo_version = std::env!("CARGO_PKG_VERSION");

    let version = if let Ok(v) = std::env::var("PRISM_BUILD_VERSION") {
        v
    } else if let Ok(r) = std::env::var("GITHUB_REF_NAME") {
        r.trim_start_matches('v').to_string()
    } else {
        cargo_version.to_string()
    };

    println!("cargo:rustc-env=PRISM_VERSION={}", version);
}
```

**All six prism-bin version-report sites are updated from `env!("CARGO_PKG_VERSION")` to
`env!("PRISM_VERSION")`:**

| File | Site (function / macro) | Before | After |
|------|------------------------|--------|-------|
| `src/main.rs` | `println!` in `Commands::Version` (2 call sites) | `env!("CARGO_PKG_VERSION")` | `env!("PRISM_VERSION")` |
| `src/cli.rs` | `#[command(version)]` on `CliArgs` | implicit `CARGO_PKG_VERSION` | `#[command(version = env!("PRISM_VERSION"))]` |
| `src/boot.rs` | `tracing::info!` boot log | `env!("CARGO_PKG_VERSION")` | `env!("PRISM_VERSION")` |
| `src/boot.rs` | `let version` in `BootAuditEmitter` init (BC-2.05.012) | `env!("CARGO_PKG_VERSION")` | `env!("PRISM_VERSION")` |
| `src/boot.rs` | `.user_agent(...)` in `build_http_client_with_timeout` (2 call sites) | `env!("CARGO_PKG_VERSION")` | `env!("PRISM_VERSION")` |
| `src/spec_driven_adapter.rs` | `.user_agent(...)` in spec-driven fetch builder | `env!("CARGO_PKG_VERSION")` | `env!("PRISM_VERSION")` |

**Cross-platform compatibility:**

`build.rs` reads environment variables and emits a `cargo:rustc-env` directive. It runs on the
HOST machine during compilation, not on the cross-compilation target. `GITHUB_REF_NAME` is set on
all GitHub Actions runner types (ubuntu-latest, macos-14, windows-latest) during tag-triggered
runs. The five build targets (aarch64-apple-darwin, x86_64-apple-darwin, x86_64-unknown-linux-gnu,
x86_64-unknown-linux-musl via cargo-zigbuild, x86_64-pc-windows-msvc) are all built from their
respective runner OS, and all runners have access to `GITHUB_REF_NAME`. No target-conditional code
is needed.

**ADR-062 D2 extension (normative):**

ADR-062 D2 states "prism-bin/Cargo.toml `version` field is bumped to match the intended release
tag before the tag is created." This ADR extends D2 as follows:

- For **stable releases** (X.Y.Z without hyphen): ADR-062 D2 unchanged. prism-bin is bumped to
  the exact stable version before tagging. Local dev and CI both report the correct version via
  `CARGO_PKG_VERSION` (or `PRISM_VERSION`, which falls through to `CARGO_PKG_VERSION` locally).
- For **pre-releases** (X.Y.Z-channel.N): Develop carries `1.0.0-dev` (D1). The tag provides the
  authoritative channel suffix. `build.rs` injects `GITHUB_REF_NAME`-derived version at compile
  time during CI. Local builds report `1.0.0-dev`, which is the correct local dev identity.
  No `Cargo.toml` bump is required between pre-releases on the same X.Y.Z cycle.

**Implementation obligation:** S-REL-BVERSION-INJECT-001 (story to be authored by product-owner).
This story is a BLOCKING PREREQUISITE for the v1.0.0-beta.1 tag. It must land on develop before
`release-tag.yml` is invoked with `v1.0.0-beta.1`.

### D3 — Single-Command Version Management via cargo-release

**Tool: `cargo-release` v1.1.5** (crate-ci/epage; verified 2026-09-05, crates.io, Rust-native,
ACTIVE maintenance). Scoped to `prism-bin` only via `[package.metadata.release]` config.

This decision applies to the **stable release path only**. Pre-release channels (D1/D2) do not
invoke cargo-release; they tag develop as-is.

**Ownership map — one writer per artifact, no double-ownership:**

| Artifact | Sole writer | Mechanism |
|----------|-------------|-----------|
| `prism-bin/Cargo.toml` `version` | **cargo-release** | manifest edit at bump time |
| Root `Cargo.lock` | **cargo-release** | auto-resolved by manifest edit |
| `tests/external/non-exhaustive-violation/Cargo.lock` | **cargo-release** | via explicit `pre-release-hook` (separate workspace, must be driven by hook) |
| `CHANGELOG.md` | **git-cliff** (ADR-063) | invoked as a cargo-release `pre-release-hook` |
| The git tag `vX.Y.Z` | **`release-promote.yml`** environment gate | cargo-release runs with `tag = false, push = false`; the promote workflow retains sole ownership of the tag push |
| Binary displayed version | **build.rs injection** (D2) | derived from `GITHUB_REF_NAME` at CI compile time; never hand-written |
| `docs/SETUP.md` residual version string | **cargo-release** `pre-release-replacements` | single count-guarded entry; see Docs Anti-Drift below |

**`[package.metadata.release]` configuration sketch (normative):**

```toml
# crates/prism-bin/Cargo.toml
[package.metadata.release]
publish = false                  # prism crates never hit crates.io
shared-version = false           # do NOT cascade to the 24 sibling crates
tag = false                      # release-promote.yml owns the tag push (environment gate)
push = false                     # prep produces bumped branch + PR only; promote owns push
pre-release-hook = [
  # 1. git-cliff prepends the new version section to CHANGELOG.md (ADR-063 D5).
  #    --latest --prepend writes ONLY the latest tag range to the top of CHANGELOG.md.
  #    Do NOT use -o/--output on the whole file — that regenerates from full history
  #    and erases any human-curated Layer-1 content already in CHANGELOG.md.
  ["git", "cliff", "--tag", "v{{version}}", "--latest", "--prepend", "CHANGELOG.md"],
  # 2. Update the second workspace lockfile (non-exhaustive-violation — separate workspace,
  #    not auto-updated by the root workspace bump).
  ["cargo", "update", "-p", "prism-bin", "--precise", "{{version}}",
    "--manifest-path", "tests/external/non-exhaustive-violation/Cargo.toml"],
]

[[package.metadata.release.pre-release-replacements]]
# Count-guarded replacement for the expected `prism --version` output line in SETUP.md.
# If SETUP.md is converted to a format-example (preferred, see Docs Anti-Drift below),
# this entry is removed entirely. Kept here as the fallback.
file    = "docs/SETUP.md"
search  = 'prism \d+\.\d+\.\d+[0-9A-Za-z.\-]*'
replace = "prism {{version}}"
exactly = 1                      # anti-drift guard: FAILS if match count changes
```

**Invocation (stable path, inside `release-prep.yml`):**

```bash
cargo release -p prism-bin "${NEXT_VERSION}" --execute --no-confirm
```

This replaces three mechanical steps in `release-prep.yml` (the Python-regex version edit,
two `cargo update` calls) with one command. Steps 3/8/9/10 (branch creation, commit, push
branch, open PR) remain in the workflow — cargo-release handles the content, not the PR lifecycle.

**Docs Anti-Drift Strategy:**

The `docs/SETUP.md` ~21 occurrences of the version string decompose into three categories:

1. **Asset-filename table and download URLs** — convert to use the GitHub "Latest Release"
   redirect (`github.com/<org>/prism/releases/latest/download/<triple-asset>`). Only stable is
   marked "Latest" (pre-releases use `--prerelease` in `release.yml`), so this resolves correctly.
   After this conversion: **zero bump targets** for asset filenames.

2. **Expected `prism --version` output line** — preferred fix: reword as a format-example
   (`prism <version>` or `prism 1.0.0` annotated "(your installed version may differ)"). Zero
   bump targets. Fallback: the single count-guarded `pre-release-replacements` entry above.

3. **`scripts/install.sh` / `install.ps1` USAGE comment examples** — genericize to `e.g. 1.0.0`
   (no specific version). Script logic is already version-agnostic (uses GitHub API). Zero
   bump targets after genericization.

The production-grade target is zero bump targets in docs (version-agnostic-first). The
`pre-release-replacements` entry is a bounded fallback; if the format-example migration lands
first (S-REL-DOCS-AGNOSTIC-001), the replacements block is empty.

**Channel-ladder fit:**

- **Pre-release path** (beta/rc/alpha via `release-tag.yml`): cargo-release is NOT invoked.
  Develop carries `1.0.0-dev`; `release-tag.yml` cuts the hyphenated tag. Unchanged behavior.
- **Stable path** (via `release-promote.yml`): `release-prep.yml` invokes cargo-release for the
  `-dev → X.Y.Z` bump. The promote workflow then pushes the tag behind the environment gate.
- **Post-stable `-dev` bump** (`1.0.0 → 1.1.0-dev` on develop): also a single
  `cargo release -p prism-bin 1.1.0-dev` invocation on develop after stable ships.

**D3 is NOT a blocking prerequisite for v1.0.0-beta.1.** D1 and D2 are the blocking
prerequisites. D3 improves release-operator ergonomics on the stable path but does not affect
pre-release correctness. The SETUP.md `v1.0.0-rc.1` occurrences are updated manually in the
D1 story as a stop-gap until S-REL-VBUMP-001 ships.

---

## Rationale

### Why build.rs (not environment variable at runtime)

A runtime env var (`PRISM_VERSION=1.0.0-beta.1 prism start`) would work but requires every
invocation to set the var and every deployment script to know the version. `env!()` is baked at
compile time; the binary carries its own version identity without external configuration.

### Why GITHUB_REF_NAME (not a separate CI env var)

`GITHUB_REF_NAME` is set by GitHub Actions on tag-triggered `release.yml` runs — the workflow
that performs the 5-platform binary build matrix. On tag-push triggers (`on: push: tags: ['v*']`),
`GITHUB_REF_NAME` is the tag name (e.g., `v1.0.0-beta.1`), which is exactly what `build.rs` needs.

**NIT-2 disambiguation:** `release-tag.yml` and `release-promote.yml` run as `workflow_dispatch`
(branch context); `GITHUB_REF_NAME` on those runs resolves to the branch name (e.g., `develop`),
not a tag. However, those workflows do NOT run the 5-platform build matrix — they only create the
tag and promote the release respectively. The binary compilation happens exclusively in
`release.yml` (tag-triggered), where `GITHUB_REF_NAME` is the tag. Local builds never have
`GITHUB_REF_NAME` set by GitHub Actions, so `build.rs` falls through to `CARGO_PKG_VERSION`
(`1.0.0-dev`), which is the correct local dev identity.

Adding a separate `PRISM_BUILD_VERSION` secret or CI variable would require workflow edits and
introduces another thing to keep in sync with the tag. `GITHUB_REF_NAME` in `release.yml` is
the authoritative tag name for the build — using it directly is the minimal no-drift approach.

The `PRISM_BUILD_VERSION` escape hatch remains available for tooling/testing (e.g., building
a local release candidate without a tag) but is not the primary path.

### Why not vergen?

`vergen` was considered and rejected for D2. `vergen` reads `git describe` at build time to
produce version strings that include the nearest tag. This works correctly for local builds from
tagged commits, but `git describe` requires **full git history** — at minimum enough ancestors
to reach the nearest tag. GitHub Actions' default checkout (`actions/checkout@v4`) uses
`fetch-depth: 1` (a shallow clone with no history beyond the commit being built). On a shallow
clone, `git describe` either fails or returns a fallback like `v1.0.0-dev-0-g<sha>` instead of
the tag. The workflow would need an explicit `fetch-depth: 0` to work reliably.

`GITHUB_REF_NAME` has no such dependency — it is set by the GitHub Actions runtime from the
event payload, with no git operations required. It is available identically on all 5 runner types
regardless of checkout depth. This makes it strictly simpler and more reliable for Prism's
CI-build use case. `vergen` would be the correct choice if accurate version strings in local
builds from tagged commits were a requirement; they are not currently.

### Why 1.0.0-dev (not 1.0.0-rc.1)

`RELEASE-CHANNELS.md §3` explicitly specifies the develop default as `X.Y.Z-dev`. Carrying
`rc.1` on develop violates this convention and creates the mislabeling bug described in §Context.
`1.0.0-dev` is distinct from any shipped channel suffix, which makes it easy to identify a
locally-built binary as a development build. `rc.1` implies a release candidate, which is
misleading for a development build.

---

## Consequences

### Positive

- `prism --version`, boot log, and HTTP user-agent all report the exact tag version on every
  tagged CI build (beta, rc, stable)
- Local builds consistently report `1.0.0-dev` — accurate, distinguishable from any release
- BASE-MATCH continues to work: `1.0.0-dev` core = `1.0.0`; all future `1.0.0-X.Y` pre-release
  tags pass the guard
- No per-pre-release `Cargo.toml` bump required; develop stays at `1.0.0-dev` throughout the
  1.0.0 channel ladder
- build.rs approach is zero-runtime, zero-extra-tooling (standard `cargo build` infrastructure)
- Cross-platform: works identically on all 5 build targets

### Negative / Trade-offs

- `crates/prism-bin/build.rs` is a new build script; adds a small (sub-millisecond) overhead on
  `cargo build` for cache invalidation if `GITHUB_REF_NAME` or `PRISM_BUILD_VERSION` changes.
  In practice, local builds never change these env vars, so the build script runs once and caches.
- Local builds report `1.0.0-dev`, not a specific pre-release version. Operators testing a local
  build need to know that the version string reflects the Cargo.toml channel value, not a tag.
  This is correct behavior but differs from what CI-built binaries report.
- ADR-062 D2 alternative Option B ("add a BUILD_VERSION env var") was rejected in the stable
  context. This ADR introduces a comparable mechanism for the pre-release path. The distinction
  is: ADR-062 D2 Option B was a *parallel* stable-version mechanism that would have bypassed the
  Cargo.toml source of truth for all builds. This ADR's build.rs injection is a *layered* mechanism
  that defers to `CARGO_PKG_VERSION` locally and applies `GITHUB_REF_NAME` only in CI tag builds,
  keeping `Cargo.toml` as the human-readable source of truth for local development.

### Status as of v1.3

ACCEPTED. All three decisions are finalized:
- D1 (S-REL-DEV-RESET-001): BLOCKING before beta.1 — prism-bin reset to `1.0.0-dev`
- D2 (S-REL-BVERSION-INJECT-001): BLOCKING before beta.1 — build.rs injection of `PRISM_VERSION`
  at all 6 prism-bin version-report sites (including cli.rs `#[command(version)]`, boot.rs audit
  record, and spec_driven_adapter.rs user-agent); vergen noted as alternative and rejected
- D3 (S-REL-VBUMP-001 + S-REL-DOCS-AGNOSTIC-001): High priority before stable v1.0.0 — cargo-release
  1.1.5 single-command bump; RELEASING.md §1 pre-release exception to be documented in
  S-REL-DOCS-AGNOSTIC-001

---

## Alternatives Considered

- **Option A: Bump prism-bin to the exact pre-release version on develop (e.g., 1.0.0-beta.1).**
  Requires a Cargo.toml commit + Cargo.lock commit before every pre-release tag. With a channel
  ladder that may have multiple pre-releases per X.Y.Z cycle (beta.1, beta.2, rc.1, rc.2),
  this creates version-bump commit noise. Also violates RELEASE-CHANNELS.md §3 spec that develop
  carries `X.Y.Z-dev`. Rejected in favor of D2 (inject at build time).

- **Option B: Per-channel Cargo.toml track (e.g., feature branch `beta` carries `1.0.0-beta.1`).**
  Requires maintaining per-channel branches and merging version bumps across. Operationally
  complex. Rejected.

- **Option C: A separate `VERSION` file as source of truth, read by build.rs.**
  Rejected by ADR-062 Option C for the same reasons: adds a third source of truth alongside
  `Cargo.toml` and the git tag. The git tag IS the authoritative version; `GITHUB_REF_NAME`
  provides direct access to it without an intermediate file.

- **Option D: Set the clap `version` string from a `version.txt` file bundled at compile time.**
  More complex than build.rs env var injection; requires file embedding. Not idiomatic Rust.
  Rejected.

**D3 alternatives (all rejected in favour of cargo-release):**

- **release-plz (0.3.161):** Per-crate version-bump + crates.io-publish model. Conflicts with
  Prism's `publish = false`, single-product-version, 24-sibling-independent-version reality.
  No arbitrary-file regex substitution for SETUP.md. Rejected (same conclusion as ADR-063 §Rationale).

- **knope (0.23.0):** Capable for per-package version + lockfile + `versioned_files` regex +
  tag + pre-release. Rejected because knope generates its own changelog — double-ownership with
  git-cliff (ADR-063). Heavier config than a single cargo-release block.

- **cocogitto (`cog bump`, 7.0.0):** Auto-decides version from Conventional Commits — fights
  prism's deliberate human-gated channel decisions. Monorepo mode emits per-crate tags (wrong for
  single product version). Generates its own changelog (double-ownership with ADR-063).
  No arbitrary-file regex substitution. Rejected.

- **cargo-edit `set-version` (0.13.13):** Manifest-only primitive. Handles Cargo.toml + root
  Cargo.lock (via resolve) but not the second lockfile, no tag, no docs, no changelog. cargo-release
  is built on this class of operation and adds everything missing. Retained as a useful fallback
  for a raw `-dev` bump on develop, but does not meet "one command → all artifacts."

- **dist / cargo-dist (0.32.0, reduced-maint):** Not a version bumper. Builds and hosts artifacts
  triggered by a pushed tag. Its own README recommends cargo-release for the bump step. Prism
  already has `release.yml` for 5-platform builds. Out of scope; flagged for reduced maintenance.

---

## Source / Origin

- `crates/prism-bin/Cargo.toml` — `version = "1.0.0-rc.1"` is the immediate bug being fixed
- `crates/prism-bin/src/main.rs` — version subcommand `println!` (2 call sites)
- `crates/prism-bin/src/cli.rs` — `#[command(version)]` clap derivation (1 site; attributed to cli.rs not main.rs)
- `crates/prism-bin/src/boot.rs` — boot log `tracing::info!` (1 site), `BootAuditEmitter` `let version` (1 site, BC-2.05.012), HTTP user-agent `build_http_client_with_timeout` (2 sites)
- `crates/prism-bin/src/spec_driven_adapter.rs` — spec-driven adapter HTTP user-agent (1 site)
- `crates/prism-spec-engine/src/pipeline.rs` — out-of-scope; acknowledged in D2
- `docs/RELEASE-CHANNELS.md §3` — develop must carry `X.Y.Z-dev`; BASE-MATCH semantics
- `.github/workflows/release-tag.yml` — BASE-MATCH guard; `GITHUB_REF_NAME` availability
- `.github/workflows/release.yml` — 5-platform build matrix; GITHUB_REF_NAME availability per runner
- ADR-062 — D2 (prism-bin tracks tag); Option B/C rejections (their scope was stable, clarified here)
- `.factory/research/version-management-2026.md` — 7-criteria tool matrix; cargo-release configuration surface; docs anti-drift strategy; ownership map

**Proposed story breakdown (hand to product-owner for E-REL-IDENTITY epic):**

| Story ID (proposed) | Title | ADR decisions | Priority | Notes |
|---------------------|-------|---------------|----------|-------|
| S-REL-DEV-RESET-001 | Reset prism-bin to 1.0.0-dev on develop | D1 | BLOCKING (must land before beta.1) | One-line Cargo.toml change + Cargo.lock update + SETUP.md stop-gap |
| S-REL-BVERSION-INJECT-001 | Build-time version injection via build.rs | D2 | BLOCKING (must land before beta.1) | New crates/prism-bin/build.rs; all 6 prism-bin version-report sites updated (main.rs 2 sites, cli.rs `#[command(version)]`, boot.rs boot-log + audit-record + user-agent 2 sites, spec_driven_adapter.rs user-agent); all 5 build targets verified in CI |
| S-REL-VBUMP-001 | cargo-release setup: single-command stable version bump | D3 | High (must land before stable v1.0.0) | Add [package.metadata.release] to prism-bin/Cargo.toml; rewrite release-prep.yml steps 4/6/7 as one cargo release invocation; verify dry-run with --no-execute; release-tag.yml and release-promote.yml unchanged |
| S-REL-DOCS-AGNOSTIC-001 | Make docs version-agnostic and align RELEASING.md with pre-release exception | D3 Docs Anti-Drift | High (must land before stable v1.0.0) | Convert SETUP.md asset-filename table + download URLs to /releases/latest/download/<triple-asset>; convert --version line to format-example; genericize install script examples; removes pre-release-replacements entry if format-example approach adopted. Also update RELEASING.md §1 to document the pre-release exception: develop carries 1.0.0-dev; no Cargo.toml bump is needed between pre-releases on the same X.Y.Z cycle; binary version is injected via PRISM_VERSION (D2). Without this update RELEASING.md §1 contradicts D2 by implying every release requires a Cargo.toml bump. |

---

## Changelog

| Version | Date | Author | Change |
|---------|------|--------|--------|
| 1.3 | 2026-09-05 | architect | NEW-1 count-consistency fix: D2 intro sentence "all four version-report sites" corrected to "all six version-report sites in prism-bin" — sole remaining stale count reference after v1.2 table/header/story/status rewrites. |
| 1.2 | 2026-09-05 | architect | BLOCKING-1: D2 version-report sites table rewritten — 4 → 6 prism-bin sites; clap site attribution corrected from main.rs to cli.rs `#[command(version)]`; boot.rs `let version` audit-record (BC-2.05.012) and spec_driven_adapter.rs adapter user-agent added; out-of-scope prism-spec-engine pipeline.rs site acknowledged. BLOCKING-3: D3 pre-release-hook corrected from flat string array to array-of-arrays; cliff invocation changed from `-o CHANGELOG.md` to `--latest --prepend CHANGELOG.md`. SHOULD-FIX-1: vergen deviation justified in §Rationale (shallow-clone incompatibility). SHOULD-FIX-2: Status section updated to reflect v1.1 finalized state. SHOULD-FIX-3: S-REL-DOCS-AGNOSTIC-001 story scope explicitly names RELEASING.md §1. NIT-2: GITHUB_REF_NAME tag-vs-branch disambiguation added to §Rationale. |
| 1.1 | 2026-09-05 | architect | D3 finalized: cargo-release 1.1.5 as single stable-path bump entrypoint; ownership map defined; docs anti-drift strategy (version-agnostic-first + count-guarded pre-release-replacements); story S-REL-VBUMP-001 scoped. Title updated. Research source: version-management-2026.md. |
| 1.0 | 2026-09-05 | architect | Initial. D1 develop reset to 1.0.0-dev; D2 build.rs GITHUB_REF_NAME injection — PRISM_VERSION replaces CARGO_PKG_VERSION at 4 version-report sites; D3 single-command version bump structure defined, tool selection PENDING version-management-2026.md. Amends ADR-062 D2 for pre-release path. |
