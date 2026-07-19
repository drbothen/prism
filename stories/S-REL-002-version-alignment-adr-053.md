---
document_type: story
story_id: S-REL-002
title: "devops: prism-bin version alignment to 1.0.0-rc.1 + ADR-053 product version policy"
wave: F-A
epic_id: E-REL
priority: P0
status: draft
version: "0.2"
level: "L4"
producer: story-writer
timestamp: "2026-07-19T00:00:00Z"
tdd_mode: strict
subsystems: [SS-22]
# Subsystem anchor justification:
#   SS-22 (Binary Entrypoint) owns prism-bin/Cargo.toml (the version field that determines
#   `prism --version` output via env!("CARGO_PKG_VERSION")). The version bump to 1.0.0-rc.1
#   is a prism-bin boundary change per ARCH-INDEX Subsystem Registry SS-22.
crates_touched: [prism-bin]
target_module: prism-bin
capabilities: []
behavioral_contracts: []
# BC status: N/A — version alignment is a build-metadata change. No subsystem behavioral
# contract governs Cargo.toml version fields. Conforming per W3-FIX-CI-001 precedent.
verification_properties: []
depends_on: []
blocks: [S-REL-004, S-REL-006]
# Dependency anchor justifications:
#   blocks S-REL-004: demo-bundle packaging embeds the version string in the bundle archive
#     name; the tag must correspond to the correct prism-bin version after the bump.
#   blocks S-REL-006: consumer-contract.md §5.2 pins `prism --version` to `prism 1.0.0-rc.1`;
#     this story must be merged before the contract can be graduated.
points: 2
estimated_days: 1
risk: LOW
# Risk justification: Single-field change in Cargo.toml; publish = false; no dependents
# consume the version field at build time. cargo semver-checks treats 0.1.0→1.0.0-rc.1
# as a MAJOR bump (MAJOR 0→1) and runs checks — but allows breaking changes for a major
# bump, so the check passes trivially. Semver-checks runs in check-ci / pre-tag, NOT
# in `just check`. Risk is LOW per delta-analysis §8.
acceptance_criteria_count: 6
red_gate_tests: 2
estimated_passes: "1-2 LOCAL adversary passes"
holdout_scenarios: []
assumption_validations: []
risk_mitigations:
  - "Cargo semver-checks MAJOR bump behavior (U6): semver-checks treats 0.1.0→1.0.0-rc.1
    as a MAJOR bump (MAJOR 0→1) and RUNS checks — it does NOT skip on pre-release. It
    allows breaking changes for a major version bump, so the check passes trivially.
    There is NO documented pre-release exemption that skips baseline comparison.
    semver-checks runs in `just check-ci` and the pre-tag hook — NOT in `just check`.
    Correct AC-004 wording accordingly: `just check` passes (no semver-checks); the
    pre-tag hook runs semver-checks via `just check-ci`."
  - "ADR-053 must be registered in ARCH-INDEX.md before the story can close. Same commit."
  - "Verify only prism-bin changed: grep '^version' crates/*/Cargo.toml to confirm."
inputs:
  - "crates/prism-bin/Cargo.toml"
  - ".factory/planning/feature-release-engineering/delta-analysis.md"
  - ".factory/planning/feature-release-engineering/prism-consumer-contract.md"
  - ".factory/research/release-engineering-uncertainties-2026.md"
input-hash: "d180c7d"
traces_to: []
cycle: "v1.0.0-release-engineering"
phase: "F3"
---

# S-REL-002 — devops: prism-bin version alignment to 1.0.0-rc.1 + ADR-053

**Story ID:** S-REL-002
**Status:** draft
**Version:** v0.2
**Wave:** F-A
**Priority:** P0
**Points:** 2

---

## Origin

`crates/prism-bin/Cargo.toml` currently has `version = "0.1.0"`. The secops-factory
`activate` skill checks `prism --version` against a minimum required version of
`1.0.0-rc.1`. Bumping prism-bin to `1.0.0-rc.1` aligns the binary's self-reported version
with the product tag. ADR-053 documents the policy: product version = git tag; prism-bin
version tracks the product tag; all other crates remain independently versioned.

Per delta-analysis §4, no `[workspace.package]` is introduced — crate versions remain
non-uniform (this is intentional; each internal crate has independent evolution history).

---

## Narrative

As a release engineer, I want `prism --version` to output `prism 1.0.0-rc.1` and for
that policy to be codified in an ADR, so that the secops-factory version check passes and
the versioning policy is documented for future maintainers.

---

## Behavioral Contracts

This story has no subsystem BCs — version alignment is a build-metadata change.

