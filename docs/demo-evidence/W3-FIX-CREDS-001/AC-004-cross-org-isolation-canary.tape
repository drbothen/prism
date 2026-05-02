# AC-004: Cross-org isolation canary — Org A credential not retrievable by Org B
# BC-3.2.002 postcondition 2 / VP-3.2.002-01 — full suite all 7 ACs
#
# Story: W3-FIX-CREDS-001 | AC: AC-001..006

Output docs/demo-evidence/W3-FIX-CREDS-001/AC-004-cross-org-isolation-canary.gif
Output docs/demo-evidence/W3-FIX-CREDS-001/AC-004-cross-org-isolation-canary.webm

Set Shell "bash"
Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 14
Set Width 1200
Set Height 600
Set Theme "Dracula"
Set Padding 20
Set PlaybackSpeed 1.0
Set TypingSpeed 40ms
Set WaitTimeout 120s

Hide
Type "source /Users/jmagady/.bash_profile 2>/dev/null" Enter
Sleep 1s
Type "cd /Users/jmagady/Dev/prism/.worktrees/W3-FIX-CREDS-001" Enter
Sleep 1s
Type "clear" Enter
Sleep 1s
Show

Type "# AC-004: Cross-org isolation — Org A credential NOT visible to Org B" Enter
Sleep 500ms
Type "# Full bc_3_2_002_trait_impl suite: 7/7 tests, BC-3.2.002 regression coverage" Enter
Sleep 500ms
Type "/opt/homebrew/bin/cargo nextest run -p prism-credentials --test bc_3_2_002_trait_impl 2>&1" Enter
Wait+Screen@120s /7 passed/
Sleep 2s
