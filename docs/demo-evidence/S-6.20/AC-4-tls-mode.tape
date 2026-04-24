# AC-4: TLS mode — binary --tls flag generates self-signed cert, prints SHA-256
# fingerprint, and all 6 clones serve HTTPS via axum_server::bind_rustls.
#
# Evidence: TD-WV1-04 fix complete. Binary end-to-end HTTPS verified.

Output AC-4-tls-mode.gif
Output AC-4-tls-mode.webm

Set FontFamily "FiraCode Nerd Font Mono"
Set Theme "Dracula"
Set Width 1200
Set Height 700
Set FontSize 14
Set Padding 20
Set WaitTimeout 30s
Set Shell "bash"

Env PATH "/Users/jmagady/Dev/prism/.worktrees/TD-WV1-04-tls-harness-wiring/target/release:/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin"

Require prism-dtu-demo-server

Hide
  Type "cd /Users/jmagady/Dev/prism/.worktrees/TD-WV1-04-tls-harness-wiring"
  Enter
  Sleep 500ms
  Type "pkill -f prism-dtu-demo-server 2>/dev/null; sleep 1"
  Enter
  Sleep 1500ms
Show

Type "# AC-4: TLS mode — binary --tls → self-signed cert, SHA-256 fingerprint, HTTPS"
Enter
Sleep 500ms

Type "./target/release/prism-dtu-demo-server start --config crates/prism-dtu-demo-server/configs/demo.toml --tls &"
Enter
Sleep 4s

Type "# Fingerprint printed above; URL table shows https:// for all 6 clones"
Enter
Sleep 1000ms

Type "curl -sk https://127.0.0.1:17080/dtu/health"
Enter
Sleep 1500ms

Type "curl -sk https://127.0.0.1:17085/dtu/health"
Enter
Sleep 1500ms

Type "# Graceful shutdown via SIGTERM"
Enter
Sleep 500ms

Type "kill %1 2>/dev/null; wait %1 2>/dev/null; echo 'Server stopped'"
Enter
Sleep 2s

Hide
  Type "rm -f /tmp/prism-dtu-demo.pid /tmp/prism-dtu-urls.txt 2>/dev/null"
  Enter
  Sleep 500ms
Show

Sleep 2s
