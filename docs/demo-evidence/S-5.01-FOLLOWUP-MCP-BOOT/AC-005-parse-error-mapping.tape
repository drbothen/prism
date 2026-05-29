Output AC-005-parse-error-mapping.gif
Output AC-005-parse-error-mapping.webm

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

Type "# AC-5: PrismError::ParseError → map_prism_error → McpError -32602 'PrismQL parse error:'"
Enter
Sleep 500ms

Type "# BC-2.10.007: error code mapping; parse errors get -32602 with detail message"
Enter
Sleep 300ms

Type "cargo nextest run -p prism-mcp -E 'test(BC_2_10_007_map_prism_error_parse_error_to_32602)' --no-capture 2>&1 | tail -20"
Enter
Sleep 8s

Type "cargo nextest run -p prism-mcp -E 'test(BC_2_10_007_parse_error_message_contains_prismql)' --no-capture 2>&1 | tail -15"
Enter
Sleep 8s

Type "# map_prism_error: ParseError => code -32602 + 'PrismQL parse error: {detail}'"
Enter
Sleep 2s
