# AC-002 — Cross-org dispatch isolation (proptest)
# Traces to: BC-3.2.001 OrgId isolation postcondition, S-3.1.06 AC-002
# Shows: cargo test -p prism-sensors --all-features test_BC_3_2_001_proptest_cross_org_lookup_always_returns_empty -- --nocapture

Output docs/demo-evidence/S-3.1.06/AC-002-cross-org-dispatch.gif
Output docs/demo-evidence/S-3.1.06/AC-002-cross-org-dispatch.webm

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
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-3.1.06"
Enter
Sleep 500ms
# pre-warm compilation so the demo shows only the test run
Type "cargo test -p prism-sensors --all-features test_BC_3_2_001_proptest_cross_org_lookup_always_returns_empty -- --nocapture 2>&1 | tail -1"
Enter
Sleep 30s
Type "clear"
Enter
Sleep 500ms
Show

# --- demo ---
Type "cargo test -p prism-sensors --all-features test_BC_3_2_001_proptest_cross_org_lookup_always_returns_empty -- --nocapture"
Enter
Sleep 15s
Sleep 2s
