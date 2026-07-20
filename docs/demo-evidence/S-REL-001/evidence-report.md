# Evidence Report — S-REL-001
## devops: release.yml repair

**Story version:** v0.22
**Branch:** feature/S-REL-001
**HEAD at evidence capture:** 384d520e
**Date:** 2026-07-20
**Produced by:** vsdd-factory:demo-recorder

---

## Evidence Inventory

| Artifact | File | Description |
|----------|------|-------------|
| TAP recording (.tape) | `TAP-001-release-gate-suite.tape` | VHS script: runs `bash tests/release-gate/run.sh` |
| TAP recording (.gif) | `TAP-001-release-gate-suite.gif` | Terminal recording of full 81/81 TAP suite |
| TAP recording (.webm) | `TAP-001-release-gate-suite.webm` | Archival recording of full 81/81 TAP suite |
| Dry-run evidence | `fork-tag-dry-run.md` | Task-12 dry-run gate (6 attempts; attempt-6 GREEN) — DO NOT MODIFY |

---

## AC Coverage Table

| AC | Description | Executable Proof | Dry-Run Runtime Proof | Status |
|----|-------------|------------------|-----------------------|--------|
| AC-001 | DEF-REL-001 closed — binary_exists guard removed | test_AC-001_binary-exists-removal.sh (5 assertions, all PASS) | Attempt-6 all 5 legs exit 0; no `if: steps.check_binary` guard in job logs | PASS |
| AC-002 | DEF-REL-002 closed — chocolatey-publish removed | test_AC-002_chocolatey-removal.sh (5 assertions, all PASS) | Attempt-6 — no `chocolatey-publish` job in workflow run | PASS |
| AC-003 | DEF-REL-003 closed — homebrew-update removed | test_AC-003_homebrew-removal.sh (5 assertions, all PASS) | Attempt-6 — no `homebrew-update` job in workflow run | PASS |
| AC-004 | DEF-REL-004 closed — crates-io-publish removed | test_AC-004_crates-io-removal.sh (4 assertions, all PASS) | Attempt-6 — no `crates-io-publish` job in workflow run | PASS |
| AC-005 | Prerelease flag via bash array; no gh auto-detection | test_AC-005_prerelease-flag.sh (14 assertions, all PASS) — includes assertion #9 "set-u-safe splice guard full form present — F-REL001-PR2-OBS-1/SID-2" | Attempt-6: `isPrerelease: true` (dry-run tag v0.0.1-rc.test matches `*-*` pattern); idempotency re-run path also present in workflow | PASS |
| AC-006 | 5-platform matrix preserved and correctly spelled | test_AC-006_matrix-targets.sh (9 assertions, all PASS) | Attempt-6: 5 matrix legs all ran — gnu, musl, aarch64-darwin, windows, x86_64-darwin | PASS |
| AC-007 | SHA-256 checksums step preserved | test_AC-007_checksums.sh (4 assertions, all PASS) | Attempt-6 release assets: `checksums.txt` (582 bytes) with 5 lines, one per platform | PASS |
| AC-008 | OIDC attestation preserved with v4.1.1 pin | test_AC-008_oidc-attestation.sh (6 assertions, all PASS) | Attempt-6: "Attest build provenance" step succeeded on all 5 legs (5/5 attestations: success) | PASS |
| AC-009 | build-release builds prism-bin AND prism-dtu-demo-server | test_AC-009_demo-server-build.sh (7 assertions, all PASS) | Attempt-6: per-OS artifacts present — 5 `.tar.gz` (Unix) + 1 `.zip` (Windows); musl/gnu static linking confirmed; `prism-dtu-demo-server` stripped on Unix legs | PASS |
| AC-010 | Linux setup installs musl-tools, pkg-config, libdbus-1-dev | test_AC-010_linux-setup.sh (20 assertions, all PASS) | Attempt-6 apt log: `libdbus-1-dev musl musl-dev musl-tools` (4 packages, no clang); gnu tree retains `libdbus-sys v0.2.7`; musl tree: empty (libdbus-free); both musl binaries: "statically linked", no dynamic section | PASS |
| AC-011 | Workflow YAML parses without errors (actionlint) | test_AC-011_actionlint.sh (2 assertions, all PASS); direct: `actionlint .github/workflows/release.yml` exit 0, zero output | N/A (static analysis) | PASS |
| AC-012 | Release-gate suite wired into automated enforcement | Justfile `test-release-gate` recipe present; ci.yml `release-gate` job present with `bash tests/release-gate/run.sh`; verify-workflow-structure guard (F-REL001-P3-OBS-002, F-REL001-P11-001) enforces both at CI time | N/A (wiring proof by inspection + CI run) | PASS |

