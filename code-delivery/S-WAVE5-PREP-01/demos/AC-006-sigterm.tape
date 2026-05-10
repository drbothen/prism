Output AC-006-sigterm.gif
Output AC-006-sigterm.webm

Set Width 1200
Set Height 550
Set FontSize 13
Set FontFamily "FiraCode Nerd Font Mono"
Set Theme "Dracula"
Set Padding 20
Set Shell "bash"

Type "export PATH=/Users/jmagady/Dev/prism/.worktrees/S-WAVE5-PREP-01/target/debug:$PATH && cd /Users/jmagady/Dev/prism/.worktrees/S-WAVE5-PREP-01"
Enter
Sleep 500ms

Type "# AC-6: SIGTERM during boot -> graceful exit 0 with shutdown log"
Enter
Sleep 300ms

Type "rm -f /tmp/prism-ac6-demo.ready /tmp/prism-ac6-demo.out"
Enter
Sleep 300ms

Type "PRISM_TEST_STOP_AFTER_STEP=6 PRISM_TEST_READY_FILE=/tmp/prism-ac6-demo.ready prism start --config-dir crates/prism-bin/fixtures/config/valid >/tmp/prism-ac6-demo.out 2>&1 & until [ -f /tmp/prism-ac6-demo.ready ]; do sleep 0.2; done && kill -TERM $! && wait $!; echo exit=$?"
Enter
Sleep 4000ms

Type "grep -E 'SIGTERM|shutting down' /tmp/prism-ac6-demo.out | grep -v 'at crates'"
Enter
Sleep 1500ms

Sleep 2s
