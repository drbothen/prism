# AC-7: Explicit backend="keyring" with failing probe is a hard error (no silent downgrade)
# BC-2.03.012 (backend selector hard error path)
# Story: S-1.06

Output docs/demo-evidence/S-1.06/AC-7-explicit-keyring-hard-error.gif
Output docs/demo-evidence/S-1.06/AC-7-explicit-keyring-hard-error.webm

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

Type "# AC-7: Explicit keyring + failing probe = hard Err, no silent downgrade (BC-2.03.012)"
Enter
Sleep 400ms

Type "cargo test -p prism-credentials -- test_BC_2_03_012_explicit_keyring_with_unavailable_probe_is_hard_error test_BC_2_03_012_explicit_file_with_missing_passphrase_is_hard_error 2>&1 | tail -4"
Enter
Wait+Screen /test result/
Sleep 2s
