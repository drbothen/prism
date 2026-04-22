# AC-4: StorageDomain::all() returns exactly 16 variants
# Acceptance criteria: 16 variants with distinct column_family_name() strings

Output docs/demo-evidence/S-1.01/AC-4-storage-domain-all-16.gif
Output docs/demo-evidence/S-1.01/AC-4-storage-domain-all-16.webm

Set FontFamily "FiraCode Nerd Font Mono"
Set Theme "Catppuccin Mocha"
Set Width 1200
Set Height 700
Set FontSize 14
Set Padding 20
Set Shell "zsh"

Hide
Type "cd /Users/jmagady/dev/prism/.worktrees/S-1.01-foundational-types"
Enter
Sleep 2s
Show

Type "/opt/homebrew/bin/cargo test --test ac_4_storage_domain_all_16 -p prism-core"
Enter
Sleep 15s
