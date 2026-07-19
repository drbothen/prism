---
document_type: story
story_id: S-REL-003
title: "devops: install.sh + install.ps1 — checksum-verified install scripts for 5 platforms"
wave: F-A
epic_id: E-REL
priority: P0
status: draft
version: "0.3"
level: "L4"
producer: story-writer
timestamp: "2026-07-19T00:00:00Z"
tdd_mode: strict
subsystems: []
# Subsystem anchor justification:
#   install.sh and install.ps1 are distribution tooling outside the prism binary's
#   subsystem boundaries. No ARCH-INDEX subsystem owns install scripts.
#   subsystems: [] per S-0.01 and S-MAINT-CI-DISK-EXHAUSTION-001 precedent.
crates_touched: [devops]
target_module: devops
capabilities: []
behavioral_contracts: []
# BC status: N/A — install scripts are distribution tooling. No subsystem BC governs
# binary installation procedures. Conforming per W3-FIX-CI-001 precedent.
verification_properties: []
depends_on: [S-REL-001]
# Dependency anchor justification:
#   depends_on S-REL-001: install.sh/install.ps1 must know the correct GitHub Releases
#   URL pattern and whether the release is tagged as --prerelease or GA. S-REL-001 repairs
#   the release workflow so that release assets exist at predictable URLs before install
#   scripts can fetch them.
blocks: [S-REL-005]
# Dependency anchor justification:
#   blocks S-REL-005: RELEASING.md operator runbook documents the install script as a
#   required deliverable; the runbook cannot be finalized until the scripts exist.
points: 3
estimated_days: 2
risk: MEDIUM
# Risk justification: New files — no regressions from removal of existing code.
# Checksum verification must be cryptographically correct (SHA-256 mismatch must abort
# install). Platform detection must correctly distinguish macOS ARM64/x86_64, Linux
# glibc/musl, and Windows. Risk is MEDIUM per delta-analysis §8: "test on all 5 platforms".
acceptance_criteria_count: 10
red_gate_tests: 3
estimated_passes: "2-3 LOCAL adversary passes"
holdout_scenarios: []
assumption_validations: []
risk_mitigations:
  - "Checksum abort: install.sh must exit non-zero with a clear error message if the
    SHA-256 checksum of the downloaded archive does not match checksums.txt. Silent
    continuation on checksum mismatch is a critical security failure."
  - "Platform detection — musl composite (U10): do NOT rely on ldd --version alone (some
    musl-based distros omit ldd entirely). Use a three-probe composite:
    1. getconf GNU_LIBC_VERSION 2>/dev/null (fails on musl → musl);
    2. test -e /lib/ld-musl-x86_64.so.1 (present on Alpine and most musl distros);
    3. ldd /bin/sh 2>&1 | grep -q musl (fallback).
    Target x86_64-unknown-linux-musl if any probe succeeds."
  - "Version resolution — no gh CLI dependency (U8): install.sh resolves the latest
    release via the GitHub REST API (/releases?per_page=1), NOT /releases/latest (which
    excludes prereleases) and NOT `gh release view` (requires gh CLI auth). Use
    `curl -sS` with a jq-free grep/sed pipeline."
  - "SHA-256 checksum tool dual-path (U9): macOS ships `shasum`, Linux ships `sha256sum`.
    Detect with `command -v sha256sum 2>/dev/null || echo shasum -a 256` at startup;
    store in CHECKSUM_CMD variable; fail with a clear error if neither is found."
  - "install.ps1 Invoke-WebRequest vs curl.exe: Windows runners have both; use
    Invoke-WebRequest with -OutFile and -UseBasicParsing (-UseBasicParsing is a no-op on
    PowerShell 7.x; harmless for 5.1 compat). Handle -ErrorAction Stop."
  - "install.ps1 PSScriptAnalyzer (U33): PSScriptAnalyzer is NOT guaranteed on
    windows-latest; install explicitly in CI before running analysis:
    `Install-Module -Name PSScriptAnalyzer -Scope CurrentUser -Force`"
  - "Shellcheck: both install.sh scripts must pass shellcheck in CI (follow S-DEMO-003
    AC-014 precedent — add shellcheck step to ci.yml for scripts/install.sh)."
  - "PATH guidance: installer must print actionable PATH instructions for both
    system-wide (/usr/local/bin) and user-local (~/.local/bin) install paths."
