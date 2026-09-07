Output docs/demo-evidence/S-REL-CLIFF-001/AC-006-breaking-changes-first.gif
Output docs/demo-evidence/S-REL-CLIFF-001/AC-006-breaking-changes-first.webm

Require git

Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 14
Set Width 1200
Set Height 800
Set Theme "Catppuccin Mocha"
Set Padding 20

Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/E-REL-NOTES"
Enter
Sleep 1s
Type "git commit --allow-empty -m 'feat(api)!: THROWAWAY remove legacy sensor API' -m 'BREAKING CHANGE: /v0/sensors removed, migrate to /v1/sensors' 2>/dev/null"
Enter
Sleep 8s
Type "clear"
Enter
Sleep 500ms
Show

Type "# AC-006: Breaking Changes renders BEFORE Added (correct multi-line commit with footer)"
Enter
Sleep 1s
Type "git cliff --unreleased --tag v1.0.0-beta.1 --output /dev/stdout 2>/dev/null"
Enter
Sleep 8s

Hide
Type "git reset --soft HEAD~1 && git reset HEAD 2>/dev/null"
Enter
Sleep 2s
Type "git log --oneline -3"
Enter
Sleep 3s
