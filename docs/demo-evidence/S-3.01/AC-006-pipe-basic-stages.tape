Output AC-006-pipe-basic-stages.gif
Output AC-006-pipe-basic-stages.webm

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

Type "# AC-6: Pipe mode — where/sort/limit/head/tail/stats/dedup/fields (BC-2.11.004)"
Enter
Sleep 500ms

Type "# SUCCESS PATH: 3-stage pipeline → PipeQuery{Where, Sort, Limit}"
Enter
Sleep 500ms

Type "cargo test -p prism-query --test parser_tests test_AC_06 test_BC_2_11_004_pipe_head_stage_produces_limit_node test_BC_2_11_004_pipe_tail_stage_produces_tail_node test_BC_2_11_004_pipe_stats_count_by_field_produces_stats_node test_BC_2_11_004_pipe_dedup_stage_produces_dedup_node test_BC_2_11_004_pipe_fields_include_stage_produces_fields_node test_BC_2_11_004_pipe_fields_exclude_stage_produces_fields_node -- --nocapture 2>&1 | tail -15"
Enter
Sleep 30s

Type "# ERROR PATH: pipe-only prefix (no source) is valid per EC-11-009"
Enter
Sleep 500ms

Type "cargo test -p prism-query --test parser_tests test_BC_2_11_004_ec_no_source_prefix_starts_with_pipe test_BC_2_11_004_ec_head_zero_is_valid -- --nocapture 2>&1 | tail -10"
Enter
Sleep 30s
