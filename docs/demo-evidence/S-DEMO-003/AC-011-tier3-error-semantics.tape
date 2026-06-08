Output docs/demo-evidence/S-DEMO-003/AC-011-tier3-error-semantics.gif
Output docs/demo-evidence/S-DEMO-003/AC-011-tier3-error-semantics.webm

Require cargo-nextest

Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 14
Set Width 1200
Set Height 700
Set Theme "Dracula"
Set Padding 20
Set PlaybackSpeed 1

# AC-011: Tier-3 keyring miss → fall-through to Tier-4 (RG-034-002)
#         Tier-3 backend error → hard E-CRED-008 / BackendUnavailable (RG-034-005)
Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-DEMO-003"
Enter
Sleep 200ms
Type "clear"
Enter
Sleep 200ms
Show

Type "# AC-011: Tier-3 error semantics (E-CRED-008)"
Enter
Sleep 500ms

Type "# RG-034-002 (miss→Tier-4) + RG-034-005 (backend error→BackendUnavailable)"
Enter
Sleep 400ms

Type "cargo nextest run -p prism-credentials --features test-helpers -E 'test(bc_2_06_003)'"
Enter
Sleep 12s
