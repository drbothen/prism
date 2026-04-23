Output docs/demo-evidence/S-1.07/AC-003-secret-redaction.gif
Output docs/demo-evidence/S-1.07/AC-003-secret-redaction.webm

Require cargo

Set FontFamily "FiraCode Nerd Font Mono"
Set Theme "Catppuccin Mocha"
Set Width 1200
Set Height 600
Set FontSize 14
Set Padding 24

# AC-3: Secret<T> redaction in Display/Debug/Serialize
# BC-2.03.007: format!("{:?}", secret) == "SecretString([REDACTED])"
# format!("{}", secret) == "[REDACTED]"
# .expose() is the only accessor

Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-1.07-credential-crud"
Enter
Sleep 1s
Type "clear"
Enter
Sleep 500ms
Show

Type "# AC-3: Secret<T> redaction — Display/Debug never leak value (BC-2.03.007)"
Enter
Sleep 800ms
Type "cargo test -p prism-credentials test_BC_2_03_007 2>&1 | grep -E 'test .* (ok|FAILED)|test result'"
Enter
Sleep 30s
