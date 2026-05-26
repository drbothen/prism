# PLUGIN-MIGRATION-001-E / AC-006
# 401 Response Triggers Token Refresh + Single Retry via PipelineExecutor
# Traces to: BC-2.01.016 acquire_token forced-refresh, VP-150 OAuth2 refresh-on-401
# Run from: docs/demo-evidence/PLUGIN-MIGRATION-001-E/

Output AC-006-401-triggers-plugin-token-refresh-and-retry.gif
Output AC-006-401-triggers-plugin-token-refresh-and-retry.webm

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

Type "# AC-006: 401 injection → PipelineExecutor refresh-on-401 via plugin acquire_token (VP-150)"
Enter
Sleep 0.5s
Type "cargo nextest run -p prism-spec-engine -E 'test(PLUGIN_MIGRATION_001_E_006)' 2>&1 | grep -E 'PASS|FAIL|Summary'"
Enter
Sleep 15s
