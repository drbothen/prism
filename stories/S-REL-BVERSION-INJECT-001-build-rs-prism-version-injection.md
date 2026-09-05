---
document_type: story
story_id: S-REL-BVERSION-INJECT-001
title: "devops: build.rs PRISM_VERSION injection — GITHUB_REF_NAME fallback chain + migrate all 6 prism-bin version-report sites"
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
#   SS-22 (Process Lifecycle) owns prism-bin/src/main.rs, boot.rs, cli.rs, and
#   spec_driven_adapter.rs. All six version-report sites live within SS-22's module
#   boundary. The new crates/prism-bin/build.rs is a build-time infrastructure
#   file owned by the prism-bin crate (SS-22).
crates_touched: [prism-bin]
target_module: prism-bin
capabilities: []
behavioral_contracts: []
# BC status: N/A — build-time version injection is a build-infrastructure change.
# No subsystem behavioral contract governs env!() macro source selection.
# Conforming per S-REL-002 precedent (behavioral_contracts: [] on version alignment stories).
verification_properties: []
depends_on: [S-REL-DEV-RESET-001]
blocks: []
# Dependency anchor justifications:
#   depends_on S-REL-DEV-RESET-001: build.rs fallback chain step 3 resolves to
#     CARGO_PKG_VERSION which must be "1.0.0-dev" on develop for the local fallback
#     to be semantically correct (ADR-064 D2 fallback chain; D1 sets that value).
points: 5
estimated_days: 1
risk: MEDIUM
acceptance_criteria_count: 8
red_gate_tests: 4
estimated_passes: "2-3 LOCAL adversary passes"
holdout_scenarios: []
assumption_validations: []
risk_mitigations:
  - "The out-of-scope site in crates/prism-spec-engine/src/pipeline.rs is explicitly
    acknowledged per ADR-064 D2 (cannot be affected by prism-bin/build.rs). Do NOT
    migrate this site in this story."
  - "GITHUB_REF_NAME on release-tag.yml / release-promote.yml runs resolves to the
    BRANCH NAME (develop), not a tag. Those workflows do NOT run the 5-platform
    build matrix. The binary compilation runs in release.yml (tag-triggered), where
    GITHUB_REF_NAME is the tag. ADR-064 §Rationale NIT-2 documents this explicitly."
  - "build.rs uses std::env::var(), not env!(). The emitted PRISM_VERSION env var is
    then consumed by env!() in the Rust source files. Build script runs on the HOST
    machine, not the cross-compilation target."
  - "Concurrent PRISM_BUILD_VERSION + GITHUB_REF_NAME env vars: if both are set, the
    fallback chain uses PRISM_BUILD_VERSION (escape hatch for tooling/testing). This
    is intentional per ADR-064 D2."
inputs:
  - "crates/prism-bin/src/main.rs"
  - "crates/prism-bin/src/cli.rs"
  - "crates/prism-bin/src/boot.rs"
  - "crates/prism-bin/src/spec_driven_adapter.rs"
  - ".factory/specs/architecture/decisions/ADR-064-pre-release-binary-version-identity.md"
input-hash: "[pending-recompute]"
traces_to: []
cycle: "v1.0.0-beta.1-release-identity"
phase: "3"
---

# S-REL-BVERSION-INJECT-001 — Build-time PRISM_VERSION Injection

**Story ID:** S-REL-BVERSION-INJECT-001
**Status:** draft
**Version:** v1.0
**Wave:** F-A
**Priority:** P0
**Points:** 5
**Beta.1-blocking:** YES — must merge to develop before the v1.0.0-beta.1 tag is cut.

---

## Origin

Six distinct version-report sites in `crates/prism-bin` use `env!("CARGO_PKG_VERSION")`.
Because `CARGO_PKG_VERSION` is baked at compile time from `Cargo.toml`, the binary
reports `1.0.0-dev` for every pre-release build on develop, even when compiled by
`release.yml` on a `v1.0.0-beta.1` tag.

