# AC-8b: Post-reset new Step 1 + Step 2 returns fixture baseline with "normal" status
#
# Given reset() is called,
# When a NEW Step 1 (GET /devices/queries/devices/v1) is issued with a fresh
# X-DTU-Session-Id and the returned IDs are fetched via Step 2,
# Then the Step 2 response contains host records with containment_status: "normal"
# (fixture baseline — containment store was cleared by reset()).
#
# Re-run: cd <worktree> && vhs docs/demo-evidence/S-6.07/AC-8b.tape

Output docs/demo-evidence/S-6.07/AC-8b.gif
Output docs/demo-evidence/S-6.07/AC-8b.webm

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

Type "# AC-8b: Post-reset fresh session returns fixture baseline: containment_status normal"
Enter
Sleep 500ms

Type "cargo test --features dtu --test ac_8_reset ac_8_reset_clears_containment_store"
Enter
Sleep 15s
