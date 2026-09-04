---
document_type: story
story_id: S-REL-007
title: "devops: Windows PowerShell demo parity — demo-setup.ps1, demo-run.ps1, demo-teardown.ps1"
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
#   Windows demo scripts are distribution and demo-tooling artifacts with no subsystem
#   boundary in ARCH-INDEX. No ARCH-INDEX subsystem owns PowerShell packaging scripts.
#   subsystems: [] per S-0.01 infra story precedent.
crates_touched: [devops]
target_module: devops
capabilities: []
behavioral_contracts: []
# BC status: N/A — Windows PowerShell demo parity is distribution tooling infrastructure.
# No subsystem behavioral contract governs demo scripts. Conforming per W3-FIX-CI-001 precedent.
verification_properties: []
depends_on: [S-REL-004]
# Dependency anchor justifications:
#   depends_on S-REL-004: demo-setup.ps1 must reference the correct demo bundle directory
#     structure (prism-dtu-demo-server.exe, plugins/*.prx, specs/*.toml) defined and
#     assembled by S-REL-004. The PowerShell scripts mirror the bash scripts in layout.
blocks: [S-REL-005, S-REL-006]
# Dependency anchor justifications:
#   blocks S-REL-005: RELEASING.md documents Windows demo as part of the demo bundle;
#     the runbook cannot finalize the Windows section without the .ps1 scripts existing.
#   blocks S-REL-006: DEMO-RUNBOOK.md Windows update (graduated to docs/) depends on
#     the Windows scripts from this story being finalized.
points: 8
estimated_days: 5
risk: MEDIUM
# Risk justification: Windows PowerShell credential model differs from bash. Credential
# delivery via System.Diagnostics.Process+StreamWriter is a new pattern (no prior .ps1 in
# codebase). ConvertFrom-Json replaces Python3 dependency. DEMO-RUNBOOK.md Windows section
# is net-new. Test validation is manual (run on Windows CI runner or Windows machine).
# Per delta-analysis §8: "MEDIUM — Windows credential pattern is new; no .ps1 test
# infrastructure in repo".
acceptance_criteria_count: 12
red_gate_tests: 3
estimated_passes: "2-3 LOCAL adversary passes"
holdout_scenarios: []
assumption_validations: []
risk_mitigations:
  - "ConvertFrom-Json not Python3: delta-analysis §10 confirms PowerShell built-in
    ConvertFrom-Json is the correct approach for parsing JSON. Do NOT add Python3 as
    a Windows demo dependency."
  - "Credential delivery via System.Diagnostics.Process + StreamWriter (U31): credentials
    MUST be delivered via `$proc.StandardInput` + `New-Object System.IO.StreamWriter($proc
    .StandardInput.BaseStream, [System.Text.UTF8Encoding]::new($false)).Write($secret)`.
    Do NOT use `$secret | prism` (PowerShell pipes convert LF to CRLF and add trailing newline
    on PS 5.1, mangling credential bytes). Do NOT write credentials to disk or temp files."
  - "DTU URL resolution via sidecar file, not prism dtu status (U28): there is no
    `prism dtu status` command. demo-setup.ps1 reads the URL from the sidecar file
    `.prism-dtu-demo-server.urls-multi.json` written by prism-dtu-demo-server at startup.
    Poll for this file with a 30s timeout (1s intervals). Parse with Get-Content -Raw |
    ConvertFrom-Json + PSObject.Properties enumeration (PS 5.1 safe; no -AsHashtable)."
  - "Overlay TOML path via DEMO_CONFIG_DIR, not APPDATA hardcode (U32): do NOT hardcode
    %APPDATA%\\prism. Read prism's actual config-dir resolution from crates/prism-bin/src/main.rs.
    Prefer a DEMO_CONFIG_DIR environment variable (mirrors demo-setup.sh DEMO_CONFIG_DIR pattern).
    If DEMO_CONFIG_DIR is unset, compute from prism's actual config path, not a hardcoded constant."
  - "PS 5.1 compatibility throughout (U29/U30): all three .ps1 scripts must have
    `#Requires -Version 5.1` at the top. Do NOT use: -AsHashtable (ConvertFrom-Json),
    ternary operator (? : syntax), null-coalescing assignment (??=), ForEach-Object -Parallel.
    Use PSObject.Properties enumeration for JSON object property iteration."
  - "PSScriptAnalyzer CI step (U33): PSScriptAnalyzer is NOT pre-installed on windows-latest.
    CI must: `Install-Module -Name PSScriptAnalyzer -Scope CurrentUser -Force` before running
    `Invoke-ScriptAnalyzer -Path scripts/demo-setup.ps1 -Severity Error`."
  - "Syntax check via Parser::ParseFile (U31): Use
    `[System.Management.Automation.Language.Parser]::ParseFile` not `Get-Command -Syntax`
    (which only works for loaded functions). In CI, run ParseFile for each .ps1 to catch
    syntax errors before PSScriptAnalyzer."
  - "No Write-Host for credentials: use Write-Verbose or no output for credential handling
    steps. Write-Host leaks to terminal history."
  - "prism.exe path: the Windows demo bundle includes prism.exe (installed via S-REL-003
    install.ps1 or PATH), not a copy bundled inside. demo-setup.ps1 must check PATH first."
