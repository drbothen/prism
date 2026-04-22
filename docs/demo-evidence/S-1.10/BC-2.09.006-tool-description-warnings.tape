Output docs/demo-evidence/S-1.10/BC-2.09.006-tool-description-warnings.gif
Output docs/demo-evidence/S-1.10/BC-2.09.006-tool-description-warnings.webm

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

# BC-2.09.006 — Tool Description Security Warnings (AC-7)
# All sensor tools must include 9 required sections:
#   DATA SOURCE, DATA TRUST LEVEL, WHEN TO USE, WHEN NOT TO USE,
#   PARAMETERS, PAGINATION, RESPONSE, ERRORS, SECURITY NOTE
# SECURITY NOTE must mention: hostnames, file paths, process names, description fields.
# ToolDescriptionRegistrar appends missing sections idempotently (framework layer).

Type "./scripts/demo-bc-2-09-006.sh"
Enter
Sleep 10s

Sleep 2s
