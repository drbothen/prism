# AC-007 — Full RFC-3339 vs Datetime column → SUCCESS
# S-PRISMQL-NATIVE-TEMPORAL-TYPING-001: ADR-052 §D4 Option-A
# Parser produces Literal::Timestamp; check_temporal_literals bypassed; query succeeds.
#
# Traces to: AC-007 (BC-2.11.003/BC-2.11.004 EC-11-003-002), RG-007, RG-011

Output docs/demo-evidence/S-PRISMQL-NATIVE-TEMPORAL-TYPING-001/AC-007-rfc3339-datetime-success.gif
Output docs/demo-evidence/S-PRISMQL-NATIVE-TEMPORAL-TYPING-001/AC-007-rfc3339-datetime-success.webm

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

Type "echo AC-007: full RFC-3339 2026-07-03T00:00:00Z vs Datetime col -- parser emits Timestamp -- SUCCESS"
Enter
Sleep 1s

Type "grep -n 'arrow_cast' crates/prism-query/src/pipe_sql_emitter.rs | grep Timestamp"
Enter
Sleep 1s

Type "cargo nextest run -p prism-query -E 'test(valid_rfc3339_utc_string_not_rejected|full_rfc3339_regression_guard)' 2>&1 | tail -5"
Enter
Sleep 10s

Sleep 2s
