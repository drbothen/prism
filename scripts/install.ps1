#Requires -Version 5.1
# scripts/install.ps1 - Checksum-verified installer for Prism on Windows (PowerShell 5.1+).
#
# USAGE
#   irm https://raw.githubusercontent.com/drbothen/prism/main/scripts/install.ps1 | iex
#   iex (irm 'https://raw.githubusercontent.com/drbothen/prism/main/scripts/install.ps1')
#
#   With version pin (set env var before piping - positional args cannot be passed through iex):
#     $env:PRISM_INSTALL_VERSION = '<version>'; irm .../install.ps1 | iex
#
#   Direct invocation with parameters:
#     pwsh -File scripts/install.ps1 -Version <version>
#     pwsh -File scripts/install.ps1 -DryRun
#
# PLATFORM
#   Always installs the x86_64-pc-windows-msvc target.
#   Requires PowerShell 5.1 or later (the #Requires directive enforces this).
#
# WHAT IT DOES
#   1. Resolves the latest release via GitHub REST API (includes prereleases).
#   2. Downloads prism-<version>-x86_64-pc-windows-msvc.zip and checksums.txt.
#   3. Verifies the SHA-256 checksum; exits non-zero on mismatch.
#   4. Extracts prism.exe and installs it to %LOCALAPPDATA%\prism\bin\.
#   5. Prints PATH guidance if the install directory is not in PATH.
#
# SECURITY
#   - Checksum mismatch aborts install immediately (exit code 1).
#   - No gh CLI dependency; uses GitHub REST API for version resolution (auth-free).
#   - Uses PSObject.Properties enumeration for JSON parsing (5.1-safe; no -AsHashtable).
#   - No credential piping; install scripts handle only binary archives (U31, AD-017).
#   - Temp directory is always cleaned up in the finally block.
#
# Stories: S-REL-003 | ACs: AC-005..AC-007

[CmdletBinding()]
param(
    [string]$Version = "",
    [switch]$DryRun,
    [string]$SpecDir = "",
    [switch]$ForceSpecs
)

$ErrorActionPreference = "Stop"

# N3: Runtime version check — #Requires is inert under 'irm | iex' (the primary documented path).
# Abort early with a clear message on PowerShell < 5.1.
if ($PSVersionTable.PSVersion.Major -lt 5 -or
    ($PSVersionTable.PSVersion.Major -eq 5 -and $PSVersionTable.PSVersion.Minor -lt 1)) {
    Write-Error "PowerShell 5.1 or later is required. Current version: $($PSVersionTable.PSVersion)"
    exit 1
}

# N2: Force TLS 1.2 for PS 5.1 compatibility.
# PS 5.1 on older Windows defaults to TLS 1.0/1.1 for .NET WebClient/ServicePoint; GitHub
# requires TLS 1.2+. Append Tls12 with -bor to avoid disabling other negotiated protocols.
[Net.ServicePointManager]::SecurityProtocol = [Net.ServicePointManager]::SecurityProtocol -bor [Net.SecurityProtocolType]::Tls12

$Repo = "drbothen/prism"
$Target = "x86_64-pc-windows-msvc"
$InstallDir = Join-Path $env:LOCALAPPDATA "prism\bin"

# ---------------------------------------------------------------------------
# Version resolution (U8: /releases/latest excludes prereleases; no gh CLI dep)
# ---------------------------------------------------------------------------

# Allow version override via environment variable for irm | iex usage (U8).
# Positional args cannot be carried through iex; env var is the documented pattern.
if (-not $Version) {
    if ($env:PRISM_INSTALL_VERSION) {
        $Version = $env:PRISM_INSTALL_VERSION
    } else {
        $ReleasesJson = Invoke-WebRequest `
            -Uri "https://api.github.com/repos/$Repo/releases?per_page=1" `
            -UseBasicParsing `
            -TimeoutSec 30 `
            -ErrorAction Stop
        # U30: PSObject.Properties enumeration — 5.1-safe, no -AsHashtable (requires PS 7.0+)
        $Releases = $ReleasesJson.Content | ConvertFrom-Json
        $Version = $Releases[0].tag_name
        if (-not $Version) {
            Write-Error "Failed to resolve latest release version from GitHub API."
            exit 1
        }
    }
}

# SEC-005/SEC-007: validate Version format before URL/path construction — full semver anchor
# rejects prefix-only matches (e.g. "v1evil") and non-semver forms (e.g. "v1", "v1.0").
if ($Version -notmatch '^v[0-9]+\.[0-9]+\.[0-9]+(-[a-zA-Z0-9.]+)?$') {
    Write-Error "Version must be a full semver tag (e.g. v1.0.0 or v1.0.0-<channel>.N); got: $Version"
    exit 1
}

$Archive = "prism-$Version-$Target.zip"
$Url = "https://github.com/$Repo/releases/download/$Version/$Archive"
$ChecksumUrl = "https://github.com/$Repo/releases/download/$Version/checksums.txt"

