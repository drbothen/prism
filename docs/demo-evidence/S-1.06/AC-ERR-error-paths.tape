# Error paths: EC-001 (NoEntry->None), EC-002 (corrupt decrypt), EC-003 (path traversal),
# EC-004 (explicit keyring hard error), EC-005 (empty passphrase)
# Story: S-1.06

Output docs/demo-evidence/S-1.06/AC-ERR-error-paths.gif
Output docs/demo-evidence/S-1.06/AC-ERR-error-paths.webm

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

Type "# Error paths EC-001..005: reject bad input, never panic (BC-2.03.003/008/012)"
Enter
Sleep 400ms

Type "cargo test -p prism-credentials -- test_BC_2_03_001_get_missing test_BC_2_03_003_decrypt_zero_byte test_BC_2_03_003_decrypt_wrong_key test_BC_2_03_008_rejects 2>&1 | tail -6"
Enter
Wait+Screen /test result/
Sleep 2s
