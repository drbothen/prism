Output docs/demo-evidence/S-1.07/AC-001-crud-confirmation-gate.gif
Output docs/demo-evidence/S-1.07/AC-001-crud-confirmation-gate.webm

Require cargo

Set FontFamily "FiraCode Nerd Font Mono"
Set Theme "Catppuccin Mocha"
Set Width 1200
Set Height 600
Set FontSize 14
Set Padding 24

# AC-1: configure_credential_source confirmation gate
# BC-2.03.005: update to existing credential returns ConfirmationRequired with E-CRED-003
# First write → Created; second write → ConfirmationRequired

Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-1.07-credential-crud"
Enter
Sleep 1s
Type "clear"
Enter
Sleep 500ms
Show

Type "# AC-1: Credential CRUD confirmation gate (BC-2.03.005)"
Enter
Sleep 800ms
Type "cargo test -p prism-credentials test_BC_2_03_005 2>&1 | grep -E 'test .* (ok|FAILED)|test result'"
Enter
Sleep 30s
