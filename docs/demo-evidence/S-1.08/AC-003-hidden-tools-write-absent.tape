Output docs/demo-evidence/S-1.08/AC-003-hidden-tools-write-absent.gif
Output docs/demo-evidence/S-1.08/AC-003-hidden-tools-write-absent.webm

Require cargo

Set FontFamily "FiraCode Nerd Font Mono"
Set Theme "Catppuccin Mocha"
Set Width 1200
Set Height 600
Set FontSize 14
Set Padding 24

# AC-3 (BC-2.04.005): Denied write capability → write tool absent from tools/list

Hide
Type "cd /Users/jmagady/dev/prism/.worktrees/S-1.08-feature-flags"
Enter
Sleep 1s
Type "clear"
Enter
Sleep 500ms
Show

Type "# AC-3: Hidden tools pattern — denied write tool absent from tools/list"
Enter
Sleep 800ms
Type "cargo test --no-default-features -p prism-security test_BC_2_04_005 2>&1 | grep -E 'passed|failed'"
Enter
Sleep 8s