inputs:
  - ".factory/planning/feature-release-engineering/delta-analysis.md"
  - ".factory/planning/feature-release-engineering/prism-consumer-contract.md"
  - ".github/workflows/release.yml"
  - ".factory/research/release-engineering-uncertainties-2026.md"
input-hash: "e11dfc9"
traces_to: []
cycle: "v1.0.0-release-engineering"
phase: "F3"
---

# S-REL-003 — devops: install.sh + install.ps1

**Story ID:** S-REL-003
**Status:** draft
**Version:** v0.3
**Wave:** F-A
**Priority:** P0
**Points:** 3

---

## Origin

`scripts/install.sh` and `scripts/install.ps1` do not exist (delta-analysis §2.1). The
secops-factory `activate` skill and the RELEASING.md runbook both need install scripts
that consumers can run to get the `prism` binary without building from source. Scripts
must perform checksum-verified downloads from GitHub Releases.

---

## Narrative

As a prism consumer (MSSP analyst or secops-factory operator), I want a single-command
installer that downloads the correct binary for my platform, verifies its SHA-256
checksum, and installs it to a writable location with PATH guidance, so that I can get
started without a Rust toolchain.

---

## Behavioral Contracts

This story has no subsystem BCs — install scripts are distribution tooling.

| Architecture Source | Clause |
|--------------------|--------|
| `delta-analysis.md` §2.1 | `scripts/install.sh` and `scripts/install.ps1` must be created |
| `delta-analysis.md` §8 | Medium risk; test on all 5 platforms |
| `prism-consumer-contract.md` §5.1 | Tag scheme: v1.0.0-rc.1 download URL pattern |

---

## Token Budget Estimate

| Artifact | Estimated Tokens |
|----------|-----------------|
| This story spec | ~2,800 |
| `delta-analysis.md` §2 and §8 | ~1,500 |
| `prism-consumer-contract.md` §5 | ~600 |
| Existing `scripts/demo-setup.sh` (pattern reference) | ~1,500 |
| `release-engineering-uncertainties-2026.md` U8/U9/U10/U29-U33 | ~1,000 |
| Total | ~7,400 |

Well within the 30% context window budget.

---

## Tasks

1. **Read `delta-analysis.md` §2.1 and §8** for requirements.
2. **Read `scripts/demo-setup.sh`** for shell scripting conventions (shellcheck-clean patterns,
   `set -euo pipefail`, error messaging style).

3. **Create `scripts/install.sh`** (macOS + Linux):
   - Header: `#!/usr/bin/env bash` + `set -euo pipefail`
   - Variables: `REPO="drbothen/prism"`, `INSTALL_DIR` (default `/usr/local/bin`, fallback
     `~/.local/bin` if not writable)
   - Accept optional `--version <tag>` argument; default to latest release via:
     ```bash
     # U8: use /releases?per_page=1 NOT /releases/latest (excludes prereleases);
     #     no gh CLI dependency in install.sh.
     VERSION=$(curl -sS "https://api.github.com/repos/${REPO}/releases?per_page=1" \
       | grep '"tag_name"' | head -1 \
       | sed 's/.*"tag_name": *"\([^"]*\)".*/\1/')
     ```
   - **SHA-256 tool detection (U9):**
     ```bash
     CHECKSUM_CMD=$(command -v sha256sum 2>/dev/null || echo "shasum -a 256")
     if [[ -z "${CHECKSUM_CMD}" ]]; then
       echo "ERROR: neither sha256sum nor shasum found" >&2; exit 1
     fi
     ```
   - **Platform detection with musl composite (U10):**
     ```bash
     OS="$(uname -s)"
     ARCH="$(uname -m)"
     case "${OS}-${ARCH}" in
       Darwin-arm64)  TARGET="aarch64-apple-darwin" ;;
       Darwin-x86_64) TARGET="x86_64-apple-darwin" ;;
       Linux-x86_64)
         # Composite musl detection: getconf → ld-musl path → ldd fallback (U10)
         if ! getconf GNU_LIBC_VERSION >/dev/null 2>&1 \
           || test -e /lib/ld-musl-x86_64.so.1 \
           || (ldd /bin/sh 2>&1 | grep -q musl); then
           TARGET="x86_64-unknown-linux-musl"
         else
           TARGET="x86_64-unknown-linux-gnu"
         fi ;;
       *) echo "Unsupported platform: ${OS}-${ARCH}" >&2; exit 1 ;;
     esac
     ```
   - **Download URL construction:**
     ```bash
     ARCHIVE="prism-${VERSION}-${TARGET}.tar.gz"
     URL="https://github.com/${REPO}/releases/download/${VERSION}/${ARCHIVE}"
     CHECKSUM_URL="https://github.com/${REPO}/releases/download/${VERSION}/checksums.txt"
     ```
   - Download archive and checksums.txt to a temp dir
   - **SHA-256 verification using CHECKSUM_CMD:**
     ```bash
     EXPECTED=$(grep "${ARCHIVE}" checksums.txt | awk '{print $1}')
     ACTUAL=$(${CHECKSUM_CMD} "${ARCHIVE}" | awk '{print $1}')
     if [[ "${EXPECTED}" != "${ACTUAL}" ]]; then
       echo "ERROR: Checksum mismatch for ${ARCHIVE}" >&2
       echo "  Expected: ${EXPECTED}" >&2
       echo "  Actual:   ${ACTUAL}" >&2
       exit 1
     fi
     ```
   - Extract archive; install `prism` binary to `$INSTALL_DIR`
   - Print PATH guidance if `$INSTALL_DIR` is not in `$PATH`
   - Print: `prism installed to ${INSTALL_DIR}/prism (version: $(prism --version))`

