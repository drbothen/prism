# AC-001 — All 5 HS-007 anchor tests GREEN
# Story: S-3.6.02
# Traces to: BC-3.5.001, BC-3.5.002, BC-3.6.001, BC-3.6.002

Output AC-001-hs-007-anchor-tests-green.gif
Output AC-001-hs-007-anchor-tests-green.webm

Require cargo

Set FontFamily "FiraCode Nerd Font Mono"
Set Theme "Dracula"
Set Width 1200
Set Height 600
Set FontSize 14
Set Padding 20
Set Shell "bash"

Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-3.6.02"
Enter
Sleep 2s
Type "clear"
Enter
Sleep 1s
Show

Type "cargo test --test hs_007_anchor_test"
Enter
Sleep 10s
