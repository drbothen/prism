# AC-3: Contain write persists to containment_store
#
# Given POST /devices/entities/devices-actions/v2?action_name=contain with {"ids": ["h-001"]},
# Then the response is HTTP 202 with containment_status: "contained",
# AND subsequent GET /devices/entities/devices/v2?ids=h-001 returns
# containment_status: "contained" in the host record (stateful write persists).
#
# Re-run: cd <worktree> && vhs docs/demo-evidence/S-6.07/AC-3.tape

Output docs/demo-evidence/S-6.07/AC-3.gif
Output docs/demo-evidence/S-6.07/AC-3.webm

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

Type "# AC-3: Contain write persists — subsequent GET reflects contained status"
Enter
Sleep 500ms

Type "cargo test --features dtu --test ac_3_contain_write"
Enter
Sleep 15s
