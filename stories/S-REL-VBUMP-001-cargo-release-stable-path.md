---
document_type: story
story_id: S-REL-VBUMP-001
title: "devops: cargo-release config stub + release-prep.yml stable-path rewrite (BEFORE-STABLE)"
wave: F-B
epic_id: E-REL-IDENTITY
priority: P1
status: draft
version: "1.1"
level: "L4"
producer: story-writer
timestamp: "2026-09-05T00:00:00Z"
tdd_mode: facade
# tdd_mode: facade — this story is config + workflow TOML/YAML only; no Rust code.
# No Red Gate tests needed.
subsystems: [SS-22]
# Subsystem anchor justification:
#   SS-22 (Process Lifecycle) owns release toolchain configuration.
crates_touched: []
target_module: devops
capabilities: []
behavioral_contracts: []
# BC status: N/A — release config; no subsystem behavioral contract.
verification_properties: []
depends_on: [S-REL-CLIFF-001]
blocks: []
# Dependency anchor justification:
#   depends_on S-REL-CLIFF-001: release-prep.yml must already incorporate
#     `git cliff` step (S-REL-CLIFF-001) before this story rewrites the
#     Python-heredoc placeholder steps to use `cargo release`. The rewrite builds
#     on the stable state of the file post-CLIFF-001.
# NOT beta.1-blocking: This is BEFORE-STABLE work only.
# The full cargo-release contract will be authored before v1.0.0 stable.
points: 5
estimated_days: 2
risk: MEDIUM
# MEDIUM risk: this is a stub — the full implementation will be authored before
# stable v1.0.0. The stub must be correct but may be incomplete for edge cases
# that only become visible during stable release preparation.
acceptance_criteria_count: 4
red_gate_tests: 0
# red_gate_tests: 0 — facade mode. No Rust code.
estimated_passes: 1
holdout_scenarios: []
assumption_validations: []
risk_mitigations:
  - "cargo-release version 1.1.5 (ADR-064 D3) — pin exactly in install step of
    release-prep.yml; do not use 'latest'."
  - "Stub scope: the cargo-release config in prism-bin/Cargo.toml sets the
    [package.metadata.release] block with the known-correct fields; full pre-release
    tag handling detail is documented as out-of-scope (BEFORE-STABLE)."
  - "release-prep.yml Python-heredoc steps are replaced by `cargo release --dry-run`
    for the stable path only; the pre-release path (S-REL-CLIFF-001 + S-REL-BVERSION-INJECT-001)
    is unchanged."
  - "Full stable release rehearsal must be run before v1.0.0 is cut; see 'Before Stable
    Gate' section below."
inputs:
  - "crates/prism-bin/Cargo.toml"
  - ".github/workflows/release-prep.yml"
  - ".factory/specs/architecture/decisions/ADR-064-pre-release-binary-version-identity.md"
  - ".factory/specs/architecture/decisions/ADR-063-changelog-release-notes-architecture.md"
input-hash: "[pending-recompute]"
traces_to: []
cycle: "v1.0.0-stable-release-prep"
phase: "3"
---

# S-REL-VBUMP-001 — cargo-release Config Stub + release-prep.yml Stable-Path Rewrite (BEFORE-STABLE)

**Story ID:** S-REL-VBUMP-001
**Status:** draft
**Version:** v1.0
**Wave:** F-B
**Priority:** P1 (BEFORE-STABLE; NOT beta.1-blocking)
**Points:** 5
**Beta.1-blocking:** NO — this is BEFORE-STABLE work. Do not implement before beta.1
releases are stable and the stable release timeline is concrete.

---

## Stub Notice

This story is an INTENTIONAL STUB per the user's explicit direction:
> "Stub is fine; full detail authored before stable v1.0.0."

