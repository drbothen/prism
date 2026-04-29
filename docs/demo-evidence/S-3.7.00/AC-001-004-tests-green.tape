Output docs/demo-evidence/S-3.7.00/AC-001-004-tests-green.gif
Output docs/demo-evidence/S-3.7.00/AC-001-004-tests-green.webm

Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 14
Set Width 1200
Set Height 700
Set Theme "Dracula"
Set Padding 20
Set Shell "bash"

Require vhs

Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-3.7.00 && clear"
Enter
Sleep 2s
Show

Type "# AC-001..004: schema derivation -- all 25 assertions GREEN"
Enter
Sleep 500ms
Type "bash tests/toolchain-gate/test_S-3.7.00_schema-derivation-artifacts.sh"
Enter
Sleep 3s
Sleep 2s
