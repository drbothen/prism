# AC-DS-002: Runtime Display strings — E-CRED-005 / E-CRED-006 / E-CRED-007
# Story: S-MAINT-ECRED-TAXONOMY-SYNC-001 v1.1 — E-CRED Namespace Reconciliation
# ACs demonstrated: AC-001 (E-CRED-006), AC-002 (E-CRED-007), AC-003 (E-CRED-005 file-I/O),
#                   AC-009 (no-collision: E-CRED-005 only in resolve_secret.rs)

Output docs/demo-evidence/S-MAINT-ECRED-TAXONOMY-SYNC-001/AC-DS-002-runtime-display-strings.gif
Output docs/demo-evidence/S-MAINT-ECRED-TAXONOMY-SYNC-001/AC-DS-002-runtime-display-strings.webm

Set Shell "zsh"
Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 18
Set Width 1200
Set Height 600
Set Theme "Catppuccin Mocha"
Set WindowBar Colorful
Set BorderRadius 8
Set Padding 20
Set Framerate 30
Set TypingSpeed 50ms

Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-MAINT-ECRED-TAXONOMY-SYNC-001 && clear"
Enter
Sleep 2s
Show

Type "# AC-001/002: Canonical E-CRED-006 and E-CRED-007 Display strings (error.rs)"
Enter
Sleep 0.5s

Type "grep -n 'E-CRED-00[567]' crates/prism-core/src/error.rs"
Enter
Sleep 1s

Type "# AC-003: Canonical E-CRED-005 file-I/O strings (resolve_secret.rs)"
Enter
Sleep 0.5s

Type "grep -n 'E-CRED-005' crates/prism-credentials/src/resolve_secret.rs"
Enter
Sleep 1s

Type "# AC-009: No-collision — E-CRED-005 limited to resolve_secret.rs only"
Enter
Sleep 0.5s

Type "git grep 'E-CRED-005' -- crates/"
Enter
Sleep 2s
