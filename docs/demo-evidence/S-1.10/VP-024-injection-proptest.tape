Output docs/demo-evidence/S-1.10/VP-024-injection-proptest.gif
Output docs/demo-evidence/S-1.10/VP-024-injection-proptest.webm

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

# VP-024 — Proptest: InjectionScanner detects all known injection patterns (AC-5)
# 4 property-based tests using proptest:
#   detects known patterns embedded in random noise strings
#   NFKC homoglyph variants of catalogue patterns are detected
#   original value is always preserved (flag-don't-strip invariant)
#   every flag has a non-empty pattern description

Type "./scripts/demo-vp-024.sh"
Enter
Sleep 12s

Sleep 2s
