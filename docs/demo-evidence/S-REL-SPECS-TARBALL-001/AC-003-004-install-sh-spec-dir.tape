# VHS Demo — S-REL-SPECS-TARBALL-001
# AC-003: install.sh --spec-dir <path> downloads, verifies, and places claroty.sensor.toml
# AC-004: install.sh idempotency — existing spec not clobbered without --force-specs
#
# Evidence: show --spec-dir argument parsing, spec-placement block,
#            no-clobber logic, and the error path for missing --spec-dir value.
# Usage: vhs AC-003-004-install-sh-spec-dir.tape

Output docs/demo-evidence/S-REL-SPECS-TARBALL-001/AC-003-004-install-sh-spec-dir.gif
Output docs/demo-evidence/S-REL-SPECS-TARBALL-001/AC-003-004-install-sh-spec-dir.webm

Require bash
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

# Show --spec-dir and --force-specs argument parsing (AC-003, AC-004)
Type "grep -n 'spec-dir\|force-specs\|SPEC_DIR\|FORCE_SPECS' scripts/install.sh | head -18"
Enter
Sleep 2s

# Show no-clobber guard (AC-004)
Type "grep -n 'already exists; skipping\|force-specs to overwrite\|Sensor specs not installed' scripts/install.sh"
Enter
Sleep 2s

# Error path: --spec-dir with missing value exits non-zero with clear message
Type "bash scripts/install.sh --spec-dir 2>&1; echo exit:$?"
Enter
Sleep 3s
