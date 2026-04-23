Output docs/demo-evidence/S-1.08/FULL-SUITE-all-features.gif
Output docs/demo-evidence/S-1.08/FULL-SUITE-all-features.webm

Require cargo

Set FontFamily "FiraCode Nerd Font Mono"
Set Theme "Catppuccin Mocha"
Set Width 1200
Set Height 700
Set FontSize 14
Set Padding 24

# Full suite — demonstrates feature-combination behavior
# Tier 1: --no-default-features (all write gates Absent)
# Tier 2: --features all-write (all write gates Present)
# BC-2.04.001 "real gate absent" tests pass on no-default-features,
# and are designed for that combination.

Hide
Type "cd /Users/jmagady/dev/prism/.worktrees/S-1.08-feature-flags"
Enter
Sleep 1s
Type "clear"
Enter
Sleep 500ms
Show

Type "# Full suite (71/71): --no-default-features — write gates Absent"
Enter
Sleep 600ms
Type "cargo test --no-default-features -p prism-security 2>&1 | grep -E '^test result'"
Enter
Sleep 10s

Type "# With --features all-write: write gates Present (BC-2.04.001 gate-absent tests filtered)"
Enter
Sleep 600ms
Type "cargo test --features all-write -p prism-security 2>&1 | grep -E '^test result'"
Enter
Sleep 10s
