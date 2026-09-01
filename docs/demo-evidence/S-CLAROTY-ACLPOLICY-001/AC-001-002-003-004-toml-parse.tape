# AC-001 + AC-002 + AC-003 + AC-004: TOML block parses without validation error;
# pagination type=none confirmed; body_template has policy_acl_syntax="Cisco dACL"
# and filter_by={policy_id, is_not_null}; 4 Tier-1 + 7 Tier-2 column classification.
#
# AC-001 (RG-001): SpecLoader::parse on claroty.sensor.toml returns Ok(SensorSpec);
#         claroty_organization_acl_policies table present; 11 ColumnSpec entries;
#         PaginationConfig::None; response_path $.organization_acl_policies.
# AC-002 (RG-002): pipeline.rs build_request for PaginationConfig::None does NOT inject
#         offset/limit into POST body; body built from body_template only.
# AC-003 (RG-003 + RG-013): body_template JSON contains "policy_acl_syntax": "Cisco dACL"
#         (REQUIRED per OpenAPI schema) AND "filter_by": {"field":"policy_id",
#         "operation":"is_not_null"} (live API cross-field selector validator, EC-016-022-011).
# AC-004 (RG-004): Exactly 4 Tier-1 columns: policy_id→metadata.uid (REQUIRED),
#         policy_name→name, policy_updated_by→actor.user.name, policy_notes→comment;
#         Exactly 7 Tier-2 columns with None ocsf_field (aggregate into raw_extensions).
#
# Tests: RG-001 (test_BC_2_16_022_claroty_org_acl_policies_toml_block_parses)
#        RG-002 (test_BC_2_16_022_claroty_org_acl_policies_pagination_none_no_offset_limit)
#        RG-003 (test_BC_2_16_022_claroty_org_acl_policies_body_template_has_policy_acl_syntax)
#        RG-004 (test_BC_2_16_022_claroty_org_acl_policies_tier1_four_tier2_seven_correct_types)
#        RG-013 (test_BC_2_16_022_claroty_org_acl_policies_body_template_has_filter_by_policy_id_is_not_null)
# Traces to: BC-2.16.022 §Postconditions §1 (TOML contract), §2 (Tier classification),
#            §4 (Pagination-None), §Invariants (body_template mandatory fields)

Output "docs/demo-evidence/S-CLAROTY-ACLPOLICY-001/AC-001-002-003-004-toml-parse.gif"
Output "docs/demo-evidence/S-CLAROTY-ACLPOLICY-001/AC-001-002-003-004-toml-parse.webm"

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

Type "# AC-001/002/003/004: TOML parse + pagination-none + body_template + Tier-1/Tier-2 (BC-2.16.022)"
Enter
Sleep 400ms

Type "cargo nextest run -p prism-spec-engine -E 'test(BC_2_16_022)' 2>&1 | tail -12"
Enter
Sleep 60s
Sleep 2s
