Output docs/demo-evidence/S-3.1.02/AC-001-workspace-tests-green.gif
Output docs/demo-evidence/S-3.1.02/AC-001-workspace-tests-green.webm

Require cargo

Set Shell "zsh"
Set FontSize 14
Set FontFamily "FiraCode Nerd Font Mono"
Set Width 1200
Set Height 600
Set Padding 20
Set Theme "Dracula"

Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-3.1.02"
Enter
Sleep 500ms
Show

Type "# AC-4 (BC-3.1.001): cargo test --workspace passes with zero failures after TenantId → OrgSlug rename"
Enter
Sleep 500ms

Type "cargo test --workspace --all-features 2>&1 | tail -20"
Enter
Sleep 130s