ADR-064 D2 mandates a new `crates/prism-bin/build.rs` that reads `GITHUB_REF_NAME`
(set by GitHub Actions on tag-triggered runs) and emits `PRISM_VERSION` as a
compile-time env var. All six sites are migrated from `env!("CARGO_PKG_VERSION")` to
`env!("PRISM_VERSION")`.

---

## Narrative

As a release engineer, I want the `prism` binary to self-report its exact release tag
version on every CI-built artifact, so that `prism --version`, the boot log, and HTTP
user-agent headers all reflect the correct channel identifier (e.g., `1.0.0-beta.1`)
regardless of what `Cargo.toml` carries on develop.

---

## Authority

- ADR-064 D2 — Build-Time Version Injection via `build.rs`
- ADR-064 §Rationale — Why `GITHUB_REF_NAME` (not `vergen`, not a separate CI var)

(No BC: build-infrastructure change; conforming per S-REL-002 precedent.)

---

## Behavioral Contracts

This story has no subsystem behavioral contracts. Authority is ADR-064 D2.

| Architecture Source | Clause |
|---------------------|--------|
| ADR-064 D2 | Fallback chain: PRISM_BUILD_VERSION → GITHUB_REF_NAME stripped of leading v → CARGO_PKG_VERSION |
| ADR-064 D2 | All six prism-bin version-report sites migrated from CARGO_PKG_VERSION to PRISM_VERSION |
| ADR-064 D2 | Out-of-scope: prism-spec-engine/pipeline.rs user-agent is NOT in scope for this story |
| ADR-064 §Rationale NIT-2 | GITHUB_REF_NAME on release.yml (tag-triggered) = tag name; on release-tag.yml/release-promote.yml (workflow_dispatch) = branch name |

---

## Token Budget Estimate

| Artifact | Estimated Tokens |
|----------|-----------------|
| This story spec | ~3,500 |
| `crates/prism-bin/src/main.rs` (relevant sections) | ~3,000 |
| `crates/prism-bin/src/cli.rs` (relevant section) | ~500 |
| `crates/prism-bin/src/boot.rs` (relevant sections) | ~5,000 |
| `crates/prism-bin/src/spec_driven_adapter.rs` (relevant section) | ~1,000 |
| ADR-064 D2 + §Rationale | ~2,000 |
| Total | ~15,000 |

Well within the 30% context window budget.

---

## Red Gate Test List (SAC-1)

Four Red Gate tests required. Write ALL as failing tests before any implementation.

| ID | Test Name | What It Asserts |
|----|-----------|----------------|
| RG-001 | `test_prism_version_env_var_is_available` | `env!("PRISM_VERSION")` compiles — fails RED before `build.rs` emits the var |
| RG-002 | `test_build_rs_fallback_uses_cargo_pkg_version_when_no_env` | When `PRISM_BUILD_VERSION` and `GITHUB_REF_NAME` are unset, `PRISM_VERSION` == `env!("CARGO_PKG_VERSION")` (== `"1.0.0-dev"` after S-REL-DEV-RESET-001) |
| RG-003 | `test_prism_version_site_version_subcommand` | `Commands::Version` output contains `env!("PRISM_VERSION")`, not the old `CARGO_PKG_VERSION` string |
| RG-004 | `test_prism_version_site_user_agent` | HTTP user-agent string in `build_http_client_with_timeout` contains `env!("PRISM_VERSION")` |

**BC-5.38.001 density note:** 4 Red Gate tests / 8 ACs = 0.50 density — exactly at the
required floor. The four tests gate the most critical failure modes: build.rs compilation,
fallback chain semantics, version subcommand surface, and user-agent string.

---

## Tasks

**RED GATE FIRST — write failing tests before any implementation.**

1. **Write RG-001 (failing):** In `crates/prism-bin/tests/version_identity.rs` (extend
   from S-REL-DEV-RESET-001), add:
   ```rust
   #[test]
   fn test_prism_version_env_var_is_available() {
       // build.rs must emit PRISM_VERSION; env!() fails to compile if absent.
       let v: &str = env!("PRISM_VERSION");
       assert!(!v.is_empty(), "PRISM_VERSION must be non-empty");
   }
   ```
   Verify it FAILS to compile (PRISM_VERSION not yet emitted). Do NOT commit yet.

