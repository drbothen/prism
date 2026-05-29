Output AC-007-feature-flag-denied.gif
Output AC-007-feature-flag-denied.webm

Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 13
Set Width 1200
Set Height 700
Set Theme "Dracula"
Set Padding 16

Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-5.01-FOLLOWUP-MCP-BOOT"
Enter
Sleep 500ms
Show

Type "# AC-7: FeatureFlagDenied → McpError -32002 'Feature flag denied:...' (BC-2.10.003)"
Enter
Sleep 500ms

Type "# WriteExecutor returns FeatureFlagDenied; map_prism_error routes to -32002"
Enter
Sleep 300ms

Type "cargo nextest run -p prism-mcp -E 'test(BC_2_10_007_map_prism_error_feature_flag_disabled_to_32002)' --no-capture 2>&1 | tail -20"
Enter
Sleep 8s

Type "# -32002 returned; tool remains visible in tools/list (stateless model per BC-2.10.003)"
Enter
Sleep 2s
