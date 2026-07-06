# AC-034 — Non-column LHS comparison (HAVING / function expr) → E-QUERY-042
# S-PRISMQL-NATIVE-TEMPORAL-TYPING-001: ADR-052 §D4 v1.10 NonColumnLhsComparison
# WHERE lower(hostname) = '2026-06-24' and HAVING max(ts) > '2026-06-24'
# rejected with E-QUERY-042 NonColumnLhsComparison -- NOT -32000 (INTERNAL_ERROR).
#
# Traces to: AC-034 (BC-2.11.021 non-column-LHS → E-QUERY-042), RG-039, HAVING RG

Output docs/demo-evidence/S-PRISMQL-NATIVE-TEMPORAL-TYPING-001/AC-034-non-col-lhs-e-query-042.gif
Output docs/demo-evidence/S-PRISMQL-NATIVE-TEMPORAL-TYPING-001/AC-034-non-col-lhs-e-query-042.webm

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

Type "echo AC-034: lower(hostname) = 2026-06-24 non-col LHS -- E-QUERY-042 NonColumnLhsComparison"
Enter
Sleep 1s

Type "grep -m1 'compared against a computed' crates/prism-core/src/error.rs"
Enter
Sleep 1s

Type "cargo nextest run -p prism-query -E 'test(non_column_lhs_date_like_e_query_042|having_agg_date_only_raises_e_query_042)' 2>&1 | tail -5"
Enter
Sleep 10s

Sleep 2s
