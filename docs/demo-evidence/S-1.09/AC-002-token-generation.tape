Output docs/demo-evidence/S-1.09/AC-002-token-generation.gif
Output docs/demo-evidence/S-1.09/AC-002-token-generation.webm

Require cargo

Set FontFamily "FiraCode Nerd Font Mono"
Set Theme "Catppuccin Mocha"
Set Width 1200
Set Height 600
Set FontSize 14
Set Padding 24

# AC-1 (BC-2.04.009): Token generation — CSPRNG 256-bit ID, cap check, sweep-before-issue
# Also covers EC-004: 101st token returns E-FLAG-007 (VP-010)

Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-1.09-confirmation-tokens"
Enter
Sleep 1s
Type "clear"
Enter
Sleep 500ms
Show

Type "# AC-1 / AC-2: Token generation + 100-token cap (BC-2.04.009)"
Enter
Sleep 800ms
Type "cargo test -p prism-security test_BC_2_04_009 2>&1 | grep -E 'passed|failed|FAILED'"
Enter
Sleep 15s
