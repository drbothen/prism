Output docs/demo-evidence/S-1.09/AC-001-risktier-routing.gif
Output docs/demo-evidence/S-1.09/AC-001-risktier-routing.webm

Require cargo

Set FontFamily "FiraCode Nerd Font Mono"
Set Theme "Catppuccin Mocha"
Set Width 1200
Set Height 600
Set FontSize 14
Set Padding 24

# AC-1 / AC-7: RiskTier routing
# Read → Allow immediately; Reversible → DryRunPreview; Irreversible → RequiresConfirmationToken

Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-1.09-confirmation-tokens"
Enter
Sleep 1s
Type "clear"
Enter
Sleep 500ms
Show

Type "# AC-1/AC-7: RiskTier gate routing (BC-2.04.007)"
Enter
Sleep 800ms
Type "cargo test -p prism-security test_BC_2_04_007 2>&1 | grep -E 'passed|failed|FAILED'"
Enter
Sleep 15s
