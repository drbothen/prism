Output AC-009-error-recovery.gif
Output AC-009-error-recovery.webm

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

Type "# AC-9: Error recovery — structured ParseError with offset + message + JSON (BC-2.11.002)"
Enter
Sleep 500ms

Type "# SUCCESS PATH: malformed input returns Vec<ParseError> with offset, message, JSON"
Enter
Sleep 500ms

Type "cargo test -p prism-query --test parser_tests test_AC_09 -- --nocapture 2>&1 | tail -15"
Enter
Sleep 30s
