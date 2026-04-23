Output docs/demo-evidence/S-1.09/AC-009-expiry-sweep.gif
Output docs/demo-evidence/S-1.09/AC-009-expiry-sweep.webm

Require cargo

Set FontFamily "FiraCode Nerd Font Mono"
Set Theme "Catppuccin Mocha"
Set Width 1200
Set Height 600
Set FontSize 14
Set Padding 24

# Token expiration sweep — sweep_expired removes expired tokens, freeing cap slots
# Success: sweep returns count of swept tokens
# Error path: consumed tokens excluded from active count

Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-1.09-confirmation-tokens"
Enter
Sleep 1s
Type "clear"
Enter
Sleep 500ms
Show

Type "# Token expiry sweep (BC-2.04.011 sweep_expired)"
Enter
Sleep 800ms
Type "cargo test -p prism-security test_BC_2_04_011_sweep 2>&1 | grep -E 'passed|failed|FAILED'"
Enter
Sleep 15s
