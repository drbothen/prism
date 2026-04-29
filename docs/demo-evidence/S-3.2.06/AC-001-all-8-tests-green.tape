Output docs/demo-evidence/S-3.2.06/AC-001-all-8-tests-green.gif
Output docs/demo-evidence/S-3.2.06/AC-001-all-8-tests-green.webm

Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 14
Set Width 1200
Set Height 700
Set Theme "Dracula"
Set Padding 20
Set Shell "bash"

Require vhs
Require cargo

Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-3.2.06 && clear"
Enter
Sleep 2s
Show

Type "# S-3.2.06: BC-3.2.004/005 — PagerDuty DTU shared-mode OrgId tagging — all 8 tests"
Enter
Sleep 500ms
Type "cargo test -p prism-dtu-pagerduty --features dtu --test org_tagging 2>&1"
Enter
Sleep 15s
