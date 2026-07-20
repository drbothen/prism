---
document_type: story
story_id: S-REL-001
title: "devops: release.yml repair — remove dead jobs (DEF-REL-001 through DEF-REL-004) + v*-rc.* prerelease handling + Linux cross-compile setup + install-script upload"
wave: F-A
epic_id: E-REL
priority: P0
status: merged
version: "0.24"
level: "L4"
producer: story-writer
timestamp: "2026-07-19T00:00:00Z"
updated: "2026-07-20"
merged_sha: 7fef57da
merged_pr: 228
merged_date: "2026-07-20"
tdd_mode: strict
subsystems: []
# Subsystem anchor justification:
#   This story modifies .github/workflows/release.yml only — pure CI/CD infrastructure.
#   No ARCH-INDEX subsystem owns GitHub Actions workflow files; subsystems: [] is correct
#   per S-0.01 and S-MAINT-CI-DISK-EXHAUSTION-001 precedent.
crates_touched: [devops, prism-credentials]
# crates_touched justification:
#   devops: .github/workflows/release.yml (primary scope).
#   prism-credentials: Cargo.toml target-conditional feature split (musl = keyutils-only;
#   gnu retains dbus Secret Service via [target.'cfg(...)'.dependencies]) per delta-analysis.md
#   §14 Option B (DEFECT-REL001-MUSL-DBUS-001). Option A (PKG_CONFIG_ALLOW_CROSS=1) was
#   rejected as ABI-unsound per architect adjudication.
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
red_gate_tests: 11  # count of executable AC test files in tests/release-gate/; suite enforces EXPECTED_TEST_FILES=11/EXPECTED_ASSERTIONS exact-count floors (EXPECTED=92 precedent; POL-34; F-REL001-P7-001); 9 were RED at gate, AC-006/007 already satisfied on develop; AC-012 verified by inspection + CI run (no new test file added to suite)
estimated_passes: "1-2 LOCAL adversary passes"
holdout_scenarios: []
assumption_validations: []
risk_mitigations:
  - "Actionlint is Go, not Rust: do NOT run `cargo install actionlint`. Install via
    `brew install actionlint` locally; CI uses direct pinned-tarball download + SHA-256
    verification — see ci.yml release-gate job (F-REL001-P20-003 supersession;
    download-actionlint.bash superseded). Research U4 confirmed: no crates.io package
    named actionlint exists. Invoke as bare `actionlint` with no arguments to lint all
    workflows."
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
  - "Linux cross-compile setup (U2): libdbus-1-dev REQUIRED unconditionally on both Linux
    legs: prism-credentials (gnu target) enables keyring-linux-native-sync-persistent →
    dbus-secret-service 4.1.0 → dbus 0.9.11 → libdbus-sys 0.2.7 (C-linked). build.rs runs
    on the glibc host for the gnu leg and must find libdbus-1-dev on the runner.
    After §14 Option B (DEFECT-REL001-MUSL-DBUS-001): musl target no longer links libdbus-sys
    (target-conditional split in prism-credentials — keyutils-only path for musl); cargo tree
    --target x86_64-unknown-linux-musl -i libdbus-sys now returns EMPTY. Only the gnu target
    shows libdbus-sys v0.2.7. The apt-get step installs libdbus-1-dev unconditionally on both
    Linux legs (harmless for musl, required for gnu). ci.yml installs libdbus-1-dev
    pkg-config unconditionally on every Linux job (ADR-034/BC-2.06.003). Do NOT add
    libssl-dev (ADR-050 mandates rustls-tls)."
  - "action pin SHAs: all `uses:` entries must be pinned to an immutable commit SHA
    (repo convention; research U20). Resolve SHAs at implementation time via
    `git ls-remote https://github.com/<owner>/<repo> refs/tags/<tag>`.
    Canonical versions from research: checkout@v6.0.2; upload-artifact@v7; attest-build-provenance@v4.1.1
    (NOT v4.1.0 which was stale); macos-13 is RETIRED -- use macos-15-intel for Intel macOS
    builds and note its Aug 2027 EOL. upload@v7 + download@v8 same-run interop is verified by the
    workflow's own build-release→publish-release artifact flow, exercised end-to-end by the
    mandatory origin test-tag dry-run gate (task 12) — no separate smoke job required (F-REL001-P7-002)."
  - "build-release builds prism-bin + prism-dtu-demo-server together (U13): the matrix
    cargo invocation uses `-p prism-bin -p prism-dtu-demo-server` in one call. The
    prism-dtu-demo-server binary is tar-wrapped (tar czf) before upload-artifact to preserve
    the +x bit (research U18: upload-artifact ZIP format strips executable bits). The build-demo-bundle
    job downloads+untars rather than re-building. The demo-server wrap step uses per-OS
    conditional logic matching the `archive_ext` matrix variable (same pattern as the main
    `prism` archive step): `tar czf` on Unix (preserves +x); `7z a ... .exe` on Windows.
    Strip applies to both `prism` and `prism-dtu-demo-server` on Unix legs."
  - "install.sh and install.ps1 are authored by S-REL-003. S-REL-003 also amends the
    publish-release job in release.yml to add the gh release upload step for those scripts.
    S-REL-001 does NOT implement the upload — it establishes the release URL pattern that
    S-REL-003 consumes. (U26 upload step ownership reassigned to S-REL-003 per pre-TDD scan
    ADJ-002.)"
  - "origin test-tag dry-run (U2 finding — fork infeasible): drbothen cannot fork
    drbothen/prism (GitHub prevents a repo owner from forking their own repo — latent
    assumption defect in research U2). HUMAN-APPROVED (2026-07-19): push the disposable
    prerelease tag v0.0.1-rc.test to ORIGIN (drbothen/prism) as the dry-run gate before
    cutting the real v1.0.0-rc.1 tag. OIDC attestation works natively on the public origin
    repo; Actions minutes are free. Push TAG ONLY — never push the feature branch ref or
    develop/main. Mandatory cleanup after evidence capture: delete the release and tag from
    origin (`gh release delete v0.0.1-rc.test --repo drbothen/prism --yes` then
    `git push origin --delete v0.0.1-rc.test`). The transient public prerelease window
    is an accepted, human-approved tradeoff with mandatory cleanup. This is a mandatory
    verification task in the story's task list (task 12)."
  - "Actions expression injection (CWE-78): NEVER textually interpolate ${{ github.ref_name }}
    or any ref-derived expression (incl. ${{ env.* }} re-exposure) inside run: script source.
    Bind via the env: map and reference plain shell variables. Applies to all
    release-engineering stories (S-REL-003/004 especially — they add more run: blocks)."
  - "Test-gate tools must fail closed: a missing external tool (actionlint) FAILS the gate
    rather than skipping. TAP runners must reconcile each file's 1..N plan against emitted
    results and treat non-zero test-file exit as aggregate failure (F-REL001-P2-002; POL-34).
    The suite additionally enforces EXPECTED_TEST_FILES and EXPECTED_ASSERTIONS exact-count
    floors — adding or removing test files without updating these constants is a gate failure,
    following the EXPECTED=92 precedent in scripts/check-non-exhaustive.sh (F-REL001-P7-001).
    Any fix-burst that adds load-bearing workflow logic MUST add a corresponding release-gate
    assertion in the same burst (F-REL001-P10-001 [process-gap] codification) — EC-only closure
    verified by inspection bypasses the regression net."
  - "Workflow build-environment parity: any toolchain step present in ci.yml build jobs
    (protoc, musl-tools, libdbus-1-dev, etc.) must be present in release.yml build-release
    unless explicitly justified; environment gaps are invisible to static review and only
    surface at dry-run time — 13 adversarial passes missed the missing protoc step, caught
    only by the origin test-tag dry-run gate (DEFECT-REL001-PROTOC-MISSING-001; dry-run run
    https://github.com/drbothen/prism/actions/runs/29709483646)."
  - "Release step idempotency (F-REL001-P9-001; F-REL001-P10-002 trigger-correction): `gh release create`
    aborts if the release already exists. Because publish-release carries `needs: build-release`, a failed
    matrix leg means publish-release never ran → no release object exists → re-run takes the CREATE path.
    The `gh release upload --clobber` path is reachable only via: (a) publish-step transient failure AFTER
    `gh release create` succeeded (e.g., asset-upload timeout); (b) manual full re-run of an
    already-successful workflow; (c) tag re-push when a release already exists. Guard every
    `gh release create` call: check `gh release view \"$TAG\"` first — existing release takes
    the `gh release upload --clobber` path (prerelease status persists, no flag needed); fresh
    tag takes the original create path with `${PRERELEASE_ARGS[@]}`. RELEASING.md (S-REL-005)
    documents the three re-run triggers and confirms no manual release deletion is required.
    This discipline applies to all release-engineering stories — S-REL-004 adds additional
    upload steps and must follow the same view→upload-clobber idempotency pattern."
  - "Cross-target C-dependency audit (§14; DEFECT-REL001-MUSL-DBUS-001): any C-linked transitive
    dependency (identified via pkg-config invocation or build.rs) MUST be validated on every
    cross-compile target independently. A passing host build or glibc build proves NOTHING about
    cross-targets — pkg-config refuses cross-compilation mode by default (exit 101). The correct
    fix is target-conditional [target.'cfg(...)'.dependencies] to eliminate C-linked deps from
    cross-targets that cannot satisfy them (§14 Option B). PKG_CONFIG_ALLOW_CROSS=1 (Option A)
    is ABI-unsound and MUST NOT be used — it silences the pkg-config refusal but produces a
    binary that dynamically links a glibc-flavored shared object into a musl binary, causing
    crashes at runtime. Artifact-level linkage proof (readelf -d | grep NEEDED; file binary
    output = 'statically linked') is the authoritative gate — build exit 0 alone is NOT
    sufficient to confirm a musl cross-compile is clean. Fast pre-flight proxy on the developer
    machine: `cargo tree --target <musl-target> -i <crate-name>` — empty output confirms the
    dep is absent from the musl tree before pushing the dry-run tag.
    C++ deps (rocksdb) need an explicit musl-capable CXX; cc-rs probes
    CXX_<target-with-underscores> env (e.g., `CXX_x86_64_unknown_linux_musl=clang++`); the
    artifact linkage gate (readelf/file static proof) validates the result
    (DEFECT-REL001-MUSL-CXX-001). A musl-capable compiler alone is insufficient — the C++
    RUNTIME must also be musl-built; a system clang++ links glibc-built `libstdc++.a`, producing
    117+ undefined glibc-only symbol refs at link time even after compile succeeds
    (DEFECT-REL001-MUSL-LIBSTDCXX-001); §15 ratified cargo-zigbuild + zig's musl-built libc++
    as the correct fix (CXX-override approach SUPERSEDED).
    The linux-gnu persistence invariant (target-cfg keyring block with linux-native-sync-persistent)
    is guarded by a release-gate Cargo.toml assertion — the compile_error! guard alone cannot detect
    its removal (silent keyutils-only downgrade; F-REL001-P14-001)."
inputs:
  - ".github/workflows/release.yml"
  - ".factory/planning/feature-release-engineering/delta-analysis.md"
  - ".factory/research/release-engineering-uncertainties-2026.md"
input-hash: "3bd3474"
traces_to: []
cycle: "v1.0.0-release-engineering"
phase: "F3"
---

# S-REL-001 — devops: release.yml repair

**Story ID:** S-REL-001
**Status:** draft
**Version:** v0.24
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
cross-compile requires a musl-tools setup step (U2); (d) install.sh/install.ps1 are authored by S-REL-003, which also amends the publish-release
job to upload them (U26 upload step ownership reassigned to S-REL-003 per pre-TDD scan ADJ-002).

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
| Architect U26 adjudication | install.sh/.ps1 uploaded as release assets by publish-release — upload step implemented in S-REL-003 (which authors the files; upload moved per pre-TDD scan ADJ-002; files do not exist at S-REL-001 implementation time) |
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
   - Add comment: `# crates-io-publish removed (DEF-REL-004): all workspace crates
     carry publish = false. crates.io publication deferred post-v1.0.0.`

7. **Update build-release matrix job (U13):**
   - Change the cargo invocation from `-p prism-bin` to `-p prism-bin -p prism-dtu-demo-server`.
   - Add Linux setup step to matrix job or a dedicated pre-build step:
     ```yaml
     - name: Install Linux build deps
       if: contains(matrix.target, 'linux')
       run: |
         sudo apt-get update
         # libdbus-1-dev: required by prism-credentials keyring-linux-native-sync-persistent
         # → dbus-secret-service (C-linked libdbus). build.rs runs on glibc host even for
         # musl cross-target. ADR-034/BC-2.06.003.
         sudo apt-get install -y musl-tools pkg-config libdbus-1-dev
     ```
   - **musl leg C++ toolchain — DEFECT-REL001-MUSL-CXX-001 + DEFECT-REL001-MUSL-LIBSTDCXX-001 (§15 SUPERSEDES v0.16 clang++ approach):**
     *[v0.16 clang++ approach — SUPERSEDED by §15 (cargo-zigbuild); history preserved below]*
     librocksdb-sys builds RocksDB from source via cc-rs, which probes for a musl-targeted C++
     compiler. musl-tools ships only `musl-gcc` (no C++); cc-rs exits 101 with `ToolNotFound`
     (dry-run attempt-3: https://github.com/drbothen/prism/actions/runs/29712784282).
     **v0.16 approach (SUPERSEDED):** install `clang`; set `CXX_x86_64_unknown_linux_musl=clang++`.
     This resolved the compile step but dry-run attempt-4
     (https://github.com/drbothen/prism/actions/runs/29714047923) confirmed the linker failure:
     system clang++ links glibc-built `libstdc++.a` — 117 undefined glibc-only symbol refs remain
     in the musl binary (DEFECT-REL001-MUSL-LIBSTDCXX-001). Rejected alternatives:
     `-static-libstdc++` (contamination is in the objects, not the link command);
     musl-cross-make (cost per §15).
     **§15 RATIFIED FIX — cargo-zigbuild:** Architect `delta-analysis.md` §15 ratified
     cargo-zigbuild for the musl leg. Zig ships its own musl-built libc++ (not system
     glibc-built `libstdc++.a`), eliminating the 117 undefined glibc refs at the source.
     Remove `clang` from the apt-get line (F-REL001-P16-001) and remove the
     `CXX_x86_64_unknown_linux_musl: clang++` env override. **Implementation-of-record is
     `.github/workflows/release.yml`** — the step YAML is not duplicated here verbatim to
     prevent drift (F-REL001-P16-002; this snippet drifted once and was the subject of pass-16).

     **Normative musl toolchain setup — key invariants (verified against shipped release.yml):**
     - ziglang installed via `pip3 install --require-hashes -r .github/workflows/requirements-musl-ci.txt`
       (`pip3`, not `pip`; `requirements-musl-ci.txt` pins `ziglang==0.16.0` + sha256 per §15-2).
     - Step order: Cache cargo-zigbuild binary FIRST, then Install-if-absent (Cache → Install;
       Install-before-Cache order would defeat the cache hit on warm runs).
     - Cache-hit skip guard: `[[ ! -f "$HOME/.cargo/bin/cargo-zigbuild" ]]` wraps the install command.
     - Install invocation: `cargo install --locked cargo-zigbuild --version 0.23.0`
       (flag order: `--locked` before `--version` — as shipped).
     - Cache key: `cargo-zigbuild-0.23.0-${{ runner.os }}`; path: `~/.cargo/bin/cargo-zigbuild`
       only (zig not cached — hash-pinned pip install runs fresh each time; cargo-zigbuild binary
       is zig-version-agnostic, so no zig version in key).
     - Replace `cargo build` with `cargo zigbuild` on the musl leg. ADR-050 rustls unaffected.
     - `prism-dtu-demo-server` receives identical cargo-zigbuild treatment on the musl leg.
     - Artifact gate: `readelf -d <binary> | grep NEEDED` must return zero matches AND
       `file <binary>` must report "statically linked" for BOTH `prism` AND
       `prism-dtu-demo-server` musl binaries — see Task 12 five-check table rows 2–3.
   - **setup-protoc step required on ALL legs (not Linux-gated):** add `arduino/setup-protoc@v3.0.0`
     (SHA-pinned, mirroring ci.yml exactly) before the cargo build step on every matrix target.
     prism-ocsf's `build.rs` invokes `prost-build` which shells out to `protoc` at compile time;
     GitHub-hosted runners do NOT pre-install protoc, so the first dry-run
     (https://github.com/drbothen/prism/actions/runs/29709483646) failed on all 5 legs with
     exit 101. (DEFECT-REL001-PROTOC-MISSING-001)
   - Replace the single tar-wrap step for demo-server with the following two-step block
     (ADJ-003 — per-OS conditional wrap matching the existing `archive_ext` matrix variable):

     Step 1 — strip demo-server on Unix (note: strip for `prism` is handled by the existing
     workflow strip step; this step covers `prism-dtu-demo-server`):
     ```yaml
     - name: Strip demo-server (Unix)
       if: runner.os != 'Windows'
       run: strip target/${{ matrix.target }}/release/prism-dtu-demo-server 2>/dev/null || true
     ```

     Step 2 — per-OS conditional wrap using the existing `archive_ext` matrix variable:
     ```yaml
     - name: Wrap prism-dtu-demo-server for artifact upload
       shell: bash
       run: |
         if [ "${{ matrix.archive_ext }}" = "zip" ]; then
           cd target/${{ matrix.target }}/release
           7z a "../../../prism-dtu-demo-server-${{ matrix.target }}.zip" prism-dtu-demo-server.exe
         else
           tar czf prism-dtu-demo-server-${{ matrix.target }}.tar.gz \
             -C target/${{ matrix.target }}/release prism-dtu-demo-server
         fi
     ```

     Upload step (job-to-job only — NOT a public release asset):
     ```yaml
     - name: Upload demo-server artifact (job-to-job)
       uses: actions/upload-artifact@...  # same pin as main artifact step
       with:
         name: prism-dtu-demo-server-${{ matrix.target }}
         path: prism-dtu-demo-server-${{ matrix.target }}.${{ matrix.archive_ext }}
     ```
   - Demo-server ships on ALL 5 targets. `build-demo-bundle` (S-REL-004) downloads this
     artifact and extracts based on `archive_ext` — that extraction logic is S-REL-004's scope.

8. **Fix the matrix typo (U1):** Verify the matrix target `x86_64-unknown-linux-musl` is
   spelled correctly (NOT `x86_x64-unknown-linux-musl`).

9. **Artifact name fix (U23):** Ensure upload-artifact name is `release-${{ matrix.target }}`
   (NOT `prism-${{ matrix.target }}`). Verify the artifact name used by download steps in
   build-demo-bundle matches exactly.

10. **Add prerelease handling using bash array pattern (U3):**
    ```yaml
    - name: Determine release flags
      id: release_flags
      env:
        TAG: ${{ github.ref_name }}
      run: |
        PRERELEASE_ARGS=()
        if [[ "$TAG" == *-* ]]; then
          PRERELEASE_ARGS+=(--prerelease)
        fi
        echo "is_prerelease=$([[ "$TAG" == *-* ]] && echo true || echo false)" >> "$GITHUB_OUTPUT"
        # Use set-u-safe array form to avoid empty positional arg when NOT prerelease.
        # Asset list: see release.yml publish-release step for implementation-of-record
        # (F-REL001-P16-002 — spec omits asset paths to prevent drift).
        gh release create "$TAG" ${PRERELEASE_ARGS[@]+"${PRERELEASE_ARGS[@]}"}
    ```
    NOTE: `gh` does NOT auto-detect prerelease from the tag — `--prerelease` MUST be explicit
    (research U3). Never pass `$PRERELEASE_FLAG` as a quoted-empty variable (use set-u-safe
    array form: `${PRERELEASE_ARGS[@]+"${PRERELEASE_ARGS[@]}"}`).
    NOTE (F-REL001-P9-001/F-REL001-P10-002 — idempotency guard): `gh release create` aborts if
    the release already exists. Because publish-release carries `needs: build-release`, a failed
    matrix leg means publish-release never ran → no release object exists → re-run takes the
    CREATE path. The `gh release upload --clobber` path is reachable only via: (a) publish-step
    transient failure AFTER `gh release create` succeeded (e.g., asset-upload timeout); (b)
    manual full re-run of an already-successful workflow; (c) tag re-push when a release already
    exists. Guard the step: run `gh release view "$TAG"` first. If the release exists, take the
    upload path — `gh release upload --clobber "$TAG" <asset-paths>` (asset paths per
    release.yml publish-release step — F-REL001-P16-002; no `--prerelease` flag needed;
    the prerelease status persists on the existing release object). If the release does not
    exist, take the original create path with
    `${PRERELEASE_ARGS[@]+"${PRERELEASE_ARGS[@]}"}`. RELEASING.md (S-REL-005)
    must document the three re-run triggers so release engineers know no manual release deletion
    is required.

11. **Update action pins (U5, U20):**
    - Pin `actions/attest-build-provenance` to `v4.1.1` (NOT v4.1.0 — research U5 confirms
      v4.1.1 is current; v4.1.0 was stale).
    - Pin `actions/upload-artifact@v7` and `actions/download-artifact@v8` (current majors).
    - Drop `macos-13` — RETIRED 2025-12-04 (research U5). Use `macos-15-intel` for Intel
      macOS builds; add comment: `# macos-15-intel: Intel macOS last runner; EOL Aug 2027.`
    - Resolve ALL `uses:` SHA pins via `git ls-remote` at implementation time (SHAs are
      INCONCLUSIVE in research — must be resolved live). Record SHA + human-readable tag in comment.
    - The v7 upload + v8 download same-run interop is verified by the workflow's own
      build-release (upload-artifact v7) → publish-release (download-artifact v8) artifact
      flow, exercised end-to-end by the mandatory origin test-tag dry-run gate (task 12) — no
      separate smoke job is required (F-REL001-P7-002 adjudication).

12. **Origin test-tag dry-run gate (U2) — capture and preserve dry-run evidence (F-REL001-P12-OBS-002; POL-32):**
    Before pushing the real `v1.0.0-rc.1` tag to origin, push the disposable prerelease tag
    `v0.0.1-rc.test` directly to ORIGIN (`drbothen/prism`). Destination rationale: drbothen
    cannot fork drbothen/prism (GitHub prevents a repo owner from forking their own repo —
    latent assumption defect in research U2); `drbothen/prism` is a public repo, so OIDC
    attestation works natively and Actions minutes are free; this is a HUMAN-APPROVED destination
    (2026-07-19). Push the TAG ONLY — never push the feature branch ref, never push develop or
    main. Verify all jobs pass. Delete the release and tag from origin immediately after evidence
    capture (mandatory cleanup — transient public prerelease window is an accepted, human-approved
    tradeoff with mandatory cleanup). Then cut the real tag.
    Document this as a mandatory verification step in RELEASING.md (S-REL-005). The RELEASING.md
    runbook must describe the origin test-tag procedure including the mandatory cleanup steps —
    NOT a fork procedure.
    The dry-run evidence MUST be captured and preserved — the origin test-tag dry-run is the sole
    committed verification that all 5 cross-compile targets (notably x86_64-unknown-linux-musl,
    which cross-compiles rocksdb's C++ via prism-storage) succeed; without captured evidence it
    is ephemeral and not auditable from committed artifacts (F-REL001-P12-OBS-002).
    Record the following in `docs/demo-evidence/S-REL-001/fork-tag-dry-run.md` (historical
    filename retained; note inside the file that the destination was origin, not a fork):
    - Origin workflow run URL (GitHub Actions run permalink)
    - Per-leg conclusion for each of the 5 matrix targets (pass/fail + job link)
    - Release-asset listing (output of `gh release view v0.0.1-rc.test --repo drbothen/prism --json assets`)
    - Fork-of-own-repo impossibility note and human approval of origin destination (2026-07-19)
    - Cleanup confirmation: release deleted (`gh release delete v0.0.1-rc.test --repo drbothen/prism --yes`)
      and tag deleted (`git push origin --delete v0.0.1-rc.test`)
    Create the file at commit time; it is part of the story's deliverable, not an optional note.
    RELEASING.md (S-REL-005) must cite `docs/demo-evidence/S-REL-001/fork-tag-dry-run.md`
    as the auditable record of cross-compile success so that musl/windows cross-compile
    validation is traceable after the fact (F-REL001-P12-OBS-002; POL-32).

    **DEFECT-REL001-PROTOC-MISSING-001 — dry-run re-run required:** The first dry-run attempt
    (https://github.com/drbothen/prism/actions/runs/29709483646) FAILED on all 5 legs because
    release.yml did not install protoc (required by prism-ocsf prost-build). After the
    implementer adds the setup-protoc step (Task 7), the dry-run MUST be re-run against origin.
    The evidence file (`docs/demo-evidence/S-REL-001/fork-tag-dry-run.md`) MUST be updated with
    the green re-run result APPENDED below the failed-run record — the failed-run record is
    preserved as history and documents the executed-evidence principle: 13 static adversarial
    passes could not catch a build-environment gap; only the dry-run could.

    **DEFECT-REL001-MUSL-DBUS-001 — dry-run attempt 3 required (§14 Option B):** The second
    dry-run attempt (https://github.com/drbothen/prism/actions/runs/29711315678) confirmed the
    protoc fix (4/5 legs green) and surfaced DEFECT-REL001-MUSL-DBUS-001: libdbus-sys
    pkg-config cross-compile refusal on the musl leg. Option B (§14 Delta B-1/B-2) — target-
    conditional feature split in prism-credentials — has been applied (Task 15). Attempt 3 MUST
    pass all five checks in the following verification table before the evidence file is updated.
    Checks 1–5 are ALL BLOCKING; a partial pass does not close the defect.

    | Check | Verification method | Pass criterion | Gate |
    |-------|--------------------|--------------|----|
    | 1. Build exit code | GitHub Actions dry-run — all 5 matrix legs | Exit 0 on all legs | BLOCKING |
    | 2. NEEDED section empty (no dynamic deps) | `readelf -d <musl-binary> \| grep NEEDED` on BOTH `prism` AND `prism-dtu-demo-server` musl artifacts | Zero matches on both binaries (no libdbus, no libstdc++, no glibc shared objects) | BLOCKING |
    | 3. Statically linked (both binaries) | `file <musl-binary>` on BOTH `prism` AND `prism-dtu-demo-server` musl artifacts | Both outputs contain "statically linked" | BLOCKING artifact gate |
    | 4. musl tree libdbus-free | `cargo tree --target x86_64-unknown-linux-musl -i libdbus-sys` | Empty output (no libdbus-sys in tree) | BLOCKING |
    | 5. gnu tree retains libdbus | `cargo tree --target x86_64-unknown-linux-gnu -i libdbus-sys` | `libdbus-sys vX.Y.Z` present | BLOCKING |

    Checks 2 and 3 require downloading BOTH musl artifacts from the dry-run run and inspecting
    them locally: `gh run download <run-id> --repo drbothen/prism --name release-x86_64-unknown-linux-musl`
    (for `prism`) and `gh run download <run-id> --repo drbothen/prism --name prism-dtu-demo-server-x86_64-unknown-linux-musl`
    (for `prism-dtu-demo-server`). Both binaries must pass checks 2 and 3.
    Checks 4 and 5 can be run on the developer machine before pushing the tag (they are fast
    pre-flight proxies — but the dry-run's exit-0 on check 1 is the authoritative gate).
    Attempts 1 and 2 are preserved as history in the evidence file
    (`docs/demo-evidence/S-REL-001/fork-tag-dry-run.md`). Append attempt 3 results below the
    attempt 2 record — do NOT overwrite prior history.

    **Dry-run attempt-6 re-verification (F-REL001-P16-001 — clang removal) — GREEN (2026-07-20):**
    F-REL001-P16-001 removed `clang` from the apt-get install line. Attempt-6
    (run https://github.com/drbothen/prism/actions/runs/29721841906) confirmed all 5 matrix legs
    exited 0 and the publish job PASSED. `clang` was absent from the apt-get install line and
    confirmed absent from the apt logs on both Linux legs — zig bundles its own musl-built libc++
    so no system clang is required on the musl leg; the gnu leg passed attempt-2 with no clang
    installed, confirming clang was never needed on that leg either. Both musl binaries (`prism`
    and `prism-dtu-demo-server`) are statically linked with no dynamic section. Evidence has been
    appended to `docs/demo-evidence/S-REL-001/fork-tag-dry-run.md` below the attempt-5 record
    per the evidence-capture discipline — prior history preserved.

    **Process note (DRIFT-ORCH-PRLEVEL-PUSH-001):** The feature branch was pushed to origin
    during the attempt-3 evidence commit — earlier than the standard per-story delivery flow's
    push step. This is harmless: the cascade streak was 0/3 at the time of push (no in-progress
    streak existed to reset). Noted here per DRIFT-ORCH-PRLEVEL-PUSH-001 discipline for
    auditability. The streak-reset rule activates only when a streak ≥ 1 is in progress.

13. **Run actionlint** (install via `brew install actionlint` locally — NOT `cargo install
    actionlint` which does not exist; CI installs via direct pinned-tarball + SHA-256,
    see ci.yml release-gate job, F-REL001-P20-003):
    `actionlint .github/workflows/release.yml` — exit code 0 required.

14. **Wire release-gate suite into automated enforcement (F-REL001-P2-001):**
    Add a `test-release-gate` recipe to the `Justfile` that runs `bash tests/release-gate/run.sh`
    and fails closed — if actionlint is absent from the PATH, the AC-011 test file must exit
    non-zero (FAIL), not skip. Add a `release-gate` job/step to `.github/workflows/ci.yml` that:
    (a) installs actionlint via direct pinned-tarball download + SHA-256 verification
    (F-REL001-P20-003 supersession — download-actionlint.bash superseded; NOT brew, NOT
    cargo install), and (b) runs `bash tests/release-gate/run.sh`. This ensures the gate
    enforces automatically on every PR touching `.github/workflows/release.yml`.
    Delete `tests/ci-gate/test_AC-7_homebrew-tap.sh` and `tests/ci-gate/test_AC-8_crates-io-publish.sh`
    (both assert jobs that S-REL-001 removes; retaining them would produce permanent false failures).
    Also modify `tests/ci-gate/test_AC-6_release-artifacts.sh` to update the build assertion for the
    §15 zigbuild conditional — the old single-line grep regresses when `cargo zigbuild` replaces
    `cargo build` on the musl leg; the updated assertion uses a dual-path check (cargo zigbuild
    present on musl leg + cargo build on remaining legs, both --locked) (F-REL001-P14-002).

15. **Apply §14 Option B deltas (DEFECT-REL001-MUSL-DBUS-001):**
    - Read `delta-analysis.md §14` in full before editing.
    - Modify `crates/prism-credentials/Cargo.toml`: implement target-conditional keyring feature
      split per §14 Delta B-1. Move the dbus Secret Service dependency under a
      `[target.'cfg(not(target_env = "musl"))'.dependencies]` (or equivalent `cfg`) block so
      that the musl target resolves to keyutils-only and the glibc target retains the dbus
      Secret Service path.
    - Modify `crates/prism-credentials/src/lib.rs`: add the comment-only two-place-invariant
      note per §14 Delta B-2 (documents the musl/gnu split at the conditional-compilation site).
    - Run local pre-flight verification proxies (both must pass before pushing the dry-run tag):
      - `cargo tree --target x86_64-unknown-linux-musl -i libdbus-sys` → MUST return empty
        (libdbus-free; if non-empty, the target-conditional split is not effective)
      - `cargo tree --target x86_64-unknown-linux-gnu -i libdbus-sys` → MUST return
        `libdbus-sys vX.Y.Z` (retained on glibc; if empty, the split was over-aggressive)
      - `just iter prism-credentials` → MUST pass (all tests green on host target)

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
binary is wrapped before upload-artifact using per-OS conditional logic: `.tar.gz` (tar,
preserves +x bit) on Unix legs; `.zip` (7z, `.exe` suffix) on Windows. Uploaded as artifact
`prism-dtu-demo-server-${{ matrix.target }}` with path
`prism-dtu-demo-server-${{ matrix.target }}.${{ matrix.archive_ext }}`. All 5 matrix targets
produce this artifact. prism-dtu-demo-server is stripped on Unix legs (alongside `prism`).
(traces to architect U13 adjudication: "one cargo invocation, demo-server tar-wrapped"; ADJ-003:
per-OS conditional wrap + strip)

### AC-010: Linux setup step installs musl-tools and pkg-config
Given: The modified build-release matrix job.
When: The Linux-gated setup step is inspected.
Then: `libdbus-1-dev` is installed unconditionally alongside `musl-tools` and `pkg-config` on
both Linux legs, with a comment citing ADR-034/BC-2.06.003 and the build.rs host-linkage
rationale. Secondary probes (all three required): (a) `cargo tree --target
x86_64-unknown-linux-gnu -i libdbus-sys` returns `libdbus-sys vX.Y.Z` (retained on glibc via
§14 Option B target-conditional split); (b) `cargo tree --target x86_64-unknown-linux-musl -i
libdbus-sys` returns EMPTY output (libdbus-free — Option B target-conditional split eliminates
libdbus-sys from the musl dependency tree; DEFECT-REL001-MUSL-DBUS-001); (c) the musl binary
artifact passes `readelf -d | grep libdbus` (zero matches) and `file` reports "statically
linked" (artifact-level linkage proof per §14 five-check table, Task 12).
(traces to architect ADJ-001 ruling: libdbus-1-dev REQUIRED unconditionally on both Linux legs;
ADR-034/BC-2.06.003; §14 Option B: musl = keyutils-only)

### AC-011: Workflow YAML parses without errors (actionlint)
Given: The modified `.github/workflows/release.yml`.
When: `actionlint .github/workflows/release.yml` is run (installed via `brew install actionlint`
locally; CI: direct pinned-tarball + SHA-256 — see ci.yml release-gate job, F-REL001-P20-003;
NOT cargo install, which does not work).
Then: Exit code 0. Zero errors reported.
(traces to delta-analysis.md §8: "manual test tag push gate before RC-1"; research U4: actionlint
is Go, not Rust — `cargo install actionlint` is INVALID)

### AC-012: Release-gate suite wired into automated enforcement (fail-closed)
Given: The story branch with the modified `.github/workflows/release.yml`.
When: The `Justfile` and `.github/workflows/ci.yml` are inspected.
Then: A `test-release-gate` Justfile recipe exists that runs `bash tests/release-gate/run.sh`;
a `ci.yml` step installs actionlint via direct pinned-tarball download + SHA-256 verification
(F-REL001-P20-003 supersession — download-actionlint.bash superseded) and runs
`bash tests/release-gate/run.sh`; and the AC-011 test file exits non-zero (FAIL) when actionlint
is absent from the PATH — the gate is fail-closed, not skip-on-missing.
(traces to F-REL001-P2-001; POL-34 fail-loud)

---

## Previous Story Intelligence

N/A — this is the first story in the E-REL epic. The release workflow was scaffolded with
`S-0.01` (greenfield phase) but never exercised against a real tag. The five defects identified
by delta-analysis §3 accumulated during development.

---

## Architecture Compliance Rules

| Rule | Source | Enforcement |
|------|--------|-------------|
| No `cargo publish` for any workspace crate | All workspace crates carry `publish = false` | Absence of any `cargo publish` in release.yml |
| 5-platform matrix is non-negotiable | ADR-022 boot contract | Matrix must remain unchanged |
| OIDC attestation with v4.1.1 pin | Supply chain security | attest-build-provenance@v4.1.1 + SHA pin |
| --prerelease via bash array, not quoted-empty var | U3: gh no auto-detect | Array form prevents empty positional arg |
| Actions expression injection guard (CWE-78) | F-REL001-P1-001 / CWE-78 | NEVER interpolate ${{ github.ref_name }} or ref-derived expressions inside run: script source; bind via env: map |
| actionlint via brew (local) or direct pinned-tarball+SHA-256 in CI (F-REL001-P20-003), NOT cargo install | U4: actionlint is Go | `brew install actionlint` local; CI: see ci.yml release-gate job |
| build-release builds prism-bin + demo-server together | U13 architect adjudication | Single cargo invocation; tar-wrap demo-server |
| musl-tools + pkg-config for Linux targets | U2: musl cross-compile | Linux-gated apt-get step |
| upload-artifact@v7 + download-artifact@v8 | U5: current majors; interop verified by workflow's own build-release→publish-release flow (F-REL001-P7-002) | Origin test-tag dry-run (task 12) exercises the full upload→download path |
| macos-13 RETIRED | U5: brownout Nov 2025, hard-fail Dec 2025 | Use macos-15-intel; note Aug 2027 EOL |

---

## Library & Framework Requirements

| Tool | Version | Source |
|------|---------|--------|
| `gh` CLI | 2.96.0 (on ubuntu-latest) | GitHub-hosted runner (pre-installed) |
| actionlint | ≥ 1.7.12 (verify latest on releases page) | `brew install actionlint` (local); CI: direct pinned-tarball + SHA-256 — see ci.yml release-gate job (F-REL001-P20-003; download-actionlint.bash superseded) |
| `actions/checkout` | v6.0.2 — resolve SHA via git ls-remote | Research U5 |
| `actions/upload-artifact` | v7 — resolve SHA via git ls-remote | Research U5 |
| `actions/download-artifact` | v8 — resolve SHA via git ls-remote | Research U5 |
| `actions/attest-build-provenance` | v4.1.1 — resolve SHA via git ls-remote | Research U5 (NOT v4.1.0) |
| `dtolnay/rust-toolchain` | @stable (moving tag) — resolve SHA via git ls-remote | Research U20 |
| `musl-tools`, `pkg-config` | ubuntu-24.04 apt packages | Research U2 |
| `libdbus-1-dev` | ubuntu-24.04 apt package | ADR-034/BC-2.06.003 |
| `arduino/setup-protoc` | v3.0.0 — SHA-pinned, mirror ci.yml | DEFECT-REL001-PROTOC-MISSING-001 (dry-run catch; prism-ocsf prost-build requires protoc at compile time on all 5 legs) |
| `clang` / `clang++` | ubuntu-24.04 apt — **SUPERSEDED by §15; NOT installed (F-REL001-P16-001)** | musl-leg CXX override SUPERSEDED; system clang++ links glibc-built `libstdc++.a` → 117 undefined glibc-only refs at link time (DEFECT-REL001-MUSL-LIBSTDCXX-001); see §15 cargo-zigbuild rows below. **Empirical basis (F-REL001-P16-001):** dry-run attempt-2 gnu leg passed with no clang installed; zig bundles its own musl-built libc++ for the musl leg — system clang is not required by either Linux leg. `clang` has been removed from the apt-get install line. |
| `ziglang` | `0.16.0` (pip, `--require-hashes`, `.github/workflows/requirements-musl-ci.txt` hash-pinned per §15-2) | §15 — zig's musl-built libc++ replaces glibc-built `libstdc++.a` on the musl cross-compile leg |
| `cargo-zigbuild` | `0.23.0` (`cargo install --locked`) | §15 — `cargo zigbuild` replaces `cargo build` on the musl leg; cache step required |

---

## File Structure Requirements

| File | Action | Notes |
|------|--------|-------|
| `.github/workflows/release.yml` | Modify | Remove 4 dead jobs; add prerelease; update matrix; Linux setup; upload installs; tar-wrap demo-server |
| `Justfile` | Modify | Add `test-release-gate` recipe that runs `bash tests/release-gate/run.sh`; fails closed on missing actionlint (F-REL001-P2-001) |
| `.github/workflows/ci.yml` | Modify | Add release-gate job/step: install actionlint via direct pinned-tarball + SHA-256 (F-REL001-P20-003; download-actionlint.bash superseded), run `bash tests/release-gate/run.sh` (F-REL001-P2-001 enforcement wiring) |
| `tests/release-gate/` | Create | Authoritative TAP suite for release.yml (AC-001..AC-011); executable test files one-per-AC |
| `tests/ci-gate/test_AC-3_matrix-5-platforms.sh` | Modify | runner-label refresh macos-13→macos-15-intel (stale assertion vs ci.yml matrix; supersession per research U5 macos-13 retirement; orchestrator-adjudicated in-scope, F-REL001-P5-002/F-REL001-P6-001) |
| `tests/ci-gate/test_AC-7_homebrew-tap.sh` | Delete | Superseded — S-REL-001 removes the homebrew-update job; this S-0.01 assertion no longer applies |
| `tests/ci-gate/test_AC-8_crates-io-publish.sh` | Delete | Superseded — S-REL-001 removes the crates-io-publish job; this S-0.01 assertion no longer applies |
| `tests/ci-gate/run.sh` | Modify | Harness hardening swept from the release-gate suite per TD-VSDD-060/POL-34: TAP plan reconciliation, non-zero-exit guard, distinct SKIP accounting, positive-coverage floor. Fail-open→fail-loud behavior change sanctioned by F-REL001-P2-002 (in-scope expansion). |
| `tests/ci-gate/README.md` | Modify | Supersession notes for deleted test_AC-7/test_AC-8 + pointer to tests/release-gate/ as authoritative suite for release.yml + scope-claim correction. |
| `tests/ci-gate/tap_lib.sh` | Modify | set-u safety (${2:-}), grep -- option-terminator hardening, unused require_cmd removal — harness sweep per TD-VSDD-060/POL-29 (F-REL001-P4-001) |
| `crates/prism-credentials/Cargo.toml` | Modify | target-conditional keyring feature split: musl target = keyutils-only (no dbus); gnu target retains dbus Secret Service via `[target.'cfg(...)'.dependencies]` — DEFECT-REL001-MUSL-DBUS-001, §14 Delta B-1 |
| `crates/prism-credentials/src/lib.rs` | Modify | comment-only two-place-invariant note update documenting the musl/gnu feature split — §14 Delta B-2 |
| `.github/workflows/requirements-musl-ci.txt` | Create | hash-pinned ziglang==0.16.0 wheel per §15 delta 15-2 |
| `tests/ci-gate/test_AC-6_release-artifacts.sh` | Modify | AC-6 build assertion updated for the §15 zigbuild conditional (dual-path check: cargo zigbuild + cargo build, both --locked); fixes the regression the zigbuild conditional introduced against the old single-line grep (F-REL001-P14-002) |
| `docs/demo-evidence/S-REL-001/fork-tag-dry-run.md` | Create | Task-12 dry-run evidence (6 attempts committed; attempt-6 GREEN (clang-removal re-verification; run 29721841906); mandated deliverable per Task 12 / F-REL001-P12-OBS-002) |

---

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|----------------|
| GitHub Actions release workflow | `.github/workflows/release.yml` | N/A (CI YAML, not Rust) |
| prism-credentials feature config | `crates/prism-credentials/Cargo.toml` | N/A (Cargo.toml config change; no purity boundary — §14 Delta B-1) |
| prism-credentials lib comment | `crates/prism-credentials/src/lib.rs` | N/A (comment-only update; no behavioral change — §14 Delta B-2) |

---

## Purity Classification

| Module | Classification | Justification |
|--------|----------------|---------------|
| `.github/workflows/release.yml` | N/A | YAML CI configuration — no Rust purity boundary applies |
| `crates/prism-credentials/Cargo.toml` | N/A | Cargo.toml config change only — no purity boundary change (§14 Option B target-conditional feature split; musl = keyutils-only, gnu retains dbus) |
| `crates/prism-credentials/src/lib.rs` | N/A | Comment-only update — no behavioral or purity change (§14 Delta B-2 two-place-invariant note) |

---

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Tag `v1.0.0-rc.1` pushed | Workflow runs; `--prerelease` flag set via array; release created as prerelease |
| EC-002 | Tag `v1.0.0` pushed (GA) | Workflow runs; no `--prerelease`; release created as GA |
| EC-003 | Tag `v2.0.0-rc.1` pushed | `*-*` pattern matches; release created as prerelease |
| EC-004 | All 5 matrix runners pass | Single release with 5 archives + checksums + attestation created |
| EC-005 | v7 upload + v8 download interop | Verified by the workflow's own build-release→publish-release artifact flow, exercised end-to-end by the origin test-tag dry-run gate (task 12); no separate smoke job required (F-REL001-P7-002) |
| EC-006 | musl target build (libdbus-sys C-linked at build time) | `libdbus-1-dev` installed on host; build succeeds. musl binary does NOT dynamically link libdbus at runtime (kernel-keyutils path only). Build fails if `libdbus-1-dev` is absent from runner. |
| EC-007 | Intel macOS build on macos-15-intel | Builds succeed; EOL Aug 2027 migration noted in workflow comment |
| EC-008 | Windows demo-server wrap (.exe + 7z) | `7z a` wraps `prism-dtu-demo-server.exe` into `.zip`; artifact path uses `${{ matrix.archive_ext }}`; `build-demo-bundle` (S-REL-004) downloads and extracts `.zip` on the Windows leg. |
| EC-010 | Release build without protoc in the runner environment | prost-build fails compiling OCSF .proto files (exit 101, all legs); prevented by the arduino/setup-protoc@v3.0.0 step added to build-release on ALL matrix targets; asserted in the release-gate suite (DEFECT-REL001-PROTOC-MISSING-001) |
| EC-009 | Workflow re-run on an already-released tag (triggers: (a) publish-step transient failure after `gh release create` succeeded, (b) manual full re-run of an already-successful workflow, (c) tag re-push when release exists — NOT failed matrix leg, which routes re-run via CREATE path per `needs: build-release`) | `gh release view "$TAG"` detects the existing release; assets re-uploaded with `gh release upload --clobber`; no manual release deletion required. Prerelease status persists on the existing release object — no `--prerelease` flag needed on the upload path. RELEASING.md (S-REL-005) documents the three re-run triggers (F-REL001-P9-001/F-REL001-P10-002). |
| EC-012 | musl cross-compile without a musl-capable C++ compiler | cc-rs `ToolNotFound` exit 101 for `x86_64-linux-musl-g++` (musl-tools ships only the C compiler, not g++); v0.16 fix (`clang` + `CXX_x86_64_unknown_linux_musl=clang++`) resolved the compile step but attempt-4 confirmed link failure: system clang++ links glibc-built `libstdc++.a` → 117 undefined glibc-only refs (DEFECT-REL001-MUSL-LIBSTDCXX-001); v0.16 clang++ approach SUPERSEDED by §15 cargo-zigbuild (DEFECT-REL001-MUSL-CXX-001) |
| EC-013 | musl link failure: glibc-built libstdc++.a contamination | system clang++ links glibc-built `libstdc++.a`; 117 undefined glibc-only symbol refs remain in the musl binary after compile succeeds (DEFECT-REL001-MUSL-LIBSTDCXX-001); prevented by §15 cargo-zigbuild — zig ships musl-built libc++ which carries no glibc deps; artifact gate (`readelf -d \| grep NEEDED` zero matches; `file` reports "statically linked") covers BOTH `prism` AND `prism-dtu-demo-server` musl binaries |
| EC-011 | musl cross-compile with dbus feature active (pre-Option B state) | pkg-config cross-mode refusal on the musl leg: libdbus-sys build.rs calls pkg-config, which refuses to run in cross-compilation mode by default, causing exit 101. `PKG_CONFIG_ALLOW_CROSS=1` (Option A) was rejected as ABI-unsound by architect adjudication (§14) — it would instruct the linker to link the host's glibc-linked libdbus into a musl binary, producing an ABI-unsound artifact that passes CI but crashes at runtime. Prevented by §14 Option B target-conditional feature split: musl target resolves to keyutils-only path, eliminating libdbus-sys from the musl dependency tree entirely. A passing glibc build does not prove musl correctness; artifact-level linkage proof (readelf -d \| grep libdbus = zero matches; file reports "statically linked") is the authoritative gate. |

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
| 0.24 | 2026-07-20 | PR-LEVEL pass-6 fix-burst (F-REL001-PR6-001 LOW): actionlint install mechanism reconciled across all spec sites — download-actionlint.bash superseded by direct pinned-tarball + SHA-256 in CI (F-REL001-P20-003); brew primary kept for local dev; "see ci.yml release-gate job" pointer added as drift-resistant convention. Sites updated: risk_mitigations, Task 13, Task 14, AC-011, AC-012, Architecture Compliance Rules, Library & Framework Requirements. POL-25 sweep complete. |
| 0.23 | 2026-07-20 | PR-LEVEL pass-1 fix-burst: F-REL001-PR1-001 ./dist/* glob corrected — Task 10 snippet trimmed per F-REL001-P16-002 convention (asset list deferred to release.yml; `<asset-paths>` placeholder in create/upload normative text); F-REL001-PR1-002 set-u-safe array splice `${PRERELEASE_ARGS[@]+"${PRERELEASE_ARGS[@]}"}` reflected in Task 10 snippet and normative text. POL-32 |
| 0.22 | 2026-07-20 | pass-20 fix-burst: F-REL001-P20-002 attempt-6 status reconciled to GREEN — all 5 legs + publish PASS, both musl binaries statically linked, no dynamic section (run 29721841906; clang absent from apt logs on both Linux legs; evidence committed); Task 12 attempt-6 paragraph rewritten from to-do voice to completed record; FSR evidence-file row updated (5→6 attempts committed; attempt-6 GREEN). Case-insensitive guard hardening + actionlint binary pin + guard-message precision handled worktree-side under F-REL001-P20-001/003/004. POL-32 |
| 0.21 | 2026-07-20 | pass-16 fix-burst: F-REL001-P16-001 clang removed from Task 7 apt-get line — spec/code/test reconciled; Library & Framework clang row empirical basis appended (gnu leg passed attempt-2 with no clang; zig bundles musl-built libc++); AC-010 enumeration already clang-free (no change); attempt-6 re-verification noted in Task 12; F-REL001-P16-002 §15 install-step snippet replaced with normative summary + pointer to release.yml as implementation-of-record (Cache→Install order, pip3, --locked before --version, cache-hit skip guard documented). POL-32 |
| 0.20 | 2026-07-20 | pass-15 fix-burst: F-REL001-P15-002 cache-step snippet aligned to shipped implementation (name "Cache cargo-zigbuild binary", path ~/.cargo/bin/cargo-zigbuild only, key cargo-zigbuild-0.23.0-${{ runner.os }}; zig not cached — hash-pinned pip install per run); F-REL001-P15-001/003 closed evidence-side. POL-32 |
| 0.19 | 2026-07-20 | pass-14 fix-burst: F-REL001-P14-002 FSR rows for test_AC-6 (dual-path cargo zigbuild + cargo build assertion) + docs/demo-evidence/S-REL-001/fork-tag-dry-run.md (Task-12 evidence file); Task-14 anchor sentence for test_AC-6 modification; F-REL001-P14-001 gnu-persistence regression guard codified in cross-target C-dep audit risk_mitigation (compile_error! guard alone cannot detect linux-native-sync-persistent removal). POL-32 |
| 0.18 | 2026-07-20 | path correction: requirements file cited as implemented — .github/workflows/requirements-musl-ci.txt; FSR row added (Create, hash-pinned ziglang==0.16.0 wheel per §15 delta 15-2); POL-32 |
| 0.17 | 2026-07-20 | attempt-4 fix-burst: DEFECT-REL001-MUSL-LIBSTDCXX-001 — §15 cargo-zigbuild ratified w/ pins (ziglang==0.16.0 hash-pinned, cargo-zigbuild 0.23.0 --locked, cache); clang++ CXX override SUPERSEDED by §15; EC-013 musl link w/ glibc-built libstdc++ → 117 undefined glibc refs; dual-binary artifact gate (BOTH prism + prism-dtu-demo-server musl binaries in checks 2/3); risk_mitigations libstdc++ lesson appended; POL-32 |
| 0.16 | 2026-07-20 | dry-run attempt-3 fix-burst: DEFECT-REL001-MUSL-CXX-001 — clang++ C++ cross-compiler for librocksdb-sys on musl leg (install clang; CXX_x86_64_unknown_linux_musl=clang++; run https://github.com/drbothen/prism/actions/runs/29712784282); EC-012 musl-no-cxx edge case; clang/clang++ library row; cross-target C-dep audit risk_mitigation extended (C++ deps + cc-rs CXX env probe); toolchain-selection rationale flagged for adversary scrutiny; branch-push-at-attempt-3 DRIFT-ORCH-PRLEVEL-PUSH-001 note in Task 12; artifact linkage gate is correctness backstop; POL-32 |
| 0.15 | 2026-07-19 | dry-run attempt-2 fix-burst: DEFECT-REL001-MUSL-DBUS-001 — §14 Option B target-conditional feature split in prism-credentials (musl = keyutils-only; gnu retains libdbus-sys via [target.cfg]); crates_touched +prism-credentials; EC-011 musl dbus cross-mode refusal; five-check attempt-3 verification table in Task 12 (build exit 0, readelf no libdbus NEEDED, file statically linked BLOCKING, cargo tree musl empty, cargo tree gnu retains libdbus-sys); cross-target C-dep audit codified in risk_mitigations; prism-credentials rows in Architecture Mapping + Purity tables; task 15 (apply §14 Option B deltas); AC-010 + Linux-cross-compile mitigation updated for Option B; POL-32 |
| 0.14 | 2026-07-19 | dry-run fix-burst: DEFECT-REL001-PROTOC-MISSING-001 — setup-protoc step added to build-release on ALL legs (mirror ci.yml arduino/setup-protoc@v3.0.0); EC-010 build-without-protoc edge case; build-environment-parity risk_mitigation; Task 7 setup-protoc bullet; Task 12 re-run required (first dry-run https://github.com/drbothen/prism/actions/runs/29709483646 failed all 5 legs); POL-32 |
| 0.13 | 2026-07-19 | task-12 dry-run destination amendment: fork infeasible for repo owner (drbothen cannot fork drbothen/prism — GitHub restriction, latent assumption defect in research U2); HUMAN-approved (2026-07-19) origin test-tag procedure (push v0.0.1-rc.test directly to drbothen/prism, tag only, no branch refs); mandatory cleanup (delete release + tag from origin after evidence capture); transient public prerelease window accepted with mandatory cleanup; evidence file retains historical name fork-tag-dry-run.md with destination rationale note; RELEASING.md runbook must document origin procedure incl. cleanup; POL-25 fork-reference sweep (task-12, risk_mitigations, EC-005, Architecture Compliance Rules); POL-32 |
| 0.12 | 2026-07-19 | LOCAL pass-12 fix-burst: F-REL001-P12-OBS-002 [process-gap LOW] — Task 12 fork-tag dry-run gate amended to require captured, preserved evidence (fork workflow run URL, per-leg conclusions for all 5 targets, release-asset listing) in docs/demo-evidence/S-REL-001/fork-tag-dry-run.md and cited in RELEASING.md (S-REL-005); POL-32 |
| 0.11 | 2026-07-19 | LOCAL pass-10 fix-burst: F-REL001-P10-002 idempotency trigger rationale corrected across EC-009/task-10/risk_mitigations (publish-release needs: build-release → failed matrix leg → CREATE path; --clobber path reachable only via 3 triggers: publish-step transient failure after create succeeded, manual full re-run, or tag re-push); F-REL001-P10-001 load-bearing-logic-needs-assertion discipline codified in risk_mitigations fail-closed entry |
| 0.10 | 2026-07-19 | LOCAL pass-9 fix-burst: F-REL001-P9-001 idempotent release step — task 10 view→upload-clobber guard + RELEASING.md (S-REL-005) linkage, EC-009 re-run recovery row, risk_mitigations release-step-idempotency entry |
| 0.9 | 2026-07-19 | LOCAL pass-7 fix-burst: F-REL001-P7-002 task-11/EC-005 interop mechanism clarified (build-release→publish-release flow exercised by fork-tag dry-run; no separate smoke job); F-REL001-P7-001 exact-count floor codified in red_gate_tests comment + fail-closed risk_mitigations entry |
| 0.8 | 2026-07-19 | LOCAL pass-6 fix-burst: F-REL001-P6-001 — tests/ci-gate/test_AC-3_matrix-5-platforms.sh Modify row added to File Structure Requirements (runner-label refresh macos-13→macos-15-intel; orchestrator-adjudicated in-scope, same supersession class as test_AC-7/AC-8 deletions) |
| 0.7 | 2026-07-19 | LOCAL pass-4 fix-burst: F-REL001-P4-001 — tap_lib.sh Modify row added to File Structure Requirements (set-u safety (${2:-}), grep -- option-terminator hardening, unused require_cmd removal; TD-VSDD-060/POL-29 sibling-sweep) |
| 0.6 | 2026-07-19 | LOCAL pass-3 fix-burst: F-REL001-P3-002 — File Structure Requirements completed with ci-gate harness modification rows (tests/ci-gate/run.sh Modify, tests/ci-gate/README.md Modify); tests/release-gate/README.md covered by existing tests/release-gate/ Create row (no separate row needed) |
| 0.5 | 2026-07-19 | LOCAL pass-2 fix-burst: F-REL001-P2-001 enforcement wiring — File Structure Requirements expanded (Justfile modify, ci.yml release-gate step, tests/release-gate/ create, 2 tests/ci-gate/ delete); Task 14 wiring added; AC-012 added (acceptance_criteria_count 11→12); F-REL001-P2-002 fail-loud harness codified in risk_mitigations; red_gate_tests comment updated (AC-012 verified by inspection + CI run) |
| 0.4 | 2026-07-19 | LOCAL pass-1 fix-burst: F-REL001-P1-001 spec-side — rewrite Task 10 snippet to env:-binding idiom (CWE-78 expression injection prevention); add CWE-78 risk_mitigations entry + Architecture Compliance Rules row; F-REL001-P1-004 — red_gate_tests 4→11 with inline semantic comment (11 AC test files; 9 RED at gate, AC-006/007 already green on develop) |
| 0.3 | 2026-07-19 | Pre-TDD fix-burst applying ADJ-001..004 per delta-analysis.md §13: ADJ-001 libdbus-1-dev REQUIRED unconditionally on both Linux legs (ADR-034/BC-2.06.003); ADJ-002 install-script upload step moved to S-REL-003; ADJ-003 per-OS demo-server wrap (.tar.gz Unix / .zip Windows + strip); ADJ-004 remove hardcoded crate count; acceptance_criteria_count 12→11 |
| 0.2 | 2026-07-19 | Fix-burst: U1 typo; U2 Linux setup musl-tools+pkg-config+fork-tag dry-run; U3 bash-array prerelease (no gh auto-detect); U4 actionlint is Go not cargo; U5 attest v4.1.0→v4.1.1+macos-13 retired+upload v7/download v8+smoke-test; U13 build-release builds demo-server+tar-wrap; U20 SHA-pinning tasks; U23 artifact name release-$target; U26 install scripts uploaded as release assets; acceptance_criteria_count 9→12 |
| 0.1 | 2026-07-19 | Initial story creation (story-writer F3 burst) |
