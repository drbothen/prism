# PLUGIN-MIGRATION-001-E / AC-005
# Expired Token Triggers Re-acquisition
# Traces to: BC-2.01.016 acquire_token forced-refresh path
# Run from: docs/demo-evidence/PLUGIN-MIGRATION-001-E/

Output AC-005-expired-token-triggers-reacquisition.gif
Output AC-005-expired-token-triggers-reacquisition.webm

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

Type "# AC-005: Expired cache (expires_at_secs in past) → /oauth2/token called twice"
Enter
Sleep 0.5s
Type "cargo nextest run -p prism-spec-engine -E 'test(PLUGIN_MIGRATION_001_E_005)' 2>&1 | grep -E 'PASS|FAIL|Summary'"
Enter
Sleep 15s
