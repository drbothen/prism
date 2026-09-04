---
document_type: story
story_id: S-REL-004
title: "devops: demo-bundle packaging — build-plugins CI job + per-platform demo bundle release asset"
wave: F-A
epic_id: E-REL
priority: P0
status: draft
version: "0.4"
level: "L4"
producer: story-writer
timestamp: "2026-07-19T00:00:00Z"
tdd_mode: strict
subsystems: []
# Subsystem anchor justification:
#   Demo bundle packaging is CI/CD infrastructure and distribution tooling. No ARCH-INDEX
#   subsystem owns GitHub Actions packaging scripts. subsystems: [] per S-0.01 precedent.
crates_touched: [devops]
target_module: devops
capabilities: []
behavioral_contracts: []
# BC status: N/A — packaging and CI tooling. No subsystem BC governs demo bundle assembly.
# Conforming per W3-FIX-CI-001 precedent.
verification_properties: []
depends_on: [S-REL-001, S-REL-002]
# Dependency anchor justifications:
#   depends_on S-REL-001: The demo bundle release asset is uploaded alongside the main
#     binary archives — it depends on the repaired release.yml base (dead jobs removed,
#     output variables cleaned up) to add the bundle packaging job without conflicts.
#   depends_on S-REL-002: The bundle archive is named `prism-demo-bundle-${TAG}-${target}.tar.gz`
#     where TAG must correspond to the correct prism-bin version (1.0.0-rc.1). S-REL-002
#     ensures the version string in the release tag and binary match.
blocks: [S-REL-007, S-REL-005]
# Dependency anchor justifications:
#   blocks S-REL-007: Windows PowerShell demo scripts (demo-setup.ps1 etc.) go INSIDE the
#     demo bundle; S-REL-007 must know the bundle structure before implementing the scripts.
#   blocks S-REL-005: RELEASING.md documents the demo bundle as a required release artifact;
#     the runbook cannot be finalized until the bundle exists.
points: 8
estimated_days: 5
risk: MEDIUM
# Risk justification: wasm-tools install on CI runner is a new dependency. WASM Component
# build is deterministic but toolchain pin matters (wasm-tools 1.248.0 per delta-analysis §7).
# Plugin artifact upload/download across job boundary is well-understood GH Actions pattern.
# Per delta-analysis §8: "MEDIUM — wasm-tools install on CI runner; build toolchain pin matters".
acceptance_criteria_count: 10
red_gate_tests: 3
estimated_passes: "2-3 LOCAL adversary passes"
holdout_scenarios: []
assumption_validations: []
risk_mitigations:
  - "wasm-tools pin (U21): install wasm-tools at exactly version 1.248.0 via
    taiki-e/install-action (NOT cargo install, which is too slow for CI). Use:
    `- uses: taiki-e/install-action@v2` with `with: tool: wasm-tools@1.248.0`."
  - "build-plugins and build-release run IN PARALLEL (U19): both jobs depend on the
    check job (or run independently); they do NOT form a sequential chain. Only
    build-demo-bundle must wait for both. This avoids serializing the entire release
    pipeline behind plugin build time."
  - "demo-server no-rebuild pattern (U13): build-release builds prism-dtu-demo-server
    and uploads it as a GH Actions artifact. build-demo-bundle DOWNLOADS this artifact
    (no rebuild). demo-bundle.sh copies from the downloaded artifact location, not from
    target/. The build-release job tar-wraps or directly uploads the demo-server binary."
  - "upload-artifact if-no-files-found: error (U18): the .prx upload step MUST set
    if-no-files-found: error to surface build failures immediately rather than silently
    creating an empty artifact that causes confusing failures downstream."
  - ".prx glob path (U18): use `crates/prism-spec-engine/plugins/*/*.prx` — the actual
    plugin build outputs .prx files under the plugin crate directory, not workspace root."
  - "crowdstrike-oauth2 deferred entirely (D-2440 sensor-scope decision): v1.0.0-rc.1 ships
    Claroty xDome only. CrowdStrike sensor, crowdstrike-oauth2.prx plugin, and associated
    TOML spec are NOT shipped in the release bundle. CrowdStrike returns via S-ADR054-WAVE-A-001
    (native auth, draft). Cyberint and Armis are also deferred per D-2440."
  - "ocsf-complex-transforms not in scope: delta-analysis §7 explicitly defers
    ocsf-complex-transforms. Only threatintel-lookup.prx is in scope for this story
    (crowdstrike-oauth2.prx deferred per D-2440 sensor-scope decision)."
  - "Windows bundle is .zip not .tar.gz (U22): Windows users have Expand-Archive (native PS 5.1
    cmdlet) but NOT tar (Windows < 1903). Build demo bundle for Windows target as .zip.
    Also include .ps1 scripts in Windows bundle, not .sh scripts."
  - "demo-bundle.sh shellcheck: the packaging script must pass shellcheck in CI."
  - "Separate bundle from main archive: prism-${TAG}-${target}.tar.gz contains ONLY prism
    binary. prism-demo-bundle-${TAG}-${target}.tar.gz is a SEPARATE release asset."
  - "attestation + checksum coverage for demo bundle public release assets (F-REL001-PR2-OBS-2):
    S-REL-001 PR-LEVEL pass-2 OBS-2 notes that prism-dtu-demo-server job-to-job artifacts and
    checksums.txt are NOT attested in S-REL-001 (acceptable — not public assets). When S-REL-004
    promotes demo bundles to public release assets, the implementer MUST explicitly decide:
    (a) whether to generate and upload a checksums.txt (SHA-256 sums for all five demo bundle
    archives) as an additional release asset alongside the bundles; and (b) whether to add SLSA
    provenance attestation via slsa-github-generator for the bundle archives (noting S-REL-001
    attests only the main prism binary archives, not the demo bundles). Both decisions must be
    recorded as inline comments in the build-demo-bundle job in release.yml before the story is
    marked complete — even if the decision is 'checksums: yes, SLSA attestation: deferred to
    follow-up story'."
