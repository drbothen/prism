# AC-001 + AC-002 + AC-009 + AC-010: TOML blocks parse; Tier-1 columns classified correctly (both tables)
#
# AC-001: SpecLoader::parse on claroty.sensor.toml returns Ok(SensorSpec);
#         claroty_servers table present; 17 ColumnSpec entries.
# AC-002: Exactly 2 columns have ocsf_field == Some(_) for claroty_servers:
#         server_name → device.name (REQUIRED), server_status → status_code.
#         Exactly 15 have ocsf_field == None (Tier-2 → raw_extensions).
# AC-009: SpecLoader::parse returns Ok(SensorSpec);
#         claroty_server_interfaces table present; 10 ColumnSpec entries;
#         path /api/v1/server_interfaces/ (SEPARATE endpoint); response_path $.server_interfaces.
# AC-010: Exactly 2 columns have ocsf_field == Some(_) for claroty_server_interfaces:
#         server_name → device.name (REQUIRED), interface_status → status_code.
#         Exactly 8 have ocsf_field == None (Tier-2 → raw_extensions, incl. interface_name).
#
# Tests: RG-001 (test_BC_2_16_018_claroty_servers_toml_block_parses)
#        RG-002 (test_BC_2_16_018_claroty_servers_tier1_columns_two_with_ocsf_field)
#        RG-009 (test_BC_2_16_019_claroty_server_interfaces_toml_block_parses)
#        RG-010 (test_BC_2_16_019_claroty_server_interfaces_tier1_columns_two_with_ocsf_field)
# Traces to: BC-2.16.018 §Postconditions §1, §2; BC-2.16.019 §Postconditions §1, §2

Output "docs/demo-evidence/S-CLAROTY-SERVERS-001/AC-001-002-009-010-toml-parse.gif"
Output "docs/demo-evidence/S-CLAROTY-SERVERS-001/AC-001-002-009-010-toml-parse.webm"

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
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-CLAROTY-SERVERS-001"
Enter
Sleep 1s
Type "clear"
Enter
Sleep 0.5s
Show

Type "# AC-001/002/009/010: TOML parse + Tier-1 classification for both Claroty G4 tables"
Enter
Sleep 400ms

Type `cargo nextest run -p prism-sensors -E 'test(BC_2_16_018_claroty_servers_toml_block) | test(BC_2_16_018_claroty_servers_tier1) | test(BC_2_16_019_claroty_server_interfaces_toml_block) | test(BC_2_16_019_claroty_server_interfaces_tier1)' 2>&1 | tail -12`
Enter
Sleep 60s
Sleep 2s
