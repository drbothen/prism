Output docs/demo-evidence/S-6.08/AC-8-reset.gif
Output docs/demo-evidence/S-6.08/AC-8-reset.webm

Require cargo

Set FontFamily "FiraCode Nerd Font Mono"
Set Theme "Dracula"
Set Width 1200
Set Height 500
Set FontSize 14
Set Padding 20
Set Shell "bash"
Set PlaybackSpeed 1.0

# AC-8: reset() clears tag store; subsequent device queries return empty tags (base fixture)

Hide
Type "cd /Users/jmagady/dev/prism/.worktrees/S-6.08-claroty && clear"
Enter
Sleep 1s
Show

Type "cargo test -p prism-dtu-claroty --features prism-dtu-claroty/dtu --test ac_8_reset"
Enter
Sleep 10s
