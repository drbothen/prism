Output docs/demo-evidence/S-DEMO-003/AC-005-credential-set-no-echo.gif
Output docs/demo-evidence/S-DEMO-003/AC-005-credential-set-no-echo.webm

Require cargo-nextest

Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 14
Set Width 1200
Set Height 700
Set Theme "Dracula"
Set Padding 20
Set PlaybackSpeed 1

# AC-005: prism credential set reads value from STDIN, never echoes it (AD-017)
# Evidence: in-process unit tests + --help shows no --value flag
Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-DEMO-003"
Enter
Sleep 200ms
Type "clear"
Enter
Sleep 200ms
Show

Type "# AC-005: prism credential set — AD-017 no-echo, OrgId-keyed write"
Enter
Sleep 500ms

Type "# 1. Verify --value flag is forbidden (EC-005)"
Enter
Sleep 400ms

Type "./target/release/prism credential set --sensor crowdstrike --name client_id --value secret 2>&1 | head -5"
Enter
Sleep 3s

Type "# 2. Run in-process Red Gate tests (OrgId-keyed write + no-echo path)"
Enter
Sleep 400ms

Type "cargo nextest run -p prism-bin -E 'test(org_id_keyed)'"
Enter
Sleep 10s