2. **Write RG-002 (failing):** In the same file, add:
   ```rust
   #[test]
   fn test_build_rs_fallback_uses_cargo_pkg_version_when_no_env() {
       // In local dev (no GITHUB_REF_NAME), PRISM_VERSION must equal CARGO_PKG_VERSION.
       // After S-REL-DEV-RESET-001, both should be "1.0.0-dev".
       assert_eq!(env!("PRISM_VERSION"), env!("CARGO_PKG_VERSION"));
   }
   ```
   Verify it FAILS to compile.

3. **Write RG-003 (failing):** Locate the `Commands::Version` arm in `main.rs` (two
   `println!` sites). Add a test (or extend the existing version test if present) that
   checks the printed string uses `env!("PRISM_VERSION")`. Since this is a compile-time
   constant, the test asserts the literal string compiled in at build time.

4. **Write RG-004 (failing):** In a test file that exercises
   `build_http_client_with_timeout`, add an assertion that the user-agent header matches
   `format!("prism/{}", env!("PRISM_VERSION"))`. Verify it fails before migration.

5. **Create `crates/prism-bin/build.rs`** with the normative contract from ADR-064 D2:
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

6. **Migrate site 1 — `src/main.rs` (2 call sites):**
   Replace both `env!("CARGO_PKG_VERSION")` occurrences in `Commands::Version` arm
   (`println!` at function containing line ~55 and line ~89) with `env!("PRISM_VERSION")`.

7. **Migrate site 2 — `src/cli.rs` (1 call site):**
   Change `#[command(version)]` on `CliArgs` to
   `#[command(version = env!("PRISM_VERSION"))]` (line ~34).

8. **Migrate site 3 — `src/boot.rs` boot log (1 call site):**
   Change `tracing::info!("Prism v{}", env!("CARGO_PKG_VERSION"))` (line ~890) to
   `tracing::info!("Prism v{}", env!("PRISM_VERSION"))`.

9. **Migrate site 4 — `src/boot.rs` BootAuditEmitter (1 call site):**
   Change `let version = env!("CARGO_PKG_VERSION")` (line ~1862) to
   `let version = env!("PRISM_VERSION")`.

10. **Migrate sites 5+6 — `src/boot.rs` user-agent (2 call sites):**
    Change both `.user_agent(concat!("prism/", env!("CARGO_PKG_VERSION")))` in
    `build_http_client_with_timeout` (lines ~2001 and ~2027) to
    `.user_agent(concat!("prism/", env!("PRISM_VERSION")))`.

11. **Migrate site 7 — `src/spec_driven_adapter.rs` user-agent (1 call site):**
    Change `.user_agent(concat!("prism/", env!("CARGO_PKG_VERSION")))` (line ~1716) to
    `.user_agent(concat!("prism/", env!("PRISM_VERSION")))`.

12. **Verify no CARGO_PKG_VERSION remains in prism-bin src:**
    `grep -rn 'CARGO_PKG_VERSION' crates/prism-bin/src/` must return empty.
    (The test file at `crates/prism-bin/tests/version_identity.rs` may retain
    `CARGO_PKG_VERSION` for the fallback-chain test — that is correct.)

13. **Verify out-of-scope site is NOT touched:**
    `grep 'CARGO_PKG_VERSION' crates/prism-spec-engine/src/pipeline.rs` must still match
    (unchanged per ADR-064 D2 explicit out-of-scope acknowledgement).

14. **Run RG-001..RG-004 green** via `just iter prism-bin`.

15. **Run `just check`** from workspace root. All tests must pass.

---

## Acceptance Criteria

### AC-001: build.rs exists with normative fallback chain
`ls crates/prism-bin/build.rs` exits 0. File contains `PRISM_BUILD_VERSION`,
`GITHUB_REF_NAME`, `CARGO_PKG_VERSION`, and `cargo:rustc-env=PRISM_VERSION` per
ADR-064 D2 normative contract. (traces to ADR-064 D2 — build.rs normative contract)

