# AC-7: TenantId serde JSON round-trip
# Acceptance criteria: Serialize to JSON, deserialize back, produces equal TenantId

Output docs/demo-evidence/S-1.01/AC-7-tenant-id-serde-round-trip.gif
Output docs/demo-evidence/S-1.01/AC-7-tenant-id-serde-round-trip.webm

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

Type "/opt/homebrew/bin/cargo test --test ac_7_tenant_id_serde_round_trip -p prism-core"
Enter
Sleep 15s
