Output docs/demo-evidence/S-1.09/AC-003-token-consumption.gif
Output docs/demo-evidence/S-1.09/AC-003-token-consumption.webm

Require cargo

Set FontFamily "FiraCode Nerd Font Mono"
Set Theme "Catppuccin Mocha"
Set Width 1200
Set Height 600
Set FontSize 14
Set Padding 24

# AC-2: Token consumption — single-use, valid at 299s executes (BC-2.04.010)
# Error path: double-consume returns E-FLAG-004

Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-1.09-confirmation-tokens"
Enter
Sleep 1s
Type "clear"
Enter
Sleep 500ms
Show

Type "# AC-2: Token consumption single-use + atomic consumed-before-dispatch (BC-2.04.010)"
Enter
Sleep 800ms
Type "cargo test -p prism-security test_BC_2_04_010 2>&1 | grep -E 'passed|failed|FAILED'"
Enter
Sleep 15s
