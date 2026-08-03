---
document_type: story
story_id: S-DEMO-CI-E2E-001
title: "ci: Scheduled E2E Red Gate CI Workflow — DTU Demo Server + Release Binary + Ignored Suite"
wave: 5
epic_id: E-DEMO
priority: P1
status: superseded
# Superseded by S-DEMO-002 PR #171 (human decision 2026-06-03).
# The core scope — a dedicated GitHub Actions e2e job that builds release binaries
# (prism-bin + prism-dtu-demo-server), launches the DTU demo server, and runs
# `cargo nextest run -p prism-bin --profile e2e --run-ignored ignored-only` on
# pull_request + push — was absorbed into S-DEMO-002 as Task 25.
# This story is CLOSED. Do not dispatch. See S-DEMO-002 v2.0 for the delivered spec.
replaced_by: S-DEMO-002
version: "1.2"
level: "L4"
producer: story-writer
timestamp: "2026-06-02T00:00:00Z"
modified: "2026-06-03T12:00:00Z"
tdd_mode: facade
# tdd_mode: facade — this story produces GitHub Actions YAML workflow files and a nextest
# profile entry, not application logic. There is no application code with `todo!()` stubs;
# the deliverable is CI configuration that is structurally correct on creation. Mutation
# testing at the wave gate replaces the Red Gate density check as the quality gate per the
# story-writer facade-mode criteria (DTU API clones, mock servers, structural fakes,
# config parsing wrappers — CI config authoring is analogous).
subsystems: [SS-22]
# Subsystem anchor justifications:
#   SS-22 (Binary Entrypoint) owns prism-bin launch, the `--profile e2e` nextest expansion,
#     and the release build of prism-bin + prism-dtu-demo-server. This story's CI workflow
#     gates on the prism-bin E2E test suite, which lives under SS-22's scope.
#   No additional subsystem anchor is required: the CI workflow itself lives in .github/ (infra)
#     and does not modify application subsystem code.
crates_touched: [prism-bin, prism-dtu-demo-server]
# crates_touched notes:
#   prism-bin: the ignored E2E suite (crates/prism-bin/tests/e2e_smoke.rs) is what the new
#     CI workflow exercises. No source changes to prism-bin itself.
#   prism-dtu-demo-server: the CI workflow builds and launches this binary. No source changes.
target_module: devops
capabilities: [CAP-034]
behavioral_contracts: []
# BC authorship pending. Advisory traces only (not frontmatter BCs):
#   BC-2.22.001 (Boot Orchestration) governs the deterministic startup the E2E tests exercise.
#   BC-2.10.010 (Graceful Shutdown) governs the SIGTERM path exercised by AC-010 in S-DEMO-002.
#   BC-3.2.001 (Per-Org Sensor Data Isolation) governs the cross-org Red Gate (AC-012).
# These BCs are exercised BY the E2E test suite that this story gates. They are NOT
# authored here; they are cited to explain the fitness criterion of the new CI job.
verification_properties: []
depends_on:
  - S-DEMO-002  # The E2E Red Gate test suite (e2e_smoke.rs) must be authored and
                # merged before this CI workflow can run it. This is a hard gate:
                # the CI job fails with zero test results if e2e_smoke.rs does not exist.
  - S-6.20      # prism-dtu-demo-server (merged): the demo server binary that the E2E
                # CI workflow launches must be buildable via `cargo build --release
                # -p prism-dtu-demo-server`. S-6.20 is the story that delivered it.