inputs:
  - ".factory/planning/feature-release-engineering/delta-analysis.md"
  - ".github/workflows/release.yml"
  - "scripts/demo-setup.sh"
  - "scripts/demo-run.sh"
  - "scripts/demo-teardown.sh"
  - ".factory/research/release-engineering-uncertainties-2026.md"
input-hash: "984d7b6"
traces_to: []
cycle: "v1.0.0-release-engineering"
phase: "F3"
---

# S-REL-004 — devops: demo-bundle packaging

**Story ID:** S-REL-004
**Status:** draft
**Version:** v0.4
**Wave:** F-A
**Priority:** P0
**Points:** 8

---

## Origin

DEF-REL-005 from delta-analysis §3: the release.yml archive step only packages the
`prism` binary; the demo bundle (prism-dtu-demo-server + scripts + sensor specs + .prx
plugins + preflight tool) is not packaged as a separate release artifact. The demo is the
RC acceptance gate — the demo bundle must be downloadable from the RC release.

Human decision (OQ-1, 2026-07-19): the bundle MUST include pre-built `.prx` plugin
artifacts so consumers can run the demo without a Rust toolchain.

**Sensor scope (D-2440, 2026-09-03):** v1.0.0-rc.1 ships **Claroty xDome only**. The bundle
specs/ directory contains only `claroty.sensor.toml`. Cyberint, Armis, and CrowdStrike
sensor code remains in the workspace (built + tested) but is NOT shipped in the release bundle
and NOT claimed as supported. CrowdStrike returns via S-ADR054-WAVE-A-001 (native auth, draft);
Cyberint/Armis deferred per D-2440. Only `threatintel-lookup.prx` is built and bundled
(crowdstrike-oauth2.prx deferred).

---

## Narrative

As a demo operator or secops-factory user, I want to download a single self-contained
demo bundle from the GitHub Release page, so that I can run the full DTU-backed demo
without installing a Rust toolchain or building from source.

---

## Behavioral Contracts

This story has no subsystem BCs — demo bundle packaging is CI/CD and distribution tooling.

| Architecture Source | Clause |
|--------------------|--------|
| `delta-analysis.md` §3 (DEF-REL-005) | Demo bundle must be a separate release asset |
| `delta-analysis.md` §7 (bundle structure) | Complete manifest of bundle contents |
| `delta-analysis.md` §7 (plugin build) | build-plugins job: wasm-tools 1.248.0, single Linux runner |
| `delta-analysis.md` §7 (OQ-1 resolved) | Pre-built .prx artifacts: toolchain-free demo |