4. **Create `scripts/install.ps1`** (Windows, PowerShell 5.1+):
   - `#Requires -Version 5.1` at the top (U29)
   - Always targets `x86_64-pc-windows-msvc`
   - Version resolution (U8): use GitHub REST API, no gh CLI:
     ```powershell
     $ReleasesJson = Invoke-WebRequest -Uri "https://api.github.com/repos/$Repo/releases?per_page=1" `
       -UseBasicParsing -ErrorAction Stop
     # PSObject.Properties enumeration (5.1-safe; no -AsHashtable which requires 7.0+) (U30)
     $Releases = $ReleasesJson.Content | ConvertFrom-Json
     $Version = $Releases[0].tag_name
     ```
   - `$Archive = "prism-$Version-x86_64-pc-windows-msvc.zip"`
   - `Invoke-WebRequest -Uri $Url -OutFile $Archive -UseBasicParsing -ErrorAction Stop`
   - SHA-256: `(Get-FileHash -Algorithm SHA256 $Archive).Hash` (built into PS 5+)
   - Checksum: parse checksums.txt with `Select-String`; compare; abort on mismatch with
     `Write-Error` and exit code 1
   - Install to `$env:LOCALAPPDATA\prism\bin\` with `[Environment]::SetEnvironmentVariable` guidance
   - Print PATH instruction for Windows
   - **No credential piping via `$secret | exe`** — install scripts handle only binary
     archives (U31; no credential delivery in this script)
   - `#Requires -Version 5.1` ensures 5.1 compat throughout; use `PSObject.Properties`
     enumeration not `-AsHashtable` for JSON parsing (U30)

5. **Add shellcheck to CI** for `scripts/install.sh`:
   - Add a step to `.github/workflows/ci.yml` shellcheck job (or create one following
     S-DEMO-003 AC-014 precedent): `shellcheck scripts/install.sh`

6. **Add PSScriptAnalyzer CI step for `scripts/install.ps1` (U33):**
   PSScriptAnalyzer is NOT pre-installed on windows-latest. Add a step to ci.yml that:
   ```yaml
   - name: Install PSScriptAnalyzer
     shell: pwsh
     run: Install-Module -Name PSScriptAnalyzer -Scope CurrentUser -Force
   - name: Lint install.ps1
     shell: pwsh
     run: Invoke-ScriptAnalyzer -Path scripts/install.ps1 -Severity Error
   ```

