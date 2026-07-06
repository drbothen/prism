# AC-030/031 — SQL GROUP BY / ORDER BY with date-like literal → E-QUERY-042
# S-PRISMQL-NATIVE-TEMPORAL-TYPING-001: ADR-052 §D4 v1.10
# check_temporal_literals detects RawTemporalLiteral in GROUP BY / ORDER BY position.
# Maps to codes::INVALID_PARAMS (-32602), NOT -32000 (INTERNAL_ERROR).
#
# Traces to: AC-030 (BC-2.11.021 GROUP BY → E-QUERY-042), AC-031 (ORDER BY), RG-035, RG-036

Output docs/demo-evidence/S-PRISMQL-NATIVE-TEMPORAL-TYPING-001/AC-030-031-group-by-order-by-e-query-042.gif
Output docs/demo-evidence/S-PRISMQL-NATIVE-TEMPORAL-TYPING-001/AC-030-031-group-by-order-by-e-query-042.webm

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

Type "echo AC-030/031: GROUP BY 2026-06-24 / ORDER BY 2026-06-24 -- E-QUERY-042 INVALID_PARAMS"
Enter
Sleep 1s

Type "grep -m1 'GROUP BY expects' crates/prism-core/src/error.rs"
Enter
Sleep 1s

Type "cargo nextest run -p prism-query -E 'test(group_by_date_like_rejects_e_query_042|order_by_date_like_rejects_e_query_042)' 2>&1 | tail -5"
Enter
Sleep 10s

Sleep 2s
