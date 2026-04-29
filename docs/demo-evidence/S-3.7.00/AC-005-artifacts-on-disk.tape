Output docs/demo-evidence/S-3.7.00/AC-005-artifacts-on-disk.gif
Output docs/demo-evidence/S-3.7.00/AC-005-artifacts-on-disk.webm

Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 14
Set Width 1200
Set Height 600
Set Theme "Dracula"
Set Padding 20
Set Shell "bash"

Require vhs

Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-3.7.00 && clear"
Enter
Sleep 2s
Show

Type "# AC-005: schema artifacts exist on disk with non-trivial content"
Enter
Sleep 500ms
Type "find .references/schemas -type f | sort"
Enter
Sleep 2s
Type "wc -l .references/schemas/armis/types.rs .references/schemas/armis/DERIVATION.md .references/schemas/crowdstrike/types.rs .references/schemas/crowdstrike/DERIVATION.md"
Enter
Sleep 2s
