# AC-2: CrowdStrike GET /detects/queries/detects/v1 returns fixture via demo server port.
#
# Drives the AC-2 integration test which starts the CrowdStrike clone, calls
# GET /detects/queries/detects/v1 with an Authorization header, and asserts 200 + resources.

Output AC-2-crowdstrike-fixture.gif
Output AC-2-crowdstrike-fixture.webm

Require cargo

Set FontFamily "FiraCode Nerd Font Mono"
Set Theme "Dracula"
Set Width 1200
Set Height 700
Set FontSize 14
Set Padding 20
Set Shell "bash"

Hide
  Type "cd /Users/jmagady/Dev/prism/.worktrees/S-6.20-dtu-demo-server"
  Enter
  Sleep 500ms
Show

Type "# AC-2: CrowdStrike fixture returns 200 + resources array via demo server"
Enter
Sleep 500ms

Type "cargo test --test ac_2_crowdstrike_fixture --features dtu -- --nocapture 2>&1 | tail -20"
Enter
Wait+Screen /test result/
Sleep 2s
