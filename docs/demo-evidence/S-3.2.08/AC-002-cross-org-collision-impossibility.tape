# AC-002 — Cross-org structural separation (collision impossibility)
# Traces to: BC-3.2.003 D-048, S-3.2.08 AC-002
# Shows: test_BC_3_2_003_cross_org_collision_impossibility_1000_each -- --nocapture

Output docs/demo-evidence/S-3.2.08/AC-002-cross-org-collision-impossibility.gif
Output docs/demo-evidence/S-3.2.08/AC-002-cross-org-collision-impossibility.webm

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
Type "cargo test -p prism-query --test crowdstrike_session_isolation test_BC_3_2_003_cross_org_collision_impossibility_1000_each -- --nocapture 2>&1 | tail -1"
Enter
Sleep 60s
Type "clear"
Enter
Sleep 500ms
Show

# --- demo ---
Type "cargo test -p prism-query --test crowdstrike_session_isolation test_BC_3_2_003_cross_org_collision_impossibility_1000_each -- --nocapture"
Enter
Sleep 15s
Sleep 2s