---

## Token Budget Estimate

| Artifact | Estimated Tokens |
|----------|-----------------|
| This story spec | ~4,000 |
| `delta-analysis.md` §3, §6, §7 | ~3,000 |
| `.github/workflows/release.yml` (current state post S-REL-001) | ~3,500 |
| `scripts/demo-setup.sh` (content reference) | ~1,500 |
| `scripts/demo-run.sh` | ~1,000 |
| `scripts/demo-teardown.sh` | ~500 |
| `release-engineering-uncertainties-2026.md` U13/U15/U16-U22 | ~1,500 |
| Total | ~15,000 |

Within the 30% context window budget. Implementer should load delta-analysis §7 first,
then the current release.yml (post S-REL-001), then the demo scripts.

---

## Tasks

1. **Read delta-analysis §7** for the complete bundle manifest (canonical reference).
2. **Read `.github/workflows/release.yml`** (post S-REL-001 repair) in full.
3. **Read `scripts/demo-setup.sh`, `demo-run.sh`, `demo-teardown.sh`** for content.
4. **Read `crates/prism-bin/src/main.rs`** and Justfile to find:
   - Where `just build-plugin-threatintel-infusion` outputs .prx files
     (confirm the glob `crates/prism-spec-engine/plugins/*/*.prx`)
     Note: crowdstrike-oauth2 plugin is deferred per D-2440 sensor-scope decision
   - Where infusion TOMLs live (confirmed as `specs/infusions/` per U16)
   - Where `threatintel-lookup.manifest.toml` lives (confirmed as
     `crates/prism-spec-engine/plugins/threatintel-lookup/threatintel-lookup.manifest.toml` per U17)

5. **Create `scripts/demo-bundle.sh`** (Unix demo bundle assembly):
   ```bash
   #!/usr/bin/env bash
   set -euo pipefail
   # Usage: ./scripts/demo-bundle.sh <TAG> <TARGET> <BUNDLE_DIR>
   # Assembles the demo bundle directory. Run after downloading artifacts from CI.
   TAG="${1:?TAG required}"
   TARGET="${2:?TARGET required}"
   BUNDLE_DIR="${3:?BUNDLE_DIR required}"

   mkdir -p "${BUNDLE_DIR}/plugins"
   mkdir -p "${BUNDLE_DIR}/scripts"
   mkdir -p "${BUNDLE_DIR}/specs"
   mkdir -p "${BUNDLE_DIR}/infusions"
   mkdir -p "${BUNDLE_DIR}/preflight"

   # Demo-server binary (U13: downloaded from build-release artifact, not rebuilt here)
   cp "prism-dtu-demo-server" "${BUNDLE_DIR}/"

   # Plugin artifacts (built by build-plugins job, downloaded to plugins/ dir)
   # v1.0.0-rc.1 ships Claroty xDome only (D-2440 sensor-scope decision).
   # crowdstrike-oauth2.prx deferred to S-ADR054-WAVE-A-001; Cyberint/Armis deferred per D-2440.
   cp "plugins/threatintel-lookup.prx"                               "${BUNDLE_DIR}/plugins/"
   # Threatintel manifest (U17: correct path is plugins/threatintel-lookup/)
   cp "crates/prism-spec-engine/plugins/threatintel-lookup/threatintel-lookup.manifest.toml" \
      "${BUNDLE_DIR}/plugins/"

   # Scripts (bash)
   cp scripts/demo-setup.sh scripts/demo-run.sh scripts/demo-teardown.sh "${BUNDLE_DIR}/scripts/"
   cp scripts/demo.toml                                                    "${BUNDLE_DIR}/scripts/"

   # Sensor TOMLs — v1.0.0-rc.1: Claroty xDome only (D-2440)
   # CrowdStrike deferred to S-ADR054-WAVE-A-001; Cyberint/Armis deferred per D-2440
   cp crates/prism-sensors/specs/claroty.sensor.toml      "${BUNDLE_DIR}/specs/"

   # Infusions (U16: specs/infusions/ NOT crates/prism-spec-engine/infusions/)
   cp specs/infusions/threatintel.infusion.toml  "${BUNDLE_DIR}/infusions/"
   cp specs/infusions/nvd.infusion.toml          "${BUNDLE_DIR}/infusions/"

   # Preflight
   cp scripts/t13-preflight-audit.py  "${BUNDLE_DIR}/preflight/"

   # Runbook
   cp docs/DEMO-RUNBOOK.md  "${BUNDLE_DIR}/"

   echo "Demo bundle assembled at ${BUNDLE_DIR}"
   ```

