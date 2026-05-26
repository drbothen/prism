# PLUGIN-MIGRATION-001-E / AC-009
# Plugin Loaded at Boot Step 7.5 (BC-2.22.001 Sequencing Invariant)
# Traces to: BC-2.22.001 boot orchestration, BC-2.17.001 unsigned WARN behavior
# Run from: docs/demo-evidence/PLUGIN-MIGRATION-001-E/

Output AC-009-plugin-loaded-at-boot-step-7-5-emits-warn.gif
Output AC-009-plugin-loaded-at-boot-step-7-5-emits-warn.webm

Require cargo

Set Shell "zsh"
Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 16
Set Width 1200
Set Height 600
Set Theme "Catppuccin Mocha"
Set WindowBar Colorful
Set BorderRadius 8
Set Padding 20
Set Framerate 30
Set TypingSpeed 30ms

Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/PLUGIN-MIGRATION-001-E"
Enter
Sleep 1s
Type "clear"
Enter
Sleep 1s
Show

Type "# AC-009: Boot step 7.5 — plugin loads with plugin_load_unsigned WARN (BC-2.22.001)"
Enter
Sleep 0.5s
Type "cargo nextest run -p prism-spec-engine -E 'test(PLUGIN_MIGRATION_001_E_009)' 2>&1 | grep -E 'PASS|FAIL|Summary'"
Enter
Sleep 15s
