# AC-6: Deterministic seeding — same query params produce identical responses
#
# Given CrowdstrikeClone is started with seed: 42,
# When GET /detects/queries/detects/v1 is called twice with the same query params,
# Then both responses are identical (deterministic seeding).
#
# Re-run: cd <worktree> && vhs docs/demo-evidence/S-6.07/AC-6.tape

Output docs/demo-evidence/S-6.07/AC-6.gif
Output docs/demo-evidence/S-6.07/AC-6.webm

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

Type "# AC-6: Determinism — seed=42 produces identical responses on repeated calls"
Enter
Sleep 500ms

Type "cargo test --features dtu --test ac_6_determinism"
Enter
Sleep 15s