| Architecture Source | Clause |
|--------------------|--------|
| `delta-analysis.md` §4 (version alignment) | Product version = git tag; prism-bin aligned; other crates independent |
| `prism-consumer-contract.md` §5.2 | `prism --version` outputs `prism 1.0.0-rc.1` |
| `secops-factory-handoff-brief.md` §2.1 | `activate` skill asserts >= 1.0.0-rc.1 |

---

## Token Budget Estimate

| Artifact | Estimated Tokens |
|----------|-----------------|
| This story spec | ~2,000 |
| `crates/prism-bin/Cargo.toml` (~40 lines) | ~600 |
| `delta-analysis.md` §4 | ~800 |
| `prism-consumer-contract.md` §5 | ~600 |
| `ARCH-INDEX.md` (for ADR registration) | ~5,000 |
| `release-engineering-uncertainties-2026.md` U6 | ~500 |
| Total | ~9,500 |

Well within the 30% context window budget.

---

## Tasks

1. **Read `crates/prism-bin/Cargo.toml`** in full.

2. **Bump version:**
   Change `version = "0.1.0"` to `version = "1.0.0-rc.1"` in `crates/prism-bin/Cargo.toml`.
   No other changes to this file.

3. **Verify no other crates changed:**
   `grep -rn '^version = "1.0.0-rc' crates/*/Cargo.toml` must match only `prism-bin`.

4. **Run `just check`:**
   Confirm compilation + tests pass. Note: `just check` does NOT run cargo-semver-checks.
   cargo-semver-checks runs in `just check-ci` and the pre-tag hook. When it does run,
   it treats 0.1.0→1.0.0-rc.1 as a MAJOR bump (allows breaking changes) and passes
   trivially — there is no prerelease exemption that skips the check (research U6).

5. **Verify `prism --version` output:**
   After `cargo build -p prism-bin --release`, run:
   - `./target/release/prism --version` → must output `prism 1.0.0-rc.1`
   - `./target/release/prism version` → must output the same string.

6. **Create ADR-053** at `.factory/specs/architecture/decisions/ADR-053-product-version-alignment.md`:

   ```markdown
   ---
   adr_id: ADR-053
   title: "Product Version Alignment — git tag as canonical version; prism-bin aligned; crates independent"
   status: ACCEPTED
   version: "1.0"
   date: 2026-07-19
   supersedes: []
   superseded_by: []
   ---

   # ADR-053: Product Version Alignment

   ## Context
   The prism workspace contains 24 crates with independently evolved version numbers.
   The product ships as `prism` from `prism-bin`. Consumers need a predictable version string.
   cargo semver-checks treats the 0.1.0→1.0.0-rc.1 transition as a MAJOR bump (allows
   breaking changes) and passes trivially; no prerelease exemption is assumed.

   ## Decision
   1. Product version = git tag (v1.0.0-rc.1, v1.0.0, etc.).
   2. prism-bin version tracks the product tag; bumped before each release.
   3. All other crates retain independent versioning (publish = false; no user-visible semver contract).
   4. No [workspace.package] introduced (non-uniform versions reflect genuine independent evolution).
   5. Canonical version check form: `prism --version` (clap auto-flag).

   ## Consequences
   - Before tagging a release, bump crates/prism-bin/Cargo.toml to match the release tag.
   - All other crates remain unchanged.
   - cargo semver-checks interprets 0.1.0→1.0.0-rc.1 as MAJOR bump; allows breaking changes;
     no skip, no exemption. Check passes trivially.
   - docs/consumer-contract.md pins the expected prism --version output per release.
   ```

7. **Register ADR-053 in ARCH-INDEX.md:**
   Add a row to the ADR Registry table:
   ```
   | ADR-053 | Product Version Alignment — git tag as canonical version; prism-bin aligned; crates independent | ACCEPTED v1.0 | 2026-07-19 | decisions/ADR-053-product-version-alignment.md |
   ```

---

## Acceptance Criteria

