# AC-001 + AC-002: TOML block parses without validation error; 13 columns declared;
#                  2 Tier-1 columns with correct ocsf_field
#
# AC-001: SpecLoader::parse on claroty.sensor.toml returns Ok(SensorSpec);
#         claroty_device_vulnerability_relations table present; 13 ColumnSpec entries;
#         pagination offset_limit page_size=1000.
# AC-002: Exactly 2 columns have ocsf_field == Some(_):
#         vulnerability_name → finding_info.title (with REQUIRED);
#         device_vulnerability_detection_date → time.
#         Exactly 11 columns have ocsf_field == None (Tier-2 into raw_extensions).
#
# Tests: RG-001 (test_BC_2_16_017_...toml_block_parses)
#        RG-002 (test_BC_2_16_017_...tier1_columns_two_with_ocsf_field)
# Traces to: BC-2.16.017 §Postconditions §1, §2

Output "docs/demo-evidence/S-CLAROTY-DEVVULNREL-001/AC-001-002-toml-parse.gif"
Output "docs/demo-evidence/S-CLAROTY-DEVVULNREL-001/AC-001-002-toml-parse.webm"

Require cargo

Set Shell "zsh"
Set WaitTimeout 180s
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
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-CLAROTY-DEVVULNREL-001"
Enter
Sleep 1s
Type "clear"
Enter
Sleep 0.5s
Show

Type "# AC-001/002: TOML parse + 2 Tier-1 column classification (RG-001, RG-002)"
Enter
Sleep 400ms

Type `cargo nextest run -p prism-sensors -E 'test(BC_2_16_017)' --no-fail-fast 2>&1 | tail -15`
Enter
Sleep 120s
Sleep 2s
