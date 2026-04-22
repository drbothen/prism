# AC-2: TenantId accepts valid inputs and round-trips correctly
# Acceptance criteria: TenantId::new acme_corp-01 returns Ok and as_str equals input

Output docs/demo-evidence/S-1.01/AC-2-tenant-id-valid-input.gif
Output docs/demo-evidence/S-1.01/AC-2-tenant-id-valid-input.webm

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

Type "/opt/homebrew/bin/cargo test --test ac_2_tenant_id_valid_input -p prism-core"
Enter
Sleep 15s
