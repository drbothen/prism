# AC-003 (CR-018): CrowdStrike validate_org_id nil-instance guard — detection endpoints
#
# Runs the 6 tests in crates/prism-dtu-crowdstrike/tests/cr018_detections_org_id_guard.rs:
#   list_detection_ids: real-org absent → 401, real-org correct → 200, nil-instance absent → 200
#   get_detection_summaries: real-org absent → 401, real-org correct → 200, nil-instance absent → 200
#
# BC-3.5.002 precondition 3; BC-3.2.001 precondition 4
# Story: W3-FIX-CODE-005, branch: feature/W3-FIX-CODE-005, SHA: 652409cf

Output AC-003-cr018-detections-org-id-guard.gif
Output AC-003-cr018-detections-org-id-guard.webm

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

# --- AC-003: run the cr018 test suite ---
Type "# AC-003 (CR-018): CrowdStrike nil-instance guard on detection endpoints — 6 tests"
Enter
Sleep 500ms
Type "cargo test -p prism-dtu-crowdstrike --features dtu --test cr018_detections_org_id_guard 2>&1 | tail -12"
Enter
Sleep 90s
