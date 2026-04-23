# AC-4: TLS mode — binary accepts --tls flag; clones serve HTTPS with self-signed cert.
#
# Drives the AC-4 integration tests which verify TLS compilation and HTTPS endpoint
# reachability with certificate acceptance (InsecureSkipVerify equivalent).

Output AC-4-tls-mode.gif
Output AC-4-tls-mode.webm

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

Type "# AC-4: TLS mode -- self-signed cert, HTTPS clones serve fixture content"
Enter
Sleep 500ms

Type "cargo test --test ac_4_tls --features dtu,tls -- --nocapture 2>&1 | tail -20"
Enter
Wait+Screen /test result/
Sleep 2s