inputs:
  - ".factory/planning/feature-release-engineering/delta-analysis.md"
  - "scripts/demo-setup.sh"
  - "scripts/demo-run.sh"
  - "scripts/demo-teardown.sh"
  - "docs/DEMO-RUNBOOK.md"
  - ".factory/research/release-engineering-uncertainties-2026.md"
input-hash: "7890ffc"
traces_to: []
cycle: "v1.0.0-release-engineering"
phase: "F3"
---

# S-REL-007 — devops: Windows PowerShell demo parity

**Story ID:** S-REL-007
**Status:** draft
**Version:** v0.3
**Wave:** F-A
**Priority:** P0
**Points:** 8

**Deferral notice (2026-09-04):** DEFERRED out of v1.0.0-rc.1. S-REL-007 depends on S-REL-004, which is gated behind S-CLAROTY-DTU-PARITY-001 (Claroty DTU 14-table parity). Executes post-rc.1 after Claroty DTU parity lands.

---

## Origin

Delta-analysis §5 (Windows gap) and §10 (credential model): The existing demo is bash-only.
Windows MSSPs running Claude Code on Windows have no path to run the prism demo. The demo is
the RC acceptance gate for secops-factory customers. Windows must be a first-class platform.

Promoted from Wave F-C to F-A (delta-analysis §12 note) because the demo bundle (S-REL-004)
must include the .ps1 scripts, and the DEMO-RUNBOOK.md update (S-REL-006) depends on them.

---

## Narrative

As a secops-factory user on Windows, I want to run the prism DTU-backed demo using
PowerShell, so that I can evaluate prism on Windows without installing WSL or bash.

---

## Behavioral Contracts

This story has no subsystem BCs — Windows demo scripts are distribution tooling.

| Architecture Source | Clause |
|--------------------|--------|
| `delta-analysis.md` §5 (Windows gap) | Windows must be a first-class demo platform for RC |
| `delta-analysis.md` §10 (credential model) | stdin-pipe credentials via Process+StreamWriter; sidecar URL |
| `delta-analysis.md` §7 (bundle structure) | .ps1 scripts are included in the per-platform Windows .zip bundle |
| `delta-analysis.md` §11 story S-REL-007 scope | demo-setup.ps1, demo-run.ps1, demo-teardown.ps1 |

---

## Token Budget Estimate

| Artifact | Estimated Tokens |
|----------|-----------------|
| This story spec | ~4,000 |
| `delta-analysis.md` §5, §10, §11 | ~3,000 |
| `scripts/demo-setup.sh` (reference implementation) | ~2,000 |
| `scripts/demo-run.sh` (reference implementation) | ~1,500 |
| `scripts/demo-teardown.sh` (reference implementation) | ~800 |
| `docs/DEMO-RUNBOOK.md` (existing content) | ~3,000 |
| `release-engineering-uncertainties-2026.md` U22/U28-U33 | ~1,500 |
| Total | ~15,800 |

Within the 30% context window budget. Implementer must read both the bash originals and the
delta-analysis §10 credential model before writing the PowerShell equivalents.

