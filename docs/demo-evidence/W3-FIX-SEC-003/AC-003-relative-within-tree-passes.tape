# VHS tape: AC-003 — relative within-tree path accepted (canonicalized)
# Story: W3-FIX-SEC-003 (CWE-22 path traversal fix)
# Acceptance Criterion: spec = "sensors/claroty.toml" passes and returns canonical path.

Require bash

Output docs/demo-evidence/W3-FIX-SEC-003/AC-003-relative-within-tree-passes.gif
Output docs/demo-evidence/W3-FIX-SEC-003/AC-003-relative-within-tree-passes.webm

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

Type "bash docs/demo-evidence/W3-FIX-SEC-003/demo-helper.sh AC-003"
Enter
Sleep 8s

Sleep 2s
