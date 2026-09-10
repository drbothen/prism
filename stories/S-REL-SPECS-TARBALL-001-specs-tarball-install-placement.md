---
document_type: story
story_id: S-REL-SPECS-TARBALL-001
title: "devops: specs tarball release asset + install.sh/install.ps1 spec placement for beta.2"
wave: F-A
epic_id: E-REL
priority: P2
status: merged
version: "1.2"
level: "L4"
producer: story-writer
timestamp: "2026-09-09T00:00:00Z"
tdd_mode: facade
# tdd_mode justification:
#   This story modifies scripts/install.sh (bash), scripts/install.ps1 (PowerShell),
#   and .github/workflows/release.yml (GitHub Actions YAML). There is no Rust logic —
#   the todo!() / Red Gate stub pattern does not apply. Quality gates: shellcheck for
#   install.sh (existing CI gate from S-REL-003) + PSScriptAnalyzer for install.ps1
#   (existing CI gate from S-REL-003) + structural verification that prism-specs-*.tar.gz
#   appears in the published release. Pattern matches S-REL-NIGHTLY-001 (facade),
#   S-REL-NIGHTLY-NOTES-001 (facade), and S-REL-DROP-INTEL-MAC-001 (facade).
subsystems: []
# Subsystem anchor justification:
#   Spec tarball packaging and install script placement are distribution tooling.
#   No ARCH-INDEX subsystem owns GitHub Actions packaging jobs or install scripts.
#   subsystems: [] per S-REL-001 and S-REL-003 precedent.
crates_touched: [devops]
target_module: devops
capabilities: []
behavioral_contracts: []
# BC status: N/A — pure CI/CD and distribution tooling. No subsystem behavioral contract
# governs GitHub Actions packaging YAML or checksum-verified install scripts.
# Conforming per S-REL-001, S-REL-003, S-REL-NIGHTLY-001 precedent. POL-14 NO-OP.
# BC status: pending N/A (infra story waiver applies).
verification_properties: []
depends_on: [S-REL-001, S-REL-003]
# Dependency anchor justifications:
#   depends_on S-REL-001: The publish-release job pattern (checksums.txt merge,
#     gh release create/upload, asset URL convention) was established by S-REL-001.
#     This story adds a step to that job; the base must exist.
#   depends_on S-REL-003: install.sh and install.ps1 were authored in S-REL-003.
#     This story extends them with --spec-dir / -SpecDir support. Both are already merged.
blocks: []
# blocks justification:
#   No currently-registered story has a technical dependency on spec tarball delivery.
#   S-REL-010 (embedded built-in specs, post-v1.0.0) will eventually supersede the
#   tarball distribution model, but does not depend on it.
points: 3
estimated_days: 2
risk: LOW
# Risk justification: install.sh and install.ps1 already have checksum download/verify
# infrastructure from S-REL-003. Adding a second artifact follows the same pattern.
# The release.yml publish-release job already creates and uploads multiple assets.
# No new toolchain dependencies; sha256sum and tar are present on all CI runners.
acceptance_criteria_count: 9
red_gate_tests: 0
# SAC-1 note: tdd_mode: facade — no RG-NNN list required. Quality gate is the existing
# CI shellcheck/PSScriptAnalyzer chain plus a structural verification of the release asset.
estimated_passes: "1-2 LOCAL adversary passes"
holdout_scenarios: []
assumption_validations: []
risk_mitigations:
  - "Tarball layout — root-level files (no prefix): The spec tarball MUST contain spec files at
    the root (not under a specs/ prefix) so that `tar -xzf prism-specs-<tag>.tar.gz -C <spec-dir>`
    produces `<spec-dir>/claroty.sensor.toml` directly. Building with
    `tar czf ... -C crates/prism-sensors/specs claroty.sensor.toml` achieves this."
  - "No-clobber idempotency: install.sh and install.ps1 MUST NOT overwrite an existing spec file
    unless --force-specs / -ForceSpecs is passed. A spec file may be user-modified; silent
    overwrite would destroy operator customizations."
  - "checksums.txt ordering: the spec tarball checksum is appended to checksums.txt AFTER
    the per-platform binary checksums are merged. The grep lookup in install scripts searches
    for the exact filename in the file; position is irrelevant."
  - "publish-release job runs on Ubuntu (ubuntu-latest); sha256sum is always available.
    The dual-path (sha256sum / shasum) is needed only in install.sh (macOS/Linux) and
    install.ps1 (Get-FileHash). The publish-release step can use sha256sum directly."
  - "Backward compatibility: --spec-dir is OPTIONAL in install.sh. Without it, the script
    still installs the binary successfully and prints instructions for obtaining specs.
    Operators who re-run the script without --spec-dir after upgrading must not see a
    regression."
  - "Windows .zip archive already contains specs: the per-platform .zip for Windows
    (x86_64-pc-windows-msvc) already includes specs/claroty.sensor.toml (per current release.yml
    'Create archive' step). install.ps1 currently ignores it. The new -SpecDir parameter
    sources from the separate prism-specs-<tag>.tar.gz, not the .zip, to keep the
    per-platform archives and the separate specs tarball as independent channels."
