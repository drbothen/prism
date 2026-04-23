Output docs/demo-evidence/S-1.08/AC-005-audit-event-emission.gif
Output docs/demo-evidence/S-1.08/AC-005-audit-event-emission.webm

Require cargo

Set FontFamily "FiraCode Nerd Font Mono"
Set Theme "Catppuccin Mocha"
Set Width 1200
Set Height 600
Set FontSize 14
Set Padding 24

# AC-5 (BC-2.04.013): Every write check emits audit event — both Allow and Deny paths

Hide
Type "cd /Users/jmagady/dev/prism/.worktrees/S-1.08-feature-flags"
Enter
Sleep 1s
Type "clear"
Enter
Sleep 500ms
Show

Type "# AC-5: Audit event emission for write checks (both Allow + Deny paths)"
Enter
Sleep 800ms
Type "cargo test --no-default-features -p prism-security test_BC_2_04_013 2>&1 | grep -E 'passed|failed'"
Enter
Sleep 8s
