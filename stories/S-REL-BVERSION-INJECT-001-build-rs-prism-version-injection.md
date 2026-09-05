---
document_type: story
story_id: S-REL-BVERSION-INJECT-001
title: "devops: build.rs PRISM_VERSION injection — GITHUB_REF_NAME fallback chain + migrate all 6 prism-bin version-report sites"
wave: F-A
epic_id: E-REL-IDENTITY
priority: P0
status: draft
version: "1.1"
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
  - "GITHUB_REF_NAME is set on ALL GitHub Actions runs, including push/pull_request
    (where it is the branch name). The tag-guard in build.rs (GITHUB_REF_TYPE=='tag'
    per ADR-064 v1.6 D2 F-VID-P1-CRIT-001) prevents branch names from being baked
    into the binary. Only release.yml (tag-triggered) sets GITHUB_REF_TYPE='tag'.
    release-tag.yml and release-promote.yml run as workflow_dispatch (branch context)
    but do NOT run the 5-platform build matrix, so they never compile prism-bin.
    ADR-064 §Rationale NIT-2 documents the CI topology."
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
| ADR-064 v1.6 D2 | Fallback chain: PRISM_BUILD_VERSION → GITHUB_REF_NAME (single-v stripped via strip_prefix, only when GITHUB_REF_TYPE=="tag") → CARGO_PKG_VERSION; empty/whitespace values rejected on every arm |
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
| RG-002 | `test_build_rs_fallback_uses_cargo_pkg_version_when_no_env` | When `PRISM_BUILD_VERSION` is unset AND `GITHUB_REF_TYPE` is absent or `"branch"` (non-tag run), `PRISM_VERSION` == `env!("CARGO_PKG_VERSION")` (== `"1.0.0-dev"` after S-REL-DEV-RESET-001). Covers the tag-guard: the `GITHUB_REF_NAME` arm is skipped when `is_tag_build` is `false` (ADR-064 v1.6 F-VID-P1-CRIT-001 fix) |
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

5. **Create `crates/prism-bin/build.rs`** with the normative contract from ADR-064 v1.6 D2:
   ```rust
   fn main() {
       println!("cargo:rerun-if-env-changed=PRISM_BUILD_VERSION");
       println!("cargo:rerun-if-env-changed=GITHUB_REF_NAME");
       println!("cargo:rerun-if-env-changed=GITHUB_REF_TYPE");
       println!("cargo:rerun-if-env-changed=GITHUB_REF");

       let cargo_version = std::env!("CARGO_PKG_VERSION");

       // GITHUB_REF_NAME is set on ALL GitHub Actions runs — branch name on
       // push/pull_request events, tag name only on tag-push events.
       // Gate on GITHUB_REF_TYPE == "tag" (or GITHUB_REF prefix) to avoid baking
       // "develop" / "feature/..." into the binary on non-release CI runs.
       // F-VID-P1-CRIT-001: unconditional use would set PRISM_VERSION="develop"
       // on ci.yml legs, failing test_cli_version_output_contains_semver.
       let is_tag_build = std::env::var("GITHUB_REF_TYPE")
           .ok()
           .filter(|s| !s.trim().is_empty())
           .map(|t| t.trim() == "tag")
           .unwrap_or_else(|| {
               // Fallback for environments that set GITHUB_REF but not GITHUB_REF_TYPE.
               std::env::var("GITHUB_REF")
                   .ok()
                   .filter(|s| !s.trim().is_empty())
                   .map(|r| r.starts_with("refs/tags/"))
                   .unwrap_or(false)
           });

       let version = std::env::var("PRISM_BUILD_VERSION")
           .ok()
           .filter(|s| !s.trim().is_empty())
           .or_else(|| {
               if is_tag_build {
                   // Strip a SINGLE leading 'v' via strip_prefix (not trim_start_matches
                   // which would strip all leading v's — F-VID-P1-LOW-001).
                   // e.g. "v1.0.0-beta.1" -> "1.0.0-beta.1". Trim whitespace first.
                   std::env::var("GITHUB_REF_NAME")
                       .ok()
                       .filter(|s| !s.trim().is_empty())
                       .map(|r| {
                           let name = r.trim();
                           name.strip_prefix('v').unwrap_or(name).to_string()
                       })
               } else {
                   None
               }
           })
           .unwrap_or_else(|| cargo_version.to_string());

       println!("cargo:rustc-env=PRISM_VERSION={}", version);
   }
   ```
   Key changes from the pre-v1.5 form (ADR-064 v1.5/v1.6, findings F-VID-P1-CRIT-001/LOW-001/MED-001):
   - Four `rerun-if-env-changed` lines (adds GITHUB_REF_TYPE, GITHUB_REF).
   - `is_tag_build` guard: GITHUB_REF_NAME is only used when GITHUB_REF_TYPE=="tag".
   - `strip_prefix('v')` (single-strip) replaces `trim_start_matches('v')` (all-strip).
   - Empty/whitespace filter on every env-var arm via `.filter(|s| !s.trim().is_empty())`.

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
`GITHUB_REF_NAME`, `GITHUB_REF_TYPE`, `GITHUB_REF`, `CARGO_PKG_VERSION`,
`is_tag_build`, and `cargo:rustc-env=PRISM_VERSION` per ADR-064 v1.6 D2 normative
contract (tag-guard + strip_prefix + empty-filter). (traces to ADR-064 v1.6 D2 —
build.rs normative contract)

