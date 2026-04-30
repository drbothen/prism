# AC-001 — All 7 spec-engine tests GREEN
# Traces to: BC-3.1.001, S-3.1.05 AC-001
# Shows: cargo test -p prism-spec-engine --test bc_3_1_001_test 2>&1 | tail -20

Output docs/demo-evidence/S-3.1.05/AC-001-all-7-tests-green.gif
Output docs/demo-evidence/S-3.1.05/AC-001-all-7-tests-green.webm

Require cargo

Set FontFamily "FiraCode Nerd Font Mono"
Set Theme "Dracula"
Set Width 1200
Set Height 600
Set FontSize 14
Set Padding 20
Set Shell "bash"

# --- setup (hidden) ---
Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-3.1.05"
Enter
Sleep 500ms
# pre-warm compilation so the demo shows only the test run
Type "cargo test -p prism-spec-engine --test bc_3_1_001_test 2>&1 | tail -1"
Enter
Sleep 30s
Show

# --- demo ---
Type "cargo test -p prism-spec-engine --test bc_3_1_001_test 2>&1 | tail -20"
Enter
Sleep 15s
Sleep 2s
