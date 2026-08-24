# VHS Demo — AC-007c + AC-014 (special behaviors)
# AC-007c: Multi-valued array source fields in raw_extensions serialized as compact JSON-list strings
# AC-014: extract_time_window_from_ast recognizes ocsf_field_to_arrow_name result as index-eligible
# Tests: RG-018 (compact JSON-list), RG-Q-017 (pushdown)

Output docs/demo-evidence/S-ADR058-OCSF-ROUTING-001/AC-007c-014-special-behaviors.gif
Output docs/demo-evidence/S-ADR058-OCSF-ROUTING-001/AC-007c-014-special-behaviors.webm

Require cargo

Set Shell "zsh"
Set WaitTimeout 60s
Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 17
Set Width 1200
Set Height 600
Set Theme "Catppuccin Mocha"
Set WindowBar Colorful
Set BorderRadius 8
Set Padding 20
Set Framerate 30
Set TypingSpeed 40ms

Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-ADR058-OCSF-ROUTING-001"
Enter
Sleep 1.5s
Type "clear"
Enter
Sleep 0.5s
Show

# AC-007c: ip_list -> compact JSON-list string (not nested array) in raw_extensions
# AC-014: OCSF-flattened time column recognized as INDEX-eligible by extract_time_window_from_ast
Type "cargo nextest run -p prism-bin -p prism-query -E 'test(ip_list_in_raw_extensions) | test(ocsf_flattened_time_column_as_index)' 2>&1 | tail -10"
Enter
Sleep 20s
Sleep 2s
