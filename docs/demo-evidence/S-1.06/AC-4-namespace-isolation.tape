# AC-4: Namespace isolation — acme credentials not visible to beta tenant
# BC-2.03.004 (namespace invariant)
# Story: S-1.06

Output docs/demo-evidence/S-1.06/AC-4-namespace-isolation.gif
Output docs/demo-evidence/S-1.06/AC-4-namespace-isolation.webm

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

Type "# AC-4: Tenant 'beta' cannot read tenant 'acme' credentials (BC-2.03.004)"
Enter
Sleep 400ms

Type "cargo test -p prism-credentials -- test_BC_2_03_004 2>&1 | tail -4"
Enter
Wait+Screen /test result/
Sleep 2s