7. **Amend `publish-release` job in `.github/workflows/release.yml` to upload install scripts (ADJ-002/U26):**
   After the `gh release create` invocation in the `publish-release` job, add a `gh release upload`
   step (or include the files in the create invocation) for `scripts/install.sh` and
   `scripts/install.ps1`:
   ```bash
   gh release upload "$TAG" scripts/install.sh scripts/install.ps1
   ```
   Version is passed to PowerShell consumers via `$env:PRISM_INSTALL_VERSION` env var set before
   the `irm | iex` pipe — NOT as a positional arg to `iex` (U8: positional args cannot be carried
   through `iex`). This upload step lands in S-REL-003's PR because the scripts are authored here;
   S-REL-001 establishes the release URL pattern but does not reference paths that do not yet exist.

8. **Verify locally:**
   - `shellcheck scripts/install.sh` → 0 errors
   - `bash scripts/install.sh --version v1.0.0-rc.1 --dry-run` (add dry-run flag that
     prints download URL without downloading) → correct URL for host platform
   - `pwsh -File scripts/install.ps1 -DryRun` (if dry-run added) → correct URL for Windows

---

## Acceptance Criteria

### AC-001: `scripts/install.sh` exists and is shellcheck-clean
Given: `scripts/install.sh` is committed.
When: `shellcheck scripts/install.sh` is run.
Then: Exit code 0. Zero errors or warnings.
(traces to delta-analysis.md §2.1: "scripts/install.sh must be created")

### AC-002: Platform detection covers all 5 targets with composite musl detection
Given: `scripts/install.sh` is read.
When: The platform detection block is inspected.
Then: All five targets are mapped: `aarch64-apple-darwin`, `x86_64-apple-darwin`,
`x86_64-unknown-linux-gnu`, `x86_64-unknown-linux-musl`, `x86_64-pc-windows-msvc`.
The musl detection uses the three-probe composite (getconf → ld-musl path → ldd fallback)
per U10 — NOT ldd --version alone. Unsupported platforms print an error to stderr and
exit non-zero.
(traces to delta-analysis.md §2.1: "5-platform" requirement; research U10: composite musl
detection)

### AC-003: SHA-256 checksum verification is enforced in install.sh
Given: A tampered archive is placed in the temp dir alongside a valid checksums.txt.
When: The checksum verification step runs.
Then: Exit code non-zero. Stderr contains "Checksum mismatch" with both the expected and
actual SHA-256 values. Installation does NOT proceed.
The script uses the dual-path `CHECKSUM_CMD` (sha256sum or shasum -a 256 per U9).
(traces to delta-analysis.md §2.1: "checksum-verified download")

### AC-004: install.sh prints actionable PATH guidance
Given: The install dir is not in $PATH.
When: `scripts/install.sh` completes successfully.
Then: Stdout includes a message explaining how to add the install dir to PATH
(e.g., `export PATH="$HOME/.local/bin:$PATH"`).
(traces to delta-analysis.md §2.1: "PATH guidance")

### AC-005: `scripts/install.ps1` exists for Windows (PS 5.1+)
Given: `scripts/install.ps1` is committed.
When: The file is read.
Then: The file has `#Requires -Version 5.1` at the top; targets `x86_64-pc-windows-msvc`;
uses `Invoke-WebRequest` with `-UseBasicParsing` for download; performs SHA-256 checksum
verification using `Get-FileHash`; installs to a user-writable location; prints PATH
instructions for Windows. JSON parsing uses `ConvertFrom-Json` with `PSObject.Properties`
enumeration (5.1-safe; no `-AsHashtable`).
(traces to delta-analysis.md §2.1: "Windows first-class"; research U29/U30)

### AC-006: install.ps1 aborts on checksum mismatch
Given: A tampered archive is presented to the PowerShell script.
When: The checksum verification step runs.
Then: The script exits with a non-zero code and prints a clear error message. Installation
does NOT proceed.
(traces to delta-analysis.md §2.1: "checksum-verified download from GH Releases")

### AC-007: Both scripts accept --version / -Version argument
Given: A specific release tag is passed (e.g., `--version v1.0.0-rc.1`).
When: The script runs.
Then: The download URL is constructed from the specified version, not the latest release.
When no version is specified, the default is resolved via GitHub REST API
`/releases?per_page=1` (not `/releases/latest`, which excludes prereleases) with no
gh CLI dependency.
(traces to delta-analysis.md §2.1: install scripts support version pinning; research U8:
/releases?per_page=1 for prerelease-inclusive latest)