**Total: 12/12 ACs covered. 81/81 TAP assertions passing.**

---

## TAP Suite Full Output (AC-001..AC-011)

Captured live on HEAD 384d520e via `bash tests/release-gate/run.sh`:

```
TAP version 13
# S-REL-001 Release Gate — Red Gate validation suite
# release.yml repair (AC-001..AC-011)
# Running from: /Users/jmagady/Dev/prism/.worktrees/S-REL-001/tests/release-gate

# --- test_AC-001_binary-exists-removal.sh ---
ok 1 - AC-001: file exists: release.yml
ok 2 - AC-001: 'binary_exists' correctly absent from release.yml
ok 3 - AC-001: 'check_binary' correctly absent from release.yml
ok 4 - AC-001: 'steps.check_binary.outputs.binary_exists == 'true'' correctly absent from release.yml
ok 5 - AC-001: 'needs.build-release.outputs.binary_exists' correctly absent from release.yml
1..5

# --- test_AC-002_chocolatey-removal.sh ---
ok 1 - AC-002: file exists: release.yml
ok 2 - AC-002: 'chocolatey-publish:' correctly absent from functional lines of release.yml
ok 3 - AC-002: 'choco pack' correctly absent from functional lines of release.yml
ok 4 - AC-002: 'choco push' correctly absent from functional lines of release.yml
ok 5 - AC-002: 'nuspec' correctly absent from functional lines of release.yml
1..5

# --- test_AC-003_homebrew-removal.sh ---
ok 1 - AC-003: file exists: release.yml
ok 2 - AC-003: 'homebrew-update:' correctly absent from functional lines of release.yml
ok 3 - AC-003: '1898co/homebrew-tap' correctly absent from functional lines of release.yml
ok 4 - AC-003: 'Formula/prism.rb' correctly absent from functional lines of release.yml
ok 5 - AC-003: 'S-REL-008' found in release.yml
1..5

# --- test_AC-004_crates-io-removal.sh ---
ok 1 - AC-004: file exists: release.yml
ok 2 - AC-004: 'crates-io-publish:' correctly absent from functional lines of release.yml
ok 3 - AC-004: 'cargo publish' correctly absent from functional lines of release.yml
ok 4 - AC-004: 'CARGO_REGISTRY_TOKEN' correctly absent from functional lines of release.yml
1..4

# --- test_AC-005_prerelease-flag.sh ---
ok 1 - AC-005: file exists: release.yml
ok 2 - AC-005: prerelease flag uses safe array or ${VAR:+} form
ok 3 - AC-005: tag-contains-dash detection pattern '*-*' present
ok 4 - AC-005: full composed prerelease array form present (SID-2)
ok 5 - AC-005: '"$PRERELEASE_FLAG"' correctly absent from release.yml
ok 6 - AC-005: idempotent guard 'gh release view "$TAG"' present (F-REL001-P10-001)
ok 7 - AC-005: idempotent guard upload path 'gh release upload "$TAG" --clobber' present (F-REL001-P10-001)
ok 8 - AC-005: create path splices '"${PRERELEASE_ARGS[@]}"' into gh release create (SID-2 / F-REL001-P10-001)
ok 9 - AC-005: set-u-safe splice guard full form present (F-REL001-PR2-OBS-1 / SID-2)
ok 10 - AC-005: run: block extraction non-empty (awk state-machine preflight)
ok 11 - AC-005: ${{ github.ref* }} absent from all run: blocks (F-REL001-P1-001 / F-REL001-P18-001)
ok 12 - AC-005: ${{ github.event* }} absent from all run: blocks (F-REL001-P1-001 / F-REL001-P18-001)
ok 13 - AC-005: ${{ github.head_ref }} absent from all run: blocks (F-REL001-P1-001 / F-REL001-P18-001)
ok 14 - AC-005: ${{ env.* }} absent from all run: blocks (F-REL001-P1-001 / F-REL001-P18-001 env-re-exposure vector)
1..14

# --- test_AC-006_matrix-targets.sh ---
ok 1 - AC-006: file exists: release.yml
ok 2 - AC-006: 'aarch64-apple-darwin' found in release.yml
ok 3 - AC-006: 'x86_64-apple-darwin' found in release.yml
ok 4 - AC-006: 'x86_64-unknown-linux-gnu' found in release.yml
ok 5 - AC-006: 'x86_64-unknown-linux-musl' found in release.yml
ok 6 - AC-006: 'x86_64-pc-windows-msvc' found in release.yml
ok 7 - AC-006: 'x86_x64-unknown-linux-musl' correctly absent from release.yml
ok 8 - AC-006: exactly 5 matrix 'target:' entries found (count=5)
ok 9 - AC-006: 'fail-fast: false' found in release.yml
1..9

# --- test_AC-007_checksums.sh ---
ok 1 - AC-007: file exists: release.yml
ok 2 - AC-007: SHA-256 computation command present (sha256sum or shasum -a 256)
ok 3 - AC-007: 'checksums.txt' found in release.yml
ok 4 - AC-007: 'artifacts/release-*/checksums.txt' found in release.yml
1..4

# --- test_AC-008_oidc-attestation.sh ---
ok 1 - AC-008: file exists: release.yml
ok 2 - AC-008: 'id-token: write' found in release.yml
ok 3 - AC-008: 'attest-build-provenance' found in release.yml
ok 4 - AC-008: '# v4.1.1' found in release.yml
ok 5 - AC-008: '# v4.1.0' correctly absent from release.yml
ok 6 - AC-008: attest-build-provenance is SHA-pinned (immutable commit SHA present)
1..6

# --- test_AC-009_demo-server-build.sh ---
ok 1 - AC-009: file exists: release.yml
ok 2 - AC-009: 'prism-dtu-demo-server' found in release.yml
ok 3 - AC-009: '-p prism-bin -p prism-dtu-demo-server' found in release.yml
ok 4 - AC-009: 'prism-dtu-demo-server-${{ matrix.target }}' found in release.yml
ok 5 - AC-009: demo-server artifact named 'prism-dtu-demo-server-${{ matrix.target }}'
ok 6 - AC-009: 'strip target/${{ matrix.target }}/release/prism-dtu-demo-server' found in release.yml
ok 7 - AC-009: demo-server archive path uses per-OS extension (archive_ext conditional)
1..7

# --- test_AC-010_linux-setup.sh ---
ok 1 - AC-010: file exists: release.yml
ok 2 - AC-010: 'libdbus-1-dev' found in release.yml
ok 3 - AC-010: 'musl-tools' found in release.yml
ok 4 - AC-010: 'pkg-config' found in release.yml
ok 5 - AC-010: 'ADR-034/BC-2.06.003' found in release.yml
ok 6 - AC-010: Linux apt step gated on contains(matrix.target, 'linux')
ok 7 - AC-010: 'arduino/setup-protoc@c65c819552d16ad3c9b72d9dfd5ba5237b9c906b # v3.0.0' found in release.yml
ok 8 - AC-010: setup-protoc step runs unconditionally on all 5 matrix legs (no if: gate)
ok 9 - AC-010: 'sudo apt-get install -y musl-tools pkg-config libdbus-1-dev' found in release.yml
ok 10 - AC-010: clang correctly absent from apt-get install line (§15/F-REL001-P16-001 regression guard)
ok 11 - AC-010: 'pip3 install --require-hashes -r .github/workflows/requirements-musl-ci.txt' found in release.yml
ok 12 - AC-010: 'cargo install --locked cargo-zigbuild --version 0.23.0' found in release.yml
ok 13 - AC-010: 'cargo zigbuild --release --locked --target ${{ matrix.target }} -p prism-bin -p prism-dtu-demo-server' found in release.yml
ok 14 - AC-010: file exists: requirements-musl-ci.txt
ok 15 - AC-010: 'ziglang==0.16.0' found in requirements-musl-ci.txt
ok 16 - AC-010: '--hash=sha256:9fcda73f62b851dd72a54b710ad40a209896db14cfb13649e62191243556342b' found in requirements-musl-ci.txt
ok 17 - AC-010: 'CXX_x86_64_unknown_linux_musl=clang++' correctly absent from release.yml
ok 18 - AC-010: target-cfg(linux-gnu) header present as functional line in Cargo.toml
ok 19 - AC-010: keyring linux-native-sync-persistent present in target-cfg(linux-gnu) block (Cargo.toml)
ok 20 - AC-010: 'keyring-linux-native-sync-persistent' correctly absent from default = [...] active entries (Cargo.toml)
1..20

# --- test_AC-011_actionlint.sh ---
ok 1 - AC-011: file exists: release.yml
ok 2 - AC-011: actionlint validated release.yml: 0 findings
1..2

# ========================================
# S-REL-001 Release Gate Summary
# Total:   81
# Passed:  81
# Failed:  0
# Skipped: 0
# ========================================
# All release-gate tests passed.
```

