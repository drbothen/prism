# AC-5: CredentialName rejects path traversal — "../../etc/passwd" is an error
# BC-2.03.008 (credential name sanitization)
# Story: S-1.06

Output docs/demo-evidence/S-1.06/AC-5-path-traversal-rejection.gif
Output docs/demo-evidence/S-1.06/AC-5-path-traversal-rejection.webm

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

Type "# AC-5: CredentialName rejects '../../etc/passwd' (BC-2.03.008)"
Enter
Sleep 400ms

Type "cargo test -p prism-credentials -- test_BC_2_03_008 2>&1 | tail -4"
Enter
Wait+Screen /test result/
Sleep 2s
