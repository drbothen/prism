#Requires -Version 5.1
# scripts/install.ps1 - Checksum-verified installer for Prism on Windows (PowerShell 5.1+).
#
# USAGE
#   irm https://raw.githubusercontent.com/drbothen/prism/main/scripts/install.ps1 | iex
#   iex (irm 'https://raw.githubusercontent.com/drbothen/prism/main/scripts/install.ps1')
#
#   With version pin (set env var before piping - positional args cannot be passed through iex):
#     $env:PRISM_INSTALL_VERSION = 'v1.0.0-rc.1'; irm .../install.ps1 | iex
#
#   Direct invocation with parameters:
#     pwsh -File scripts/install.ps1 -Version v1.0.0-rc.1
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
    [switch]$DryRun
)

$ErrorActionPreference = "Stop"

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
    Invoke-WebRequest -Uri $Url -OutFile $ArchivePath -UseBasicParsing -ErrorAction Stop
    Invoke-WebRequest -Uri $ChecksumUrl -OutFile $ChecksumPath -UseBasicParsing -ErrorAction Stop

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
        Write-Error "ERROR: $Archive not found in checksums.txt"
        exit 1
    }

    # checksums.txt format: "<hash>  <filename>" (sha256sum/shasum output)
    $Parts = $MatchedLine -split '\s+'
    $ExpectedHash = $Parts[0].ToLower()

    if ($ActualHash -ne $ExpectedHash) {
        Write-Error "ERROR: Checksum mismatch for $Archive"
        Write-Error "  Expected: $ExpectedHash"
        Write-Error "  Actual:   $ActualHash"
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
        Write-Host ""
        Write-Host "NOTE: $InstallDir is not in your PATH."
        Write-Host "  To add it permanently, run the following in an elevated terminal:"
        Write-Host "  [Environment]::SetEnvironmentVariable('PATH', `"$InstallDir;`$env:PATH`", 'User')"
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

} finally {
    # Always clean up temp directory
    if (Test-Path $TempPath) {
        Remove-Item -Path $TempPath -Recurse -Force -ErrorAction SilentlyContinue
    }
}
