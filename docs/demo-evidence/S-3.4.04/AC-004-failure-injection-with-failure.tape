# AC-004 — Failure injection via with_failure (BC-3.6.001)
# S-3.4.04: Timeout failure injection leaves OrgB unaffected (EC-001)
# Traces to BC-3.6.001 postcondition 1

Output docs/demo-evidence/S-3.4.04/AC-004-failure-injection-with-failure.gif
Output docs/demo-evidence/S-3.4.04/AC-004-failure-injection-with-failure.webm

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

# Success path — OrgA injected with Timeout failure, OrgB returns 200 unaffected
Type "cargo test -p prism-dtu-cyberint --features dtu --test harness_tests harness_tests::ac_failure_injection_via_with_failure"
Enter
Sleep 10s
Sleep 2s
