# AC-002: POST /dtu/reset with correct X-Admin-Token returns 200 on all 4 clones.
#
# BC-3.5.001 postcondition 3: authorised reset must succeed and clear state.
# Supplying the clone's own admin token must yield 200 {"status": "ok"}.
# This is the happy path — it must continue to work after the auth gate was added.

Output docs/demo-evidence/W3-FIX-SEC-002/AC-002-reset-with-correct-token-returns-200.gif
Output docs/demo-evidence/W3-FIX-SEC-002/AC-002-reset-with-correct-token-returns-200.webm

Require cargo

Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 14
Set Width 1200
Set Height 600
Set Theme "Dracula"
Set Padding 20
Set PlaybackSpeed 1

Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/W3-FIX-SEC-002 && clear"
Enter
Sleep 3s
Show

Type "# AC-002: correct admin token → 200 (happy path still works)"
Enter
Sleep 500ms
Type "cargo test -p prism-dtu-claroty -p prism-dtu-crowdstrike -p prism-dtu-armis -p prism-dtu-slack --features dtu --test dtu_reset_auth -- test_AC_002 --test-threads=1"
Enter
Sleep 20s
