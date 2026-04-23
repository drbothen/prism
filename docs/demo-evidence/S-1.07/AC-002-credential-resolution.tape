Output docs/demo-evidence/S-1.07/AC-002-credential-resolution.gif
Output docs/demo-evidence/S-1.07/AC-002-credential-resolution.webm

Require cargo

Set FontFamily "FiraCode Nerd Font Mono"
Set Theme "Catppuccin Mocha"
Set Width 1200
Set Height 600
Set FontSize 14
Set Padding 24

# AC-2: Credential resolution at sensor query time
# BC-2.03.006: resolver returns opaque reference, never raw value
# CredentialResolver::resolve() for (tenant, sensor, name) → CredentialRef (not raw value)

Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-1.07-credential-crud"
Enter
Sleep 1s
Type "clear"
Enter
Sleep 500ms
Show

Type "# AC-2: Credential resolution — opaque ref, no raw value leak (BC-2.03.006)"
Enter
Sleep 800ms
Type "cargo test -p prism-credentials test_BC_2_03_006 2>&1 | grep -E 'test .* (ok|FAILED)|test result'"
Enter
Sleep 30s
