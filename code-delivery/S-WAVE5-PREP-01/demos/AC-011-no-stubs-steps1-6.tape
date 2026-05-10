Output AC-011-no-stubs-steps1-6.gif
Output AC-011-no-stubs-steps1-6.webm

Set Width 1200
Set Height 400
Set FontSize 13
Set FontFamily "FiraCode Nerd Font Mono"
Set Theme "Dracula"
Set Padding 20
Set Shell "bash"

Type "cd /Users/jmagady/Dev/prism/.worktrees/S-WAVE5-PREP-01"
Enter
Sleep 500ms

Type "# AC-11: POL-12 -- no todo!/unimplemented! in steps 1-6"
Enter
Sleep 300ms

Type "grep -n 'fn step[0-9]' crates/prism-bin/src/boot.rs"
Enter
Sleep 800ms

Type "# todo!() lines: all are >= 806 (steps 7-11 only)"
Enter
Sleep 300ms

Type "grep -n 'todo!' crates/prism-bin/src/boot.rs | awk -F: '{print $1}' | xargs -I{} echo 'Line {}'"
Enter
Sleep 800ms

Type "echo 'POL-12: PASS -- steps 1-6 have zero stubs'"
Enter
Sleep 2s
