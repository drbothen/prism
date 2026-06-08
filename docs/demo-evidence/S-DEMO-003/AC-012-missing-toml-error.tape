Output docs/demo-evidence/S-DEMO-003/AC-012-missing-toml-error.gif
Output docs/demo-evidence/S-DEMO-003/AC-012-missing-toml-error.webm

Require cargo-nextest

Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 14
Set Width 1200
Set Height 700
Set Theme "Dracula"
Set Padding 20
Set PlaybackSpeed 1

# AC-012: missing prism.toml + no --org-slug → exit 2 + actionable error (HIGH-3)
# EC-006: --org-slug not in orgs → exit 2
Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-DEMO-003"
Enter
Sleep 200ms
Type "clear"
Enter
Sleep 200ms
Show

Type "# AC-012: prism credential set without prism.toml → clear error, exit 2"
Enter
Sleep 500ms

Type "# Live demo: missing config dir triggers config-invalid path"
Enter
Sleep 400ms

Type "./target/release/prism --config-dir /tmp/no-such-s-demo-003 credential set --sensor armis --name bearer_token 2>&1; echo exit_code=$?"
Enter
Sleep 4s

Type "# AC-012 unit test (RG-034-003 — in-process, no keychain)"
Enter
Sleep 400ms

Type "cargo nextest run -p prism-bin -E 'test(resolve_org_slug_errors)'"
Enter
Sleep 8s
