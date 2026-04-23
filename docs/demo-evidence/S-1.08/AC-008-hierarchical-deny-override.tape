Output docs/demo-evidence/S-1.08/AC-008-hierarchical-deny-override.gif
Output docs/demo-evidence/S-1.08/AC-008-hierarchical-deny-override.webm

Require cargo

Set FontFamily "FiraCode Nerd Font Mono"
Set Theme "Catppuccin Mocha"
Set Width 1200
Set Height 600
Set FontSize 14
Set Padding 24

# AC-8 (BC-2.04.003): Most-specific-path wins — explicit Deny at sub-path overrides parent Allow

Hide
Type "cd /Users/jmagady/dev/prism/.worktrees/S-1.08-feature-flags"
Enter
Sleep 1s
Type "clear"
Enter
Sleep 500ms
Show

Type "# AC-8: Hierarchical resolution — specific Deny overrides parent Allow (BTreeMap)"
Enter
Sleep 800ms
Type "cargo test --no-default-features -p prism-security test_BC_2_04_003 2>&1 | grep -E 'passed|failed'"
Enter
Sleep 8s
