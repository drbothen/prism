Output docs/demo-evidence/S-6.08/FIDELITY-full-suite.gif
Output docs/demo-evidence/S-6.08/FIDELITY-full-suite.webm

Require cargo

Set FontFamily "FiraCode Nerd Font Mono"
Set Theme "Dracula"
Set Width 1200
Set Height 700
Set FontSize 14
Set Padding 20
Set Shell "bash"
Set PlaybackSpeed 1.0

# Full test suite: 53 tests across all ACs + edge cases + fidelity validator
# FidelityValidator runs against all 9 routes (7 API + 2 DTU control)

Hide
Type "cd /Users/jmagady/dev/prism/.worktrees/S-6.08-claroty && clear"
Enter
Sleep 1s
Show

Type "cargo test -p prism-dtu-claroty --features prism-dtu-claroty/dtu"
Enter
Sleep 30s
