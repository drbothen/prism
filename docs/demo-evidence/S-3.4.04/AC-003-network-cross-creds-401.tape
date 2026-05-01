# AC-003 — Network cross-credentials return HTTP 401
# S-3.4.04: Cross-org credential mismatch on Network harness → HTTP 401
# Traces to BC-3.5.002 postcondition 2 (TV-3)

Output docs/demo-evidence/S-3.4.04/AC-003-network-cross-creds-401.gif
Output docs/demo-evidence/S-3.4.04/AC-003-network-cross-creds-401.webm

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
Set WaitTimeout 120s

Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-3.4.04"
Enter
Sleep 2s
Type "clear"
Enter
Sleep 1s
Show

# Success path — ac_network_cross_creds_401 verifies 401 on cross-org credential mismatch
Type "cargo test -p prism-dtu-cyberint --features dtu --test harness_tests harness_tests::ac_network_cross_creds_401"
Enter
Sleep 10s
Sleep 2s
