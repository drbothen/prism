Output docs/demo-evidence/S-1.10/BC-2.09.007-output-schema.gif
Output docs/demo-evidence/S-1.10/BC-2.09.007-output-schema.webm

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

# BC-2.09.007 — OutputSchema for Type-Safe LLM Reasoning (AC-8)
# Every sensor tool declares outputSchema with:
#   _meta envelope fields (tool, data_source, query_time, trust_level, safety_flags, pagination)
#   results array item types with field names and descriptions
#   _meta.safety_flags as type:"array" with structured items {field, index, pattern, category}
# Forbidden: per-field {field}_safety_flag keys anywhere in schema.

Type "./scripts/demo-bc-2-09-007.sh"
Enter
Sleep 10s

Sleep 2s
