Output docs/demo-evidence/S-1.09/AC-005-content-hash.gif
Output docs/demo-evidence/S-1.09/AC-005-content-hash.webm

Require cargo

Set FontFamily "FiraCode Nerd Font Mono"
Set Theme "Catppuccin Mocha"
Set Width 1200
Set Height 600
Set FontSize 14
Set Padding 24

# AC-4: Content hash invariance — key reordering same hash (EC-04-025 / VP-009)
# Error path: tampered params → different hash → rejection (BC-2.04.012)

Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-1.09-confirmation-tokens"
Enter
Sleep 1s
Type "clear"
Enter
Sleep 500ms
Show

Type "# AC-4: Content hash mismatch rejection + key-order invariance (BC-2.04.012)"
Enter
Sleep 800ms
Type "cargo test -p prism-security test_BC_2_04_012 2>&1 | grep -E 'passed|failed|FAILED'"
Enter
Sleep 15s
