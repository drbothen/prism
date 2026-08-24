# VHS Demo — AC-003 + AC-004 + AC-007a + AC-012
# AC-003: pipeline_result_to_record_batch uses flattened names when ocsf_column_naming=true
# AC-004: pipeline_result_to_record_batch uses col.name when ocsf_column_naming=false
# AC-007a: columns without ocsf_field go to raw_extensions Arrow schema
# AC-012: pipeline_result_to_record_batch gains sensor_spec: &SensorSpec as explicit parameter
# Tests: RG-005, RG-006, RG-008, RG-024

Output docs/demo-evidence/S-ADR058-OCSF-ROUTING-001/AC-003-004-pipeline-routing-branches.gif
Output docs/demo-evidence/S-ADR058-OCSF-ROUTING-001/AC-003-004-pipeline-routing-branches.webm

Require cargo

Set Shell "zsh"
Set WaitTimeout 60s
Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 18
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

# flag=true: Arrow schema field named finding_info_uid (not id)
# flag=false: Arrow schema field named id (col.name preserved)
# sensor_spec parameter threading (compile-fail without it)
# raw_extensions schema column for ocsf_field==None columns
Type "cargo nextest run -p prism-bin -E 'test(pipeline_result_to_record_batch_ocsf_flag) | test(pipeline_result_to_record_batch_sensor_spec_parameter) | test(spec_driven_adapter_columns_without_ocsf_field)' 2>&1 | tail -10"
Enter
Sleep 15s
Sleep 2s
