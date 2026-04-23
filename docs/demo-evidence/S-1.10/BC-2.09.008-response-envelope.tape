Output docs/demo-evidence/S-1.10/BC-2.09.008-response-envelope.gif
Output docs/demo-evidence/S-1.10/BC-2.09.008-response-envelope.webm

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

# BC-2.09.008 — Response Envelope with Trust Annotations (AC-2, AC-4)
# Every response wrapped in: { _meta: { trust_level, safety_flags, data_source, query_time,
#   total_results, page, has_more, next_cursor }, results, content, structuredContent }
# _meta.safety_flags is always present (never absent, even as empty array).
# Zero-result responses still have the full _meta envelope.

Type "./scripts/demo-bc-2-09-008.sh"
Enter
Sleep 10s

Sleep 2s
