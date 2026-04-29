Output docs/demo-evidence/S-3.1.01/AC-001-all-11-tests-green.gif
Output docs/demo-evidence/S-3.1.01/AC-001-all-11-tests-green.webm

Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 14
Set Width 1000
Set Height 600
Set Theme "Dracula"
Set Padding 20
Set Shell "bash"

Require vhs
Require cargo

Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-3.1.01 && clear"
Enter
Sleep 2s
Show

Type "# S-3.1.01 BC-3.1.001: OrgId newtype — all 11 tests GREEN"
Enter
Sleep 500ms
Type "cargo test --test bc_3_1_001_org_id 2>&1"
Enter
Sleep 15s
