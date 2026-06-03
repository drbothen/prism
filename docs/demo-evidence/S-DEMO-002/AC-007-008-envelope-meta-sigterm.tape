# S-DEMO-002 / AC-007 + AC-008
# AC-007: ResponseEnvelope _meta — trust_level=="untrusted_external", safety_flags==[], data_source contains sensor.
# AC-008: SIGTERM — both prism-bin + DTU demo server exit cleanly within 5s (exit code 0).
#
# BC: BC-2.09.008 (trust annotations), BC-2.10.010 (graceful shutdown).

Output "docs/demo-evidence/S-DEMO-002/AC-007-008-envelope-meta-sigterm.gif"
Output "docs/demo-evidence/S-DEMO-002/AC-007-008-envelope-meta-sigterm.webm"

Require cargo

Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 18
Set Width 1200
Set Height 600
Set Theme "Catppuccin Mocha"
Set WindowBar Colorful
Set BorderRadius 8
Set Padding 20
Set Framerate 30
Set TypingSpeed 40ms

Sleep 2s

Hide
Type "export PATH=/Users/jmagady/.cargo/bin:$PATH"
Enter
Sleep 500ms
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-DEMO-002"
Enter
Sleep 500ms
Show

# AC-007: _meta.trust_level / safety_flags / data_source assertions
Type "# AC-007: trust_level=untrusted_external, safety_flags=[], data_source contains sensor"
Enter
Sleep 500ms
Type "cargo nextest run -p prism-bin --profile e2e --run-ignored all -E 'test(e2e_response_envelope_meta)' 2>&1 | grep -E 'PASS|FAIL|Summary'"
Enter
Sleep 20s

Type "clear"
Enter
Sleep 500ms

# AC-008: SIGTERM — clean shutdown within 5s
Type "# AC-008: SIGTERM — prism-bin + DTU exit 0 within 5s (BC-2.10.010)"
Enter
Sleep 500ms
Type "cargo nextest run -p prism-bin --profile e2e --run-ignored all -E 'test(e2e_sigterm_cleanly)' 2>&1 | grep -E 'PASS|FAIL|Summary'"
Enter
Sleep 20s

Sleep 2s