6. **Add `build-plugins` job to `.github/workflows/release.yml`** (runs parallel to build-release):
   ```yaml
   build-plugins:
     name: Build WASM plugins
     runs-on: ubuntu-latest
     steps:
       - uses: actions/checkout@v4
       - uses: dtolnay/rust-toolchain@stable
         with:
           toolchain: stable
           targets: wasm32-wasip1
       - name: Install wasm-tools
         uses: taiki-e/install-action@v2
         with:
           tool: wasm-tools@1.248.0
       # v1.0.0-rc.1 ships Claroty xDome only (D-2440); crowdstrike-oauth2 deferred to S-ADR054-WAVE-A-001
       - name: Build threatintel-infusion plugin
         run: just build-plugin-threatintel-infusion
       - name: Upload plugin artifacts
         uses: actions/upload-artifact@v4
         with:
           name: prism-plugins
           path: crates/prism-spec-engine/plugins/*/*.prx
           if-no-files-found: error
           retention-days: 1
   ```
   Note: taiki-e/install-action@v2 SHA must be pinned at implementation time via
   `git ls-remote https://github.com/taiki-e/install-action refs/tags/v2` (research U20).

7. **Extend build-release job** in release.yml to also upload prism-dtu-demo-server as a
   separate GH Actions artifact (NOT a release asset) alongside the prism binary:
   ```yaml
   - name: Upload demo-server artifact
     uses: actions/upload-artifact@v4
     with:
       name: prism-dtu-demo-server-${{ matrix.target }}
       path: target/${{ matrix.target }}/release/prism-dtu-demo-server${{ matrix.exe_suffix || '' }}
       if-no-files-found: error
       retention-days: 1
   ```

8. **Add `build-demo-bundle` job** (runs after build-release, build-plugins, AND publish-release):
   ```yaml
   build-demo-bundle:
     name: Build demo bundle (${{ matrix.target }})
     needs: [build-release, build-plugins, publish-release]
     runs-on: ${{ matrix.runner }}
     strategy:
       matrix:
         include:
           - target: aarch64-apple-darwin
             runner: macos-latest
             bundle_ext: tar.gz
           - target: x86_64-apple-darwin
             runner: macos-latest
             bundle_ext: tar.gz
           - target: x86_64-unknown-linux-gnu
             runner: ubuntu-latest
             bundle_ext: tar.gz
           - target: x86_64-unknown-linux-musl
             runner: ubuntu-latest
             bundle_ext: tar.gz
           - target: x86_64-pc-windows-msvc
             runner: windows-latest
             bundle_ext: zip
     steps:
       - uses: actions/checkout@v4
       - name: Download demo-server artifact
         uses: actions/download-artifact@v4
         with:
           name: prism-dtu-demo-server-${{ matrix.target }}
       - name: Download plugin artifacts
         uses: actions/download-artifact@v4
         with:
           name: prism-plugins
           path: plugins/
       - name: Assemble demo bundle (Unix)
         if: matrix.bundle_ext == 'tar.gz'
         run: |
           BDIR="prism-demo-bundle-${{ github.ref_name }}-${{ matrix.target }}"
           bash scripts/demo-bundle.sh "${{ github.ref_name }}" "${{ matrix.target }}" "${BDIR}"
           tar czf "${BDIR}.tar.gz" "${BDIR}/"
       - name: Assemble demo bundle (Windows)
         if: matrix.bundle_ext == 'zip'
         shell: pwsh
         run: |
           $bdir = "prism-demo-bundle-${{ github.ref_name }}-${{ matrix.target }}"
           # Windows bundle assembly: includes .ps1 scripts (from S-REL-007), .zip format
           New-Item -ItemType Directory -Path $bdir/plugins, $bdir/scripts, $bdir/specs, $bdir/infusions, $bdir/preflight -Force
           Copy-Item prism-dtu-demo-server.exe $bdir/
           # v1.0.0-rc.1: Claroty xDome only — crowdstrike-oauth2.prx deferred to S-ADR054-WAVE-A-001 (D-2440)
           Copy-Item plugins/threatintel-lookup.prx $bdir/plugins/
           Copy-Item crates/prism-spec-engine/plugins/threatintel-lookup/threatintel-lookup.manifest.toml $bdir/plugins/
           Copy-Item scripts/demo-setup.ps1, scripts/demo-run.ps1, scripts/demo-teardown.ps1, scripts/demo.toml $bdir/scripts/
           # v1.0.0-rc.1: Claroty xDome only (D-2440); crowdstrike/armis/cyberint deferred
           Copy-Item crates/prism-sensors/specs/claroty.sensor.toml $bdir/specs/
           Copy-Item specs/infusions/*.infusion.toml $bdir/infusions/
           Copy-Item scripts/t13-preflight-audit.py $bdir/preflight/
           Copy-Item docs/DEMO-RUNBOOK.md $bdir/
           Compress-Archive -Path $bdir -DestinationPath "${bdir}.zip"
       - name: Upload demo bundle to release
         run: |
           gh release upload "${{ github.ref_name }}" \
             "prism-demo-bundle-${{ github.ref_name }}-${{ matrix.target }}.${{ matrix.bundle_ext }}"
         env:
           GH_TOKEN: ${{ secrets.GITHUB_TOKEN }}
   ```

