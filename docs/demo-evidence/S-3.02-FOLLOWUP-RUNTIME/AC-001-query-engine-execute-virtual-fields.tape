# AC-1 — QueryEngine::execute returns non-empty result with virtual fields
# Traces to BC-2.11.001 + BC-2.11.012
# Story: S-3.02-FOLLOWUP-RUNTIME

Output docs/demo-evidence/S-3.02-FOLLOWUP-RUNTIME/AC-001-query-engine-execute-virtual-fields.gif
Output docs/demo-evidence/S-3.02-FOLLOWUP-RUNTIME/AC-001-query-engine-execute-virtual-fields.webm

Require cargo

Set Shell "zsh"
Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 18
Set Width 1200
Set Height 600
Set Theme "Catppuccin Mocha"
Set WindowBar Colorful
Set BorderRadius 8
Set Padding 20
Set Framerate 30
Set TypingSpeed 50ms

Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-3.02-FOLLOWUP-RUNTIME && clear"
Enter
Sleep 2s
Show

# AC-1: QueryEngine::execute returns non-empty QueryResult with _sensor virtual field
Type "cargo nextest run -p prism-bin -E 'test(test_BC_2_11_001_query_engine_execute_after_boot)'"
Enter
Sleep 10s
Sleep 2s
