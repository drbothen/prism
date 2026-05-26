# PLUGIN-MIGRATION-001-E / AC-010
# Credential Opaqueness — Token Value Not Logged (Security AC)
# Traces to: BC-2.01.016 Debug safety, AD-017 AI-opaque credential model
# Run from: docs/demo-evidence/PLUGIN-MIGRATION-001-E/

Output AC-010-credential-opaqueness-token-not-logged.gif
Output AC-010-credential-opaqueness-token-not-logged.webm

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

Type "# AC-010: Security — access_token NOT in tracing output (AD-017, BC-2.01.016)"
Enter
Sleep 0.5s
Type "cargo nextest run -p prism-spec-engine -E 'test(PLUGIN_MIGRATION_001_E_010)' 2>&1 | grep -E 'PASS|FAIL|Summary'"
Enter
Sleep 15s