9. **Decide attestation + checksum coverage for demo bundle release assets (F-REL001-PR2-OBS-2):**
   Before the `gh release upload` step in the `build-demo-bundle` job (Task 8) can be
   considered complete, the implementer must make and record the following two decisions as
   inline comments directly in `.github/workflows/release.yml` inside the `build-demo-bundle`
   job:
   - **Checksums:** whether to generate a `checksums.txt` (SHA-256 sums of all five demo bundle
     archives) and upload it as an additional release asset alongside the bundles. If yes, add a
     step using `sha256sum` (Linux/macOS) / `Get-FileHash` (Windows) before the upload step.
   - **SLSA attestation:** whether to wire `slsa-github-generator` provenance attestation for the
     demo bundle archives (noting that S-REL-001 only attests the main `prism` binary archives;
     demo bundles are a distinct, new public asset class). If deferred, cite the follow-up story
     ID in the comment.
   Neither decision may be left as "TBD" — the comment must state a concrete choice with brief
   rationale. This gate applies even if both decisions are "no / deferred to STORY-NNN".

10. **Verify demo-bundle.sh passes shellcheck.**

---

## Acceptance Criteria

### AC-001: build-plugins job exists in release.yml and installs wasm-tools 1.248.0 via taiki-e/install-action
Given: `.github/workflows/release.yml` post this story.
When: The `build-plugins` job definition is read.
Then: The job runs on `ubuntu-latest`; installs `wasm32-wasip1` target; installs wasm-tools
at exactly version 1.248.0 via `taiki-e/install-action@v2` with `tool: wasm-tools@1.248.0`
(NOT via `cargo install`); runs the `build-plugin-threatintel-infusion` Justfile recipe
(crowdstrike-oauth2 step absent — deferred per D-2440); uploads .prx files with
`if-no-files-found: error` and path glob `crates/prism-spec-engine/plugins/*/*.prx`.
(traces to delta-analysis.md §7: "build-plugins job: wasm-tools 1.248.0, single Linux runner";
research U18 if-no-files-found: error; research U21: taiki-e/install-action;
D-2440: crowdstrike-oauth2 step absent — Claroty-only scope)

### AC-002: .prx artifact is architecture-independent (WASM bytecode)
Given: The build-plugins job runs once on Linux.
When: The uploaded artifact is inspected.
Then: `threatintel-lookup.prx` is present. The same .prx file is included in all 5
per-platform demo bundles — it is NOT built separately per platform (WASM bytecode is
platform-agnostic). Note: crowdstrike-oauth2.prx is deferred to S-ADR054-WAVE-A-001
per D-2440 sensor-scope decision; it is NOT built in this story.
(traces to delta-analysis.md §7: "build once on single runner, share across platforms";
D-2440: v1.0.0-rc.1 ships Claroty xDome only)

