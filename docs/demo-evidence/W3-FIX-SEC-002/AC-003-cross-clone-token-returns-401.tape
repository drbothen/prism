# AC-003: POST /dtu/reset on clone A with clone B's token returns 401.
#
# BC-3.5.002 precondition 3: the admin token is per-clone and is not shared.
# Two independent instances are started; clone B's token is presented to clone A;
# clone A must reject it with 401.

Output docs/demo-evidence/W3-FIX-SEC-002/AC-003-cross-clone-token-returns-401.gif
Output docs/demo-evidence/W3-FIX-SEC-002/AC-003-cross-clone-token-returns-401.webm

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

Type "# AC-003: cross-clone token rejected → 401 (token is per-instance)"
Enter
Sleep 500ms
Type "cargo test -p prism-dtu-claroty -p prism-dtu-crowdstrike -p prism-dtu-armis -p prism-dtu-slack --features dtu --test dtu_reset_auth -- test_AC_003 --test-threads=1"
Enter
Sleep 25s
