Output docs/demo-evidence/S-3.7.05/AC-002-quirks-pagination-oauth2.gif
Output docs/demo-evidence/S-3.7.05/AC-002-quirks-pagination-oauth2.webm

Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 14
Set Width 1000
Set Height 600
Set Theme "Dracula"
Set Padding 20
Set Shell "bash"

Require vhs
Require cargo

Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-3.7.05 && clear"
Enter
Sleep 2s
Show

Type "# AC-002/AC-003: 2-step pagination (IdPage->detail join) + OAuth2 token fixture fields"
Enter
Sleep 500ms
Type "cargo test -p prism-dtu-crowdstrike --features fixture-gen --test bc_3_4_crowdstrike_generator -- two_step oauth2 --nocapture 2>&1"
Enter
Sleep 25s
