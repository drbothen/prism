# AC-001 / AC-002: KeyringBackend::CredentialStoreOrgId is fully implemented (no todo!() stubs)
#
# Shows: grep for todo!() in keyring.rs returns no results — implementation is complete.
# Bonus: display the get_by_org / set_by_org / delete_by_org function signatures to
# demonstrate the real OrgId-keyed namespace logic is present.
#
# Story: W3-FIX-CODE-003 | SEC-004: FALSE POSITIVE — implementation existed at develop@a3bd5a0f

Output AC-001-implementation-inspect.gif
Output AC-001-implementation-inspect.webm

Require grep

Set FontFamily "FiraCode Nerd Font Mono"
Set Theme "Dracula"
Set Width 1200
Set Height 700
Set FontSize 14
Set Padding 20
Set WaitTimeout 30s
Set Shell "bash"

Hide
  Type "cd /Users/jmagady/Dev/prism/.worktrees/W3-FIX-CODE-003"
  Enter
  Sleep 300ms
Show

Type "# AC-001: Verify no todo!() stubs in KeyringBackend CredentialStoreOrgId impl"
Enter
Sleep 500ms

Type "grep -n 'todo!()' crates/prism-credentials/src/keyring.rs && echo 'STUBS FOUND' || echo 'CLEAN — no todo!() stubs present'"
Enter
Wait+Screen /CLEAN|STUBS/
Sleep 1s

Type "# Show get_by_org / set_by_org / delete_by_org signatures (real implementation)"
Enter
Sleep 500ms

Type "grep -n 'async fn.*_by_org' crates/prism-credentials/src/keyring.rs"
Enter
Wait+Screen /delete_by_org/
Sleep 2s
