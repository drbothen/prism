# AC-001 + AC-002: TOML block parses without validation error; 4 Tier-1 columns
#
# AC-001: SpecLoader::parse on claroty.sensor.toml returns Ok(SensorSpec);
#         claroty_ot_activity_events table present; 21 ColumnSpec entries.
# AC-002: Exactly 4 columns have ocsf_field == Some(_): event_id→finding_info.uid,
#         detection_time→time, event_type→activity_name, description→message.
#
# Tests: RG-001 (test_BC_2_16_016_claroty_ot_activity_events_toml_block_parses)
#        RG-002 (test_BC_2_16_016_claroty_ot_activity_events_four_tier1_columns)
# Traces to: BC-2.16.016 §Postconditions §1, §2

Output "docs/demo-evidence/S-CLAROTY-OT-EVENTS-001/AC-001-002-toml-parse.gif"
Output "docs/demo-evidence/S-CLAROTY-OT-EVENTS-001/AC-001-002-toml-parse.webm"

Require cargo

Set Shell "zsh"
Set WaitTimeout 120s
Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 16
Set Width 1200
Set Height 500
Set Theme "Catppuccin Mocha"
Set WindowBar Colorful
Set BorderRadius 8
Set Padding 20
Set Framerate 30
Set TypingSpeed 40ms

Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-CLAROTY-OT-EVENTS-001"
Enter
Sleep 1s
Type "clear"
Enter
Sleep 0.5s
Show

Type "# AC-001/002: TOML parse + 4 Tier-1 column classification (RG-001, RG-002)"
Enter
Sleep 400ms

Type `cargo nextest run -p prism-spec-engine -E 'test(BC_2_16_016)' 2>&1 | tail -8`
Enter
Sleep 60s
Sleep 2s
