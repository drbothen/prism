# AC-RG-001: All 5 Red Gate tests GREEN
# Story: S-MAINT-ECRED-TAXONOMY-SYNC-001 v1.1 — E-CRED Namespace Reconciliation
# Covers: RG-ECRED-001/002/004/005 (prism-core) + RG-ECRED-003 + AC-010 (prism-credentials)
# ACs demonstrated: AC-001, AC-002, AC-003, AC-004, AC-007, AC-009, AC-010

Output docs/demo-evidence/S-MAINT-ECRED-TAXONOMY-SYNC-001/AC-RG-001-red-gate-tests-green.gif
Output docs/demo-evidence/S-MAINT-ECRED-TAXONOMY-SYNC-001/AC-RG-001-red-gate-tests-green.webm

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
Type `cd "$(git rev-parse --show-toplevel)" && clear`
Enter
Sleep 2s
Show

Type "# S-MAINT-ECRED-TAXONOMY-SYNC-001: Red Gate suite — prism-core"
Enter
Sleep 0.5s

Type "cargo nextest run -p prism-core -E 'test(e_cred)'"
Enter
Sleep 8s

Type "# Red Gate suite — prism-credentials (RG-ECRED-003 + AC-010)"
Enter
Sleep 0.5s

Type "cargo nextest run -p prism-credentials -E 'test(BC_2_03_009)'"
Enter
Sleep 8s
