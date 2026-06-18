Output AC-003-enrich-descriptor-output-columns.gif
Output AC-003-enrich-descriptor-output-columns.webm
Set Shell "bash"
Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 16
Set Width 1200
Set Height 600
Set Theme "Catppuccin Mocha"
Set WindowBar Colorful
Set BorderRadius 8
Set Padding 20
Set Framerate 30
Set TypingSpeed 30ms
Hide
Sleep 2s
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-1.14-REDO && clear"
Enter
Sleep 1s
Show
Type "# AC-3: enrich_descriptor() returns correct EnrichStageDescriptor (BC-2.19.001 v2.0 A1/A2)"
Enter
Sleep 500ms
Type "cargo nextest run -p prism-spec-engine -E 'test(test_BC_2_19_001_enrich_descriptor_no_pipe_stage_returns_all_field_names)' 2>&1 | grep -E 'PASS|FAIL|Summary'"
Enter
Sleep 4s
Type "# Strict subset: pipe_stage adds_columns returns only declared subset"
Enter
Sleep 300ms
Type "cargo nextest run -p prism-spec-engine -E 'test(test_BC_2_19_001_enrich_descriptor_pipe_stage_strict_subset_returns_subset_only)' 2>&1 | grep -E 'PASS|FAIL|Summary'"
Enter
Sleep 4s
Type "# Error: unknown infusion name → E-INFUSE-001"
Enter
Sleep 300ms
Type "cargo nextest run -p prism-spec-engine -E 'test(test_BC_2_19_001_enrich_descriptor_returns_e_infuse_001_for_unknown_name)' 2>&1 | grep -E 'PASS|FAIL|Summary'"
Enter
Sleep 4s
Sleep 2s