---

## AC-011 Direct actionlint Evidence

```
$ which actionlint
/opt/homebrew/bin/actionlint

$ actionlint .github/workflows/release.yml
(no output)

$ echo $?
0
```

Exit code: **0**. Zero errors. YAML parses cleanly.

---

## AC-012 Wiring Proof

### Justfile test-release-gate Recipe

Lines 89-95 of `Justfile`:

```
# Run the S-REL-001 release-gate TAP test suite (S-REL-001 AC-012, F-REL001-P2-001).
# Fails closed when actionlint is absent from PATH — the AC-011 test exits non-zero
# rather than skipping. Install actionlint locally via: brew install actionlint
# (NOT cargo install actionlint — actionlint is Go, not Rust; research U4).
# Usage: just test-release-gate
test-release-gate:
    @bash tests/release-gate/run.sh
```

`just test-release-gate` output confirms 81/81 pass (same TAP output as above).

### ci.yml release-gate Job Block

Job definition at `release-gate:` in `.github/workflows/ci.yml`:

```yaml
  release-gate:
    name: Release gate (S-REL-001 AC-012)
    # F-REL001-P2-001: automated enforcement for tests/release-gate/ suite on every
    # PR/push. Installs actionlint via direct pinned-tarball download (F-REL001-P20-003;
    # NOT brew — unavailable on ubuntu-latest; NOT cargo install — actionlint is Go,
    # not Rust; research U4). Fails closed when actionlint is absent from PATH
    # (AC-012 fail-closed gate, POL-34). No Rust toolchain needed — lightweight
    # YAML/shell validation only.
    runs-on: ubuntu-latest
    timeout-minutes: 5
    permissions:
      contents: read
    steps:
      - uses: actions/checkout@de0fac2e4500dabe0009e67214ff5f5447ce83dd # v6.0.2
      - name: Install actionlint
        run: |
          # Pin to 1.7.12 — minimum confirmed working version per S-REL-001 risk_mitigations.
          # F-REL001-P20-003 / CWE-494: direct pinned-tarball download with SHA-256 verification
          # of the release asset before extraction.
          EXPECTED_SHA256="8aca8db96f1b94770f1b0d72b6dddcb1ebb8123cb3712530b08cc387b349a3d8"
          ...
      - name: Run release-gate suite
        run: bash tests/release-gate/run.sh
```