inputs:
  - ".github/workflows/release.yml"
  - "scripts/install.sh"
  - "scripts/install.ps1"
  - "crates/prism-sensors/specs/claroty.sensor.toml"
  - "prism.toml.example"
  - "crates/prism-bin/src/boot.rs"
input-hash: "31cde6b"
# input-hash: populated by state-manager after commit (compute-input-hash).
traces_to: []
cycle: "v1.0.0-beta.2"
phase: "F3"
---

# S-REL-SPECS-TARBALL-001 — Specs Tarball Release Asset + Install Script Spec Placement

**Story ID:** S-REL-SPECS-TARBALL-001
**Status:** merged v1.2
**Target release:** v1.0.0-beta.2

---

## Background and Reconciliation Findings

This story closes a gap in the beta.2 release chain. The four elements of spec distribution
must be consistent for `prism` to discover its sensor specs after installation:

| Element | Current State | Required State | Change Type |
|---------|---------------|----------------|-------------|
| **Spec source** | `crates/prism-sensors/specs/claroty.sensor.toml` (authoritative) | Same — unchanged | None |
| **Release asset** | Specs bundled INSIDE each per-platform binary archive (`specs/claroty.sensor.toml`). No separate platform-neutral tarball exists. | Add a separate `prism-specs-<tag>.tar.gz` with specs at root | `release.yml` change (script-only) |
| **Install placement** | install.sh/ps1 extract the binary only; specs are silently skipped. Post-install notice says "binary-only install is intentional". | install.sh `--spec-dir <path>` and install.ps1 `-SpecDir <path>` download, verify, and place specs | `scripts/` change (script-only) |
| **Runtime discovery** | `config.spec_dir` resolved via `resolve_config_paths()` in `boot.rs` (`config_dir.join(&config.spec_dir)` if relative). Default in `prism.toml.example`: `spec_dir = "./specs"`. | Match: installed path must equal `spec_dir` value in `prism.toml`. Achieved when installer places specs at `<spec-dir>/claroty.sensor.toml` and user sets `spec_dir = "<spec-dir>"` in `prism.toml`. | Docs only (correct prism.toml.example instructions) |

**No Rust code changes required.** All fixes are script-only (install.sh, install.ps1) and
workflow-only (release.yml).

**Redundancy note:** Specs are ALREADY bundled inside each per-platform binary archive
(see "Create archive" step in `.github/workflows/release.yml`). The separate tarball
this story introduces is an additional distribution channel — it is smaller (specs only,
no binary), platform-neutral, and allows operators to update specs independently of the binary.
The per-platform archives retain their `specs/` directory for manual extraction use cases.

---

## Narrative

As a prism operator installing beta.2 on a fresh system, I want `install.sh` (or `install.ps1`)
to place the Claroty xDome sensor spec in the directory my `prism.toml` references as `spec_dir`,
so that `prism start` finds the spec without requiring me to locate and copy files manually from
a GitHub source tree.

---

## Behavioral Contracts

This story has no subsystem BCs — spec packaging and install scripts are distribution tooling.

| Convention / Authority | Clause |
|-----------------------|--------|
| `release.yml` "Create archive" step | Specs already bundled inside per-platform archives at `specs/claroty.sensor.toml` |
| `prism.toml.example` | `spec_dir = "./specs"` — runtime resolves relative to config dir via `resolve_config_paths()` |
| `boot.rs` `resolve_config_paths()` | `config.spec_dir = config_dir.join(&config.spec_dir)` when relative |
| S-REL-003 checksum-verify pattern | SHA-256 verification from `checksums.txt` before any extraction |
| S-REL-003 shellcheck/PSScriptAnalyzer CI gates | Both scripts must pass CI lint with zero errors after modification |

---

## Token Budget Estimate

| Artifact | Estimated Tokens |
|----------|-----------------|
| This story spec | ~4,000 |
| `scripts/install.sh` (existing, read reference) | ~2,500 |
| `scripts/install.ps1` (existing, read reference) | ~2,000 |
| `.github/workflows/release.yml` (relevant jobs) | ~3,000 |
| `crates/prism-bin/src/boot.rs` (`resolve_config_paths` vicinity) | ~600 |
| `prism.toml.example` | ~600 |
| Total | ~12,700 |

Well within the 30% context window budget.

---

## Tasks

1. **Read `scripts/install.sh`** to understand the full argument-parsing loop, checksum
   download/verify pattern (`CHECKSUM_URL`, `CHECKSUM_CMD`, grep-then-awk extraction),
   and the "Extract and install" section that currently runs only `tar ... prism`.

