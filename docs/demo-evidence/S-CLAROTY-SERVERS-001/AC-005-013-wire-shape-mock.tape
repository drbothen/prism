# AC-005 + AC-013: Wire shape mock tests — class_uid=5001, Tier-1 fields, raw_extensions (both tables)
#
# AC-005: SELECT * FROM claroty_servers LIMIT 1 serialized JSON (via mock) contains:
#         class_uid=5001, device_name present, status_code present, raw_extensions present,
#         no standalone Tier-2 root keys.
# AC-013: SELECT * FROM claroty_server_interfaces LIMIT 1 serialized JSON (via mock) contains:
#         class_uid=5001, device_name present, status_code present, raw_extensions present with
#         interface_name and interface_type keys (composite PK join keys accessible via raw_extensions),
#         no standalone Tier-2 root keys. _source_table = "claroty_server_interfaces" (separate endpoint).
#
# Tests: RG-017 (test_BC_2_16_018_claroty_servers_wire_shape_class_uid_5001_mock — prism-bin SAP-4 path)
#        RG-020 (test_BC_2_16_019_claroty_server_interfaces_wire_shape_class_uid_5001_mock — prism-bin SAP-4 path)
# Traces to: BC-2.16.018 §Postconditions §1, §2; BC-2.16.019 §Postconditions §1, §2, §3

Output "docs/demo-evidence/S-CLAROTY-SERVERS-001/AC-005-013-wire-shape-mock.gif"
Output "docs/demo-evidence/S-CLAROTY-SERVERS-001/AC-005-013-wire-shape-mock.webm"

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

Type "# AC-005/013: class_uid=5001 wire shape mock (RG-017, RG-020 — SAP-4 production path)"
Enter
Sleep 400ms

Type `cargo nextest run -p prism-bin --test bc_2_16_018_claroty_servers_wire_shape --test bc_2_16_019_claroty_server_interfaces_wire_shape -E 'test(wire_shape_class_uid)' 2>&1 | tail -8`
Enter
Sleep 60s
Sleep 2s
