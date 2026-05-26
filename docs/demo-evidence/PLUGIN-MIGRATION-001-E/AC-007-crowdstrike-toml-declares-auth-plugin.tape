# PLUGIN-MIGRATION-001-E / AC-007
# crowdstrike.sensor.toml Amended with auth_plugin Field
# Traces to: BC-2.16.013 spec file amended, ADR-028 §D2 auth declaration via plugin path
# Both success path (auth_plugin Some) and error path (unknown plugin → E-SPEC-012)
# Run from: docs/demo-evidence/PLUGIN-MIGRATION-001-E/

Output AC-007-crowdstrike-toml-declares-auth-plugin.gif
Output AC-007-crowdstrike-toml-declares-auth-plugin.webm

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

Type "# AC-007: TOML declares auth_plugin; unknown plugin → E-SPEC-012 (4 test variants)"
Enter
Sleep 0.5s
Type "cargo nextest run -p prism-spec-engine -E 'test(PLUGIN_MIGRATION_001_E_007)' --no-fail-fast 2>&1 | grep -E 'PASS|FAIL|Summary'"
Enter
Sleep 20s
