# VHS tape: AC-002 — full prism-dtu-harness suite 70/70 regression safety
# Traces to: BC-3.5.001 postcondition 4 / VP-124
# Story: W3-FIX-WIN-001

Output docs/demo-evidence/W3-FIX-WIN-001/AC-002-harness-suite-regression-safe.gif
Output docs/demo-evidence/W3-FIX-WIN-001/AC-002-harness-suite-regression-safe.webm

Set FontFamily "FiraCode Nerd Font Mono"
Set Theme "Dracula"
Set Width 1400
Set Height 600
Set FontSize 13
Set Padding 20
Set Shell "bash"

Env PATH "/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin"

# --- demo: run the full harness suite showing 70/70 pass ---
Type "bash /Users/jmagady/Dev/prism/.worktrees/W3-FIX-WIN-001/docs/demo-evidence/W3-FIX-WIN-001/run-ac002.sh"
Enter
Sleep 60s
