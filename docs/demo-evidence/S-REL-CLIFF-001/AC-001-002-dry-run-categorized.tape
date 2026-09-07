Output docs/demo-evidence/S-REL-CLIFF-001/AC-001-002-dry-run-categorized.gif
Output docs/demo-evidence/S-REL-CLIFF-001/AC-001-002-dry-run-categorized.webm

Require git

Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 14
Set Width 1200
Set Height 750
Set Theme "Catppuccin Mocha"
Set Padding 20

Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/E-REL-NOTES"
Enter
Sleep 1s
Type "clear"
Enter
Sleep 500ms
Show

Type "# AC-001/AC-002: dry-run — categorized sections, PR links (no token), skip types absent"
Enter
Sleep 1s
Type "git cliff --unreleased --tag v1.0.0-beta.1 --output /dev/stdout 2>/dev/null"
Enter
Sleep 8s
