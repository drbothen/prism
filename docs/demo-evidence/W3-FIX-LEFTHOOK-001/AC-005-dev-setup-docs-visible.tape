# AC-005: docs/dev-setup.md is visible and contains new sections
# Demonstrates: new developer documentation for pre-push gate, standalone targets, pre-tag hook
# Expected: file shows Pre-push gate section, Standalone targets, Pre-tag hook content

Output AC-005-dev-setup-docs-visible.gif
Output AC-005-dev-setup-docs-visible.webm

Require cat

Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 14
Set Width 1280
Set Height 720
Set Padding 20
Set Theme "Dracula"

Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/W3-FIX-LEFTHOOK-001"
Enter
Sleep 1s
Show

Type "grep -n -- '##' docs/dev-setup.md"
Enter
Sleep 2s

Type "head -30 docs/dev-setup.md"
Enter
Sleep 2s
