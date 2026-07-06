# AC-020 — Date-like literal vs Integer/Float/Bool column → E-QUERY-002
# S-PRISMQL-NATIVE-TEMPORAL-TYPING-001: ADR-052 §D4 three-way dispatch numeric arm
# check_temporal_literals resolves numeric col → QueryTypeMismatch (E-QUERY-002).
#
# Traces to: AC-020 (error-taxonomy §E-QUERY-041 three-way dispatch F-P5-MED-2), RG-016, RG-017, RG-018

Output docs/demo-evidence/S-PRISMQL-NATIVE-TEMPORAL-TYPING-001/AC-020-numeric-col-e-query-002.gif
Output docs/demo-evidence/S-PRISMQL-NATIVE-TEMPORAL-TYPING-001/AC-020-numeric-col-e-query-002.webm

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

Type "echo AC-020: date-like 2026-06-24 vs Integer/Float/Bool col -- E-QUERY-002 QueryTypeMismatch"
Enter
Sleep 1s

Type "cargo nextest run -p prism-query -E 'test(integer_col_date_like_e_query_002|float_col_date_like_e_query_002|bool_col_date_like_e_query_002)' 2>&1 | tail -5"
Enter
Sleep 10s

Sleep 2s
