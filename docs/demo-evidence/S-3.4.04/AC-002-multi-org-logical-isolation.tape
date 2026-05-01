# AC-002 — Multi-org logical isolation
# S-3.4.04: Two-org harness returns pairwise-disjoint alert/indicator sets
# Traces to BC-3.5.001 postcondition 2 (TV-2)

Output docs/demo-evidence/S-3.4.04/AC-002-multi-org-logical-isolation.gif
Output docs/demo-evidence/S-3.4.04/AC-002-multi-org-logical-isolation.webm

Require cargo

Set Shell "zsh"
Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 20
Set Width 1200
Set Height 600
Set Theme "Catppuccin Mocha"
Set WindowBar Colorful
Set BorderRadius 8
Set Padding 20
Set Framerate 30
Set TypingSpeed 50ms
Set WaitTimeout 120s

Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-3.4.04"
Enter
Sleep 2s
Type "clear"
Enter
Sleep 1s
Show

# Success path — ac_multi_org_logical_isolation verifies pairwise-disjoint sets
Type "cargo test -p prism-dtu-cyberint --features dtu --test harness_tests harness_tests::ac_multi_org_logical_isolation"
Enter
Sleep 10s
Sleep 2s
