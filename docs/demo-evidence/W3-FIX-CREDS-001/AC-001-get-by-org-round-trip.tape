# AC-001: get_by_org returns credential stored under {org_id_uuid}/{sensor}/{name}
# BC-3.2.002 postcondition 1 — set+get round-trip via EncryptedFileBackend
#
# Story: W3-FIX-CREDS-001 | AC: AC-001

Output docs/demo-evidence/W3-FIX-CREDS-001/AC-001-get-by-org-round-trip.gif
Output docs/demo-evidence/W3-FIX-CREDS-001/AC-001-get-by-org-round-trip.webm

Set Shell "bash"
Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 14
Set Width 1200
Set Height 600
Set Theme "Dracula"
Set Padding 20
Set PlaybackSpeed 1.0
Set TypingSpeed 40ms
Set WaitTimeout 120s

Hide
Type "source /Users/jmagady/.bash_profile 2>/dev/null" Enter
Sleep 1s
Type "cd /Users/jmagady/Dev/prism/.worktrees/W3-FIX-CREDS-001" Enter
Sleep 1s
Type "clear" Enter
Sleep 1s
Show

Type "# AC-001: get_by_org returns credential stored under org_id_uuid/sensor/name" Enter
Sleep 500ms
Type "# BC-3.2.002 postcondition 1 — set+get round-trip via EncryptedFileBackend" Enter
Sleep 500ms
Type "/opt/homebrew/bin/cargo nextest run -p prism-credentials --test bc_3_2_002_trait_impl test_BC_3_2_002_AC_001 2>&1" Enter
Wait+Screen@120s /Summary/
Sleep 2s
