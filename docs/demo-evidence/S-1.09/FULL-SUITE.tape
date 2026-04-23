Output docs/demo-evidence/S-1.09/FULL-SUITE.gif
Output docs/demo-evidence/S-1.09/FULL-SUITE.webm

Require cargo

Set FontFamily "FiraCode Nerd Font Mono"
Set Theme "Catppuccin Mocha"
Set Width 1200
Set Height 600
Set FontSize 14
Set Padding 24

# Full test suite — all S-1.09 tests (200/200 green)

Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-1.09-confirmation-tokens"
Enter
Sleep 1s
Type "clear"
Enter
Sleep 500ms
Show

Type "# S-1.09 full test suite — 200/200 green"
Enter
Sleep 800ms
Type "cargo test -p prism-security 2>&1 | grep 'test result'"
Enter
Sleep 30s
