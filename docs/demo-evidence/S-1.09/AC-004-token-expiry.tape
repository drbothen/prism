Output docs/demo-evidence/S-1.09/AC-004-token-expiry.gif
Output docs/demo-evidence/S-1.09/AC-004-token-expiry.webm

Require cargo

Set FontFamily "FiraCode Nerd Font Mono"
Set Theme "Catppuccin Mocha"
Set Width 1200
Set Height 600
Set FontSize 14
Set Padding 24

# AC-3: Token expiry — 301s returns E-FLAG-003; boundary at exactly 300s is expired (BC-2.04.011)
# Error path: expired token (EC-001)

Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-1.09-confirmation-tokens"
Enter
Sleep 1s
Type "clear"
Enter
Sleep 500ms
Show

Type "# AC-3: Token expiry E-FLAG-003; boundary at 300s expired (BC-2.04.011)"
Enter
Sleep 800ms
Type "cargo test -p prism-security test_BC_2_04_011 2>&1 | grep -E 'passed|failed|FAILED'"
Enter
Sleep 15s