The scope here is:
1. Add `[package.metadata.release]` to `crates/prism-bin/Cargo.toml`
2. Install `cargo-release 1.1.5` in the release-prep.yml workflow
3. Replace the Python-heredoc `tag:` steps in release-prep.yml with
   `cargo release --dry-run` for the stable path

Full cargo-release contract detail (pre-release tag patterns, sign-tag configuration,
publish gates, changelog update hooks) is deferred to a separate full-detail story
that will be authored before `v1.0.0` is cut.

---

## Origin

ADR-064 D3 designates `cargo-release 1.1.5` as the stable-release version bump tool.
The current `release-prep.yml` uses Python heredocs to compute the version bump and
construct the new tag. For the stable path (`v1.0.0`, `v1.0.1`, etc.) this should be
replaced by `cargo release -p prism-bin` which handles semver bump, Cargo.toml update,
and tag creation in a single idiomatic step.

The pre-release path (`v1.0.0-beta.N`) is NOT changed by this story — it remains
driven by the git tag approach established in `S-REL-CLIFF-001` and `S-REL-BVERSION-INJECT-001`.

---

## Narrative

As a release engineer cutting the first stable release of Prism, I want a working
`cargo release` configuration in prism-bin with a dry-run step in release-prep.yml,
so that I can verify the stable-path release process before the actual `v1.0.0` tag
is pushed.

---

## Authority

- ADR-064 D3 — cargo-release 1.1.5 for stable path; pre-release path unchanged
- ADR-063 — git-cliff handles CHANGELOG; cargo-release SHOULD NOT also try to update
  CHANGELOG (disable that feature in cargo-release config)

(No BC: release config; no subsystem behavioral contract.)

---

## Behavioral Contracts

This story has no subsystem behavioral contracts. Authority is ADR-064 D3.

| Architecture Source | Clause |
|---------------------|--------|
| ADR-064 D3 | cargo-release 1.1.5 for stable path only |
| ADR-063 D1 | git-cliff owns CHANGELOG; cargo-release must NOT also update it |
| ADR-064 D1 | prism-bin Cargo.toml already at 1.0.0-dev before this story executes |

---

## Token Budget Estimate

| Artifact | Estimated Tokens |
|----------|-----------------|
| This story spec | ~2,500 |
| `crates/prism-bin/Cargo.toml` | ~500 |
| `.github/workflows/release-prep.yml` | ~2,000 |
| ADR-064 D3 section | ~600 |
| cargo-release docs (subset) | ~1,500 |
| Total | ~7,100 |

Well within the 30% context window budget.

---

## Red Gate Test List (SAC-1)

**tdd_mode: facade — no Rust Red Gate tests.**

Verification is a `cargo release --dry-run` output check:
- The dry-run must exit 0 and show a version bump plan from `1.0.0-dev` to `1.0.0`
  (the first stable release) without errors.

---

## Tasks

1. **Add `[package.metadata.release]` to `crates/prism-bin/Cargo.toml`:**

   ```toml
   [package.metadata.release]
   # cargo-release 1.1.5 (ADR-064 v1.4 D3) — stable path only.
   # Pre-release versioning is handled by build.rs + GITHUB_REF_NAME (ADR-064 D2).
   sign-tag = true
   sign-commit = false
   push = false                  # CI pushes; not cargo-release
   publish = false               # Workspace is not published to crates.io
   # pre-release-commit-message is intentionally OMITTED.
   # That key expects a STRING template (e.g. "chore: release {{version}}"), not a bool.
   # Passing `false` is a type error in cargo-release 1.1.5. To suppress the interim
   # release commit simply omit the key; cargo-release 1.1.5 default is no commit step
   # when the key is absent.
   tag-name = "v{{version}}"
   # CHANGELOG update disabled — git-cliff owns CHANGELOG (ADR-063 D1).
   # pre-release-hook: single bash -c wrapper per ADR-064 v1.4 D3.
   # Runs git-cliff to prepend the new section, then bumps the lockfile pin in the
   # non-exhaustive compile-fail crate so its Cargo.lock tracks the new version.
   pre-release-hook = ["bash", "-c", "git cliff --tag v{{version}} --unreleased --prepend CHANGELOG.md && cargo update -p prism-bin --precise {{version}} --manifest-path tests/external/non-exhaustive-violation/Cargo.toml"]
   ```

   Adjust fields to match actual `cargo-release` 1.1.5 TOML key names — verify
   against `cargo release --list-release-steps` before finalizing. In particular:
   - `disable-publish` is a stale pre-1.0 key not present in cargo-release 1.1.5;
     use `publish = false` (already present above) for the correct key.
   - `pre-release-commit-message` must be a string or omitted; never a bool.

