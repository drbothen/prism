Output AC-009-output-schema.gif
Output AC-009-output-schema.webm

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

Type "# AC-9: Each tool's outputSchema declares _meta envelope + results + _meta.safety_flags"
Enter
Sleep 500ms

Type "# BC-2.09.007: OutputSchema for type-safe LLM reasoning; no per-field safety_flag"
Enter
Sleep 300ms

Type "cargo nextest run -p prism-mcp -E 'test(BC_2_09_007_tool_registration_carries_output_schema_with_meta_fields)' --no-capture 2>&1 | tail -25"
Enter
Sleep 8s

Type "# outputSchema: {_meta: {trust_level, safety_flags: array}, results: array} VERIFIED"
Enter
Sleep 2s
