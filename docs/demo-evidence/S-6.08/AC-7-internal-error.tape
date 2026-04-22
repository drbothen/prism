Output docs/demo-evidence/S-6.08/AC-7-internal-error.gif
Output docs/demo-evidence/S-6.08/AC-7-internal-error.webm

Require cargo

Set FontFamily "FiraCode Nerd Font Mono"
Set Theme "Dracula"
Set Width 1200
Set Height 500
Set FontSize 14
Set Padding 20
Set Shell "bash"
Set PlaybackSpeed 1.0

# AC-7: InternalError{at_request_n:1} — first POST returns HTTP 500 (E-SENSOR-002 path)

Hide
Type "cd /Users/jmagady/dev/prism/.worktrees/S-6.08-claroty && clear"
Enter
Sleep 1s
Show

Type "cargo test -p prism-dtu-claroty --features prism-dtu-claroty/dtu --test ac_7_internal_error"
Enter
Sleep 10s
