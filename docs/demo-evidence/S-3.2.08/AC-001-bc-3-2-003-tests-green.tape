# AC-001 — All 15 BC-3.2.003 tests GREEN
# Traces to: BC-3.2.003 D-048, S-3.2.08 AC-001
# Shows: cargo test -p prism-query --test crowdstrike_session_isolation -- --nocapture 2>&1 | tail -25

Output docs/demo-evidence/S-3.2.08/AC-001-bc-3-2-003-tests-green.gif
Output docs/demo-evidence/S-3.2.08/AC-001-bc-3-2-003-tests-green.webm

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
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-3.2.08"
Enter
Sleep 500ms
# pre-warm compilation so the demo shows only the test run
Type "cargo test -p prism-query --test crowdstrike_session_isolation 2>&1 | tail -1"
Enter
Sleep 60s
Type "clear"
Enter
Sleep 500ms
Show

# --- demo ---
Type "cargo test -p prism-query --test crowdstrike_session_isolation -- --nocapture 2>&1 | tail -25"
Enter
Sleep 15s
Sleep 2s
