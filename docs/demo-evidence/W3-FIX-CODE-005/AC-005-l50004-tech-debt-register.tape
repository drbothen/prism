# AC-005 (L-50-004): TD-W3-POLL-NOTIFY-001 filed in tech-debt-register.md
#
# Confirms that TD-W3-POLL-NOTIFY-001 (tokio::sync::Notify-based cancellation for
# poll_test_hook busy-wait) is present in .factory/tech-debt-register.md.
# Filed per pass-50 finding L-50-004.
#
# BC-3.5.001 postcondition 5 (50ms cadence is acceptable short-term workaround;
# Notify-based replacement deferred to Wave 4).
# Story: W3-FIX-CODE-005, branch: feature/W3-FIX-CODE-005, SHA: 652409cf

Output AC-005-l50004-tech-debt-register.gif
Output AC-005-l50004-tech-debt-register.webm

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

# --- AC-005: show TD-W3-POLL-NOTIFY-001 in tech-debt-register.md ---
Type "# AC-005 (L-50-004): TD-W3-POLL-NOTIFY-001 filed in .factory/tech-debt-register.md"
Enter
Sleep 500ms
Type "grep -c 'TD-W3-POLL-NOTIFY-001' .factory/tech-debt-register.md && echo 'PASS: entry present' || echo 'FAIL: entry missing'"
Enter
Sleep 1s

Type "grep -n 'TD-W3-POLL-NOTIFY-001' .factory/tech-debt-register.md | head -2"
Enter
Sleep 2s
