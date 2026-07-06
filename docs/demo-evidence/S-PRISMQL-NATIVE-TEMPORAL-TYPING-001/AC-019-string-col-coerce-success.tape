# AC-019 — Date-like literal vs String/Utf8 column → COERCE, SUCCESS
# S-PRISMQL-NATIVE-TEMPORAL-TYPING-001: ADR-052 §D4 RISK-5 RESOLVED BY DESIGN
# check_temporal_literals resolves String col → rewrites RawTemporalLiteral → Literal::String.
#
# Traces to: AC-019 (BC-2.11.003 coercion success path), RG-013, RG-014, RG-031

Output docs/demo-evidence/S-PRISMQL-NATIVE-TEMPORAL-TYPING-001/AC-019-string-col-coerce-success.gif
Output docs/demo-evidence/S-PRISMQL-NATIVE-TEMPORAL-TYPING-001/AC-019-string-col-coerce-success.webm

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

Type "echo AC-019: date-like 2026-06-24 vs String col -- RawTemporalLiteral coerced to String -- SUCCESS"
Enter
Sleep 1s

Type "cargo nextest run -p prism-query -E 'test(string_col_coercion_date_only_succeeds|string_col_coercion_offset_less_succeeds|string_col_coercion_space_sep_succeeds)' 2>&1 | tail -5"
Enter
Sleep 10s

Sleep 2s
