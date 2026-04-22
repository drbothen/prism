Output docs/demo-evidence/S-1.10/BC-2.09.003-injection-scanner.gif
Output docs/demo-evidence/S-1.10/BC-2.09.003-injection-scanner.webm

Require cargo

Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 20
Set Width 1200
Set Height 600
Set Theme "Catppuccin Mocha"
Set WindowBar Colorful
Set BorderRadius 8
Set Padding 20
Set Framerate 30
Set TypingSpeed 50ms

Hide
Sleep 1s
Show

# BC-2.09.003 — Injection Scanner with NFKC normalization (AC-1)
# 16 tests covering: prompt injection ("ignore previous instructions"), role impersonation
# (SYSTEM:/ASSISTANT:/Human:/Claude:), XML context escape (<system>/<instructions>),
# code fence escape (```), base64-encoded payloads, NFKC homoglyph detection,
# false-positive rejection, truncated-scan flag, case-insensitive matching.

Type "./scripts/demo-bc-2-09-003.sh"
Enter
Sleep 10s

Sleep 2s
