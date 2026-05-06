Output AC-008-security-limits.gif
Output AC-008-security-limits.webm

Require cargo

Set Shell "bash"
Set FontSize 14
Set Width 1200
Set Height 600
Set Theme "Catppuccin Mocha"
Set Padding 20
Set FontFamily "FiraCode Nerd Font Mono"

Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-3.01"
Enter
Sleep 1s
Type "clear"
Enter
Sleep 500ms
Show

Type "# AC-8: Security limits — 64KB cap, depth-64 cap, 32-stage cap, 1024-byte regex cap"
Enter
Sleep 500ms

Type "# SUCCESS PATH: limits at boundary values accepted (64KB, depth-64, 32 stages, 1024-byte regex)"
Enter
Sleep 500ms

Type "cargo test -p prism-query --test parser_tests test_BC_2_11_006_ec_exactly_64kb_is_valid test_VP_015_check_nesting_depth_accepts_depth_64 test_BC_2_11_006_pipe_stage_limit_32_stages_valid test_BC_2_11_006_regex_pattern_exactly_1024_bytes_valid -- --nocapture 2>&1 | tail -10"
Enter
Sleep 30s

Type "# ERROR PATH: over-limit inputs rejected with E-QUERY-003"
Enter
Sleep 500ms

Type "cargo test -p prism-query --test parser_tests test_AC_08_oversized_query_returns_e_query_003 test_VP_014_check_query_size_rejects_65537_bytes test_VP_015_check_nesting_depth_rejects_depth_65 test_BC_2_11_006_pipe_stage_limit_33_stages_rejected test_BC_2_11_006_canonical_tv_33_pipe_stages_error test_BC_2_11_006_regex_pattern_exceeds_1024_bytes_rejected test_BC_2_11_006_canonical_tv_65_levels_nesting_error -- --nocapture 2>&1 | tail -15"
Enter
Sleep 30s
