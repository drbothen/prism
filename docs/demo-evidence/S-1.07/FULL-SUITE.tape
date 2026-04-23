Output docs/demo-evidence/S-1.07/FULL-SUITE.gif
Output docs/demo-evidence/S-1.07/FULL-SUITE.webm

Require cargo

Set FontFamily "FiraCode Nerd Font Mono"
Set Theme "Catppuccin Mocha"
Set Width 1200
Set Height 700
Set FontSize 13
Set Padding 24

# Full suite: all 78 prism-credentials tests
# BC-2.03.001 through BC-2.03.012 (S-1.06) + BC-2.03.005/006/007/009/010 (S-1.07)

Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-1.07-credential-crud"
Enter
Sleep 1s
Type "clear"
Enter
Sleep 500ms
Show

Type "# S-1.07 full suite: 78/78 prism-credentials tests"
Enter
Sleep 800ms
Type "cargo test -p prism-credentials 2>&1 | tail -20"
Enter
Sleep 60s
