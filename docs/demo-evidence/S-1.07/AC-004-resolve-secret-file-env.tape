Output docs/demo-evidence/S-1.07/AC-004-resolve-secret-file-env.gif
Output docs/demo-evidence/S-1.07/AC-004-resolve-secret-file-env.webm

Require cargo

Set FontFamily "FiraCode Nerd Font Mono"
Set Theme "Catppuccin Mocha"
Set Width 1200
Set Height 600
Set FontSize 14
Set Padding 24

# AC-4: resolve_secret() FILE env var chain
# BC-2.03.009: {NAME}_FILE → reads file content (strips trailing newline)
# Priority: {NAME}_FILE > {NAME} direct env var > credential store
# K8s secret mount compatibility

Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-1.07-credential-crud"
Enter
Sleep 1s
Type "clear"
Enter
Sleep 500ms
Show

Type "# AC-4: resolve_secret() _FILE env var chain (BC-2.03.009)"
Enter
Sleep 800ms
Type "cargo test -p prism-credentials test_BC_2_03_009 2>&1 | grep -E 'test .* (ok|FAILED)|test result'"
Enter
Sleep 30s
