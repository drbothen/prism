# AC-001: POST /dtu/reset without X-Admin-Token returns 401 on all 4 clones.
#
# BC-3.2.001 invariant 1: unauthenticated reset must be rejected.
# An HTTP request with no X-Admin-Token header must yield 401 +
# {"error": "missing or invalid admin token"} on every DTU clone.

Output docs/demo-evidence/W3-FIX-SEC-002/AC-001-reset-without-token-returns-401.gif
Output docs/demo-evidence/W3-FIX-SEC-002/AC-001-reset-without-token-returns-401.webm

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

Type "# AC-001: POST /dtu/reset without X-Admin-Token → 401 (CWE-306 fix)"
Enter
Sleep 500ms
Type "cargo test -p prism-dtu-claroty -p prism-dtu-crowdstrike -p prism-dtu-armis -p prism-dtu-slack --features dtu --test dtu_reset_auth -- test_AC_001 --test-threads=1"
Enter
Sleep 20s
