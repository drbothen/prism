Output docs/demo-evidence/S-1.14/AC-006-credential-redaction.gif
Output docs/demo-evidence/S-1.14/AC-006-credential-redaction.webm

Require cargo

Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 14
Set Width 1200
Set Height 600
Set Theme "Dracula"
Set Padding 20
Set Shell "zsh"
Set WaitTimeout 120s

Hide
Type "cd /Users/jmagady/dev/prism/.worktrees/S-1.14-infusion-specs"
Enter
Type "clear"
Enter
Show

# AC-6 / BC-2.19.005: CredentialRef Debug output redacts values; errors never expose secrets
Type "# AC-6 (BC-2.19.005): CredentialRef <redacted>; E-INFUSE-005 hides secret values"
Enter
Sleep 500ms
Type "cargo test -q --package prism-spec-engine BC_2_19_005 2>&1 | grep -E 'ok|FAILED|finished'"
Enter
Wait+Screen /finished/
Sleep 2s
