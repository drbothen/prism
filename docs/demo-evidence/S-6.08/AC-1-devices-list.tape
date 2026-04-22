Output docs/demo-evidence/S-6.08/AC-1-devices-list.gif
Output docs/demo-evidence/S-6.08/AC-1-devices-list.webm

Require cargo

Set FontFamily "FiraCode Nerd Font Mono"
Set Theme "Dracula"
Set Width 1200
Set Height 500
Set FontSize 14
Set Padding 20
Set Shell "bash"
Set PlaybackSpeed 1.0

# AC-1: POST /api/v1/devices with valid Bearer returns 20 devices
# each with device_uid, name, ip_list, risk_level

Hide
Type "cd /Users/jmagady/dev/prism/.worktrees/S-6.08-claroty && clear"
Enter
Sleep 1s
Show

Type "cargo test -p prism-dtu-claroty --features prism-dtu-claroty/dtu --test ac_1_devices_list"
Enter
Sleep 10s