### AC-008: install.sh handles v*-rc.* tags correctly
Given: `--version v1.0.0-rc.1` is passed.
When: The archive URL is constructed.
Then: URL is `https://github.com/drbothen/prism/releases/download/v1.0.0-rc.1/prism-v1.0.0-rc.1-<TARGET>.tar.gz`.
No special treatment needed — prereleases use the same URL pattern as GA releases.
(traces to delta-analysis.md §2.1 + S-REL-001 AC-005: prerelease and GA share the same
archive URL pattern; --prerelease flag only affects GH release page display)

### AC-009: shellcheck runs in CI for install.sh; PSScriptAnalyzer runs for install.ps1
Given: `.github/workflows/ci.yml` is modified.
When: The CI pipeline runs on any PR.
Then: A step runs `shellcheck scripts/install.sh` (zero errors). A separate step installs
PSScriptAnalyzer explicitly (`Install-Module -Name PSScriptAnalyzer -Scope CurrentUser
-Force`) and runs `Invoke-ScriptAnalyzer -Path scripts/install.ps1 -Severity Error`.
(traces to delta-analysis.md §8: "test install scripts"; CLAUDE.md §Conventions shellcheck
discipline; research U33: PSScriptAnalyzer must be explicitly installed)

### AC-010: publish-release job in release.yml uploads install.sh and install.ps1
Given: `.github/workflows/release.yml` is modified by this story.
When: `grep -n 'install\.sh\|install\.ps1' .github/workflows/release.yml` is run.
Then: At least one match appears inside the publish-release job context (a `gh release create`
or `gh release upload` invocation that includes `scripts/install.sh` and `scripts/install.ps1`).
(traces to delta-analysis.md §13 ADJ-002: S-REL-003 owns both authoring the install scripts
and the publish-release upload step; upload moved from S-REL-001 per pre-TDD scan ADJ-002)

---

## Previous Story Intelligence

S-DEMO-003 established the pattern for shellcheck-clean shell scripts and CI enforcement
(AC-014 pattern). `scripts/demo-setup.sh` is the canonical reference for:
- `set -euo pipefail` header
- Error messages to stderr via `echo "... " >&2`
- Platform detection via `uname -s` / `uname -m`
- shellcheck-clean function definitions

Read `scripts/demo-setup.sh` before writing `scripts/install.sh` to reuse its conventions.

Key lessons from fix-burst U8/U9/U10 research:
- U8: `/releases/latest` GitHub endpoint EXCLUDES prereleases — NEVER use it in install.sh;
  use `/releases?per_page=1` to get the most recent release including pre-releases.
- U9: macOS ships `shasum`, Linux ships `sha256sum`; detect at startup with `command -v`.
- U10: musl distros (Alpine) may not have ldd; use three-probe composite detection.

---

## Architecture Compliance Rules

| Rule | Source | Enforcement |
|------|--------|-------------|
| Scripts must be shellcheck-clean | CLAUDE.md §Conventions + S-DEMO-003 AC-014 precedent | CI gate: `shellcheck scripts/install.sh` |
| Checksum mismatch must abort install | Security: tampered binary prevention | Non-zero exit + clear error message (AC-003) |
| No credential values in script output | AD-017 | Install scripts handle only binary archives; no credentials touched |
| install.sh targets only the main `prism` binary | delta-analysis §6: main archive contains only `prism` | No prism-dtu-demo-server in install script |
| No gh CLI dependency in install.sh | Research U8 | Use GitHub REST API for version resolution |
| SHA-256 tool dual-path | Research U9 | command -v sha256sum with shasum -a 256 fallback on macOS (per U9) |
| Musl composite detection | Research U10 | getconf → ld-musl path → ldd fallback |
| PS 5.1 compat throughout install.ps1 | Research U29/U30 | `#Requires -Version 5.1`; no `-AsHashtable` |
| PSScriptAnalyzer explicitly installed in CI | Research U33 | `Install-Module -Name PSScriptAnalyzer -Scope CurrentUser -Force` |

---

## Library & Framework Requirements

