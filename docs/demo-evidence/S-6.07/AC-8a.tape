# AC-8a: reset() clears all stores; pre-reset session returns empty resources
#
# Given reset() is called,
# Then the containment store, detection status store, and session registry are all cleared,
# AND a Step 2 request carrying the PRE-RESET X-DTU-Session-Id returns HTTP 200
# with an empty resources array (EC-003: cleared session is a registry miss).
#
# Re-run: cd <worktree> && vhs docs/demo-evidence/S-6.07/AC-8a.tape

Output docs/demo-evidence/S-6.07/AC-8a.gif
Output docs/demo-evidence/S-6.07/AC-8a.webm

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

Type "# AC-8a: reset() clears stores; pre-reset session ID returns empty resources"
Enter
Sleep 500ms

Type "cargo test --features dtu --test ac_8_reset ac_8_reset_clears_session_registry"
Enter
Sleep 15s
