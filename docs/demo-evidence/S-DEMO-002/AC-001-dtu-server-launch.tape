# AC-001 / BC-2.22.001: DTU demo server launches and writes urls.json
# Story: S-DEMO-002
# Demonstrates: prism-dtu-demo-server starts all 4 sensor clones on ephemeral ports
# and writes .prism-dtu-demo-server.urls.json (the ready-signal used by the test harness).

Output docs/demo-evidence/S-DEMO-002/AC-001-dtu-server-launch.gif
Output docs/demo-evidence/S-DEMO-002/AC-001-dtu-server-launch.webm

Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 14
Set Width 1200
Set Height 600
Set Theme "Dracula"
Set Padding 20
Set Shell "bash"
Env PATH "/Users/jmagady/Dev/prism/.worktrees/S-DEMO-002/target/release:/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin"

Hide
Type "mkdir -p /tmp/prism-dtu-work-demo && cd /tmp/prism-dtu-work-demo && rm -f .prism-dtu-demo-server.urls.json .prism-dtu-demo-server.pid"
Enter
Sleep 500ms
Show

Type "# Step 1: Launch DTU demo server (all 4 sensor clones on ephemeral ports)"
Enter
Sleep 300ms

Type "prism-dtu-demo-server start --config /Users/jmagady/Dev/prism/.worktrees/S-DEMO-002/crates/prism-bin/fixtures/e2e-demo/demo.toml &"
Enter
Sleep 4s

Type "# Step 2: Read the urls.json ready-signal (AC-001 synchronization point)"
Enter
Sleep 300ms

Type "cat .prism-dtu-demo-server.urls.json | python3 -m json.tool"
Enter
Sleep 1s

Type "# AC-001: DTU server bound all 4 clones and wrote urls.json within 10s"
Enter
Sleep 2s

Hide
Type "kill %1 2>/dev/null; true"
Enter
Show
