# AC-002 — Violation detection with synthetic non-conformant crate
# Traces to: BC-3.7.001 postcondition 2 + 3, VP-135
# Story: S-3.5.01
#
# Runs ac-002-run.sh which creates a temp dir with a bad layout (lib.rs at
# crate root, no src/), runs check-crate-layout.sh, shows the violation output
# and non-zero exit, then cleans up automatically via trap.
#
# VHS tape — produces AC-002-violation-detection.{gif,webm}

Output docs/demo-evidence/S-3.5.01/AC-002-violation-detection.gif
Output docs/demo-evidence/S-3.5.01/AC-002-violation-detection.webm

Require bash

Set Shell "bash"
Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 14
Set Width 1200
Set Height 600
Set Theme "Dracula"
Set Padding 20
Set TypingSpeed 60ms

# Setup — hidden
Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-3.5.01"
Enter
Sleep 1s
Show

# Demo — run the violation detection helper script
Type "bash docs/demo-evidence/S-3.5.01/ac-002-run.sh"
Enter
Sleep 5s

Sleep 2s
