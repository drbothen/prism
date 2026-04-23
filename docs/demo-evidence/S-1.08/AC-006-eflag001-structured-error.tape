Output docs/demo-evidence/S-1.08/AC-006-eflag001-structured-error.gif
Output docs/demo-evidence/S-1.08/AC-006-eflag001-structured-error.webm

Require cargo

Set FontFamily "FiraCode Nerd Font Mono"
Set Theme "Catppuccin Mocha"
Set Width 1200
Set Height 600
Set FontSize 14
Set Padding 24

# AC-6 (BC-2.04.015): Denied write → E-FLAG-001 with resolution_trace

Hide
Type "cd /Users/jmagady/dev/prism/.worktrees/S-1.08-feature-flags"
Enter
Sleep 1s
Type "clear"
Enter
Sleep 500ms
Show

Type "# AC-6: E-FLAG-001 structured error includes resolution_trace on denial"
Enter
Sleep 800ms
Type "cargo test --no-default-features -p prism-security test_BC_2_04_015 2>&1 | grep -E 'passed|failed'"
Enter
Sleep 8s
