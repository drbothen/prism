# AC-8 and AC-9: TenantId length boundary validation
# Acceptance criteria: 64-char string is Ok (AC-8), 65-char string is Err (AC-9)

Output docs/demo-evidence/S-1.01/AC-8-AC-9-tenant-id-boundary.gif
Output docs/demo-evidence/S-1.01/AC-8-AC-9-tenant-id-boundary.webm

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

Type "/opt/homebrew/bin/cargo test --test ac_8_ac_9_tenant_id_boundary -p prism-core"
Enter
Sleep 15s
