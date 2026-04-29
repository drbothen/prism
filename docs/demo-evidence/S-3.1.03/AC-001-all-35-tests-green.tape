Output AC-001-all-35-tests-green.gif
Output AC-001-all-35-tests-green.webm

Require cargo

Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 14
Set Width 1200
Set Height 600
Set Theme "Dracula"
Set Padding 20
Set PlaybackSpeed 1.0
Set Shell "bash"

# AC-001: All 35 OrgRegistry tests GREEN
# Traces to BC-3.1.001, BC-3.1.003, BC-3.1.004

# Setup in hidden preamble — cd is not part of the demo
Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-3.1.03"
Enter
Sleep 1s
Show

# Visible demo: run the full test suite
Type "cargo test -p prism-core --test bc_3_1_003_org_registry"
Enter
Sleep 15s
