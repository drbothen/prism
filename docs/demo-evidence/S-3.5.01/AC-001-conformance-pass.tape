# AC-001 — Conformance check on real workspace (all 22 crates passing)
# Traces to: BC-3.7.001 postcondition 1, VP-134
# Story: S-3.5.01
#
# VHS tape — produces AC-001-conformance-pass.{gif,webm}

Output docs/demo-evidence/S-3.5.01/AC-001-conformance-pass.gif
Output docs/demo-evidence/S-3.5.01/AC-001-conformance-pass.webm

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

# Demo — run the conformance check; expect exit 0, no violation output
Type "bash scripts/check-crate-layout.sh && echo 'All crates conform — exit 0'"
Enter
Sleep 3s

Sleep 2s
