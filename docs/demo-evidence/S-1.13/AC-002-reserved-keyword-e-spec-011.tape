Output docs/demo-evidence/S-1.13/AC-002-reserved-keyword-e-spec-011.gif
Output docs/demo-evidence/S-1.13/AC-002-reserved-keyword-e-spec-011.webm

Require cargo

Set FontFamily "FiraCode Nerd Font Mono"
Set Theme "Dracula"
Set Width 1200
Set Height 600
Set FontSize 14
Set Padding 24
Set Shell zsh

Hide
Type "cd /Users/jmagady/dev/prism/.worktrees/S-1.13-sensor-write-specs"
Enter
Sleep 2s
Show

Type "# AC-2: pipe_verb=where (reserved keyword) -> validation returns E-SPEC-011"
Enter
Sleep 500ms
Type "cargo test -p prism-spec-engine --test write_endpoint_tests reserved_keyword 2>&1 | tail -10"
Enter
Sleep 8s
Sleep 2s
