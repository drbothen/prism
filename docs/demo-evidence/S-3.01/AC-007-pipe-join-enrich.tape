Output AC-007-pipe-join-enrich.gif
Output AC-007-pipe-join-enrich.webm

Require cargo

Set Shell "bash"
Set FontSize 14
Set Width 1200
Set Height 600
Set Theme "Catppuccin Mocha"
Set Padding 20
Set FontFamily "FiraCode Nerd Font Mono"

Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-3.01"
Enter
Sleep 1s
Type "clear"
Enter
Sleep 500ms
Show

Type "# AC-7: Pipe mode — join + enrich stages (BC-2.11.004)"
Enter
Sleep 500ms

Type "# SUCCESS PATH: join + enrich produce typed JoinStage/EnrichStage AST nodes"
Enter
Sleep 500ms

Type "cargo test -p prism-query --test parser_tests test_AC_07 -- --nocapture 2>&1 | tail -10"
Enter
Sleep 30s
