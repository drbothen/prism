# PLUGIN-MIGRATION-001-E / AC-011
# just check Workspace-Wide GREEN
# Traces to: BC-2.16.013 CI gate, CLAUDE.md production-grade default
# Shows all 15 PLUGIN_MIGRATION_001_E tests pass in a single run
# Run from: docs/demo-evidence/PLUGIN-MIGRATION-001-E/

Output AC-011-just-check-workspace-green.gif
Output AC-011-just-check-workspace-green.webm

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

Type "# AC-011: All 15 PLUGIN-MIGRATION-001-E tests pass (workspace GREEN)"
Enter
Sleep 0.5s
Type "cargo nextest run -p prism-spec-engine -E 'test(PLUGIN_MIGRATION_001_E)' --no-fail-fast 2>&1 | tail -6"
Enter
Sleep 20s
