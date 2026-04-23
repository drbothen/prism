Output docs/demo-evidence/S-1.09/AC-006-dry-run-reversible.gif
Output docs/demo-evidence/S-1.09/AC-006-dry-run-reversible.webm

Require cargo

Set FontFamily "FiraCode Nerd Font Mono"
Set Theme "Catppuccin Mocha"
Set Width 1200
Set Height 600
Set FontSize 14
Set Padding 24

# AC-5: Reversible operation dry-run default (BC-2.04.008)
# Success: acknowledge_alert without dry_run:false → DryRunPreview
# Error path: explicit dry_run:false → Allow (no preview, operation executes)

Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-1.09-confirmation-tokens"
Enter
Sleep 1s
Type "clear"
Enter
Sleep 500ms
Show

Type "# AC-5: Reversible op dry-run default — no mutation without explicit dry_run:false (BC-2.04.008)"
Enter
Sleep 800ms
Type "cargo test -p prism-security test_BC_2_04_008 2>&1 | grep -E 'passed|failed|FAILED'"
Enter
Sleep 15s
