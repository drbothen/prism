Output AC-005-json-log.gif
Output AC-005-json-log.webm

Set Width 1400
Set Height 350
Set FontSize 13
Set FontFamily "FiraCode Nerd Font Mono"
Set Theme "Dracula"
Set Padding 20
Set Shell "bash"

Type "export PATH=/Users/jmagady/Dev/prism/.worktrees/S-WAVE5-PREP-01/target/debug:$PATH && cd /Users/jmagady/Dev/prism/.worktrees/S-WAVE5-PREP-01"
Enter
Sleep 500ms

Type "# AC-5: first log line is JSON with level:INFO and message:Prism vX.Y.Z"
Enter
Sleep 300ms

Type "PRISM_LOG_FORMAT=json prism validate-config --config-dir crates/prism-bin/fixtures/config/valid 2>&1 | head -1"
Enter
Sleep 1500ms

Sleep 2s
