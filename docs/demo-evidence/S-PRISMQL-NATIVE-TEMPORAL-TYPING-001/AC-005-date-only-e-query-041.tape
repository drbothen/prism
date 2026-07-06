# AC-005 — Date-only / offset-less vs Datetime column → E-QUERY-041
# S-PRISMQL-NATIVE-TEMPORAL-TYPING-001: ADR-052 §D4 Option-A AST walker
# Parser emits RawTemporalLiteral; check_temporal_literals fires E-QUERY-041.
# Both SQL mode and pipe mode verified.
#
# Traces to: AC-005 (BC-2.11.021 E-QUERY-041, BC-2.11.003, BC-2.11.004), RG-004, RG-005

Output docs/demo-evidence/S-PRISMQL-NATIVE-TEMPORAL-TYPING-001/AC-005-date-only-e-query-041.gif
Output docs/demo-evidence/S-PRISMQL-NATIVE-TEMPORAL-TYPING-001/AC-005-date-only-e-query-041.webm

Require cargo

Set Shell "zsh"
Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 18
Set Width 1200
Set Height 600
Set Theme "Catppuccin Mocha"
Set WindowBar Colorful
Set BorderRadius 8
Set Padding 20
Set Framerate 30
Set TypingSpeed 40ms

Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-PRISMQL-NATIVE-TEMPORAL-TYPING-001"
Enter
Sleep 1s
Type "clear"
Enter
Sleep 500ms
Show

Type "echo AC-005: date-only 2026-06-24 vs Datetime col -- E-QUERY-041 from check_temporal_literals"
Enter
Sleep 1s

Type "grep -m1 'E-QUERY-041: The value' crates/prism-core/src/error.rs"
Enter
Sleep 1s

Type "cargo nextest run -p prism-query -E 'test(e_query_041_sql_mode_date_only|e_query_041_pipe_mode_date_only)' 2>&1 | tail -5"
Enter
Sleep 10s

Sleep 2s
