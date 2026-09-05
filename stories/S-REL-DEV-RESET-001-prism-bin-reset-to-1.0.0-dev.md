---
document_type: story
story_id: S-REL-DEV-RESET-001
title: "devops: reset prism-bin Cargo.toml version 1.0.0-rc.1 → 1.0.0-dev + lockfile update + SETUP.md stop-gap"
wave: F-A
epic_id: E-REL-IDENTITY
priority: P0
status: draft
version: "1.0"
level: "L4"
producer: story-writer
timestamp: "2026-09-05T00:00:00Z"
tdd_mode: strict
subsystems: [SS-22]
# Subsystem anchor justification:
#   SS-22 (Process Lifecycle) owns prism-bin/Cargo.toml (the version field that
#   determines prism --version output). The reset to 1.0.0-dev brings develop into
#   compliance with RELEASE-CHANNELS.md §3 ("develop always carries X.Y.Z-dev").
crates_touched: [prism-bin]
target_module: prism-bin
capabilities: []
behavioral_contracts: []
# BC status: N/A — version reset is a build-metadata change. No subsystem behavioral
# contract governs Cargo.toml version fields. Conforming per S-REL-002 precedent
# (behavioral_contracts: [] on version alignment stories).
verification_properties: []
depends_on: []
blocks: [S-REL-BVERSION-INJECT-001, S-REL-DOCS-AGNOSTIC-001]
# Dependency anchor justifications:
#   blocks S-REL-BVERSION-INJECT-001: build.rs injection story depends on
#     1.0.0-dev being the Cargo.toml version for the local fallback path
#     (ADR-064 D2 fallback chain step 3: CARGO_PKG_VERSION = "1.0.0-dev" locally).
#   blocks S-REL-DOCS-AGNOSTIC-001: docs story replaces SETUP.md rc.1 strings;
#     DEV-RESET provides the new base-version context for the stop-gap text.
points: 2
estimated_days: 0.5
risk: LOW
acceptance_criteria_count: 5
red_gate_tests: 2
estimated_passes: "1 LOCAL adversary pass"
holdout_scenarios: []
assumption_validations: []
risk_mitigations:
  - "Only prism-bin version changes — grep confirms single-crate edit before commit."
  - "Cargo.lock entries for both the root workspace and tests/external/non-exhaustive-violation
    workspace must each be updated separately. Run cargo update -p prism-bin in each."
  - "BASE-MATCH guard in release-tag.yml compares X.Y.Z core only; 1.0.0-dev core is
    1.0.0 — passes the guard for v1.0.0-beta.1 per ADR-064 D1."
  - "SETUP.md stop-gap in this story is intentionally scoped to one comment line.
    Full version-agnostic conversion belongs to S-REL-DOCS-AGNOSTIC-001."
traces_to: []
inputs:
  - "crates/prism-bin/Cargo.toml"
  - ".factory/specs/architecture/decisions/ADR-064-pre-release-binary-version-identity.md"
input-hash: "[pending-recompute]"
cycle: "v1.0.0-beta.1-release-identity"
phase: "3"
---

# S-REL-DEV-RESET-001 — Reset prism-bin to 1.0.0-dev

**Story ID:** S-REL-DEV-RESET-001
**Status:** draft
**Version:** v1.0
**Wave:** F-A
**Priority:** P0
**Points:** 2
**Beta.1-blocking:** YES — must merge to develop before the v1.0.0-beta.1 tag is cut.

---

## Origin