---

## Tasks

1. **Read `scripts/demo-setup.sh`, `scripts/demo-run.sh`, `scripts/demo-teardown.sh`** in full.

2. **Read `delta-analysis.md` §5, §10** for Windows-specific requirements.

3. **Read `docs/DEMO-RUNBOOK.md`** for the existing bash walkthrough structure to mirror.

4. **Read `crates/prism-bin/src/main.rs`** to determine how prism resolves its config
   directory on Windows (this informs the DEMO_CONFIG_DIR default for the overlay TOML).

5. **Create `scripts/demo-setup.ps1`** (PowerShell equivalent of demo-setup.sh):
   - `#Requires -Version 5.1` at the top (U29)
   - `[CmdletBinding()]`, `Set-StrictMode -Version Latest`, `$ErrorActionPreference = 'Stop'`
   - Accepts same parameters as demo-setup.sh (API base URL, prism binary path)
   - Checks for `prism` on PATH; if not found, prints guidance to run install.ps1 first
   - Starts prism-dtu-demo-server.exe in the background:
     ```powershell
     $serverProc = Start-Process -FilePath ".\prism-dtu-demo-server.exe" -PassThru
     ```
   - **DTU URL resolution via sidecar file (U28):** do NOT call `prism dtu status` (no such
     command). Instead poll `.prism-dtu-demo-server.urls-multi.json` with 30s timeout:
     ```powershell
     $sidecarPath = ".prism-dtu-demo-server.urls-multi.json"
     $deadline = [DateTime]::UtcNow.AddSeconds(30)
     while (-not (Test-Path $sidecarPath)) {
         if ([DateTime]::UtcNow -gt $deadline) {
             Write-Error "DTU server did not write sidecar within 30s"
             exit 1
         }
         Start-Sleep -Seconds 1
     }
     # PS 5.1-safe: PSObject.Properties enumeration, no -AsHashtable (U30)
     $urls = Get-Content -Raw $sidecarPath | ConvertFrom-Json
     foreach ($prop in $urls.PSObject.Properties) {
         Write-Verbose "DTU endpoint: $($prop.Name) = $($prop.Value)"
     }
     ```
   - **Overlay TOML path via DEMO_CONFIG_DIR (U32):** do NOT hardcode `%APPDATA%\prism`.
     Read prism's actual config-dir resolution from `crates/prism-bin/src/main.rs`. Use:
     ```powershell
     if ($env:DEMO_CONFIG_DIR) {
         $configDir = $env:DEMO_CONFIG_DIR
     } else {
         # Mirror prism's actual config resolution from crates/prism-bin/src/main.rs
         # (implementer: replace this comment with the actual resolution logic found there)
         $configDir = "<read from prism-bin source>"
     }
     $overlayPath = Join-Path $configDir "demo-override.toml"
     ```
   - Generates overlay TOML at `$overlayPath` extending production sensor spec with DTU
     allowed_urls (mirrors demo-setup.sh's overlay generation)
   - **Credential delivery via Process+StreamWriter (U31):** do NOT use `$secret | prism`
     (PowerShell pipes mangle UTF-8 bytes on PS 5.1). Use:
     ```powershell
     $proc = [System.Diagnostics.Process]::new()
     $proc.StartInfo.FileName = "prism"
     $proc.StartInfo.RedirectStandardInput = $true
     $proc.StartInfo.UseShellExecute = $false
     [void]$proc.Start()
     $enc = [System.Text.UTF8Encoding]::new($false)  # no-BOM UTF-8
     $writer = [System.IO.StreamWriter]::new($proc.StandardInput.BaseStream, $enc)
     $writer.Write($secret)  # no trailing newline
     $writer.Close()
     $proc.WaitForExit()
     ```
   - Verifies demo server is healthy via `Invoke-WebRequest -Uri "http://127.0.0.1:<port>/health" -UseBasicParsing`
   - Stores server PID to `$env:TEMP\prism-demo-server.pid` for teardown

6. **Create `scripts/demo-run.ps1`** (PowerShell equivalent of demo-run.sh):
   - `#Requires -Version 5.1` at the top (U29)
   - Same structural conventions as demo-setup.ps1
   - Checks that demo server is still running (check PID from temp file)
   - Runs the same demo query sequence as demo-run.sh against the MCP interface
   - Parses JSON responses using ConvertFrom-Json with PSObject.Properties enumeration (5.1-safe)
   - No `-AsHashtable` (PS 7.0+ only) (U30)
   - Displays formatted output matching the bash script's output structure
   - Exit 0 on success; non-zero on any query failure

7. **Create `scripts/demo-teardown.ps1`** (PowerShell equivalent of demo-teardown.sh):
   - `#Requires -Version 5.1` at the top (U29)
   - Reads PID from `$env:TEMP\prism-demo-server.pid`
   - Stops `prism-dtu-demo-server.exe` (Stop-Process -Id $pid -ErrorAction SilentlyContinue)
   - Removes the overlay TOML generated by demo-setup.ps1
   - Removes the PID temp file
   - Removes `.prism-dtu-demo-server.urls-multi.json` sidecar if present
   - Prints confirmation

8. **Add Windows section to `docs/DEMO-RUNBOOK.md`**:
   - Add `## Windows (PowerShell)` section after the existing bash section
   - Mirrors the bash section structure: Prerequisites → Setup → Run → Teardown
   - Links to install.ps1 (S-REL-003) for installation
   - Documents the DEMO_CONFIG_DIR environment variable pattern (U32)
   - Explains credential delivery via Process+StreamWriter (no `$secret | prism`)
   - Notes ConvertFrom-Json as the JSON parsing method (no Python3 required)
   - Notes Windows AV/Defender may require an exclusion for prism-dtu-demo-server.exe

9. **CI step for PSScriptAnalyzer (U33):** add to ci.yml (or release.yml smoke test job):
   ```yaml
   - name: Install PSScriptAnalyzer
     shell: pwsh
     run: Install-Module -Name PSScriptAnalyzer -Scope CurrentUser -Force
   - name: Lint PowerShell demo scripts
     shell: pwsh
     run: |
       foreach ($f in @('scripts/demo-setup.ps1','scripts/demo-run.ps1','scripts/demo-teardown.ps1')) {
         $errors = @()
         $null = [System.Management.Automation.Language.Parser]::ParseFile($f, [ref]$null, [ref]$errors)
         if ($errors.Count -gt 0) { throw "Parse error in $f" }
         Invoke-ScriptAnalyzer -Path $f -Severity Error
       }
   ```

---

## Acceptance Criteria

### AC-001: demo-setup.ps1 exists, has #Requires -Version 5.1, and passes Parser::ParseFile
Given: `scripts/demo-setup.ps1` is committed.
When: The following check is run in CI (U31):
  `$errors = @(); [void][System.Management.Automation.Language.Parser]::ParseFile("scripts/demo-setup.ps1", [ref]$null, [ref]$errors); if ($errors.Count -gt 0) { throw "parse error" }`
Then: Zero parse errors. Script has `#Requires -Version 5.1`, `[CmdletBinding()]`,
`Set-StrictMode -Version Latest`, and `$ErrorActionPreference = 'Stop'` at the top.
(traces to delta-analysis.md §11 S-REL-007: "demo-setup.ps1"; research U29/U31)

### AC-002: demo-run.ps1 exists, has #Requires -Version 5.1, and passes Parser::ParseFile
Given: `scripts/demo-run.ps1` is committed.
When: Parser::ParseFile check is run (same as AC-001 check for this file).
Then: Zero parse errors. Same structural conventions as demo-setup.ps1.
(traces to delta-analysis.md §11 S-REL-007: "demo-run.ps1"; research U29)

### AC-003: demo-teardown.ps1 exists, has #Requires -Version 5.1, and passes Parser::ParseFile
Given: `scripts/demo-teardown.ps1` is committed.
When: Parser::ParseFile check is run.
Then: Zero parse errors. Same structural conventions.
(traces to delta-analysis.md §11 S-REL-007: "demo-teardown.ps1"; research U29)

### AC-004: ConvertFrom-Json used for JSON parsing; no Python3 dependency; no -AsHashtable
Given: All three .ps1 scripts are read.
When: Searches for Python3 and -AsHashtable are run.
Then: Zero Python3 references. Zero `-AsHashtable` occurrences (PS 7.0+ only). JSON is
parsed exclusively with `ConvertFrom-Json`. PSObject.Properties enumeration used for
JSON object property iteration throughout.
(traces to delta-analysis.md §10: "ConvertFrom-Json replaces Python3 dependency on Windows";
research U30: no -AsHashtable)

### AC-005: Credential delivery via Process+StreamWriter; no $secret pipe to exe
Given: `scripts/demo-setup.ps1` is read.
When: Credential delivery code is inspected.
Then: The script uses `System.Diagnostics.Process` + `System.IO.StreamWriter` with
`UTF8Encoding::new($false)` (no-BOM) for credential delivery. No `$secret | prism` or
`$secret | &` pipe pattern. No credential values written to temp files.
(traces to delta-analysis.md §10: "stdin-pipe credential pattern"; research U31)

### AC-006: DTU URL resolved by polling sidecar file, not via prism dtu status
Given: demo-setup.ps1 is read.
When: The DTU URL resolution code is inspected.
Then: No `prism dtu status` call (no such command per U28). Instead, the script polls
`.prism-dtu-demo-server.urls-multi.json` with a 30-second timeout (1s poll interval),
reads it with `Get-Content -Raw | ConvertFrom-Json`, and enumerates properties via
`PSObject.Properties` (5.1-safe; no -AsHashtable).
(traces to delta-analysis.md §10 + research U28: sidecar-based URL resolution)

### AC-007: Overlay TOML path uses DEMO_CONFIG_DIR pattern, not %APPDATA% hardcode
Given: demo-setup.ps1 is read.
When: The overlay TOML path construction code is inspected.
Then: The script checks `$env:DEMO_CONFIG_DIR` first; falls back to prism's actual
config-dir resolution read from `crates/prism-bin/src/main.rs`. Does NOT hardcode
`$env:APPDATA\prism` or `%APPDATA%\prism`.
(traces to delta-analysis.md §10 overlay TOML pattern; research U32: no APPDATA hardcode)

### AC-008: demo-teardown.ps1 stops the demo server and cleans up sidecar
Given: demo-setup.ps1 has run and the demo server is running.
When: `scripts/demo-teardown.ps1` is run.
Then: prism-dtu-demo-server.exe process is stopped; overlay TOML removed; PID file removed;
`.prism-dtu-demo-server.urls-multi.json` sidecar removed. Exit code 0. No leftover processes.
(traces to delta-analysis.md §11 S-REL-007: "demo-teardown.ps1")

### AC-009: No Write-Host for credential-adjacent output
Given: All three .ps1 scripts are read.
When: `Select-String -Pattern 'Write-Host' scripts/demo-setup.ps1` etc. is run.
Then: Zero matches involving credential values. Informational messages use `Write-Verbose`
or `Write-Information`. No credential value is ever echoed to host.
(traces to risk_mitigations: "no Write-Host for credentials")

### AC-010: DEMO-RUNBOOK.md has a Windows (PowerShell) section
Given: `docs/DEMO-RUNBOOK.md` is read.
When: `Select-String -Pattern 'Windows' docs/DEMO-RUNBOOK.md` is run.
Then: A `## Windows (PowerShell)` section exists with subsections for Prerequisites, Setup,
Run, and Teardown. The section documents the DEMO_CONFIG_DIR pattern, credential delivery
via Process+StreamWriter, and ConvertFrom-Json for JSON parsing (no Python3).
(traces to delta-analysis.md §11 S-REL-007: "DEMO-RUNBOOK.md Windows update")

### AC-011: .ps1 scripts are included in the Windows demo bundle (.zip format)
Given: The build-demo-bundle CI job for `x86_64-pc-windows-msvc` completes.
When: `unzip -l prism-demo-bundle-${TAG}-x86_64-pc-windows-msvc.zip | grep '.ps1'` is run
(or PowerShell: `[System.IO.Compression.ZipFile]::OpenRead($zip).Entries | where Name -match ps1`).
Then: `demo-setup.ps1`, `demo-run.ps1`, and `demo-teardown.ps1` are listed in the .zip archive.
(traces to delta-analysis.md §7: ".ps1 scripts in Windows demo bundle"; research U22: .zip not .tar.gz)

### AC-012: .sh scripts are NOT in Windows bundle and .ps1 scripts are NOT in Unix bundles
Given: The demo bundle archives for all 5 platforms.
When: Cross-platform script contamination check is run:
  - Windows .zip: `unzip -l ...windows...zip | grep '\.sh'` → zero results
  - Unix .tar.gz (Linux/macOS): `tar tzf ...<unix-target>...tar.gz | grep '\.ps1'` → zero results
Then: Each platform bundle contains only the scripts appropriate for its shell environment.
(traces to delta-analysis.md §7: per-platform bundle; Windows gets .ps1, Unix gets .sh;
research U22: Windows bundle is .zip)

---

## Previous Story Intelligence

S-REL-004 defines the demo bundle structure. The implementer MUST read S-REL-004 and the
assembled bundle manifest before writing demo-setup.ps1 to ensure the path references
(prism-dtu-demo-server.exe, plugins/, specs/) match exactly.

S-DEMO-003 (demo-setup.sh) is the canonical bash reference. The PowerShell scripts must
match the bash scripts' behavioral semantics exactly — same setup sequence, same query
sequence, same teardown sequence.

Key lessons from fix-burst research:
- U28: No `prism dtu status` command exists. Poll `.prism-dtu-demo-server.urls-multi.json`
  sidecar (written by demo server at startup) with 30s timeout. Parse with PSObject.Properties.
- U29: `#Requires -Version 5.1` at top of every .ps1 file — this is mandatory, not optional.
- U30: No `-AsHashtable` in ConvertFrom-Json (requires PS 7.0); use PSObject.Properties.
- U31: Credential delivery via System.Diagnostics.Process + StreamWriter(UTF8 no-BOM).Write().
  PowerShell pipe ($secret | exe) mangles UTF-8 on PS 5.1 (CRLF conversion, trailing newline).
- U32: Do NOT hardcode %APPDATA%\prism. Read prism's config-dir resolution from source.
- U33: PSScriptAnalyzer not pre-installed; install explicitly in CI.

---

## Architecture Compliance Rules

| Rule | Source | Enforcement |
|------|--------|-------------|
| ConvertFrom-Json, not Python3 | delta-analysis §10 | AC-004 no Python grep |
| PSObject.Properties enumeration, no -AsHashtable | Research U30 | AC-004 |
| Process+StreamWriter for credential delivery | Research U31 | AC-005 |
| Sidecar file for DTU URL, not prism dtu status | Research U28 | AC-006 |
| DEMO_CONFIG_DIR, no APPDATA hardcode | Research U32 | AC-007 |
| PS 5.1 #Requires at top of every script | Research U29 | AC-001/002/003 |
| Parser::ParseFile for syntax check in CI | Research U31 | AC-001/002/003 |
| No Write-Host for credential output | risk_mitigations | AC-009 |
| Windows bundle gets .ps1 only, is .zip | Research U22 | AC-011/012 |
| Unix bundles get .sh only, are .tar.gz | Research U22 | AC-012 |
| Set-StrictMode + ErrorActionPreference | PowerShell production convention | AC-001/002/003 |

---

## Library & Framework Requirements

| Tool | Version | Notes |
|------|---------|-------|
| PowerShell | 5.1+ minimum | #Requires -Version 5.1 in every .ps1; avoid 7.0-only features |
| ConvertFrom-Json | Built-in (PS 5+) | No -AsHashtable; use PSObject.Properties for iteration |
| System.Diagnostics.Process | .NET (PS 5.1+) | Credential delivery via StandardInput+StreamWriter |
| System.IO.StreamWriter | .NET (PS 5.1+) | UTF8Encoding::new($false) — no-BOM UTF-8 |
| Invoke-WebRequest | Built-in | Health check; -UseBasicParsing (no-op on PS 7.x, needed on 5.1) |
| Start-Process | Built-in | Background process launch for demo server |
| PSScriptAnalyzer | Via Install-Module | Must be explicitly installed in CI on windows-latest |

---

## File Structure Requirements

| File | Action | Notes |
|------|--------|-------|
| `scripts/demo-setup.ps1` | Create | PowerShell demo setup; #Requires -Version 5.1; Process+StreamWriter credentials |
| `scripts/demo-run.ps1` | Create | PowerShell demo run; PSObject.Properties for JSON |
| `scripts/demo-teardown.ps1` | Create | PowerShell demo teardown; cleans up sidecar |
| `docs/DEMO-RUNBOOK.md` | Modify | Add Windows (PowerShell) section with DEMO_CONFIG_DIR docs |

---

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|----------------|
| `scripts/demo-setup.ps1` | `scripts/` | N/A (PowerShell script) |
| `scripts/demo-run.ps1` | `scripts/` | N/A (PowerShell script) |
| `scripts/demo-teardown.ps1` | `scripts/` | N/A (PowerShell script) |
| `docs/DEMO-RUNBOOK.md` (Windows section) | `docs/` | N/A (documentation) |

---

## Purity Classification

| Module | Classification | Justification |
|--------|----------------|---------------|
| `scripts/demo-setup.ps1` | N/A | PowerShell script — no Rust purity boundary applies |
| `scripts/demo-run.ps1` | N/A | PowerShell script — no Rust purity boundary applies |
| `scripts/demo-teardown.ps1` | N/A | PowerShell script — no Rust purity boundary applies |
| `docs/DEMO-RUNBOOK.md` | N/A | Documentation — no Rust purity boundary applies |

---

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | prism not on PATH on Windows | demo-setup.ps1 prints install.ps1 guidance and exits non-zero |
| EC-002 | demo server port already in use | demo-setup.ps1 detects port conflict and prints error |
| EC-003 | PID file missing when teardown runs | demo-teardown.ps1 prints warning, attempts process name search as fallback |
| EC-004 | PS 5.1 vs 7.x differences | #Requires -Version 5.1; avoid all 7.0+ features; test confirmed 5.1-safe patterns |
| EC-005 | Sidecar file not created within 30s | demo-setup.ps1 errors with clear message: demo server did not start |
| EC-006 | APPDATA path with spaces | All file paths quoted or via Join-Path; no string concatenation with paths |
| EC-007 | Windows Defender blocking demo server | Document in DEMO-RUNBOOK.md Windows prerequisites |

---

## Forbidden Dependencies

- No Python3 in any .ps1 script (AC-004)
- No bash or sh subprocess calls from .ps1 scripts
- No WSL dependencies
- No third-party PowerShell modules (must work on bare Windows + pwsh install; PSScriptAnalyzer only for CI)
- No chocolatey or winget calls (installation handled by install.ps1 from S-REL-003)
- No `-AsHashtable` on ConvertFrom-Json (PS 7.0+ only)
- No `$secret | prism` pipe for credential delivery (UTF-8 mangling on PS 5.1)
- No `prism dtu status` command (does not exist)
- No `%APPDATA%\prism` hardcoded path

---

## Changelog

| Version | Date | Summary |
|---------|------|---------|
| 0.3 | 2026-09-04 | DEFERRED out of v1.0.0-rc.1 as downstream consequence of S-REL-004 deferral. S-REL-007 depends_on S-REL-004; S-REL-004 is gated behind S-CLAROTY-DTU-PARITY-001 (Claroty DTU 14-table parity). Human decision 2026-09-04. Executes post-rc.1 after Claroty DTU parity lands. |
| 0.2 | 2026-07-19 | Fix-burst: U22 Windows bundle is .zip (AC-011/012 updated); U28 DTU URL via sidecar poll not prism dtu status; U29 #Requires -Version 5.1 mandatory; U30 no -AsHashtable + PSObject.Properties; U31 credential delivery via Process+StreamWriter (no $secret pipe) + Parser::ParseFile for syntax check; U32 DEMO_CONFIG_DIR not %APPDATA%; U33 PSScriptAnalyzer explicit install; research file added to inputs |
| 0.1 | 2026-07-19 | Initial story creation (story-writer F3 burst) |
