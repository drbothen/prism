# AC-8 / VP-034: AES-256-GCM encrypt->decrypt proptest — arbitrary byte sequences
# BC-2.03.003, VP-034
# Story: S-1.06

Output docs/demo-evidence/S-1.06/AC-8-VP034-encryption-round-trip.gif
Output docs/demo-evidence/S-1.06/AC-8-VP034-encryption-round-trip.webm

Require cargo

Set Shell "bash"
Set FontSize 14
Set FontFamily "FiraCode Nerd Font Mono"
Set Width 1200
Set Height 600
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

Type "# AC-8 / VP-034: AES-256-GCM encrypt->decrypt proptest (256 cases)"
Enter
Sleep 400ms

Type "cargo test -p prism-credentials -- test_BC_2_03_003_prop_encrypt_decrypt_round_trip 2>&1 | tail -4"
Enter
Wait+Screen /test result/
Sleep 2s
