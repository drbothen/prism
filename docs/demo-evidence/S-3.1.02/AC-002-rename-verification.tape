Output docs/demo-evidence/S-3.1.02/AC-002-rename-verification.gif
Output docs/demo-evidence/S-3.1.02/AC-002-rename-verification.webm

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

Type "# AC-1/AC-3 (BC-3.1.001): OrgSlug declared, TENANT_ID_PATTERN → ORG_SLUG_PATTERN, deprecation alias present"
Enter
Sleep 500ms

Type "grep -n 'OrgSlug\|ORG_SLUG_PATTERN\|deprecated.*TenantId' crates/prism-core/src/tenant.rs"
Enter
Sleep 2s

Type "# Confirm no non-deprecated TenantId in production source (only deprecation alias expected)"
Enter
Sleep 500ms

Type "grep -rn 'TenantId' crates/ --include='*.rs' | grep -v 'deprecated\|#\[allow\|test\|//!' | grep -v 'Binary' | head -20"
Enter
Sleep 2s

Type "# OrgSlug symbol count across workspace"
Enter
Sleep 500ms

Type "grep -rn 'OrgSlug' crates/ --include='*.rs' | grep -v 'Binary' | wc -l"
Enter
Sleep 2s
