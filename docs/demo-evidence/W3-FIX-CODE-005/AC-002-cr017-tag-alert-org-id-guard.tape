# AC-002 (CR-017 / M-50-001): Armis validate_org_id dual-mode guard — tag and alert endpoints
#
# Runs the 8 tests in crates/prism-dtu-armis/tests/cr017_tag_alert_org_id_guard.rs:
#   post_device_tag: real-org absent → 401, real-org correct → 201, default absent → 201
#   delete_device_tag: real-org absent → 401, default absent allows request
#   get_alerts: real-org absent → 401, real-org correct → 200, default absent → 200
#
# BC-3.5.002 precondition 3; BC-3.2.001 precondition 4
# Story: W3-FIX-CODE-005, branch: feature/W3-FIX-CODE-005, SHA: 652409cf

Output AC-002-cr017-tag-alert-org-id-guard.gif
Output AC-002-cr017-tag-alert-org-id-guard.webm

Require cargo

Set Shell "bash"
Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 14
Set Width 1200
Set Height 700
Set Theme "Dracula"
Set Padding 20
Set PlaybackSpeed 1.0
Set WaitTimeout 120s

# --- Setup (hidden) ---
Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/W3-FIX-CODE-005"
Enter
Sleep 1s
Type "clear"
Enter
Sleep 500ms
Show

# --- AC-002: run the cr017 test suite ---
Type "# AC-002 (CR-017): Armis dual-mode guard on tag/alert endpoints — 8 tests"
Enter
Sleep 500ms
Type "cargo test -p prism-dtu-armis --features dtu --test cr017_tag_alert_org_id_guard 2>&1 | tail -12"
Enter
Sleep 90s
