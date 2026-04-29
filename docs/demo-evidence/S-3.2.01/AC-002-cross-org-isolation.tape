# AC-002 — Cross-org isolation: test_bc_3_2_001_http_cross_org_tag_not_visible_to_other_org
# Traces to: BC-3.2.001 postcondition 1, S-3.2.01 AC-001
# Shows: the specific cross-org HTTP isolation test with --nocapture output

Output docs/demo-evidence/S-3.2.01/AC-002-cross-org-isolation.gif
Output docs/demo-evidence/S-3.2.01/AC-002-cross-org-isolation.webm

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
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-3.2.01"
Enter
Sleep 500ms
# pre-warm compilation
Type "cargo test -p prism-dtu-claroty --features dtu --test multi_tenant test_bc_3_2_001_http_cross_org_tag_not_visible_to_other_org -- --nocapture 2>&1 | tail -1"
Enter
Sleep 15s
Show

# --- demo ---
Type "cargo test -p prism-dtu-claroty --features dtu --test multi_tenant test_bc_3_2_001_http_cross_org_tag_not_visible_to_other_org -- --nocapture"
Enter
Sleep 10s
Sleep 2s
