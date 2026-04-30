# AC-002 — Cross-org spec isolation: test_BC_3_1_001_cross_org_spec_isolation
# Traces to: BC-3.1.001 postcondition (OrgId isolation), S-3.1.05 AC-002
# Shows: specific cross-org spec isolation test with --nocapture output

Output docs/demo-evidence/S-3.1.05/AC-002-cross-org-spec-isolation.gif
Output docs/demo-evidence/S-3.1.05/AC-002-cross-org-spec-isolation.webm

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
# pre-warm compilation
Type "cargo test -p prism-spec-engine --test bc_3_1_001_test test_BC_3_1_001_cross_org_spec_isolation -- --nocapture 2>&1 | tail -1"
Enter
Sleep 30s
Show

# --- demo ---
Type "cargo test -p prism-spec-engine --test bc_3_1_001_test test_BC_3_1_001_cross_org_spec_isolation -- --nocapture"
Enter
Sleep 10s
Sleep 2s
