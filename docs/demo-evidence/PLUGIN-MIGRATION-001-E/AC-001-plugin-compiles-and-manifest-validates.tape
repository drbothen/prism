# PLUGIN-MIGRATION-001-E / AC-001
# Plugin Source Exists at Canonical Location and Compiles
# Traces to: BC-2.17.007 (manifest schema), BC-2.17.006 (WIT export)
# Run from: docs/demo-evidence/PLUGIN-MIGRATION-001-E/

Output AC-001-plugin-compiles-and-manifest-validates.gif
Output AC-001-plugin-compiles-and-manifest-validates.webm

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

Type "# AC-001: PluginRuntime::load_plugin returns Ok — WIT + manifest validated"
Enter
Sleep 0.5s
Type "cargo nextest run -p prism-spec-engine -E 'test(PLUGIN_MIGRATION_001_E_001)' 2>&1 | grep -E 'PASS|FAIL|Summary'"
Enter
Sleep 15s
