# E2E Binary CLI Capstone: `start` + `stop` flow using the real release binary.
#
# Demonstrates the full prism-dtu-demo-server binary wired end-to-end:
#   1. cargo build (release, features dtu,tls)
#   2. `start` in background — all 6 clones bind, URL table printed, PID file written
#   3. Health checks on crowdstrike (port 17080) and nvd (port 17085)
#   4. Inspect the PID file
#   5. `stop` — SIGTERM sent, graceful shutdown, PID file removed
#
# This complements AC-1..13 (library-API tests) with a capstone that exercises
# the CLI binary itself.

Output E2E-binary-cli.gif
Output E2E-binary-cli.webm

Require cargo

Set FontFamily "FiraCode Nerd Font Mono"
Set Theme "Dracula"
Set Width 1200
Set Height 750
Set FontSize 14
Set Padding 20
Set WaitTimeout 120s
Set Shell "bash"

Hide
  Type "cd /Users/jmagady/Dev/prism/.worktrees/S-6.20-dtu-demo-server"
  Enter
  Sleep 500ms
  Type "rm -f .prism-dtu-demo-server.pid; pkill -f 'prism-dtu-demo-server start' 2>/dev/null; true"
  Enter
  Sleep 500ms
Show

Type "# Step 1: build the release binary (features: dtu + tls)"
Enter
Sleep 300ms

Type "cargo build -p prism-dtu-demo-server --features dtu,tls --release"
Enter
Sleep 10s

Type "# Step 2: start the demo harness (all 6 clones) in the background"
Enter
Sleep 300ms

Type "./target/release/prism-dtu-demo-server start --config crates/prism-dtu-demo-server/configs/demo.toml &"
Enter
Sleep 3s

Type "# Step 3: health check — crowdstrike clone (port 17080)"
Enter
Sleep 300ms

Type "curl -s http://127.0.0.1:17080/dtu/health"
Enter
Sleep 1s

Type "# Step 4: health check — nvd clone (port 17085)"
Enter
Sleep 300ms

Type "curl -s http://127.0.0.1:17085/dtu/health"
Enter
Sleep 1s

Type "# Step 5: inspect the PID file written by the harness"
Enter
Sleep 300ms

Type "cat .prism-dtu-demo-server.pid && echo"
Enter
Sleep 500ms

Type "# Step 6: graceful shutdown via `stop`"
Enter
Sleep 300ms

Type "./target/release/prism-dtu-demo-server stop"
Enter
Sleep 3s

Type "# Step 7: PID file removed after clean shutdown"
Enter
Sleep 300ms

Type "ls .prism-dtu-demo-server.pid 2>/dev/null || echo '(PID file removed — clean shutdown confirmed)'"
Enter
Sleep 2s