### AC-003: `scripts/demo-bundle.sh` produces the correct directory structure
Given: `scripts/demo-bundle.sh v1.0.0-rc.1 x86_64-unknown-linux-gnu /tmp/bundle` is run
with prism-dtu-demo-server binary and plugins/ directory present in the working dir.
When: The assembled directory is listed.
Then: The structure is as follows (v1.0.0-rc.1 — Claroty xDome only; D-2440):
  - `prism-dtu-demo-server` (binary, downloaded from build-release artifact)
  - `plugins/threatintel-lookup.prx` (crowdstrike-oauth2.prx absent — deferred to S-ADR054-WAVE-A-001)
  - `plugins/threatintel-lookup.manifest.toml` (from crates/prism-spec-engine/plugins/threatintel-lookup/)
  - `scripts/demo-setup.sh`, `scripts/demo-run.sh`, `scripts/demo-teardown.sh`
  - `scripts/demo.toml`
  - `specs/claroty.sensor.toml` (crowdstrike/armis/cyberint absent — deferred per D-2440)
  - `infusions/threatintel.infusion.toml`, `infusions/nvd.infusion.toml`
    (from `specs/infusions/`, NOT `crates/prism-spec-engine/infusions/`)
  - `preflight/t13-preflight-audit.py`
  - `DEMO-RUNBOOK.md`
(traces to delta-analysis.md §7: bundle manifest; research U16/U17: correct paths;
D-2440: sensor-scope — v1.0.0-rc.1 ships Claroty xDome only)

### AC-004: No crowdstrike artifacts are included in the bundle
Given: The assembled bundle directory.
When: `find <bundle_dir> -name '*crowdstrike*'` is run.
Then: Zero matches. The crowdstrike-oauth2.manifest.toml, crowdstrike-oauth2.prx, and
crowdstrike.sensor.toml are all absent from the bundle. CrowdStrike is deferred to
S-ADR054-WAVE-A-001 (native auth, draft). crowdstrike-oauth2.manifest.toml was also
previously excluded because it is dynamically generated by demo-setup.sh/ps1.
(traces to D-2440: v1.0.0-rc.1 ships Claroty xDome only;
CrowdStrike returns via S-ADR054-WAVE-A-001)

### AC-005: ocsf-complex-transforms is NOT in the bundle
Given: The assembled bundle directory.
When: `find <bundle_dir> -name 'ocsf*'` is run.
Then: Zero matches. ocsf-complex-transforms is out of scope per delta-analysis §7 naming
flag — only the two confirmed demo plugins are included.
(traces to delta-analysis.md §7: "S-REL-004 scoped to two confirmed demo plugins")

### AC-006: Per-platform demo bundle archives are uploaded to the GitHub Release
Given: The release workflow completes for a v*-rc.* tag.
When: The GitHub Release page is viewed.
Then: Five demo bundle assets are present — four `.tar.gz` (Unix) + one `.zip` (Windows) —
SEPARATE from the five main `prism-${TAG}-${target}.tar.gz` archives.
(traces to delta-analysis.md §3 DEF-REL-005: "separate release asset"; research U22: Windows .zip)

### AC-007: build-demo-bundle waits for build-release, build-plugins, AND publish-release
Given: The release.yml workflow definition.
When: The `build-demo-bundle` job `needs:` field is inspected.
Then: `needs: [build-release, build-plugins, publish-release]` is present. The demo bundle
job cannot start until all three predecessor jobs complete.
(traces to delta-analysis.md §7: "build-plugins job runs before matrix jobs"; research U15:
publish-release must exist before gh release upload runs)

### AC-008: demo-bundle.sh is shellcheck-clean
Given: `scripts/demo-bundle.sh` is committed.
When: `shellcheck scripts/demo-bundle.sh` is run.
Then: Exit code 0. Zero errors or warnings.
(traces to CLAUDE.md §Conventions: shellcheck for all scripts/ files)

### AC-009: Main binary archives do NOT contain demo server or plugins
Given: A `prism-${TAG}-${target}.tar.gz` archive.
When: `tar tzf prism-${TAG}-${target}.tar.gz` is run.
Then: Only `prism` (or `prism.exe`) is listed. No `prism-dtu-demo-server`, no .prx files.
(traces to delta-analysis.md §6: "prism-${TAG}-${target}.tar.gz — contains ONLY prism binary")

