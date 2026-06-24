# AC-2: HTTP 401 -> auth_invalid (NOT down) (BC-2.08.002 postcondition 1)
# EC-004 error path: connection refused -> connectivity=Down, auth=Unknown.
# 403 also yields auth=Invalid.

Output docs/demo-evidence/S-5.04/AC-02-auth-invalid-401-not-down.gif
Output docs/demo-evidence/S-5.04/AC-02-auth-invalid-401-not-down.webm

Require cargo

Set Shell "zsh"
Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 20
Set Width 1200
Set Height 600
Set Theme "Catppuccin Mocha"
Set WindowBar Colorful
Set BorderRadius 8
Set Padding 20
Set Framerate 30
Set TypingSpeed 50ms

Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-5.04"
Enter
Sleep 1s
Type "clear"
Enter
Sleep 500ms
Show

Type "cargo nextest run -p prism-mcp -E 'test(live_probe_401_returns_auth_invalid_not_down) + test(connection_refused_returns_down_auth_unknown) + test(live_probe_403_returns_auth_invalid)' --no-fail-fast 2>&1 | grep -E 'PASS|FAIL|Summary'"
Enter
Sleep 30s

Sleep 2s
