Output AC-003-response-envelope.gif
Output AC-003-response-envelope.webm

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

Type "# AC-3: ResponseEnvelope wraps tool results with _meta.trust_level + _meta.safety_flags"
Enter
Sleep 500ms

Type "# BC-2.09.008: every tool response wrapped; BC-2.09.005: trust-level metadata"
Enter
Sleep 300ms

Type "cargo nextest run -p prism-mcp -E 'test(BC_2_09_008_response_envelope_wrapping_with_trust_metadata)' --no-capture 2>&1 | tail -25"
Enter
Sleep 8s

Type "# Envelope: {_meta: {trust_level: 'untrusted_external', safety_flags: []}, results: [...]}"
Enter
Sleep 2s
