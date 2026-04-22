# AC-1: start + bound_addr + GET /detects/queries/detects/v1 returns 200
#
# Given CrowdstrikeClone::start() is called,
# Then bound_addr() returns a valid socket address
# AND GET /detects/queries/detects/v1 with a valid Bearer token returns HTTP 200
# with a resources array.
#
# Re-run: cd <worktree> && vhs docs/demo-evidence/S-6.07/AC-1.tape

Output docs/demo-evidence/S-6.07/AC-1.gif
Output docs/demo-evidence/S-6.07/AC-1.webm

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

Type "# AC-1: Clone starts, binds ephemeral port, detections endpoint returns 200"
Enter
Sleep 500ms

Type "cargo test --features dtu --test ac_1_happy_path"
Enter
Sleep 15s
