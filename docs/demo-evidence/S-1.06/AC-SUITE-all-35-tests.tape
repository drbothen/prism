# Full test suite: all 35 tests pass — AC-1 through AC-10 + all ECs
# Story: S-1.06

Output docs/demo-evidence/S-1.06/AC-SUITE-all-35-tests.gif
Output docs/demo-evidence/S-1.06/AC-SUITE-all-35-tests.webm

Require cargo

Set Shell "bash"
Set FontSize 14
Set FontFamily "FiraCode Nerd Font Mono"
Set Width 1200
Set Height 700
Set Theme "Dracula"
Set Padding 20
Set TypingSpeed 40ms
Set WaitTimeout 120s

Hide
Type "cd /Users/jmagady/dev/prism/.worktrees/S-1.06-credential-store"
Enter
Sleep 2s
Type "clear"
Enter
Sleep 1s
Show

Type "# S-1.06 prism-credentials — full test suite (35 tests, AC-1..10 + ECs)"
Enter
Sleep 400ms

Type "cargo test -p prism-credentials 2>&1 | tail -8"
Enter
Wait+Screen /test result/
Sleep 2s
