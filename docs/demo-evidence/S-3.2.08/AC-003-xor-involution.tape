# AC-003 — XOR involution (applying twice restores original bytes)
# Traces to: BC-3.2.003 D-048, S-3.2.08 AC-003
# Shows: test_BC_3_2_003_xor_org_involutive_table_driven -- --nocapture

Output docs/demo-evidence/S-3.2.08/AC-003-xor-involution.gif
Output docs/demo-evidence/S-3.2.08/AC-003-xor-involution.webm

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
Type "cargo test -p prism-query --test crowdstrike_session_isolation test_BC_3_2_003_xor_org_involutive_table_driven -- --nocapture 2>&1 | tail -1"
Enter
Sleep 60s
Type "clear"
Enter
Sleep 500ms
Show

# --- demo ---
Type "cargo test -p prism-query --test crowdstrike_session_isolation test_BC_3_2_003_xor_org_involutive_table_driven -- --nocapture"
Enter
Sleep 15s
Sleep 2s
