# VHS Demo — S-REL-SPECS-TARBALL-001
# AC-005: install.ps1 -SpecDir <path> downloads, verifies, and places claroty.sensor.toml
# AC-006: install.ps1 idempotency — existing spec not clobbered without -ForceSpecs
#
# Evidence: show -SpecDir and -ForceSpecs param block, spec-placement block,
#            no-clobber logic, tar.exe absence guard (EC-008).
# Usage: vhs AC-005-006-install-ps1-specdir.tape

Output docs/demo-evidence/S-REL-SPECS-TARBALL-001/AC-005-006-install-ps1-specdir.gif
Output docs/demo-evidence/S-REL-SPECS-TARBALL-001/AC-005-006-install-ps1-specdir.webm

Require grep

Set Shell "zsh"
Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 16
Set Width 1200
Set Height 500
Set Theme "Catppuccin Mocha"
Set WindowBar Colorful
Set BorderRadius 8
Set Padding 20
Set Framerate 30
Set TypingSpeed 40ms

Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-REL-SPECS-TARBALL-001"
Enter
Sleep 1s
Type "clear"
Enter
Sleep 500ms
Show

# Show -SpecDir and -ForceSpecs parameter block (AC-005, AC-006)
Type "grep -n 'SpecDir\|ForceSpecs' scripts/install.ps1 | head -18"
Enter
Sleep 2s

# Show no-clobber guard and tar.exe absence check (AC-006, EC-008)
Type "grep -n 'already exists; skipping\|ForceSpecs to overwrite\|tar.exe.*SilentlyContinue\|Sensor specs not installed' scripts/install.ps1"
Enter
Sleep 3s
