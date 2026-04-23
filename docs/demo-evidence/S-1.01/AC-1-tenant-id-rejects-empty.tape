# AC-1: TenantId rejects empty string and whitespace
# Acceptance criteria: TenantId::new empty string and whitespace return Err(PrismError::InvalidTenantId) with E-AUTH

Output docs/demo-evidence/S-1.01/AC-1-tenant-id-rejects-empty.gif
Output docs/demo-evidence/S-1.01/AC-1-tenant-id-rejects-empty.webm

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

Type "/opt/homebrew/bin/cargo test --test ac_1_tenant_id_rejects_empty -p prism-core"
Enter
Sleep 15s
