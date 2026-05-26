# PLUGIN-MIGRATION-001-E / AC-002
# Plugin auth_type_name() Returns Canonical Value
# Traces to: BC-2.01.016 postcondition, INV-AUTH-OPEN-003 Rule A
# Run from: docs/demo-evidence/PLUGIN-MIGRATION-001-E/

Output AC-002-auth-type-name-returns-canonical-value.gif
Output AC-002-auth-type-name-returns-canonical-value.webm

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

Type "# AC-002: auth-type-name() returns oauth2_client_credentials (INV-AUTH-OPEN-003 Rule A)"
Enter
Sleep 0.5s
Type "cargo nextest run -p prism-spec-engine -E 'test(PLUGIN_MIGRATION_001_E_002)' 2>&1 | grep -E 'PASS|FAIL|Summary'"
Enter
Sleep 15s
