Output docs/demo-evidence/S-DEMO-003/AC-007-credential-delete.gif
Output docs/demo-evidence/S-DEMO-003/AC-007-credential-delete.webm

Require cargo-nextest

Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 14
Set Width 1200
Set Height 700
Set Theme "Dracula"
Set Padding 20
Set PlaybackSpeed 1

# AC-007: prism credential delete — OrgId-keyed delete, idempotent, exit 0
# Red Gate test: test_handle_credential_delete_uses_org_id_keyed_namespace
Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-DEMO-003"
Enter
Sleep 200ms
Type "clear"
Enter
Sleep 200ms
Show

Type "# AC-007: prism credential delete — OrgId-keyed namespace, idempotent"
Enter
Sleep 500ms

Type "# Red Gate: test_handle_credential_delete_uses_org_id_keyed_namespace"
Enter
Sleep 400ms

Type "cargo nextest run -p prism-bin -E 'test(credential_delete)'"
Enter
Sleep 10s
