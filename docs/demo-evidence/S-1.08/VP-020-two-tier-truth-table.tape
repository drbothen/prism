Output docs/demo-evidence/S-1.08/VP-020-two-tier-truth-table.gif
Output docs/demo-evidence/S-1.08/VP-020-two-tier-truth-table.webm

Require cargo

Set FontFamily "FiraCode Nerd Font Mono"
Set Theme "Catppuccin Mocha"
Set Width 1200
Set Height 650
Set FontSize 14
Set Padding 24

# VP-020: Two-tier truth table — unit test counterpart to Kani harness
# Covers (compile=Absent|Present) x (runtime=Deny|Allow) — all 4 combinations

Hide
Type "cd /Users/jmagady/dev/prism/.worktrees/S-1.08-feature-flags"
Enter
Sleep 1s
Type "clear"
Enter
Sleep 500ms
Show

Type "# VP-020: Two-tier gate truth table (4 combinations, unit test harness)"
Enter
Sleep 800ms
Type "cargo test --no-default-features -p prism-security test_VP_020 2>&1 | grep -E 'passed|failed'"
Enter
Sleep 8s
