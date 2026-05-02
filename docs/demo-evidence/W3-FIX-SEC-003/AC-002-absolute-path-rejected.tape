# VHS tape: AC-002 — absolute path rejected with E-CFG-018
# Story: W3-FIX-SEC-003 (CWE-22 path traversal fix)
# Acceptance Criterion: spec paths starting with "/" (absolute) must be rejected.

Require bash

Output docs/demo-evidence/W3-FIX-SEC-003/AC-002-absolute-path-rejected.gif
Output docs/demo-evidence/W3-FIX-SEC-003/AC-002-absolute-path-rejected.webm

Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 14
Set Width 1200
Set Height 600
Set Theme "Catppuccin Mocha"
Set Padding 20
Set Shell "bash"

Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/W3-FIX-SEC-003 && clear"
Enter
Sleep 500ms
Show

Type "bash docs/demo-evidence/W3-FIX-SEC-003/demo-helper.sh AC-002"
Enter
Sleep 8s

Sleep 2s