| Tool | Version | Notes |
|------|---------|-------|
| bash | 3.2+ | macOS ships bash 3.2; use POSIX-compatible features only |
| `sha256sum` / `shasum` | coreutils / macOS built-in | Dual-path per U9: command -v sha256sum; falls back to shasum -a 256 on macOS |
| `curl` | system | Used for GitHub REST API calls in install.sh; no gh CLI dependency |
| PowerShell | 5.1+ | `#Requires -Version 5.1` at top of install.ps1; avoid 7.0-only features |
| `Get-FileHash` | Built into PowerShell 5+ | No external dep for checksum on Windows |
| PSScriptAnalyzer | Latest via Install-Module | Must be explicitly installed in CI; not pre-installed on windows-latest |

---

## File Structure Requirements

| File | Action | Notes |
|------|--------|-------|
| `scripts/install.sh` | Create | macOS + Linux install; shellcheck-clean |
| `scripts/install.ps1` | Create | Windows install; PowerShell 5.1+ |
| `.github/workflows/ci.yml` | Modify | Add shellcheck step for install.sh + PSScriptAnalyzer step for install.ps1 |
| `.github/workflows/release.yml` | Modify | Amend publish-release job to upload scripts/install.sh and scripts/install.ps1 (ADJ-002) |

---

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|----------------|
| `scripts/install.sh` | `scripts/` | N/A (shell script; not Rust) |
| `scripts/install.ps1` | `scripts/` | N/A (PowerShell script; not Rust) |

---

## Purity Classification

| Module | Classification | Justification |
|--------|----------------|---------------|
| `scripts/install.sh` | N/A | Shell script — no Rust purity boundary applies |
| `scripts/install.ps1` | N/A | PowerShell script — no Rust purity boundary applies |

---

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | macOS ARM64 (Apple Silicon) | Targets `aarch64-apple-darwin` |
| EC-002 | macOS x86_64 (Intel) | Targets `x86_64-apple-darwin` |
| EC-003 | Linux glibc | Targets `x86_64-unknown-linux-gnu` |
| EC-004 | Linux musl (Alpine) | Targets `x86_64-unknown-linux-musl`; detected via three-probe composite (getconf → ld-musl-x86_64.so.1 path → ldd fallback); ldd --version alone insufficient (U10) |
| EC-005 | Unsupported OS (e.g., FreeBSD) | Error to stderr + exit 1; no partial install |
| EC-006 | Checksum mismatch (corrupted download) | Error to stderr + exit 1; temp files cleaned up |
| EC-007 | /usr/local/bin not writable | Fallback to ~/.local/bin with PATH guidance |
| EC-008 | `--version v1.0.0-rc.1` specified | Correct pre-release URL constructed; no special handling needed |
| EC-009 | `sha256sum` not on macOS | `CHECKSUM_CMD` falls back to `shasum -a 256`; install proceeds correctly |
| EC-010 | GitHub API returns prerelease as latest | `/releases?per_page=1` correctly includes prereleases unlike `/releases/latest` |

---

## Forbidden Dependencies

- No curl/wget hardcoded preference — use whichever is available (prefer curl, fallback wget)
- No dependency on Rust toolchain or cargo
- No `prism-dtu-demo-server` in main install archive (separation per delta-analysis §6)
- No `gh` CLI dependency in install.sh (requires auth; breaks non-GitHub-authed environments per U8)
- No PowerShell 7.0+ features in install.ps1 (`-AsHashtable`, `??=`, ternary operator, etc.)

---

## Changelog

| Version | Date | Summary |
|---------|------|---------|
| 0.3 | 2026-07-19 | ADJ-002 per delta-analysis.md §13: added task to amend publish-release job in release.yml to upload install.sh/install.ps1; added AC-010 for upload verification; acceptance_criteria_count 9→10 |
| 0.2 | 2026-07-19 | Fix-burst: U8 version resolution via /releases?per_page=1 (no gh CLI dep); U9 SHA-256 dual-path (sha256sum/shasum); U10 composite musl detection (getconf→ld-musl path→ldd); U29/U30/U31/U33 PS 5.1 constraints (#Requires + PSObject.Properties + PSScriptAnalyzer explicit install); research file added to inputs |
| 0.1 | 2026-07-19 | Initial story creation (story-writer F3 burst) |