blocks: []
# blocks was: [S-DEMO-003] — S-DEMO-003 dependency is now carried by S-DEMO-002
# (which delivers the e2e CI job). S-DEMO-003 should not depend on this superseded story.
points: 5
# Points justification:
#   - GitHub Actions workflow file (.github/workflows/e2e.yml) with release build, DTU
#     launch, nextest --profile e2e, artifact upload on failure: ~2.5 pts
#   - nextest [profile.e2e] entry in .config/nextest.toml (run-ignored = all, slow-timeout):
#     0.5 pts (already partially specified in S-DEMO-002 Task 14; this story authors it)
#   - OS keyring bootstrap step in CI (macOS keyring emulation via env-var shim): 1 pt
#   - DTU process lifecycle in CI (start, health-poll, cleanup): 0.5 pts
#   - Documentation: inline workflow comments + OBS-2 lineage comment: 0.5 pts
#   Total: 5 points (~1 day)
estimated_days: 1
risk: MEDIUM
# Risk justification:
#   The OS keyring is macOS-native (Security framework). GitHub-hosted macOS runners
#   (macos-latest) support the Security framework, but environment setup for test-helpers
#   keyring bootstrap may require additional env vars or mocking. The implementer MUST
#   verify that prism-credentials test-helpers feature writes credentials in a way
#   compatible with the runner's keyring, or switch to env-var credential injection
#   for CI only (coordinate with AD-017 model: credentials must not transit AI context,
#   but test credentials can be env-injected via GitHub Actions secrets).
#
#   E2E tests are flake-prone (subprocess timing, port binding). The nextest profile
#   should include `retries = 1` to absorb single-run transient failures while still
#   failing on consistent regressions.
acceptance_criteria_count: 6
red_gate_tests: 0
# red_gate_tests: 0 — this is a CI configuration story (tdd_mode: facade). There are
# no Rust test functions written as part of this story. The quality gate is: the new
# CI workflow runs and the existing E2E Red Gate tests (authored in S-DEMO-002) pass.
estimated_passes: "1-2 LOCAL adversary passes (YAML linting + CI behavior review)"
holdout_scenarios: []
assumption_validations: []
risk_mitigations:
  - "OS keyring in CI: use prism-credentials test-helpers env-var shim (or GitHub Actions
    secret injection) for CI-only credential bootstrap; document the approach in the
    workflow YAML comments."
  - "Flake mitigation: add `retries = 1` to [profile.e2e] in nextest.toml; if the test
    fails twice consecutively it is a real regression, not a timing flake."
  - "DTU cleanup on CI failure: use `if: always()` on the DTU teardown step so the
    process is killed even if tests fail; prevents zombie DTU processes consuming runner
    ports on subsequent jobs."
  - "Release build caching: use Swatinem/rust-cache with shared-key='e2e-release' to
    avoid rebuilding prism-bin + prism-dtu-demo-server from scratch on every run."
  - "Windows CI skip: E2E tests use SIGTERM (Unix-only); the e2e workflow MUST NOT include
    a Windows runner. Only macos-latest (the demo target platform) is required."
inputs:
  - ".github/workflows/ci.yml"
  - ".config/nextest.toml"
  - "crates/prism-bin/tests/e2e_smoke.rs"
  - ".factory/stories/S-DEMO-002-e2e-subprocess-smoke-test-all-sensors.md"
  - ".factory/stories/S-6.20-dtu-demo-server.md"
input-hash: null
traces_to: []
cycle: "v1.0.0-brownfield"
phase: 3
originating_finding: "S-DEMO-002 LOCAL adversarial cascade OBS-2 (process-gap)"
# originating_finding: OBS-2 surfaced that the #[ignore]'d E2E Red Gate tests in
# crates/prism-bin/tests/e2e_smoke.rs are never executed by `just check` or any CI job.
# The dot-vs-underscore table-name drift (HIGH-1 in the same cascade) made AC-012 un-passable
# and was caught only by fresh-context adversarial review. This story closes the structural
# gap: CI must execute the ignored suite against a live DTU so regressions are caught
# before human review, not discovered during it.
---

# S-DEMO-CI-E2E-001 v1.2 — ci: Scheduled E2E Red Gate CI Workflow [SUPERSEDED]

**Story ID:** S-DEMO-CI-E2E-001
**Status:** SUPERSEDED — absorbed into S-DEMO-002 PR #171
**Version:** v1.2
**Wave:** 5
**Priority:** P1
**Points:** 5 (absorbed; see S-DEMO-002 v2.0 +2 pts)

---

## Authority