### AC-002: build.rs emits rerun-if-env-changed for both vars
`grep 'cargo:rerun-if-env-changed' crates/prism-bin/build.rs` returns two lines,
one for `PRISM_BUILD_VERSION` and one for `GITHUB_REF_NAME`. (traces to ADR-064 D2 —
cargo rebuild semantics: build.rs re-runs only when these env vars change)

### AC-003: All six prism-bin version-report sites use PRISM_VERSION
`grep -rn 'CARGO_PKG_VERSION' crates/prism-bin/src/` returns no output.
(traces to ADR-064 D2 — "All six prism-bin version-report sites are updated")

### AC-004: prism-spec-engine/pipeline.rs user-agent NOT changed
`grep 'CARGO_PKG_VERSION' crates/prism-spec-engine/src/pipeline.rs` returns at least
one match. (traces to ADR-064 D2 — explicit out-of-scope acknowledgement)

### AC-005: Fallback chain test passes (local dev path)
`test_build_rs_fallback_uses_cargo_pkg_version_when_no_env` passes: `env!("PRISM_VERSION")
== env!("CARGO_PKG_VERSION")` (both resolve to `"1.0.0-dev"` on develop).
(traces to ADR-064 D2 fallback chain — step 3: CARGO_PKG_VERSION local fallback)

### AC-006: cli.rs clap version attribution updated
`grep '#\[command(version' crates/prism-bin/src/cli.rs` shows
`#[command(version = env!("PRISM_VERSION"))]` (not the implicit form).
(traces to ADR-064 D2 — cli.rs site: "fix = #[command(version = env!("PRISM_VERSION"))]")

### AC-007: boot.rs BootAuditEmitter version updated
`grep 'let version = env!' crates/prism-bin/src/boot.rs` shows `PRISM_VERSION` (not
`CARGO_PKG_VERSION`). (traces to ADR-064 D2 — boot.rs BootAuditEmitter site;
BC-2.05.012 audit record must report the correct product version)

