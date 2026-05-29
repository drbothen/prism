Output AC-004-missing-param-error.gif
Output AC-004-missing-param-error.webm

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

Type "# AC-4: tool_query with missing 'query' field → McpError -32602 InvalidParams (no panic)"
Enter
Sleep 500ms

Type "# BC-2.10.007: structured error responses; required-field validation"
Enter
Sleep 300ms

Type "cargo nextest run -p prism-mcp -E 'test(BC_2_10_007_map_prism_error_mcp_parameter_invalid_to_32602)' --no-capture 2>&1 | tail -20"
Enter
Sleep 8s

Type "# Returns -32602 with field name; no unwrap/panic anywhere in the handler"
Enter
Sleep 2s
