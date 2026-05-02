# AC-002: SEC-P3-002 / CR-019 — find_snippet_pipe anchored to numeric prefix (CWE-209)
#
# Traces to: BC-3.3.004 postcondition 2
# Story: W3-FIX-SEC-004
#
# Demonstrates that a credential value containing " | " does NOT fool
# find_snippet_pipe into a false pipe match. Two test cases:
#   - credential with embedded " | " → pipe found at digit-prefix position, line redacted
#   - non-snippet line with " | " in value → find_snippet_pipe returns None

Output docs/demo-evidence/W3-FIX-SEC-004/AC-002-find-snippet-pipe-robustness.gif
Output docs/demo-evidence/W3-FIX-SEC-004/AC-002-find-snippet-pipe-robustness.webm

Require cargo

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
Type "cd /Users/jmagady/Dev/prism/.worktrees/W3-FIX-SEC-004"
Enter
Sleep 1s
Show

# Run both pipe-anchor test cases
Type "# AC-002 SEC-P3-002: pipe-anchor robustness — credential value with | not confused"
Enter
Sleep 500ms
Type "cargo test -p prism-customer-config --test sec_p3_002_pipe_anchor 2>&1 | tail -4"
Enter
Sleep 15s
