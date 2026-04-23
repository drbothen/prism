Output docs/demo-evidence/S-1.13/AC-005-ten-write-verbs-partial.gif
Output docs/demo-evidence/S-1.13/AC-005-ten-write-verbs-partial.webm

Require cargo

Set FontFamily "FiraCode Nerd Font Mono"
Set Theme "Dracula"
Set Width 1200
Set Height 700
Set FontSize 14
Set Padding 24
Set Shell zsh

Hide
Type "cd /Users/jmagady/dev/prism/.worktrees/S-1.13-sensor-write-specs"
Enter
Sleep 2s
Show

Type "# AC-5 (passing): verbs_for_sensor(crowdstrike) returns 4 verbs"
Enter
Sleep 500ms
Type "cargo test -p prism-spec-engine --test write_endpoint_tests verbs_for_crowdstrike 2>&1 | tail -6"
Enter
Sleep 8s

Type ""
Enter
Type "# AC-5 (blocked): all_four_sensors FAILS -- claroty+armis both use pipe_verb=tag (EC-002)"
Enter
Sleep 500ms
Type "cargo test -p prism-spec-engine --test write_endpoint_tests all_four_sensors_ten_write_verbs 2>&1 | grep 'already registered' | head -2"
Enter
Sleep 8s
Sleep 2s
