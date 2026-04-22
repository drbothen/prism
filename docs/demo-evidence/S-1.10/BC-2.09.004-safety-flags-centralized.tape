Output docs/demo-evidence/S-1.10/BC-2.09.004-safety-flags-centralized.gif
Output docs/demo-evidence/S-1.10/BC-2.09.004-safety-flags-centralized.webm

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

# BC-2.09.004 — Centralized Safety Flags (AC-4)
# Flags are in _meta.safety_flags array — NOT in per-field {field}_safety_flag keys.
# Flag-don't-strip: original sensor data is always preserved unchanged.
# Multiple patterns in one field => multiple flags, all appended to the array.

Type "./scripts/demo-bc-2-09-004.sh"
Enter
Sleep 10s

Sleep 2s
