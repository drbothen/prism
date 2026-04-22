# AC-7: Missing Authorization header returns HTTP 401
#
# Given a request to any auth-required endpoint without an Authorization header,
# Then the response is HTTP 401 with {"errors": [{"code": 401, "message": "..."}]}.
#
# Re-run: cd <worktree> && vhs docs/demo-evidence/S-6.07/AC-7.tape

Output docs/demo-evidence/S-6.07/AC-7.gif
Output docs/demo-evidence/S-6.07/AC-7.webm

Require cargo

Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 14
Set Width 1200
Set Height 600
Set Theme "Dracula"
Set Padding 24
Set WindowBar Colorful

Hide
Type "cd /Users/jmagady/dev/prism/.worktrees/S-6.07-crowdstrike"
Enter
Sleep 1s
Type "clear"
Enter
Sleep 500ms
Show

Type "# AC-7: No Authorization header -> HTTP 401 with error body"
Enter
Sleep 500ms

Type "cargo test --features dtu --test ac_7_auth"
Enter
Sleep 15s