### AC-010: Demo bundle is Rust-toolchain-free for the consumer
Given: A consumer downloads `prism-demo-bundle-${TAG}-${target}.tar.gz` (or .zip).
When: The bundle is extracted and `scripts/demo-setup.sh` (or .ps1) is run (with prism binary
separately installed via S-REL-003 install.sh).
Then: No `cargo`, `rustup`, or `just` commands are required. All binaries and WASM
artifacts are pre-built.
(traces to delta-analysis.md §7 OQ-1: "toolchain-free demo bundle")

---

## Previous Story Intelligence

S-DEMO-003 established the demo scripts (`demo-setup.sh`, `demo-run.sh`, `demo-teardown.sh`)
and the `scripts/demo.toml` pattern. Read those scripts before writing `demo-bundle.sh`
to understand the exact file layout expected by the setup scripts.

S-REL-001 repairs release.yml — this story adds new jobs to the repaired base. Do NOT
re-add any of the removed jobs (chocolatey, homebrew, crates.io) in any new job's
`needs:` chain.

S-REL-007 (Windows PowerShell demo parity) builds `demo-setup.ps1`, `demo-run.ps1`,
`demo-teardown.ps1`. The Windows branch of build-demo-bundle (Task 8) copies these
.ps1 scripts. Since S-REL-007 blocks on this story, the .ps1 filenames are agreed
here and S-REL-007 produces them to match.

Key lessons from fix-burst research:
- U13: build-demo-bundle DOES NOT rebuild prism-dtu-demo-server — it downloads from artifact.
- U15: publish-release must exist before `gh release upload` runs; add to needs.
- U16: infusion TOMLs are at `specs/infusions/` (not under crates/).
- U17: threatintel manifest is at `crates/prism-spec-engine/plugins/threatintel-lookup/`.
- U18: upload-artifact `if-no-files-found` defaults to `warn` — must override to `error`.
- U19: build-plugins and build-release run IN PARALLEL (not sequentially).
- U21: use taiki-e/install-action@v2 for wasm-tools, not cargo install (too slow for CI).
- U22: Windows bundle is .zip, not .tar.gz; includes .ps1 scripts.

D-2440 sensor-scope (2026-09-03): v1.0.0-rc.1 ships Claroty xDome only. Demo bundle
includes only claroty.sensor.toml and threatintel-lookup.prx. CrowdStrike returns via
S-ADR054-WAVE-A-001; Cyberint/Armis deferred per D-2440. Code remains in workspace
(built+tested) but not shipped in the release bundle.

---

## Architecture Compliance Rules

| Rule | Source | Enforcement |
|------|--------|-------------|
| wasm-tools pinned at 1.248.0 | delta-analysis §7 | taiki-e/install-action@v2 with wasm-tools@1.248.0 |
| build-plugins and build-release run IN PARALLEL | Research U19 | Neither job has needs: the other |
| build-demo-bundle waits for all three predecessors | Research U15 | needs: [build-release, build-plugins, publish-release] |
| Main archive contains ONLY prism binary | delta-analysis §6 separation rule | AC-009 verification |
| All crowdstrike artifacts excluded | D-2440 sensor-scope: v1.0.0-rc.1 Claroty-only | AC-004 verification |
| .prx files are architecture-independent | WASM bytecode is platform-agnostic | AC-002 |
| upload-artifact if-no-files-found: error | Research U18 | Explicit field in both upload steps |
| infusion TOMLs at specs/infusions/ | Research U16 | Path verified; NOT crates/prism-spec-engine/infusions/ |
| threatintel manifest path | Research U17 | crates/prism-spec-engine/plugins/threatintel-lookup/ |
| Windows bundle is .zip | Research U22 | bundle_ext: zip in matrix; Compress-Archive in PS step |

---

## Library & Framework Requirements

| Tool | Version | Notes |
|------|---------|-------|
| wasm-tools | 1.248.0 | Via taiki-e/install-action@v2 (not cargo install) |
| wasm32-wasip1 target | Rust stable | rustup target add wasm32-wasip1 |
| `just` (Justfile runner) | As installed by repo toolchain | Recipe: build-plugin-threatintel-infusion (crowdstrike-oauth2 deferred per D-2440) |
| taiki-e/install-action | v2 (SHA-pinned at impl time) | Fast binary install for CI tools |
| actions/upload-artifact | v4 | Must set if-no-files-found: error |
| actions/download-artifact | v4 | Must match upload version |

