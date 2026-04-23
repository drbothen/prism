Output docs/demo-evidence/S-1.07/AC-005-audit-logging.gif
Output docs/demo-evidence/S-1.07/AC-005-audit-logging.webm

Require cargo

Set FontFamily "FiraCode Nerd Font Mono"
Set Theme "Catppuccin Mocha"
Set Width 1200
Set Height 600
Set FontSize 14
Set Padding 24

# AC-5: Credential access audit event emission
# BC-2.03.010: AuditEvent contains operator_id, credential_name (not value),
#              access_type, and timestamp
# Emitted on both success and failure paths

Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-1.07-credential-crud"
Enter
Sleep 1s
Type "clear"
Enter
Sleep 500ms
Show

Type "# AC-5: Audit event emission — operator_id + credential_name + outcome (BC-2.03.010)"
Enter
Sleep 800ms
Type "cargo test -p prism-credentials test_BC_2_03_010 2>&1 | grep -E 'test .* (ok|FAILED)|test result'"
Enter
Sleep 30s