This story is SUPERSEDED — scope absorbed into S-DEMO-002 Task 25 (PR #171, 2026-06-03).
See §SUPERSESSION NOTICE for the full disposition. Do not dispatch.

The former governing framework for CI workflow authoring: `ci.yml` action SHA pins are the
source of truth for all `uses:` entries in this story's Architecture Compliance Rules. No ADR
governs this story's CI configuration scope. BC-2.22.001 §Postconditions governs the E2E test
suite that the former workflow would have exercised, but that authoring is now owned by
S-DEMO-002.

---

## SUPERSESSION NOTICE

**Decision date:** 2026-06-03
**Decision authority:** Human (Joshua Magady)
**Absorbed into:** S-DEMO-002 PR #171 (Task 25)
**Replaced by:** S-DEMO-002 v2.0

The e2e CI job (build release binaries, launch DTU, run
`cargo nextest run -p prism-bin --profile e2e --run-ignored ignored-only` on
`pull_request` + `push` to `develop`) was added directly into S-DEMO-002's scope.
This story is fully superseded. Do not dispatch.

**Rationale for supersession (not narrowing):**
- The primary structural gap (OBS-2: ignored suite never runs in CI) is fully closed by
  S-DEMO-002's PR+push job. The PR+push trigger is the regression-before-merge gate, which
  was the entire motivation for this story.
- The residual scope (daily `schedule:` cron trigger) was itself unresolved at story creation
  (Open Question 2: "confirm schedule frequency" — not yet answered). A daily drift-detection
  cron is a 0.5-point enhancement to `e2e.yml` that devops-engineer can add at any time;
  it does not constitute a standalone 5-point story.
- This story was `status: draft` and `behavioral_contracts: []`, never having cleared the
  S-7.01 Spec-First Gate. There is no downstream dependency that counted on its delivery
  in isolation (the `blocks: S-DEMO-003` relationship is now satisfied by S-DEMO-002 itself).
- Absorbing into S-DEMO-002 keeps the CI workflow and the e2e test spec in a single atomic
  PR, ensuring both land together and the job is immediately testable on the branch that
  introduces the test suite.

**If a daily scheduled run is desired in future:**
Add a `schedule: - cron: '0 4 * * *'` trigger to `.github/workflows/e2e.yml` in a
maintenance PR — no new story required. Reference this supersession notice as context.

---

---

## Origin

Process-gap OBS-2 surfaced by the S-DEMO-002 LOCAL adversarial cascade (2026-06-02).

The gap: `crates/prism-bin/tests/e2e_smoke.rs` contains `#[ignore]`'d E2E Red Gate tests
(tagged `// E2E-001:`) that require a live DTU demo server (`prism-dtu-demo-server`) and
release binaries. These tests are never executed by `just check` or any standard CI job
(`cargo nextest run --workspace --profile ci`), so they can silently rot out of sync with
production code.

Concretely: a dot-vs-underscore table-name drift (HIGH-1 in the S-DEMO-002 LOCAL cascade)
made the AC-012 Red Gate un-passable. This was caught only by fresh-context adversarial
review, not by CI. The `#[ignore]` annotation preserves the test for non-standard execution
but no non-standard execution was scheduled. This story closes the structural gap.

---

## Narrative

As a Prism platform engineer, I want a dedicated CI workflow that (1) builds release
binaries for `prism-bin` and `prism-dtu-demo-server`, (2) starts the DTU demo server, (3)
runs `cargo nextest run -p prism-bin --profile e2e` (which un-ignores the `// E2E-001:`
tagged suite), and (4) fails the build on any E2E regression, so that the `#[ignore]`'d
E2E Red Gate tests cannot silently drift from the resolver contracts they protect, and
regressions are caught automatically before human review.

---

## Behavioral Contracts

Advisory references only. This story has no authored BCs yet (pending PO, S-7.01 gate).

| BC ID (advisory) | Title | Relevance |
|-----------------|-------|-----------|
| BC-2.22.001 | Boot Orchestration — Sequencing, Exit-Code Map, and Pre-Traffic Gate | The E2E tests exercise boot orchestration; this CI job gates on them passing |
| BC-2.10.010 | Graceful Shutdown on SIGTERM/SIGINT | SIGTERM teardown is exercised by AC-008 of S-DEMO-002; this CI job validates it |
| BC-3.2.001 | Per-Org Sensor Data Isolation via Composite HashMap Key | Cross-org Red Gate (S-DEMO-002 AC-012) is exercised by this CI job |
| BC-2.11.005 | Ephemeral Materialization | The four-sensor query assertions in S-DEMO-002 AC-003..006 exercise this BC |

_These BCs are exercised by the E2E test suite that this story's CI workflow gates. They are_
_NOT authored in this story. Cited here to explain the fitness criterion of the new job._

---

## Acceptance Criteria

### AC-001: A `.github/workflows/e2e.yml` CI workflow file exists and is syntactically valid
Given: The story is implemented.
When: GitHub Actions parses `.github/workflows/e2e.yml`.
Then: The workflow file is syntactically valid YAML; GitHub Actions accepts it without a
"Invalid workflow file" error; the workflow defines at least one job named `e2e-smoke`.
(process/infra-scoped — no BC clause; validates OBS-2 gap closure)

### AC-002: The workflow builds `prism-bin` and `prism-dtu-demo-server` in release mode
Given: The `e2e-smoke` job starts on a macOS runner.
When: The `cargo build --release -p prism-bin -p prism-dtu-demo-server` step runs.
Then: Both binaries are present in `target/release/` after the step; the step exits with
code 0; build caching via `Swatinem/rust-cache` (shared-key `e2e-release`) is configured
to avoid cold-build on every run.
(process/infra-scoped — no BC clause; prerequisite for the test suite to run)

### AC-003: The workflow launches the DTU demo server and polls for the ready signal
Given: Release binaries are built.
When: The DTU launch step runs:
  `./target/release/prism-dtu-demo-server start --config crates/prism-bin/fixtures/e2e-demo/demo.toml`
Then: The step polls for `.prism-dtu-demo-server.urls.json` with a 30-second timeout and
exponential backoff before advancing; if the file does not appear within 30 seconds, the
step fails with a clear message: "DTU server did not write urls.json within 30s"; the DTU
process PID is captured for cleanup in a `if: always()` teardown step.
(process/infra-scoped — no BC clause; mirrors S-DEMO-002 AC-001 in CI)

### AC-004: The workflow executes the ignored E2E suite via `--profile e2e`
Given: The DTU server is running and the ready signal has been received.
When: The nextest step runs:
  `cargo nextest run -p prism-bin --profile e2e`
Then: All `#[ignore]`'d tests tagged `// E2E-001:` in `crates/prism-bin/tests/e2e_smoke.rs`
are executed (un-ignored by the e2e profile); any test failure causes the CI job to exit
non-zero and the PR or scheduled run to fail; a JUnit XML artifact is uploaded on failure
via `actions/upload-artifact` for post-mortem diagnosis.
(process/infra-scoped — no BC clause; this is the direct OBS-2 gap closure)

### AC-005: The `.config/nextest.toml` `[profile.e2e]` entry exists and is correct
Given: The story is implemented.
When: `cat .config/nextest.toml` is inspected.
Then: A `[profile.e2e]` section exists with:
  - `run-ignored = "all"` (executes ignored tests)
  - `slow-timeout = { period = "120s" }` (subprocess tests are slower than unit tests)
  - `retries = 1` (absorb single-run transient flakes; double-failure is a real regression)
  - `failure-output = "immediate-final"` (consistent with the existing `[profile.ci]`)
(process/infra-scoped — no BC clause; required for AC-004 to work; S-DEMO-002 Task 14
specified this entry; this story authors and owns it)

### AC-006: The workflow includes a DTU teardown step that runs unconditionally
Given: The `e2e-smoke` job runs (regardless of test pass/fail outcome).
When: The teardown step is reached.
Then: The DTU demo server process is killed (`kill <pid>` or equivalent); the step runs
even when prior steps failed (`if: always()` condition); no zombie DTU processes remain
after the job completes; the workflow includes a code comment:
  `# OBS-2: DTU teardown must always run — zombie processes consume runner ports on retries.`
(process/infra-scoped — no BC clause; prevents port-conflict flake on CI runner re-use)

---

## Architecture Compliance Rules

| Rule | Source | Enforcement |
|------|--------|-------------|
| Workflow YAML uses pinned action SHAs (not tag aliases) | ci.yml convention — all existing actions use `@<sha>` pins | CI linter will warn on unpinned actions; all `uses:` entries MUST use full SHA pins matching the project's existing action version table |
| `e2e.yml` MUST NOT run on Windows runners | S-DEMO-002 AC-010 / architecture compliance rule | SIGTERM-based subprocess teardown is Unix-only; add an explicit `runs-on: macos-latest` and no Windows matrix |
| No real API credentials in workflow environment | AD-017 AI-opaque credential model | Test credentials are injected via `prism-credentials` test-helpers env-var shim or GitHub Secrets (dummy values only); plaintext credential values must not appear in YAML |
| Release binary required (not debug) | S-DEMO-002 Architecture Compliance rule | `cargo build --release`; debug binary is too slow for the 30-second DTU ready timeout |
| Workflow triggers: `push` to `develop` + `pull_request` + `schedule` | OBS-2 gap requirement | The gap was "never scheduled"; the workflow MUST include a `schedule:` trigger (`cron: '0 4 * * *'` — daily at 04:00 UTC) in addition to PR and push triggers |

---

## Library & Framework Requirements

| Tool / Action | Version / Pin | Purpose |
|---------------|--------------|---------|
| `actions/checkout` | `de0fac2e4500dabe0009e67214ff5f5447ce83dd` (v6.0.2) — match ci.yml | Checkout repository |
| `dtolnay/rust-toolchain` | `29eef336d9b2848a0b548edc03f92a220660cdb8` (stable) — match ci.yml | Install Rust stable toolchain |
| `arduino/setup-protoc` | `c65c819552d16ad3c9b72d9dfd5ba5237b9c906b` (v3.0.0) — match ci.yml | protoc for prost-build (prism-ocsf) |
| `Swatinem/rust-cache` | `c19371144df3bb44fab255c43d04cbc2ab54d1c4` (v2.9.1) — match ci.yml | Incremental build caching |
| `taiki-e/install-action` | `cf525cb33f51aca27cd6fa02034117ab963ff9f1` (v2.75.22) — match ci.yml | Prebuilt cargo-nextest |
| `actions/upload-artifact` | latest pinned SHA (check GitHub Marketplace for current v4 SHA) | Upload JUnit XML on failure |
| `cargo-nextest` | workspace version (installed via `taiki-e/install-action`) | Execute `--profile e2e` suite |

**Version source:** All action SHA pins are read from the existing `ci.yml` to ensure consistency.
The implementer MUST copy SHA values verbatim from `ci.yml` — do NOT invent or refresh SHA pins
independently.

---

## File Structure Requirements

| File | Action | Purpose |
|------|--------|---------|
| `.github/workflows/e2e.yml` | CREATE | New dedicated E2E CI workflow |
| `.config/nextest.toml` | MODIFY — add `[profile.e2e]` section | Un-ignore E2E-001 tagged tests; configure slow-timeout + retries |

No application source files are modified by this story. All changes are CI configuration
and nextest profile configuration.

---

## Tasks

1. **Read** `ci.yml` in full — extract all action SHA pins; collect the exact
   `Swatinem/rust-cache`, `taiki-e/install-action`, `dtolnay/rust-toolchain`,
   `arduino/setup-protoc`, and `actions/checkout` SHA values to reuse verbatim.

2. **Read** `.config/nextest.toml` in full — understand the existing `[profile.ci]` shape
   before writing the `[profile.e2e]` addition.

3. **Read** `crates/prism-bin/tests/e2e_smoke.rs` (or confirm it exists after S-DEMO-002
   merges) — verify the `// E2E-001:` comment annotation is present on the `#[ignore]`
   attribute so `--profile e2e` with `run-ignored = "all"` will target the right tests.

4. **Read** `crates/prism-credentials/src/lib.rs` — determine the correct mechanism for
   bootstrapping dummy test credentials on a macOS CI runner without a real keyring entry.
   If `test-helpers` feature provides an env-var shim, use it. Otherwise plan a
   `security add-generic-password` step in the workflow (macOS Security framework CLI).

5. **Add** `[profile.e2e]` to `.config/nextest.toml` with the values specified in AC-005.
   Insert after the `[profile.ci]` section; add a comment:
   `# E2E profile: un-ignores E2E-001 tagged tests; requires DTU server running (S-DEMO-CI-E2E-001)`

6. **Write** `.github/workflows/e2e.yml` with the following structure:
   ```
   name: E2E Smoke Tests
   on:
     push:
       branches: [develop]
     pull_request:
     schedule:
       - cron: '0 4 * * *'   # daily 04:00 UTC
   concurrency:
     group: e2e-${{ github.ref }}
     cancel-in-progress: ${{ github.event_name == 'pull_request' }}
   jobs:
     e2e-smoke:
       runs-on: macos-latest
       timeout-minutes: 30
       steps:
         - checkout
         - rust-toolchain (stable, pinned SHA)
         - setup-protoc (pinned SHA)
         - rust-cache (shared-key: e2e-release)
         - install nextest (pinned SHA)
         - cargo build --release -p prism-bin -p prism-dtu-demo-server
         - credential bootstrap step (env-var shim or macOS keyring CLI)
         - launch DTU + poll urls.json (30s timeout)
         - cargo nextest run -p prism-bin --profile e2e
         - upload JUnit artifact on failure
         - DTU teardown (if: always())
   ```

7. **Add** workflow comment at the top of `e2e.yml`:
   ```
   # E2E Red Gate workflow — closes OBS-2 process-gap from S-DEMO-002 LOCAL cascade (2026-06-02).
   # Runs the #[ignore]'d E2E suite (E2E-001) against a live DTU demo server + release binary.
   # Standard CI (ci.yml) never runs this suite; this workflow is the only gate that
   # catches E2E contract drift between CI pushes.
   ```

8. **Run** `yamllint .github/workflows/e2e.yml` (or equivalent) to validate YAML syntax
   before committing.

9. **Run** `just check` — final pre-push gate (lint + format; no E2E tests run).

10. **Verify** via a manual workflow dispatch on a feature branch that the new job:
    - builds successfully
    - launches the DTU
    - runs the E2E suite (even if tests fail for unrelated reasons — confirm the suite is
      actually un-ignored and executing, not silently skipped)

---

## Previous Story Intelligence

- **S-DEMO-002** (depends_on): This story depends on `crates/prism-bin/tests/e2e_smoke.rs`
  existing with `#[ignore]`'d tests tagged `// E2E-001:`. S-DEMO-002 Task 14 specified
  that `[profile.e2e]` should be added to `.config/nextest.toml`. This story **owns** that
  entry (the devops/CI story is the correct author of nextest profile configuration, not
  the test-writing story). If S-DEMO-002 has not merged when this story is dispatched, Task 3
  (read e2e_smoke.rs) will find the file absent — the implementer must coordinate merge order.
  S-DEMO-CI-E2E-001 MUST be dispatched after S-DEMO-002 merges.

- **S-0.01** (merged): Delivered the base CI pipeline (`ci.yml`). This story inherits all
  action SHA pins from S-0.01's delivered artifact. The `[profile.ci]` nextest configuration
  was also delivered by S-0.01 (see `.config/nextest.toml`); this story adds `[profile.e2e]`
  alongside it.

- **W3-FIX-CI-001** (merged): Wall-clock optimization story that tuned `ci.yml`. The
  Swatinem/rust-cache and mold linker patterns from that story inform the e2e workflow
  design (use `Swatinem/rust-cache` with a unique `shared-key`; no mold on macOS).

- **S-6.20** (merged): Delivered `prism-dtu-demo-server`. The demo server binary is already
  available; this story only needs to build and launch it.

---

## Open Questions

1. **Credential bootstrap in CI**: macOS GitHub-hosted runners have the Security framework
   available, but the keyring is per-user and may not be pre-populated. The safest CI
   approach is: (a) if `prism-credentials` `test-helpers` feature accepts env vars
   (`PRISM_CRED_<SENSOR>_CLIENT_ID`, etc.) as a shim, use that; (b) otherwise, use
   `security add-generic-password` CLI steps. The implementer MUST read
   `crates/prism-credentials/src/lib.rs` and confirm the correct mechanism before writing
   the workflow (Task 4).

2. **Schedule frequency**: `0 4 * * *` (daily at 04:00 UTC) is proposed. If the E2E suite
   takes ~5 minutes, daily is appropriate. If the team wants faster drift detection,
   `0 */6 * * *` (every 6 hours) is an alternative. Architect/product-owner to confirm.

3. **PR gate vs scheduled-only**: Should the E2E workflow run on EVERY pull request (which
   adds ~5-10 min to PR latency) or only on push to develop + scheduled? The current spec
   says both. If PR latency is a concern, the `pull_request` trigger can be scoped to
   paths that affect E2E behaviour: `paths: ['crates/prism-bin/**', 'crates/prism-dtu-*/**']`.

---

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `e2e_smoke.rs` does not yet exist (S-DEMO-002 not merged) | `cargo nextest run -p prism-bin --profile e2e` exits 0 with zero tests run; CI job passes but emits a warning comment; implementer must coordinate merge order |
| EC-002 | DTU server fails to start within 30s (port conflict on shared runner) | CI step fails with: "DTU server did not write urls.json within 30s"; job fails; DTU teardown runs via `if: always()` |
| EC-003 | E2E test flake (single transient failure) | `retries = 1` in `[profile.e2e]` absorbs single transient failures; double-failure surfaces as a real regression |
| EC-004 | Scheduled run on a day when develop is broken for unrelated reasons | Job fails; GH Actions workflow failure email/notification goes to repository watchers; does NOT block PRs (separate workflow) |
| EC-005 | `prism-bin` E2E tests require credentials not available in CI environment | Credential bootstrap step (Task 4) must be implemented before the workflow can pass; if the step is missing, tests fail with keyring errors, which is an implementation defect in this story, not EC behaviour |

---

## Token Budget Estimate

| Context source | Estimated tokens |
|----------------|-----------------|
| This story spec | ~3,500 |
| `ci.yml` (full read for SHA pins) | ~4,000 |
| `.config/nextest.toml` (current content) | ~600 |
| `crates/prism-bin/tests/e2e_smoke.rs` (S-DEMO-002 deliverable; scan for E2E-001 annotations) | ~2,500 |
| `crates/prism-credentials/src/lib.rs` (credential bootstrap mechanism) | ~2,000 |
| S-DEMO-002 story (dependency context; architecture compliance rules) | ~4,500 |
| `.github/workflows/*.yml` sibling files (pattern reference) | ~2,000 |
| **Total estimate** | **~19,100 tokens (~7% of 256K context)** |

Well within budget. This is the smallest story in the E-DEMO arc.

---

## Forbidden Dependencies

| Forbidden | Reason |
|-----------|--------|
| Real API credentials or live sensor endpoints in workflow environment | AD-017 AI-opaque credential model; test credentials only |
| `runs-on: windows-latest` in `e2e.yml` | SIGTERM-based subprocess teardown is Unix-only (S-DEMO-002 Architecture Compliance rule) |
| Unpinned action tags (e.g., `@v4`, `@main`) | ci.yml convention; all `uses:` entries require full commit SHA pins |
| Hard-coded port numbers in workflow steps | DTU server binds to ephemeral port; port read from urls.json; do not assume 8080 or any fixed port |

---

## References

- S-DEMO-002 LOCAL adversarial cascade OBS-2 (originating process-gap) — 2026-06-02
- S-DEMO-002 v1.6 — `crates/prism-bin/tests/e2e_smoke.rs` + `[profile.e2e]` requirement
- ci.yml — action SHA pins (source of truth for all `uses:` entries in `e2e.yml`)
- `.config/nextest.toml` — `[profile.ci]` as structural template for `[profile.e2e]`
- SID-1 (CLAUDE.md Standing Implementer Discipline) — `#[ignore]` tests require blocking
  dependency citations; un-gated execution in CI is the correct resolution when the
  dependency (DTU) is available

---

## Changelog

| Version | Date | Author | Notes |
|---------|------|--------|-------|
| 1.2 | 2026-08-02 | story-writer | Added ## Authority section (DRIFT-STORY-AUTHORITY-ABSENT-CORPUS-001 Round 6, D-2084). Synced stale `**Version:**` pseudo-field and H1 title from v1.1 to v1.2 to match frontmatter (TD-VSDD-060 sibling-sweep correction, orchestrator-authorized). |
| 1.1 | 2026-06-03 | product-owner | SUPERSEDED — absorbed into S-DEMO-002 PR #171 per human decision 2026-06-03. Status set to `superseded`; `replaced_by: S-DEMO-002`; `blocks: []` (S-DEMO-003 dependency transferred to S-DEMO-002). Supersession rationale documented inline (PR+push gate covers the OBS-2 gap; daily schedule residual is a 0.5-pt enhancement, not a standalone story; story was draft/BC-pending, never ready). |
| 1.0 | 2026-06-02 | story-writer | Initial draft — OBS-2 process-gap closure from S-DEMO-002 LOCAL cascade. Process/infra story; tdd_mode: facade; status: draft pending BC authorship (S-7.01 gate). |
