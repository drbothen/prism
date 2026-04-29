Output AC-002-bijection-conflicts.gif
Output AC-002-bijection-conflicts.webm

Require cargo

Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 14
Set Width 1200
Set Height 600
Set Theme "Dracula"
Set Padding 20
Set PlaybackSpeed 1.0
Set Shell "bash"

# AC-002: Bijection conflict tests — SlugConflict, IdConflict, idempotent Ok
# Traces to BC-3.1.003 / BC-3.1.004 (3 RegistrationError variants)

# Setup in hidden preamble — cd is not part of the demo
Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-3.1.03"
Enter
Sleep 1s
Show

# Visible demo: run the conflict-focused test slice
Type "cargo test -p prism-core --test bc_3_1_003_org_registry test_BC_3_1_004_tv -- --nocapture"
Enter
Sleep 15s
