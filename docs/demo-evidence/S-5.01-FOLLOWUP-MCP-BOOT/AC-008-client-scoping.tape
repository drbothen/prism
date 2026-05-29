Output AC-008-client-scoping.gif
Output AC-008-client-scoping.webm

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

Type "# AC-8: client=acme vs client=globex — no cross-client data leakage (BC-2.10.004)"
Enter
Sleep 500ms

Type "# Stateless model: same PrismServer instance; data_source array scoped per client"
Enter
Sleep 300ms

Type "cargo nextest run -p prism-mcp -E 'test(cross_client)' --no-capture 2>&1 | tail -25"
Enter
Sleep 8s

Type "# cross_client_query_data_source_is_array PASS — no leakage across client contexts"
Enter
Sleep 2s
