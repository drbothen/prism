# PLUGIN-MIGRATION-001-E / AC-004
# Token Cached with TTL; Subsequent Calls Within TTL Reuse Cache
# Traces to: BC-2.01.016 SensorAuth stateless invariant, BC-2.17.001 KV store state
# Run from: docs/demo-evidence/PLUGIN-MIGRATION-001-E/

Output AC-004-token-cached-within-ttl-no-second-request.gif
Output AC-004-token-cached-within-ttl-no-second-request.webm

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

Type "# AC-004: KV cache hit — /oauth2/token called exactly once within TTL"
Enter
Sleep 0.5s
Type "cargo nextest run -p prism-spec-engine -E 'test(PLUGIN_MIGRATION_001_E_004)' 2>&1 | grep -E 'PASS|FAIL|Summary'"
Enter
Sleep 15s
