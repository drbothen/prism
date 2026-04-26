Output docs/demo-evidence/S-2.06/ac-3-adapter-registry.gif

Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 14
Set Width 1000
Set Height 600
Set Theme "Dracula"
Set Padding 20
Set Shell "bash"

Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-2.06-datasource-trait"
Enter
Sleep 1s
Type "clear"
Enter
Sleep 500ms
Show

Type "# AC-3: AdapterRegistry register+get, object-safety, sealed SensorAuth (BC-2.01.013)"
Enter
Type "cargo test -p prism-sensors --lib -- bc_2_01_013 --nocapture 2>&1 | tail -16"
Enter
Sleep 20s
Sleep 2s
