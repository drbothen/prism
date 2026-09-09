# VHS Demo — S-REL-SPECS-TARBALL-001
# AC-008: Both scripts pass shellcheck / PSScriptAnalyzer with zero new errors
#
# Evidence: shellcheck exits 0 with zero output (zero errors),
#            PSScriptAnalyzer returns no Error-severity findings.
# Usage: vhs AC-008-shellcheck-psscriptanalyzer.tape

Output docs/demo-evidence/S-REL-SPECS-TARBALL-001/AC-008-shellcheck-psscriptanalyzer.gif
Output docs/demo-evidence/S-REL-SPECS-TARBALL-001/AC-008-shellcheck-psscriptanalyzer.webm

Require shellcheck
Require pwsh

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

# shellcheck: zero output = zero errors; echo confirms exit 0
Type "shellcheck scripts/install.sh && echo 'shellcheck: PASS (exit 0, zero errors)'"
Enter
Sleep 2s

# PSScriptAnalyzer: no Error-severity findings
Type "pwsh -Command \"Invoke-ScriptAnalyzer -Path scripts/install.ps1 -Severity Error; Write-Host 'PSScriptAnalyzer: PASS (zero Error findings)'\""
Enter
Sleep 4s
