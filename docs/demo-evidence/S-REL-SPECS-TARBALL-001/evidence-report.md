# Demo Evidence Report — S-REL-SPECS-TARBALL-001

**Story:** S-REL-SPECS-TARBALL-001 — Specs Tarball Release Asset + Install Script Spec Placement (beta.2)
**tdd_mode:** facade (shell scripts + CI YAML — no Rust changes)
**Recorded:** 2026-09-09
**Recording method:** VHS tape (terminal recordings) + structural verification output

---

## Coverage Summary

| AC | Surface Recorded | Recording | Status |
|----|-----------------|-----------|--------|
| AC-001 | `release.yml` "Create specs tarball" step — `tar czf` command and `GITHUB_ENV` propagation | `AC-001-002-release-yml-tarball-steps.tape` + `.gif` | PASS |
| AC-002 | `release.yml` "Compute specs tarball checksum" step — `sha256sum >> checksums.txt` append | `AC-001-002-release-yml-tarball-steps.tape` + `.gif` | PASS |
| AC-003 | `install.sh --spec-dir` block: `curl` download + `grep || true` + `CHECKSUM_CMD` verify + `tar -xzf ... claroty.sensor.toml` | `AC-003-004-install-sh-spec-dir.tape` | PASS |
| AC-004 | `install.sh` no-clobber guard: `-f "${SPEC_TARGET}"` check + `--force-specs` override path | `AC-003-004-install-sh-spec-dir.tape` | PASS |
| AC-005 | `install.ps1 -SpecDir` block: `Invoke-WebRequest` + `Get-FileHash` + `tar.exe` + `$LASTEXITCODE` check | `AC-005-006-install-ps1-specdir.tape` | PASS |
| AC-006 | `install.ps1` no-clobber guard: `Test-Path $SpecTarget` check + `-ForceSpecs` override path | `AC-005-006-install-ps1-specdir.tape` | PASS |
| AC-007 | `resolve_config_paths()` in `boot.rs` — `config_dir.join(&config.spec_dir)` join logic; `prism.toml.example` `spec_dir = "./specs"` | Structural verification (code read) | CONFIRMED |
| AC-008 | `shellcheck scripts/install.sh` exit 0; PSScriptAnalyzer zero errors via CI gate | `AC-008-shellcheck-psscriptanalyzer.tape` | PASS |
| AC-009 | Post-merge LIVE-VERIFY (beta.2 release required — cannot record pre-merge) | N/A — post-merge human-in-loop gate | **PENDING post-merge** |

**Coverage: 8/9 ACs recorded or structurally verified. AC-009 is explicitly post-merge per story spec.**

---

## Recording Details

### AC-001/AC-002: `release.yml` tarball steps

**File:** `AC-001-002-release-yml-tarball-steps.tape` / `AC-001-002-release-yml-tarball-steps.gif`

Structural verification capture showing:
- `grep -A 10 "Create specs tarball"` output from `.github/workflows/release.yml` — confirms `tar czf "${SPECS_ARCHIVE}" -C crates/prism-sensors/specs claroty.sensor.toml` and `echo "SPECS_ARCHIVE=${SPECS_ARCHIVE}" >> "$GITHUB_ENV"` (CWE-78 hardening)
- `grep -A 5 "Compute specs tarball checksum"` output — confirms `sha256sum "$SPECS_ARCHIVE" >> checksums.txt`
- `grep -c '"$SPECS_ARCHIVE"'` in the `gh release create` and `gh release upload --clobber` arms — confirms both paths include the asset

AC-001: tarball layout verified: `tar czf ... -C crates/prism-sensors/specs claroty.sensor.toml` produces root-level file (no `specs/` prefix) as required.

AC-002: checksum append pattern `sha256sum "$SPECS_ARCHIVE" >> checksums.txt` is append-only (no truncation) — existing per-platform checksums are preserved.

---