2. **Read `scripts/install.ps1`** to understand the `-Version` / `-DryRun` parameter block,
   `Invoke-WebRequest` checksum download, `Select-String` based lookup, and the `Expand-Archive`
   + file copy pattern.

3. **Read the `publish-release` job** in `.github/workflows/release.yml` — specifically:
   the "Merge checksums" step (which creates `checksums.txt` from 4 per-platform entries),
   the "Create GitHub Release" step (the `gh release create` invocation), and the
   idempotent re-run guard (`gh release upload --clobber` path).

4. **Amend `.github/workflows/release.yml`** — `publish-release` job only:

   After the "Merge checksums" step, add two new steps:

   ```yaml
   - name: Create specs tarball
     shell: bash
     env:
       TAG: ${{ github.ref_name }}
     run: |
       set -euo pipefail
       # prism-specs-<tag>.tar.gz: platform-neutral archive containing only the
       # Claroty xDome sensor spec. Files are placed at the root of the tarball
       # (no specs/ prefix) so that `tar -xzf ... -C <spec-dir>` extracts directly
       # into the target directory without an intermediate subdirectory.
       # v1/beta.2 scope: Claroty xDome only (D-2443).
       SPECS_ARCHIVE="prism-specs-${TAG}.tar.gz"
       tar czf "${SPECS_ARCHIVE}" -C crates/prism-sensors/specs claroty.sensor.toml
       echo "SPECS_ARCHIVE=${SPECS_ARCHIVE}" >> "$GITHUB_ENV"

   - name: Compute specs tarball checksum
     shell: bash
     run: |
       # Append to checksums.txt that was merged from the 4 per-platform build legs.
       # publish-release always runs on ubuntu-latest; sha256sum is always present.
       sha256sum "$SPECS_ARCHIVE" >> checksums.txt
   ```

   Add `"$SPECS_ARCHIVE"` to BOTH the
   `gh release create` invocation AND the `gh release upload --clobber` invocation
   in the "Create GitHub Release" step, alongside the existing `.tar.gz`, `.zip`,
   `checksums.txt`, `scripts/install.sh`, `scripts/install.ps1` entries.
   Use the bare shell variable form ONLY — the `${{ env.SPECS_ARCHIVE }}` expression-
   substitution-into-run-body form reintroduces the CWE-78 injection anti-pattern
   (F-REL001-P1-001) this workflow was hardened against. Value propagation via
   `echo "SPECS_ARCHIVE=…" >> "$GITHUB_ENV"` in the "Create specs tarball" step
   ensures `$SPECS_ARCHIVE` is correctly set in the release step's shell context.