---

## File Structure Requirements

| File | Action | Notes |
|------|--------|-------|
| `scripts/demo-bundle.sh` | Create | Unix bundle assembly script; shellcheck-clean |
| `.github/workflows/release.yml` | Modify | Add build-plugins job + extend build-release + add build-demo-bundle job |

---

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|----------------|
| `scripts/demo-bundle.sh` | `scripts/` | N/A (shell script) |
| `build-plugins` CI job | `.github/workflows/release.yml` | N/A (CI YAML) |
| `build-demo-bundle` CI job | `.github/workflows/release.yml` | N/A (CI YAML) |
| WASM plugin artifacts | `*.prx` | N/A (WASM bytecode; built by Justfile recipes) |

---

## Purity Classification

| Module | Classification | Justification |
|--------|----------------|---------------|
| `scripts/demo-bundle.sh` | N/A | Shell script — no Rust purity boundary applies |
| `.github/workflows/release.yml` additions | N/A | YAML CI configuration — no Rust purity boundary applies |

---

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Justfile recipe output path differs | Implementer reads Justfile before assuming .prx path; glob must match actual output |
| EC-002 | specs/infusions/ path not found | Read actual repo; delta-analysis §7 says specs/infusions/ per U16 |
| EC-003 | threatintel-lookup.manifest.toml path | crates/prism-spec-engine/plugins/threatintel-lookup/ per U17 |
| EC-004 | Windows bundle (.zip + .ps1 scripts) | Separate PS step creates .zip; copies .ps1 not .sh |
| EC-005 | build-plugins job fails | build-demo-bundle is blocked; clear error surfaced in GH Actions UI |
| EC-006 | demo-server binary not in artifact | if-no-files-found: error in build-release upload step catches this |
| EC-007 | .prx files not found by glob | if-no-files-found: error in build-plugins upload step catches this |

---

## Forbidden Dependencies

- No `ocsf-complex-transforms` plugin in bundle (no Justfile recipe exists; not in demo-setup.sh)
- No `crowdstrike-oauth2.prx`, `crowdstrike-oauth2.manifest.toml`, or `crowdstrike.sensor.toml`
  in bundle (CrowdStrike deferred to S-ADR054-WAVE-A-001 per D-2440 sensor-scope decision)
- No `armis.sensor.toml` or `cyberint.sensor.toml` in bundle (deferred per D-2440)
- No source code in bundle (bundle is consumer-facing; no Rust source)
- No mixing of demo bundle contents into main `prism-${TAG}-${target}.tar.gz`
- No `cargo install` for wasm-tools in CI (too slow; use taiki-e/install-action per U21)
- No `needs: [build-plugins]` on build-release (they run in parallel per U19)

---

## Changelog

| Version | Date | Summary |
|---------|------|---------|
| 0.4 | 2026-09-03 | D-2440 sensor-scope: v1.0.0-rc.1 Claroty-only — crowdstrike-oauth2.prx build step removed from build-plugins job; crowdstrike/armis/cyberint sensor TOMLs removed from bundle; Windows bundle sensor glob narrowed to claroty.sensor.toml; AC-001/002/003/004 updated; deferral note added in Origin and Previous Story Intelligence; Forbidden Dependencies updated |
| 0.3 | 2026-07-20 | Forward-note (F-REL001-PR2-OBS-2): attestation + checksum coverage for demo bundle public release assets must be explicitly decided at implementation time — risk_mitigations entry added; Task 9 attestation decision gate inserted; shellcheck task renumbered to Task 10 |
| 0.2 | 2026-07-19 | Fix-burst: U13 demo-server downloaded from artifact (no rebuild); U15 build-demo-bundle needs publish-release; U16 infusion paths corrected to specs/infusions/; U17 manifest path corrected to plugins/threatintel-lookup/; U18 if-no-files-found error + prx glob; U19 build-plugins parallel to build-release; U21 taiki-e/install-action for wasm-tools; U22 Windows bundle is .zip with .ps1 scripts; research file added to inputs |
| 0.1 | 2026-07-19 | Initial story creation (story-writer F3 burst) |
