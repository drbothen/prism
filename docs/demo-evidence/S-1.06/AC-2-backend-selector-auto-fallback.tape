# AC-2: BackendSelector with backend="auto" and unavailable keyring -> EncryptedFileBackend
# BC-2.03.012 (backend selection fallback)
# Story: S-1.06

Output docs/demo-evidence/S-1.06/AC-2-backend-selector-auto-fallback.gif
Output docs/demo-evidence/S-1.06/AC-2-backend-selector-auto-fallback.webm

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

Type "# AC-2: backend='auto' falls back to EncryptedFileBackend (BC-2.03.012)"
Enter
Sleep 400ms

Type "cargo test -p prism-credentials -- test_BC_2_03_012_auto_with_unavailable_keyring_selects_file test_BC_2_03_012_container_auto_falls_back_to_file 2>&1 | tail -4"
Enter
Wait+Screen /test result/
Sleep 2s
