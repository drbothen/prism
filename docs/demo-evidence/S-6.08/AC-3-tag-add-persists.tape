Output docs/demo-evidence/S-6.08/AC-3-tag-add-persists.gif
Output docs/demo-evidence/S-6.08/AC-3-tag-add-persists.webm

Require cargo

Set FontFamily "FiraCode Nerd Font Mono"
Set Theme "Dracula"
Set Width 1200
Set Height 500
Set FontSize 14
Set Padding 20
Set Shell "bash"
Set PlaybackSpeed 1.0

# AC-3: POST /api/v1/devices/{id}/tags/ — tag add persists in subsequent device list

Hide
Type "cd /Users/jmagady/dev/prism/.worktrees/S-6.08-claroty && clear"
Enter
Sleep 1s
Show

Type "cargo test -p prism-dtu-claroty --features prism-dtu-claroty/dtu --test ac_3_tag_add_persists"
Enter
Sleep 10s
