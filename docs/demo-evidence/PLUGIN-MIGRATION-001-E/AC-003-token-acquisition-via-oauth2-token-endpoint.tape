# PLUGIN-MIGRATION-001-E / AC-003
# Token Acquisition via POST /oauth2/token Against DTU Clone
# Traces to: BC-2.01.016 preconditions, BC-2.16.013 plugin path OCSF output
# Run from: docs/demo-evidence/PLUGIN-MIGRATION-001-E/

Output AC-003-token-acquisition-via-oauth2-token-endpoint.gif
Output AC-003-token-acquisition-via-oauth2-token-endpoint.webm

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

Type "# AC-003: acquire_token() → POST /oauth2/token; DTU returns dtu-fake-cs-token"
Enter
Sleep 0.5s
Type "cargo nextest run -p prism-spec-engine -E 'test(PLUGIN_MIGRATION_001_E_003)' 2>&1 | grep -E 'PASS|FAIL|Summary'"
Enter
Sleep 15s
