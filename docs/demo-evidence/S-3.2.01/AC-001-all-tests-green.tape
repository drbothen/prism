# AC-001 — All multi-tenant tests GREEN
# Traces to: BC-3.2.001, S-3.2.01 AC-001 through AC-007
# Shows: cargo test -p prism-dtu-claroty --features dtu --test multi_tenant

Output docs/demo-evidence/S-3.2.01/AC-001-all-tests-green.gif
Output docs/demo-evidence/S-3.2.01/AC-001-all-tests-green.webm

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
# pre-warm compilation so the demo shows the test run
Type "cargo test -p prism-dtu-claroty --features dtu --test multi_tenant 2>&1 | tail -1"
Enter
Sleep 15s
Show

# --- demo ---
Type "cargo test -p prism-dtu-claroty --features dtu --test multi_tenant"
Enter
Sleep 15s
Sleep 2s
