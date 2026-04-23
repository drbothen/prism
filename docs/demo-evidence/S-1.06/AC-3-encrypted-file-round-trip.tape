# AC-3: EncryptedFileBackend AES-256-GCM set -> get round-trip
# BC-2.03.003 (encrypted file backend)
# Story: S-1.06

Output docs/demo-evidence/S-1.06/AC-3-encrypted-file-round-trip.gif
Output docs/demo-evidence/S-1.06/AC-3-encrypted-file-round-trip.webm

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

Type "# AC-3: AES-256-GCM EncryptedFileBackend set->get round-trip (BC-2.03.003)"
Enter
Sleep 400ms

Type "cargo test -p prism-credentials -- test_BC_2_03_003_encrypted_file_set_get_round_trip 2>&1 | tail -4"
Enter
Wait+Screen /test result/
Sleep 2s
