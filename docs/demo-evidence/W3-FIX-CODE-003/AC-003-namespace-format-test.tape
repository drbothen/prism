# AC-003: Namespace key format matches UUID/sensor/name pattern (BC-3.2.002 precondition 1)
#
# Shows: 1 test passes (test_AC_003), 2 tests ignored (require live OS keyring).
# Pure unit test — does NOT touch OS keyring. Verifies namespace_key_by_org_id
# produces "{uuid}/{sensor}/{name}" where org component is OrgId UUID, not slug.
#
# Story: W3-FIX-CODE-003 | SEC-004: FALSE POSITIVE (implementation was already complete)

Output AC-003-namespace-format-test.gif
Output AC-003-namespace-format-test.webm

Require cargo

Set FontFamily "FiraCode Nerd Font Mono"
Set Theme "Dracula"
Set Width 1200
Set Height 700
Set FontSize 14
Set Padding 20
Set WaitTimeout 120s
Set Shell "bash"

Hide
  Type "cd /Users/jmagady/Dev/prism/.worktrees/W3-FIX-CODE-003"
  Enter
  Sleep 500ms
Show

Type "# AC-003: namespace format test (pure unit test — no keyring service required)"
Enter
Sleep 500ms

Type "cargo test -p prism-credentials --test keyring_org_id 2>&1"
Enter
Wait+Screen /test result/
Sleep 2s
