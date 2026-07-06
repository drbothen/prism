# AC-029 — SELECT projection date-like literal → COERCE to Literal::String, SUCCESS
# S-PRISMQL-NATIVE-TEMPORAL-TYPING-001: ADR-052 §D4 v1.10 projection-coerce arm
# SELECT '2026-06-24' FROM t -- non-comparison projection position.
# check_temporal_literals rewrites RawTemporalLiteral → Literal::String → query succeeds.
#
# Traces to: AC-029 (BC-2.11.021 non-comparison SELECT projection coerce), RG-023

Output docs/demo-evidence/S-PRISMQL-NATIVE-TEMPORAL-TYPING-001/AC-029-select-projection-coerce-success.gif
Output docs/demo-evidence/S-PRISMQL-NATIVE-TEMPORAL-TYPING-001/AC-029-select-projection-coerce-success.webm

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

Type "echo AC-029: SELECT 2026-06-24 FROM t -- projection coerces to String -- SUCCESS"
Enter
Sleep 1s

Type "cargo nextest run -p prism-query -E 'test(projection_position_coerces_to_string)' 2>&1 | tail -5"
Enter
Sleep 10s

Sleep 2s
