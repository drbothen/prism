# AC-032/033 — Pipe stats-by / sort with date-like literal → parse-time E-QUERY-001
# S-PRISMQL-NATIVE-TEMPORAL-TYPING-001: ADR-052 §D4 v1.10 parse-time rejection
# filter_parser.rs rejects bare literals in stats-by/sort positions at PARSE TIME.
# Error is E-QUERY-001 (parse/syntax), NOT E-QUERY-042 (plan/semantic).
#
# Traces to: AC-032 (BC-2.11.004 pipe stats-by rejection), AC-033 (sort), RG-037, RG-038

Output docs/demo-evidence/S-PRISMQL-NATIVE-TEMPORAL-TYPING-001/AC-032-033-pipe-stats-sort-e-query-001.gif
Output docs/demo-evidence/S-PRISMQL-NATIVE-TEMPORAL-TYPING-001/AC-032-033-pipe-stats-sort-e-query-001.webm

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

Type "echo AC-032/033: pipe stats by literal / sort literal -- parse-time E-QUERY-001"
Enter
Sleep 1s

Type "cargo nextest run -p prism-query -E 'test(pipe_stats_by_date_like_e_query_001|pipe_sort_date_like_e_query_001)' 2>&1 | tail -5"
Enter
Sleep 10s

Sleep 2s
