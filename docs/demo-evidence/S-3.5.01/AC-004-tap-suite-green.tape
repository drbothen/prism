# AC-004 — TAP test suite GREEN (24/24 crate-layout-gate tests passing)
# Traces to: BC-3.7.001 (VP-134, VP-135, VP-136)
# Story: S-3.5.01
#
# Runs tests/crate-layout-gate/run.sh and shows the 24/24 summary.
#
# VHS tape — produces AC-004-tap-suite-green.{gif,webm}

Output docs/demo-evidence/S-3.5.01/AC-004-tap-suite-green.gif
Output docs/demo-evidence/S-3.5.01/AC-004-tap-suite-green.webm

Require bash

Set Shell "bash"
Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 14
Set Width 1200
Set Height 700
Set Theme "Dracula"
Set Padding 20
Set TypingSpeed 60ms

# Setup — hidden
Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-3.5.01"
Enter
Sleep 1s
Show

# Demo — run TAP gate and tail the summary (last 15 lines)
Type "bash tests/crate-layout-gate/run.sh 2>&1 | tail -15"
Enter
Sleep 20s

Sleep 2s
