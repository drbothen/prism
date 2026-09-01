# AC-008 + AC-009 + AC-011: Mock-based wire tests — applied_models as JSON array not string;
# REQUIRED policy_id absent produces null row; datetime null passthrough.
# Also covers: RG-012 wire-shape gate (applied_models JSON-typed in serialized wire output).
#
# AC-008 (RG-008 + sub-case): applied_models column_type="json" — value serialized in
#         raw_extensions as a native JSON array (NOT a JSON-stringified string token).
#         Non-empty applied_models: ["Siemens SIMATIC S7", "Rockwell"] → JSON array.
#         Empty applied_models: [] → serialized as JSON empty array, NOT null (EC-016-022-005).
#         RG-012: wire-shape gate confirms applied_models is JSON-typed in serialized wire
#         output via SpecDrivenSensorAdapter::fetch production path.
# AC-009 (RG-009): Row missing policy_id (REQUIRED field, metadata.uid PK) produces a
#         null row in the result set. No hard error raised; subsequent rows continue.
#         Arrow column metadata_uid is null for that row (REQUIRED semantics).
# AC-011 (RG-011): policy_creation_date and policy_last_updated declared column_type="datetime"
#         with no timestamp_formats key (ADR-028 §D8-B implicit iso8601 default).
#         Null/absent datetime field → null Arrow cell, NOT E-SPEC-018 (EC-016-022-010 pattern).
#         ISO-8601 datetime strings parse correctly; non-null values produce non-null cells.
#
# Tests: RG-008 (test_BC_2_16_022_applied_models_raw_extensions_json_array_not_string)
#        sub-case (test_BC_2_16_022_applied_models_empty_array_wire_shape) [EC-016-022-005]
#        RG-009 (test_BC_2_16_022_null_metadata_uid_when_policy_id_absent)
#        RG-011 (test_BC_2_16_022_datetime_fields_null_passthrough_in_raw_extensions)
#        RG-012 (test_BC_2_16_022_claroty_org_acl_policies_wire_shape_applied_models_json_array)
#        SAP-2-NA (test_BC_2_16_022_claroty_acl_policies_wire_shape_sap2_na_documented)
# Traces to: BC-2.16.022 §Postconditions §5 (Json column serialization), §Invariants
#            (policy_id REQUIRED null-row, applied_models must be json not string),
#            EC-016-022-001 (REQUIRED absent→null), EC-016-022-005 (empty array not null),
#            EC-016-022-010 (datetime null passthrough), ADR-028 §D8-B

Output "docs/demo-evidence/S-CLAROTY-ACLPOLICY-001/AC-008-009-011-unit-tests.gif"
Output "docs/demo-evidence/S-CLAROTY-ACLPOLICY-001/AC-008-009-011-unit-tests.webm"

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
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-CLAROTY-ACLPOLICY-001"
Enter
Sleep 1s
Type "clear"
Enter
Sleep 0.5s
Show

Type "# AC-008/009/011: applied_models JSON array + REQUIRED null row + datetime null passthrough (BC-2.16.022)"
Enter
Sleep 400ms

Type "cargo nextest run -p prism-bin -E 'test(BC_2_16_022)' 2>&1 | tail -12"
Enter
Sleep 60s
Sleep 2s
