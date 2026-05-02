# AC-003 + AC-007: Armis network-mode dispatch — 403 returned (auth gate active)
#
# BC-3.5.001 postcondition 1 + BC-3.5.002 postcondition 1 (CR-004)
# Exhaustive match in start_clone: Armis router enforces Bearer auth (HTTP 403)
# Network mode: same dispatch, Armis auth gate active for IsolationMode::Network
# Story: W3-FIX-CODE-002, branch: feature/W3-FIX-CODE-002, SHA: 85a453d4

Output AC-003-armis-network-dispatch-403.gif
Output AC-003-armis-network-dispatch-403.webm

Require cargo

Set Shell "bash"
Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 14
Set Width 1200
Set Height 700
Set Theme "Dracula"
Set Padding 20
Set PlaybackSpeed 1.0
Set WaitTimeout 180s

# --- Setup (hidden) ---
Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/W3-FIX-CODE-002"
Enter
Sleep 1s
Type "clear"
Enter
Sleep 500ms
Show

# --- Demo begins ---
Type "# AC-003/AC-007: exhaustive match dispatch (CR-004)"
Enter
Sleep 500ms

Type "# Armis router: GET /api/v1/devices without Bearer → HTTP 403"
Enter
Sleep 500ms

Type "# Network mode dispatch also correct (IsolationMode::Network)"
Enter
Sleep 1s

Type "cargo test -p prism-dtu-harness --features dtu --test cr004_dispatch_consolidation 2>&1 | tail -20"
Enter
Wait+Screen /test result:/
Sleep 2s
