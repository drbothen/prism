# AC-002: just check-ci is full-strength (all 7 steps present)
# Demonstrates: check-ci includes audit, deny, semver-checks
# Expected: recipe printout shows all 7 steps, exit 0

Output AC-002-just-check-ci-full-strength.gif
Output AC-002-just-check-ci-full-strength.webm

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

Type "just --show check-ci"
Enter
Sleep 3s

Type "echo check-ci has 7 steps: fmt clippy test deny audit semver-checks check-layout"
Enter
Sleep 2s
