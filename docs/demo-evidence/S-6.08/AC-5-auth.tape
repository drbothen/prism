Output docs/demo-evidence/S-6.08/AC-5-auth.gif
Output docs/demo-evidence/S-6.08/AC-5-auth.webm

Require cargo

Set FontFamily "FiraCode Nerd Font Mono"
Set Theme "Dracula"
Set Width 1200
Set Height 500
Set FontSize 14
Set Padding 20
Set Shell "bash"
Set PlaybackSpeed 1.0

# AC-5: Any request without Authorization header returns HTTP 401 with JSON body

Hide
Type "cd /Users/jmagady/dev/prism/.worktrees/S-6.08-claroty && clear"
Enter
Sleep 1s
Show

Type "cargo test -p prism-dtu-claroty --features prism-dtu-claroty/dtu --test ac_5_auth"
Enter
Sleep 10s