2. **Add `cargo-release` install step to `.github/workflows/release-prep.yml`:**

   In the `Install tools` step (or a new step adjacent to `git-cliff` install):
   ```yaml
   - name: Install cargo-release
     run: cargo install cargo-release --version 1.1.5 --locked
   ```

3. **Add `cargo release --dry-run` step to release-prep.yml stable path:**

   After the git-cliff CHANGELOG generation step, add:
   ```yaml
   - name: Validate stable-path release plan (dry-run)
     if: ${{ !contains(github.ref_name, '-beta') && !contains(github.ref_name, '-rc') }}
     run: cargo release -p prism-bin --dry-run
   ```
   This step is gated to the stable path (no `-beta`, no `-rc` in the tag name) to
   avoid accidentally running cargo-release on pre-release tags.

4. **Replace the Python-heredoc tag step (stable path only):**

   Read the current Python-heredoc `tag:` step in `release-prep.yml`. Replace the
   stable-path invocation with `cargo release -p prism-bin`.

   IMPORTANT: Do NOT replace the pre-release path steps. The pre-release path
   (used for `v1.0.0-beta.N`) continues to use the git-tag approach.

5. **Run dry-run locally to verify AC-003:**

   ```bash
   cargo install cargo-release --version 1.1.5 --locked
   cargo release -p prism-bin --dry-run
   ```
   Expected output: bump plan showing `1.0.0-dev` → `1.0.0`; no CHANGELOG mutation;
   no publish steps.

6. **Verify AC-001 through AC-004.**

---

## Before Stable Gate

Before cutting `v1.0.0`, a SEPARATE full-detail story must be authored that covers:
- Pre-release tag pattern exclusion from cargo-release's default tag scheme
- Signed tag verification
- Full release rehearsal (dry-run against `main`)
- Alignment with the git-cliff `--unreleased --tag` invocation for stable changelog (per ADR-063 v1.2 D5)
- Post-release branch strategy (develop bump back to 1.x.y-dev after stable)

This stub is NOT sufficient for that gate. It is sufficient for:
- Establishing the `[package.metadata.release]` block in Cargo.toml
- Installing cargo-release in CI
- Confirming a dry-run completes without error

---

## Acceptance Criteria

### AC-001: prism-bin/Cargo.toml contains [package.metadata.release] block
`grep '\[package\.metadata\.release\]' crates/prism-bin/Cargo.toml` returns 1 match.
(traces to ADR-064 D3 — cargo-release config required for stable path)

### AC-002: cargo-release disables CHANGELOG updates
The `[package.metadata.release]` block does not configure cargo-release to update
CHANGELOG.md. Either the CHANGELOG-update feature is explicitly disabled, or no
CHANGELOG-related key is present (the default is disabled in cargo-release 1.1.5
unless `changelog = true` is set).
(traces to ADR-063 D1 — git-cliff owns CHANGELOG; cargo-release must not also update it)

### AC-003: cargo release --dry-run exits 0
```bash
cargo release -p prism-bin --dry-run
```
Exits 0 with a version bump plan visible. No CHANGELOG mutation reported.
No publish steps reported (publish disabled).
(traces to ADR-064 D3 — stable-path release must be operable)