### AC-002: build.rs emits rerun-if-env-changed for all four trigger vars
`grep 'cargo:rerun-if-env-changed' crates/prism-bin/build.rs` returns exactly four
lines: `PRISM_BUILD_VERSION`, `GITHUB_REF_NAME`, `GITHUB_REF_TYPE`, and `GITHUB_REF`.
(traces to ADR-064 v1.6 D2 — cargo rebuild semantics: build.rs re-runs only when
these env vars change; GITHUB_REF_TYPE and GITHUB_REF are required for the tag-guard
that prevents branch-name leak — F-VID-P1-CRIT-001)

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
| build.rs fallback chain: PRISM_BUILD_VERSION → GITHUB_REF_NAME (tag-gated via GITHUB_REF_TYPE) → CARGO_PKG_VERSION | ADR-064 v1.6 D2 normative | RG-002 tests the tag-guard fallback |
| GITHUB_REF_NAME gated on GITHUB_REF_TYPE == "tag"; branch-name arm skipped on non-tag CI runs | ADR-064 v1.6 D2 (F-VID-P1-CRIT-001) | EC-002 describes the failure mode; tag-guard prevents it |
| GITHUB_REF_NAME on release.yml is the tag; on workflow_dispatch runs is the branch | ADR-064 §Rationale NIT-2 | CI topology invariant; tag-guard makes this safe |
| No `vergen` crate added | ADR-064 §Rationale (rejected vergen: shallow-clone incompatibility) | Cargo.lock diff shows no vergen entry |
| build.rs must emit rerun-if-env-changed for all four trigger vars | ADR-064 v1.6 D2 | AC-002 grep check (4 lines: PRISM_BUILD_VERSION, GITHUB_REF_NAME, GITHUB_REF_TYPE, GITHUB_REF) |

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
| EC-002 | `GITHUB_REF_NAME` is set to a branch name (e.g., `develop`) on a non-tag CI run | `build.rs` gates on `GITHUB_REF_TYPE == "tag"` (or `GITHUB_REF` starts-with `refs/tags/`). On push/pull_request events, `GITHUB_REF_TYPE` is `"branch"` so `is_tag_build` is `false` and the `GITHUB_REF_NAME` arm is skipped entirely — `PRISM_VERSION` falls through to `CARGO_PKG_VERSION` (`1.0.0-dev`). Branch names are NEVER baked into the binary (F-VID-P1-CRIT-001 fix, ADR-064 v1.5/v1.6) |
| EC-003 | `GITHUB_REF_NAME` is `v1.0.0-beta.1` (leading `v`) | `strip_prefix('v')` → `1.0.0-beta.1`; correct per ADR-064 v1.6 D2. `strip_prefix` (single-strip) is used instead of `trim_start_matches` (F-VID-P1-LOW-001 — the latter would strip ALL leading v's) |
| EC-004 | All three env vars absent (local dev) | Falls through to `CARGO_PKG_VERSION` = `1.0.0-dev`; correct local identity |
| EC-005 | `prism-spec-engine/pipeline.rs` user-agent after this story | Still reads `CARGO_PKG_VERSION` from prism-spec-engine's own Cargo.toml; the crate version, not the product version. Documented out-of-scope per ADR-064 D2 |

---

## Changelog

| Version | Date | Author | Change |
|---------|------|--------|--------|
| 1.1 | 2026-09-05 | story-writer | Sync Task 5 / AC-002 / RG-002 to ADR-064 v1.6: tag-guard (GITHUB_REF_TYPE=="tag"), strip_prefix single-v strip, 4 rerun-if-env-changed lines (PRISM_BUILD_VERSION, GITHUB_REF_NAME, GITHUB_REF_TYPE, GITHUB_REF), empty-string filter on all arms; AC-001 updated to name is_tag_build; EC-002/EC-003 corrected; Architecture Compliance Rules and risk_mitigations updated; closes pass-5 HIGH-1 stale-body defect |
| 1.0 | 2026-09-05 | story-writer | Initial — ADR-064 D2 materialization |
