# AC-001: just check completes under 10 min (fast local gate — target 3-8 min)
# Demonstrates: W3-FIX-LEFTHOOK-001 pre-push gate tuning removes slow steps
# Expected: recipe shows 4 steps (no audit/deny/semver-checks), exit 0

Output AC-001-just-check-fast.gif
Output AC-001-just-check-fast.webm

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

Type "# AC-001: just check — fast pre-push gate (target: 3-8 min, not 35 min)"
Enter
Sleep 1s

Type "just --show check"
Enter
Sleep 3s

Type "echo Recipe has 4 steps: fmt, clippy, test PROPTEST_CASES=100, check-layout"
Enter
Sleep 1s

Type "echo No audit / deny / semver-checks -- those moved to just check-ci"
Enter
Sleep 2s