### AC-004: release-prep.yml contains a cargo-release dry-run step gated to stable path
`grep 'cargo release.*--dry-run' .github/workflows/release-prep.yml` returns 1 match,
and the surrounding step has an `if:` condition excluding pre-release tags.
(traces to ADR-064 D3 — stable-path CI must validate cargo-release plan)

---

## Previous Story Intelligence

| Story | Key Decisions | Patterns Established | Gotchas Discovered |
|-------|--------------|---------------------|-------------------|
| S-REL-CLIFF-001 | release-prep.yml already has git-cliff step | File structure established | Stable-path steps must be gated; pre-release steps left intact |
| S-REL-BVERSION-INJECT-001 | build.rs handles version; no Cargo.toml bump for pre-releases | cargo-release must NOT be triggered for pre-release tags | ADR-064 D3 is explicit: cargo-release for stable only |

---

## Architecture Compliance Rules

| Rule | Source | Enforcement |
|------|--------|-------------|
| cargo-release version must be 1.1.5 exactly | ADR-064 D3 | `cargo install cargo-release --version 1.1.5 --locked` |
| git-cliff owns CHANGELOG | ADR-063 D1 | AC-002: cargo-release CHANGELOG update disabled |
| cargo-release is stable-path only | ADR-064 D3 | AC-004: dry-run step gated on `!contains(ref, '-beta')` |
| publish disabled (workspace is not a crates.io crate) | ADR-064 D3 prose | `publish = false` in Cargo.toml metadata (`disable-publish` is a stale pre-1.0 key; not valid in 1.1.5) |

---

## Library & Framework Requirements

| Dependency | Version | Notes |
|------------|---------|-------|
| `cargo-release` | `1.1.5` | Pin exactly per ADR-064 D3; `--locked` flag required |

---

## File Structure Requirements

| File | Action | Notes |
|------|--------|-------|
| `crates/prism-bin/Cargo.toml` | Modify | Add `[package.metadata.release]` block |
| `.github/workflows/release-prep.yml` | Modify | Add cargo-release install step + dry-run step gated to stable path |

---

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| `[package.metadata.release]` in prism-bin/Cargo.toml | prism-bin | Pure (TOML config; no runtime behavior) |
| `cargo release` CI step | release-prep.yml | Effectful (reads git state, mutates version files, creates tags) |

---

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| `[package.metadata.release]` TOML block | pure-core | Static configuration; no runtime behavior |
| `cargo release` CLI step in CI | effectful-shell | Reads git history; mutates Cargo.toml version; creates signed git tag |

---

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | cargo-release dry-run attempts to update CHANGELOG.md | If this happens, the CHANGELOG config is wrong; disable per AC-002 |
| EC-002 | dry-run shows publish steps for crates.io | `publish = false` must be set; fix before closing (`disable-publish` is not a valid key in 1.1.5) |
| EC-003 | A pre-release tag triggers the cargo-release step | The `if:` gate must exclude `-beta` and `-rc`; fix the gate expression |
| EC-004 | cargo-release 1.1.5 key names differ from the stub | Verify exact key names with `cargo release help` before finalizing the TOML block |
| EC-005 | Workspace Cargo.toml has features that cargo-release interacts with unexpectedly | Use `-p prism-bin` exclusively; never `cargo release` without `-p` on this workspace |

---

## Changelog

| Version | Date | Author | Change |
|---------|------|--------|--------|
| 1.1 | 2026-09-05 | story-writer | Sync to ADR-064 v1.4 D3 — remove stale `disable-publish` key; remove `pre-release-commit-message = false` (type error; key expects string or omission); add `pre-release-hook` as single bash -c wrapper; fix Before Stable Gate and EC-002 references |
| 1.0 | 2026-09-05 | story-writer | Initial stub — ADR-064 D3 cargo-release configuration; full detail deferred to BEFORE-STABLE |