5. **Amend `scripts/install.sh`**:

   a. Add `--spec-dir` (and `--force-specs`) to the argument-parsing `while` loop:
   ```bash
   SPEC_DIR=""
   FORCE_SPECS=false
   # … in case block:
     --spec-dir)
       if [[ $# -lt 2 ]]; then
         printf 'ERROR: --spec-dir requires a value\n' >&2; exit 1
       fi
       SPEC_DIR="${2}"; shift 2 ;;
     --force-specs)
       FORCE_SPECS=true; shift ;;
   ```

   b. After "Extract and install" (after the existing `chmod 755 ...` line), add:
   ```bash
   # ---------------------------------------------------------------------------
   # Spec placement (AC-003, AC-004)
   # ---------------------------------------------------------------------------
   if [[ -n "${SPEC_DIR}" ]]; then
     SPECS_ARCHIVE="prism-specs-${VERSION}.tar.gz"
     SPECS_URL="https://github.com/${REPO}/releases/download/${VERSION}/${SPECS_ARCHIVE}"
     printf 'Downloading sensor specs for %s...\n' "${VERSION}"
     curl -fsSL --output "${TMPDIR_PRISM}/${SPECS_ARCHIVE}" "${SPECS_URL}"

     # Verify checksum using the already-downloaded checksums.txt
     # The `|| true` prevents set -euo pipefail from aborting the script when
     # grep finds no match (exit 1); the empty-string check below then fires
     # the friendly error.  Without `|| true` the friendly branch is unreachable.
     EXPECTED_SPEC="$(grep -F -- "${SPECS_ARCHIVE}" "${TMPDIR_PRISM}/checksums.txt" | awk '{print $1}' || true)"
     if [[ -z "${EXPECTED_SPEC}" ]]; then
       printf 'ERROR: %s not found in checksums.txt\n' "${SPECS_ARCHIVE}" >&2; exit 1
     fi
     ACTUAL_SPEC="$("${CHECKSUM_CMD[@]}" "${TMPDIR_PRISM}/${SPECS_ARCHIVE}" | awk '{print $1}')"
     if [[ "${EXPECTED_SPEC}" != "${ACTUAL_SPEC}" ]]; then
       printf 'ERROR: Checksum mismatch for %s\n' "${SPECS_ARCHIVE}" >&2
       printf '  Expected: %s\n' "${EXPECTED_SPEC}" >&2
       printf '  Actual:   %s\n' "${ACTUAL_SPEC}" >&2
       exit 1
     fi
     printf 'Specs checksum verified.\n'

     # No-clobber: protect user-modified spec files
     SPEC_TARGET="${SPEC_DIR}/claroty.sensor.toml"
     if [[ -f "${SPEC_TARGET}" ]] && [[ "${FORCE_SPECS}" != "true" ]]; then
       printf 'NOTE: %s already exists; skipping (pass --force-specs to overwrite).\n' "${SPEC_TARGET}"
     else
       mkdir -p "${SPEC_DIR}"
       tar -xzf "${TMPDIR_PRISM}/${SPECS_ARCHIVE}" -C "${SPEC_DIR}" claroty.sensor.toml
       printf 'Sensor spec installed to %s\n' "${SPEC_TARGET}"
       printf '  Set spec_dir = "%s" in your prism.toml.\n' "${SPEC_DIR}"
     fi
   fi
   ```

   c. **Sibling-sweep (U-1 / TD-VSDD-060): fix the identical pipefail-abort bug in the
   pre-existing `scripts/install.sh` "SHA-256 verification (AC-003: abort on mismatch)"
   section.** The current binary-archive checksum lookup:
   ```bash
   EXPECTED="$(grep -F -- "${ARCHIVE}" "${TMPDIR_PRISM}/checksums.txt" | awk '{print $1}')"
   ```
   has the same latent bug — with `set -euo pipefail`, a no-match grep exits 1 and
   aborts the script, making the `if [[ -z "${EXPECTED}" ]]` friendly-error branch
   unreachable. Apply the same `|| true` guard:
   ```bash
   EXPECTED="$(grep -F -- "${ARCHIVE}" "${TMPDIR_PRISM}/checksums.txt" | awk '{print $1}' || true)"
   ```
   This sibling fix MUST land in the same delivery as the new spec block (TD-VSDD-060
   sibling-sweep discipline, production-grade default rule 4). The section to amend is
   identified by its comment anchor — "SHA-256 verification (AC-003: abort on mismatch)"
   — not by line number.

   d. Update the post-install notice to remove "binary-only install is intentional":
   ```bash
   if [[ -z "${SPEC_DIR}" ]]; then
     printf '\nNOTE: Sensor specs not installed (no --spec-dir provided).\n'
     printf '  To install specs, re-run with: --spec-dir <path-to-config-dir>/specs\n'
     printf '  Then set spec_dir = "<path>" in your prism.toml.\n'
     printf '  See docs/SETUP.md §4 or RELEASING.md for the full setup guide.\n'
   fi
   ```

