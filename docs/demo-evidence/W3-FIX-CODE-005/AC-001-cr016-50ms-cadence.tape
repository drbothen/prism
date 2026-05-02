# AC-001 (CR-016): All three clone-specific poll mirrors updated to 50ms
#
# Verifies that the 10ms→50ms cadence fix (originally applied to clone_server.rs
# by W3-FIX-CODE-002 AC-004) has been propagated to the three sibling clone files:
#   - crates/prism-dtu-harness/src/clones/armis.rs      (poll_armis_test_hook)
#   - crates/prism-dtu-harness/src/clones/claroty.rs    (poll_claroty_test_hook)
#   - crates/prism-dtu-harness/src/clones/crowdstrike.rs (poll_test_hook_crowdstrike)
#
# BC-3.5.001 postcondition 5: combined wake-up rate drops from ~300/s to ~60/s.
# Story: W3-FIX-CODE-005, branch: feature/W3-FIX-CODE-005, SHA: 652409cf

Output AC-001-cr016-50ms-cadence.gif
Output AC-001-cr016-50ms-cadence.webm

Require grep

Set Shell "bash"
Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 14
Set Width 1200
Set Height 700
Set Theme "Dracula"
Set Padding 20
Set PlaybackSpeed 1.0
Set WaitTimeout 60s

# --- Setup (hidden) ---
Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/W3-FIX-CODE-005"
Enter
Sleep 1s
Type "clear"
Enter
Sleep 500ms
Show

# --- AC-001: confirm no from_millis(10) remains in clone files ---
Type "# AC-001 (CR-016): confirm from_millis(10) fully removed from harness clone files"
Enter
Sleep 500ms
Type "grep -r 'from_millis(10)' crates/prism-dtu-harness/src/clones/ && echo FAIL || echo 'PASS: zero occurrences'"
Enter
Sleep 2s

# --- show 50ms in each sibling ---
Type "# armis.rs — poll_armis_test_hook at line ~901"
Enter
Sleep 300ms
Type "grep -n 'from_millis(50)' crates/prism-dtu-harness/src/clones/armis.rs | head -3"
Enter
Sleep 1s

Type "# claroty.rs — poll_claroty_test_hook at line ~851"
Enter
Sleep 300ms
Type "grep -n 'from_millis(50)' crates/prism-dtu-harness/src/clones/claroty.rs | head -3"
Enter
Sleep 1s

Type "# crowdstrike.rs — poll_test_hook_crowdstrike at line ~1162"
Enter
Sleep 300ms
Type "grep -n 'from_millis(50)' crates/prism-dtu-harness/src/clones/crowdstrike.rs | head -3"
Enter
Sleep 2s
