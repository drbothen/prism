# AC-5 — register_internal_tables makes prism_audit queryable
# Traces to BC-2.15.011
# Story: S-3.02-FOLLOWUP-RUNTIME

Output docs/demo-evidence/S-3.02-FOLLOWUP-RUNTIME/AC-005-internal-tables-accessible-after-step7.gif
Output docs/demo-evidence/S-3.02-FOLLOWUP-RUNTIME/AC-005-internal-tables-accessible-after-step7.webm

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

# AC-5: register_internal_tables registers all 7 BC-2.15.011 internal tables
Type "cargo nextest run -p prism-bin -E 'test(test_BC_2_15_011_internal_tables_accessible_after_step7)'"
Enter
Sleep 10s
Sleep 2s
