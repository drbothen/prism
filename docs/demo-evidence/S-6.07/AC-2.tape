# AC-2: Two-step pagination — Step 1 registers IDs, Step 2 returns matching records
#
# Given GET /devices/queries/devices/v1 is called (Step 1),
# Then the response resources array contains host IDs AND those IDs are registered
# in the session registry under the X-DTU-Session-Id header value;
# subsequent GET /devices/entities/devices/v2 with those IDs returns matching records.
#
# Re-run: cd <worktree> && vhs docs/demo-evidence/S-6.07/AC-2.tape

Output docs/demo-evidence/S-6.07/AC-2.gif
Output docs/demo-evidence/S-6.07/AC-2.webm

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

Type "# AC-2: Two-step pagination — Step 1 registers IDs, Step 2 returns detail records"
Enter
Sleep 500ms

Type "cargo test --features dtu --test ac_2_two_step_pagination"
Enter
Sleep 15s