6. **Amend `scripts/install.ps1`**:

   a. Add `-SpecDir` and `-ForceSpecs` to the `param()` block:
   ```powershell
   param(
       [string]$Version = "",
       [switch]$DryRun,
       [string]$SpecDir = "",
       [switch]$ForceSpecs
   )
   ```

   b. After binary install (after "Confirm version"), add:
   ```powershell
   # ---------------------------------------------------------------------------
   # Spec placement (AC-005, AC-006)
   # ---------------------------------------------------------------------------
   if ($SpecDir) {
       $SpecsArchive = "prism-specs-$Version.tar.gz"
       $SpecsUrl = "https://github.com/$Repo/releases/download/$Version/$SpecsArchive"
       Write-Host "Downloading sensor specs for $Version..."
       $SpecsArchivePath = Join-Path $TempPath $SpecsArchive
       Invoke-WebRequest -Uri $SpecsUrl -OutFile $SpecsArchivePath -UseBasicParsing -TimeoutSec 30 -ErrorAction Stop

       # Verify checksum using the already-downloaded checksums.txt
       $SpecsActualHash = (Get-FileHash -Algorithm SHA256 -Path $SpecsArchivePath).Hash.ToLower()
       $SpecsMatchedLine = $null
       foreach ($Line in $ChecksumLines) {
           if ($Line -match [regex]::Escape($SpecsArchive)) { $SpecsMatchedLine = $Line; break }
       }
       if ($null -eq $SpecsMatchedLine) {
           Write-Error "ERROR: $SpecsArchive not found in checksums.txt"; exit 1
       }
       $SpecsExpectedHash = ($SpecsMatchedLine -split '\s+')[0].ToLower()
       if ($SpecsActualHash -ne $SpecsExpectedHash) {
           Write-Host "ERROR: Checksum mismatch for $SpecsArchive" -ForegroundColor Red
           Write-Host "  Expected: $SpecsExpectedHash"; Write-Host "  Actual:   $SpecsActualHash"
           exit 1
       }
       Write-Host "Specs checksum verified."

       # No-clobber: protect user-modified spec files (AC-006)
       $SpecTarget = Join-Path $SpecDir "claroty.sensor.toml"
       if ((Test-Path $SpecTarget) -and (-not $ForceSpecs)) {
           Write-Host "NOTE: $SpecTarget already exists; skipping (use -ForceSpecs to overwrite)."
       } else {
           # Guard: tar.exe is required (Windows 10 1803+ / Server 2019+).
           # Gracefully error if absent — do not silently skip (AC-005 / EC-008).
           if (-not (Get-Command tar.exe -ErrorAction SilentlyContinue)) {
               Write-Error "tar.exe is required (Windows 10 1803+ / Server 2019+); manual extraction required."; exit 1
           }
           New-Item -ItemType Directory -Path $SpecDir -Force | Out-Null
           & tar.exe -xzf $SpecsArchivePath -C $SpecDir claroty.sensor.toml
           if ($LASTEXITCODE -ne 0) {
               Write-Error "tar.exe extraction failed (exit code $LASTEXITCODE)"; exit 1
           }
           Write-Host "Sensor spec installed to $SpecTarget"
           Write-Host "  Set spec_dir = `"$SpecDir`" in your prism.toml."
       }
   } else {
       Write-Host ""
       Write-Host "NOTE: Sensor specs not installed (no -SpecDir provided)."
       Write-Host "  To install specs, re-run with: -SpecDir <path-to-config-dir>\specs"
       Write-Host "  Then set spec_dir = `"<path>`" in your prism.toml."
       Write-Host "  See docs/SETUP.md §4 or RELEASING.md for the full setup guide."
   }
   ```

   Note: `tar.exe` is built into Windows 10 1803+ (version 1.26 from libarchive). It can
   extract `.tar.gz` files. PSScriptAnalyzer compatibility: ensure `& tar.exe ...` form
   is used (not `Invoke-Expression`). The prescribed snippet above guards absence with
   `Get-Command tar.exe -ErrorAction SilentlyContinue` and checks `$LASTEXITCODE` after
   extraction — both are required by AC-005 / EC-008 (graceful error, not silent skip).

7. **Run shellcheck locally**: `shellcheck scripts/install.sh` — zero errors after amending.

8. **Verify locally (dry-run)**: `bash scripts/install.sh --version v1.0.0-beta.1 --dry-run`
   confirms the dry-run path still exits 0. (The spec-dir path runs only when not `$DRY_RUN`.)

---

## Acceptance Criteria

### AC-001: `prism-specs-<tag>.tar.gz` is built and attached to the GitHub Release
Given: A `v*` tag is pushed to `develop`, triggering `release.yml`.
When: The `publish-release` job runs.
Then: A `prism-specs-<tag>.tar.gz` asset (e.g., `prism-specs-v1.0.0-beta.2.tar.gz`) exists on
the GitHub Release page and is downloadable via the standard release asset URL:
`https://github.com/BOHICA-LABS/prism/releases/download/<tag>/prism-specs-<tag>.tar.gz`.
The tarball contains `claroty.sensor.toml` at the root level (no subdirectory prefix).
(traces to release convention: platform-neutral spec asset required for install.sh consumption)

### AC-002: `prism-specs-<tag>.tar.gz` checksum is in `checksums.txt`
Given: The GitHub Release for tag `<tag>` exists with `checksums.txt` attached.
When: `grep "prism-specs-${TAG}" checksums.txt` is run.
Then: Exactly one line appears, formatted as `<sha256-hex>  prism-specs-<tag>.tar.gz` (two-space
separator, consistent with sha256sum output). The checksum matches the actual file's SHA-256.
(traces to release convention: checksums.txt must cover all release assets including specs tarball)

### AC-003: `install.sh --spec-dir <path>` downloads, verifies, and places `claroty.sensor.toml`
Given: `scripts/install.sh` is amended with `--spec-dir` support AND a `prism-specs-<tag>.tar.gz`
asset exists on the GitHub Release for the specified version.
When: `bash scripts/install.sh --version <tag> --spec-dir /tmp/test-specs` is run.
Then: `sha256sum /tmp/test-specs/claroty.sensor.toml` exits 0 (file exists). The SHA-256 of the
installed file matches the SHA-256 of `crates/prism-sensors/specs/claroty.sensor.toml` at the
same git tag. The checksum verification step in the script ran and passed (no ERROR lines in output).
(traces to charter: install.sh must download, verify, and place specs at spec_dir target)

### AC-004: `install.sh` idempotency — existing spec is not clobbered without `--force-specs`
Given: `<spec-dir>/claroty.sensor.toml` already exists (simulating a user-modified spec).
When: `bash scripts/install.sh --version <tag> --spec-dir <spec-dir>` is run (no `--force-specs`).
Then: The script exits 0, prints "already exists; skipping", and the existing file content is
unchanged. When run with `--force-specs`, the file IS overwritten with the release version.
(traces to charter: "must not clobber user-modified specs without a defined rule")

### AC-005: `install.ps1 -SpecDir <path>` downloads, verifies, and places `claroty.sensor.toml`
Given: `scripts/install.ps1` is amended with `-SpecDir` support AND a `prism-specs-<tag>.tar.gz`
exists on the GitHub Release.
When: `pwsh -File scripts/install.ps1 -Version <tag> -SpecDir C:\Users\test\prism-specs` is run.
Then: `C:\Users\test\prism-specs\claroty.sensor.toml` exists. The SHA-256 of the installed file
matches the source. The checksum verification step passed (no error lines).
Note: `tar.exe` must be available (Windows 10 1803+, Server 2019+). The script must gracefully
error if `tar.exe` is absent (not silently skip).
(traces to charter: install.ps1 must match install.sh behavior on Windows)

### AC-006: `install.ps1` idempotency — existing spec not clobbered without `-ForceSpecs`
Given: `<SpecDir>\claroty.sensor.toml` exists.
When: `install.ps1 -Version <tag> -SpecDir <SpecDir>` is run without `-ForceSpecs`.
Then: Script exits 0, prints "already exists; skipping", file content unchanged. With
`-ForceSpecs`, the file is overwritten.
(traces to charter: "must not clobber user-modified specs without a defined rule")

### AC-007: Runtime discovery match — installed spec path is correctly found by prism runtime
Given: Specs are installed to `<spec-dir>` by install.sh or install.ps1.
When: `prism.toml` is configured with `spec_dir = "<spec-dir>"` (absolute path) OR
`spec_dir = "./specs"` with `config_dir = <parent-of-spec-dir>` (relative path).
Then: `prism start` finds `claroty.sensor.toml` via `resolve_config_paths()` in `boot.rs`
(`config.spec_dir = config_dir.join(&config.spec_dir)` for relative paths). No
`spec_dir not found` boot error.
This is verified by the LIVE-VERIFY AC (AC-009) — no separate automated test required for
beta.2 scope.
(traces to `boot.rs` `resolve_config_paths()` — `spec_dir` resolved relative to `config_dir`)

### AC-008: Both scripts pass shellcheck / PSScriptAnalyzer with zero new errors
Given: `scripts/install.sh` and `scripts/install.ps1` are amended.
When: `shellcheck scripts/install.sh` and `Invoke-ScriptAnalyzer -Path scripts/install.ps1 -Severity Error`
are run (same CI gates established by S-REL-003 AC-001 and AC-009).
Then: Both exit 0. Zero errors. Any new code added in this story must be shellcheck-clean
(bash 3.2+ compatible; `set -euo pipefail` safe) and PSScriptAnalyzer-clean (PS 5.1 compatible).
(traces to S-REL-003 CI gate convention: shellcheck + PSScriptAnalyzer mandatory for install scripts)

### AC-009: LIVE-VERIFY (post-merge, human-in-loop) — prism finds Claroty spec after install
Given: The beta.2 GitHub Release is published with `prism-specs-v1.0.0-beta.2.tar.gz` attached.
Given: A clean config directory (empty or newly created).
When: `bash scripts/install.sh --version v1.0.0-beta.2 --spec-dir <config-dir>/specs` is run
(or the equivalent install.ps1 on Windows).
Then: `<config-dir>/specs/claroty.sensor.toml` exists. When `prism.toml` at `<config-dir>` is
configured with `spec_dir = "./specs"` and `prism start --config-dir <config-dir>` is run,
the server boots successfully and the `prism_describe` MCP tool returns the Claroty sensor's
table list without a `spec_dir not found` error. This AC is MANUAL VERIFICATION post-merge.
(traces to charter: "LIVE-VERIFY AC … install a beta.2 release on a clean config dir and
confirm prism finds the Claroty spec")

---

## Previous Story Intelligence

S-REL-003 established all conventions this story extends:
- Argument-parsing loop (`while [[ $# -gt 0 ]]; do case "$1" in`) — follow this pattern for `--spec-dir`
- `CHECKSUM_CMD` dual-path (sha256sum / shasum -a 256) — reuse the already-set variable
- `TMPDIR_PRISM` temp dir + `_cleanup` trap — the spec tarball is downloaded INTO the same temp dir
- Checksum verification pattern (`grep -F -- "${ARCHIVE}" ... | awk '{print $1}'`) — reuse for spec tarball
- shellcheck-clean shell scripting conventions — `set -euo pipefail`, `printf` not `echo -e`
- `$DRY_RUN` guard — spec placement must be skipped when DRY_RUN=true
- PS 5.1 compatibility constraints (`#Requires -Version 5.1`, `PSObject.Properties` enumeration) — no new PS 7.0+ features
- PSScriptAnalyzer in CI — already wired; no new CI changes required for linting

Key lesson from S-REL-003 Task 3 ("read scripts/demo-setup.sh before writing install.sh"):
Read the existing `install.sh` in full before amending to understand the control flow from
argument parsing through cleanup trap. The spec-placement block must slot AFTER binary install
but BEFORE the PATH guidance section (so that a failed spec install does not suppress the
PATH guidance message).

S-REL-001 pattern for `publish-release` job: the `ARCHIVE` env var is passed between steps via
`$GITHUB_ENV`. Follow the same pattern for `SPECS_ARCHIVE`. Use the `env:` binding in the
"Create GitHub Release" step to prevent CWE-78 injection (same as the existing `TAG:` binding).

---

## Architecture Compliance Rules

| Rule | Source | Enforcement |
|------|--------|-------------|
| `scripts/install.sh` must be shellcheck-clean | CLAUDE.md §Conventions + S-REL-003 AC-001 | CI gate: `shellcheck scripts/install.sh` (existing, no new step needed) |
| `scripts/install.ps1` must pass PSScriptAnalyzer | S-REL-003 AC-009 | CI gate: `Invoke-ScriptAnalyzer -Severity Error` (existing) |
| Checksum mismatch MUST abort install | Security: tampered binary / spec prevention | Non-zero exit + clear error message (AC-003, AC-005) |
| No gh CLI dependency | Research U8 (S-REL-003) | GitHub REST API for version resolution; direct URL construction for asset download |
| `$DRY_RUN` guard for spec placement | install.sh behavior contract | Spec download/extraction MUST be skipped when `DRY_RUN=true` |
| Env var TAG binding in workflow | CWE-78 per S-REL-001 F-REL001-P1-001 | `TAG: ${{ github.ref_name }}` under `env:` map; never textual substitution |
| spec tarball at root (no `specs/` prefix) | Extraction convention | `tar czf ... -C crates/prism-sensors/specs claroty.sensor.toml` |
| No-clobber default | Operator safety | File-exists check before extraction; `--force-specs` / `-ForceSpecs` required to overwrite |
| PS 5.1 compatibility | S-REL-003 constraint U29/U30 | No `-AsHashtable`, no PS 7.0+ ternary; `tar.exe` is a PS 5.1-compatible system binary |

---

## Library & Framework Requirements

| Tool | Version / Notes |
|------|-----------------|
| bash | 3.2+ (macOS ships bash 3.2; use `printf`, avoid `echo -e`, no `[[ ... ]]` bash-4-only features) |
| `sha256sum` / `shasum` | Dual-path per U9 (already in `CHECKSUM_CMD` from S-REL-003 amend) |
| `curl` | Already required by install.sh; spec tarball download uses the same `curl -fsSL` pattern |
| `tar` | GNU tar (Linux) / BSD tar (macOS) — both support `tar -xzf ... -C <dir>`; no `--strip-components` |
| PowerShell | 5.1+ — `#Requires -Version 5.1` already enforced by install.ps1 |
| `tar.exe` | Windows 10 1803+ / Server 2019+ built-in. Version: system default. Supports `.tar.gz`. |
| `Get-FileHash` | Built into PowerShell 5+ — already used for binary checksum in install.ps1 |
| `sha256sum` (CI runner) | Ubuntu-latest has it; used in `publish-release` job for spec tarball checksum |

---

## File Structure Requirements

| File | Action | Notes |
|------|--------|-------|
| `.github/workflows/release.yml` | Modify | Add "Create specs tarball" + "Compute specs tarball checksum" steps in `publish-release` job; add `$SPECS_ARCHIVE` to `gh release create` and `gh release upload` commands |
| `scripts/install.sh` | Modify | Add `--spec-dir` + `--force-specs` argument parsing; add spec-placement block after binary install; update post-install notice |
| `scripts/install.ps1` | Modify | Add `-SpecDir` + `-ForceSpecs` parameters; add spec-placement block after "Confirm version" section; update post-install notice |

No new files created. No Rust crates modified. No `prism.toml.example` change required
(it already documents `spec_dir = "./specs"` correctly).

---

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|----------------|
| `publish-release` job specs tarball step | `.github/workflows/release.yml` | N/A (CI YAML) |
| `scripts/install.sh` spec placement | `scripts/` | N/A (bash shell script) |
| `scripts/install.ps1` spec placement | `scripts/` | N/A (PowerShell script) |

---

## Purity Classification

| Module | Classification | Justification |
|--------|----------------|---------------|
| `scripts/install.sh` | N/A | Shell script — Rust purity boundary does not apply |
| `scripts/install.ps1` | N/A | PowerShell script — Rust purity boundary does not apply |

---

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `--spec-dir` not provided | Binary installs successfully; spec-placement block is skipped; post-install notice prints `--spec-dir` instructions |
| EC-002 | `--spec-dir <path>` provided but directory does not exist | `mkdir -p "${SPEC_DIR}"` creates it; spec is extracted |
| EC-003 | `<spec-dir>/claroty.sensor.toml` already exists, no `--force-specs` | Print "already exists; skipping"; exit 0; file unchanged |
| EC-004 | `<spec-dir>/claroty.sensor.toml` already exists, `--force-specs` passed | File is overwritten with release version; exit 0 |
| EC-005 | Spec tarball checksum mismatch (corrupted download) | Print "Checksum mismatch" with expected/actual; exit non-zero; binary install already completed (spec install is additive, not atomic with binary) |
| EC-006 | `prism-specs-<tag>.tar.gz` not found on GitHub Release (e.g., old pre-beta.2 tag) | `curl -fsSL` exits non-zero; install.sh prints error and exits non-zero |
| EC-007 | `--dry-run` combined with `--spec-dir` | Dry run exits before downloading; spec-placement block is never reached |
| EC-008 | Windows: `tar.exe` not available (Windows < 10 1803) | install.ps1 must detect absence and print a clear error: "tar.exe is required (Windows 10 1803+); manual extraction required" |
| EC-009 | `spec_dir` in prism.toml uses relative path `"./specs"` with `prism start --config-dir <dir>` | `resolve_config_paths()` in `boot.rs` resolves `<dir>/specs/` — correct as long as install placed specs at `<dir>/specs/claroty.sensor.toml` |
| EC-010 | Re-run of install.sh after binary upgrade (new version, spec already present) | No-clobber path: spec file from old version is not automatically upgraded; user must pass `--force-specs` or manually delete the old file first |

---

## Traceability

| AC | Mechanism | Source |
|----|-----------|--------|
| AC-001 | `publish-release` job "Create specs tarball" step | charter: "build a prism-specs-<tag>.tar.gz release asset" |
| AC-002 | `publish-release` job "Compute specs tarball checksum" step | charter: "emit its checksum into … checksums.txt" |
| AC-003 | `install.sh --spec-dir` + curl + checksum verify + tar extract | charter: "install.sh update: download … verify its checksum … extract/place" |
| AC-004 | No-clobber logic in install.sh | charter: "must not clobber user-modified specs without a defined rule" |
| AC-005 | `install.ps1 -SpecDir` + Invoke-WebRequest + checksum verify + tar.exe extract | charter: "install.ps1 update: same behavior on Windows" |
| AC-006 | No-clobber logic in install.ps1 | charter: "must not clobber user-modified specs without a defined rule" |
| AC-007 | `resolve_config_paths()` in `boot.rs` + prism.toml `spec_dir` field | charter: "runtime discovery reconciliation … install destination MUST match what the runtime discovers" |
| AC-008 | Existing shellcheck / PSScriptAnalyzer CI gates (S-REL-003) | charter: "CI gates (shellcheck install scripts, PSScriptAnalyzer, release gate)" |
| AC-009 | Manual post-merge live verify | charter: "a LIVE-VERIFY AC … install a beta.2 release on a clean config dir" |

---

## Forbidden Dependencies

- No `gh` CLI dependency in `install.sh` (requires auth; breaks non-GitHub-authed environments — S-REL-003 research U8)
- No new dependencies on Python, jq, or any non-standard POSIX tools in install.sh
- No PowerShell 7.0+ features in install.ps1 (no `-AsHashtable`, no ternary operator, no `??=`)
- No changes to any Rust crate (`prism-bin`, `prism-spec-engine`, etc.) — this story is script/workflow-only

---

## Open Questions for Architect (None — all decisions resolved in scope)

All decisions are answerable from existing project conventions (Canonical Principle §6):

- **Separate tarball vs. extract from existing archive**: Decided for separate tarball per
  charter request. Rationale: platform-neutral format, smaller download for spec-only updates,
  consistent with the charter's explicit requirement. Alternative (extract from per-platform archive)
  was considered and noted in Background; not selected for this story.

- **`--spec-dir` vs `--config-dir`**: Decided for `--spec-dir` as it directly names the extraction
  target without implying any additional prism.toml setup responsibilities. Operators provide the
  exact directory where specs should land.

- **No-clobber rule**: Decided for "never overwrite without `--force-specs`". Rationale: protects
  operator customizations; idempotent and safe for automated re-runs.

---

## Changelog

| Version | Date | Summary |
|---------|------|---------|
| 1.2 | 2026-09-09 | NIT-1 reconciliation: mkdir/New-Item moved to write-path post-verify per PR #279 review (merged @54523dccd) |
| 1.1 | 2026-09-09 | remove-uncertainty (D-1110): U-1 pipefail-abort guard + binary-block sibling sweep task, U-2 CWE-78 injection option removed, U-3 tar.exe absence check added to install.ps1 snippet |
| 1.0 | 2026-09-09 | Initial story creation (story-writer; beta.2 charter) |