### verify-workflow-structure Guard Lines (F-REL001-P3-OBS-002 / F-REL001-P11-001)

The `verify-workflow-structure` step in ci.yml (lines 2082-2113) enforces two guards:

**Guard 1 — release-gate job existence (F-REL001-P3-OBS-002):**
```bash
grep -qE '^  release-gate:' .github/workflows/ci.yml || {
  echo "::error::F-REL001-P3-OBS-002: 'release-gate' job is missing from ci.yml"
  exit 1
}
```

**Guard 2 — scoped run assertion (F-REL001-P3-OBS-002 / F-REL001-P11-001):**
```bash
RELEASE_GATE_BLOCK=$(awk '/^  release-gate:/{f=1;next} /^  [a-z][a-zA-Z-]*:/{f=0} f' .github/workflows/ci.yml)
echo "$RELEASE_GATE_BLOCK" | grep -qE '^\s+bash tests/release-gate/run\.sh\s*$' || {
  echo "::error::F-REL001-P3-OBS-002: 'bash tests/release-gate/run.sh' missing from release-gate job block"
  exit 1
}
```

**Guard 3 — Justfile recipe (F-REL001-P11-001):**
```bash
grep -qE '^test-release-gate:' Justfile || {
  echo "::error::F-REL001-P11-001: 'test-release-gate:' recipe missing from Justfile"
  exit 1
}
```

