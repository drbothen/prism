Output AC-006-credential-redaction.gif
Output AC-006-credential-redaction.webm
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
Type "# AC-6: Credential values NEVER appear in errors or logs (BC-2.19.005 / AD-017)"
Enter
Sleep 500ms
Type "cargo nextest run -p prism-spec-engine -E 'test(test_BC_2_19_005_e_infuse_005_error_message_contains_field_name_not_value)' 2>&1 | grep -E 'PASS|FAIL|Summary'"
Enter
Sleep 4s
Type "# Debug output redacts credential values completely"
Enter
Sleep 300ms
Type "cargo nextest run -p prism-spec-engine -E 'test(test_BC_2_19_005_credential_ref_debug_output_redacts_value)' 2>&1 | grep -E 'PASS|FAIL|Summary'"
Enter
Sleep 4s
Type "# Error: empty env_var credential reference rejected at structural validation"
Enter
Sleep 300ms
Type "cargo nextest run -p prism-spec-engine -E 'test(test_BC_2_19_001_parse_rejects_spec_with_empty_env_var_credential)' 2>&1 | grep -E 'PASS|FAIL|Summary'"
Enter
Sleep 4s
Sleep 2s
