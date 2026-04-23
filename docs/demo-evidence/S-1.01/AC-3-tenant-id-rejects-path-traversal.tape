# AC-3: TenantId rejects path-traversal and invalid characters
# Acceptance criteria: TenantId::new ../etc returns Err -- dot and slash outside allowed set

Output docs/demo-evidence/S-1.01/AC-3-tenant-id-rejects-path-traversal.gif
Output docs/demo-evidence/S-1.01/AC-3-tenant-id-rejects-path-traversal.webm

Set FontFamily "FiraCode Nerd Font Mono"
Set Theme "Catppuccin Mocha"
Set Width 1200
Set Height 600
Set FontSize 14
Set Padding 20
Set Shell "zsh"

Hide
Type "cd /Users/jmagady/dev/prism/.worktrees/S-1.01-foundational-types"
Enter
Sleep 2s
Show

Type "/opt/homebrew/bin/cargo test --test ac_3_tenant_id_rejects_path_traversal -p prism-core"
Enter
Sleep 15s
