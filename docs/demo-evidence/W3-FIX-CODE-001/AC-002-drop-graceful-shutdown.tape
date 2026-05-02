# AC-002: Drop honors 5-second graceful shutdown -- no immediate handle.abort()
#
# BC-3.5.001 EC-004 -- "waits up to 5s for graceful exit, then calls handle.abort()"
# Drop semantics chosen: remove-abort (axum::Server::with_graceful_shutdown drains in-flight requests)
# VP-124 (drop releases resources), VP-130 (drop grace period)
# Story: W3-FIX-CODE-001, branch: feature/W3-FIX-CODE-001, SHA: 8d4a472a

Output AC-002-drop-graceful-shutdown.gif
Output AC-002-drop-graceful-shutdown.webm

Require cargo

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
Type "cd /Users/jmagady/Dev/prism/.worktrees/W3-FIX-CODE-001"
Enter
Sleep 1s
Type "clear"
Enter
Sleep 500ms
Show

# --- Demo begins ---
Type "# AC-002: Drop honors 5-second graceful shutdown budget"
Enter
Sleep 500ms

Type "# Drop semantics: remove-abort (graceful drain via axum::with_graceful_shutdown)"
Enter
Sleep 500ms

Type "# Test: spawn harness, drop it within 5s timeout gate (no immediate abort)"
Enter
Sleep 500ms

Type "# BC-3.5.001 EC-004; VP-124 (drop releases ports); VP-130 (drop grace period)"
Enter
Sleep 1s

Type "cargo test -p prism-dtu-harness --features dtu --test failure_scope_test test_AC_002 -- --nocapture 2>&1"
Enter
Wait+Screen /test result/
Sleep 2s
