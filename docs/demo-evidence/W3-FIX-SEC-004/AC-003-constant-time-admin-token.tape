# AC-003: SEC-P3-003 — constant-time X-Admin-Token in 4 DTU clones (CWE-208)
#
# Traces to: BC-3.5.002 precondition 6
# Story: W3-FIX-SEC-004
#
# Code inspection showing subtle::ConstantTimeEq::ct_eq at all 8 handler sites
# (reset + configure) across the 4 DTU clones: armis, claroty, crowdstrike, slack.

Output docs/demo-evidence/W3-FIX-SEC-004/AC-003-constant-time-admin-token.gif
Output docs/demo-evidence/W3-FIX-SEC-004/AC-003-constant-time-admin-token.webm

Require cargo
Require grep

Set Shell "bash"
Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 14
Set Width 1200
Set Height 700
Set Theme "Dracula"
Set Padding 20
Set TypingSpeed 60ms

# Setup — hidden
Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/W3-FIX-SEC-004"
Enter
Sleep 1s
Show

# Show ct_eq call sites across all 4 DTU clones
Type "# AC-003 SEC-P3-003: subtle::ct_eq in all 4 DTU clone admin handlers"
Enter
Sleep 500ms
Type "grep -rn 'ct_eq' crates/prism-dtu-armis/src crates/prism-dtu-claroty/src crates/prism-dtu-crowdstrike/src crates/prism-dtu-slack/src"
Enter
Sleep 3s

# Run existing dtu_reset_auth tests to confirm behavior unchanged
Type "# existing dtu_reset_auth tests still pass (behavioral contract unchanged)"
Enter
Sleep 500ms
Type "cargo test -p prism-dtu-armis -p prism-dtu-claroty -p prism-dtu-crowdstrike -p prism-dtu-slack --features dtu --test dtu_reset_auth 2>&1 | grep 'test result'"
Enter
Sleep 45s