### AC-008: `just check` passes
All workspace tests pass after the migration. (traces to ADR-064 D2 — "All 5 build
targets verified in CI"; `just check` is the local proxy for CI correctness)

---

## Previous Story Intelligence

| Story | Key Decisions | Patterns Established | Gotchas Discovered |
|-------|--------------|---------------------|-------------------|
| S-REL-DEV-RESET-001 | Set prism-bin Cargo.toml to `1.0.0-dev` | Version test file at `crates/prism-bin/tests/version_identity.rs` established | `just check` does not run cargo-semver-checks; use `just iter prism-bin` for fast inner loop |

This story extends `tests/version_identity.rs` created in S-REL-DEV-RESET-001 rather
than creating a new test file.

---

## Architecture Compliance Rules

| Rule | Source | Enforcement |
|------|--------|-------------|
| All six prism-bin sites migrated | ADR-064 D2 six-site table | grep returns empty for `CARGO_PKG_VERSION` in `src/` |
| prism-spec-engine out-of-scope | ADR-064 D2 explicit scope boundary | grep confirms pipeline.rs unchanged |
| build.rs fallback chain: PRISM_BUILD_VERSION > GITHUB_REF_NAME > CARGO_PKG_VERSION | ADR-064 D2 normative | RG-002 tests the fallback order |
| GITHUB_REF_NAME on release.yml is the tag; on workflow_dispatch runs is the branch | ADR-064 §Rationale NIT-2 | No test needed: CI topology invariant |
| No `vergen` crate added | ADR-064 §Rationale (rejected vergen: shallow-clone incompatibility) | Cargo.lock diff shows no vergen entry |
| build.rs must emit rerun-if-env-changed for both trigger vars | ADR-064 D2 | AC-002 grep check |

---

## Library & Framework Requirements

| Dependency | Version | Notes |
|------------|---------|-------|
| Rust toolchain | Per `rust-toolchain.toml` | No change; build.rs uses std only |
| `std::env::var` | std | Fallback chain reads env vars at build time |
| `cargo:rustc-env` | Cargo build script protocol | Emits compile-time env var for `env!()` macro |

---

## File Structure Requirements

| File | Action | Notes |
|------|--------|-------|
| `crates/prism-bin/build.rs` | Create | Normative fallback chain per ADR-064 D2 |
| `crates/prism-bin/src/main.rs` | Modify | 2 sites: `CARGO_PKG_VERSION` → `PRISM_VERSION` |
| `crates/prism-bin/src/cli.rs` | Modify | 1 site: `#[command(version)]` → `#[command(version = env!("PRISM_VERSION"))]` |
| `crates/prism-bin/src/boot.rs` | Modify | 4 sites: boot log + BootAuditEmitter + 2× user-agent |
| `crates/prism-bin/src/spec_driven_adapter.rs` | Modify | 1 site: user-agent |
| `crates/prism-bin/tests/version_identity.rs` | Modify | Extend with RG-001..RG-004 |
| `crates/prism-spec-engine/src/pipeline.rs` | DO NOT modify | Explicitly out of scope per ADR-064 D2 |

---

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| `build.rs` | `crates/prism-bin/build.rs` | Pure (reads env vars at build time; emits compile-time directive) |
| Version subcommand | `crates/prism-bin/src/main.rs` `Commands::Version` | Pure (compile-time constant) |
| clap version flag | `crates/prism-bin/src/cli.rs` `#[command(version)]` | Pure (compile-time constant) |
| Boot log emission | `crates/prism-bin/src/boot.rs` | Effectful (tracing emission) |
| BootAuditEmitter version | `crates/prism-bin/src/boot.rs` | Effectful (audit record) |
| HTTP user-agent | `crates/prism-bin/src/boot.rs` + `src/spec_driven_adapter.rs` | Effectful (HTTP header) |

---

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| `crates/prism-bin/build.rs` | pure-core | Reads env vars and emits build directives only; no I/O at runtime |
| `crates/prism-bin/src/main.rs` (Version arm) | pure-core | `env!()` is a compile-time constant; the `println!` side-effect is deliberate CLI output |
| `crates/prism-bin/src/cli.rs` (version attr) | pure-core | Clap derive attribute; compile-time |
| `crates/prism-bin/src/boot.rs` (boot log + audit + user-agent) | effectful-shell | tracing::info!, BootAuditEmitter, reqwest user-agent header are all I/O side effects |
| `crates/prism-bin/src/spec_driven_adapter.rs` (user-agent) | effectful-shell | reqwest HTTP header; I/O |

---

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Both `PRISM_BUILD_VERSION` and `GITHUB_REF_NAME` are set | `PRISM_BUILD_VERSION` wins (first in fallback chain) per ADR-064 D2 |
| EC-002 | `GITHUB_REF_NAME` is set to a branch name (e.g., `develop`) rather than a tag | `build.rs` strips the leading `v` and uses whatever value is present; `1.0.0-dev` (via CARGO_PKG_VERSION fallback) is the correct local-dev identity; branch-name injection is only a risk if release-tag.yml/release-promote.yml triggered the build matrix (they do not — only release.yml is tag-triggered; see ADR-064 §Rationale NIT-2) |
| EC-003 | `GITHUB_REF_NAME` is `v1.0.0-beta.1` (leading `v`) | `trim_start_matches('v')` → `1.0.0-beta.1`; correct per ADR-064 D2 |
| EC-004 | All three env vars absent (local dev) | Falls through to `CARGO_PKG_VERSION` = `1.0.0-dev`; correct local identity |
| EC-005 | `prism-spec-engine/pipeline.rs` user-agent after this story | Still reads `CARGO_PKG_VERSION` from prism-spec-engine's own Cargo.toml; the crate version, not the product version. Documented out-of-scope per ADR-064 D2 |

---

## Changelog

| Version | Date | Author | Change |
|---------|------|--------|--------|
| 1.0 | 2026-09-05 | story-writer | Initial — ADR-064 D2 materialization |