All three guards pass on current HEAD.

---

## Per-AC Test File Mapping

| Test File | AC | Assertion Count | Proof Type |
|-----------|----|-----------------|------------|
| `test_AC-001_binary-exists-removal.sh` | AC-001 | 5 | YAML grep (absence + build-release outputs: block) |
| `test_AC-002_chocolatey-removal.sh` | AC-002 | 5 | YAML grep (absence — functional lines only) |
| `test_AC-003_homebrew-removal.sh` | AC-003 | 5 | YAML grep (absence + S-REL-008 reference present) |
| `test_AC-004_crates-io-removal.sh` | AC-004 | 4 | YAML grep (absence — cargo publish, CARGO_REGISTRY_TOKEN) |
| `test_AC-005_prerelease-flag.sh` | AC-005 | 14 | YAML grep (array form, idempotency guard, set-u-safe splice guard, CWE-78 injection absence) |
| `test_AC-006_matrix-targets.sh` | AC-006 | 9 | YAML grep (5 targets present, typo absent, exact count=5) |
| `test_AC-007_checksums.sh` | AC-007 | 4 | YAML grep (sha256sum/shasum command, checksums.txt path) |
| `test_AC-008_oidc-attestation.sh` | AC-008 | 6 | YAML grep (id-token:write, v4.1.1 present, v4.1.0 absent, SHA pin present) |
| `test_AC-009_demo-server-build.sh` | AC-009 | 7 | YAML grep (-p prism-bin -p demo-server, per-OS wrap, strip step) |
| `test_AC-010_linux-setup.sh` | AC-010 | 20 | YAML grep + Cargo.toml (libdbus-1-dev, musl-tools, zigbuild, clang absent, prism-credentials target-cfg split) |
| `test_AC-011_actionlint.sh` | AC-011 | 2 | Live actionlint execution on release.yml (exit 0, zero findings) |
| (Justfile + ci.yml inspection) | AC-012 | N/A | verify-workflow-structure guards + just test-release-gate recipe |

**Total executable assertions: 81** (test files 1-11; AC-012 verified by inspection + CI run per story spec red_gate_tests note).

---

## Runtime Proof: Dry-Run Evidence (Task 12 Gate)

Reference: `docs/demo-evidence/S-REL-001/fork-tag-dry-run.md` (DO NOT MODIFY).

**Final green attempt: Attempt 6**
- Run URL: https://github.com/drbothen/prism/actions/runs/29721841906
- Tag: `v0.0.1-rc.test` at commit `339a0c04`
- Tag push time: 2026-07-20T06:29:38Z

### Per-Leg Results (Attempt 6)

| Target | Job ID | Conclusion | Duration |
|--------|--------|------------|----------|
| x86_64-unknown-linux-gnu | 88286303293 | PASS | 24m 5s |
| x86_64-unknown-linux-musl | 88286303308 | PASS | 19m 6s |
| aarch64-apple-darwin | 88286303326 | PASS | 30m 13s |
| x86_64-pc-windows-msvc | 88286303334 | PASS | 6m 42s |
| x86_64-apple-darwin | 88286303337 | PASS | 5m 46s |
| Create GitHub Release (publish-release) | 88290498683 | PASS | 27s |

### Per-AC Dry-Run Citations

