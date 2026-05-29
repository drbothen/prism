Output AC-002-injection-defense.gif
Output AC-002-injection-defense.webm

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

Type "# AC-2: Prompt injection 'ignore previous instructions' → -32002 InjectionDetected"
Enter
Sleep 500ms

Type "# BC-2.09.003: InjectionScanner fires BEFORE domain logic at every tool boundary"
Enter
Sleep 300ms

Type "cargo nextest run -p prism-mcp -E 'test(BC_2_09_003_injection_scan_rejects_malicious_input)' --no-capture 2>&1 | tail -20"
Enter
Sleep 8s

Type "# InjectionScanner rejects input; QueryEngine is NEVER called (BC-2.09.001)"
Enter
Sleep 2s
