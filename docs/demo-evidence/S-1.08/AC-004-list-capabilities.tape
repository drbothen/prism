Output docs/demo-evidence/S-1.08/AC-004-list-capabilities.gif
Output docs/demo-evidence/S-1.08/AC-004-list-capabilities.webm

Require cargo

Set FontFamily "FiraCode Nerd Font Mono"
Set Theme "Catppuccin Mocha"
Set Width 1200
Set Height 600
Set FontSize 14
Set Padding 24

# AC-4 (BC-2.04.006): list_capabilities returns full effective capability set for client

Hide
Type "cd /Users/jmagady/dev/prism/.worktrees/S-1.08-feature-flags"
Enter
Sleep 1s
Type "clear"
Enter
Sleep 500ms
Show

Type "# AC-4: list_capabilities meta-tool returns effective capability matrix per client"
Enter
Sleep 800ms
Type "cargo test --no-default-features -p prism-security test_BC_2_04_006 2>&1 | grep -E 'passed|failed'"
Enter
Sleep 8s
