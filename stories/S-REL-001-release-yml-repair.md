---
document_type: story
story_id: S-REL-001
title: "devops: release.yml repair — remove dead jobs (DEF-REL-001 through DEF-REL-004) + v*-rc.* prerelease handling + Linux cross-compile setup + install-script upload"
wave: F-A
epic_id: E-REL
priority: P0
status: draft
version: "0.2"
level: "L4"
producer: story-writer
timestamp: "2026-07-19T00:00:00Z"
tdd_mode: strict
subsystems: []
# Subsystem anchor justification:
#   This story modifies .github/workflows/release.yml only — pure CI/CD infrastructure.
#   No ARCH-INDEX subsystem owns GitHub Actions workflow files; subsystems: [] is correct
#   per S-0.01 and S-MAINT-CI-DISK-EXHAUSTION-001 precedent.
crates_touched: [devops]
target_module: devops
capabilities: []
behavioral_contracts: []
# BC status: N/A — this is a CI/CD infrastructure story. No subsystem behavioral contract
# governs GitHub Actions workflow YAML. Conforming per W3-FIX-CI-001 precedent.
verification_properties: []
depends_on: []
blocks: [S-REL-003, S-REL-004]
# Dependency anchor justifications:
#   blocks S-REL-003: install.sh/install.ps1 must know the correct GH Releases URL pattern
#     (download URL format after release.yml repair); the prerelease flag determines the
#     release URL structure consumers fetch from.
#   blocks S-REL-004: demo-bundle packaging job in release.yml depends on a repaired
#     workflow base (removed dead jobs frees up job-name namespace and output variables).
points: 3
estimated_days: 2
risk: LOW
# Risk justification: All four defects affect jobs that already fail unconditionally
# (DEF-REL-002: missing nuspec; DEF-REL-003: wrong tap org 404; DEF-REL-004: publish=false
# rejection). Removing dead jobs can only improve CI — no regression risk. DEF-REL-001
# removal eliminates non-deterministic matrix output behavior. Prerelease flag addition is
# additive-only. Risk is LOW per delta-analysis §8.
acceptance_criteria_count: 12
red_gate_tests: 4
estimated_passes: "1-2 LOCAL adversary passes"
holdout_scenarios: []
assumption_validations: []
risk_mitigations:
  - "Actionlint is Go, not Rust: do NOT run `cargo install actionlint`. Install via
    `brew install actionlint` locally or the official download-actionlint.bash script in
    CI (research U4 confirmed: no crates.io package named actionlint exists). Invoke
    as bare `actionlint` with no arguments to lint all workflows."
  - "Prerelease flag -- gh does NOT auto-detect: `gh release create v1.0.0-rc.1` does NOT
    auto-set --prerelease (research U3 confirmed). Derive is_prerelease from the tag
    ([[ \"$TAG\" == *-* ]]) and pass via bash array pattern (args+=(--prerelease)) so
    that no empty positional arg is sent when the flag is absent."
  - "DEF-REL-001 output variable: removing the outputs block from build-release also
    requires removing all `needs.build-release.outputs.binary_exists` references in
    downstream job `if:` conditions. Run a grep after editing to confirm zero residual
    references."
  - "TD-VSDD-060 sweep: grep for 'binary_exists', 'check_binary', 'chocolatey', 'homebrew-update',
    'crates-io' in release.yml after edits to confirm zero residual references."
  - "Linux cross-compile setup (U2): build-release matrix job needs `musl-tools pkg-config`
    on ubuntu runners for x86_64-unknown-linux-musl. libdbus-1-dev is INCONCLUSIVE (depends
    on keyring crate backend — verify against Cargo.lock at implementation time). Do NOT
    add libssl-dev (ADR-050 mandates rustls-tls). Verify BEFORE finalizing: check the
    pinned `keyring` version and enabled features in prism workspace Cargo.toml/Cargo.lock."
  - "action pin SHAs: all `uses:` entries must be pinned to an immutable commit SHA
    (repo convention; research U20). Resolve SHAs at implementation time via
    `git ls-remote https://github.com/<owner>/<repo> refs/tags/<tag>`.
    Canonical versions from research: checkout@v6.0.2; upload-artifact@v7; attest-build-provenance@v4.1.1
    (NOT v4.1.0 which was stale); macos-13 is RETIRED -- use macos-15-intel for Intel macOS
    builds and note its Aug 2027 EOL. upload@v7 + download@v8 same-run interop is INCONCLUSIVE --
    add a smoke-test verification task."
  - "build-release builds prism-bin + prism-dtu-demo-server together (U13): the matrix
    cargo invocation uses `-p prism-bin -p prism-dtu-demo-server` in one call. The
    prism-dtu-demo-server binary is tar-wrapped (tar czf) before upload-artifact to preserve
    the +x bit (research U18: upload-artifact ZIP format strips executable bits). The build-demo-bundle
    job downloads+untars rather than re-building."
  - "install.sh and install.ps1 uploaded as release assets by publish-release job (U26):
    the publish-release step must include `scripts/install.sh` and `scripts/install.ps1` in
    the `gh release upload` glob. Version passed to install.ps1 consumers via
    `$env:PRISM_INSTALL_VERSION` env var (research U8: irm|iex piping cannot carry positional args)."
  - "fork-tag dry-run (U2 finding): before cutting the real v1.0.0-rc.1 tag, gate RC-1 on
    a successful full workflow green run on a fork tag push. Add this as a mandatory
    verification task in the story's task list."
inputs:
  - ".github/workflows/release.yml"
  - ".factory/planning/feature-release-engineering/delta-analysis.md"
  - ".factory/research/release-engineering-uncertainties-2026.md"
input-hash: "77224a8"
traces_to: []
cycle: "v1.0.0-release-engineering"
phase: "F3"
---

# S-REL-001 — devops: release.yml repair

**Story ID:** S-REL-001
**Status:** draft
**Version:** v0.2
**Wave:** F-A
**Priority:** P0
**Points:** 3

---

## Origin

Five defects (DEF-REL-001 through DEF-REL-005) were identified in `.github/workflows/release.yml`
by the F1 delta analysis (`delta-analysis.md` §3). All four RC-1-blocking defects (DEF-REL-001
through DEF-REL-004) must be fixed before v1.0.0-rc.1 can be tagged. DEF-REL-005 (demo bundle
not packaged) is addressed by S-REL-004.

Additionally: (a) the current workflow does not set `--prerelease` on RC tags — required so that
consumers can distinguish RC releases from GA; (b) the build-release matrix must build both
`prism-bin` and `prism-dtu-demo-server` together (U13, architect adjudication); (c) Linux
cross-compile requires a musl-tools setup step (U2); (d) install.sh/install.ps1 are uploaded
as release assets by the publish-release job (U26).

---

## Narrative

As a release engineer, I want the GitHub Actions release workflow to run cleanly on a
`v1.0.0-rc.1` tag push, so that the 5-platform binary archives, checksums, OIDC attestation,
and install scripts are created and uploaded without errors and the release is correctly
flagged as a prerelease.

---

## Behavioral Contracts

This story has no subsystem BCs — it is CI/CD infrastructure. Compliance is verified by
observing workflow execution on a test tag push.

| Architecture Source | Clause |
|--------------------|--------|
| `delta-analysis.md` §3 (DEF-REL-001) | Remove binary_exists guard and matrix output non-determinism |
| `delta-analysis.md` §3 (DEF-REL-002) | Remove chocolatey-publish job (packaging/ does not exist) |
| `delta-analysis.md` §3 (DEF-REL-003) | Disable homebrew-update job (1898co tap org does not exist) |
| `delta-analysis.md` §3 (DEF-REL-004) | Remove crates-io-publish job (all crates have publish = false) |
| `delta-analysis.md` §2.1 (prerelease) | Add --prerelease flag for v*-rc.* tags |
| Architect U13 adjudication | build-release builds -p prism-bin -p prism-dtu-demo-server; demo-server tar-wrapped |
| Architect U26 adjudication | install.sh/.ps1 uploaded as release assets by publish-release |
| Research U2 (release-engineering-uncertainties-2026.md) | musl-tools + pkg-config setup step for Linux |

---

## Token Budget Estimate

| Artifact | Estimated Tokens |
|----------|-----------------|
| This story spec | ~3,000 |
| `.github/workflows/release.yml` (current, ~210 lines) | ~3,500 |
| `delta-analysis.md` §3 (defect catalog) | ~1,500 |
| `release-engineering-uncertainties-2026.md` U2, U3, U4, U5, U20, U26 | ~3,000 |
| Total | ~11,000 |

Within the 30% context window budget.

---

## Tasks

1. **Read current `.github/workflows/release.yml`** in full before any edits.

2. **Read `release-engineering-uncertainties-2026.md`** sections U2, U3, U4, U5, U20, U26.

3. **Fix DEF-REL-001 — remove binary_exists guard:**
   - Delete the `check_binary` step (the `if [[ -d "crates/prism-bin" ]]; then` guard).
   - Remove the `outputs:` block from the `build-release` job.
   - Remove all `if: steps.check_binary.outputs.binary_exists == 'true'` conditions from
     any downstream jobs (`upload-release`, etc.).
   - Confirm with grep: `grep -n 'binary_exists\|check_binary' .github/workflows/release.yml`
     must return zero matches.

4. **Fix DEF-REL-002 — remove chocolatey-publish job:**
   - Delete the entire `chocolatey-publish` job block.
   - Add comment: `# chocolatey-publish removed (DEF-REL-002): packaging/chocolatey/ does
     not exist. Chocolatey packaging is a v1.1+ consideration.`
   - Grep confirm: `grep -n 'chocolatey' release.yml` returns zero or only the comment.

5. **Fix DEF-REL-003 — disable homebrew-update job:**
   - Remove or comment out the entire `homebrew-update` job block.
   - Add comment: `# homebrew-update removed (DEF-REL-003): tap org 1898co/homebrew-tap
     does not exist. Re-enable once tap is established. S-REL-008.`

6. **Fix DEF-REL-004 — remove crates-io-publish job:**
   - Delete the entire `crates-io-publish` job block.
   - Add comment: `# crates-io-publish removed (DEF-REL-004): all 24 workspace crates
     carry publish = false. crates.io publication deferred post-v1.0.0.`

7. **Update build-release matrix job (U13):**
   - Change the cargo invocation from `-p prism-bin` to `-p prism-bin -p prism-dtu-demo-server`.
   - Add a tar-wrap step after build: `tar czf prism-dtu-demo-server-${{ matrix.target }}.tar.gz -C target/${{ matrix.target }}/release prism-dtu-demo-server` (preserves +x bit — upload-artifact ZIP strips executable mode).
   - Add Linux setup step to matrix job or a dedicated pre-build step:
     ```yaml
     - name: Install Linux build deps
       if: contains(matrix.target, 'linux')
       run: |
         sudo apt-get update
         sudo apt-get install -y musl-tools pkg-config
         # NOTE: libdbus-1-dev: VERIFY against Cargo.lock keyring backend before adding.
         # If keyring uses zbus (pure-Rust), libdbus-1-dev is NOT needed.
         # If keyring uses the C-linked backend, ADD libdbus-1-dev here.
     ```
   - Verify: `grep -n 'libdbus-1-dev' Cargo.lock` or check `keyring` crate features before
     making the final decision on libdbus-1-dev.

8. **Fix the matrix typo (U1):** Verify the matrix target `x86_64-unknown-linux-musl` is
   spelled correctly (NOT `x86_x64-unknown-linux-musl`).

9. **Artifact name fix (U23):** Ensure upload-artifact name is `release-${{ matrix.target }}`
   (NOT `prism-${{ matrix.target }}`). Verify the artifact name used by download steps in
   build-demo-bundle matches exactly.

10. **Add prerelease handling using bash array pattern (U3):**
    ```yaml
    - name: Determine release flags
      id: release_flags
      run: |
        TAG="${{ github.ref_name }}"
        PRERELEASE_ARGS=()
        if [[ "$TAG" == *-* ]]; then
          PRERELEASE_ARGS+=(--prerelease)
        fi
        echo "is_prerelease=$([[ "$TAG" == *-* ]] && echo true || echo false)" >> "$GITHUB_OUTPUT"
        # Use array form to avoid empty positional arg when NOT prerelease
        gh release create "$TAG" "${PRERELEASE_ARGS[@]}" ./dist/*
    ```
    NOTE: `gh` does NOT auto-detect prerelease from the tag — `--prerelease` MUST be explicit
    (research U3). Never pass `$PRERELEASE_FLAG` as a quoted-empty variable (use array form).

11. **Update action pins (U5, U20):**
    - Pin `actions/attest-build-provenance` to `v4.1.1` (NOT v4.1.0 — research U5 confirms
      v4.1.1 is current; v4.1.0 was stale).
    - Pin `actions/upload-artifact@v7` and `actions/download-artifact@v8` (current majors).
    - Drop `macos-13` — RETIRED 2025-12-04 (research U5). Use `macos-15-intel` for Intel
      macOS builds; add comment: `# macos-15-intel: Intel macOS last runner; EOL Aug 2027.`
    - Resolve ALL `uses:` SHA pins via `git ls-remote` at implementation time (SHAs are
      INCONCLUSIVE in research — must be resolved live). Record SHA + human-readable tag in comment.
    - VERIFY v7 upload + v8 download same-run interop works by running a smoke-test job
      (upload then immediately download in same workflow run before finalizing pins).

12. **Upload install.sh and install.ps1 as release assets (U26):**
    In the publish-release or create-release job, include `scripts/install.sh` and
    `scripts/install.ps1` in the `gh release upload` invocation:
    ```bash
    gh release upload "$TAG" scripts/install.sh scripts/install.ps1 ./dist/*
    ```
    The Windows PowerShell one-liner passes the version via `$env:PRISM_INSTALL_VERSION`
    (env var before pipe), not as a positional arg to `iex`.

13. **Fork-tag dry-run gate (U2):**
    Before pushing the real `v1.0.0-rc.1` tag to origin, run a complete dry-run on a fork:
    push a test tag (e.g., `v0.0.1-rc.test`) to a fork; verify all jobs pass; then cut the real tag.
    Document this as a mandatory verification step in RELEASING.md (S-REL-005).

14. **Run actionlint** (install via `brew install actionlint` or the official
    `download-actionlint.bash` script — NOT `cargo install actionlint` which does not exist):
    `actionlint .github/workflows/release.yml` — exit code 0 required.

---

## Acceptance Criteria

### AC-001: DEF-REL-001 closed — binary_exists guard removed
Given: `.github/workflows/release.yml` is the modified file.
When: `grep -n 'binary_exists\|check_binary' .github/workflows/release.yml` is run.
Then: Zero matches. The step `check_binary` does not appear. No job has an `if:` condition
referencing `binary_exists`. The `build-release` job has no `outputs:` block.
(traces to delta-analysis.md §3 DEF-REL-001: "Remove the check_binary step and all guards")

### AC-002: DEF-REL-002 closed — chocolatey-publish job removed
Given: `.github/workflows/release.yml` is the modified file.
When: `grep -n 'chocolatey\|choco\|nuspec' .github/workflows/release.yml` is run.
Then: Zero functional matches (comment-only references acceptable).
(traces to delta-analysis.md §3 DEF-REL-002: "Remove the chocolatey-publish job")

### AC-003: DEF-REL-003 closed — homebrew-update job removed
Given: `.github/workflows/release.yml` is the modified file.
When: `grep -n 'homebrew' .github/workflows/release.yml` is run.
Then: Zero functional matches (comment documenting the deferral acceptable). Comment references `S-REL-008`.
(traces to delta-analysis.md §3 DEF-REL-003: "Disable the homebrew-update job")

### AC-004: DEF-REL-004 closed — crates-io-publish job removed
Given: `.github/workflows/release.yml` is the modified file.
When: `grep -n 'crates.io\|crates-io\|cargo publish' .github/workflows/release.yml` is run.
Then: Zero functional matches (comment-only references acceptable).
(traces to delta-analysis.md §3 DEF-REL-004: "Remove the crates-io-publish job")

### AC-005: Prerelease flag applied via bash array pattern; gh not relying on auto-detection
Given: A tag matching `v*-*` (e.g., `v1.0.0-rc.1`) triggers the release workflow.
When: The `gh release create` invocation is inspected.
Then: The `--prerelease` flag is set via a bash array (`args+=(--prerelease)`) or equivalent
parameter-expansion form (`${PRERELEASE_FLAG:+--prerelease}`), NOT via a quoted-empty variable
that would send an empty positional arg. For tags NOT containing `-`, `--prerelease` is absent.
(traces to delta-analysis.md §2.1; research U3: gh does NOT auto-detect prerelease from tag)

### AC-006: 5-platform matrix preserved and correctly spelled
Given: The modified `.github/workflows/release.yml`.
When: The matrix strategy block is inspected.
Then: All five targets are present and correctly spelled:
  - `aarch64-apple-darwin`
  - `x86_64-apple-darwin`
  - `x86_64-unknown-linux-gnu`
  - `x86_64-unknown-linux-musl` (NOT `x86_x64-unknown-linux-musl`)
  - `x86_64-pc-windows-msvc`
(traces to delta-analysis.md §2.1; U1: typo fix x86_x64→x86_64-unknown-linux-musl)

### AC-007: SHA-256 checksums step preserved
Given: The modified `.github/workflows/release.yml`.
When: `grep -n 'sha256\|checksum' .github/workflows/release.yml` is run.
Then: At least one step that generates SHA-256 checksums and uploads `checksums.txt` is present.
(traces to delta-analysis.md §2.1: "keep checksums")

### AC-008: OIDC attestation preserved with correct pin
Given: The modified `.github/workflows/release.yml`.
When: `grep -n 'id-token\|attest' .github/workflows/release.yml` is run.
Then: `id-token: write` permission is present; `attest-build-provenance` step uses `v4.1.1`
(NOT v4.1.0). Pinned to a resolved commit SHA with a version comment.
(traces to delta-analysis.md §2.1: "OIDC attestation"; research U5: v4.1.1 is correct current)

### AC-009: build-release builds prism-bin AND prism-dtu-demo-server together
Given: The modified build-release job.
When: The cargo invocation is inspected.
Then: `-p prism-bin -p prism-dtu-demo-server` appears in one cargo command. prism-dtu-demo-server
binary is tar-wrapped before upload (to preserve +x bit).
(traces to architect U13 adjudication: "one cargo invocation, demo-server tar-wrapped")

### AC-010: Linux setup step installs musl-tools and pkg-config
Given: The modified build-release matrix job.
When: The Linux-gated setup step is inspected.
Then: `sudo apt-get install -y musl-tools pkg-config` is present, gated to
`contains(matrix.target, 'linux')`. libdbus-1-dev presence is justified by a comment citing
the verified keyring crate backend.
(traces to research U2: "musl-tools + pkg-config required; libdbus-1-dev INCONCLUSIVE")

### AC-011: install.sh and install.ps1 uploaded as release assets
Given: The publish-release job in the modified release.yml.
When: The `gh release upload` or `gh release create` invocation is read.
Then: `scripts/install.sh` and `scripts/install.ps1` are included in the upload glob.
(traces to architect U26 adjudication: "install scripts uploaded as release assets")

### AC-012: Workflow YAML parses without errors (actionlint)
Given: The modified `.github/workflows/release.yml`.
When: `actionlint .github/workflows/release.yml` is run (installed via `brew install actionlint`
or the official bash download script — NOT cargo install, which does not work).
Then: Exit code 0. Zero errors reported.
(traces to delta-analysis.md §8: "manual test tag push gate before RC-1"; research U4: actionlint
is Go, not Rust — `cargo install actionlint` is INVALID)

---

## Previous Story Intelligence

N/A — this is the first story in the E-REL epic. The release workflow was scaffolded with
`S-0.01` (greenfield phase) but never exercised against a real tag. The five defects identified
by delta-analysis §3 accumulated during development.

---

## Architecture Compliance Rules

| Rule | Source | Enforcement |
|------|--------|-------------|
| No `cargo publish` for any workspace crate | All 24 crates carry `publish = false` | Absence of any `cargo publish` in release.yml |
| 5-platform matrix is non-negotiable | ADR-022 boot contract | Matrix must remain unchanged |
| OIDC attestation with v4.1.1 pin | Supply chain security | attest-build-provenance@v4.1.1 + SHA pin |
| --prerelease via bash array, not quoted-empty var | U3: gh no auto-detect | Array form prevents empty positional arg |
| actionlint via brew/script, NOT cargo install | U4: actionlint is Go | brew install or download-actionlint.bash |
| build-release builds prism-bin + demo-server together | U13 architect adjudication | Single cargo invocation; tar-wrap demo-server |
| musl-tools + pkg-config for Linux targets | U2: musl cross-compile | Linux-gated apt-get step |
| upload-artifact@v7 + download-artifact@v8 | U5: current majors; smoke-test interop | Smoke test before finalizing |
| macos-13 RETIRED | U5: brownout Nov 2025, hard-fail Dec 2025 | Use macos-15-intel; note Aug 2027 EOL |

---

## Library & Framework Requirements

| Tool | Version | Source |
|------|---------|--------|
| `gh` CLI | 2.96.0 (on ubuntu-latest) | GitHub-hosted runner (pre-installed) |
| actionlint | ≥ 1.7.12 (verify latest on releases page) | `brew install actionlint` or download-actionlint.bash |
| `actions/checkout` | v6.0.2 — resolve SHA via git ls-remote | Research U5 |
| `actions/upload-artifact` | v7 — resolve SHA via git ls-remote | Research U5 |
| `actions/download-artifact` | v8 — resolve SHA via git ls-remote | Research U5 |
| `actions/attest-build-provenance` | v4.1.1 — resolve SHA via git ls-remote | Research U5 (NOT v4.1.0) |
| `dtolnay/rust-toolchain` | @stable (moving tag) — resolve SHA via git ls-remote | Research U20 |
| `musl-tools`, `pkg-config` | ubuntu-24.04 apt packages | Research U2 |

---

## File Structure Requirements

| File | Action | Notes |
|------|--------|-------|
| `.github/workflows/release.yml` | Modify | Remove 4 dead jobs; add prerelease; update matrix; Linux setup; upload installs; tar-wrap demo-server |

---

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|----------------|
| GitHub Actions release workflow | `.github/workflows/release.yml` | N/A (CI YAML, not Rust) |

---

## Purity Classification

| Module | Classification | Justification |
|--------|----------------|---------------|
| `.github/workflows/release.yml` | N/A | YAML CI configuration — no Rust purity boundary applies |

---

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Tag `v1.0.0-rc.1` pushed | Workflow runs; `--prerelease` flag set via array; release created as prerelease |
| EC-002 | Tag `v1.0.0` pushed (GA) | Workflow runs; no `--prerelease`; release created as GA |
| EC-003 | Tag `v2.0.0-rc.1` pushed | `*-*` pattern matches; release created as prerelease |
| EC-004 | All 5 matrix runners pass | Single release with 5 archives + checksums + attestation created |
| EC-005 | v7 upload + v8 download interop | Smoke test verifies; if broken, pin both to v7 |
| EC-006 | musl target build without libdbus-1-dev | If keyring uses zbus (pure-Rust), builds succeed; verify from Cargo.lock |
| EC-007 | Intel macOS build on macos-15-intel | Builds succeed; EOL Aug 2027 migration noted in workflow comment |

---

## Forbidden Dependencies

- No `packaging/` directory references (does not exist in the repo)
- No `1898co/homebrew-tap` repository checkout (org does not exist)
- No `cargo publish` invocations (all crates carry `publish = false`)
- No `cargo install actionlint` (actionlint is Go; no crates.io package)
- No `macos-13` runner (RETIRED Dec 2025)
- No `native-tls` or `libssl-dev` (ADR-050: rustls-tls mandatory)

---

## Changelog

| Version | Date | Summary |
|---------|------|---------|
| 0.2 | 2026-07-19 | Fix-burst: U1 typo; U2 Linux setup musl-tools+pkg-config+fork-tag dry-run; U3 bash-array prerelease (no gh auto-detect); U4 actionlint is Go not cargo; U5 attest v4.1.0→v4.1.1+macos-13 retired+upload v7/download v8+smoke-test; U13 build-release builds demo-server+tar-wrap; U20 SHA-pinning tasks; U23 artifact name release-$target; U26 install scripts uploaded as release assets; acceptance_criteria_count 9→12 |
| 0.1 | 2026-07-19 | Initial story creation (story-writer F3 burst) |
