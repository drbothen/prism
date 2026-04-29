Output docs/demo-evidence/S-3.7.03/AC-003-schema-drift-behavior.gif
Output docs/demo-evidence/S-3.7.03/AC-003-schema-drift-behavior.webm

Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 13
Set Width 1400
Set Height 600
Set Theme "Dracula"
Set Padding 20
Set Shell "bash"

Require vhs

Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-3.7.03 && clear"
Enter
Sleep 2s
Show

Type "# AC-003 — SchemaDrift: alert[0] invalid, other surfaces valid"
Enter
Sleep 300ms
Type "cargo test -p prism-dtu-cyberint --features fixture-gen --test bc_3_4_cyberint_generator schema_drift -- --nocapture 2>&1 | grep -E '(test |SCHEMA|_schema_valid|ok$|running)'"
Enter
Sleep 8s
