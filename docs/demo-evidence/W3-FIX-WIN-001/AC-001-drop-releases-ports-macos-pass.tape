# VHS tape: AC-001 — drop_releases_ports passes on macOS
# Traces to: BC-3.5.001 postcondition 4 / VP-124
# Story: W3-FIX-WIN-001

Output docs/demo-evidence/W3-FIX-WIN-001/AC-001-drop-releases-ports-macos-pass.gif
Output docs/demo-evidence/W3-FIX-WIN-001/AC-001-drop-releases-ports-macos-pass.webm

Set FontFamily "FiraCode Nerd Font Mono"
Set Theme "Dracula"
Set Width 1400
Set Height 600
Set FontSize 13
Set Padding 20
Set Shell "bash"

Env PATH "/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin"

# --- demo: run the targeted test with --nocapture to show pass output ---
Type "bash /Users/jmagady/Dev/prism/.worktrees/W3-FIX-WIN-001/docs/demo-evidence/W3-FIX-WIN-001/run-ac001.sh"
Enter
Sleep 20s
