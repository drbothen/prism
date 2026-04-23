# AC-1: CredentialStore trait — set then get returns stored value
# BC-2.03.001 (trait postcondition), BC-2.03.002 (KeyringBackend)
# Story: S-1.06

Output docs/demo-evidence/S-1.06/AC-1-credential-store-set-get.gif
Output docs/demo-evidence/S-1.06/AC-1-credential-store-set-get.webm

Require cargo

Set Shell "bash"
Set FontSize 14
Set FontFamily "FiraCode Nerd Font Mono"
Set Width 1200
Set Height 600
Set Theme "Dracula"
Set Padding 20
Set TypingSpeed 40ms
Set WaitTimeout 60s

Hide
Type "cd /Users/jmagady/dev/prism/.worktrees/S-1.06-credential-store"
Enter
Sleep 2s
Type "clear"
Enter
Sleep 1s
Show

Type "# AC-1: CredentialStore set -> get returns stored value (BC-2.03.001)"
Enter
Sleep 400ms

Type "cargo test -p prism-credentials -- test_BC_2_03_001_set_then_get_returns_value 2>&1 | tail -4"
Enter
Wait+Screen /test result/
Sleep 2s
