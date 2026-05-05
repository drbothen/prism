Output AC-010-full-suite-all-green.gif
Output AC-010-full-suite-all-green.webm

Require cargo

Set Shell "bash"
Set FontSize 14
Set Width 1200
Set Height 600
Set Theme "Catppuccin Mocha"
Set Padding 20
Set FontFamily "FiraCode Nerd Font Mono"

Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-3.01"
Enter
Sleep 1s
Type "clear"
Enter
Sleep 500ms
Show

Type "# AC-10: VP-014/VP-015 Kani proof stubs compile; VP-021 fuzz target registered"
Enter
Sleep 500ms
Type "# Demonstrates: all 177 tests green (12 unit + 15 unit + 150 integration)"
Enter
Sleep 500ms

Type "cargo test -p prism-query 2>&1 | tail -20"
Enter
Sleep 60s
