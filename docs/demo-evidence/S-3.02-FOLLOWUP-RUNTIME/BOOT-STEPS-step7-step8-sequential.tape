# Boot Steps 7 + 8 — sequential integration
# Traces to BC-2.22.001
# Story: S-3.02-FOLLOWUP-RUNTIME
#
# Demonstrates: step7 (health_check) and step8 (mark_query_phase_started)
# both pass individually and in sequence.

Output docs/demo-evidence/S-3.02-FOLLOWUP-RUNTIME/BOOT-STEPS-step7-step8-sequential.gif
Output docs/demo-evidence/S-3.02-FOLLOWUP-RUNTIME/BOOT-STEPS-step7-step8-sequential.webm

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

# Boot step 7: RocksDB health_check validation
Type "cargo nextest run -p prism-bin -E 'test(test_BC_2_22_001_step7_validates_storage_backend)'"
Enter
Sleep 8s

# Boot step 8: mark_query_phase_started closes write-tool registration window
Type "cargo nextest run -p prism-bin -E 'test(test_BC_2_22_001_step8_constructs_query_engine)'"
Enter
Sleep 8s

# Sequential integration: step7 -> step8 in order
Type "cargo nextest run -p prism-bin -E 'test(test_BC_2_22_001_step7_step8_sequential_integration)'"
Enter
Sleep 8s
Sleep 2s