### AC-003/AC-004: `install.sh --spec-dir` + no-clobber

**File:** `AC-003-004-install-sh-spec-dir.tape`

Structural verification capture showing:
- `grep -A 5 "\-\-spec-dir"` from `scripts/install.sh` argument-parsing loop — confirms `SPEC_DIR="${2}"; shift 2`
- The spec-placement block: `curl -fsSL ... "${TMPDIR_PRISM}/${SPECS_ARCHIVE}"`, `EXPECTED_SPEC="$(grep -F ... || true)"`, `tar -xzf ... -C "${SPEC_DIR}" claroty.sensor.toml` (SEC-001 explicit-filename fix present)
- No-clobber block: `if [[ -f "${SPEC_TARGET}" ]] && [[ "${FORCE_SPECS}" != "true" ]]`
- U-1 sibling sweep: pre-existing binary `EXPECTED="$(grep -F ... || true)"` — `|| true` guard present in both blocks

AC-003: download + SHA-256 verify (via `$CHECKSUM_CMD` dual-path) + explicit-filename extraction confirmed.
AC-004: no-clobber default + `--force-specs` override path confirmed.

---

### AC-005/AC-006: `install.ps1 -SpecDir` + no-clobber

**File:** `AC-005-006-install-ps1-specdir.tape`

Structural verification capture showing:
- `param()` block with `[string]$SpecDir = ""` and `[switch]$ForceSpecs`
- `Invoke-WebRequest -Uri $SpecsUrl -OutFile $SpecsArchivePath -UseBasicParsing -TimeoutSec 30`
- `Get-FileHash -Algorithm SHA256` + `$SpecsExpectedHash` comparison
- `if (-not (Get-Command tar.exe -ErrorAction SilentlyContinue))` absence guard (EC-008)
- `& tar.exe -xzf $SpecsArchivePath -C $SpecDir claroty.sensor.toml` (SEC-001 explicit-filename fix present)
- `if ($LASTEXITCODE -ne 0)` extraction result check
- No-clobber: `if ((Test-Path $SpecTarget) -and (-not $ForceSpecs))`
- `[regex]::Escape($SpecsArchive)` in checksum line-match loop (injection-safe pattern)

AC-005: download + Get-FileHash verify + tar.exe guard + explicit-filename extraction confirmed.
AC-006: no-clobber default + `-ForceSpecs` override path confirmed.

---

### AC-008: shellcheck + PSScriptAnalyzer

**File:** `AC-008-shellcheck-psscriptanalyzer.tape`

- `shellcheck scripts/install.sh` — exit code 0, zero errors or warnings
- PSScriptAnalyzer result: confirmed via CI gate (psscriptanalyzer-install-ps1 job in ci.yml)

---

### AC-007: Runtime discovery match (structural verification)

Read `crates/prism-bin/src/boot.rs` `resolve_config_paths()` function:
```rust
config.spec_dir = config_dir.join(&config.spec_dir);
```
`prism.toml.example` default: `spec_dir = "./specs"`. When install script places specs at `<spec-dir>/claroty.sensor.toml` and `prism.toml` is configured with `spec_dir = "<spec-dir>"` (absolute) or `spec_dir = "./specs"` (relative to config dir), the join logic correctly resolves the path. No code change required — confirmed match.

---

### AC-009: Post-merge LIVE-VERIFY

**Status: PENDING**

Cannot be recorded pre-merge. Requires:
1. v1.0.0-beta.2 GitHub Release published with `prism-specs-v1.0.0-beta.2.tar.gz` attached
2. Clean config directory
3. `bash scripts/install.sh --version v1.0.0-beta.2 --spec-dir <config-dir>/specs`
4. `prism.toml` configured with `spec_dir = "./specs"`
5. `prism start --config-dir <config-dir>` — confirm boot success and `prism_describe` returns Claroty tables

This is a manual human-in-loop verification gate post-beta.2 publish.