if ($DryRun) {
    Write-Host "Dry run - would download:"
    Write-Host "  Archive:   $Url"
    Write-Host "  Checksums: $ChecksumUrl"
    Write-Host "  Target:    $Target"
    Write-Host "  Version:   $Version"
    exit 0
}

# ---------------------------------------------------------------------------
# Temp directory
# ---------------------------------------------------------------------------
$TempRoot = [System.IO.Path]::GetTempPath()
$TempPath = Join-Path $TempRoot ([System.IO.Path]::GetRandomFileName())
New-Item -ItemType Directory -Path $TempPath | Out-Null

try {
    $ArchivePath = Join-Path $TempPath $Archive
    $ChecksumPath = Join-Path $TempPath "checksums.txt"

    # ---------------------------------------------------------------------------
    # Download
    # ---------------------------------------------------------------------------
    Write-Host "Downloading prism $Version for Windows ($Target)..."
    Invoke-WebRequest -Uri $Url -OutFile $ArchivePath -UseBasicParsing -TimeoutSec 30 -ErrorAction Stop
    Invoke-WebRequest -Uri $ChecksumUrl -OutFile $ChecksumPath -UseBasicParsing -TimeoutSec 30 -ErrorAction Stop

    # ---------------------------------------------------------------------------
    # SHA-256 verification (AC-006: abort on mismatch)
    # Get-FileHash is built into PowerShell 5+ - no external dependency.
    # ---------------------------------------------------------------------------
    $ActualHash = (Get-FileHash -Algorithm SHA256 -Path $ArchivePath).Hash.ToLower()
    $ChecksumLines = Get-Content -Path $ChecksumPath

    $MatchedLine = $null
    foreach ($Line in $ChecksumLines) {
        if ($Line -match [regex]::Escape($Archive)) {
            $MatchedLine = $Line
            break
        }
    }

    if ($null -eq $MatchedLine) {
        Write-Host "ERROR: $Archive not found in checksums.txt" -ForegroundColor Red
        exit 1
    }

    # checksums.txt format: "<hash>  <filename>" (sha256sum/shasum output)
    $Parts = $MatchedLine -split '\s+'
    $ExpectedHash = $Parts[0].ToLower()

    if ($ActualHash -ne $ExpectedHash) {
        Write-Host "ERROR: Checksum mismatch for $Archive" -ForegroundColor Red
        Write-Host "  Expected: $ExpectedHash" -ForegroundColor Red
        Write-Host "  Actual:   $ActualHash" -ForegroundColor Red
        exit 1
    }

    Write-Host "Checksum verified."

    # ---------------------------------------------------------------------------
    # Extract and install
    # ---------------------------------------------------------------------------
    New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
    Expand-Archive -Path $ArchivePath -DestinationPath $TempPath -Force

    $ExtractedExe = Join-Path $TempPath "prism.exe"
    if (-not (Test-Path $ExtractedExe)) {
        Write-Error "ERROR: prism.exe not found in archive after extraction."
        exit 1
    }

    $DestExe = Join-Path $InstallDir "prism.exe"
    Copy-Item -Path $ExtractedExe -Destination $DestExe -Force

    Write-Host "prism installed to $DestExe"

    # ---------------------------------------------------------------------------
    # PATH guidance
    # ---------------------------------------------------------------------------
    $UserPath = [Environment]::GetEnvironmentVariable("PATH", "User")
    if ($null -eq $UserPath -or $UserPath -notlike "*$InstallDir*") {
        # B3: deduplicate — append only to user-scoped PATH; never reference machine PATH.
        $NewUserPath = if ($null -ne $UserPath -and $UserPath.Length -gt 0) { "$InstallDir;$UserPath" } else { $InstallDir }
        Write-Host ""
        Write-Host "NOTE: $InstallDir is not in your PATH."
        Write-Host "  To add it permanently (user scope only — no machine PATH duplication), run:"
        Write-Host "  [Environment]::SetEnvironmentVariable('PATH', '$NewUserPath', 'User')"
        Write-Host "  Then restart your terminal."
    }

    # ---------------------------------------------------------------------------
    # Confirm version
    # ---------------------------------------------------------------------------
    if (Test-Path $DestExe) {
        try {
            $VersionOutput = & $DestExe --version 2>&1
            Write-Host "Version: $VersionOutput"
        } catch {
            Write-Host "Version: $Version"
        }
    }

    # ---------------------------------------------------------------------------
    # Spec placement (AC-005, AC-006)
    # ---------------------------------------------------------------------------
    if ($SpecDir) {
        New-Item -ItemType Directory -Path $SpecDir -Force | Out-Null
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
            & tar.exe -xzf $SpecsArchivePath -C $SpecDir
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

    # ---------------------------------------------------------------------------
    # Post-install notice
    # ---------------------------------------------------------------------------
    Write-Host ""
    Write-Host "Configuration: obtain prism.toml.example from the repository:"
    Write-Host "  https://github.com/$Repo/blob/main/prism.toml.example"

} finally {
    # Always clean up temp directory
    if (Test-Path $TempPath) {
        Remove-Item -Path $TempPath -Recurse -Force -ErrorAction SilentlyContinue
    }
}
