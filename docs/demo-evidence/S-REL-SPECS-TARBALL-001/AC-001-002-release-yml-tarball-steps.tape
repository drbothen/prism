# VHS Demo — S-REL-SPECS-TARBALL-001
# AC-001: prism-specs-<tag>.tar.gz is built and attached to GitHub Release
# AC-002: prism-specs-<tag>.tar.gz checksum is in checksums.txt
#
# Evidence: show the two new steps + SPECS_ARCHIVE usage in release.yml
# Usage: vhs AC-001-002-release-yml-tarball-steps.tape

Output docs/demo-evidence/S-REL-SPECS-TARBALL-001/AC-001-002-release-yml-tarball-steps.gif
Output docs/demo-evidence/S-REL-SPECS-TARBALL-001/AC-001-002-release-yml-tarball-steps.webm

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

# AC-001 + AC-002: show "Create specs tarball" and "Compute specs tarball checksum" steps
Type "grep -n 'Create specs tarball\|Compute specs tarball' .github/workflows/release.yml"
Enter
Sleep 2s

# Show SPECS_ARCHIVE in both gh release create and gh release upload --clobber arms
Type "grep -n 'SPECS_ARCHIVE' .github/workflows/release.yml"
Enter
Sleep 3s
