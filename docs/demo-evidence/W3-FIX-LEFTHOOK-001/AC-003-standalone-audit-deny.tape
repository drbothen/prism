# AC-003: Standalone targets just audit and just deny work independently
# Demonstrates: audit + deny dispatch correctly to installed system tools
# Expected: each command resolves and exits 0

Output AC-003-standalone-audit-deny.gif
Output AC-003-standalone-audit-deny.webm

Require just

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

Type "just --show audit"
Enter
Sleep 2s

Type "which cargo-audit && cargo-audit --version"
Enter
Sleep 2s

Type "just --show deny"
Enter
Sleep 2s

Type "which cargo-deny && cargo-deny --version"
Enter
Sleep 2s