**AC-005 (prerelease flag):** `isPrerelease: true` confirmed in `gh release view v0.0.1-rc.test --repo drbothen/prism --json isPrerelease,assets,tagName` output. Tag `v0.0.1-rc.test` matches `*-*` pattern — PRERELEASE_ARGS array populated with `--prerelease`. Idempotency re-run path: `gh release view "$TAG"` guard present in workflow (asserted by AC-005 test assertion #6).

**AC-006 (5-platform matrix):** 5 build-release legs ran — all 5 targets present, no `x86_x64` typo, `fail-fast: false` confirmed.

**AC-007 (checksums):** Release asset listing shows `checksums.txt` (582 bytes). Checksums.txt content: 5 lines, one per platform.

**AC-008 (OIDC attestation):** All 5 `attest-build-provenance` steps reported `success`. 5/5 attestations complete.

**AC-009 (per-OS artifacts including Windows zip + musl static linkage):**
- Windows: `prism-v0.0.1-rc.test-x86_64-pc-windows-msvc.zip` (47,008,966 bytes) in release assets.
- musl binary `prism`: `file` output: `ELF 64-bit LSB executable, x86-64, version 1 (SYSV), statically linked, stripped`.
- musl binary `prism-dtu-demo-server`: `file` output: `ELF 64-bit LSB executable, x86-64, version 1 (SYSV), statically linked, stripped`.
- `readelf -d` both musl binaries: `There is no dynamic section in this file.`

**AC-010 (clang-free apt logs + cargo tree pair):**
- apt log musl leg (job 88286303308): `libdbus-1-dev musl musl-dev musl-tools` — 4 packages, no clang.
- apt log gnu leg (job 88286303293): `libdbus-1-dev musl musl-dev musl-tools` — 4 packages, no clang.
- `cargo tree --target x86_64-unknown-linux-musl -i libdbus-sys`: `warning: nothing to print.` (empty — libdbus-free).
- `cargo tree --target x86_64-unknown-linux-gnu -i libdbus-sys`: `libdbus-sys v0.2.7` (retained on glibc).

**Evidence-representativeness:** Orchestrator verified `git diff 339a0c04..acb718fb -- .github/workflows/release.yml .github/workflows/requirements-musl-ci.txt crates/prism-credentials/` = EMPTY. Attempt-6 GREEN fully represents HEAD 75ce8cbf (commits after the tag touch only tests and evidence files, not the verified subjects).

**Cleanup confirmed:** GitHub Release deleted (`gh api repos/drbothen/prism/releases/356544832 -X DELETE`), remote tag deleted (`gh api repos/drbothen/prism/git/refs/tags/v0.0.1-rc.test -X DELETE`), local tag deleted. All three deletion verifications confirmed clean.

---

## Gates Summary

| Gate | Outcome | Reference |
|------|---------|-----------|
| LOCAL adversarial cascade | CONVERGED — 3-CLEAN @ 75ce8cbf (23 passes, BC-5.39.001); evidence regenerated @ 384d520e (F-REL001-PR8-001) | .factory/STATE.md D-1880 |
| TAP release-gate suite | GREEN — 81/81 @ 384d520e | This report §TAP Suite Full Output |
| actionlint | GREEN — exit 0, zero findings @ 384d520e | This report §AC-011 |
| Task-12 dry-run gate | GREEN — Attempt 6, run 29721841906 (2026-07-20) | fork-tag-dry-run.md §Attempt 6 |
| Story-level holdout gate | HUMAN-DIRECTED WAIVER — D-1880 adjudication (2026-07-20): dry-run gate stands as observed-output evidence for this CI-infra story (no MCP stdio surface; holdout-evaluator cannot drive CI/CD workflow execution against live GitHub Actions) | .factory/STATE.md D-1880 |

---

## VHS Terminal Recording

This is a CI/CD infrastructure story. VHS is present on this machine (`/opt/homebrew/bin/vhs` v0.11.0). A terminal recording of the full TAP suite run was captured:

- `TAP-001-release-gate-suite.tape` — VHS script
- `TAP-001-release-gate-suite.gif` — 2.5M animated GIF (PR embed)
- `TAP-001-release-gate-suite.webm` — 1.5M archival recording

The TAP output itself (81/81) is the primary evidence as specified by the story spec and VSDD demo-recorder constraints for CI-infra stories. The VHS recording supplements it with a visual terminal capture.

---

## Self-Audit Checklist

- [x] Every AC (001-012) has a recorded proof linking to a specific AC.
- [x] Evidence report generated after all recordings complete.
- [x] No source code or test files modified — recording only.
- [x] Output placed in `docs/demo-evidence/S-REL-001/` (story-scoped, POL-10).
- [x] `fork-tag-dry-run.md` not modified — referenced only.
- [x] VHS present: recording produced (TAP-001-release-gate-suite.{tape,gif,webm}).