`crates/prism-bin/Cargo.toml` currently carries `version = "1.0.0-rc.1"`. Develop has
held this value since S-REL-002 (merged PR #253). `docs/RELEASE-CHANNELS.md §3`
specifies that develop always carries `X.Y.Z-dev` between tags. Under BASE-MATCH
(`release-tag.yml`), the version guard compares only the `X.Y.Z` core of the tag
against the `X.Y.Z` core of `prism-bin/Cargo.toml`. Any `v1.0.0-beta.1` tag would
pass the guard, but the built binary would report `prism 1.0.0-rc.1` from
`env!("CARGO_PKG_VERSION")` — a stale, incorrect channel identifier.

ADR-064 D1 mandates the immediate reset to `1.0.0-dev`. This story executes that
one-line change and resolves both Cargo.lock files.

---

## Narrative

As a release engineer, I want `crates/prism-bin/Cargo.toml` to carry `1.0.0-dev`
on the develop branch, so that local builds self-identify as development builds,
develop complies with RELEASE-CHANNELS.md §3, and no future pre-release tag can bake
a stale channel suffix into the binary via `env!("CARGO_PKG_VERSION")`.

---

## Authority

- ADR-064 D1 — Develop Carries `1.0.0-dev` (Immediate Interim)
- `docs/RELEASE-CHANNELS.md §3` — develop always carries `X.Y.Z-dev`

(No BC: version reset is a build-metadata change; conforming per S-REL-002 precedent.)

---

## Behavioral Contracts

This story has no subsystem behavioral contracts. Authority is ADR-064 D1.

| Architecture Source | Clause |
|---------------------|--------|
| ADR-064 D1 | Develop carries `1.0.0-dev`; BASE-MATCH guard passes (core 1.0.0 = 1.0.0) |
| RELEASE-CHANNELS.md §3 | Develop always carries `X.Y.Z-dev` |
| ADR-062 D2 (extended by ADR-064) | Pre-release path: no Cargo.toml bump required between pre-releases |

---

## Token Budget Estimate

| Artifact | Estimated Tokens |
|----------|-----------------|
| This story spec | ~2,500 |
| `crates/prism-bin/Cargo.toml` (~80 lines) | ~1,000 |
| `tests/external/non-exhaustive-violation/Cargo.toml` (~30 lines) | ~500 |
| ADR-064 §D1 (relevant excerpt) | ~800 |
| `docs/RELEASE-CHANNELS.md §3` | ~400 |
| `docs/SETUP.md` (grep lines only) | ~500 |
| Total | ~5,700 |

Well within the 30% context window budget.

---

## Red Gate Test List (SAC-1)

Two Red Gate tests required. Write them as failing tests BEFORE any implementation.

| ID | Test Name | What It Asserts |
|----|-----------|----------------|
| RG-001 | `test_prism_bin_cargo_toml_version_is_dev` | Compile-time: `env!("CARGO_PKG_VERSION") == "1.0.0-dev"` — fails RED before the Cargo.toml edit |
| RG-002 | `test_non_exhaustive_lockfile_version_matches` | Reads `tests/external/non-exhaustive-violation/Cargo.lock` and asserts the `prism-bin` version entry is `"1.0.0-dev"` — fails RED before lockfile regeneration |

**BC-5.38.001 density note:** 2 Red Gate tests / 5 ACs = 0.40 density. Justified below
the 0.5 floor: ACs 003/004 are mechanical grep assertions verifiable without a
compile-time gate; AC-005 is a `just check` pass gate. RG-001 provides the compile-time
gate that the version field is correct; RG-002 provides the lockfile gate for the
nested workspace. These two RG tests are the minimum blocking-failure proof set.

---

## Tasks

**RED GATE FIRST — write failing tests before any implementation.**

1. **Write RG-001 (failing):** Create
   `crates/prism-bin/tests/version_identity.rs` containing:
   ```rust
   #[test]
   fn test_prism_bin_cargo_toml_version_is_dev() {
       assert_eq!(env!("CARGO_PKG_VERSION"), "1.0.0-dev");
   }
   ```
   Run `cargo nextest run -p prism-bin -E 'test(test_prism_bin_cargo_toml_version_is_dev)'`.
   Confirm it FAILS (currently reports `1.0.0-rc.1`).

2. **Write RG-002 (failing):** In the same file, add:
   ```rust
   #[test]
   fn test_non_exhaustive_lockfile_version_matches() {
       let lock = include_str!(
           "../../../tests/external/non-exhaustive-violation/Cargo.lock"
       );
       let idx = lock.find(r#"name = "prism-bin""#)
           .expect("prism-bin entry not found in non-exhaustive Cargo.lock");
       let tail = &lock[idx..];
       assert!(
           tail.contains(r#"version = "1.0.0-dev""#),
           "non-exhaustive-violation Cargo.lock still pins prism-bin to an old version"
       );
   }
   ```
   Confirm it FAILS.

3. **Edit `crates/prism-bin/Cargo.toml`:**
   Change `version = "1.0.0-rc.1"` to `version = "1.0.0-dev"`. No other edits to
   this file.

4. **Regenerate root Cargo.lock:**
   From the workspace root, run `cargo update -p prism-bin`. Confirm
   `Cargo.lock` contains `version = "1.0.0-dev"` for the `prism-bin` entry.

5. **Regenerate non-exhaustive-violation Cargo.lock:**
   From `tests/external/non-exhaustive-violation/`, run `cargo update -p prism-bin`.
   Confirm `tests/external/non-exhaustive-violation/Cargo.lock` contains
   `version = "1.0.0-dev"` for `prism-bin`.

6. **Verify RG-001 and RG-002 go GREEN:**
   Run `just iter prism-bin version_identity` (or equivalent per-crate nextest run).

7. **SETUP.md stop-gap (scoped to one line):**
   Find the line `# Expected: prism 1.0.0-rc.1` in `docs/SETUP.md` and update
   it to `# Expected: prism 1.0.0-dev`. This is a stop-gap only.
   Do NOT change any other SETUP.md occurrence — full conversion is S-REL-DOCS-AGNOSTIC-001.

8. **Single-crate verification:**
   Run `grep -rn '^version = "1.0.0' crates/*/Cargo.toml`.
   Must match only `crates/prism-bin/Cargo.toml`.

9. **Run `just check`** from the workspace root. All tests must pass.

---

## Acceptance Criteria

### AC-001: prism-bin Cargo.toml version is 1.0.0-dev
`grep '^version' crates/prism-bin/Cargo.toml` outputs `version = "1.0.0-dev"`.
No other field in the file changed. (traces to ADR-064 D1 — immediate reset obligation)

### AC-002: Root Cargo.lock updated
`grep -A2 '^name = "prism-bin"' Cargo.lock` shows `version = "1.0.0-dev"`.
(traces to ADR-064 D1 — root Cargo.lock update)

### AC-003: Non-exhaustive-violation Cargo.lock updated
`grep -A2 '^name = "prism-bin"' tests/external/non-exhaustive-violation/Cargo.lock`
shows `version = "1.0.0-dev"`. (traces to ADR-064 D1 / D3 ownership map — both
lockfiles are the responsibility of the version-bumping mechanism)

### AC-004: No other crate version changed
`grep -rn '^version = "1.0.0' crates/*/Cargo.toml` matches exactly one file:
`crates/prism-bin/Cargo.toml`. (traces to ADR-062 D3 — other crates independently versioned)

### AC-005: `just check` passes
After the version reset and both lockfile updates, `just check` exits 0.
(traces to ADR-064 D1 — "story must land on develop before the beta.1 tag is cut")

---

## Previous Story Intelligence

Predecessor: S-REL-002 (merged PR #253 @18646aa44) — set prism-bin to `1.0.0-rc.1`.
That story's pattern (single-crate version change + lockfile + grep verification)
is the template for this story. Key difference: S-REL-002 bumped from `0.1.0`;
this story resets from `rc.1` to `1.0.0-dev`.

Lesson from S-REL-002: `just check` does NOT run `cargo-semver-checks`; that runs
only in `just check-ci` and the pre-tag hook. AC-005 is scoped to `just check` only.

---

## Architecture Compliance Rules

| Rule | Source | Enforcement |
|------|--------|-------------|
| Only prism-bin version changes | ADR-064 D1 | grep: single-crate match |
| Both Cargo.lock files updated | ADR-064 D3 ownership map | Grep on each lockfile |
| Develop carries `X.Y.Z-dev` | RELEASE-CHANNELS.md §3 | AC-001 + AC-002 |
| No new crate dependencies | ADR-064 D1 scope | Cargo.lock diff: only version rows change |
| SETUP.md stop-gap scoped to comment line | ADR-064 D1 note | Single-line grep |

---

## Library & Framework Requirements

| Dependency | Version | Notes |
|------------|---------|-------|
| Rust toolchain | Per `rust-toolchain.toml` | No change |
| `cargo update` | Workspace Cargo | Standard; no extra install |

---

## File Structure Requirements

| File | Action | Notes |
|------|--------|-------|
| `crates/prism-bin/Cargo.toml` | Modify | `version = "1.0.0-rc.1"` → `"1.0.0-dev"` |
| `Cargo.lock` | Regenerate | `cargo update -p prism-bin` from workspace root |
| `tests/external/non-exhaustive-violation/Cargo.lock` | Regenerate | `cargo update -p prism-bin` from that directory |
| `crates/prism-bin/tests/version_identity.rs` | Create | RG-001 + RG-002 compile-time tests |
| `docs/SETUP.md` | Modify (scoped) | Comment line only: `1.0.0-rc.1` → `1.0.0-dev` |

---

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| prism-bin version field | `crates/prism-bin/Cargo.toml` | Pure (build metadata) |
| Non-exhaustive violation test crate | `tests/external/non-exhaustive-violation/` | Pure (compile-time gate) |

---

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| `crates/prism-bin/Cargo.toml` | pure-core | Build metadata change; no runtime I/O |
| `Cargo.lock` | pure-core | Deterministic lockfile regeneration; no I/O at runtime |
| `crates/prism-bin/tests/version_identity.rs` | pure-core | Compile-time `env!()` assertions; no runtime side effects |
| `docs/SETUP.md` | pure-core | Documentation file; no runtime behavior |

---

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Root `cargo update` also regenerates nested lockfile | Does NOT happen — nested workspace has its own lockfile; requires separate `cargo update` inside the nested directory |
| EC-002 | Other workspace crates reference prism-bin via path dep | Path deps resolve by path, not version; no breakage; grep confirms only prism-bin Cargo.toml changed |
| EC-003 | `just check` triggers cargo-semver-checks | `just check` does NOT include semver-checks (check-ci / pre-tag only); AC-005 scoped accordingly |

---

## Changelog

| Version | Date | Author | Change |
|---------|------|--------|--------|
| 1.0 | 2026-09-05 | story-writer | Initial — ADR-064 D1 materialization |