### AC-001: prism-bin Cargo.toml version updated
Given: `crates/prism-bin/Cargo.toml` is read.
When: `grep '^version' crates/prism-bin/Cargo.toml` is run.
Then: Output is `version = "1.0.0-rc.1"`. No other field in the file changed.
(traces to delta-analysis.md §4: "Only prism-bin/Cargo.toml version changes from 0.1.0
to 1.0.0-rc.1. All other crates unchanged.")

### AC-002: `prism --version` outputs the correct string
Given: `cargo build -p prism-bin --release` has run after the version bump.
When: `./target/release/prism --version` is executed.
Then: Output is exactly `prism 1.0.0-rc.1`. Exit code 0.
(traces to prism-consumer-contract.md §5.2: "prism --version canonical form; output:
prism 1.0.0-rc.1")

### AC-003: No other crate version changed
Given: The workspace Cargo.toml files.
When: `grep -rn '^version = "1.0.0-rc' crates/*/Cargo.toml` is run.
Then: Exactly one match: `crates/prism-bin/Cargo.toml`.
(traces to delta-analysis.md §4: "No workspace.package change needed")

### AC-004: `just check` passes
Given: The version bump is applied.
When: `just check` is run.
Then: Exit code 0. All tests pass. Note: `just check` does NOT include cargo-semver-checks
(semver-checks runs in `just check-ci` / pre-tag hook only).
(traces to delta-analysis.md §8: "Verify just check passes; verify prism version prints
new string"; research U6: semver-checks in check-ci not just check)

### AC-005: ADR-053 file created with required content
Given: The implementation is complete.
When: `ls .factory/specs/architecture/decisions/ADR-053*.md` is run.
Then: Exactly one file found. The file contains all five decision points from Tasks §6.
ADR body correctly notes that cargo-semver-checks treats 0.1.0→1.0.0-rc.1 as a MAJOR bump
(no pre-release exemption).
(traces to delta-analysis.md §2.1: "New ADR: product version vs crate version relationship")

### AC-006: ADR-053 registered in ARCH-INDEX.md
Given: `.factory/specs/architecture/ARCH-INDEX.md` is read.
When: `grep 'ADR-053' .factory/specs/architecture/ARCH-INDEX.md` is run.
Then: Exactly one row found with ADR-053, status ACCEPTED, correct path.
(traces to delta-analysis.md §2.1: "ADR-053 proper registration")

---

## Previous Story Intelligence

N/A — first story in E-REL touching Rust build metadata. S-REL-001 (parallel) modifies
only GitHub Actions YAML. These two stories are fully independent within Wave F-A.

---

## Architecture Compliance Rules

| Rule | Source | Enforcement |
|------|--------|-------------|
| Only prism-bin version changes for 1.0.0-rc.1 | delta-analysis §4 | Grep confirms single-crate change |
| `prism --version` reads `env!("CARGO_PKG_VERSION")` | `crates/prism-bin/src/main.rs` | Existing code path; no changes to main.rs |
| ADR must be ACCEPTED before story closes | VSDD ADR lifecycle | ADR-053 status: ACCEPTED in frontmatter |
| No new crate dependencies | delta-analysis §5: no engine changes | Cargo.lock diff shows only prism-bin version bump |
| semver-checks runs in check-ci / pre-tag, NOT just check | Research U6; Justfile | AC-004 scoped to `just check` only |

---

## Library & Framework Requirements

| Dependency | Version | Notes |
|------------|---------|-------|
| Rust toolchain | Per `rust-toolchain.toml` | No change |
| `cargo-semver-checks` | As installed by `just setup` | Runs in check-ci/pre-tag; treats 0.1.0→1.0.0-rc.1 as MAJOR bump; allows breaking changes; passes trivially |

---

## File Structure Requirements

| File | Action | Notes |
|------|--------|-------|
| `crates/prism-bin/Cargo.toml` | Modify | `version = "0.1.0"` → `"1.0.0-rc.1"` |
| `.factory/specs/architecture/decisions/ADR-053-product-version-alignment.md` | Create | See Tasks §6 |
| `.factory/specs/architecture/ARCH-INDEX.md` | Modify | Add ADR-053 row |

---

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|----------------|
| prism-bin version metadata | `crates/prism-bin/Cargo.toml` | N/A (build metadata) |
| `prism --version` output | `crates/prism-bin/src/main.rs` `env!("CARGO_PKG_VERSION")` | Pure (compile-time constant) |
| ADR-053 | `.factory/specs/architecture/decisions/` | N/A (spec artifact) |

---

## Purity Classification

| Module | Classification | Justification |
|--------|----------------|---------------|
| `crates/prism-bin/Cargo.toml` | N/A | Build metadata — no Rust purity boundary applies |
| ADR-053 | N/A | Spec artifact — no Rust purity boundary applies |

---

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | semver-checks on 1.0.0-rc.1 | MAJOR bump (0→1); allows breaking changes; passes trivially; no prerelease skip |
| EC-002 | `prism version` (subcommand) | Also outputs `prism 1.0.0-rc.1`; both forms equivalent |
| EC-003 | Future bump to 1.0.0 | Only prism-bin changes again; policy codified in ADR-053 |

---

## Forbidden Dependencies

- No `[workspace.package]` introduction (explicitly decided against in delta-analysis §4)
- No version bumps to any crate other than prism-bin

---

## Changelog

| Version | Date | Summary |
|---------|------|---------|
| 0.2 | 2026-07-19 | Fix-burst: U6 semver-checks framing corrected — 0.1.0→1.0.0-rc.1 is MAJOR bump (runs, not skipped); semver-checks is in check-ci/pre-tag NOT just check; risk_mitigations + AC-004 + EC-001 + ADR-053 body updated accordingly; research file added to inputs |
| 0.1 | 2026-07-19 | Initial story creation (story-writer F3 burst) |
