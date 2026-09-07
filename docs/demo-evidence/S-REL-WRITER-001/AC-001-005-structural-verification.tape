Output docs/demo-evidence/S-REL-WRITER-001/AC-001-005-structural-verification.gif
Output docs/demo-evidence/S-REL-WRITER-001/AC-001-005-structural-verification.webm

Require git

Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 13
Set Width 1200
Set Height 900
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

Type "# AC-001: technical-writer/Layer-1 step exists AFTER git-cliff in release-prep.yml"
Enter
Sleep 800ms
Type "grep -n 'git cliff\|Layer-1\|technical-writer' .github/workflows/release-prep.yml"
Enter
Sleep 3s
Type "# AC-002/AC-003: RELEASING.md §5 documents two-layer model + INSIDE placement"
Enter
Sleep 800ms
Type "grep -n 'Layer 1\|Layer-1\|Highlights\|INSIDE' RELEASING.md | head -10"
Enter
Sleep 3s
Type "# AC-004: PR checklist includes Layer-1 review items"
Enter
Sleep 800ms
Type "grep -n 'Layer-1\|Highlights.*reviewed' .github/workflows/release-prep.yml"
Enter
Sleep 3s
Type "# AC-005: release.yml NOT modified (empty diff)"
Enter
Sleep 800ms
Type "git diff develop -- .github/workflows/release.yml | wc -l"
Enter
Sleep 3s
