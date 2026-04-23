Output docs/demo-evidence/S-1.09/AC-008-overflow-eflag007.gif
Output docs/demo-evidence/S-1.09/AC-008-overflow-eflag007.webm

Require cargo

Set FontFamily "FiraCode Nerd Font Mono"
Set Theme "Catppuccin Mocha"
Set Width 1200
Set Height 600
Set FontSize 14
Set Padding 24

# EC-004 / VP-010: 100-token cap overflow → E-FLAG-007 (no eviction)
# Success: generating up to cap succeeds
# Error path: 101st token returns E-FLAG-007

Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-1.09-confirmation-tokens"
Enter
Sleep 1s
Type "clear"
Enter
Sleep 500ms
Show

Type "# Overflow / E-FLAG-007: 101st token generation rejected, no eviction (VP-010)"
Enter
Sleep 800ms
Type "cargo test -p prism-security test_VP_010 2>&1 | grep -E 'passed|failed|FAILED'"
Enter
Sleep 15s
