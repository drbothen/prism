Output AC-004-api-backed-udf-rejection-e-rule-012.gif
Output AC-004-api-backed-udf-rejection-e-rule-012.webm
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
Type "# AC-4: plugin-type UDFs are API-backed → rejected in detection rules (BC-2.19.003)"
Enter
Sleep 500ms
Type "cargo nextest run -p prism-spec-engine -E 'test(test_BC_2_19_003_is_api_backed_returns_true_for_plugin_infusion_udfs)' 2>&1 | grep -E 'PASS|FAIL|Summary'"
Enter
Sleep 4s
Type "# E-RULE-012 error message format validated"
Enter
Sleep 300ms
Type "cargo nextest run -p prism-spec-engine -E 'test(test_BC_2_19_003_e_rule_012_error_message_format)' 2>&1 | grep -E 'PASS|FAIL|Summary'"
Enter
Sleep 4s
Type "# Local lookup UDFs are NOT api-backed (safe for filter expressions)"
Enter
Sleep 300ms
Type "cargo nextest run -p prism-spec-engine -E 'test(test_BC_2_19_003_is_api_backed_returns_false_for_local_lookup_udfs)' 2>&1 | grep -E 'PASS|FAIL|Summary'"
Enter
Sleep 4s
Sleep 2s
