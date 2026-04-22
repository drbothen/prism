Output docs/demo-evidence/S-6.08/AC-4-tag-remove.gif
Output docs/demo-evidence/S-6.08/AC-4-tag-remove.webm

Require cargo

Set FontFamily "FiraCode Nerd Font Mono"
Set Theme "Dracula"
Set Width 1200
Set Height 500
Set FontSize 14
Set Padding 20
Set Shell "bash"
Set PlaybackSpeed 1.0

# AC-4: DELETE /api/v1/devices/{id}/tags/{key} — tag removed from subsequent device list

Hide
Type "cd /Users/jmagady/dev/prism/.worktrees/S-6.08-claroty && clear"
Enter
Sleep 1s
Show

Type "cargo test -p prism-dtu-claroty --features prism-dtu-claroty/dtu --test ac_4_tag_remove"
Enter
Sleep 10s
