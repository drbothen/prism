# AC-10: list("acme") after 3 stores returns exactly 3 (sensor, name) pairs
# BC-2.03.001 (trait postcondition — list)
# Story: S-1.06

Output docs/demo-evidence/S-1.06/AC-10-list-returns-all-entries.gif
Output docs/demo-evidence/S-1.06/AC-10-list-returns-all-entries.webm

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

Type "# AC-10: list() returns exactly 3 pairs after 3 stores (BC-2.03.001)"
Enter
Sleep 400ms

Type "cargo test -p prism-credentials -- test_BC_2_03_001_list_returns_all_stored_entries test_BC_2_03_001_list_empty_tenant_returns_empty_vec 2>&1 | tail -4"
Enter
Wait+Screen /test result/
Sleep 2s
